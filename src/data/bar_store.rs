/// Hybrid Bar Storage - Ring Buffer (memory) + JSONL (disk)
/// Optimized for O(1) append and fast recent reads
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error};

use crate::types::Bar;
use crate::error::{Result, TradingError};

/// Hybrid bar storage combining in-memory ring buffer and disk persistence
pub struct HybridBarStore {
    /// Hot path: in-memory ring buffer (last N bars)
    memory_buffer: VecDeque<Bar>,
    memory_capacity: usize,
    
    /// Cold path: disk storage (JSONL format)
    disk_file: PathBuf,
    
    /// Metadata
    total_bars: usize,
    symbol: String,
    timeframe: String,
}

impl HybridBarStore {
    pub fn new(symbol: String, timeframe: String, disk_file: PathBuf, memory_capacity: usize) -> Self {
        HybridBarStore {
            memory_buffer: VecDeque::with_capacity(memory_capacity),
            memory_capacity,
            disk_file,
            total_bars: 0,
            symbol,
            timeframe,
        }
    }
    
    /// Append a new bar (O(1) operation)
    pub async fn append(&mut self, bar: Bar) -> Result<()> {
        // Write to disk immediately for durability
        self.append_to_disk(&bar).await?;
        
        // Add to memory buffer
        if self.memory_buffer.len() >= self.memory_capacity {
            self.memory_buffer.pop_front();
        }
        self.memory_buffer.push_back(bar);
        
        self.total_bars += 1;
        
        debug!(
            "Appended bar for {} {} - total: {}, in-memory: {}",
            self.symbol,
            self.timeframe,
            self.total_bars,
            self.memory_buffer.len()
        );
        
        Ok(())
    }
    
    /// Get recent N bars (O(1) if all in memory)
    pub async fn get_recent(&self, n: usize) -> Result<Vec<Bar>> {
        // Fast path: all in memory
        if n <= self.memory_buffer.len() {
            return Ok(self.memory_buffer
                .iter()
                .rev()
                .take(n)
                .rev()
                .cloned()
                .collect());
        }
        
        // Slow path: need to read from disk
        self.load_from_disk_and_memory(n).await
    }
    
    /// Get the last bar (most recent)
    pub fn get_last(&self) -> Option<&Bar> {
        self.memory_buffer.back()
    }
    
    /// Get all bars in memory
    pub fn get_all_in_memory(&self) -> Vec<Bar> {
        self.memory_buffer.iter().cloned().collect()
    }
    
    /// Total number of bars stored
    pub fn total_count(&self) -> usize {
        self.total_bars
    }
    
    /// Number of bars in memory
    pub fn memory_count(&self) -> usize {
        self.memory_buffer.len()
    }
    
    /// Append bar to disk (JSONL format)
    async fn append_to_disk(&self, bar: &Bar) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.disk_file)
            .await?;
        
        let json_line = serde_json::to_string(bar)?;
        file.write_all(format!("{}\n", json_line).as_bytes()).await?;
        file.sync_all().await?;
        
        Ok(())
    }
    
    /// Load bars from disk and combine with memory
    async fn load_from_disk_and_memory(&self, n: usize) -> Result<Vec<Bar>> {
        let file = tokio::fs::File::open(&self.disk_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        // Read all lines from disk
        let mut disk_bars = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Ok(bar) = serde_json::from_str::<Bar>(&line) {
                disk_bars.push(bar);
            }
        }
        
        // Combine disk + memory, take last N
        disk_bars.extend(self.memory_buffer.iter().cloned());
        
        let result = disk_bars
            .into_iter()
            .rev()
            .take(n)
            .rev()
            .collect();
        
        Ok(result)
    }
    
    /// Load existing data from disk into memory (on startup)
    pub async fn load_from_disk(&mut self, load_last_n: usize) -> Result<()> {
        if !self.disk_file.exists() {
            debug!("No existing disk file for {} {}", self.symbol, self.timeframe);
            return Ok(());
        }
        
        let file = tokio::fs::File::open(&self.disk_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        
        let mut all_bars = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Ok(bar) = serde_json::from_str::<Bar>(&line) {
                all_bars.push(bar);
            }
        }
        
        self.total_bars = all_bars.len();
        
        // Load last N into memory
        let bars_to_load: Vec<Bar> = all_bars
            .into_iter()
            .rev()
            .take(load_last_n)
            .rev()
            .collect();
        
        for bar in bars_to_load {
            if self.memory_buffer.len() >= self.memory_capacity {
                self.memory_buffer.pop_front();
            }
            self.memory_buffer.push_back(bar);
        }
        
        debug!(
            "Loaded {} {} from disk: {} total bars, {} in memory",
            self.symbol,
            self.timeframe,
            self.total_bars,
            self.memory_buffer.len()
        );
        
        Ok(())
    }
    
    /// Rotate disk file (e.g., daily archival)
    pub async fn rotate_disk_file(&mut self, new_file: PathBuf) -> Result<()> {
        // Archive old file
        if self.disk_file.exists() {
            let archive_name = format!(
                "{}.{}.archive",
                self.disk_file.display(),
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            );
            tokio::fs::rename(&self.disk_file, archive_name).await?;
        }
        
        // Set new file
        self.disk_file = new_file;
        self.total_bars = self.memory_buffer.len();
        
        // Rewrite memory buffer to new file
        for bar in &self.memory_buffer {
            self.append_to_disk(bar).await?;
        }
        
        debug!("Rotated disk file for {} {}", self.symbol, self.timeframe);
        Ok(())
    }
}

/// Thread-safe wrapper for HybridBarStore
pub struct ConcurrentBarStore {
    store: Arc<RwLock<HybridBarStore>>,
}

impl ConcurrentBarStore {
    pub fn new(symbol: String, timeframe: String, disk_file: PathBuf, memory_capacity: usize) -> Self {
        ConcurrentBarStore {
            store: Arc::new(RwLock::new(HybridBarStore::new(
                symbol,
                timeframe,
                disk_file,
                memory_capacity,
            ))),
        }
    }
    
    pub async fn append(&self, bar: Bar) -> Result<()> {
        let mut store = self.store.write().await;
        store.append(bar).await
    }
    
    pub async fn get_recent(&self, n: usize) -> Result<Vec<Bar>> {
        let store = self.store.read().await;
        store.get_recent(n).await
    }
    
    pub async fn get_last(&self) -> Option<Bar> {
        let store = self.store.read().await;
        store.get_last().cloned()
    }
    
    pub async fn get_all_in_memory(&self) -> Vec<Bar> {
        let store = self.store.read().await;
        store.get_all_in_memory()
    }
    
    pub async fn load_from_disk(&self, load_last_n: usize) -> Result<()> {
        let mut store = self.store.write().await;
        store.load_from_disk(load_last_n).await
    }
    
    pub async fn total_count(&self) -> usize {
        let store = self.store.read().await;
        store.total_count()
    }
    
    pub async fn memory_count(&self) -> usize {
        let store = self.store.read().await;
        store.memory_count()
    }
    
    pub async fn rotate_disk_file(&self, new_file: PathBuf) -> Result<()> {
        let mut store = self.store.write().await;
        store.rotate_disk_file(new_file).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_hybrid_bar_store() {
        let temp_file = PathBuf::from("test_bars.jsonl");
        let mut store = HybridBarStore::new(
            "NIFTY".to_string(),
            "1h".to_string(),
            temp_file.clone(),
            100,
        );
        
        // Append bars
        for i in 0..150 {
            let bar = Bar {
                timestamp: Utc::now(),
                timestamp_ms: Utc::now().timestamp_millis(),
                open: 19000.0 + i as f64,
                high: 19100.0 + i as f64,
                low: 18900.0 + i as f64,
                close: 19050.0 + i as f64,
                volume: 1000000,
                bar_complete: true,
            };
            store.append(bar).await.unwrap();
        }
        
        // Check memory count (should be capped at 100)
        assert_eq!(store.memory_count(), 100);
        assert_eq!(store.total_count(), 150);
        
        // Get recent bars
        let recent = store.get_recent(50).await.unwrap();
        assert_eq!(recent.len(), 50);
        
        // Get last bar
        let last = store.get_last().unwrap();
        assert_eq!(last.close, 19050.0 + 149.0);
        
        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }
}


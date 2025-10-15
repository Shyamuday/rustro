/// Tick buffer for real-time market data
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::types::Tick;

/// Buffer for storing recent ticks per symbol
pub struct TickBuffer {
    buffers: HashMap<String, VecDeque<Tick>>,
    capacity: usize,
}

impl TickBuffer {
    pub fn new(capacity: usize) -> Self {
        TickBuffer {
            buffers: HashMap::new(),
            capacity,
        }
    }
    
    /// Add a tick to the buffer
    pub fn push(&mut self, tick: Tick) {
        let buffer = self.buffers
            .entry(tick.symbol.clone())
            .or_insert_with(|| VecDeque::with_capacity(self.capacity));
        
        if buffer.len() >= self.capacity {
            buffer.pop_front();
        }
        buffer.push_back(tick);
    }
    
    /// Get the last tick for a symbol
    pub fn get_last(&self, symbol: &str) -> Option<&Tick> {
        self.buffers.get(symbol)?.back()
    }
    
    /// Get recent N ticks for a symbol
    pub fn get_recent(&self, symbol: &str, n: usize) -> Vec<Tick> {
        if let Some(buffer) = self.buffers.get(symbol) {
            buffer.iter()
                .rev()
                .take(n)
                .rev()
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all ticks for a symbol
    pub fn get_all(&self, symbol: &str) -> Vec<Tick> {
        if let Some(buffer) = self.buffers.get(symbol) {
            buffer.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Clear buffer for a symbol
    pub fn clear(&mut self, symbol: &str) {
        if let Some(buffer) = self.buffers.get_mut(symbol) {
            buffer.clear();
        }
    }
    
    /// Clear all buffers
    pub fn clear_all(&mut self) {
        self.buffers.clear();
    }
}

/// Thread-safe tick buffer
pub struct ConcurrentTickBuffer {
    buffer: Arc<RwLock<TickBuffer>>,
}

impl ConcurrentTickBuffer {
    pub fn new(capacity: usize) -> Self {
        ConcurrentTickBuffer {
            buffer: Arc::new(RwLock::new(TickBuffer::new(capacity))),
        }
    }
    
    pub async fn push(&self, tick: Tick) {
        let mut buffer = self.buffer.write().await;
        buffer.push(tick);
    }
    
    pub async fn get_last(&self, symbol: &str) -> Option<Tick> {
        let buffer = self.buffer.read().await;
        buffer.get_last(symbol).cloned()
    }
    
    pub async fn get_recent(&self, symbol: &str, n: usize) -> Vec<Tick> {
        let buffer = self.buffer.read().await;
        buffer.get_recent(symbol, n)
    }
    
    pub async fn get_all(&self, symbol: &str) -> Vec<Tick> {
        let buffer = self.buffer.read().await;
        buffer.get_all(symbol)
    }
    
    pub async fn clear(&self, symbol: &str) {
        let mut buffer = self.buffer.write().await;
        buffer.clear(symbol);
    }
    
    pub async fn clear_all(&self) {
        let mut buffer = self.buffer.write().await;
        buffer.clear_all();
    }
}


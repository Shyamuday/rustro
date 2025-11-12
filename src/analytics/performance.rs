/// Performance metrics and reporting module
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};

use crate::error::Result;
use crate::types::Position;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub date: String,
    pub timestamp: DateTime<Utc>,
    
    // Trade Statistics
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub breakeven_trades: usize,
    
    // Win/Loss Metrics
    pub win_rate: f64,
    pub loss_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    
    // P&L Metrics
    pub total_pnl: f64,
    pub gross_profit: f64,
    pub gross_loss: f64,
    pub net_pnl: f64,
    pub profit_factor: f64,
    
    // Risk Metrics
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_ratio: Option<f64>,
    pub avg_risk_reward: f64,
    
    // Execution Metrics
    pub avg_hold_time_minutes: f64,
    pub fastest_trade_minutes: f64,
    pub longest_trade_minutes: f64,
    
    // Strategy Performance
    pub ce_trades: usize,
    pub pe_trades: usize,
    pub ce_win_rate: f64,
    pub pe_win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPerformanceReport {
    pub metrics: PerformanceMetrics,
    pub trades: Vec<TradeRecord>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub position_id: String,
    pub symbol: String,
    pub option_type: String,
    pub strike: i32,
    pub side: String,
    pub quantity: i32,
    pub entry_price: f64,
    pub exit_price: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub hold_time_minutes: f64,
    pub pnl: f64,
    pub pnl_pct: f64,
    pub exit_reason: String,
}

pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Calculate comprehensive performance metrics from closed positions
    pub fn calculate_metrics(positions: &[Position]) -> PerformanceMetrics {
        let total_trades = positions.len();
        
        if total_trades == 0 {
            return Self::empty_metrics();
        }

        // Separate winning, losing, and breakeven trades
        let winning_trades: Vec<&Position> = positions.iter().filter(|p| p.pnl > 0.0).collect();
        let losing_trades: Vec<&Position> = positions.iter().filter(|p| p.pnl < 0.0).collect();
        let breakeven_trades: Vec<&Position> = positions.iter().filter(|p| p.pnl == 0.0).collect();

        let win_count = winning_trades.len();
        let loss_count = losing_trades.len();
        let breakeven_count = breakeven_trades.len();

        // Win/Loss rates
        let win_rate = if total_trades > 0 {
            (win_count as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };
        let loss_rate = if total_trades > 0 {
            (loss_count as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };

        // Average win/loss
        let avg_win = if !winning_trades.is_empty() {
            winning_trades.iter().map(|p| p.pnl).sum::<f64>() / winning_trades.len() as f64
        } else {
            0.0
        };
        let avg_loss = if !losing_trades.is_empty() {
            losing_trades.iter().map(|p| p.pnl).sum::<f64>() / losing_trades.len() as f64
        } else {
            0.0
        };

        // Largest win/loss
        let largest_win = winning_trades.iter().map(|p| p.pnl).fold(0.0, f64::max);
        let largest_loss = losing_trades.iter().map(|p| p.pnl).fold(0.0, f64::min);

        // P&L metrics
        let gross_profit: f64 = winning_trades.iter().map(|p| p.pnl).sum();
        let gross_loss: f64 = losing_trades.iter().map(|p| p.pnl.abs()).sum();
        let total_pnl: f64 = positions.iter().map(|p| p.pnl).sum();
        let net_pnl = total_pnl;

        // Profit factor
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else if gross_profit > 0.0 {
            f64::INFINITY
        } else {
            0.0
        };

        // Max drawdown
        let (max_dd, max_dd_pct) = Self::calculate_max_drawdown(positions);

        // Risk/Reward ratio
        let avg_risk_reward = if avg_loss != 0.0 {
            avg_win / avg_loss.abs()
        } else {
            0.0
        };

        // Hold time metrics
        let hold_times: Vec<f64> = positions.iter()
            .filter_map(|p| {
                if let Ok(exit_time) = chrono::DateTime::parse_from_rfc3339(&p.entry_time.to_rfc3339()) {
                    Some((Utc::now() - exit_time.with_timezone(&Utc)).num_minutes() as f64)
                } else {
                    None
                }
            })
            .collect();

        let avg_hold_time = if !hold_times.is_empty() {
            hold_times.iter().sum::<f64>() / hold_times.len() as f64
        } else {
            0.0
        };
        let fastest_trade = hold_times.iter().cloned().fold(f64::INFINITY, f64::min);
        let longest_trade = hold_times.iter().cloned().fold(0.0, f64::max);

        // Strategy breakdown
        let ce_trades = positions.iter().filter(|p| p.option_type.as_str() == "CE").count();
        let pe_trades = positions.iter().filter(|p| p.option_type.as_str() == "PE").count();
        
        let ce_wins = positions.iter().filter(|p| p.option_type.as_str() == "CE" && p.pnl > 0.0).count();
        let pe_wins = positions.iter().filter(|p| p.option_type.as_str() == "PE" && p.pnl > 0.0).count();

        let ce_win_rate = if ce_trades > 0 {
            (ce_wins as f64 / ce_trades as f64) * 100.0
        } else {
            0.0
        };
        let pe_win_rate = if pe_trades > 0 {
            (pe_wins as f64 / pe_trades as f64) * 100.0
        } else {
            0.0
        };

        PerformanceMetrics {
            date: Utc::now().format("%Y-%m-%d").to_string(),
            timestamp: Utc::now(),
            total_trades,
            winning_trades: win_count,
            losing_trades: loss_count,
            breakeven_trades: breakeven_count,
            win_rate,
            loss_rate,
            avg_win,
            avg_loss,
            largest_win,
            largest_loss,
            total_pnl,
            gross_profit,
            gross_loss,
            net_pnl,
            profit_factor,
            max_drawdown: max_dd,
            max_drawdown_pct: max_dd_pct,
            sharpe_ratio: None, // Would need returns series
            avg_risk_reward,
            avg_hold_time_minutes: avg_hold_time,
            fastest_trade_minutes: if fastest_trade.is_finite() { fastest_trade } else { 0.0 },
            longest_trade_minutes: longest_trade,
            ce_trades,
            pe_trades,
            ce_win_rate,
            pe_win_rate,
        }
    }

    /// Calculate maximum drawdown
    fn calculate_max_drawdown(positions: &[Position]) -> (f64, f64) {
        if positions.is_empty() {
            return (0.0, 0.0);
        }

        let mut cumulative_pnl = 0.0;
        let mut peak = 0.0;
        let mut max_dd = 0.0;

        for position in positions {
            cumulative_pnl += position.pnl;
            
            if cumulative_pnl > peak {
                peak = cumulative_pnl;
            }

            let drawdown = peak - cumulative_pnl;
            if drawdown > max_dd {
                max_dd = drawdown;
            }
        }

        let max_dd_pct = if peak > 0.0 {
            (max_dd / peak) * 100.0
        } else {
            0.0
        };

        (max_dd, max_dd_pct)
    }

    /// Create empty metrics (for days with no trades)
    fn empty_metrics() -> PerformanceMetrics {
        PerformanceMetrics {
            date: Utc::now().format("%Y-%m-%d").to_string(),
            timestamp: Utc::now(),
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            breakeven_trades: 0,
            win_rate: 0.0,
            loss_rate: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            total_pnl: 0.0,
            gross_profit: 0.0,
            gross_loss: 0.0,
            net_pnl: 0.0,
            profit_factor: 0.0,
            max_drawdown: 0.0,
            max_drawdown_pct: 0.0,
            sharpe_ratio: None,
            avg_risk_reward: 0.0,
            avg_hold_time_minutes: 0.0,
            fastest_trade_minutes: 0.0,
            longest_trade_minutes: 0.0,
            ce_trades: 0,
            pe_trades: 0,
            ce_win_rate: 0.0,
            pe_win_rate: 0.0,
        }
    }

    /// Generate daily performance report
    pub fn generate_daily_report(positions: &[Position]) -> DailyPerformanceReport {
        let metrics = Self::calculate_metrics(positions);
        
        let trades: Vec<TradeRecord> = positions.iter().map(|p| TradeRecord {
            position_id: p.position_id.clone(),
            symbol: p.symbol.clone(),
            option_type: p.option_type.as_str().to_string(),
            strike: p.strike,
            side: p.side.as_str().to_string(),
            quantity: p.quantity,
            entry_price: p.entry_price,
            exit_price: p.current_price,
            entry_time: p.entry_time,
            exit_time: Utc::now(),
            hold_time_minutes: (Utc::now() - p.entry_time).num_minutes() as f64,
            pnl: p.pnl,
            pnl_pct: p.pnl_pct,
            exit_reason: "EOD".to_string(),
        }).collect();

        let mut notes = Vec::new();
        
        // Add performance notes
        if metrics.win_rate >= 60.0 {
            notes.push("✅ Excellent win rate today!".to_string());
        } else if metrics.win_rate < 40.0 && metrics.total_trades > 0 {
            notes.push("⚠️  Low win rate - review strategy".to_string());
        }

        if metrics.profit_factor >= 2.0 {
            notes.push("✅ Strong profit factor".to_string());
        } else if metrics.profit_factor < 1.0 && metrics.total_trades > 0 {
            notes.push("⚠️  Profit factor below 1 - losing more than winning".to_string());
        }

        if metrics.max_drawdown_pct > 10.0 {
            notes.push("⚠️  Significant drawdown today - review risk management".to_string());
        }

        DailyPerformanceReport {
            metrics,
            trades,
            notes,
        }
    }

    /// Save performance report to disk
    pub async fn save_report(report: &DailyPerformanceReport) -> Result<()> {
        let data_dir = "data/performance";
        tokio::fs::create_dir_all(data_dir).await?;

        let filename = format!("{}/performance_{}.json", 
                              data_dir, 
                              report.metrics.date);
        
        let json = serde_json::to_string_pretty(report)?;
        tokio::fs::write(&filename, json).await?;

        info!("💾 Saved performance report to {}", filename);
        
        // Also save a summary CSV for easy analysis
        Self::append_to_summary_csv(report).await?;

        Ok(())
    }

    /// Append metrics to summary CSV for trend analysis
    async fn append_to_summary_csv(report: &DailyPerformanceReport) -> Result<()> {
        let csv_file = "data/performance/summary.csv";
        let m = &report.metrics;

        // Create header if file doesn't exist
        if !Path::new(csv_file).exists() {
            let header = "Date,Total Trades,Win Rate,Profit Factor,Total P&L,Max Drawdown %,Avg Win,Avg Loss,CE Trades,PE Trades\n";
            tokio::fs::write(csv_file, header).await?;
        }

        // Append data
        let row = format!(
            "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{}\n",
            m.date, m.total_trades, m.win_rate, m.profit_factor, m.total_pnl,
            m.max_drawdown_pct, m.avg_win, m.avg_loss, m.ce_trades, m.pe_trades
        );

        let mut content = tokio::fs::read_to_string(csv_file).await.unwrap_or_default();
        content.push_str(&row);
        tokio::fs::write(csv_file, content).await?;

        info!("📊 Updated performance summary CSV");
        Ok(())
    }

    /// Load historical performance metrics
    pub async fn load_historical_metrics(days: usize) -> Result<Vec<PerformanceMetrics>> {
        let mut metrics = Vec::new();
        let data_dir = "data/performance";

        if !Path::new(data_dir).exists() {
            return Ok(metrics);
        }

        let mut entries = tokio::fs::read_dir(data_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(report) = serde_json::from_str::<DailyPerformanceReport>(&content) {
                        metrics.push(report.metrics);
                    }
                }
            }
        }

        // Sort by date (newest first)
        metrics.sort_by(|a, b| b.date.cmp(&a.date));
        
        // Limit to requested days
        metrics.truncate(days);

        Ok(metrics)
    }

    /// Print performance summary to console
    pub fn print_summary(metrics: &PerformanceMetrics) {
        info!("📊 ═══════════════════════════════════════════════════");
        info!("📊 DAILY PERFORMANCE SUMMARY - {}", metrics.date);
        info!("📊 ═══════════════════════════════════════════════════");
        info!("");
        info!("📈 TRADE STATISTICS:");
        info!("   Total Trades: {}", metrics.total_trades);
        info!("   Winning: {} ({:.1}%)", metrics.winning_trades, metrics.win_rate);
        info!("   Losing: {} ({:.1}%)", metrics.losing_trades, metrics.loss_rate);
        info!("   Breakeven: {}", metrics.breakeven_trades);
        info!("");
        info!("💰 P&L METRICS:");
        info!("   Total P&L: ₹{:.2}", metrics.total_pnl);
        info!("   Gross Profit: ₹{:.2}", metrics.gross_profit);
        info!("   Gross Loss: ₹{:.2}", metrics.gross_loss);
        info!("   Profit Factor: {:.2}", metrics.profit_factor);
        info!("");
        info!("📊 TRADE QUALITY:");
        info!("   Avg Win: ₹{:.2}", metrics.avg_win);
        info!("   Avg Loss: ₹{:.2}", metrics.avg_loss);
        info!("   Risk/Reward: {:.2}", metrics.avg_risk_reward);
        info!("   Largest Win: ₹{:.2}", metrics.largest_win);
        info!("   Largest Loss: ₹{:.2}", metrics.largest_loss);
        info!("");
        info!("⚠️  RISK METRICS:");
        info!("   Max Drawdown: ₹{:.2} ({:.2}%)", metrics.max_drawdown, metrics.max_drawdown_pct);
        info!("");
        info!("⏱️  EXECUTION:");
        info!("   Avg Hold Time: {:.1} min", metrics.avg_hold_time_minutes);
        info!("   Fastest Trade: {:.1} min", metrics.fastest_trade_minutes);
        info!("   Longest Trade: {:.1} min", metrics.longest_trade_minutes);
        info!("");
        info!("🎯 STRATEGY BREAKDOWN:");
        info!("   CE Trades: {} (Win Rate: {:.1}%)", metrics.ce_trades, metrics.ce_win_rate);
        info!("   PE Trades: {} (Win Rate: {:.1}%)", metrics.pe_trades, metrics.pe_win_rate);
        info!("📊 ═══════════════════════════════════════════════════");
    }
}




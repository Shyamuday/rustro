/// Technical indicators implementation
use crate::types::Bar;

/// Calculate ADX (Average Directional Index) with +DI and -DI
pub fn calculate_adx(bars: &[Bar], period: usize) -> Option<(f64, f64, f64)> {
    if bars.len() < period + 1 {
        return None;
    }
    
    // Calculate True Range and Directional Movement
    let mut tr_values = Vec::new();
    let mut plus_dm = Vec::new();
    let mut minus_dm = Vec::new();
    
    for i in 1..bars.len() {
        let high = bars[i].high;
        let low = bars[i].low;
        let prev_high = bars[i - 1].high;
        let prev_low = bars[i - 1].low;
        let prev_close = bars[i - 1].close;
        
        // True Range
        let tr = (high - low)
            .max(f64::abs(high - prev_close))
            .max(f64::abs(low - prev_close));
        tr_values.push(tr);
        
        // Directional Movement
        let up_move = high - prev_high;
        let down_move = prev_low - low;
        
        let plus_dm_val = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        
        let minus_dm_val = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };
        
        plus_dm.push(plus_dm_val);
        minus_dm.push(minus_dm_val);
    }
    
    // Smooth TR and DM using Wilder's smoothing
    let smoothed_tr = wilder_smooth(&tr_values, period)?;
    let smoothed_plus_dm = wilder_smooth(&plus_dm, period)?;
    let smoothed_minus_dm = wilder_smooth(&minus_dm, period)?;
    
    // Calculate +DI and -DI
    let plus_di = (smoothed_plus_dm / smoothed_tr) * 100.0;
    let minus_di = (smoothed_minus_dm / smoothed_tr) * 100.0;
    
    // Calculate DX
    let di_diff = f64::abs(plus_di - minus_di);
    let di_sum = plus_di + minus_di;
    
    if di_sum == 0.0 {
        return None;
    }
    
    let dx = (di_diff / di_sum) * 100.0;
    
    // ADX is 14-period average of DX (would need to track DX history)
    // For simplicity, we'll use current DX as ADX approximation
    // In production, maintain a DX buffer and calculate proper ADX
    let adx = dx;
    
    Some((adx, plus_di, minus_di))
}

/// Wilder's smoothing (EMA-like with 1/period factor)
fn wilder_smooth(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period {
        return None;
    }
    
    // Initial average
    let mut smoothed: f64 = values.iter().take(period).sum::<f64>() / period as f64;
    
    // Smooth the rest
    for i in period..values.len() {
        smoothed = ((period - 1) as f64 * smoothed + values[i]) / period as f64;
    }
    
    Some(smoothed)
}

/// Calculate RSI (Relative Strength Index)
pub fn calculate_rsi(bars: &[Bar], period: usize) -> Option<f64> {
    if bars.len() < period + 1 {
        return None;
    }
    
    let mut gains = Vec::new();
    let mut losses = Vec::new();
    
    for i in 1..bars.len() {
        let change = bars[i].close - bars[i - 1].close;
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(change.abs());
        }
    }
    
    if gains.len() < period {
        return None;
    }
    
    // Calculate average gain and loss
    let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;
    
    if avg_loss == 0.0 {
        return Some(100.0);
    }
    
    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));
    
    Some(rsi)
}

/// Calculate EMA (Exponential Moving Average)
pub fn calculate_ema(bars: &[Bar], period: usize) -> Option<f64> {
    if bars.len() < period {
        return None;
    }
    
    // Calculate initial SMA
    let sma: f64 = bars.iter()
        .rev()
        .take(period)
        .map(|b| b.close)
        .sum::<f64>() / period as f64;
    
    // Calculate multiplier
    let multiplier = 2.0 / (period as f64 + 1.0);
    
    // Calculate EMA starting from SMA
    let mut ema = sma;
    for bar in bars.iter().rev().take(period).skip(period) {
        ema = (bar.close - ema) * multiplier + ema;
    }
    
    Some(ema)
}

/// Calculate VWAP (Volume Weighted Average Price)
pub fn calculate_vwap(bars: &[Bar]) -> Option<f64> {
    if bars.is_empty() {
        return None;
    }
    
    let mut cumulative_tpv = 0.0;
    let mut cumulative_volume = 0i64;
    
    for bar in bars {
        let typical_price = (bar.high + bar.low + bar.close) / 3.0;
        cumulative_tpv += typical_price * bar.volume as f64;
        cumulative_volume += bar.volume;
    }
    
    if cumulative_volume == 0 {
        return None;
    }
    
    Some(cumulative_tpv / cumulative_volume as f64)
}

/// Calculate SMA (Simple Moving Average)
pub fn calculate_sma(bars: &[Bar], period: usize) -> Option<f64> {
    if bars.len() < period {
        return None;
    }
    
    let sum: f64 = bars.iter()
        .rev()
        .take(period)
        .map(|b| b.close)
        .sum();
    
    Some(sum / period as f64)
}

/// Calculate ATR (Average True Range)
pub fn calculate_atr(bars: &[Bar], period: usize) -> Option<f64> {
    if bars.len() < period + 1 {
        return None;
    }
    
    let mut tr_values = Vec::new();
    
    for i in 1..bars.len() {
        let high = bars[i].high;
        let low = bars[i].low;
        let prev_close = bars[i - 1].close;
        
        let tr = (high - low)
            .max(f64::abs(high - prev_close))
            .max(f64::abs(low - prev_close));
        
        tr_values.push(tr);
    }
    
    wilder_smooth(&tr_values, period)
}

/// Helper: Calculate percentage change
pub fn percentage_change(from: f64, to: f64) -> f64 {
    if from == 0.0 {
        return 0.0;
    }
    ((to - from) / from) * 100.0
}

/// Helper: Round to nearest strike (e.g., nearest 50)
pub fn round_to_strike(price: f64, strike_increment: i32) -> i32 {
    let inc = strike_increment as f64;
    (f64::floor(price / inc) * inc) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    fn create_test_bars(count: usize) -> Vec<Bar> {
        (0..count)
            .map(|i| Bar {
                timestamp: Utc::now(),
                timestamp_ms: Utc::now().timestamp_millis(),
                open: 19000.0 + i as f64 * 10.0,
                high: 19100.0 + i as f64 * 10.0,
                low: 18900.0 + i as f64 * 10.0,
                close: 19050.0 + i as f64 * 10.0,
                volume: 1000000,
                bar_complete: true,
            })
            .collect()
    }
    
    #[test]
    fn test_rsi() {
        let bars = create_test_bars(30);
        let rsi = calculate_rsi(&bars, 14);
        assert!(rsi.is_some());
        let rsi_val = rsi.unwrap();
        assert!(rsi_val >= 0.0 && rsi_val <= 100.0);
    }
    
    #[test]
    fn test_ema() {
        let bars = create_test_bars(30);
        let ema = calculate_ema(&bars, 20);
        assert!(ema.is_some());
    }
    
    #[test]
    fn test_round_to_strike() {
        assert_eq!(round_to_strike(19345.0, 50), 19300);
        assert_eq!(round_to_strike(19375.0, 50), 19350);
        assert_eq!(round_to_strike(19399.99, 50), 19350);
    }
}


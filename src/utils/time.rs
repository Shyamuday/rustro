/// Time utilities for market session management
use chrono::{DateTime, Datelike, NaiveTime, TimeZone, Utc};
use chrono_tz::Asia::Kolkata;

/// Check if current time is within entry window
pub fn is_in_entry_window(
    now: DateTime<Utc>,
    window_start: &str,
    window_end: &str,
) -> bool {
    let now_ist = now.with_timezone(&Kolkata);
    
    let start_time = NaiveTime::parse_from_str(window_start, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(window_start, "%H:%M"))
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(10, 0, 0).unwrap());
    
    let end_time = NaiveTime::parse_from_str(window_end, "%H:%M:%S")
        .or_else(|_| NaiveTime::parse_from_str(window_end, "%H:%M"))
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    
    let current_time = now_ist.time();
    current_time >= start_time && current_time < end_time
}

/// Check if market is open
pub fn is_market_open(now: DateTime<Utc>) -> bool {
    let now_ist = now.with_timezone(&Kolkata);
    let current_time = now_ist.time();
    
    let market_open = NaiveTime::from_hms_opt(9, 15, 0).unwrap();
    let market_close = NaiveTime::from_hms_opt(15, 30, 0).unwrap();
    
    current_time >= market_open && current_time < market_close
}

/// Get next market open time
pub fn next_market_open(now: DateTime<Utc>) -> DateTime<Utc> {
    let now_ist = now.with_timezone(&Kolkata);
    
    let market_open_time = NaiveTime::from_hms_opt(9, 15, 0).unwrap();
    let today_open = Kolkata.with_ymd_and_hms(
        now_ist.year(),
        now_ist.month(),
        now_ist.day(),
        9,
        15,
        0,
    ).unwrap();
    
    if now_ist < today_open {
        today_open.with_timezone(&Utc)
    } else {
        // Next day
        (today_open + chrono::Duration::days(1)).with_timezone(&Utc)
    }
}

/// Calculate days to expiry (simplified - assumes weekly Thursday expiry)
pub fn calculate_days_to_expiry(now: DateTime<Utc>) -> i32 {
    let now_ist = now.with_timezone(&Kolkata);
    let current_day = now_ist.weekday().num_days_from_monday();
    
    // Thursday is day 3 (Mon=0, Tue=1, Wed=2, Thu=3, Fri=4)
    let days_until_thursday = if current_day <= 3 {
        3 - current_day
    } else {
        7 - current_day + 3
    };
    
    days_until_thursday as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entry_window() {
        // Create a test time: 10:30 IST
        let test_time = Kolkata.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let test_time_utc = test_time.with_timezone(&Utc);
        
        assert!(is_in_entry_window(test_time_utc, "10:00:00", "15:00:00"));
        assert!(!is_in_entry_window(test_time_utc, "11:00:00", "15:00:00"));
    }
    
    #[test]
    fn test_market_open() {
        let market_time = Kolkata.with_ymd_and_hms(2025, 1, 15, 10, 30, 0).unwrap();
        let market_time_utc = market_time.with_timezone(&Utc);
        
        assert!(is_market_open(market_time_utc));
        
        let before_market = Kolkata.with_ymd_and_hms(2025, 1, 15, 9, 0, 0).unwrap();
        let before_market_utc = before_market.with_timezone(&Utc);
        
        assert!(!is_market_open(before_market_utc));
    }
}

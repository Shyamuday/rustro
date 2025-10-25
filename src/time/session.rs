/// Market session and timing utilities
use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::Asia::Kolkata;

/// Check if today is a trading day (simplified - doesn't check holidays)
pub fn is_trading_day(date: DateTime<Utc>) -> bool {
    let date_ist = date.with_timezone(&Kolkata);
    let weekday = date_ist.weekday();
    
    // Monday = 0, Saturday = 5, Sunday = 6
    let day_num = weekday.num_days_from_monday();
    day_num < 5 // Monday to Friday only
}

/// Get market timings for today
pub fn get_market_timings(date: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let date_ist = date.with_timezone(&Kolkata);
    
    // Market open: 9:15 AM IST
    let market_open = Kolkata
        .with_ymd_and_hms(
            date_ist.year(),
            date_ist.month(),
            date_ist.day(),
            9,
            15,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    
    // Market close: 3:30 PM IST
    let market_close = Kolkata
        .with_ymd_and_hms(
            date_ist.year(),
            date_ist.month(),
            date_ist.day(),
            15,
            30,
            0,
        )
        .unwrap()
        .with_timezone(&Utc);
    
    (market_open, market_close)
}

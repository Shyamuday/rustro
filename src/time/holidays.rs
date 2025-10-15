/// NSE Holiday Calendar Management
use chrono::{Datelike, NaiveDate};
use std::collections::HashSet;

/// NSE Holidays for 2025 (update annually)
pub fn get_nse_holidays_2025() -> HashSet<NaiveDate> {
    let mut holidays = HashSet::new();
    
    // January 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 1, 26).unwrap()); // Republic Day
    
    // February 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 2, 26).unwrap()); // Mahashivratri
    
    // March 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 3, 14).unwrap()); // Holi
    holidays.insert(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()); // Id-Ul-Fitr
    
    // April 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 4, 10).unwrap()); // Mahavir Jayanti
    holidays.insert(NaiveDate::from_ymd_opt(2025, 4, 14).unwrap()); // Dr. Ambedkar Jayanti
    holidays.insert(NaiveDate::from_ymd_opt(2025, 4, 18).unwrap()); // Good Friday
    
    // May 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());  // Maharashtra Day
    holidays.insert(NaiveDate::from_ymd_opt(2025, 5, 12).unwrap()); // Buddha Purnima
    
    // June 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 6, 7).unwrap());  // Bakri Id
    
    // July 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 7, 7).unwrap());  // Muharram
    
    // August 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 8, 15).unwrap()); // Independence Day
    holidays.insert(NaiveDate::from_ymd_opt(2025, 8, 27).unwrap()); // Ganesh Chaturthi
    
    // September 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 9, 5).unwrap());  // Eid-E-Milad
    
    // October 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 10, 2).unwrap());  // Mahatma Gandhi Jayanti
    holidays.insert(NaiveDate::from_ymd_opt(2025, 10, 12).unwrap()); // Dussehra
    holidays.insert(NaiveDate::from_ymd_opt(2025, 10, 20).unwrap()); // Diwali Balipratipada
    holidays.insert(NaiveDate::from_ymd_opt(2025, 10, 21).unwrap()); // Diwali
    
    // November 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 11, 5).unwrap());  // Gurunanak Jayanti
    
    // December 2025
    holidays.insert(NaiveDate::from_ymd_opt(2025, 12, 25).unwrap()); // Christmas
    
    holidays
}

/// Check if a date is a trading day (not weekend, not holiday)
pub fn is_trading_day(date: NaiveDate) -> bool {
    // Check weekend
    let weekday = date.weekday();
    if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
        return false;
    }
    
    // Check holiday
    let holidays = get_nse_holidays_2025();
    !holidays.contains(&date)
}

/// Get next trading day
pub fn next_trading_day(from_date: NaiveDate) -> NaiveDate {
    let mut date = from_date + chrono::Duration::days(1);
    
    while !is_trading_day(date) {
        date = date + chrono::Duration::days(1);
    }
    
    date
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_republic_day_holiday() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 26).unwrap();
        assert!(!is_trading_day(date));
    }
    
    #[test]
    fn test_weekend() {
        let sat = NaiveDate::from_ymd_opt(2025, 1, 4).unwrap(); // Saturday
        let sun = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(); // Sunday
        assert!(!is_trading_day(sat));
        assert!(!is_trading_day(sun));
    }
    
    #[test]
    fn test_regular_weekday() {
        let mon = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(); // Monday (not holiday)
        assert!(is_trading_day(mon));
    }
}


/// Pre-order validation - All 9 checks from spec
use crate::error::{Result, TradingError};
use crate::types::{Config, Instrument, Side};

pub struct OrderValidator {
    config: std::sync::Arc<Config>,
}

impl OrderValidator {
    pub fn new(config: std::sync::Arc<Config>) -> Self {
        OrderValidator { config }
    }
    
    /// Validate order before placement (all 9 checks)
    pub fn validate_order(
        &self,
        symbol: &str,
        quantity: i32,
        price: f64,
        _side: Side,
        instrument: &Instrument,
        account_balance: f64,
    ) -> Result<()> {
        // Check 1: Freeze quantity
        self.check_freeze_quantity(quantity, &instrument.name)?;
        
        // Check 2: Lot size
        self.check_lot_size(quantity, instrument.lotsize)?;
        
        // Check 3: Tick size
        self.check_tick_size(price, instrument.tick_size)?;
        
        // Check 4: Price bands (circuit limits)
        self.check_price_bands(price, instrument)?;
        
        // Check 5: Margin requirement
        self.check_margin(quantity, price, account_balance)?;
        
        // Check 6: Symbol validity
        self.check_symbol_validity(symbol, instrument)?;
        
        // Check 7: Market hours
        self.check_market_hours()?;
        
        // Check 8: Quantity > 0
        self.check_positive_quantity(quantity)?;
        
        // Check 9: Price > 0
        self.check_positive_price(price)?;
        
        Ok(())
    }
    
    /// Check 1: Freeze quantity not exceeded
    fn check_freeze_quantity(&self, quantity: i32, underlying: &str) -> Result<()> {
        let freeze_limit = self.config.get_freeze_quantity(underlying);
        
        if quantity > freeze_limit {
            return Err(TradingError::FreezeQuantityBreach(format!(
                "Quantity {} exceeds freeze limit {} for {}",
                quantity, freeze_limit, underlying
            )));
        }
        
        Ok(())
    }
    
    /// Check 2: Quantity is multiple of lot size
    fn check_lot_size(&self, quantity: i32, lot_size: i32) -> Result<()> {
        if quantity % lot_size != 0 {
            return Err(TradingError::InvalidParameter(format!(
                "Quantity {} is not a multiple of lot size {}",
                quantity, lot_size
            )));
        }
        
        Ok(())
    }
    
    /// Check 3: Price is multiple of tick size
    fn check_tick_size(&self, price: f64, tick_size: f64) -> Result<()> {
        let remainder = (price % tick_size).abs();
        
        if remainder > 0.001 { // Allow small floating point errors
            return Err(TradingError::InvalidParameter(format!(
                "Price {} is not a multiple of tick size {}",
                price, tick_size
            )));
        }
        
        Ok(())
    }
    
    /// Check 4: Price within circuit limits (±20%)
    fn check_price_bands(&self, price: f64, instrument: &Instrument) -> Result<()> {
        // Simplified: assume current price = strike price
        let reference_price = instrument.strike;
        let max_deviation = reference_price * (self.config.price_band_pct / 100.0);
        
        let upper_limit = reference_price + max_deviation;
        let lower_limit = (reference_price - max_deviation).max(0.0);
        
        if price > upper_limit || price < lower_limit {
            return Err(TradingError::PriceBandBreach(format!(
                "Price {} outside bands [{:.2}, {:.2}]",
                price, lower_limit, upper_limit
            )));
        }
        
        Ok(())
    }
    
    /// Check 5: Sufficient margin available
    fn check_margin(&self, quantity: i32, price: f64, account_balance: f64) -> Result<()> {
        // Simplified margin calculation
        // For options: Premium + margin (assume 20% of contract value)
        let premium = quantity as f64 * price;
        let margin_required = premium * 0.20;
        let total_required = premium + margin_required;
        
        if total_required > account_balance {
            return Err(TradingError::InsufficientMargin(format!(
                "Required: {:.2}, Available: {:.2}",
                total_required, account_balance
            )));
        }
        
        Ok(())
    }
    
    /// Check 6: Symbol matches instrument
    fn check_symbol_validity(&self, symbol: &str, instrument: &Instrument) -> Result<()> {
        if symbol != instrument.symbol {
            return Err(TradingError::InvalidParameter(format!(
                "Symbol mismatch: {} != {}",
                symbol, instrument.symbol
            )));
        }
        
        Ok(())
    }
    
    /// Check 7: Market is open
    fn check_market_hours(&self) -> Result<()> {
        use crate::utils::is_market_open;
        
        if !is_market_open(chrono::Utc::now()) {
            return Err(TradingError::MarketClosed(
                "Market is closed".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check 8: Quantity > 0
    fn check_positive_quantity(&self, quantity: i32) -> Result<()> {
        if quantity <= 0 {
            return Err(TradingError::InvalidParameter(format!(
                "Quantity must be positive, got {}",
                quantity
            )));
        }
        
        Ok(())
    }
    
    /// Check 9: Price > 0
    fn check_positive_price(&self, price: f64) -> Result<()> {
        if price <= 0.0 {
            return Err(TradingError::InvalidParameter(format!(
                "Price must be positive, got {}",
                price
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OptionType, Instrument};
    
    fn create_test_instrument() -> Instrument {
        Instrument {
            token: "12345".to_string(),
            symbol: "NIFTY24OCT19500CE".to_string(),
            name: "NIFTY".to_string(),
            expiry: "24OCT2024".to_string(),
            strike: 19500.0,
            lotsize: 50,
            instrument_type: "OPTIDX".to_string(),
            exch_seg: "NFO".to_string(),
            tick_size: 0.05,
        }
    }
    
    #[test]
    fn test_lot_size_validation() {
        let validator = OrderValidator::new(std::sync::Arc::new(create_test_config()));
        
        // Valid: 50 is multiple of 50
        assert!(validator.check_lot_size(50, 50).is_ok());
        
        // Invalid: 75 is not multiple of 50
        assert!(validator.check_lot_size(75, 50).is_err());
    }
    
    #[test]
    fn test_tick_size_validation() {
        let validator = OrderValidator::new(std::sync::Arc::new(create_test_config()));
        
        // Valid: 125.50 is multiple of 0.05
        assert!(validator.check_tick_size(125.50, 0.05).is_ok());
        
        // Invalid: 125.53 is not multiple of 0.05
        assert!(validator.check_tick_size(125.53, 0.05).is_err());
    }
    
    fn create_test_config() -> Config {
        // Would create actual config in real test
        unimplemented!()
    }
}

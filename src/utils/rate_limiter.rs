/// Token bucket rate limiter
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct RateLimiter {
    capacity: u32,
    tokens: Arc<Mutex<u32>>,
    refill_rate: Duration,
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        RateLimiter {
            capacity: requests_per_second,
            tokens: Arc::new(Mutex::new(requests_per_second)),
            refill_rate: Duration::from_secs(1),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    /// Try to acquire a token, returns true if successful
    pub async fn try_acquire(&self) -> bool {
        // Refill tokens based on elapsed time
        self.refill().await;
        
        let mut tokens = self.tokens.lock().await;
        if *tokens > 0 {
            *tokens -= 1;
            true
        } else {
            false
        }
    }
    
    /// Wait until a token is available, then acquire it
    pub async fn acquire(&self) {
        loop {
            if self.try_acquire().await {
                return;
            }
            
            // Wait a bit before retry
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Refill tokens based on elapsed time
    async fn refill(&self) {
        let mut last_refill = self.last_refill.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);
        
        if elapsed >= self.refill_rate {
            let periods = (elapsed.as_secs_f64() / self.refill_rate.as_secs_f64()) as u32;
            
            let mut tokens = self.tokens.lock().await;
            *tokens = (*tokens + periods).min(self.capacity);
            *last_refill = now;
        }
    }
    
    /// Get current available tokens
    pub async fn available(&self) -> u32 {
        self.refill().await;
        let tokens = self.tokens.lock().await;
        *tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2); // 2 requests per second
        
        // Should get 2 tokens immediately
        assert!(limiter.try_acquire().await);
        assert!(limiter.try_acquire().await);
        
        // Third should fail
        assert!(!limiter.try_acquire().await);
        
        // Wait for refill
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should work again
        assert!(limiter.try_acquire().await);
    }
}


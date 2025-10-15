/// Idempotency key generation
use sha2::{Digest, Sha256};

pub fn generate_idempotency_key(components: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for component in components {
        hasher.update(component.as_bytes());
    }
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_idempotency_key() {
        let key1 = generate_idempotency_key(&["session1", "NIFTY", "CE", "19000"]);
        let key2 = generate_idempotency_key(&["session1", "NIFTY", "CE", "19000"]);
        let key3 = generate_idempotency_key(&["session1", "NIFTY", "PE", "19000"]);
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}


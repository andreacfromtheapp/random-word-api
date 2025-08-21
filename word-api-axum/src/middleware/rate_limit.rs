//! Rate limiting middleware implementation
//!
//! Provides simple helper functions for rate limiting with tower-governor.

use crate::config::ApiConfig;

/// Check if rate limiting is enabled based on configuration
pub fn is_enabled(config: &ApiConfig) -> bool {
    config.rate_limit_per_second > 0
}

/// Get the per-second rate limit value
pub fn get_per_second(config: &ApiConfig) -> u64 {
    config.rate_limit_per_second
}

/// Get the burst size (2x the per-second rate)
pub fn get_burst_size(config: &ApiConfig) -> u32 {
    (config.rate_limit_per_second * 2) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;

    #[test]
    fn test_rate_limiting_enabled() {
        let mut config = ApiConfig::default();
        config.rate_limit_per_second = 5;
        assert!(is_enabled(&config));
    }

    #[test]
    fn test_rate_limiting_disabled() {
        let mut config = ApiConfig::default();
        config.rate_limit_per_second = 0;
        assert!(!is_enabled(&config));
    }

    #[test]
    fn test_get_per_second() {
        let mut config = ApiConfig::default();
        config.rate_limit_per_second = 10;
        assert_eq!(get_per_second(&config), 10);
    }

    #[test]
    fn test_get_burst_size() {
        let mut config = ApiConfig::default();
        config.rate_limit_per_second = 3;
        assert_eq!(get_burst_size(&config), 6);
    }
}

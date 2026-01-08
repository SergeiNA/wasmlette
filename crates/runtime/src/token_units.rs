use anyhow::{anyhow, Result};

// ============= Denomination Constants =============

/// Number of decimal places for token precision
pub const DECIMALS: u32 = 6;

/// Smallest units per 1 token (10^6 micro-tokens)
///
/// Example:
/// - 1.0 token = 1_000_000 units
/// - 0.001 token = 1_000 units
/// - 0.000001 token = 1 unit (smallest possible)
pub const ONE_TOKEN: u64 = 1_000_000;

// ============= Token Amount Conversion =============

/// Utilities for converting between human-readable tokens and smallest units
pub struct TokenUnit;

impl TokenUnit {
    /// Convert human-readable tokens to smallest units
    ///
    /// # Example
    /// ```
    /// let units = TokenAmount::from_tokens(1.5);
    /// assert_eq!(units, 1_500_000);
    /// ```
    pub fn from_tokens(tokens: f64) -> u64 {
        (tokens * ONE_TOKEN as f64) as u64
    }

    /// Convert smallest units to human-readable tokens
    ///
    /// # Example
    /// ```
    /// let tokens = TokenAmount::from_units(1_500_000);
    /// assert_eq!(tokens, 1.5);
    /// ```
    pub fn from_units(units: u64) -> f64 {
        units as f64 / ONE_TOKEN as f64
    }

    /// Format token amount as string with proper decimals
    ///
    /// # Example
    /// ```
    /// let formatted = TokenAmount::format(1_500_000);
    /// assert_eq!(formatted, "1.500000000");
    /// ```
    pub fn format(units: u64) -> String {
        format!("{:.9}", Self::from_units(units))
    }
}

mod tests {
    use super::*;
    #[test]
    fn test_token_amount_conversion() {
        assert_eq!(TokenUnit::from_tokens(1.5), 1_500_000);
        assert_eq!(TokenUnit::from_units(1_500_000), 1.5);
        assert_eq!(TokenUnit::format(1_500_000), "1.500000000");
    }
}
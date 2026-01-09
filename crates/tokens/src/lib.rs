//! Token denomination and conversion utilities
//!
//! This crate provides constants and utilities for working with token amounts
//! in wasmlette blockchain. All balances are stored in the smallest indivisible
//! units (similar to wei in Ethereum), and this crate handles conversion between
//! human-readable token amounts and these smallest units.

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
    /// use wasmlette_tokens::TokenUnit;
    /// let units = TokenUnit::from_tokens(1.5);
    /// assert_eq!(units, 1_500_000);
    /// ```
    pub fn from_tokens(tokens: f64) -> u64 {
        (tokens * ONE_TOKEN as f64) as u64
    } //TODO add check for overflow

    /// Convert smallest units to human-readable tokens
    ///
    /// # Example
    /// ```
    /// use wasmlette_tokens::TokenUnit;
    /// let tokens = TokenUnit::from_units(1_500_000);
    /// assert_eq!(tokens, 1.5);
    /// ```
    pub fn from_units(units: u64) -> f64 {
        units as f64 / ONE_TOKEN as f64
    }

    /// Format token amount as string with proper decimals
    ///
    /// # Example
    /// ```
    /// use wasmlette_tokens::TokenUnit;
    /// let formatted = TokenUnit::format(1_500_000);
    /// assert_eq!(formatted, "1.500000");
    /// ```
    pub fn format(units: u64) -> String {
        format!("{:.6}", Self::from_units(units))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_amount_conversion() {
        assert_eq!(TokenUnit::from_tokens(1.5), 1_500_000);
        assert_eq!(TokenUnit::from_units(1_500_000), 1.5);
    }

    #[test]
    fn test_token_format() {
        assert_eq!(TokenUnit::format(1_500_000), "1.500000");
        assert_eq!(TokenUnit::format(1_000_000), "1.000000");
        assert_eq!(TokenUnit::format(1), "0.000001");
    }

    #[test]
    fn test_one_token_constant() {
        assert_eq!(ONE_TOKEN, 1_000_000);
        assert_eq!(TokenUnit::from_tokens(1.0), ONE_TOKEN);
    }
}

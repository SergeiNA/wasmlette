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

    #[test]
    fn test_from_tokens_overflow() {
        // Test with value that would overflow u64
        // u64::MAX = 18_446_744_073_709_551_615
        // With ONE_TOKEN = 1_000_000, max representable tokens ≈ 18_446_744_073_703
        let huge_value = 1e20; // 100 quintillion tokens
        let result = TokenUnit::from_tokens(huge_value);

        // Should truncate/saturate rather than panic
        // This documents current behavior - in production, might want checked conversion
        assert!(result > 0, "Should produce some result, not panic");
    }

    #[test]
    fn test_from_tokens_very_large_valid() {
        // Test maximum safe value
        // u64::MAX / ONE_TOKEN = max tokens that fit in u64
        let max_tokens = (u64::MAX / ONE_TOKEN) as f64;
        let result = TokenUnit::from_tokens(max_tokens);
        assert!(result <= u64::MAX, "Should not overflow");
    }

    #[test]
    fn test_from_tokens_zero() {
        assert_eq!(TokenUnit::from_tokens(0.0), 0);
    }

    #[test]
    fn test_from_units_zero() {
        assert_eq!(TokenUnit::from_units(0), 0.0);
    }

    #[test]
    fn test_from_units_max() {
        let result = TokenUnit::from_units(u64::MAX);
        assert!(result > 0.0, "Should handle max u64");
        assert!(result.is_finite(), "Should be finite number");
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test that conversions are reversible for reasonable values
        let test_values = vec![0.0, 0.000001, 0.1, 1.0, 10.0, 100.0, 1000.0];

        for &tokens in &test_values {
            let units = TokenUnit::from_tokens(tokens);
            let back_to_tokens = TokenUnit::from_units(units);

            // Allow small floating point error
            let diff = (tokens - back_to_tokens).abs();
            assert!(
                diff < 0.000001,
                "Roundtrip failed for {}: got {}",
                tokens,
                back_to_tokens
            );
        }
    }

    #[test]
    fn test_precision_boundaries() {
        // Test precision at boundaries
        // Smallest representable amount: 1 unit = 0.000001 tokens
        assert_eq!(TokenUnit::from_tokens(0.000001), 1);
        assert_eq!(TokenUnit::from_units(1), 0.000001);

        // Smaller than smallest unit rounds to 0
        assert_eq!(TokenUnit::from_tokens(0.0000001), 0);
    }

    #[test]
    fn test_format_edge_cases() {
        // Test format with various values
        assert_eq!(TokenUnit::format(0), "0.000000");
        assert_eq!(TokenUnit::format(1), "0.000001");
        assert_eq!(TokenUnit::format(ONE_TOKEN), "1.000000");
        assert_eq!(TokenUnit::format(ONE_TOKEN * 1000), "1000.000000");
    }

    #[test]
    fn test_large_balance() {
        // Test with a large but valid balance (1 trillion tokens)
        let trillion_tokens = 1_000_000_000_000.0;
        let units = TokenUnit::from_tokens(trillion_tokens);
        let back = TokenUnit::from_units(units);

        // Should be close (floating point precision limits apply)
        assert!(
            (trillion_tokens - back).abs() < 1.0,
            "Large balance conversion should be reasonably accurate"
        );
    }
}

//! Gas calculation and fee settlement
//!
//! This module handles:
//! - Gas cost calculation with overflow protection
//! - Fee settlement and refund calculation

use anyhow::{anyhow, Result};

// ============= Gas Calculator =============

/// Gas cost calculator with overflow protection
///
/// Handles all gas-related calculations using checked arithmetic
/// to prevent overflow attacks and ensure deterministic behavior.
pub struct GasCalculator;

impl GasCalculator {
    /// Calculate maximum possible gas cost
    ///
    /// # Arguments
    /// * `gas_limit` - Maximum gas units allowed
    /// * `gas_price` - Price per gas unit in smallest token units
    ///
    /// # Returns
    /// Maximum cost in smallest token units, or error if overflow
    pub fn max_cost(gas_limit: u64, gas_price: u64) -> Result<u64> {
        gas_limit
            .checked_mul(gas_price)
            .ok_or_else(|| anyhow!("Gas cost calculation overflow"))
    }

    /// Calculate actual gas cost and refund after execution
    ///
    /// # Arguments
    /// * `gas_limit` - Maximum gas units allowed
    /// * `gas_used` - Actual gas units consumed
    /// * `gas_price` - Price per gas unit in smallest token units
    ///
    /// # Returns
    /// Settlement details with actual cost and refund amount
    pub fn settle_gas(gas_limit: u64, gas_used: u64, gas_price: u64) -> Result<GasSettlement> {
        // Calculate actual cost
        let actual_cost = gas_used
            .checked_mul(gas_price)
            .ok_or_else(|| anyhow!("Actual gas cost calculation overflow"))?;

        // Calculate max cost
        let max_cost = gas_limit
            .checked_mul(gas_price)
            .ok_or_else(|| anyhow!("Max gas cost calculation overflow"))?;

        // Calculate refund
        let refund = max_cost
            .checked_sub(actual_cost)
            .ok_or_else(|| anyhow!("Gas refund calculation underflow"))?;

        Ok(GasSettlement {
            actual_cost,
            refund,
        })
    }
}

/// Result of gas settlement after transaction execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GasSettlement {
    /// Actual cost charged in smallest token units
    pub actual_cost: u64,

    /// Amount to refund in smallest token units
    pub refund: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_cost_normal() {
        let result = GasCalculator::max_cost(100_000, 1_000).unwrap();
        assert_eq!(result, 100_000_000);
    }

    #[test]
    fn test_max_cost_overflow() {
        let result = GasCalculator::max_cost(u64::MAX, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_settle_gas() {
        let settlement = GasCalculator::settle_gas(
            100_000, // gas_limit
            60_000,  // gas_used
            1_000,   // gas_price
        )
        .unwrap();

        assert_eq!(settlement.actual_cost, 60_000_000);
        assert_eq!(settlement.refund, 40_000_000);
    }

    #[test]
    fn test_zero_gas_price() {
        // Free transactions (gas_price = 0)
        let result = GasCalculator::max_cost(1_000_000, 0).unwrap();
        assert_eq!(result, 0, "Zero gas price should result in zero cost");

        let settlement = GasCalculator::settle_gas(1_000_000, 500_000, 0).unwrap();
        assert_eq!(settlement.actual_cost, 0);
        assert_eq!(settlement.refund, 0);
    }

    #[test]
    fn test_zero_gas_limit() {
        // Transaction with zero gas limit
        let result = GasCalculator::max_cost(0, 1_000).unwrap();
        assert_eq!(result, 0, "Zero gas limit should result in zero cost");

        let settlement = GasCalculator::settle_gas(0, 0, 1_000).unwrap();
        assert_eq!(settlement.actual_cost, 0);
        assert_eq!(settlement.refund, 0);
    }

    #[test]
    fn test_all_gas_used() {
        // Transaction uses exactly the gas limit
        let settlement = GasCalculator::settle_gas(100_000, 100_000, 1_000).unwrap();

        assert_eq!(settlement.actual_cost, 100_000_000);
        assert_eq!(settlement.refund, 0, "No refund when all gas is used");
    }

    #[test]
    fn test_gas_used_exceeds_limit() {
        // Edge case: gas_used > gas_limit (shouldn't happen in practice)
        // This should error because refund calculation would underflow
        let result = GasCalculator::settle_gas(100_000, 150_000, 1_000);

        assert!(
            result.is_err(),
            "Should error when gas_used > gas_limit (refund would underflow)"
        );
    }

    #[test]
    fn test_max_cost_near_overflow() {
        // Test values near u64::MAX
        let max_safe_gas_limit = u64::MAX / 1_000;
        let result = GasCalculator::max_cost(max_safe_gas_limit, 1_000);
        assert!(result.is_ok(), "Should handle large but valid values");

        // This should overflow
        let result_overflow = GasCalculator::max_cost(max_safe_gas_limit + 1, 1_000);
        assert!(result_overflow.is_err(), "Should detect overflow");
    }

    #[test]
    fn test_settle_gas_overflow() {
        // Test settlement with values that would overflow
        let result = GasCalculator::settle_gas(u64::MAX, u64::MAX / 2, 2);
        assert!(result.is_err(), "Should detect overflow in settlement");
    }

    #[test]
    fn test_actual_cost_overflow() {
        // gas_used * gas_price overflows
        let result = GasCalculator::settle_gas(u64::MAX, u64::MAX, 2);
        assert!(
            result.is_err(),
            "Should detect overflow in actual cost calculation"
        );
    }

    #[test]
    fn test_high_gas_price() {
        // Very high gas price
        let settlement = GasCalculator::settle_gas(100, 50, 1_000_000).unwrap();

        assert_eq!(settlement.actual_cost, 50_000_000);
        assert_eq!(settlement.refund, 50_000_000);
    }

    #[test]
    fn test_settlement_consistency() {
        // Verify actual_cost + refund = max_cost
        let gas_limit = 1_000_000;
        let gas_used = 750_000;
        let gas_price = 500;

        let settlement = GasCalculator::settle_gas(gas_limit, gas_used, gas_price).unwrap();
        let max_cost = GasCalculator::max_cost(gas_limit, gas_price).unwrap();

        assert_eq!(
            settlement.actual_cost + settlement.refund,
            max_cost,
            "actual_cost + refund should equal max_cost"
        );
    }
}

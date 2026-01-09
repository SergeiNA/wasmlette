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
}

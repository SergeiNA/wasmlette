//! Gas metering logic

const MINIMAL_DEPLOY_OPERATION_COST: u32 = 10000;
const MINIMAL_SET_OPERATION_COST: u32 = 500;
const MINIMAL_GET_OPERATION_COST: u32 = 400;
const MINIMAL_HASH_OPERATION_COST: u32 = 400;

const GET_BALANCE_OPERATION_COST: u32 = 400;
const TRANSFER_OPERATION_COST: u32 = 11000;

const SET_OPERATION_BYTE_COST: u32 = 40;
const GET_OPERATION_BYTE_COST: u32 = 30;
const HASH_OPERATION_BYTE_COST: u32 = 20;
const DEPLOY_OPERATION_BYTE_COST: u32 = 5;

const FAIL_STORE_OPERATION_COST: u32 = 200;
const FAIL_GET_BALANCE_OPERATION_COST: u32 = 200;
const FAIL_HASH_OPERATION_COST: u32 = 200;

/// Various gas meters for tracking execution costs
#[derive(Debug, Clone)]
pub enum StoreOperationType {
    Set,
    Get,
}

#[derive(Debug, Clone)]
pub struct StoreGasMeter {
    minimal_set_operation_cost: u32,
    pub minimal_get_operation_cost: u32,
    set_operation_byte_cost: u32,
    get_operation_byte_cost: u32,
    pub fail_operation_cost: u32,
}

impl StoreGasMeter {
    pub fn new() -> Self {
        StoreGasMeter {
            minimal_set_operation_cost: MINIMAL_SET_OPERATION_COST,
            minimal_get_operation_cost: MINIMAL_GET_OPERATION_COST,
            set_operation_byte_cost: SET_OPERATION_BYTE_COST,
            get_operation_byte_cost: GET_OPERATION_BYTE_COST,
            fail_operation_cost: FAIL_STORE_OPERATION_COST,
        }
    }
    pub fn operation_cost(&self, data_len: u32, operation_type: StoreOperationType) -> u32 {
        let minimal_cost = match operation_type {
            StoreOperationType::Set => self.minimal_set_operation_cost,
            StoreOperationType::Get => self.minimal_get_operation_cost,
        };
        let byte_cost = match operation_type {
            StoreOperationType::Set => self.set_operation_byte_cost,
            StoreOperationType::Get => self.get_operation_byte_cost,
        };

        minimal_cost + data_len * byte_cost
    }
}

#[derive(Debug, Clone)]
pub struct DeployGasMeter {
    minimal_operation_cost: u32,
    bytecode_cost_per_byte: u32,
    pub fail_operation_cost: u32,
}

impl DeployGasMeter {
    pub fn new() -> Self {
        DeployGasMeter {
            minimal_operation_cost: MINIMAL_DEPLOY_OPERATION_COST,
            bytecode_cost_per_byte: DEPLOY_OPERATION_BYTE_COST,
            fail_operation_cost: FAIL_STORE_OPERATION_COST,
        }
    }
    pub fn operation_cost(&self, bytecode_len: u32) -> u32 {
        self.minimal_operation_cost + bytecode_len * self.bytecode_cost_per_byte
    }
}

#[derive(Debug, Clone)]
pub struct GetBalanceGasMeter {
    pub operation_cost: u32,
    pub fail_operation_cost: u32,
}

impl GetBalanceGasMeter {
    pub fn new() -> Self {
        GetBalanceGasMeter {
            operation_cost: GET_BALANCE_OPERATION_COST,
            fail_operation_cost: FAIL_GET_BALANCE_OPERATION_COST,
        }
    }
    pub fn operation_cost(&self) -> u32 {
        self.operation_cost
    }
}

#[derive(Debug, Clone)]
pub struct TransferGasMeter {
    pub operation_cost: u32,
}

impl TransferGasMeter {
    pub fn new() -> Self {
        TransferGasMeter {
            operation_cost: TRANSFER_OPERATION_COST,
        }
    }
    pub fn operation_cost(&self) -> u32 {
        self.operation_cost
    }
}

#[derive(Debug, Clone)]
pub struct HashGasMeter {
    minimal_operation_cost: u32,
    operation_byte_cost: u32,
    pub fail_operation_cost: u32,
}

impl HashGasMeter {
    pub fn new() -> Self {
        HashGasMeter {
            minimal_operation_cost: MINIMAL_HASH_OPERATION_COST,
            operation_byte_cost: HASH_OPERATION_BYTE_COST,
            fail_operation_cost: FAIL_HASH_OPERATION_COST,
        }
    }
    pub fn operation_cost(&self, data_len: u32) -> u32 {
        self.minimal_operation_cost + data_len * self.operation_byte_cost
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Store Cost Calculator Tests ==========

    #[test]
    fn test_store_cost_calculator_get() {
        let calculator = StoreGasMeter::new();

        // Small data (10 bytes)
        let cost = calculator.operation_cost(10, StoreOperationType::Get);
        assert_eq!(
            cost,
            MINIMAL_GET_OPERATION_COST + 10 * GET_OPERATION_BYTE_COST
        );

        // Large data (1000 bytes)
        let cost = calculator.operation_cost(1000, StoreOperationType::Get);
        assert_eq!(
            cost,
            MINIMAL_GET_OPERATION_COST + 1000 * GET_OPERATION_BYTE_COST
        );

        // Zero data
        let cost = calculator.operation_cost(0, StoreOperationType::Get);
        assert_eq!(cost, MINIMAL_GET_OPERATION_COST);
    }

    #[test]
    fn test_store_cost_calculator_set() {
        let calculator = StoreGasMeter::new();

        // Small data (10 bytes)
        let cost = calculator.operation_cost(10, StoreOperationType::Set);
        assert_eq!(
            cost,
            MINIMAL_SET_OPERATION_COST + 10 * SET_OPERATION_BYTE_COST
        );

        // Large data (1000 bytes)
        let cost = calculator.operation_cost(1000, StoreOperationType::Set);
        assert_eq!(
            cost,
            MINIMAL_SET_OPERATION_COST + 1000 * SET_OPERATION_BYTE_COST
        );

        // Zero data
        let cost = calculator.operation_cost(0, StoreOperationType::Set);
        assert_eq!(cost, MINIMAL_SET_OPERATION_COST);
    }

    #[test]
    fn test_store_cost_set_more_expensive_than_get() {
        let calculator = StoreGasMeter::new();

        // Set should always be more expensive than get for same data size
        for size in [0, 10, 100, 1000] {
            let set_cost = calculator.operation_cost(size, StoreOperationType::Set);
            let get_cost = calculator.operation_cost(size, StoreOperationType::Get);
            assert!(
                set_cost > get_cost,
                "Set cost ({}) should be > get cost ({}) for size {}",
                set_cost,
                get_cost,
                size
            );
        }
    }

    #[test]
    fn test_store_fail_cost() {
        let calculator = StoreGasMeter::new();
        assert_eq!(calculator.fail_operation_cost, FAIL_STORE_OPERATION_COST);
    }

    // ========== Deploy Cost Calculator Tests ==========

    #[test]
    fn test_deploy_cost_calculator() {
        let calculator = DeployGasMeter::new();

        // Small contract (100 bytes)
        let cost = calculator.operation_cost(100);
        assert_eq!(
            cost,
            MINIMAL_DEPLOY_OPERATION_COST + 100 * DEPLOY_OPERATION_BYTE_COST
        );

        // Large contract (10KB)
        let cost = calculator.operation_cost(10_000);
        assert_eq!(
            cost,
            MINIMAL_DEPLOY_OPERATION_COST + 10_000 * DEPLOY_OPERATION_BYTE_COST
        );

        // Empty contract (edge case)
        let cost = calculator.operation_cost(0);
        assert_eq!(cost, MINIMAL_DEPLOY_OPERATION_COST);
    }

    #[test]
    fn test_deploy_fail_cost() {
        let calculator = DeployGasMeter::new();
        assert_eq!(calculator.fail_operation_cost, FAIL_STORE_OPERATION_COST);
    }

    // ========== Hash Cost Calculator Tests ==========

    #[test]
    fn test_hash_cost_calculator() {
        let calculator = HashGasMeter::new();

        // Small data (32 bytes - typical hash input)
        let cost = calculator.operation_cost(32);
        assert_eq!(
            cost,
            MINIMAL_HASH_OPERATION_COST + 32 * HASH_OPERATION_BYTE_COST
        );

        // Large data (1MB)
        let cost = calculator.operation_cost(1_000_000);
        assert_eq!(
            cost,
            MINIMAL_HASH_OPERATION_COST + 1_000_000 * HASH_OPERATION_BYTE_COST
        );

        // Zero data
        let cost = calculator.operation_cost(0);
        assert_eq!(cost, MINIMAL_HASH_OPERATION_COST);
    }

    #[test]
    fn test_hash_fail_cost() {
        let calculator = HashGasMeter::new();
        assert_eq!(calculator.fail_operation_cost, FAIL_STORE_OPERATION_COST);
    }

    // ========== Balance Cost Calculator Tests ==========

    #[test]
    fn test_get_balance_cost() {
        let calculator = GetBalanceGasMeter::new();
        assert_eq!(calculator.operation_cost(), GET_BALANCE_OPERATION_COST);
    }

    // ========== Transfer Cost Calculator Tests ==========

    #[test]
    fn test_transfer_cost() {
        let calculator = TransferGasMeter::new();
        assert_eq!(calculator.operation_cost(), TRANSFER_OPERATION_COST);
    }

    // ========== Cost Constants Sanity Tests ==========

    #[test]
    fn test_cost_constants_sanity() {
        // Verify cost hierarchy makes sense
        assert!(
            MINIMAL_DEPLOY_OPERATION_COST > MINIMAL_SET_OPERATION_COST,
            "Deploy should be more expensive than set"
        );
        assert!(
            MINIMAL_SET_OPERATION_COST > MINIMAL_GET_OPERATION_COST,
            "Set should be more expensive than get"
        );
        assert!(
            TRANSFER_OPERATION_COST > GET_BALANCE_OPERATION_COST,
            "Transfer should be more expensive than balance query"
        );
        assert!(
            SET_OPERATION_BYTE_COST > GET_OPERATION_BYTE_COST,
            "Set per-byte should be more expensive than get per-byte"
        );
    }

    #[test]
    fn test_byte_costs_are_meaningful() {
        // Verify byte costs aren't zero
        assert!(SET_OPERATION_BYTE_COST > 0);
        assert!(GET_OPERATION_BYTE_COST > 0);
        assert!(HASH_OPERATION_BYTE_COST > 0);
        assert!(DEPLOY_OPERATION_BYTE_COST > 0);
    }

    // ========== Realistic Scenario Tests ==========

    #[test]
    fn test_realistic_storage_costs() {
        let calculator = StoreGasMeter::new();

        // Typical key size: 20 bytes (address) + 8 bytes (prefix) = 28 bytes
        // Typical value size: 8 bytes (u64)
        let key_len = 28;
        let value_len = 8;
        let total_len = key_len + value_len; // 36 bytes

        let get_cost = calculator.operation_cost(total_len, StoreOperationType::Get);
        let set_cost = calculator.operation_cost(total_len, StoreOperationType::Set);

        // Verify costs are reasonable
        assert!(get_cost > 0);
        assert!(set_cost > get_cost);

        // Expected: Get = 400 + 36*30 = 1480
        // Expected: Set = 500 + 36*40 = 1940
        assert_eq!(get_cost, 1480);
        assert_eq!(set_cost, 1940);
    }

    #[test]
    fn test_realistic_deploy_cost() {
        let calculator = DeployGasMeter::new();

        // Typical small contract: ~20KB
        let bytecode_len = 20_000;
        let cost = calculator.operation_cost(bytecode_len);

        // Expected: 10000 + 20000*5 = 1,010,000
        assert_eq!(cost, 110_000);
    }

    #[test]
    fn test_realistic_hash_cost() {
        let calculator = HashGasMeter::new();

        // Hashing 256 bytes of data
        let data_len = 256;
        let cost = calculator.operation_cost(data_len);

        // Expected: 400 + 256*20 = 5,520
        assert_eq!(cost, 5_520);
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_zero_length_operations() {
        let store_calc = StoreGasMeter::new();
        let deploy_calc = DeployGasMeter::new();
        let hash_calc = HashGasMeter::new();

        // All should return minimal cost
        assert_eq!(
            store_calc.operation_cost(0, StoreOperationType::Get),
            MINIMAL_GET_OPERATION_COST
        );
        assert_eq!(
            store_calc.operation_cost(0, StoreOperationType::Set),
            MINIMAL_SET_OPERATION_COST
        );
        assert_eq!(deploy_calc.operation_cost(0), MINIMAL_DEPLOY_OPERATION_COST);
        assert_eq!(hash_calc.operation_cost(0), MINIMAL_HASH_OPERATION_COST);
    }

    #[test]
    fn test_large_data_no_overflow() {
        let calculator = StoreGasMeter::new();

        // Test with large but valid data size (1MB)
        let large_size = 1_000_000;
        let cost = calculator.operation_cost(large_size, StoreOperationType::Set);

        // Should not overflow and should be reasonable
        assert!(cost > MINIMAL_SET_OPERATION_COST);
        assert_eq!(
            cost,
            MINIMAL_SET_OPERATION_COST + large_size * SET_OPERATION_BYTE_COST
        );
    }
}

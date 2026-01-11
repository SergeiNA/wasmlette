# Wasmlette Contracts SDK - Procedural Macros

This crate provides procedural macros for writing smart contracts with less boilerplate.

## Macros

### `#[contract_init]`

Marks a function as a contract initialization function. Automatically adds `#[no_mangle]` and `pub extern "C"`.

**Before:**
```rust
#[no_mangle]
pub extern "C" fn init(initial_supply: u64) {
    // initialization code
}
```

**After:**
```rust
use wasmlette_contracts_sdk::contract_init;

#[contract_init]
fn init(initial_supply: u64) {
    // initialization code
}
```

### `#[contract_call]`

Marks a function as a contract callable function (state-changing). Automatically adds `#[no_mangle]` and `pub extern "C"`.

**Before:**
```rust
#[no_mangle]
pub extern "C" fn transfer(to: &[u8; 20], amount: u64) -> i32 {
    // transfer logic
    0
}
```

**After:**
```rust
use wasmlette_contracts_sdk::contract_call;

#[contract_call]
fn transfer(to: &[u8; 20], amount: u64) -> i32 {
    // transfer logic
    0
}
```

### `#[contract_query]`

Marks a function as a read-only query function. Automatically adds `#[no_mangle]` and `pub extern "C"`.

**Before:**
```rust
#[no_mangle]
pub extern "C" fn balance_of(address: &[u8; 20]) -> u64 {
    // query logic
    0
}
```

**After:**
```rust
use wasmlette_contracts_sdk::contract_query;

#[contract_query]
fn balance_of(address: &[u8; 20]) -> u64 {
    // query logic
    0
}
```

## Benefits

1. **Less Boilerplate** - No need to write `#[no_mangle]` and `pub extern "C"` on every function
2. **Semantic Clarity** - Function attributes clearly indicate their purpose:
   - `#[contract_init]` - Initialization (called once on deployment)
   - `#[contract_call]` - State-changing operations
   - `#[contract_query]` - Read-only queries
3. **Future Extensibility** - Easy to add features like:
   - Automatic gas tracking
   - Parameter validation
   - Return type validation
   - Access control checks

## Complete Example

```rust
#![no_std]

use wasmlette_contracts_sdk::{
    contract_call, contract_init, contract_query,
    context, storage, ADDRESS_LENGTH
};

#[contract_init]
fn init(owner: &[u8; ADDRESS_LENGTH]) {
    storage::set(b"owner", owner);
}

#[contract_call]
fn transfer(to: &[u8; ADDRESS_LENGTH], amount: u64) -> i32 {
    let from = context::get_caller();
    // transfer logic...
    0
}

#[contract_query]
fn balance_of(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    // query balance...
    0
}
```

## Implementation Details

All three macros transform the function by:
1. Adding `#[no_mangle]` attribute to prevent name mangling
2. Setting visibility to `pub`
3. Setting ABI to `extern "C"` for WASM compatibility

The macros preserve all other attributes, documentation comments, and function bodies unchanged.

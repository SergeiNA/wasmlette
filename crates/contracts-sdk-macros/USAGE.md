# Using Contract Macros - Quick Start

## Installation

The macros are automatically available when you use the `wasmlette-contracts-sdk`:

```toml
[dependencies]
wasmlette-contracts-sdk = { path = "../../../crates/contracts-sdk" }
```

## Import the Macros

```rust
use wasmlette_contracts_sdk::{contract_call, contract_init, contract_query};
```

## Quick Comparison

### Old Style (Manual)
```rust
#![no_std]

use wasmlette_contracts_sdk::{storage, context, ADDRESS_LENGTH};

#[no_mangle]
pub extern "C" fn init(initial_supply: u64) {
    storage::set(b"supply", &initial_supply.to_le_bytes());
}

#[no_mangle]
pub extern "C" fn transfer(to: &[u8; ADDRESS_LENGTH], amount: u64) -> i32 {
    let from = context::get_caller();
    // ... transfer logic
    0
}

#[no_mangle]
pub extern "C" fn balance_of(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    // ... query logic
    0
}
```

### New Style (With Macros)
```rust
#![no_std]

use wasmlette_contracts_sdk::{
    contract_call, contract_init, contract_query,
    storage, context, ADDRESS_LENGTH
};

#[contract_init]
fn init(initial_supply: u64) {
    storage::set(b"supply", &initial_supply.to_le_bytes());
}

#[contract_call]
fn transfer(to: &[u8; ADDRESS_LENGTH], amount: u64) -> i32 {
    let from = context::get_caller();
    // ... transfer logic
    0
}

#[contract_query]
fn balance_of(address: &[u8; ADDRESS_LENGTH]) -> u64 {
    // ... query logic
    0
}
```

## Macro Reference

| Macro | Purpose | When to Use |
|-------|---------|-------------|
| `#[contract_init]` | Contract initialization | Called once during deployment. Sets up initial state. |
| `#[contract_call]` | State-changing function | Modifies contract storage. Costs gas. |
| `#[contract_query]` | Read-only query | Only reads state. Cannot modify storage. |

## Key Points

1. ✅ **No need for `#[no_mangle]`** - The macro adds it automatically
2. ✅ **No need for `pub extern "C"`** - The macro handles visibility and ABI
3. ✅ **Functions can be private** - The macro makes them public for WASM export
4. ✅ **Cleaner code** - Focus on business logic, not boilerplate
5. ✅ **Better semantics** - Clear intent: init vs call vs query

## Example: Full Token Contract

See `crates/contracts-sdk/examples/simple_token/src/lib.rs` for a complete working example using all three macros.

## Migration Guide

To migrate existing contracts:

1. Add macro imports to your use statement:
   ```rust
   use wasmlette_contracts_sdk::{contract_call, contract_init, contract_query, /* ... */};
   ```

2. Replace function attributes:
   - `#[no_mangle] pub extern "C" fn init(...)` → `#[contract_init] fn init(...)`
   - `#[no_mangle] pub extern "C" fn some_call(...)` → `#[contract_call] fn some_call(...)`
   - `#[no_mangle] pub extern "C" fn some_query(...)` → `#[contract_query] fn some_query(...)`

3. Build and test:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   cargo test
   ```

That's it! Your contract will work exactly the same, but with cleaner code.

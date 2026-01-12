//! Procedural macros for wasmlette contract SDK

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Attribute macro for contract callable functions
///
/// Automatically adds:
/// - `#[no_mangle]` - Prevents name mangling for WASM exports
/// - `pub extern "C"` - Makes function use C ABI and publicly exported
///
/// # Example
///
/// ```ignore
/// use wasmlette_contracts_sdk::contract_call;
///
/// #[contract_call]
/// fn transfer(to: &[u8; 20], amount: u64) -> i32 {
///     // Your implementation
///     0
/// }
/// ```
///
/// Expands to:
///
/// ```ignore
/// #[no_mangle]
/// pub extern "C" fn transfer(to: &[u8; 20], amount: u64) -> i32 {
///     // Your implementation
///     0
/// }
/// ```
#[proc_macro_attribute]
pub fn contract_call(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    // Force the function to be `pub extern "C"`
    input.vis = syn::Visibility::Public(syn::token::Pub::default());
    input.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern::default(),
        name: Some(syn::LitStr::new("C", proc_macro2::Span::call_site())),
    });

    // Generate the expanded code with #[no_mangle]
    let expanded = quote! {
        #[no_mangle]
        #input
    };

    TokenStream::from(expanded)
}

/// Attribute macro for contract initialization functions
///
/// Same as `#[contract_call]` but semantically indicates this is an init function.
/// Init functions are called once when the contract is deployed.
///
/// # Example
///
/// ```ignore
/// use wasmlette_contracts_sdk::contract_init;
///
/// #[contract_init]
/// fn init(initial_supply: u64) {
///     // Initialize contract state
/// }
/// ```
#[proc_macro_attribute]
pub fn contract_init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Reuse the same implementation as contract_call
    contract_call(_attr, item)
}

/// Attribute macro for contract query functions (read-only)
///
/// Same as `#[contract_call]` but semantically indicates this is a read-only query.
/// Query functions should not modify state.
///
/// # Example
///
/// ```ignore
/// use wasmlette_contracts_sdk::contract_query;
///
/// #[contract_query]
/// fn balance_of(address: &[u8; 20]) -> u64 {
///     // Read balance from storage
///     0
/// }
/// ```
#[proc_macro_attribute]
pub fn contract_query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Reuse the same implementation as contract_call
    contract_call(_attr, item)
}

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(AppState)]
pub fn derive_app_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        // The generated impl.
        impl AppState for #name {
            unsafe fn state_cell() -> &'static mut OnceCell<Self> {
                static mut STATE: OnceCell<#name> = OnceCell::new();
                &mut STATE
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

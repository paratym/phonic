use proc_macro::TokenStream;
use syn::parse_macro_input;

mod deref_signal;

#[proc_macro]
pub fn impl_deref_signal(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as deref_signal::DerefSignalInput);
    deref_signal::generate_deref_signal_impl(parsed_input)
}

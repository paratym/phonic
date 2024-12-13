use proc_macro::TokenStream;
use syn::parse_macro_input;

mod utils;

mod delegate_group;
mod delegate_impl;

#[proc_macro]
pub fn delegate_group(_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(_input as delegate_group::DelegateGroupInput);
    delegate_group::gen_delegate_group(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn delegate_impl(_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(_input as delegate_impl::DelegateImplInput);
    delegate_impl::gen_delegate_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

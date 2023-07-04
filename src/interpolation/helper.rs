use proc_macro2::{TokenTree, TokenStream};
use quote::{quote, ToTokens};


pub fn parse_interpolation(interpolation: &TokenTree) -> TokenStream {
    let inner = interpolation.to_token_stream();
    quote!(#inner.into())
}
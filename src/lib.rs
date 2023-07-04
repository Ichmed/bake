use proc_macro2::{TokenStream, TokenTree};
pub use quote::quote;
pub use syn::parse2;


pub mod basic_types;
pub mod ops;
pub mod tuples;
pub mod collections;
pub mod pointers;
pub mod time;
pub mod functions;

pub use bake_macro::*;

pub mod interpolation;

pub mod util {
    pub use proc_macro2::{Literal, TokenStream, TokenTree};
    pub use syn::{parse2 as parse, parse_quote, parse_str, Path};
    pub use quote::{quote, ToTokens};
}

//TODO: https://doc.rust-lang.org/std/cell/index.html needs implementing

pub trait Bake {
    /// Return a TokenStream the produces an equivalent struct
    fn to_stream(&self) -> TokenStream;

    fn to_token_tree(&self) -> TokenTree {
        let inner = self.to_stream();
        parse2(quote!({#inner})).expect("Wrapping a stream in brackets should always be a valid tree")
    }
}

pub trait Bakeable {
    fn bake(&self) -> TokenStream;
}

// Helper trait to avoid namespace conflicts in macros 
// And circumvent incoherent impls of `Bake` on `Interpoaltabale`
impl<T: Bake> Bakeable for T {
    fn bake(&self) -> TokenStream {
        self.to_stream()
    }
}

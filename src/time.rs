use proc_macro2::TokenStream;
use quote::quote;

use crate::Bake;


impl Bake for std::time::Duration {
    fn to_stream(&self) -> TokenStream {
        let secs = self.as_secs().to_stream();
        let nanos = self.subsec_nanos().to_stream();

        quote!(std::time::Duration::new(#secs, #nanos))
    }
}

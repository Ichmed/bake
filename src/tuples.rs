use proc_macro2::TokenStream;
use quote::quote;

use crate::Bake;

impl Bake for () {
    fn to_stream(&self) -> TokenStream {
        quote!(())
    }
}

macro_rules! impl_tuple {
    ($($($T:ident)+),*) => {
        $(
            impl<$($T: Bake),*> Bake for ($($T,)*) {
                #[allow(non_snake_case)]
                fn to_stream(&self) -> TokenStream {
                    let ($($T,)*) = self;
                    $(
                        let $T = $T.to_stream();
                    )*
                    quote!(($(#$T,)*))
                }
            }
        )*
    };
}

impl_tuple!(
    A,
    A B,
    A B C,
    A B C D,
    A B C D E,
    A B C D E F,
    A B C D E F G,
    A B C D E F G H,
    A B C D E F G H I,
    A B C D E F G H I J
);

use proc_macro2::TokenStream;
use quote::quote;

use crate::Bake;

macro_rules! impl_literals {
    ($($name:ty)*) => {
        $(impl Bake for $name {
            fn to_stream(&self) -> TokenStream {
                quote!(#self)
            }
        })*
    };
}

impl_literals!(
    bool
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    f32 f64
    char &str
);

impl<T: Bake> Bake for Option<T> {
    fn to_stream(&self) -> TokenStream {
        match self {
            Self::Some(t) => {
                let inner = t.to_stream();
                quote!(Some(#inner))
            }
            None => quote!(None),
        }
    }
}

impl<T: Bake> Bake for [T] {
    fn to_stream(&self) -> TokenStream {
        let elements= self.iter().map(Bake::to_stream);
        quote!([#(#elements),*])
    }
}

impl<T: Bake, const S: usize> Bake for [T; S] {
    fn to_stream(&self) -> TokenStream {
        let elements= self.iter().map(Bake::to_stream);
        quote!([#(#elements),*])
    }
}

impl<T: Bake> Bake for Vec<T> {
    fn to_stream(&self) -> TokenStream {
        let elements = self.iter().map(Bake::to_stream);
        quote!(vec![#(#elements),*])
    }
}

impl<T: Bake, E: Bake> Bake for Result<T, E> {
    fn to_stream(&self) -> TokenStream {
        match self {
            Ok(value) => {
                let value = value.to_stream();
                quote!(#value)
            }
            Err(error) => {
                let error = error.to_stream();
                quote!(#error)
            }
        }
    }
}

impl Bake for String {
    fn to_stream(&self) -> TokenStream {
        quote!(#self.to_owned())
    }
}

impl<T: Bake> Bake for Box<T> {
    fn to_stream(&self) -> TokenStream {
        let element = self.as_ref().to_stream();
        quote!(Box::new(#element))
    }
}

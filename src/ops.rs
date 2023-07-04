use proc_macro2::TokenStream;
use quote::quote;

use crate::Bake;

impl<B: Bake, C: Bake> Bake for std::ops::ControlFlow<B, C> {
    fn to_stream(&self) -> TokenStream {
        match self {
            std::ops::ControlFlow::Continue(c) => {
                let c = c.to_stream();
                quote!(std::ops::ControlFlow::Continue(#c))
            }
            std::ops::ControlFlow::Break(b) => {
                let b = b.to_stream();
                quote!(std::ops::ControlFlow::Break(#b))
            }
        }
    }
}

impl<Idx: Bake> Bake for std::ops::Range<Idx> {
    fn to_stream(&self) -> TokenStream {
        let std::ops::Range { start, end } = self;
        let start = start.to_stream();
        let end = end.to_stream();
        quote!(#start .. #end)
    }
}

impl<Idx: Bake> Bake for std::ops::RangeFrom<Idx> {
    fn to_stream(&self) -> TokenStream {
        let std::ops::RangeFrom { start } = self;
        let start = start.to_stream();
        quote!(#start..)
    }
}

impl<Idx: Bake> Bake for std::ops::RangeTo<Idx> {
    fn to_stream(&self) -> TokenStream {
        let std::ops::RangeTo { end } = self;
        let end = end.to_stream();
        quote!( .. #end)
    }
}

impl<Idx: Bake> Bake for std::ops::RangeInclusive<Idx> {
    fn to_stream(&self) -> TokenStream {
        let start = self.start().to_stream();
        let end = self.end().to_stream();
        quote!(#start ..= #end)
    }
}

impl<Idx: Bake> Bake for std::ops::RangeToInclusive<Idx> {
    fn to_stream(&self) -> TokenStream {
        let std::ops::RangeToInclusive { end } = self;
        let end = end.to_stream();
        quote!( .. #end)
    }
}

impl<T: Bake> Bake for std::ops::Bound<T> {
    fn to_stream(&self) -> TokenStream {
        match self {
            Self::Included(t) => {
                let t = t.to_stream();
                quote!(std::ops::Bound::Included(#t))
            }
            Self::Excluded(t) => {
                let t = t.to_stream();
                quote!(std::ops::Bound::Excluded(#t))
            }
            Self::Unbounded => quote!(std::ops::Bound::Unbounded),
        }
    }
}

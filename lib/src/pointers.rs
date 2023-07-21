#[cfg(feature = "allow_pointers")]
use std::ops::Deref;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{Bake, Bakeable};

const POINTER_WARNING: &str = "Smart pointers may not be baked corectly, try to implement the baking logic for this struct yourself or enable the `allow_pointers` feature if you know what you are doing";

impl<T: Bake> Bake for std::rc::Rc<T> {
    #[cfg(feature = "allow_pointers")]
    fn bake(&self) -> TokenStream {
        let inner = self.deref().bake();
        quote!(std::rc::Rc::new(#inner))
    }

    #[cfg(not(feature = "allow_pointers"))]
    fn to_stream(&self) -> TokenStream {
        panic!("{}", POINTER_WARNING)
    }
}

impl<T: Bake> Bake for std::sync::Arc<T> {
    #[cfg(feature = "allow_pointers")]
    fn bake(&self) -> TokenStream {
        let inner = self.deref().bake();
        quote!(std::sync::Arc::new(#inner))
    }

    #[cfg(not(feature = "allow_pointers"))]
    fn to_stream(&self) -> TokenStream {
        panic!("{}", POINTER_WARNING)
    }
}

impl<T: Bake + Clone> Bake for std::borrow::Cow<'_, T> {
    #[cfg(feature = "allow_pointers")]
    fn bake(&self) -> TokenStream {
        let inner = self.deref().bake();
        quote!(std::borrow::Cow::new(#inner))
    }

    #[cfg(not(feature = "allow_pointers"))]
    fn to_stream(&self) -> TokenStream {
        panic!("{}", POINTER_WARNING)
    }
}

impl<B: Bake + Copy> Bake for std::cell::Cell<B> {
    fn to_stream(&self) -> TokenStream {
        let inner = self.get().bake();
        quote!(std::cell::Cell::new(#inner))
    }
}

impl<B: Bake> Bake for std::cell::RefCell<B> {
    fn to_stream(&self) -> TokenStream {
        let inner = self.borrow().bake();
        quote!(std::cell::Cell::new(#inner))
    }
}

impl<B: Bake> Bake for std::cell::OnceCell<B> {
    fn to_stream(&self) -> TokenStream {
        match self.get() {
            Some(inner) => {
                let inner = inner.bake();
                quote! {
                    let cell = std::cell::OnceCell::new();
                    cell.set(#inner);
                    cell
                }
            }
            None => quote!(std::cell::OnceCell::new()),
        }
    }
}

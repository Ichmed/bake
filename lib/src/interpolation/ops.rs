use quote::quote;
use syn::parse2;
use crate::{Bake, Bakeable, TokenTree, interpolation::Interpolatable};


macro_rules! impl_operator {
    ($tr:ident::$method:ident) => {
        impl<B: Bake + std::ops::$tr<Rhs, Output = B>, Rhs: Bake> std::ops::$tr<Interpolatable<Rhs>> for Interpolatable<B> {
        type Output = Self;
        fn $method(self, rhs: Interpolatable<Rhs>) -> Self::Output {
            use Interpolatable::*;

            match (self, rhs) {
                (Actual(lhs), Actual(rhs)) => Actual(lhs.$method(rhs)),
                (lhs, rhs) => {
                    let lhs = lhs.bake();
                    let rhs = rhs.bake();
                    let joined: TokenTree = parse2(quote!(#lhs.$mthod(#rhs))).unwrap();
                    Inter(joined)
                }
            }
        }
    }};
}

impl_operator!(Add::add);
impl_operator!(Sub::sub);
impl_operator!(Mul::mul);
impl_operator!(Div::div);
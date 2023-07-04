use std::{
    error::Error,
    fmt,
    fmt::{Display, Formatter},
    ops::Deref,
};

use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{parse2, parse_quote};

use crate::{Bake, Bakeable, functions::BakeableFnOnce};

pub mod ops;
pub mod helper;

#[derive(Debug, Clone)]
pub enum Interpolatable<T> {
    Interpolation(TokenTree),
    Actual(T),
}

impl<T: Bake + PartialEq> PartialEq for Interpolatable<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Actual(a), Self::Actual(b)) => a.eq(b),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct RuntimeInterpolationError(TokenTree);

impl Display for RuntimeInterpolationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime Interpolation. {}", self.0)
    }
}

impl Error for RuntimeInterpolationError {}

trait UnwrapInterpolation<T> {
    fn unwrap(self) -> T;
}

impl<T: Bake> Bakeable for Interpolatable<T> {
    fn bake(&self) -> TokenStream {
        match self {
            Interpolatable::Interpolation(tree) => {
                let stream = tree.to_token_stream();
                quote!(#stream.into())
            }
            Interpolatable::Actual(t) => t.to_stream(),
        }
    }
}

pub trait Interpolate<T> {
    fn fit(self) -> Result<T, RuntimeInterpolationError>;

    ///Panics: If T can not be transformed
    fn force_fit(self) -> T 
    where
        Self: Sized
    {
        self.fit().expect("Interpolated during runtime")
    }
}

impl<T: Bake> Interpolate<T> for Interpolatable<T> {
    fn fit(self) -> Result<T, RuntimeInterpolationError> {
        match self {
            Interpolatable::Actual(t) => Ok(t),
            Interpolatable::Interpolation(tree) => Err(RuntimeInterpolationError(tree)),
        }
    }
}

impl<'a, T: Bake> Interpolate<&'a T> for &'a Interpolatable<T> {
    fn fit(self) -> Result<&'a T, RuntimeInterpolationError> {
        match self {
            Interpolatable::Actual(t) => Ok(t),
            Interpolatable::Interpolation(tree) => Err(RuntimeInterpolationError(tree.clone())),
        }
    }
}

impl<T: Bake> Interpolate<Interpolatable<T>> for Interpolatable<T> {
    fn fit(self) -> Result<Interpolatable<T>, RuntimeInterpolationError> {
        Ok(self)
    }
}

impl<T: Bake> Interpolate<Interpolatable<T>> for T {
    fn fit(self) -> Result<Interpolatable<T>, RuntimeInterpolationError> {
        Ok(Interpolatable::Actual(self))
    }
}

impl<T: Bake> Interpolate<T> for T {
    fn fit(self) -> Result<T, RuntimeInterpolationError> {
        Ok(self)
    }
}

impl<T: Bake> From<T> for Interpolatable<T> {
    fn from(value: T) -> Self {
        Self::Actual(value)
    }
}

impl<T> From<TokenTree> for Interpolatable<T> {
    fn from(value: TokenTree) -> Self {
        Self::Interpolation(value)
    }
}

impl From<RuntimeInterpolationError> for syn::Error {
    fn from(value: RuntimeInterpolationError) -> Self {
        Self::new_spanned(value.0, "Runtime interpolation is not possible")
    }
}

pub trait IntoInterpolation
where
    Self: Sized + Bake,
{
    fn into_interpolation(self) -> Interpolatable<Self>;
}

impl<T: Bake> IntoInterpolation for T {
    fn into_interpolation(self) -> Interpolatable<Self> {
        Interpolatable::Actual(self)
    }
}

//TODO: This may be really dumb
impl<T: Bake> Deref for Interpolatable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.fit().expect("Can not deref an interpolation")
    }
}

impl<B: Bake> From<Vec<Interpolatable<B>>> for Interpolatable<Vec<B>> {
    fn from(value: Vec<Interpolatable<B>>) -> Self {
        let mut visited: Vec<B> = Vec::with_capacity(value.len());
        let mut result: Option<Vec<TokenTree>> = None;

        use Interpolatable::*;

        for element in value {
            match element {
                Interpolation(tree) => {
                    if result.is_none() {
                        let mut v = Vec::with_capacity(visited.len());
                        v.extend(visited.iter().map(|b| b.to_token_tree()));
                        result = Some(v);
                    }
                    result.as_mut().unwrap().push(tree);
                },
                Actual(item) => match result.as_mut() {
                    Some(vec) => vec.push(item.to_token_tree()),
                    None => visited.push(item),
                },
            }
        }

        match result {
            Some(list) => Interpolation(parse_quote!({vec![#(#list.into(),)*]})),
            None => Actual(visited),
        }
    }
}

impl<B: Bake, Collection: FromIterator<B>> FromIterator<Interpolatable<B>> for Interpolatable<Collection> {
    
    fn from_iter<T: IntoIterator<Item = Interpolatable<B>>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let mut visited: Vec<B> = Vec::with_capacity(iter.size_hint().1.unwrap_or_default());
        let mut result: Option<Vec<TokenTree>> = None;

        use Interpolatable::*;

        for element in iter {
            match element {
                Interpolation(tree) => {
                    if result.is_none() {
                        let mut v = Vec::with_capacity(visited.len());
                        v.extend(visited.iter().map(|b| b.to_token_tree()));
                        result = Some(v);
                    }
                    result.as_mut().unwrap().push(tree);
                },
                Actual(item) => match result.as_mut() {
                    Some(vec) => vec.push(item.to_token_tree()),
                    None => visited.push(item),
                },
            }
        }

        match result {
            Some(list) => Interpolation(parse_quote!({FromIterator::from_iter(vec![#(#list.into(),)*])})),
            None => Actual(FromIterator::from_iter(visited)),
        }
    }
}

impl<T: Bake> Interpolatable<T> {
    /// Maps an `Interpolatable<T>` to `Interpolatable<U>` by applying a function to its contents
    ///
    /// - `Actual(T)` gets mapped to `Actual(U)`
    /// - `Interpolation` gets mapped to an `Interpolation` that applies f to the content at runtime
    pub fn map<F, U: Bake>(self, f: BakeableFnOnce<F, T, U>) -> Interpolatable<U>
    where
        F: FnOnce(T) -> U,
    {
        use Interpolatable::*;
        match self {
            Actual(inner) => Actual(f.call(inner)),
            Interpolation(tree) => {
                let function_path = f.bake();
                Interpolation(parse2(quote!(#function_path(#tree.into()))).unwrap())
            },
        }
    }

    pub fn actual(self) -> Option<T> {
        match self {
            Interpolatable::Interpolation(_) => None,
            Interpolatable::Actual(t) => Some(t),
        }
    }

    pub fn tree(self) -> Option<TokenTree> {
        match self {
            Interpolatable::Interpolation(tree) => Some(tree),
            Interpolatable::Actual(_) => None,
        }
    }
}

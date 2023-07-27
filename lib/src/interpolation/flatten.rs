use std::{collections::HashMap, hash::Hash};

use crate::{Bake, Bakeable};

use super::Interpolatable;


pub trait FlattenInterpolation<T> {
    /// Converts types with an inner interpolation like 
    /// 
    /// `Vec<Interpolatable<T>>` or `(Interpolatable<A>, Interpolatable<B>)`
    /// 
    /// Into an outer Interpolation like
    /// 
    /// `Interpolatable<Vec<T>>` or `Interpolatable<(A, B)>` respectively
    /// 
    /// Implementations should take care that inner types types that only consist of `Actual`s are always 
    /// conveted into an outer Interpolation that is also an `Actual`
    /// 
    /// Note that it is never possible to convert an inner type with any `Inter` into an outer `Actual`
    fn flatten_interpolation(self) -> Interpolatable<T>;
}

impl<T: Bake> FlattenInterpolation<Vec<T>> for Vec<Interpolatable<T>> {
    fn flatten_interpolation(self) -> Interpolatable<Vec<T>> {
        self.into_iter().collect()        
    }
}

impl<K, V> FlattenInterpolation<HashMap<K, V>> for HashMap<Interpolatable<K>, Interpolatable<V>> 
where
    K: Bake + Eq + Hash,
    V: Bake
{
    fn flatten_interpolation(self) -> Interpolatable<HashMap<K, V>> {
        self.into_iter().map(|t| t.flatten_interpolation()).collect()        
    }
}

macro_rules! impl_tuple {
    ($($($T:ident)+),*) => {
        $(
            impl<$($T: Bake),*> FlattenInterpolation<($($T,)*)> for ($(Interpolatable<$T>,)*) {
                #[allow(non_snake_case)]
                fn flatten_interpolation(self) -> Interpolatable<($($T,)*)> {
                    match self {
                        ($(Interpolatable::Actual($T),)*) => {
                            Interpolatable::Actual(($($T,)*))
                        },
                        _ => {
                            let ($($T,)*) = self;
                            $(let $T = $T.bake();)*
                            Interpolatable::Inter($crate::util::parse_quote!({($(#$T.into(),)*)}))
                        }
    
                    }
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

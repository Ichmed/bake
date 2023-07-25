use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

use proc_macro2::TokenStream;
use quote::quote;

use crate::Bake;

macro_rules! impl_lists {
    ($($T:ident),*) => {
        $(impl<B: Bake> Bake for $T<B> {
            fn to_stream(&self) -> TokenStream {
                let content = self.iter().map(Bake::to_stream);
                quote!(std::collections::$T::from([#(#content),*]))
            }
        })*
    };
}

impl_lists!(VecDeque, LinkedList, BTreeSet, HashSet, BinaryHeap);

macro_rules! impl_maps {
    ($($T:ident),*) => {
        $(impl<K: Bake, V: Bake> Bake for $T<K, V> {
            fn to_stream(&self) -> TokenStream {
                let values = self.iter()
                    .map(|(k, v)| (k.to_stream(), v.to_stream()))
                    .map(|(k, v)| quote!((#k, #v)));
                
                quote!(std::collections::$T::from([#((#values)),*]))
            }
        })*
    };
}

impl_maps!(BTreeMap, HashMap);

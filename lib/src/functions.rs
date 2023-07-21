use std::marker::PhantomData;

use quote::ToTokens;
use syn::Path;

use crate::Bakeable;

/// A wrapper around an FnOnce and the corresponding path
/// 
/// Meant to be used mainly with Interpolation::map
pub struct BakeableFnOnce<F, In, Out>
where
    F: FnOnce(In) -> Out,
{
    path: Path,
    func: F,
    phantom_in: PhantomData<In>,
    phantom_out: PhantomData<Out>
}

impl<F, In, Out> Bakeable for BakeableFnOnce<F, In, Out>
where
    F: FnOnce(In) -> Out,
{
    fn bake(&self) -> proc_macro2::TokenStream {
        self.path.to_token_stream()
    }
}

impl<F, In, Out> BakeableFnOnce<F, In, Out>
where
    F: FnOnce(In) -> Out,
{
    pub fn call(self, input: In) -> Out {
        (self.func)(input)
    }

    /// To ensure consistency between the Path and the function do not call this constructor directly.
    /// 
    /// Use the bake_fn! macro instead.
    pub fn _new(func: F, path: Path) -> Self
    {
        BakeableFnOnce {
            path,
            func,
            phantom_in: PhantomData,
            phantom_out: PhantomData,
        }
    }
}

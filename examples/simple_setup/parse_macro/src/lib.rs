use bake::{interpolation::*, Bakeable, util::quote};
use lib::*;


#[proc_macro]
pub fn test_bake_macro(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use bake::*;
    Test::new(true, 5).bake().into()
}

#[proc_macro]
pub fn test_interpolate_macro_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let b: bake::util::TokenTree = syn::parse(input).unwrap();

    let result = Ipol{
        field_a: Interpolatable::Actual(10).force_fit(),
        field_b: b.into(),
    }.bake();

    quote!({let x = #result; x}).into()
}


#[proc_macro]
pub fn parse_node(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse(&input.to_string()).unwrap().bake().into()
}
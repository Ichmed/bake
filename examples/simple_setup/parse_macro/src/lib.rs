use bake::Bakeable;
use lib::json::parse_json_node;



#[proc_macro]
pub fn json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let input = input.to_string();
    let (rest, inner) = parse_json_node(&input).unwrap();

    if rest != "" {
        panic!("Unknown syntax {rest}")
    }

    let inner = inner.bake();

    bake::util::quote!(#inner).into()
}
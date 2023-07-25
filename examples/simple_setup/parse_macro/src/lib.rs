use bake::Bakeable;
use lib::json::parse_json_node;



#[proc_macro]
pub fn json(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let inner = parse_json_node(&input.to_string()).unwrap().1.bake();

    bake::util::quote!(#inner).into()
}
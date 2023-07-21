use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, AttributeArgs, Data, DeriveInput,
    Expr, Ident, NestedMeta, Path, Meta, Attribute, Visibility, Generics,
};

mod derive;
mod interpolation;

#[proc_macro_derive(Bake, attributes(interpolate, bake_via))]
pub fn derive_bake(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    derive::generate_impl(derive_input).into()
}

#[proc_macro_attribute]
pub fn bake_new(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as AttributeArgs);

    let base = input.to_token_stream();

    let fields: Vec<_> = args.iter().collect();

    let imp = determine_constructr_arguments(input.clone(), fields);

    let ident = input.ident;
    let generics = input.generics;

    quote!(

        #base

        impl #generics bake::Bake for #ident #generics {
            fn to_stream(&self) -> bake::util::TokenStream {
                #imp.into()
            }
        }
    )
    .into()
}

fn determine_constructr_arguments(
    input: DeriveInput,
    fields: Vec<&NestedMeta>,
) -> proc_macro2::TokenStream {
    let ident = input.ident;

    if fields.is_empty() {
        match input.data {
            Data::Struct(x) => {
                let fields: Vec<_> = x.fields.into_iter().map(|f| f.ident).collect();
                quote! {
                    #(let #fields = self.#fields.bake();)*
                    bake::util::quote!(
                        #ident::new(#(# #fields),*)
                    )
                }
            }
            _ => todo!(),
        }
    } else {
        let (fields, inits): (Vec<_>, Vec<_>) = fields
            .clone()
            .iter()
            .map(|field| {
                let mut iter = field.into_token_stream().into_iter();
                let name = &iter.next().unwrap();

                (
                    quote!(#name),
                    match iter.next() {
                        Some(equals) => {
                            let exp = format!("{}", iter.next().expect("literal"));
                            let exp = snailquote::unescape(exp.as_str()).expect("literal");
                            match syn::parse_str::<Expr>(exp.as_str()) {
                                Ok(exp) => quote!(#name #equals #exp),
                                Err(_) => quote!(#exp),
                            }
                        }
                        None => quote!(#name = self.#name),
                    },
                )
            })
            .unzip();
        quote! {
            #(let #inits.bake();)*
            quote::quote!(
                #ident::new(#(# #fields),*)
            )
        }
    }
}

fn find_arg<'a>(args: &'a AttributeArgs, name: &str) -> Option<&'a NestedMeta> {
    args.iter().find(|arg| match arg {
        NestedMeta::Meta(Meta::Path(x)) => x.is_ident(name),
        NestedMeta::Meta(Meta::List(x)) => x.path.is_ident(name),
        _ => false,
    })
}

#[derive(Clone)]
struct BakeInfo {
    /// Attributes tagged on the whole struct or enum.
    pub attrs: Vec<Attribute>,

    /// Visibility of the struct or enum.
    pub vis: Visibility,

    /// Name of the struct or enum.
    pub path: Path,

    /// Generics required to complete the definition.
    pub generics: Generics,

}


#[proc_macro_attribute]
pub fn bake(args: TokenStream, input: TokenStream) -> TokenStream {
    let DeriveInput { attrs, vis, ident, generics, data } = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as AttributeArgs);

    let path = ident.clone().into();

    let info = BakeInfo {
        attrs, vis, path, generics
    };

    let interpolate_all = find_arg(&args, "interpolate").is_some();


    let main_impl = match data {
            Data::Struct(data) => interpolation::interpolate_struct(info.clone(), data, interpolate_all),
            Data::Enum(data) => interpolation::interpolate_enum(info.clone(), data, interpolate_all),
            Data::Union(_) => todo!(),
    };

    let to_tokens_impl = find_arg(&args, "to_tokens").map(|_| to_tokens(info)).unwrap_or_default();

    quote!{
        #main_impl

        #to_tokens_impl
    }.into()

}

fn to_tokens(input: BakeInfo) -> proc_macro2::TokenStream {
    let BakeInfo {
        path,
        generics,
        ..
    } = input;
    
    quote!(
        impl #generics bake::util::ToTokens for #path #generics {
            fn to_tokens(&self, tokens: &mut bake::util::TokenStream) {
                tokens.extend(self.bake())
            }
        }
    )

}

// #[proc_macro_attribute]
// pub fn bake_interpolated(_args: TokenStream, input: TokenStream) -> TokenStream {
//     let derive_input = parse_macro_input!(input as DeriveInput);

//     match derive_input.data.clone() {
//         Data::Struct(data) => bake_interpolated_struct(data, derive_input),
//         Data::Enum(data) => bake_interpolated_enum(data, derive_input),
//         Data::Union(_) => panic!("Not allowed on unions"),
//     }
// }

#[proc_macro_attribute]
pub fn ct_parser(args: TokenStream, input: TokenStream) -> TokenStream {
    let macro_name: Ident = syn::parse(args).unwrap();

    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident.clone();
    let struc = input.into_token_stream();

    quote!(
        #[proc_macro]
        pub fn #macro_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            quote::quote!(
                <#type_name as FromStr>::from_str(stringify!(input)).unwrap().bake()
            ).into()
        }

        #struc
    )
    .into()
}

#[proc_macro]
/// Construct a new BakeableFnOnce
pub fn bake_fn_once(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as Path);

    quote!({
        bake::functions::BakeableFnOnce::_new(#path, bake::util::parse_quote!(#path))
    }).into()
}
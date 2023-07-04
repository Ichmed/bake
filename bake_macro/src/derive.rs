use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{DeriveInput, Visibility, Data, DataStruct, Fields, DataEnum, NestedMeta, Meta, Attribute, AttributeArgs, Path, Field, Type, TypePath};

use crate::find_arg;


fn get_attrs(attrs: &[Attribute], name: &str) -> AttributeArgs {
    
    attrs.iter()
        .find(|x| x.path.is_ident(name))
        .and_then(|x| x.parse_meta().ok())
        .and_then(|x| match x {
            Meta::List(x) => Some(x),
            _ => None,
        })
        
        .map(|x| x.nested.into_iter().collect())
        .unwrap_or_default()

}


pub fn generate_impl(derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = derive_input;


    let args = get_attrs(&attrs, "bake");

    
    let alias = if let Some(NestedMeta::Meta(Meta::List(alias))) = find_arg(&args, "bake_as") {
        match alias.nested.first().cloned() {
            Some(NestedMeta::Meta(Meta::Path(x))) => Some(x),
            _ => None,
        }
    } else {
        None
    };
    

    if !matches!(vis, Visibility::Public(_)) {
        panic!("Can only be used on public Types")
    }

    let imp = match data {
        Data::Struct(x) => inplace_struct(&ident, alias, x),
        Data::Enum(x) => inplace_enum(&ident, alias, x),
        Data::Union(_) => todo!(),
    };


    quote! {
        impl #generics bake::Bake for #ident #generics {
            fn to_stream(&self) -> bake::util::TokenStream {
                let object = self;
                #imp.into()
            }
        }
    }
}

fn inplace_struct(ident: &Ident, alias: Option<Path>, data: DataStruct) -> proc_macro2::TokenStream {
    let destructured = destructure(&data.fields);
    let conversion = convert(&data.fields);
    let restructured = restructure(&data.fields);

    match alias {
        Some(alias) => {
            quote!(
                let #ident #destructured = object;

                #conversion
                bake::util::quote!(
                    #alias #restructured
                )
            )   
        },
        None => {
            let path = Ident::new("__path", Span::call_site());

            quote!(
                let #ident #destructured = object;
                let #path: bake::util::Path = bake::util::parse_str(module_path!()).unwrap();

                #conversion
                bake::util::quote!(
                    ##path :: #ident #restructured
                )
            )            
        },
    }

}

/// Creates a destructuring for the given fields without the type
/// so
///
/// `{a, b}`
///
/// for
/// ```
/// struct A {
///     a,
///     b
/// }
/// ```
fn destructure(fields: &Fields) -> proc_macro2::TokenStream {
    let idents = determine_idents(fields);

    match fields {
        Fields::Named(_) => quote!({ #(#idents),* }),
        Fields::Unnamed(_) => quote!( ( #(#idents),* ) ),
        Fields::Unit => quote!(),
    }
}

fn convert(fields: &Fields) -> proc_macro2::TokenStream {
    let conversions = match fields {
        Fields::Named(fields) => fields.named.iter().collect(),
        Fields::Unnamed(fields) => fields.unnamed.iter().collect(),
        Fields::Unit => Vec::new(),
    }.into_iter().enumerate().map(field_conversion);
    quote! {
        #(#conversions)*
    }
}

fn restructure(fields: &Fields) -> proc_macro2::TokenStream {
    
    
    let idents = determine_idents(fields);

    match fields {
        Fields::Named(_) => quote!({ #(#idents: # #idents),* }),
        Fields::Unnamed(_) => quote!( ( #(# #idents),* ) ),
        Fields::Unit => quote!(),
    }
}

fn get_path(attrs: &[Attribute], name: &str) -> Option<Path> {
    let attrs = get_attrs(attrs, name);
    attrs
        .first()
        .and_then(|x| match x {
            NestedMeta::Meta(Meta::Path(path)) => Some(path),
            _ => None,
        })
        .cloned()
}

fn field_conversion((index, field): (usize, &Field)) -> proc_macro2::TokenStream {    
    let bake_as = get_path(&field.attrs, "bake_via");

    let ident = field.ident.clone().unwrap_or_else(|| Ident::new(format!("x_{}", index).as_str(), Span::call_site()));

    if let Some(alias) = bake_as.clone() {
        if field.ty == Type::Path(TypePath { qself: None, path: alias }) {
            let u = &field.ty;
            return quote!(let #ident = quote!(#u););
        }
    }


    match bake_as {
        Some(alias) => quote!(let #ident = Into::<#alias>::into(#ident).bake();),
        None => quote!(let #ident = #ident.bake();),
    }

}


fn determine_idents(fields: &Fields) -> Vec<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|x| x.ident.as_ref().unwrap().to_token_stream())
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _)| format!("x_{i}").parse().unwrap())
            .collect(),
        Fields::Unit => vec![],
    }
}

fn inplace_enum(ident: &Ident, alias: Option<Path>, data: DataEnum) -> proc_macro2::TokenStream {
    let variants = enum_variants(ident, alias, &data);

    quote!(match object {
        #(#variants),*
    })
}

fn enum_variants(ident: &Ident, alias: Option<Path>, data: &DataEnum) -> Vec<proc_macro2::TokenStream> {
    data.variants
        .iter()
        .map(|variant| {
            let destructured = destructure(&variant.fields);
            let conversion = convert(&variant.fields);
            let restructured = restructure(&variant.fields);
            let var_ident = &variant.ident;

            match alias.as_ref() {
                Some(alias) => quote! {
                    Self :: #var_ident #destructured => {
                        #conversion
    
                        quote!(#alias :: #var_ident #restructured)
                    }                

                },
                None => {
                    let path = Ident::new("__path", Span::call_site());
                    quote! {
                        Self :: #var_ident #destructured => {
                            #conversion
                            let #path: bake::util::Path = bake::util::parse_str(module_path!()).unwrap();

                            quote!(##path :: #ident :: #var_ident #restructured)
                        }
                    }
                },
            }

            
        })
        .collect()
}
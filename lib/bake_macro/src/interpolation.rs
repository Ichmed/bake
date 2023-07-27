use quote::quote;
use syn::{Attribute, DataEnum, DataStruct, Field, Fields, LitBool, Variant};

use crate::BakeInfo;

pub(crate) fn interpolate_struct(
    input: BakeInfo,
    data: DataStruct,
    interpolate_all: bool,
) -> proc_macro2::TokenStream {
    let BakeInfo {
        attrs,
        vis,
        path,
        generics,
        ..
    } = input;

    let imp = interpolate_struct_content(data.fields, interpolate_all);

    quote!(
        #(#attrs)*
        #vis struct #path #generics #imp
    )
}

pub(crate) fn interpolate_enum(input: BakeInfo, data: DataEnum, interpolate_all: bool) -> proc_macro2::TokenStream {
    let BakeInfo {
        attrs,
        vis,
        path,
        generics,
        ..
    } = input;

    let DataEnum {
        enum_token: _,
        brace_token: _,
        variants: variants_base,
    } = data;

    let inner: Vec<_> = variants_base
        .into_iter()
        .map(|variant| {
            let Variant {
                mut attrs,
                ident,
                fields,
                discriminant,
            } = variant;

            let discriminant = discriminant.map(|(eq, expr)| quote!(#eq #expr));

            let inner =
                interpolate_struct_content(fields, should_interpolate(&mut attrs, interpolate_all));
            quote!(
                #(#attrs)*
                #ident #inner #discriminant,
            )
        })
        .collect();

    quote! {
        #(#attrs)* #vis enum #path #generics {
            #(#inner)*
        }
    }
}

fn interpolate_struct_content(fields: Fields, interpolate_all: bool) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let fields: Vec<_> = fields
                .named
                .into_iter()
                .map(|field| interpolate_named_field(field, interpolate_all))
                .collect();
            quote!({#(#fields)*})
        }
        syn::Fields::Unnamed(fields) => {
            let fields: Vec<_> = fields
                .unnamed
                .into_iter()
                .map(|field| interpolate_unnamed_field(field, interpolate_all))
                .collect();
            quote!((#(#fields)*))
        }
        syn::Fields::Unit => quote!(),
    }
}

fn should_interpolate(attrs: &mut Vec<Attribute>, interpolate_all: bool) -> bool {
    if let Some(index) = attrs
        .iter()
        .position(|attr| attr.path.is_ident("interpolate"))
    {
        let att = attrs.swap_remove(index);

        match att.parse_meta().unwrap() {
            syn::Meta::Path(_) => true,
            syn::Meta::List(_) => {
                att.parse_args::<LitBool>()
                    .expect("Only boolean arguments are allowed")
                    .value
            }
            syn::Meta::NameValue(named) => match named.lit {
                syn::Lit::Bool(LitBool { value, .. }) => value,
                _ => panic!("Only boolean arguments are allowed"),
            },
        }
    } else {
        interpolate_all
    }
}

fn interpolate_named_field(field: Field, interpolate_all: bool) -> proc_macro2::TokenStream {
    let Field {
        mut attrs,
        ident,
        ty,
        vis,
        ..
    } = field;

    if should_interpolate(&mut attrs, interpolate_all) {
        quote! {
            #(#attrs)*
            #[cfg(feature = "macro")]
            #vis #ident : struct_baker::interpolation::Interpolatable<#ty>,
            #[cfg(not(feature = "macro"))]
            #vis #ident : #ty,
        }
    } else {
        quote! {
            #(#attrs)*
            #vis #ident : #ty,
        }
    }
}

fn interpolate_unnamed_field(field: Field, interpolate_all: bool) -> proc_macro2::TokenStream {
    let Field {
        mut attrs, ty, vis, ..
    } = field;

    if should_interpolate(&mut attrs, interpolate_all) {
        quote! {
            #(#attrs)*
            #[cfg(feature = "macro")]
            #vis struct_baker::interpolation::Interpolatable<#ty>,
            #[cfg(not(feature = "macro"))]
            #vis #ty,
        }
    } else {
        quote! {
            #(#attrs)*
            #vis #ty,
        }
    }
}

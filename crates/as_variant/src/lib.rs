use convert_case::{Case, Casing};
use quote::{format_ident, quote, quote_spanned};

#[proc_macro_derive(AsVariant)]
pub fn as_variant_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // 1. assert that it is an enum
    // 2. For each variant field with data, generate an `as_` method
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let syn::Data::Enum(data) = input.data else {
        // compile_error!("Only enums are supported");
        let span = input.ident.span();
        return quote_spanned! {
            span => compile_error!("Only enums are supported");
        }
        .into();
        // panic!("Only enums are supported");
    };

    let variants = data.variants;
    let enum_name = input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut output = proc_macro2::TokenStream::new();

    let mut as_variant_methods = proc_macro2::TokenStream::new();

    for variant in variants {
        let syn::Fields::Unnamed(_) = variant.fields else {
            continue;
        };
        let variant_ident = &variant.ident;

        let (data_pattern, ret_value, data_types) = get_field_info(&variant.fields);
        let pattern = quote! { #enum_name :: #variant_ident #data_pattern };

        let variant_name = &variant.ident;

        let fn_name = format_ident!(
            "as_{}",
            variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span()
        );

        let mut_fn_name = format_ident!(
            "as_{}_mut",
            variant.ident.to_string().to_case(Case::Snake),
            span = variant.ident.span()
        );

        // TODO:
        let doc_comment = format!(" Return a reference to `{enum_name}::{variant_ident}` variant.");
        let doc_if_none = " Return `None` if this value os of any other type";

        let some_value = if data_types.len() == 1 {
            quote! { #(&#data_types),* }
        } else {
            quote! { (#(&#data_types),*) }
        };

        let as_ = quote! {
            #[inline]
            #[track_caller]
            #[doc = #doc_comment]
            #[doc = #doc_if_none]
            // fn #fn_name(&self) -> Option<&#data.unnamed> {
            // fn #fn_name(&self) -> Option<&#data_pattern> {
            // fn #fn_name(&self) -> Option<#(&#data_types),*> {
            pub const fn #fn_name(&self) -> Option<#some_value> {
                // match self {
                //     #pattern => Some(#ret_value),
                //     _ => None,
                // }
                if let Self::#variant_name(ref inner) = self {
                    Some(inner)
                } else {
                    None
                }
            }
        };

        as_variant_methods.extend(as_);

        {
            let some_value = if data_types.len() == 1 {
                quote! { #(&mut #data_types),* }
            } else {
                quote! { (#(&mut #data_types),*) }
            };

            let doc_comment = format!(" Return a mutable reference to `{enum_name}::{variant_ident}` variant.");
            let as_mut_ = quote! {
               #[inline]
               #[track_caller]
               #[doc = #doc_comment]
               #[doc = #doc_if_none]
               // fn #fn_name(&self) -> Option<&#data.unnamed> {
               // fn #fn_name(&self) -> Option<&#data_pattern> {
               // fn #fn_name(&self) -> Option<#(&#data_types),*> {
               pub fn #mut_fn_name(&mut self) -> Option<#some_value> {
                   if let Self::#variant_name(ref mut inner) = self {
                       Some(inner)
                   } else {
                       None
                   }
               }
            };

            as_variant_methods.extend(as_mut_);
        }
    }

    output.extend(quote! {
        #[automatically_derived]
        impl #impl_generics #enum_name #type_generics #where_clause {
            #as_variant_methods
        }
    });

    output.into()
}

// #[derive(AsVariant)]
// enum Example {
//     #[as_variant(ignore)]
//     A(usize)
//     B(bool, [usize; 4]),
//     C,
//     D { a: u32, b: u32 }
// }

// Example::as_b();
// Example::as_b_mut();

// https://github.com/JelteF/derive_more/blob/e0d169887d1d5f026be077a9458467e970a168f9/impl/src/unwrap.rs#L130-L144
fn get_field_info(fields: &syn::Fields) -> (proc_macro2::TokenStream, proc_macro2::TokenStream, Vec<&syn::Type>) {
    match fields {
        syn::Fields::Named(_) => panic!("cannot unwrap anonymous records"),
        syn::Fields::Unnamed(ref fields) => {
            let (idents, types) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(n, it)| (format_ident!("field_{n}"), &it.ty))
                .unzip::<_, _, Vec<_>, Vec<_>>();
            (quote! { (#(#idents),*) }, quote! { (#(#idents),*) }, types)
        }
        syn::Fields::Unit => (quote! {}, quote! { () }, vec![]),
    }
}

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned, Field, ImplItemFn};

pub fn impl_updatable(name: &Ident, fields: &Vec<&Field>) -> TokenStream {
    let update_name = gen_update_name(name);
    let update_fields = gen_update_fields(fields);
    let builder_fns = gen_update_builder_fns(fields);
    let with_name = format_ident!("with");
    let update_fn_body = gen_update_fn_body(fields, &with_name);

    quote! {
        #[derive(std::default::Default, std::fmt::Debug, serde::Serialize, serde::Deserialize, core::clone::Clone)]
        pub struct #update_name {
            #(#update_fields),*
        }

        impl #update_name {
            #(#builder_fns)*
        }

        impl fullstack_entity::Updatable<#update_name> for #name {
            fn update(&mut self, #with_name: &#update_name) {
                #update_fn_body
            }
        }
    }
}

pub fn gen_update_name(name: &Ident) -> Ident {
    format_ident!("Updated{}", name)
}

fn gen_update_fields(fields: &Vec<&Field>) -> Vec<Field> {
    fields
        .iter()
        .map(|&f| {
            let mut update_field = f.clone();
            let original_type = &f.ty;
            update_field.ty = parse_quote! { std::option::Option<#original_type> };
            update_field
                .attrs
                .push(parse_quote!(#[serde(skip_serializing_if = "std::option::Option::is_none")]));
            update_field
        })
        .collect()
}

fn gen_update_builder_fns(fields: &Vec<&Field>) -> Vec<ImplItemFn> {
    fields
        .iter()
        .map(|&f| {
            let name = &f.ident;
            let ty = &f.ty;
            parse_quote_spanned! {f.span()=>
                pub fn #name(mut self, val: #ty) -> Self {
                    self.#name = core::option::Option::Some(val);
                    self
                }
            }
        })
        .collect()
}

fn gen_update_fn_body(fields: &Vec<&Field>, with_name: &Ident) -> TokenStream {
    let lines = fields.iter().map(|&f| {
        let id = &f.ident;
        quote_spanned! {f.ty.span()=>
            fullstack_entity::Updatable::update(&mut self.#id, &#with_name.#id);
        }
    });
    quote! { #(#lines)* }
}

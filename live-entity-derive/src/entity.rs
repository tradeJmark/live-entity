use super::updatable::gen_update_name;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Attribute, Error, Expr, Field, FieldsNamed, Lit, LitStr};

pub fn impl_entity(
    name: &Ident,
    name_str: &LitStr,
    id_field: &Field,
    other_fields: &Vec<&Field>,
) -> TokenStream {
    let update_name = gen_update_name(name);
    let id_name = &id_field.ident;
    let id_type = &id_field.ty;
    let mut output = impl_eq_for_entity(name, id_field);
    output.extend(impl_into_update(&name, &update_name, other_fields));
    output.extend(quote! {
        impl live_entity::Entity for #name {
            type Update = #update_name;
            type ID = #id_type;
            const TYPE_NAME: &'static str = #name_str;

            fn get_id(&self) -> &Self::ID {
                &self.#id_name
            }
        }
    });
    output
}

fn impl_into_update(name: &Ident, update_name: &Ident, fields: &Vec<&Field>) -> TokenStream {
    let update_var = format_ident!("update");
    let field_copies = fields.iter().map(|&f| {
        let name = f.ident.as_ref();
        quote_spanned! {f.span()=> #update_var.#name = std::option::Option::Some(self.#name); }
    });
    quote! {
        impl std::convert::Into<#update_name> for #name {
            fn into(self) -> #update_name {
                let mut #update_var = #update_name::default();
                #(#field_copies)*
                update
            }
        }
    }
}

fn impl_eq_for_entity(name: &Ident, id_field: &Field) -> TokenStream {
    let id_ident = &id_field.ident;
    quote_spanned! {id_field.span()=>
        impl core::cmp::PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                self.#id_ident == other.#id_ident
            }
        }
        impl core::cmp::Eq for #name {}
    }
}

pub fn extract_id_field(fields: &FieldsNamed) -> (Result<&Field, Error>, Vec<&Field>) {
    let (ids, others): (Vec<_>, Vec<_>) = fields
        .named
        .iter()
        .partition(|f| f.attrs.iter().any(|a| a.path().is_ident("entity_id")));
    let id = ids.into_iter().next().ok_or(Error::new(
        fields.span(),
        "No ID field specified for Entity.",
    ));
    (id, others)
}

pub fn get_name_str(attrs: &Vec<Attribute>, name_span: Span) -> Result<&LitStr, Error> {
    attrs
        .iter()
        .find(|&a| a.path().is_ident("entity_name"))
        .ok_or(Error::new(name_span, "No name specified for entity."))
        .and_then(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                Expr::Lit(lit_expr) => match &lit_expr.lit {
                    Lit::Str(lit) => Ok(lit),
                    _ => Err(Error::new(name_span, "Malformed entity_name attribute.")),
                },
                _ => Err(Error::new(name_span, "Malformed entity_name attribute.")),
            },
            _ => Err(Error::new(name_span, "Malformed entity_name attribute.")),
        })
}

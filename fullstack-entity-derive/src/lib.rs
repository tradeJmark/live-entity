use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{Data, DataStruct, DeriveInput, Error, Field, Fields, FieldsNamed, ImplItemFn, parse_macro_input, parse_quote, parse_quote_spanned};
use syn::spanned::Spanned;

#[proc_macro_derive(Entity, attributes(entity_id))]
pub fn derive_entity(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;
    let fields = named_fields_of_struct(input_as_struct(&input));
    let (id, other_fields) = extract_id_field(fields);

    let mut output = impl_updatable(name, &other_fields);

    output.extend(match id {
        Ok(id_field) => impl_entity(name, id_field).into(),
        Err(e) => e.into_compile_error()
    });
    output.into()
}

#[proc_macro_derive(Updatable)]
pub fn derive_updatable(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;
    let fields = named_fields_of_struct(input_as_struct(&input)).named.iter().collect();
    impl_updatable(name, &fields).into()
}

fn impl_entity(name: &Ident, id_field: &Field) -> TokenStream {
    let update_name = gen_update_name(name);
    let id_name = &id_field.ident;
    let id_type = &id_field.ty;
    let mut output = impl_eq_for_entity(name, id_field);
    output.extend(quote! {
        impl fullstack_entity::Entity for #name {
            type Update = #update_name;
            type ID = #id_type;

            fn get_id(&self) -> &Self::ID {
                &self.#id_name
            }
        }
    });
    output
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

fn impl_updatable(name: &Ident, fields: &Vec<&Field>) -> TokenStream {
    let update_name = gen_update_name(name);
    let update_fields = gen_update_fields(fields);
    let builder_fns = gen_update_builder_fns(fields);
    let with_name = format_ident!("with");
    let update_fn_body = gen_update_fn_body(fields, &with_name);

    quote! {
        #[derive(Default)]
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

fn gen_update_name(name: &Ident) -> Ident {
    format_ident!("Updated{}", name)
}

fn gen_update_fields(fields: &Vec<&Field>) -> Vec<Field> {
    fields.iter()
        .map(|&f| {
            let mut update_field = f.clone();
            let original_type = &f.ty;
            update_field.ty = parse_quote! { std::option::Option<#original_type> };
            update_field
        })
        .collect()
}

fn gen_update_builder_fns(fields: &Vec<&Field>) -> Vec<ImplItemFn> {
    fields.iter()
        .map(|&f| {
            let name = &f.ident;
            let ty = &f.ty;
            parse_quote_spanned! {f.span()=>
                pub fn #name(mut self, val: #ty) -> Self {
                    self.#name = core::option::Option::Some(val);
                    self
                }
            }
        }).collect()
}

fn gen_update_fn_body(fields: &Vec<&Field>, with_name: &Ident) -> TokenStream {
    let lines = fields.iter()
        .map(|&f| {
            let id = &f.ident;
            quote_spanned! {f.ty.span()=>
                fullstack_entity::Updatable::update(&mut self.#id, &#with_name.#id);
            }
        });
    quote! { #(#lines)* }
}

fn input_as_struct(input: &DeriveInput) -> &DataStruct {
    match &input.data {
        Data::Struct(s) => s,
        _ => unimplemented!("Can only derive for struct.")
    }
}

fn named_fields_of_struct(s: &DataStruct) -> &FieldsNamed {
    match &s.fields {
        Fields::Named(named) => named,
        _ => unimplemented!("Can only derive for structs with named fields.")
    }
}

fn extract_id_field(fields: &FieldsNamed) -> (Result<&Field, Error>, Vec<&Field>) {
    let (ids, others): (Vec<_>, Vec<_>) = fields.named.iter().partition(|f| {
        f.attrs.iter().any(|a| a.path().is_ident("entity_id"))
    });
    let id = ids.into_iter().next()
        .ok_or(Error::new(fields.span(), "No ID field specified for Entity."));
    (id, others)
}
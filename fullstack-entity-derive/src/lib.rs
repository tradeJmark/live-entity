mod util;

use util::*;
mod entity;
use entity::*;
mod updatable;
use updatable::*;
mod storage_wrapper;
use storage_wrapper::*;

use syn::{parse_macro_input, DeriveInput, ItemStruct};

#[proc_macro_derive(Entity, attributes(entity_id))]
pub fn derive_entity(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;
    let fields = named_fields_of_struct(input_as_struct(&input));
    let (id, other_fields) = extract_id_field(fields);

    let mut output = impl_updatable(name, &other_fields);

    output.extend(match id {
        Ok(id_field) => impl_entity(name, id_field, &other_fields).into(),
        Err(e) => e.into_compile_error(),
    });
    output.into()
}

#[proc_macro_derive(Updatable)]
pub fn derive_updatable(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;
    let fields = named_fields_of_struct(input_as_struct(&input))
        .named
        .iter()
        .collect();
    impl_updatable(name, &fields).into()
}

#[proc_macro_attribute]
pub fn storage_wrapper(
    args: proc_macro::TokenStream,
    stream: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as StorageWrapperArgs);
    let mut st = parse_macro_input!(stream as ItemStruct);
    build_storage_wrapper(&args, &mut st).into()
}

#[cfg(feature = "mongo")]
mod mongo;
#[cfg(feature = "mongo")]
use mongo::*;

#[cfg(feature = "mongo")]
#[proc_macro_derive(MongoStorage, attributes(mongo_collections))]
pub fn derive_mongo_storage(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use syn::{parse2, spanned::Spanned, Meta};

    let input = parse_macro_input!(stream as DeriveInput);
    let helper_res = get_helper(&input.attrs, input.ident.span()).and_then(|a| match &a.meta {
        Meta::List(l) => parse2(l.tokens.clone()),
        _ => Err(syn::Error::new(
            a.span(),
            "Malformed mongo_collections attribute.",
        )),
    });
    let helper = match helper_res {
        Ok(mca) => mca,
        Err(e) => return e.to_compile_error().into(),
    };
    impl_mongo_storage(&input.ident, helper).into()
}

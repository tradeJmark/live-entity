mod util;

use util::*;
mod entity;
use entity::*;
mod updatable;
use updatable::*;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Entity, attributes(entity_id, entity_name))]
pub fn derive_entity(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);
    let name = &input.ident;
    let name_str = get_name_str(&input.attrs, name.span());
    let fields = named_fields_of_struct(input_as_struct(&input));
    let (id, other_fields) = extract_id_field(fields);

    let mut output = impl_updatable(name, &other_fields);

    output.extend(match id {
        Ok(id_field) => match name_str {
            Ok(name_str) => impl_entity(name, name_str, id_field, &other_fields).into(),
            Err(e) => e.into_compile_error(),
        },
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

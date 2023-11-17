use syn::{Data, DataStruct, DeriveInput, Fields, FieldsNamed};

pub fn input_as_struct(input: &DeriveInput) -> &DataStruct {
    match &input.data {
        Data::Struct(s) => s,
        _ => unimplemented!("Can only derive for struct."),
    }
}

pub fn named_fields_of_struct(s: &DataStruct) -> &FieldsNamed {
    match &s.fields {
        Fields::Named(named) => named,
        _ => unimplemented!("Can only derive for structs with named fields."),
    }
}

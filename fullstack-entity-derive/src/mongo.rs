use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    token::{Colon, Comma},
    Attribute, LitStr, Type,
};

pub struct MongoCollectionsAttribute {
    collection_map: Vec<MongoCollectionData>,
}

impl IntoIterator for MongoCollectionsAttribute {
    type Item = MongoCollectionData;
    type IntoIter = <Vec<MongoCollectionData> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.collection_map.into_iter()
    }
}

pub struct MongoCollectionData {
    pub collection_name: LitStr,
    pub entity: Type,
}

impl Parse for MongoCollectionsAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let collection_map = input
            .parse_terminated(MongoCollectionData::parse, Comma)?
            .into_iter()
            .collect();
        Ok(Self { collection_map })
    }
}

impl Parse for MongoCollectionData {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let collection_name = input.parse()?;
        input.parse::<Colon>()?;
        Ok(Self {
            collection_name,
            entity: input.parse()?,
        })
    }
}

pub fn get_helper(attrs: &Vec<Attribute>, name_span: Span) -> Result<&Attribute, syn::Error> {
    attrs
        .iter()
        .find(|&a| a.path().is_ident(&format_ident!("mongo_collections")))
        .ok_or(syn::Error::new(
            name_span,
            "Missing mongo_collections attribute.",
        ))
}

pub fn impl_mongo_storage(name: &Ident, helper: MongoCollectionsAttribute) -> TokenStream {
    let impls = helper.into_iter().map(
        |MongoCollectionData {
             collection_name,
             entity,
         }| {
            quote! {
                impl MongoEntity for #entity {
                    const COLLECTION_NAME: &'static str = #collection_name;
                }
            }
        },
    );
    quote! {
        #(#impls)*
        impl #name {
            async fn of_mongo(connection_string: String, database_name: String, app_name: Option<String>) -> Result<Self, mongodb::error::Error> {
                let store = MongoEntityStorage::new(connection_string, database_name, app_name).await?;
                Ok(Self::new(store))
            }
        }
    }
}

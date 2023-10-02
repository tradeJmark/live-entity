use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Colon, Comma, Plus};
use syn::{parse_quote, Error, Fields, Ident, ImplItemFn, ItemStruct, Type};

pub struct StorageWrapperArgs(Vec<StorageWrapperArg>);

impl IntoIterator for StorageWrapperArgs {
    type Item = StorageWrapperArg;
    type IntoIter = <Vec<StorageWrapperArg> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct StorageWrapperArg {
    name: Ident,
    ty: Type,
}

impl Parse for StorageWrapperArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Colon>()?;
        Ok(StorageWrapperArg {
            name,
            ty: input.parse()?,
        })
    }
}

impl Parse for StorageWrapperArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = input.parse_terminated(StorageWrapperArg::parse, Comma)?;
        Ok(StorageWrapperArgs(args.into_iter().collect()))
    }
}
fn get_store_ident() -> Ident {
    format_ident!("store")
}

fn get_store_trait_name(name: &Ident) -> Ident {
    format_ident!("{}StoreType", name)
}

impl StorageWrapperArgs {
    fn gen_stores_of(&self) -> impl Iterator<Item = Type> + '_ {
        self.0.iter().map(|StorageWrapperArg { name: _, ty }| {
            parse_quote! { fullstack_entity::StoreOf<#ty, Filter = F> }
        })
    }
    fn gen_entity_functions(&self) -> impl Iterator<Item = ImplItemFn> + '_ {
        let store_ident = get_store_ident();
        self.0.iter().flat_map(move |StorageWrapperArg{ name, ty }| {
            let create_fn = format_ident!("create_{}", name);
            let update_fn = format_ident!("update_{}", name);
            let delete_fn = format_ident!("delete_{}", name);
            let delete_by_id_fn = format_ident!("delete_{}_by_id", name);
            let get_fn = format_ident!("get_{}", name);
            let get_by_id_fn = format_ident!("get_{}_by_id", name);
            let watch_fn = format_ident!("watch_{}", name);
            [
                parse_quote! {
                    pub async fn #create_fn(&self, new: &#ty) -> core::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
                        self.#store_ident.create(new).await
                    }
                },
                parse_quote! {
                    pub async fn #update_fn(&self, id: &<#ty as fullstack_entity::Entity>::ID, update: &<#ty as fullstack_entity::Entity>::Update) -> core::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
                        fullstack_entity::StoreOf::<#ty>::update(&*self.#store_ident, id, update).await
                    }
                },
                parse_quote! {
                    pub async fn #delete_fn(&self, filter: std::option::Option<&F>) -> core::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
                        fullstack_entity::StoreOf::<#ty>::delete(&*self.#store_ident, filter).await
                    }
                },
                parse_quote! {
                    pub async fn #delete_by_id_fn(&self, id: &<#ty as fullstack_entity::Entity>::ID) -> core::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
                        fullstack_entity::StoreOf::<#ty>::delete_by_id(&*self.#store_ident, id).await
                    }
                },
                parse_quote! {
                    pub async fn #get_fn(&self, filter: std::option::Option<&F>) -> core::result::Result<Vec<#ty>, std::boxed::Box<dyn std::error::Error>> {
                        self.#store_ident.get(filter).await
                    }
                },
                parse_quote! {
                    pub async fn #get_by_id_fn(&self, id: &<#ty as fullstack_entity::Entity>::ID) -> core::result::Result<#ty, std::boxed::Box<dyn std::error::Error>> {
                        fullstack_entity::StoreOf::<#ty>::get_by_id(&*self.#store_ident, id).await
                    }
                },
                parse_quote! {
                    pub async fn #watch_fn(&self, channel: tokio::sync::broadcast::Sender<fullstack_entity::Event<#ty>>, filter: std::option::Option<&F>) -> core::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
                        fullstack_entity::StoreOf::<#ty>::watch(&*self.#store_ident, channel, filter).await
                    }
                }
            ]
        })
    }
}

pub fn build_storage_wrapper(args: &StorageWrapperArgs, mut st: ItemStruct) -> TokenStream {
    let name = st.ident.clone();
    let store_ident = get_store_ident();
    let stores_of: Punctuated<_, Plus> = args.gen_stores_of().collect();
    let trt = get_store_trait_name(&st.ident);
    let mut out = quote! {
        trait #trt<F>: #stores_of {}
        impl<F, T: #stores_of> #trt<F> for T {}
    };
    let store_type: Type = parse_quote! { std::sync::Arc<dyn #trt<F>> };
    st.generics = parse_quote! { <F> };
    if let Fields::Unit = st.fields {
        st.fields = Fields::Named(parse_quote! {{
            #store_ident: #store_type
        }});
    } else {
        return Error::new(st.fields.span(), "Expected unit struct.").into_compile_error();
    }
    out.extend(st.into_token_stream());

    let fns = args.gen_entity_functions();
    out.extend(quote! {
        impl<F> #name<F> {
            pub fn new(#store_ident: impl #trt<F> + 'static) -> #name<F> {
                #name::<F> { #store_ident: std::sync::Arc::new(#store_ident) }
            }
            #(#fns)*
        }
    });
    out
}

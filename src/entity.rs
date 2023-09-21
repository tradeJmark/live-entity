use crate::Updatable;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::hash::Hash;

pub trait Entity: Serialize + DeserializeOwned + Clone + Eq + Updatable<Self::Update> {
    type Update;
    type ID: Eq + Hash;

    fn get_id(&self) -> &Self::ID;
}

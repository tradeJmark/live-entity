use crate::Updatable;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Entity:
    Serialize
    + DeserializeOwned
    + Clone
    + Eq
    + Updatable<Self::Update>
    + Send
    + Sync
    + Unpin
    + Debug
    + Into<Self::Update>
    + 'static
{
    type Update: Send + Sync + Serialize + DeserializeOwned + Debug + Clone;
    type ID: Eq + Hash + Send + Sync + DeserializeOwned + Serialize + Debug + Clone;
    const NAME: &'static str;

    fn get_id(&self) -> &Self::ID;
}

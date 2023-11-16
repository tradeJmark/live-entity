use crate::Updatable;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
pub trait IDTrait: Eq + Hash + Send + Sync + DeserializeOwned + Serialize + Debug + Clone {}
impl<T: Eq + Hash + Send + Sync + DeserializeOwned + Serialize + Debug + Clone> IDTrait for T {}

pub trait UpdateTrait: Send + Sync + Serialize + DeserializeOwned + Debug + Clone {}
impl<T: Send + Sync + Serialize + DeserializeOwned + Debug + Clone> UpdateTrait for T {}

pub trait ProtoEntity<U: UpdateTrait>: 
    Serialize
    + DeserializeOwned
    + Clone
    + Eq
    + Updatable<U>
    + Send
    + Sync
    + Unpin
    + Debug
    + 'static {}
impl<U: UpdateTrait, T: 
    Serialize
    + DeserializeOwned
    + Clone
    + Eq
    + Updatable<U>
    + Send
    + Sync
    + Unpin
    + Debug
    + 'static> ProtoEntity<U> for T {}

pub trait Entity: ProtoEntity<Self::Update>
{
    type Update: UpdateTrait;
    type ID: IDTrait;
    const TYPE_NAME: &'static str;

    fn get_id(&self) -> &Self::ID;
}

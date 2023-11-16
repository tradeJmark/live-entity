use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize, Deserialize};

use crate::{Entity, UpdateTrait, Updatable};

pub trait Singleton: Updatable<Self::Update> + Debug + Clone + Serialize + DeserializeOwned + Unpin + Send + Sync + 'static {
  type Update: UpdateTrait;
  const ENTITY_ID: &'static str;
  const TYPE_NAME: &'static str;
}

#[derive(Clone)]
pub struct SingletonEntity<S: Singleton>(pub(crate) S, String);
impl<S: Singleton> SingletonEntity<S> {
  pub fn new(singleton: S) -> Self {
    Self(singleton, S::ENTITY_ID.to_owned())
  }
}
#[derive(Clone)]
pub struct SingletonEntityUpdate<S: Singleton>(pub(crate) S::Update);
impl<S: Singleton> Debug for SingletonEntity<S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.0.fmt(f)
  }
}
impl<S: Singleton> Debug for SingletonEntityUpdate<S> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.0.fmt(f)
  }
}
impl<S: Singleton> PartialEq for SingletonEntity<S> {
  fn eq(&self, _: &Self) -> bool {
      true
  }
}
impl<S: Singleton> Eq for SingletonEntity<S> {}
impl<S: Singleton> Serialize for SingletonEntity<S> {
  fn serialize<SE>(&self, serializer: SE) -> Result<SE::Ok, SE::Error>
      where
          SE: serde::Serializer {
      self.0.serialize(serializer)
  }
}
impl<S: Singleton> Serialize for SingletonEntityUpdate<S> {
  fn serialize<SE>(&self, serializer: SE) -> Result<SE::Ok, SE::Error>
      where
          SE: serde::Serializer {
      self.0.serialize(serializer)
  }
}
impl<'de, S: Singleton> Deserialize<'de> for SingletonEntity<S> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
          D: serde::Deserializer<'de> {
      let data = S::deserialize(deserializer)?;
      Ok(SingletonEntity::new(data))
  } 
}
impl<'de, S: Singleton> Deserialize<'de> for SingletonEntityUpdate<S> {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
          D: serde::Deserializer<'de> {
      let data = S::Update::deserialize(deserializer)?;
      Ok(SingletonEntityUpdate(data))
  }
}
impl<S: Singleton> Updatable<SingletonEntityUpdate<S>> for SingletonEntity<S> {
  fn update(&mut self, with: &SingletonEntityUpdate<S>) {
      self.0.update(&with.0)
  }
}

impl<S: Singleton> Entity for SingletonEntity<S> {
  type ID = String;
  type Update = SingletonEntityUpdate<S>;
  const TYPE_NAME: &'static str = S::TYPE_NAME;
  fn get_id(&self) -> &Self::ID {
    &self.1
  }
}
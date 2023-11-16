use crate::{Entity, Singleton, SingletonEntity};

#[derive(Debug, Clone)]
pub enum Event<E: Entity> {
    Create(E),
    Update { id: E::ID, update: E::Update },
    Delete(E::ID),
}

#[derive(Clone, Debug)]
pub enum SingletonEvent<S: Singleton> {
    Create(S),
    Update(S::Update),
    Delete
}

impl<S: Singleton> From<Event<SingletonEntity<S>>> for SingletonEvent<S> {
    fn from(value: Event<SingletonEntity<S>>) -> Self {
        match value {
            Event::Create(e) => Self::Create(e.0),
            Event::Update { id: _, update } => Self::Update(update.0),
            Event::Delete(_) => Self::Delete
        }
    }
}
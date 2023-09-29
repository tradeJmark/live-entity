use crate::Entity;

#[derive(Debug, Clone)]
pub enum Event<E: Entity> {
    Create(E),
    Update { id: E::ID, update: E::Update },
    Delete(E::ID),
}

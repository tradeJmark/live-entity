/// `Updatable` types can be upgraded in place based on data given by `U` values.
pub trait Updatable<U> {
    /// Update this value with data from `with`.
    fn update(&mut self, with: &U);
}

impl<U: Clone> Updatable<Option<U>> for U {
    fn update(&mut self, with: &Option<U>) {
        if let Some(update) = with.as_ref() {
            self.clone_from(update)
        }
    }
}
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler<T> {
    pub id: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for Handler<T> {
    fn default() -> Self {
        Self {
            id: 0,
            _phantom: Default::default(),
        }
    }
}

impl<T> Clone for Handler<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handler<T> {}

impl<T> Handler<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

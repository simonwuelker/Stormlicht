use crate::dom::inheritance_match;
use std::any::TypeId;
use std::fmt::Debug;

#[derive(Debug)]
pub struct DOMPtr<T: Debug> {
    content: Box<T>,
    size: usize,
    id: TypeId,
}

pub trait SuperClassOf<T: Debug>: Debug {}

impl<T: Debug + 'static> DOMPtr<T> {
    pub fn new(content: T) -> Self {
        Self {
            content: Box::new(content),
            size: std::mem::size_of::<T>(),
            id: TypeId::of::<T>(),
        }
    }

    pub fn is_a<C: Debug + 'static>(&self) -> bool {
        inheritance_match::<C>(self.id)
    }

    pub fn upcast<S: Debug>(self) -> DOMPtr<S>
    where
        S: SuperClassOf<T>,
    {
        unsafe { std::mem::transmute(self) }
    }

    pub fn downcast<C: Debug + 'static>(self) -> Result<DOMPtr<C>, Self>
    where
        T: SuperClassOf<C>,
    {
        if self.is_a::<C>() {
            unsafe { Ok(std::mem::transmute(self)) }
        } else {
            Err(self)
        }
    }
}

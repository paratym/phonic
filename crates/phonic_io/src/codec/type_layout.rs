use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    mem::{align_of, size_of},
};

#[derive(Debug, Clone, Copy)]
pub struct TypeLayout {
    id: TypeId,
    size: usize,
    align: usize,
}

impl TypeLayout {
    pub fn of<T: 'static>() -> Self {
        Self {
            id: TypeId::of::<T>(),
            size: size_of::<T>(),
            align: align_of::<T>(),
        }
    }

    pub fn is<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.id
    }

    pub fn id(&self) -> TypeId {
        self.id
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn align(&self) -> usize {
        self.align
    }
}

impl Eq for TypeLayout {}
impl PartialEq for TypeLayout {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Hash for TypeLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

use std::cell::{Ref, RefCell, RefMut};

pub mod prelude {
    pub use super::{GetRcRef, GetRcRefMut};
}

/// A trait to get the [`Ref`] out of a [`RefCell`], waiting until it can be
/// obtained.
pub trait GetRcRef<T> {
    fn get_ref(&self) -> Ref<'_, T>;
}

impl<T> GetRcRef<T> for RefCell<T> {
    fn get_ref(&self) -> Ref<'_, T> {
        loop {
            if let Ok(reference) = self.try_borrow() {
                return reference;
            }
        }
    }
}

/// A trait to get the [`RefMut`] out of a [`RefCell`], waiting until it can be
/// obtained.
pub trait GetRcRefMut<T> {
    fn get_mut_ref(&self) -> RefMut<'_, T>;
}

impl<T> GetRcRefMut<T> for RefCell<T> {
    fn get_mut_ref(&self) -> RefMut<'_, T> {
        loop {
            if let Ok(reference) = self.try_borrow_mut() {
                return reference;
            }
        }
    }
}

use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

// Re-exports
pub use std::cell::{Ref, RefMut};
pub use std::rc::Weak as RcWeak;

/// A shared, mutable handle.
pub struct Handle<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> Handle<T> {
    pub fn new(value: T) -> Self {
        Self {
            rc: Rc::new(RefCell::new(value)),
        }
    }

    #[inline(always)]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.rc.borrow()
    }

    #[inline(always)]
    pub fn borrow_mut(&mut self) -> RefMut<'_, T> {
        self.rc.borrow_mut()
    }

    pub fn ptr_eq(&self, other: &Handle<T>) -> bool {
        Rc::ptr_eq(&self.rc, &other.rc)
    }

    /// TODO: Weak newtype so users can omit `RefCell` from `Weak<RefCell<...>>`
    pub fn downgrade(&self) -> RcWeak<RefCell<T>> {
        Rc::downgrade(&self.rc)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            rc: self.rc.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Handle").field(&*self.rc.borrow()).finish()
    }
}

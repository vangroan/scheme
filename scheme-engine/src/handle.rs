use std::cell::RefCell;
pub use std::cell::{Ref, RefMut};
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

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
}

impl<T: fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&*self.rc.borrow(), f)
    }
}

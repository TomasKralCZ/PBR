use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
    time::Instant,
};

pub struct RcMut<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> RcMut<T> {
    pub fn new(val: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(val)),
        }
    }

    pub fn get_mut(&self) -> RefMut<T> {
        self.inner.borrow_mut()
    }

    pub fn get(&self) -> Ref<T> {
        self.inner.borrow()
    }
}

impl<T> Clone for RcMut<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub fn timed_scope<R, F: FnOnce() -> R>(label: &str, fun: F) -> R {
    let start = Instant::now();

    let res = fun();

    let time = Instant::now().duration_since(start);
    println!("{label} took: {time:?}");

    res
}

// Mostly copied (with some modernisation applied) from:
// https://github.com/reem/rust-lazy/blob/master/src/single.rs

use std::cell::UnsafeCell;
use std::ptr;
use std::ops::{Deref, DerefMut};

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};

pub trait Invoke<A = (), R = ()> {
    fn invoke(self: Box<Self>, arg: A) -> R;
}

impl<A, R, F> Invoke<A, R> for F
    where F: FnOnce(A) -> R
{
    fn invoke(self: Box<F>, arg: A) -> R {
        (*self)(arg)
    }
}

struct Producer<'a, T> {
    inner: Box<Invoke<(), T> + 'a>
}

impl<'a, T> Producer<'a, T> {
    fn new<F: 'a + FnOnce() -> T>(f: F) -> Producer<'a, T> {
        Producer {
            inner: Box::new(move |()| {
                f()
            }) as Box<Invoke<(), T>>
        }
    }

    fn invoke(self) -> T {
        self.inner.invoke(())
    }
}

enum Inner<'a, T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(Producer<'a, T>)
}

pub struct Thunk<'a, T> {
    inner: UnsafeCell<Inner<'a, T>>
}

impl<'a, T> Thunk<'a, T> {
    pub fn new<F>(producer: F) -> Thunk<'a, T>
        where F: 'a + FnOnce() -> T
    {
        Thunk {
            inner: UnsafeCell::new(Unevaluated(Producer::new(producer))),
        }
    }

    pub fn evaluated<'b>(val: T) -> Thunk<'b, T> {
        Thunk { inner: UnsafeCell::new(Evaluated(val)) }
    }

    pub fn force(&self) {
        unsafe {
            match *self.inner.get() {
                Evaluated(_) => return,
                EvaluationInProgress => {
                    panic!("Thunk::force called recursively. (A Thunk tried to force itself while trying to force itself).")
                },
                Unevaluated(_) => ()
            }

            match ptr::replace(self.inner.get(), EvaluationInProgress) {
                Unevaluated(producer) => *self.inner.get() = Evaluated(producer.invoke()),
                _ => unreachable!()
            };
        }
    }

    pub fn unwrap(self) -> T {
        self.force();
        unsafe {
            match self.inner.into_inner() {
                Evaluated(val) => { val },
                _ => unreachable!()
            }
        }
    }
}

impl<'x, T> Deref for Thunk<'x, T> {
    type Target = T;

     fn deref<'a>(&'a self) -> &'a T {
        self.force();
        match unsafe { &*self.inner.get() } {
            &Evaluated(ref val) => val,
            _ => unreachable!()
        }
    }
}

impl<'x, T> DerefMut for Thunk<'x, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.force();
        match unsafe { &mut *self.inner.get() } {
            &mut Evaluated(ref mut val) => val,
            _ => unreachable!()
        }
    }
}

macro_rules! lazy {
    ($e:expr) => {
        $crate::lazy::Thunk::new(move || { $e })
    }
}

#[test]
fn force_thunk() {
    let t = lazy!(2 + 2);
    assert_eq!(*t, 4);
}

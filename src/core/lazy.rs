// Thunks copied (with some modernisation applied) from:
// https://github.com/reem/rust-lazy/blob/master/src/single.rs
// The Invoke implementation hails from:
// https://github.com/sfackler/r2d2/blob/master/src/thunk.rs
// And the lazy list is from:
// https://github.com/reem/rust-lazylist/blob/master/src/lib.rs

use std::cell::UnsafeCell;
use std::rc::Rc;
use std::iter::FromIterator;
use std::clone::Clone;
use std::ptr;
use std::ops::{Deref, DerefMut};

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};
use self::List::{Cons, Nil};

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

macro_rules! list {
    ($val:expr) => { Rc::new(lazy!($val)) }
}

macro_rules! pair {
    ($val:expr, $list:expr) => {
        list!($crate::lazy::List::Cons($val, $list))
    }
}

macro_rules! nil {
    () => { list!($crate::lazy::List::Nil) }
}

pub type RcList<'a, T> = Rc<Thunk<'a, List<T>>>;

#[derive(Clone)]
pub enum List<T: 'static> {
    Nil,
    Cons(T, RcList<'static, T>)
}

impl<T: 'static> List<T> where T: Clone {
    pub fn new() -> RcList<'static, T> {
        nil!()
    }

    pub fn singleton(val: T) -> RcList<'static, T> {
        pair!(val, nil!())
    }

    pub fn head(&self) -> Option<&T> {
        match *self {
            Cons(ref val, _) => Some(val),
            Nil => None
        }
    }

    pub fn tail(&self) -> Option<List<T>> {
        match *self {
            Cons(_, ref tail) => Some((**tail).clone()),
            Nil => None
        }
    }

    pub fn from_iter<I>(mut i: I) -> RcList<'static, T> where I: Iterator<Item = T> + 'static {
        list!({
            match i.next() {
                Some(val) => Cons(val, List::from_iter(i)),
                None => Nil
            }
        })
    }
}

#[allow(extra_requirement_in_impl)]
impl<T> FromIterator<T> for List<T> where T: Clone {
    fn from_iter<I>(iter: I) -> List<T>
        where I: IntoIterator<Item = T> + 'static
    {
        (**List::from_iter(iter.into_iter())).clone()
    }
}

pub struct Iter<'a, T: 'static> {
    current: &'a List<T>
}

impl<'a, T> Iter<'a, T> {
    fn new(list: &'a List<T>) -> Iter<'a, T> {
        Iter { current: list }
    }
}

impl<'a, T> Iterator for Iter<'a, T> where T: Clone {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match *self.current {
            Nil => None,
            Cons(ref head, ref tail) => {
                self.current = &***tail;
                Some((*head).clone())
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a List<T> where T: Clone {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(&self)
    }
}

#[test]
fn force_thunk() {
    let t = lazy!(2 + 2);
    assert_eq!(*t, 4);
}

#[test]
fn lazy_list() {
    let l = pair!(1, pair!(2, pair!(3, nil!())));
    let i = l.into_iter();
    let v: Vec<i32> = i.collect();
    assert_eq!(vec![1, 2, 3], v);
}

#[test]
fn fib_seq() {
    fn fib(n: u64) -> u64 {
        let mut n0 = 0;
        let mut n1 = 1;

        for _ in 0..n {
            let sum = n0 + n1;
            n0 = n1;
            n1 = sum;
        }

        return n0;
    }

    fn fibs() -> RcList<'static, u64> {
        fn fibs_inner(n0: u64, n1: u64) -> RcList<'static, u64> {
            pair!(n0, fibs_inner(n1, n0 + n1))
        }

        fibs_inner(0, 1)
    }

    for (i, x) in (**fibs()).into_iter().take(10).enumerate() {
        assert_eq!(x, fib(i as u64))
    }
}

use std::iter::FromIterator;
use std::fmt::{Debug, Formatter, Error};
use std::ops::Deref;
use std::sync::Arc;

pub use self::List::{Cons, Nil};

pub enum List<A> {
    Cons(A, Arc<List<A>>),
    Nil,
}

impl<A> List<A> where A: Clone {
    pub fn empty() -> Arc<List<A>> {
        Arc::new(Nil)
    }

    pub fn singleton(v: A) -> Arc<List<A>> {
        Arc::new(Cons(v, List::empty()))
    }

    pub fn from_slice(slice: &[A]) -> Arc<List<A>> {
        Arc::new(From::from(slice))
    }

    pub fn null(&self) -> bool {
        match self {
            &Nil => true,
            _ => false
        }
    }

    pub fn cons(car: A, cdr: Arc<List<A>>) -> Arc<List<A>> {
        Arc::new(Cons(car, cdr))
    }

    pub fn car<'a>(&'a self) -> Option<&'a A> {
        match self {
            &Cons(ref a, _) => Some(a),
            _ => None
        }
    }

    pub fn cdr(&self) -> Arc<List<A>> {
        match self {
            &Cons(_, ref d) => d.clone(),
            _ => List::empty()
        }
    }

    pub fn length(&self) -> usize {
        self.iter().count()
    }

    pub fn append(l: Arc<List<A>>, r: Arc<List<A>>) -> Arc<List<A>> {
        match *l {
            Nil => r,
            Cons(ref a, ref d) => List::cons(a.clone(), List::append(d.clone(), r))
        }
    }

    pub fn reverse(list: Arc<List<A>>) -> Arc<List<A>> {
        match *list {
            Nil => list.clone(),
            Cons(ref a, ref d) => List::append(List::reverse(d.clone()), List::singleton(a.clone()))
        }
    }

    pub fn iter(&self) -> ListIter<A> {
        ListIter { current: Arc::new(self.clone()) }
    }
}

impl<A> Clone for List<A> where A: Clone {
    fn clone(&self) -> Self {
        match self {
            &Nil => Nil,
            &Cons(ref a, ref d) => Cons(a.clone(), d.clone())
        }
    }
}

impl<A> Default for List<A> {
    fn default() -> Self {
        Nil
    }
}

pub struct ListIter<A> {
    current: Arc<List<A>>
}

impl<A> Iterator for ListIter<A> where A: Clone {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        match *self.current.clone() {
            Nil => None,
            Cons(ref a, ref d) => {
                self.current = d.clone();
                Some(a.clone())
            }
        }
    }
}

impl<A> IntoIterator for List<A> where A: Clone {
    type Item = A;
    type IntoIter = ListIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        ListIter { current: Arc::new(self) }
    }
}

impl<A> FromIterator<A> for List<A> where A: Clone {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=A> {
        let mut l = List::empty();
        for i in iter {
            l = List::cons(i, l)
        }
        List::reverse(l).deref().clone()
    }
}

impl<'a, A> From<&'a [A]> for List<A> where A: Clone {
    fn from(slice: &'a [A]) -> List<A> {
        slice.iter().cloned().collect()
    }
}

impl<A> Into<Vec<A>> for List<A> where A: Clone {
    fn into(self) -> Vec<A> {
        let mut v = Vec::with_capacity(self.length());
        for i in self.iter() {
            v.push(i.clone())
        }
        v
    }
}

impl<A> PartialEq for List<A> where A: PartialEq {
    fn eq(&self, other: &List<A>) -> bool {
        match (self, other) {
            (&Nil, &Nil) => true,
            (&Cons(ref a1, ref d1), &Cons(ref a2, ref d2)) if a1 == a2 => d1 == d2,
            _ => false
        }
    }
}

impl<A> Eq for List<A> where A: Eq {}

impl<A> Debug for List<A> where A: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        fn items<A>(l: &List<A>, f: &mut Formatter) -> Result<(), Error> where A: Debug {
            match l {
                &Nil => Ok(()),
                &Cons(ref a, ref d) => {
                    write!(f, ", {:?}", a)?;
                    items(d, f)
                }
            }
        }
        write!(f, "[")?;
        match self {
            &Nil => Ok(()),
            &Cons(ref a, ref d) => {
                write!(f, "{:?}", a)?;
                items(d, f)
            }
        }?;
        write!(f, "]")
    }
}

macro_rules! list {
    () => { ::list::List::empty() };

    ( $($x:expr),* ) => {{
        let mut l = ::list::List::empty();
        $(
            l = ::list::List::cons($x, l);
        )*
        ::list::List::reverse(l)
    }};
}

#[test]
fn cons_up_some_trouble() {
    let l = List::cons(1, List::cons(2, List::cons(3, List::empty())));
    assert_eq!(l, list![1, 2, 3]);
    let m = List::append(l.clone(), l.clone());
    assert_eq!(m, list![1, 2, 3, 1, 2, 3]);
    let n = List::append(l.clone(), List::reverse(l.clone()));
    assert_eq!(n, list![1, 2, 3, 3, 2, 1]);
    let o: List<i32> = vec![5, 6, 7].iter().cloned().collect();
    assert_eq!(Arc::new(o), list![5, 6, 7]);
}

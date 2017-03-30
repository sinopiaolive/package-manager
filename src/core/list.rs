use std::iter::FromIterator;
use std::rc::Rc;
use std::fmt::{Debug, Formatter, Error};
use std::ops::Deref;

pub use self::List::{Cons, Nil};

pub enum List<A> {
    Cons(A, Rc<List<A>>),
    Nil,
}

impl<A> List<A> where A: Clone {
    pub fn empty() -> Rc<List<A>> {
        Rc::new(Nil)
    }

    pub fn singleton(v: A) -> Rc<List<A>> {
        Rc::new(Cons(v, List::empty()))
    }

    pub fn null(&self) -> bool {
        match self {
            &Nil => true,
            _ => false
        }
    }

    pub fn cons(car: A, cdr: Rc<List<A>>) -> Rc<List<A>> {
        Rc::new(Cons(car, cdr))
    }

    pub fn car<'a>(&'a self) -> Option<&'a A> {
        match self {
            &Cons(ref a, _) => Some(a),
            _ => None
        }
    }

    pub fn cdr(&self) -> Rc<List<A>> {
        match self {
            &Cons(_, ref d) => d.clone(),
            _ => List::empty()
        }
    }

    pub fn append(l: Rc<List<A>>, r: Rc<List<A>>) -> Rc<List<A>> {
        match *l {
            Nil => r,
            Cons(ref a, ref d) => List::cons(a.clone(), List::append(d.clone(), r))
        }
    }

    pub fn reverse(list: Rc<List<A>>) -> Rc<List<A>> {
        match *list {
            Nil => list.clone(),
            Cons(ref a, ref d) => List::append(List::reverse(d.clone()), List::singleton(a.clone()))
        }
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
    current: Rc<List<A>>
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
        ListIter { current: Rc::new(self) }
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
    assert_eq!(Rc::new(o), list![5, 6, 7]);
}

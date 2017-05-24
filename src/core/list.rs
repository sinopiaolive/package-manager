use std::iter::{Iterator, FromIterator};
use std::fmt::{Debug, Formatter, Error};
use std::ops::Deref;
use std::sync::Arc;
use std::hash::{Hash, Hasher};

use self::ListNode::{Cons, Nil};

macro_rules! list {
    () => { ::list::List::empty() };

    ( $($x:expr),* ) => {{
        let mut l = ::list::List::empty();
        $(
            l = l.cons($x);
        )*
        l.reverse()
    }};
}

pub fn cons<A>(car: A, cdr: &List<A>) -> List<A> {
    cdr.cons(car)
}

pub enum List<A> {
    List(Arc<ListNode<A>>)
}

pub enum ListNode<A> {
    Cons(usize, A, List<A>),
    Nil,
}

impl<A> List<A> {
    pub fn empty() -> List<A> {
        List::List(Arc::new(Nil))
    }

    pub fn singleton(v: A) -> List<A> {
        List::List(Arc::new(Cons(1, v, list![])))
    }

    fn as_arc<'a>(&'a self) -> &'a Arc<ListNode<A>> {
        match self {
            &List::List(ref arc) => arc
        }
    }

    pub fn null(&self) -> bool {
        match self.as_arc().deref() {
            &Nil => true,
            _ => false
        }
    }

    pub fn cons(&self, car: A) -> List<A> {
        match self.as_arc().deref() {
            &Nil => List::singleton(car),
            &Cons(l, _, _) => List::List(Arc::new(Cons(l + 1, car, self.clone())))
        }
    }

    pub fn head<'a>(&'a self) -> Option<&'a A> {
        match self.as_arc().deref() {
            &Cons(_, ref a, _) => Some(a),
            _ => None
        }
    }

    pub fn tail(&self) -> Option<List<A>> {
        match self.as_arc().deref() {
            &Cons(_, _, ref d) => Some(d.clone()),
            &Nil => None
        }
    }

    pub fn uncons<'a>(&'a self) -> Option<(&'a A, List<A>)> {
        match self.as_arc().deref() {
            &Nil => None,
            &Cons(_, ref a, ref d) => Some((a, d.clone()))
        }
    }

    pub fn length(&self) -> usize {
        match self.as_arc().deref() {
            &Nil => 0,
            &Cons(l, _, _) => l
        }
    }
}

impl List<i32> {
    pub fn range(from: i32, to: i32) -> List<i32> {
        let mut list = List::empty();
        let mut c = to;
        while c >= from {
            list = cons(c, &list);
            c -= 1;
        }
        list
    }
}

impl<A> List<A> where A: Clone + Ord {
    pub fn insert(&self, item: &A) -> List<A> {
        match self.as_arc().deref() {
            &Nil => List::singleton(item.clone()),
            &Cons(_, ref a, ref d) => if a > item {
                cons(item.clone(), self)
            } else {
                cons(a.clone(), &d.insert(item))
            }
        }
    }
}

impl<A> List<A> where A: Clone {
    pub fn from_slice(slice: &[A]) -> List<A> {
        From::from(slice)
    }

    pub fn append(&self, r: &List<A>) -> List<A> {
        match self.as_arc().deref() {
            &Nil => r.as_ref().clone(),
            &Cons(_, ref a, ref d) => cons(a.clone(), &d.append(r.as_ref()))
        }
    }

    pub fn reverse(&self) -> List<A> {
        let mut out = List::empty();
        for i in self.iter() {
            out = out.cons(i);
        }
        out
    }

    pub fn iter(&self) -> ListIter<A> {
        ListIter { current: self.as_arc().clone() }
    }
}

impl<A> Clone for List<A> {
    fn clone(&self) -> Self {
        match self {
            &List::List(ref arc) => List::List(arc.clone())
        }
    }
}

impl<A> Default for List<A> {
    fn default() -> Self {
        List::empty()
    }
}

pub struct ListIter<A> {
    current: Arc<ListNode<A>>
}

impl<A> Iterator for ListIter<A> where A: Clone {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.clone().deref() {
            &Nil => None,
            &Cons(_, ref a, ref d) => {
                self.current = d.as_arc().clone();
                Some(a.clone())
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.current.deref() {
            &Nil => (0, Some(0)),
            &Cons(l, _, _) => (l, Some(l))
        }
    }
}

impl<A> ExactSizeIterator for ListIter<A> where A: Clone {}

impl<A> IntoIterator for List<A> where A: Clone {
    type Item = A;
    type IntoIter = ListIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A> FromIterator<A> for List<A> where A: Clone {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=A> {
        let mut l = List::empty();
        for i in iter {
            l = cons(i, &l)
        }
        l.reverse()
    }
}

impl<'a, A> FromIterator<&'a A> for List<A> where A: 'a + Clone {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item=&'a A> {
        let mut l = List::empty();
        for i in iter {
            l = cons(i.clone(), &l)
        }
        l.reverse()
    }
}

impl<'a, A> From<&'a [A]> for List<A> where A: Clone {
    fn from(slice: &'a [A]) -> List<A> {
        slice.iter().cloned().collect()
    }
}

impl<A> PartialEq for List<A> where A: PartialEq {
    fn eq(&self, other: &List<A>) -> bool {
        match (self.as_arc().deref(), other.as_arc().deref()) {
            (&Nil, &Nil) => true,
            (&Cons(l1, _, _), &Cons(l2, _, _)) if l1 != l2 => false,
            (&Cons(_, ref a1, ref d1), &Cons(_, ref a2, ref d2)) if a1 == a2 => d1 == d2,
            _ => false
        }
    }
}

impl<A> Eq for List<A> where A: Eq {}

impl<A> Hash for List<A> where A: Clone + Hash {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        for i in self.iter() {
            i.hash(state);
        }
    }
}

impl<A> AsRef<List<A>> for List<A> {
    fn as_ref(&self) -> &List<A> {
        self
    }
}

impl<A> Debug for List<A> where A: Debug {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        fn items<A>(l: &List<A>, f: &mut Formatter) -> Result<(), Error> where A: Debug {
            match l.as_arc().deref() {
                &Nil => Ok(()),
                &Cons(_, ref a, ref d) => {
                    write!(f, ", {:?}", a)?;
                    items(d, f)
                }
            }
        }
        write!(f, "[")?;
        match self.as_arc().deref() {
            &Nil => Ok(()),
            &Cons(_, ref a, ref d) => {
                write!(f, "{:?}", a)?;
                items(d, f)
            }
        }?;
        write!(f, "]")
    }
}

#[test]
fn cons_up_some_trouble() {
    let l = cons(1, &cons(2, &cons(3, &list![])));
    assert_eq!(l, list![1, 2, 3]);
    assert_eq!(l, From::from(&[1, 2, 3][..]));
    assert_eq!(3, l.length());
    assert_eq!(List::range(1, 3), l);
    let m = l.append(&l);
    assert_eq!(m, list![1, 2, 3, 1, 2, 3]);
    assert_eq!(l, list![1, 2, 3]);
    let n = l.append(&l.reverse());
    assert_eq!(n, list![1, 2, 3, 3, 2, 1]);
    assert_eq!(l, list![1, 2, 3]);
    let o: List<i32> = vec![5, 6, 7].iter().cloned().collect();
    assert_eq!(o, list![5, 6, 7]);
    assert_eq!(list![2, 4, 6].insert(&5).insert(&1).insert(&3), list![1, 2, 3, 4, 5, 6]);
    assert_eq!(3, l.iter().len());
}

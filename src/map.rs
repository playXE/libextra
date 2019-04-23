use crate::vec::VecUtils;
use std::hash::{Hash, Hasher};
use std::iter::{Enumerate, FilterMap};

//#[derive(Hash, PartialEq, Eq)]
pub struct SmallIntMap<T> {
    v: Vec<Option<T>>,
}

impl<T: Hash> Hash for SmallIntMap<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.v.hash(state);
    }
}

impl<T: PartialEq> PartialEq for SmallIntMap<T> {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<T: Eq> Eq for SmallIntMap<T> {}

impl<T: Clone> Clone for SmallIntMap<T> {
    fn clone(&self) -> SmallIntMap<T> {
        SmallIntMap { v: self.v.clone() }
    }
}

pub struct SmallIntMapIterator<'a, T> {
    front: usize,
    back: usize,
    iter: Iterator<Item = Option<T>> + 'a,
}

macro_rules! iterator {
    (impl $name:ident -> $elem:ty, $getter:ident) => {
        impl<'a, T: 'a> Iterator for $name<T> {
            type Item = (usize, &'a T);
            #[inline]
            fn next(&mut self) -> Option<$elem> {
                while self.front < self.back {
                    match self.iter.next() {
                        Some(elem) => {
                            if elem.is_some() {
                                let index = self.front;
                                self.front += 1;
                                return Some((index, elem.$getter().unwrap()));
                            }
                        }
                        _ => (),
                    }
                    self.front += 1;
                }
                None
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, Some(self.back - self.front))
            }
        }
    };
}

impl<'a, T: 'a> Iterator for SmallIntMapIterator<'a, T> {
    type Item = (usize, &'a T);
    #[inline]
    fn next(&mut self) -> Option<(usize, &'a T)> {
        while self.front < self.back {
            match self.iter.next() {
                Some(elem) => {
                    if elem.is_some() {
                        let index = self.front;
                        self.front += 1;
                        unsafe {
                            let eptr: *const T = &elem.unwrap() as *const T;
                            return Some((index, &*eptr));
                        }
                    }
                }
                _ => (),
            }
            self.front += 1;
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.back - self.front))
    }
}

//TODO
//iterator!(impl SmallIntMap -> (usize,&'a T),as_ref);

impl<T> SmallIntMap<T> {
    pub fn new() -> SmallIntMap<T> {
        SmallIntMap { v: Vec::new() }
    }

    pub fn push(&mut self, v: T) {
        self.v.push(Some(v));
    }

    pub fn pop(&mut self, key: &usize) -> Option<T> {
        if *key >= self.v.len() {
            return None;
        }
        self.v[*key].take()
    }

    pub fn is_empty(&self) -> bool {
        self.v.iter().all(|elt| elt.is_none())
    }

    pub fn len(&self) -> usize {
        self.v.count(&|elt| elt.is_some())
    }

    pub fn clear(&mut self) {
        self.v.clear();
    }

    pub fn find_mut<'a>(&'a mut self, key: &usize) -> Option<&'a mut T> {
        if *key < self.len() {
            match self.v[*key] {
                Some(ref mut value) => Some(value),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn find<'a>(&'a self, key: &usize) -> Option<&'a T> {
        if *key < self.len() {
            match self.v[*key] {
                Some(ref value) => Some(value),
                None => None,
            }
        } else {
            None
        }
    }

    pub fn contains_key(&self, key: &usize) -> bool {
        self.find(key).is_some()
    }

    pub fn insert(&mut self, key: usize, value: T) -> bool {
        let exists = self.contains_key(&key);
        let len = self.v.len();

        if len <= key {
            for _ in 0..len + key {
                self.v.push(None);
            }
        }
        self.v[key] = Some(value);
        !exists
    }

    pub fn remove(&mut self, key: &usize) -> bool {
        self.v.remove(*key).is_some()
    }

    pub fn get<'a>(&'a self, key: &usize) -> &'a T {
        self.find(key).expect("key not present")
    }

    pub fn iter<'r>(&'r self) -> ::std::slice::Iter<'r, Option<T>> {
        self.v.iter()
    }

    pub fn iter_mut<'r>(&'r mut self) -> ::std::slice::IterMut<'r, Option<T>> {
        self.v.iter_mut()
    }
}

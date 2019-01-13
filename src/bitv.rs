use std::iter::{Iterator,repeat};
use std::cmp::{min,max};



#[inline]
fn iterate_bits(base: usize, bits: usize, f: &Fn(usize) -> bool) -> bool {
    if bits == 0 {
        return true;
    }
    for i in 0..64 {
        if bits & (1 << i) != 0 {
            if !f(base + i) {
                return false;
            }
        }
    }
    return true;
}



pub fn big_mask(nbits: usize, elem: usize) -> usize {
    let rmd = nbits % 64;
    let nelemns = nbits / 64 + if rmd == 0 { 0 } else { 1 };

    if elem < nelemns - 1 || rmd == 0 {
        !0
    } else {
        (1 << rmd) - 1
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BigBitv {
    storage: Vec<usize>,
}

impl BigBitv {
    pub fn new(storage: Vec<usize>) -> BigBitv {
        BigBitv { storage }
    }

    pub fn process(&mut self, b: &BigBitv, nbits: usize, op: &Fn(usize, usize) -> usize) -> bool {
        let len = b.storage.len();
        assert_eq!(self.storage.len(), len);
        let mut changed = false;
        for (i, (a, b)) in self.storage.iter_mut().zip(b.storage.iter()).enumerate() {
            let mask = big_mask(nbits, i);
            let w0 = *a & mask;
            let w1 = *b & mask;
            let w = op(w0, w1) & mask;
            if w0 != w {
                changed = true;
                *a = w;
            }
        }
        changed
    }

    pub fn each_storage(&mut self, op: &fn(v: &mut usize) -> bool) -> bool {
        let mut each_storage = false;
        for v in self.storage.iter_mut() {
            each_storage = op(v);
        }

        each_storage
    }

    pub fn negate(&mut self) {
        for v in self.storage.iter_mut() {
            *v = !*v;
        }
    }

    pub fn union(&mut self, b: &BigBitv, nbits: usize) -> bool {
        self.process(b, nbits, &|w1, w2| w1 & w2)
    }

    pub fn intersect(&mut self, b: &BigBitv, nbits: usize) -> bool {
        self.process(b, nbits, &|w1, w2| w1 & w2)
    }

    pub fn become_(&mut self, b: &BigBitv, nbits: usize) -> bool {
        self.process(b, nbits, &|_, w| w)
    }

    pub fn difference(&mut self, b: &BigBitv, nbits: usize) -> bool {
        self.process(b, nbits, &|w1, w2| w1 & !w2)
    }

    pub fn get(&self, i: usize) -> bool {
        let w = i / 64;
        let b = i % 64;
        let x = 1 & self.storage[w] >> b;
        x == 1
    }

    pub fn set(&mut self, i: usize, x: bool) {
        let w = i / 64;
        let b = i % 64;
        let flag = 1 << b;
        self.storage[w] = if x {
            self.storage[w] | flag
        } else {
            self.storage[w] & !flag
        };
    }

    pub fn equals(&self, b: &BigBitv, nbits: usize) -> bool {
        for (i, elt) in b.storage.iter().enumerate() {
            let mask = big_mask(nbits, i);
            if mask & self.storage[i] != mask & *elt {
                return false;
            }
        }
        true
    }
}

enum Op {
    Union,
    Intersect,
    Assign,
    Difference,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BitvSet {
    size: usize,
    bitv: BigBitv,
}

pub struct BitvSetIterator<'a> {
    set: &'a BitvSet,
    next_idx: usize,
}


impl<'a> Iterator for BitvSetIterator<'a> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<usize> {
        while self.next_idx < self.set.capacity() {
            let idx = self.next_idx;
            self.next_idx += 1;
            if self.set.contains(&idx) {
                return Some(idx);
            }
        }
        return None;
    }
}
impl BitvSet {
    pub fn new() -> BitvSet {
        BitvSet {
            size: 0,
            bitv: BigBitv::new(vec![0]),
        }
    }

    pub fn common_iter<'a>(&'a self,other: &'a BitvSet) -> impl Iterator<Item=(usize, usize, usize)> + 'a {
        let min = min(self.bitv.storage.len(), other.bitv.storage.len());
        self.bitv.storage[0..min].iter().enumerate()
            .zip(repeat(&other.bitv.storage))
            .map(|((i, &w), o_store)| (i * 64, w, o_store[i]))
    }
    pub fn outlier_iter<'a>(&'a self,other: &'a BitvSet) -> impl Iterator<Item=(bool,usize,usize)> + 'a {
        let slen = self.bitv.storage.len();
        let olen = other.bitv.storage.len();

        
        return self.bitv.storage[0..olen].iter().enumerate()
                .zip(repeat(olen))
                .map(move |((i, &w), min)| (olen < slen, (i + min) * 64, w));
    }


    pub fn difference(&self, other: &BitvSet, f: &fn(&usize) -> bool) -> bool {
        /*for (i, w1, w2) in self.common_iter(other) {
            if !iterate_bits(i, w1 & !w2, &|b| f(&b)) {
                return false
            }
        };*/
        /* everything we have that they don't also shows up */
        /*self.outlier_iter(other).advance(|(mine, i, w)|
            !mine || iterate_bits(i, w, |b| f(&b))
        )*/
        unimplemented!()
    }


    pub fn capacity(&self) -> usize {
        self.bitv.storage.len() * 64
    }

    pub fn union_with(&mut self, other: &BitvSet) {
        self.other_op(other, &|w1, w2| w1 | w2);
    }

    pub fn difference_with(&mut self, other: &BitvSet) {
        self.other_op(other, &|w1, w2| w1 & !w2);
    }

    pub fn iter<'a>(&'a self) -> BitvSetIterator {
        BitvSetIterator {
            set: self,next_idx: 0
        }
    }

    /// Symmetric difference in-place with the specified other bit vector
    pub fn symmetric_difference_with(&mut self, other: &BitvSet) {
        self.other_op(other, &|w1, w2| w1 ^ w2);
    }

    pub fn other_op(&mut self, other: &BitvSet, f: &Fn(usize, usize) -> usize) {
        fn nbits(mut w: usize) -> usize {
            let mut bits = 0;
            for _ in 0..64 {
                if w == 0 {
                    break;
                }
                bits += w & 1;
                w >>= 1;
            }
            return bits;
        }
        if self.capacity() < other.capacity() {
            // self.bitv.storage.grow(other.capacity() / 64, &0);
            for _ in 0..other.capacity() / 64 {
                self.bitv.storage.push(0);
            }
        }
        for (i, &w) in other.bitv.storage.iter().enumerate() {
            let old = self.bitv.storage[i];
            let new = f(old, w);
            self.bitv.storage[i] = new;
            self.size += nbits(new) - nbits(old);
        }
    }

    pub fn intersect_with(&mut self, other: &BitvSet) {
        self.other_op(other, &|w1, w2| w1 & w2);
    }

    pub fn contains(&self, value: &usize) -> bool {
        *value < self.bitv.storage.len() * 64 && self.bitv.get(*value)
    }

    pub fn insert(&mut self, value: usize) -> bool {
        if self.contains(&value) {
            return false;
        }

        let nbits = self.capacity();

        if value >= nbits {
            let newsize = max(value, nbits / 2) / 64 + 1;
            assert!(newsize > self.bitv.storage.len());
            for _ in 0..newsize {
                self.bitv.storage.push(0);
            }
        }
        self.size += 1;
        self.bitv.set(value, true);
        return true;
    }

    pub fn remove(&mut self, value: &usize) -> bool {
        if !self.contains(value) {
            return false;
        }
        self.size -= 1;
        self.bitv.set(*value, false);

        // Attempt to truncate our storage
        let mut i = self.bitv.storage.len();
        while i > 1 && self.bitv.storage[i - 1] == 0 {
            i -= 1;
        }
        self.bitv.storage.truncate(i);

        return true;
    }

    pub fn clear(&mut self) {
        for i in 0..self.bitv.storage.len() {
            self.bitv.storage[i] = 0;
        }
        self.size = 0;
    }

    pub fn len(&self) -> usize {
        self.size
    }
}
//! An emulation of CPython 3.13's `set` for hashable tuples.
//!
//! Several mazelib algorithms build Python sets of cells and then iterate
//! them (`list(s)`, `random.choice(list(s))`), so the output depends on
//! CPython's hash-table layout. [`PySet`] reproduces `Objects/setobject.c`
//! closely enough to iterate in the same order: the same hashes, probe
//! sequence, resize policy and `set_merge` fast paths. Deletion is not
//! needed by mazelib and is not supported.

use crate::Cell;

const MIN_SIZE: usize = 8;
const LINEAR_PROBES: usize = 9;
const PERTURB_SHIFT: u32 = 5;

const XXPRIME_1: u64 = 11_400_714_785_074_694_791;
const XXPRIME_2: u64 = 14_029_467_366_897_019_727;
const XXPRIME_5: u64 = 2_870_177_450_012_600_261;

/// Values that hash the way CPython hashes them.
pub trait PyHash {
    /// `hash(self)` as CPython computes it on a 64-bit build.
    fn py_hash(&self) -> i64;
}

/// `hash(int)` for integers well inside `±2**61`.
fn int_hash(v: i64) -> i64 {
    const MODULUS: i64 = (1 << 61) - 1;
    let h = if v >= 0 {
        v % MODULUS
    } else {
        -((-v) % MODULUS)
    };
    if h == -1 {
        -2
    } else {
        h
    }
}

/// CPython's `tuplehash` over already-hashed items.
fn tuple_hash(lanes: impl ExactSizeIterator<Item = i64>) -> i64 {
    let len = lanes.len() as u64;
    let mut acc = XXPRIME_5;
    for lane in lanes {
        acc = acc.wrapping_add((lane as u64).wrapping_mul(XXPRIME_2));
        acc = acc.rotate_left(31);
        acc = acc.wrapping_mul(XXPRIME_1);
    }
    acc = acc.wrapping_add(len ^ (XXPRIME_5 ^ 3_527_539));
    if acc == u64::MAX {
        1_546_275_796
    } else {
        acc as i64
    }
}

impl PyHash for Cell {
    fn py_hash(&self) -> i64 {
        tuple_hash([int_hash(self.0), int_hash(self.1)].into_iter())
    }
}

impl PyHash for Vec<Cell> {
    fn py_hash(&self) -> i64 {
        tuple_hash(self.iter().map(PyHash::py_hash))
    }
}

/// A CPython-compatible set (insertions only).
#[derive(Clone, Debug)]
pub struct PySet<K> {
    table: Vec<Option<(i64, K)>>,
    used: usize,
}

impl<K: PyHash + PartialEq + Clone> PySet<K> {
    /// An empty set, like `set()`.
    pub fn new() -> Self {
        PySet {
            table: vec![None; MIN_SIZE],
            used: 0,
        }
    }

    /// `set(iterable)`.
    pub fn from_items<I: IntoIterator<Item = K>>(items: I) -> Self {
        let mut s = PySet::new();
        for k in items {
            s.add(k);
        }
        s
    }

    fn mask(&self) -> usize {
        self.table.len() - 1
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.used
    }

    /// True if the set has no elements.
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// `set.add(key)` (`set_add_entry`).
    pub fn add(&mut self, key: K) {
        let hash = key.py_hash();
        let mask = self.mask();
        let mut i = (hash as u64 as usize) & mask;
        let mut perturb = hash as u64;
        loop {
            let probes = if i + LINEAR_PROBES <= mask {
                LINEAR_PROBES
            } else {
                0
            };
            for j in i..=i + probes {
                match &self.table[j] {
                    None => {
                        self.table[j] = Some((hash, key));
                        self.used += 1;
                        if self.used * 5 >= mask * 3 {
                            let minused = if self.used > 50_000 {
                                self.used * 2
                            } else {
                                self.used * 4
                            };
                            self.resize(minused);
                        }
                        return;
                    }
                    Some((h, k)) if *h == hash && *k == key => return,
                    Some(_) => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize))
                & mask;
        }
    }

    /// `set_insert_clean`: place a key known to be absent.
    fn insert_clean(table: &mut [Option<(i64, K)>], hash: i64, key: K) {
        let mask = table.len() - 1;
        let mut i = (hash as u64 as usize) & mask;
        let mut perturb = hash as u64;
        loop {
            if table[i].is_none() {
                table[i] = Some((hash, key));
                return;
            }
            if i + LINEAR_PROBES <= mask {
                if let Some(slot) = table[i + 1..=i + LINEAR_PROBES]
                    .iter_mut()
                    .find(|s| s.is_none())
                {
                    *slot = Some((hash, key));
                    return;
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize))
                & mask;
        }
    }

    /// `set_table_resize`.
    fn resize(&mut self, minused: usize) {
        let mut newsize = MIN_SIZE;
        while newsize <= minused {
            newsize <<= 1;
        }
        if newsize == MIN_SIZE && self.table.len() == MIN_SIZE {
            // CPython keeps the small table as is when there are no dummies.
            return;
        }
        let old = std::mem::replace(&mut self.table, vec![None; newsize]);
        for (h, k) in old.into_iter().flatten() {
            Self::insert_clean(&mut self.table, h, k);
        }
    }

    /// `set_merge`: add every element of `other`.
    pub fn merge(&mut self, other: &PySet<K>) {
        if other.used == 0 {
            return;
        }
        if (self.used + other.used) * 5 >= self.mask() * 3 {
            self.resize((self.used + other.used) * 2);
        }
        if self.used == 0 && self.table.len() == other.table.len() {
            self.table.clone_from(&other.table);
            self.used = other.used;
            return;
        }
        if self.used == 0 {
            for (h, k) in other.table.iter().flatten() {
                Self::insert_clean(&mut self.table, *h, k.clone());
            }
            self.used = other.used;
            return;
        }
        for (_, k) in other.table.iter().flatten() {
            self.add(k.clone());
        }
    }

    /// `self.union(other)`: a copy of `self` merged with `other`.
    pub fn union(&self, other: &PySet<K>) -> PySet<K> {
        let mut result = PySet::new();
        result.merge(self);
        result.merge(other);
        result
    }

    /// `key in self`.
    pub fn contains(&self, key: &K) -> bool {
        let hash = key.py_hash();
        let mask = self.mask();
        let mut i = (hash as u64 as usize) & mask;
        let mut perturb = hash as u64;
        loop {
            let probes = if i + LINEAR_PROBES <= mask {
                LINEAR_PROBES
            } else {
                0
            };
            for j in i..=i + probes {
                match &self.table[j] {
                    None => return false,
                    Some((h, k)) if *h == hash && k == key => return true,
                    Some(_) => {}
                }
            }
            perturb >>= PERTURB_SHIFT;
            i = (i
                .wrapping_mul(5)
                .wrapping_add(1)
                .wrapping_add(perturb as usize))
                & mask;
        }
    }

    /// `self == other` (same elements).
    pub fn set_eq(&self, other: &PySet<K>) -> bool {
        self.used == other.used && self.iter().all(|k| other.contains(k))
    }

    /// `len(self.intersection(other)) > 0`.
    pub fn intersects(&self, other: &PySet<K>) -> bool {
        self.iter().any(|k| other.contains(k))
    }

    /// Iterate in CPython's order (`list(s)`).
    pub fn iter(&self) -> impl Iterator<Item = &K> {
        self.table.iter().flatten().map(|(_, k)| k)
    }

    /// `list(s)`.
    pub fn to_vec(&self) -> Vec<K> {
        self.iter().cloned().collect()
    }
}

impl<K: PyHash + PartialEq + Clone> Default for PySet<K> {
    fn default() -> Self {
        PySet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuple_hashes_match_cpython() {
        // hash((1, 2)), hash((0, 0)), hash(((1, 1), (1, 2)))
        assert_eq!((1i64, 2i64).py_hash(), -3_550_055_125_485_641_917);
        assert_eq!((0i64, 0i64).py_hash(), -8_458_139_203_682_520_985);
        assert_eq!(
            vec![(1i64, 1i64), (1, 2)].py_hash(),
            -1_059_626_643_558_330_182
        );
    }

    #[test]
    fn int_hash_minus_one() {
        assert_eq!(int_hash(-1), -2);
        assert_eq!(int_hash(-5), -5);
    }

    #[test]
    fn iteration_order_matches_cpython() {
        // recorded with CPython 3.13: list(set(a)), list(set(b).union(set(a)))
        let a: Vec<Cell> = vec![
            (19, 9),
            (23, 11),
            (25, 23),
            (27, 23),
            (21, 29),
            (17, 1),
            (27, 15),
            (25, 7),
            (21, 1),
            (29, 5),
            (3, 11),
            (15, 27),
            (7, 13),
            (17, 3),
            (19, 7),
            (1, 23),
            (7, 13),
            (9, 5),
            (29, 27),
            (25, 13),
            (5, 25),
            (25, 3),
            (5, 19),
            (19, 15),
            (5, 5),
            (1, 27),
            (1, 7),
            (25, 7),
            (5, 27),
            (5, 9),
        ];
        let b: Vec<Cell> = vec![
            (11, 7),
            (17, 29),
            (21, 21),
            (7, 5),
            (23, 7),
            (29, 13),
            (9, 1),
        ];
        let sa = PySet::from_items(a);
        let sb = PySet::from_items(b);
        assert_eq!(
            sa.to_vec(),
            vec![
                (25, 23),
                (15, 27),
                (17, 3),
                (9, 5),
                (19, 9),
                (5, 19),
                (5, 25),
                (25, 7),
                (19, 15),
                (25, 13),
                (7, 13),
                (1, 27),
                (5, 9),
                (27, 15),
                (25, 3),
                (5, 27),
                (1, 23),
                (23, 11),
                (3, 11),
                (17, 1),
                (29, 27),
                (5, 5),
                (21, 29),
                (19, 7),
                (27, 23),
                (1, 7),
                (29, 5),
                (21, 1)
            ]
        );
        assert_eq!(
            sb.union(&sa).to_vec(),
            vec![
                (25, 23),
                (23, 7),
                (15, 27),
                (17, 3),
                (9, 5),
                (19, 9),
                (5, 19),
                (5, 25),
                (25, 7),
                (19, 15),
                (25, 13),
                (7, 13),
                (29, 13),
                (1, 27),
                (21, 21),
                (5, 9),
                (9, 1),
                (11, 7),
                (27, 15),
                (25, 3),
                (17, 29),
                (5, 27),
                (1, 23),
                (23, 11),
                (3, 11),
                (17, 1),
                (29, 27),
                (5, 5),
                (21, 29),
                (19, 7),
                (27, 23),
                (1, 7),
                (7, 5),
                (29, 5),
                (21, 1)
            ]
        );
    }
}

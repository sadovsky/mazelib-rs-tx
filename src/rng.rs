//! Bit-for-bit ports of the two random streams mazelib draws from.
//!
//! mazelib uses Python's global `random` module (for `randrange`, `choice`,
//! `random`, ...) and NumPy's global legacy `RandomState` (only for
//! `numpy.random.shuffle`). Both are MT19937 generators but they are seeded
//! differently and turn raw words into values differently. [`Rng`] bundles
//! one of each, seeded together the way `Maze.set_seed` does.

use crate::MazeError;

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// The MT19937 core shared by CPython and NumPy.
#[derive(Clone)]
struct Mt19937 {
    state: [u32; N],
    index: usize,
}

impl Mt19937 {
    /// `init_genrand`: seed from a single 32-bit word.
    fn from_u32(seed: u32) -> Self {
        let mut state = [0u32; N];
        state[0] = seed;
        for i in 1..N {
            let prev = state[i - 1];
            state[i] = 1_812_433_253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        Mt19937 { state, index: N }
    }

    /// `init_by_array`: seed from a key of 32-bit words.
    fn from_key(key: &[u32]) -> Self {
        let mut mt = Mt19937::from_u32(19_650_218);
        let mt_state = &mut mt.state;
        let mut i = 1usize;
        let mut j = 0usize;
        let len = key.len();
        let mut k = N.max(len);
        while k > 0 {
            let prev = mt_state[i - 1];
            mt_state[i] = (mt_state[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt_state[0] = mt_state[N - 1];
                i = 1;
            }
            if j >= len {
                j = 0;
            }
            k -= 1;
        }
        k = N - 1;
        while k > 0 {
            let prev = mt_state[i - 1];
            mt_state[i] = (mt_state[i] ^ (prev ^ (prev >> 30)).wrapping_mul(1_566_083_941))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt_state[0] = mt_state[N - 1];
                i = 1;
            }
            k -= 1;
        }
        mt_state[0] = 0x8000_0000;
        mt
    }

    fn twist(&mut self) {
        let mag01 = [0u32, MATRIX_A];
        let mt = &mut self.state;
        let mut kk = 0;
        while kk < N - M {
            let y = (mt[kk] & UPPER_MASK) | (mt[kk + 1] & LOWER_MASK);
            mt[kk] = mt[kk + M] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            kk += 1;
        }
        while kk < N - 1 {
            let y = (mt[kk] & UPPER_MASK) | (mt[kk + 1] & LOWER_MASK);
            mt[kk] = mt[kk + M - N] ^ (y >> 1) ^ mag01[(y & 1) as usize];
            kk += 1;
        }
        let y = (mt[N - 1] & UPPER_MASK) | (mt[0] & LOWER_MASK);
        mt[N - 1] = mt[M - 1] ^ (y >> 1) ^ mag01[(y & 1) as usize];
        self.index = 0;
    }

    fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
    }
}

/// Python's `random.Random` (CPython 3.13 semantics) for integer seeds.
#[derive(Clone)]
pub struct PyRandom {
    mt: Mt19937,
}

/// Python floor division for `i64`.
fn floor_div(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

impl PyRandom {
    /// `random.seed(seed)` for an integer seed: the key is `abs(seed)` split
    /// into 32-bit words, least significant first.
    pub fn new(seed: i64) -> Self {
        let mut n = seed.unsigned_abs();
        let mut key = Vec::new();
        while n > 0 {
            key.push(n as u32);
            n >>= 32;
        }
        if key.is_empty() {
            key.push(0);
        }
        PyRandom {
            mt: Mt19937::from_key(&key),
        }
    }

    /// `random.getrandbits(k)` for `0 <= k <= 64`.
    pub fn getrandbits(&mut self, k: u32) -> u64 {
        assert!(k <= 64, "getrandbits supports at most 64 bits");
        if k == 0 {
            return 0;
        }
        if k <= 32 {
            return u64::from(self.mt.next_u32() >> (32 - k));
        }
        let mut result = 0u64;
        let mut remaining = k;
        let mut shift = 0;
        while remaining > 0 {
            let mut r = self.mt.next_u32();
            if remaining < 32 {
                r >>= 32 - remaining;
            }
            result |= u64::from(r) << shift;
            shift += 32;
            remaining = remaining.saturating_sub(32);
        }
        result
    }

    /// `random._randbelow(n)` for `n > 0`.
    pub fn randbelow(&mut self, n: u64) -> u64 {
        let k = 64 - n.leading_zeros();
        let mut r = self.getrandbits(k);
        while r >= n {
            r = self.getrandbits(k);
        }
        r
    }

    /// `random.random()`: a float in `[0, 1)` with 53 random bits.
    pub fn random(&mut self) -> f64 {
        let a = self.mt.next_u32() >> 5;
        let b = self.mt.next_u32() >> 6;
        (f64::from(a) * 67_108_864.0 + f64::from(b)) * (1.0 / 9_007_199_254_740_992.0)
    }

    /// `random.randrange(stop)`.
    pub fn randrange_to(&mut self, stop: i64) -> Result<i64, MazeError> {
        if stop > 0 {
            Ok(self.randbelow(stop as u64) as i64)
        } else {
            Err(MazeError::Value("empty range for randrange()".into()))
        }
    }

    /// `random.randrange(start, stop, step)`.
    pub fn randrange(&mut self, start: i64, stop: i64, step: i64) -> Result<i64, MazeError> {
        let width = stop - start;
        if step == 1 {
            if width > 0 {
                return Ok(start + self.randbelow(width as u64) as i64);
            }
            return Err(MazeError::Value("empty range for randrange()".into()));
        }
        let n = if step > 0 {
            floor_div(width + step - 1, step)
        } else if step < 0 {
            floor_div(width + step + 1, step)
        } else {
            return Err(MazeError::Value("zero step for randrange()".into()));
        };
        if n <= 0 {
            return Err(MazeError::Value("empty range for randrange()".into()));
        }
        Ok(start + step * self.randbelow(n as u64) as i64)
    }

    /// `random.randint(a, b)`.
    pub fn randint(&mut self, a: i64, b: i64) -> Result<i64, MazeError> {
        self.randrange(a, b + 1, 1)
    }

    /// The index `random.choice(seq)` picks for a sequence of length `len`.
    pub fn choice_index(&mut self, len: usize) -> Result<usize, MazeError> {
        if len == 0 {
            return Err(MazeError::Index(
                "Cannot choose from an empty sequence".into(),
            ));
        }
        Ok(self.randbelow(len as u64) as usize)
    }

    /// `random.choice(seq)`.
    pub fn choice<T: Clone>(&mut self, seq: &[T]) -> Result<T, MazeError> {
        let i = self.choice_index(seq.len())?;
        Ok(seq[i].clone())
    }

    /// `random.shuffle(x)`.
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.randbelow(i as u64 + 1) as usize;
            x.swap(i, j);
        }
    }
}

/// NumPy's legacy `numpy.random.RandomState`, limited to what mazelib uses.
#[derive(Clone)]
pub struct NumpyRandom {
    mt: Mt19937,
}

impl NumpyRandom {
    /// `numpy.random.seed(seed)`; NumPy only accepts `0 <= seed < 2**32`.
    pub fn new(seed: i64) -> Result<Self, MazeError> {
        let seed = u32::try_from(seed)
            .map_err(|_| MazeError::Value("Seed must be between 0 and 2**32 - 1".into()))?;
        Ok(NumpyRandom {
            mt: Mt19937::from_u32(seed),
        })
    }

    /// `random_interval(max)`: uniform in `[0, max]` by masked rejection.
    pub fn random_interval(&mut self, max: u64) -> u64 {
        if max == 0 {
            return 0;
        }
        let mut mask = max;
        mask |= mask >> 1;
        mask |= mask >> 2;
        mask |= mask >> 4;
        mask |= mask >> 8;
        mask |= mask >> 16;
        mask |= mask >> 32;
        if max <= 0xffff_ffff {
            loop {
                let value = u64::from(self.mt.next_u32()) & mask;
                if value <= max {
                    return value;
                }
            }
        }
        loop {
            let hi = u64::from(self.mt.next_u32());
            let lo = u64::from(self.mt.next_u32());
            let value = ((hi << 32) | lo) & mask;
            if value <= max {
                return value;
            }
        }
    }

    /// `numpy.random.shuffle(x)`.
    pub fn shuffle<T>(&mut self, x: &mut [T]) {
        for i in (1..x.len()).rev() {
            let j = self.random_interval(i as u64) as usize;
            x.swap(i, j);
        }
    }
}

/// Both random streams mazelib draws from, seeded together.
#[derive(Clone)]
pub struct Rng {
    /// The Python `random` stream.
    pub py: PyRandom,
    /// The NumPy legacy stream.
    pub np: NumpyRandom,
}

impl Rng {
    /// `Maze.set_seed(seed)`: seeds Python's `random`, then NumPy's legacy
    /// generator. Fails like NumPy for seeds outside `0..2**32`.
    pub fn seeded(seed: i64) -> Result<Self, MazeError> {
        let py = PyRandom::new(seed);
        let np = NumpyRandom::new(seed)?;
        Ok(Rng { py, np })
    }

    /// An unseeded generator, seeded from the system clock.
    pub fn from_entropy() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seed = (nanos as u64 & 0xffff_ffff) as i64;
        Rng {
            py: PyRandom::new(seed ^ 0x5eed),
            np: NumpyRandom {
                mt: Mt19937::from_u32(seed as u32),
            },
        }
    }
}

impl Default for Rng {
    fn default() -> Self {
        Rng::from_entropy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_random_seed_zero() {
        // random.seed(0); random.random()
        let mut r = PyRandom::new(0);
        assert_eq!(r.random(), 0.8444218515250481);
    }

    #[test]
    fn numpy_seed_range() {
        assert!(NumpyRandom::new(-1).is_err());
        assert!(NumpyRandom::new(1 << 32).is_err());
        assert!(NumpyRandom::new((1 << 32) - 1).is_ok());
    }

    #[test]
    fn floor_division() {
        assert_eq!(floor_div(-7, 2), -4);
        assert_eq!(floor_div(7, -2), -4);
        assert_eq!(floor_div(-8, -2), 4);
    }
}

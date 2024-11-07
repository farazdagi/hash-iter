#![doc = include_str!("../README.md")]

use {
    num_traits::{self, Num},
    std::{
        fmt,
        hash::{self},
    },
    xxhash_rust::xxh3::Xxh3Builder,
};

/// Provides an iterator over multiple hash values for a given key.
pub trait HashIterHasher<V> {
    fn hash_iter<K: hash::Hash>(&self, key: &K) -> impl Iterator<Item = V>;
}

/// Builds hash iterator hasher -- a hasher capable of generating multiple hash
/// values.
pub trait BuildHashIterHasher<T> {
    type Hasher: HashIterHasher<T>;

    fn build_hash_iter_hasher(&self) -> Self::Hasher;
}

/// Holds the state for the hasher that implements enhanced double hashing.
///
/// Serves as a builder, allowing to configure the hasher with custom seeds,
/// number of required hashes, and the size of the hash table.
pub struct DoubleHasherState {
    seed1: u64,
    seed2: u64,
    n: usize,
    k: usize,
}

impl DoubleHasherState {
    /// Constructs a new hash iterator builder, with default seeds.
    pub fn new(k: usize) -> Self {
        // Seeds for double hashing: essentially, we can use any seeds, to
        // initialize the hasher (by default XXH3 uses `0`).
        Self {
            seed1: 12345,
            seed2: 67890,
            n: usize::MAX,
            k,
        }
    }

    pub fn with_seed1(self, seed1: u64) -> Self {
        Self { seed1, ..self }
    }

    pub fn with_seed2(self, seed2: u64) -> Self {
        Self { seed2, ..self }
    }

    pub fn with_n(self, n: usize) -> Self {
        Self { n, ..self }
    }

    pub fn with_k(self, k: usize) -> Self {
        Self { k, ..self }
    }
}

impl BuildHashIterHasher<u64> for DoubleHasherState {
    type Hasher = DoubleHasher<Xxh3Builder, Xxh3Builder>;

    fn build_hash_iter_hasher(&self) -> Self::Hasher {
        DoubleHasher::with_hash_builders(
            Xxh3Builder::new().with_seed(self.seed1),
            Xxh3Builder::new().with_seed(self.seed2),
            self.n,
            self.k,
        )
    }
}

/// Enhanced double hashing hasher.
///
/// Emits an iterator (for a given input key) over hash values generated using
/// enhanced double hashing.
pub struct DoubleHasher<H1, H2> {
    hash_builder1: H1,
    hash_builder2: H2,
    n: usize,
    k: usize,
}

impl DoubleHasher<Xxh3Builder, Xxh3Builder> {
    /// Constructs a new double hasher using default hash builders.
    pub fn new(k: usize) -> Self {
        let state = DoubleHasherState::new(k);
        state.build_hash_iter_hasher()
    }
}

impl<H1, H2> DoubleHasher<H1, H2> {
    pub fn with_hash_builders(hash_builder1: H1, hash_builder2: H2, n: usize, k: usize) -> Self {
        Self {
            hash_builder1,
            hash_builder2,
            n,
            k,
        }
    }
}

impl<H1, H2> HashIterHasher<u64> for DoubleHasher<H1, H2>
where
    H1: hash::BuildHasher,
    H2: hash::BuildHasher,
{
    fn hash_iter<K: hash::Hash>(&self, key: &K) -> impl Iterator<Item = u64> {
        let hash1 = self.hash_builder1.hash_one(key);
        let hash2 = self.hash_builder2.hash_one(key);

        Hashes::new(hash1, hash2, self.n as u64, self.k as u64)
    }
}

/// Iterator over hash values generated using enhanced double hashing technique.
///
/// Implements enhanced double hashing technique as described in [Bloom Filters
/// in Probabilistic Verification paper][1] (see section 5.1 for details).
///
/// Mathematically, the hash function is defined as:
/// ```math
/// h(i) = h1(k) + i * h2(k) + (i^3-i)/6 (mod n)
/// ```
///
/// [1]: https://www.khoury.northeastern.edu/~pete/pub/bloom-filters-verification.pdf
#[derive(Debug)]
pub struct Hashes<T: Num> {
    /// The first hash point.
    hash1: T,

    /// The second hash point.
    hash2: T,

    /// The size of the hash table.
    n: T,

    /// The number of hash points to generate.
    k: T,

    /// The current number of hash points generated.
    cnt: T,
}

impl<T> Hashes<T>
where
    T: num_traits::Num,
{
    /// Constructs a new hash iterator.
    ///
    /// The iterator is configured with the given starting hash points, for the
    /// hashmap of size `n`, with expected number of generated hash points
    /// equal to `k`.
    pub fn new(hash1: T, hash2: T, n: T, k: T) -> Self {
        Self {
            hash1,
            hash2,
            n,
            k,
            cnt: T::zero(),
        }
    }
}

impl<T> Iterator for Hashes<T>
where
    T: num_traits::Num + num_traits::WrappingAdd + Copy + fmt::Debug,
{
    type Item = T;

    /// Returns the next hash point using enhanced double hashing algorithm.
    /// The computation is optimized using forward differencing.
    fn next(&mut self) -> Option<Self::Item> {
        if self.cnt == self.k {
            return None;
        }

        if self.cnt == T::zero() {
            self.cnt = self.cnt.add(T::one());
            return Some(self.hash1.rem(self.n));
        }

        self.hash1 = self.hash1.wrapping_add(&self.hash2).rem(self.n);
        self.hash2 = self.hash2.wrapping_add(&self.cnt).rem(self.n);
        self.cnt = self.cnt.add(T::one());

        Some(self.hash1)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::hash::BuildHasher};

    /// Mathematical representation of the enhanced double hashing function.
    /// h(i) = h1(k) + i * h2(k) + (i^3-i)/6 (mod n)
    ///
    /// Straight from the paper:
    /// x, y := a(δ) MOD m, b(δ) MOD m
    /// f[0] := x
    /// for i := 1 .. k-1
    ///   x := (x + y) MOD m
    ///   y := (y + i) MOD m
    ///   f[i] := x
    fn hasn_fn(i: u64, hash1: u64, hash2: u64, n: u64) -> u64 {
        let x = hash1.wrapping_rem(n);
        let y = hash2.wrapping_rem(n);
        x.wrapping_add(y.wrapping_mul(i))
            .wrapping_add((i.pow(3) - i) / 6)
            .wrapping_rem(n)
    }

    #[test]
    fn default_build_hash_iter() {
        let key = "mykey";
        let n = 1e9 as u64;
        let k = 100u64;

        let builder = DoubleHasherState::new(k as usize)
            .with_n(n as usize)
            .with_seed1(1)
            .with_seed2(2);

        let hash_builder = Xxh3Builder::new();
        let hash1 = hash_builder.with_seed(1).hash_one(key);
        let hash2 = hash_builder.with_seed(2).hash_one(key);

        let hasher = builder.build_hash_iter_hasher();
        for (i, hash) in hasher.hash_iter(&key).enumerate() {
            assert_eq!(hash, hasn_fn(i as u64, hash1, hash2, n));
        }
    }

    #[test]
    fn hashes_next() {
        let hash_builder = Xxh3Builder::new();
        let hash1 = hash_builder.with_seed(1).hash_one("mykey");
        let hash2 = hash_builder.with_seed(2).hash_one("mykey");

        let n = 1e9 as u64;
        let k = 100;
        let mut iter = Hashes::new(hash1, hash2, n, k);
        for i in 0..k {
            assert_eq!(iter.next(), Some(hasn_fn(i as u64, hash1, hash2, n)));
        }
    }
}

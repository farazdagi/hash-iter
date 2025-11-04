#![doc = include_str!("../README.md")]

use {num_traits, std::hash, xxhash_rust::xxh3::Xxh3Builder};

/// Represents a number type.
///
/// This allows to clients to create hashers that emit hashes of different sizes
/// (`usize`, `u64`, and `u128` when `std::Hasher` supports emitting it).
pub trait Number:
    num_traits::Num
    + num_traits::WrappingAdd
    + num_traits::FromPrimitive
    + num_traits::ToPrimitive
    + Copy
{
}

impl<T> Number for T where
    T: num_traits::Num
        + num_traits::WrappingAdd
        + num_traits::FromPrimitive
        + num_traits::ToPrimitive
        + Copy
{
}

/// Provides an iterator over multiple hash values for a given key.
pub trait HashIterHasher<T> {
    /// Returns an iterator over `count` number of hash values generated using
    /// enhanced double hashing.
    fn hash_iter<K: hash::Hash + ?Sized>(&self, key: &K, count: usize) -> impl Iterator<Item = T>;
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
#[derive(Clone, Copy)]
pub struct DoubleHashBuilder<T: Number = u64> {
    seed1: T,
    seed2: T,
    n: T,
}

impl<T: Number> DoubleHashBuilder<T> {
    /// Constructs a new hash iterator builder, with default seeds.
    pub fn new() -> Self {
        // Seeds for double hashing: essentially, we can use any seeds, to
        // initialize the hasher (by default XXH3 uses `0`).
        let seed1 = num_traits::FromPrimitive::from_u64(12345).expect("cannot create seed1");
        let seed2 = num_traits::FromPrimitive::from_u64(67890).expect("cannot create seed2");
        let n = num_traits::FromPrimitive::from_u64(usize::MAX as u64).expect("cannot create n");
        Self { seed1, seed2, n }
    }

    pub fn with_seed1(self, seed1: T) -> Self {
        Self { seed1, ..self }
    }

    pub fn with_seed2(self, seed2: T) -> Self {
        Self { seed2, ..self }
    }

    pub fn with_n(self, n: T) -> Self {
        Self { n, ..self }
    }
}

impl<T: Number> Default for DoubleHashBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Number + PartialOrd> BuildHashIterHasher<T> for DoubleHashBuilder<T> {
    type Hasher = DoubleHashHasher<T, Xxh3Builder, Xxh3Builder>;

    fn build_hash_iter_hasher(&self) -> Self::Hasher {
        let seed1 = num_traits::ToPrimitive::to_u64(&self.seed1).expect("cannot create seed1");
        let seed2 = num_traits::ToPrimitive::to_u64(&self.seed2).expect("cannot create seed2");
        DoubleHashHasher::with_hash_builders(
            Xxh3Builder::new().with_seed(seed1),
            Xxh3Builder::new().with_seed(seed2),
            self.n,
        )
    }
}

/// Enhanced double hashing hasher.
///
/// Emits an iterator (for a given input key) over hash values generated using
/// enhanced double hashing.
#[derive(Clone, Copy)]
pub struct DoubleHashHasher<T = u64, H1 = Xxh3Builder, H2 = Xxh3Builder> {
    hash_builder1: H1,
    hash_builder2: H2,
    n: T,
}

impl DoubleHashHasher<u64, Xxh3Builder, Xxh3Builder> {
    /// Constructs a new double hasher using default hash builders.
    pub fn new() -> Self {
        DoubleHashBuilder::new().build_hash_iter_hasher()
    }
}

impl<T, H1, H2> DoubleHashHasher<T, H1, H2> {
    pub fn with_hash_builders(hash_builder1: H1, hash_builder2: H2, n: T) -> Self {
        Self {
            hash_builder1,
            hash_builder2,
            n,
        }
    }
}

impl<T, H1, H2> HashIterHasher<T> for DoubleHashHasher<T, H1, H2>
where
    T: Number + PartialOrd,
    H1: hash::BuildHasher,
    H2: hash::BuildHasher,
{
    fn hash_iter<K: hash::Hash + ?Sized>(&self, key: &K, count: usize) -> impl Iterator<Item = T> {
        let hash1 = self.hash_builder1.hash_one(key);
        let hash2 = self.hash_builder2.hash_one(key);

        let x = num_traits::FromPrimitive::from_u64(hash1).expect("invalid hash point");
        let y = num_traits::FromPrimitive::from_u64(hash2).expect("invalid hash point");
        let count = num_traits::FromPrimitive::from_usize(count).expect("invalid count");
        Hashes::new(x, y, self.n, count)
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
pub struct Hashes<T: Number> {
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
    T: Number,
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
    T: Number + PartialOrd,
{
    type Item = T;

    /// Returns the next hash point using enhanced double hashing algorithm.
    /// The computation is optimized using forward differencing.
    fn next(&mut self) -> Option<Self::Item> {
        if self.cnt == self.k {
            return None;
        }

        // Helper function for modular addition: computes (a + b) mod n.
        // Assumes a and b are already reduced mod n (i.e., a < n and b < n).
        // This avoids overflow issues that arise with naive wrapping_add + rem.
        let add_mod = |a: T, b: T, n: T| -> T {
            debug_assert!(a < n && b < n, "operands must be reduced mod n");

            // Check if a + b >= n by testing a >= n - b
            // This is safe because b < n, so n - b doesn't underflow
            if a >= n - b {
                // a + b >= n, so result is (a + b) - n
                // Compute as a - (n - b) to avoid overflow
                a - (n - b)
            } else {
                // a + b < n, just add normally
                a + b
            }
        };

        if self.cnt == T::zero() {
            self.cnt = self.cnt.add(T::one());
            // Reduce initial values on first iteration
            self.hash1 = self.hash1.rem(self.n);
            self.hash2 = self.hash2.rem(self.n);
            return Some(self.hash1);
        }

        // Both hash1 and hash2 are now guaranteed to be < n (reduced in previous
        // iteration).
        self.hash1 = add_mod(self.hash1, self.hash2, self.n);
        self.hash2 = add_mod(self.hash2, self.cnt, self.n);
        self.cnt = self.cnt.add(T::one());

        Some(self.hash1)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.k - self.cnt).to_usize().unwrap_or(0);
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for Hashes<T> where T: Number + PartialOrd {}

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
        let k = 100;

        let builder = DoubleHashBuilder::new()
            .with_n(n)
            .with_seed1(1)
            .with_seed2(2);

        let hash_builder = Xxh3Builder::new();
        let hash1 = hash_builder.with_seed(1).hash_one(key);
        let hash2 = hash_builder.with_seed(2).hash_one(key);

        let hasher = builder.build_hash_iter_hasher();
        for (i, hash) in hasher.hash_iter(&key, k).enumerate() {
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

#![doc = include_str!("../README.md")]

use {std::hash, xxhash_rust::xxh3::Xxh3Builder};

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
pub struct DoubleHashBuilder<T> {
    seed1: T,
    seed2: T,
    n: T,
}

/// Enhanced double hashing hasher.
///
/// Emits an iterator (for a given input key) over hash values generated using
/// enhanced double hashing.
#[derive(Clone, Copy)]
pub struct DoubleHashHasher<T, H1, H2> {
    hash_builder1: H1,
    hash_builder2: H2,
    n: T,
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
#[derive(Debug, Clone, Copy)]
pub struct Hashes<T> {
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

/// Macro to generate implementations for different numeric types.
macro_rules! impl_hash_iter_for_type {
    ($num_type:ty, $type_name:expr) => {
        impl DoubleHashBuilder<$num_type> {
            /// Constructs a new hash iterator builder, with default seeds.
            pub fn new() -> Self {
                // Seeds for double hashing: essentially, we can use any seeds, to
                // initialize the hasher (by default XXH3 uses `0`).
                // Using wrapping_add to handle truncation for smaller types
                let seed1 = (12345_u64 as $num_type).wrapping_add(0);
                let seed2 = (67890_u64 as $num_type).wrapping_add(0);
                let n = <$num_type>::MAX;
                Self { seed1, seed2, n }
            }

            pub fn with_seed1(self, seed1: $num_type) -> Self {
                Self { seed1, ..self }
            }

            pub fn with_seed2(self, seed2: $num_type) -> Self {
                Self { seed2, ..self }
            }

            pub fn with_n(self, n: $num_type) -> Self {
                Self { n, ..self }
            }
        }

        impl Default for DoubleHashBuilder<$num_type> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl BuildHashIterHasher<$num_type> for DoubleHashBuilder<$num_type> {
            type Hasher = DoubleHashHasher<$num_type, Xxh3Builder, Xxh3Builder>;

            fn build_hash_iter_hasher(&self) -> Self::Hasher {
                DoubleHashHasher::<$num_type, _, _>::with_hash_builders(
                    Xxh3Builder::new().with_seed(self.seed1 as u64),
                    Xxh3Builder::new().with_seed(self.seed2 as u64),
                    self.n,
                )
            }
        }

        impl<H1, H2> DoubleHashHasher<$num_type, H1, H2> {
            pub fn with_hash_builders(hash_builder1: H1, hash_builder2: H2, n: $num_type) -> Self {
                Self {
                    hash_builder1,
                    hash_builder2,
                    n,
                }
            }
        }

        impl<H1, H2> HashIterHasher<$num_type> for DoubleHashHasher<$num_type, H1, H2>
        where
            H1: hash::BuildHasher,
            H2: hash::BuildHasher,
        {
            fn hash_iter<K: hash::Hash + ?Sized>(
                &self,
                key: &K,
                count: usize,
            ) -> impl Iterator<Item = $num_type> {
                let hash1 = self.hash_builder1.hash_one(key);
                let hash2 = self.hash_builder2.hash_one(key);

                // Convert u64 hashes to target type
                let x = hash1 as $num_type;
                let y = hash2 as $num_type;

                // Convert count to target type
                // Safe: u32::MAX (4.3 billion) > any practical count value
                // u64::MAX and u128::MAX are even larger
                let count_t = count as $num_type;

                Hashes::<$num_type>::new(x, y, self.n, count_t)
            }
        }

        impl Hashes<$num_type> {
            /// Constructs a new hash iterator.
            ///
            /// The iterator is configured with the given starting hash points, for the
            /// hashmap of size `n`, with expected number of generated hash points
            /// equal to `k`.
            pub fn new(hash1: $num_type, hash2: $num_type, n: $num_type, k: $num_type) -> Self {
                Self {
                    hash1,
                    hash2,
                    n,
                    k,
                    cnt: 0,
                }
            }
        }

        impl Iterator for Hashes<$num_type> {
            type Item = $num_type;

            /// Returns the next hash point using enhanced double hashing algorithm.
            /// The computation is optimized using forward differencing.
            fn next(&mut self) -> Option<Self::Item> {
                if self.cnt == self.k {
                    return None;
                }

                // Helper function for modular addition: computes (a + b) mod n.
                // Assumes a and b are already reduced mod n (i.e., a < n and b < n).
                // This avoids overflow issues that arise with naive wrapping_add + rem.
                let add_mod = |a: $num_type, b: $num_type, n: $num_type| -> $num_type {
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

                if self.cnt == 0 {
                    self.cnt = self.cnt + 1;
                    // Reduce initial values on first iteration
                    self.hash1 = self.hash1 % self.n;
                    self.hash2 = self.hash2 % self.n;
                    return Some(self.hash1);
                }

                // Both hash1 and hash2 are now guaranteed to be < n (reduced in previous
                // iteration).
                self.hash1 = add_mod(self.hash1, self.hash2, self.n);
                self.hash2 = add_mod(self.hash2, self.cnt, self.n);
                self.cnt = self.cnt + 1;

                Some(self.hash1)
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let remaining = (self.k - self.cnt) as usize;
                (remaining, Some(remaining))
            }
        }

        impl ExactSizeIterator for Hashes<$num_type> {}
    };
}

impl DoubleHashHasher<u64, Xxh3Builder, Xxh3Builder> {
    /// Constructs a new double hasher using default parameters.
    pub fn new() -> Self {
        DoubleHashBuilder::<u64>::new().build_hash_iter_hasher()
    }
}

impl_hash_iter_for_type!(u32, "u32");
impl_hash_iter_for_type!(u64, "u64");
impl_hash_iter_for_type!(u128, "u128");

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

        let builder = DoubleHashBuilder::<u64>::new()
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
        let mut iter = Hashes::<u64>::new(hash1, hash2, n, k);
        for i in 0..k {
            assert_eq!(iter.next(), Some(hasn_fn(i as u64, hash1, hash2, n)));
        }
    }
}

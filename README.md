# hash-iter

[![crates.io](https://img.shields.io/crates/d/hash-iter.svg)](https://crates.io/crates/hash-iter)
[![docs.rs](https://docs.rs/hash-iter/badge.svg)](https://docs.rs/hash-iter)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![dependencies](https://deps.rs/repo/github/farazdagi/hash-iter/status.svg)](https://deps.rs/repo/github/farazdagi/hash-iter)

Implementation of the **enhanced double hashing** technique based on the
[Bloom Filters in Probabilistic Verification](https://www.khoury.northeastern.edu/~pete/pub/bloom-filters-verification.pdf)
paper (Dillinger and Manolios, 2004).

## Motivation

This crate is very simple: given a key `key`, one can hash it to a sequence of `k` hashes
`[hash_1, hash_2, .., hash_k]`, instead of a single hash.

This is useful for implementing many hash-based algorithms, such as:

- Hash tables with open addressing, where double hashing is used to resolve collisions.
- Bloom filters, since as shown in
  [Less Hashing, Same Performance: Building a Better Bloom Filter](https://www.eecs.harvard.edu/~michaelm/postscripts/rsa2008.pdf)
  (Kirsch and Mitzenmacher, 2006) instead of using `k` different hashers, we can rely on double
  hashing to produce `k` filter positions.
- Consistent hashing, where double hashing is used to map keys to a ring of servers (for example in
  [Multi-probe consistent hashing](https://crates.io/crates/mpchash)).

## Usage

Create and configure a hasher either by directly calling constructor of `DoubleHashHasher` or by
using the builder object `DoubleHashBuilder`.

### Basic usage

When we are relying on default parameters (for seed values, max hash value etc), and do not need to
keep builder around (which is normally the case, as a single generated hasher is often enough), we
can use the `DoubleHashHasher` constructor directly.

The library supports three practical numeric types (`u32`, `u64`, `u128`) for output hash values.
For the default output type (`u64`) use `DoubleHashHasher` constructor:


``` rust
use hash_iter::{DoubleHashHasher, HashIterHasher};

// Using u64 (most common, aligns with std::hash::Hasher)
let hasher = DoubleHashHasher::new();

// Hash each key to three u64 hash points.
let hashes = hasher.hash_iter(&"foo", 3).collect::<Vec<_>>();
let hashes = hasher.hash_iter(&"bar", 3).collect::<Vec<_>>();
```

### Configuring hasher state

There are several optional parameters that can be configured:

- `n`: the maximum hash value producible (by default it is `T::MAX` for the chosen numeric type).
- `seed1` and `seed2`: seeds for the two hash functions (by default they are `12345` and `67890`
  respectively, truncated to fit the target type).

``` rust
use hash_iter::{BuildHashIterHasher, DoubleHashBuilder, HashIterHasher};

// Configure hasher by seeding and capping the output value (`n`).
let hasher = DoubleHashBuilder::<u64>::new()
    .with_seed1(12345)
    .with_seed2(67890)
    .with_n(u64::MAX)
    .build_hash_iter_hasher();

let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();
```

Additionally, by giving `DoubleHashBuilder` explicit type parameter (for example, `DoubleHashBuilder::<u128>`) you can configure hash output type, as well:

``` rust
use hash_iter::{BuildHashIterHasher, DoubleHashBuilder, HashIterHasher};

// Configure for u32 with custom modulus
let hasher = DoubleHashBuilder::<u32>::new()
    .with_n(10000)
    .build_hash_iter_hasher();

let hashes: Vec<u32> = hasher.hash_iter(&"key", 5).collect();
```

### Custom hash functions

One can specify which hash functions to use when creating the first two hash values used to produce
the sequence.

All you need to do is to supply `DoubleHashHasher::with_hash_builders()` function with two structs
that implement [`hash::BuildHasher`](https://doc.rust-lang.org/std/hash/trait.BuildHasher.html):

``` rust
use hash_iter::{DoubleHashHasher, HashIterHasher};
use xxhash_rust::xxh3::Xxh3Builder;

let hasher = DoubleHashHasher::<u64, _, _>::with_hash_builders(
    Xxh3Builder::new().with_seed(12345),
    Xxh3Builder::new().with_seed(67890),
    u64::MAX, // n
);

let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();
```

You can use custom hash builders with any supported numeric type:

``` rust
use hash_iter::{DoubleHashHasher, HashIterHasher};
use xxhash_rust::xxh3::Xxh3Builder;

// Using u32 with custom hash builders
let hasher = DoubleHashHasher::<u32, _, _>::with_hash_builders(
    Xxh3Builder::new().with_seed(111),
    Xxh3Builder::new().with_seed(222),
    u32::MAX,
);

let hashes: Vec<u32> = hasher.hash_iter(&"test", 10).collect();
```

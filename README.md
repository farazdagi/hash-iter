# hash-iter

[![crates.io](https://img.shields.io/crates/d/hash-iter.svg)](https://crates.io/crates/hash-iter)
[![docs.rs](https://docs.rs/hash-iter/badge.svg)](https://docs.rs/hash-iter)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![dependencies](https://deps.rs/repo/github/farazdagi/hash-iter/status.svg)](https://deps.rs/repo/github/farazdagi/hash-iter)

Implementation of the **enhanced double hashing** technique based on the [Bloom Filters in Probabilistic Verification](https://www.khoury.northeastern.edu/~pete/pub/bloom-filters-verification.pdf) paper (Dillinger and Manolios, 2004).

## Motivation

Given a key `key`, instead of hashing it to a single value, one can hash it to a sequence of `k` hashes `[hash_1, hash_2, .., hash_k]`. 

See use cases below for details.

## Quick Start

```rust
use hash_iter::{DoubleHashHasher, HashIterHasher};

let hasher = DoubleHashHasher::new();
let hashes: Vec<u64> = hasher.hash_iter(&"key", 10).collect();
```

## Usage

### Basic Usage

Hash a key to generate a sequence of hash values:

```rust
use hash_iter::{DoubleHashHasher, HashIterHasher};

let hasher = DoubleHashHasher::new();

// Generate 3 hash values for each key
let hashes = hasher.hash_iter(&"foo", 3).collect::<Vec<_>>();
let hashes = hasher.hash_iter(&"bar", 3).collect::<Vec<_>>();
```

### Configuration

Customize the hasher with seeds, modulus, and output type:

```rust
use hash_iter::{BuildHashIterHasher, DoubleHashBuilder, HashIterHasher};

// Configure for u32 with custom parameters
let hasher = DoubleHashBuilder::<u32>::new()
    .with_seed1(12345)
    .with_seed2(67890)
    .with_n(10000)  // Maximum hash value
    .build_hash_iter_hasher();

let hashes: Vec<u32> = hasher.hash_iter(&"key", 5).collect();
```

Default configuration:
- Seeds: `12345`, `67890`
- Modulus `n`: `T::MAX` for the chosen type
- Hash function: XXH3

### Custom Hash Functions

Use any hash function implementing [`std::hash::BuildHasher`](https://doc.rust-lang.org/std/hash/trait.BuildHasher.html):

```rust
use hash_iter::{DoubleHashHasher, HashIterHasher};
use xxhash_rust::xxh3::Xxh3Builder;

let hasher = DoubleHashHasher::<u64, _, _>::with_hash_builders(
    Xxh3Builder::new().with_seed(111),
    Xxh3Builder::new().with_seed(222),
    u64::MAX,
);

let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();
```

## Use Cases

This crate enables efficient implementations of hash-based algorithms:

- **Bloom filters** - Generate multiple filter positions from a single key ([Kirsch and Mitzenmacher, 2006](https://www.eecs.harvard.edu/~michaelm/postscripts/rsa2008.pdf))
- **Hash tables** - Double hashing for collision resolution in open addressing
- **Consistent hashing** - Map keys to server rings (e.g., [mpchash](https://crates.io/crates/mpchash))

Instead of computing `k` independent hashes, double hashing produces `k` hash values from just two base hashes using the formula:

`h(i) = h₁(key) + i·h₂(key) + (i³-i)/6  (mod n)`

The implementation uses forward differencing for O(1) computation per iteration.

## Documentation

- **[API Documentation](https://docs.rs/hash-iter)** - Full API reference
- **[Algorithm Paper](https://www.khoury.northeastern.edu/~pete/pub/bloom-filters-verification.pdf)** - Enhanced double hashing technique
- **[Implementation Guide](CLAUDE.md)** - Detailed architecture and development notes

## License

MIT

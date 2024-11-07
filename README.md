# hash-iter

[![crates.io](https://img.shields.io/crates/d/hash-iter.svg)](https://crates.io/crates/hash-iter)
[![docs.rs](https://docs.rs/hash-iter/badge.svg)](https://docs.rs/hash-iter)

Implementation of the *enhanced double hashing* technique based on the
[Bloom Filters in Probabilistic Verification paper](https://www.khoury.northeastern.edu/~pete/pub/bloom-filters-verification.pdf)
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
  Multi-probe consistent hashing).

## Usage

Create and configure a hasher state (basically a reusable configuration), use it to create a hasher
object. The hasher object then can be used to hash keys into sequence of `k` hash points.

### Basic usage

Hasher state allows you to configure how hash iterators are produced. The only required parameter is
number of hashes per key input, `k`.

``` rust
    // Create a hasher state with 3 hashes per key.
    let state = DoubleHasherState::new(3);

    // Create a hasher object.
    // It holds state and can be used to produce hash iterators.
    let hasher = state.build_hash_iter_hasher();

    // Hash keys to several hash points (`hash_iter()` returns an iterator).
    let key = "hello";
    let hashes = hasher.hash_iter(&key).collect::<Vec<_>>();
```

When we are relying on default parameters (for seed values, max hash value etc), and do not need to
keep state around (which is normally the case, as a single generated hasher is often enough), we can
use the `DoubleHasher` directly:

``` rust
    let hasher = DoubleHasher::new(3);

    let hashes = hasher.hash_iter(&"foo").collect::<Vec<_>>();
    let hashes = hasher.hash_iter(&"bar").collect::<Vec<_>>();
```

### Configuring hasher state

In addition to `k`, there are several optional parameters that can be configured: `n` (max hash
value produced, by default it is `usize::MAX`, so that array indexing is safe), `seed1` and `seed2`
(seeds for the two hash functions, by default they are `12345` and `67890` respectively).

``` rust
    // Specify default values explicitly.
    let hasher = DoubleHasherState::new(3)
        .with_seed1(12345)
        .with_seed2(67890)
        .with_n(usize::MAX)
        .build_hash_iter_hasher();

    let hashes = hasher.hash_iter(&"hello").collect::<Vec<_>>();
```

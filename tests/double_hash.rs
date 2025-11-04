use {
    hash_iter::{BuildHashIterHasher, DoubleHashBuilder, DoubleHashHasher, HashIterHasher},
    xxhash_rust::xxh3::Xxh3Builder,
};

#[test]
fn default_config() {
    {
        // Implicit builder object.
        let hasher = DoubleHashHasher::<u64, _, _>::new();
        let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();

        assert_eq!(hashes, vec![
            10179864958193109059,
            16936771314159985077,
            5246933596417309481
        ]);
    }

    {
        // Explicit builder object.
        let builder = DoubleHashBuilder::<u64>::new();
        let hasher = builder.build_hash_iter_hasher();
        let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();

        assert_eq!(hashes, vec![
            10179864958193109059,
            16936771314159985077,
            5246933596417309481
        ]);
    }
}

#[test]
fn custom_config() {
    let hasher = DoubleHashBuilder::<u64>::new()
        .with_seed1(12345)
        .with_seed2(67890)
        .with_n(u64::MAX)
        .build_hash_iter_hasher();

    let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();
    assert_eq!(hashes, vec![
        10179864958193109059,
        16936771314159985077,
        5246933596417309481
    ]);
}

#[test]
fn custom_hash_builders() {
    let hasher = DoubleHashHasher::<u64, _, _>::with_hash_builders(
        Xxh3Builder::new().with_seed(12345),
        Xxh3Builder::new().with_seed(67890),
        u64::MAX,
    );

    let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();
    assert_eq!(hashes, vec![
        10179864958193109059,
        16936771314159985077,
        5246933596417309481
    ]);
}

#[test]
fn use_as_struct_field() {
    {
        // Implicit builder types.
        struct Foo {
            hasher: DoubleHashHasher<u64, Xxh3Builder, Xxh3Builder>,
        }

        impl Foo {
            fn new() -> Self {
                Self {
                    hasher: DoubleHashHasher::<u64, _, _>::new(),
                }
            }

            fn hash(&self, key: u64, count: usize) -> Vec<u64> {
                self.hasher.hash_iter(&key, count).collect()
            }
        }

        let foo = Foo::new();
        let hashes = foo.hash(42, 3);
        assert_eq!(hashes, vec![
            2604207548944960858,
            14475308512507584086,
            7899665402360655700
        ]);
    }
    {
        // Explicit builder types.
        struct Foo {
            hasher: DoubleHashHasher<u64, Xxh3Builder, Xxh3Builder>,
        }

        impl Foo {
            fn new() -> Self {
                Self {
                    hasher: DoubleHashHasher::<u64, _, _>::new(),
                }
            }

            fn hash(&self, key: u64, count: usize) -> Vec<u64> {
                self.hasher.hash_iter(&key, count).collect()
            }
        }

        let foo = Foo::new();
        let hashes = foo.hash(42, 3);
        assert_eq!(hashes, vec![
            2604207548944960858,
            14475308512507584086,
            7899665402360655700
        ]);
    }
}

#[test]
fn modular_arithmetic_overflow_regression() {
    // Regression test for correct modular arithmetic with large hash values.
    // When hash values are close to u64::MAX, wrapping_add can produce incorrect
    // results even after applying the modulus operator.
    //
    // This test uses large initial hash values (near u64::MAX) with a small modulus
    // to expose the overflow issue in the original implementation.
    use hash_iter::Hashes;

    // Setup: Large hash values near u64::MAX, small modulus
    let hash1: u64 = u64::MAX - 100; // 18446744073709551515
    let hash2: u64 = u64::MAX - 50; // 18446744073709551565
    let n: u64 = 1000;
    let k: u64 = 3;

    let mut iter = Hashes::<u64>::new(hash1, hash2, n, k);

    // First value: hash1 % n
    let first = iter.next();
    assert_eq!(first, Some(515), "First hash should be hash1 % n = 515");

    // Second value: This is where the overflow bug manifests
    // Correct calculation:
    //   hash1_reduced = 515
    //   hash2_reduced = 565
    //   (515 + 565) % 1000 = 1080 % 1000 = 80
    //
    // Buggy calculation (using wrapping_add without proper modular reduction):
    //   (u64::MAX - 100) wrapping_add (u64::MAX - 50) = u64::MAX - 151
    //   (u64::MAX - 151) % 1000 = 465 (WRONG!)
    let second = iter.next();
    assert_eq!(
        second,
        Some(80),
        "If you see 465, the implementation is using wrapping_add incorrectly."
    );

    // Third value: Continue the sequence
    // After second iteration:
    //   hash1 = 80, hash2 = (565 + 1) % 1000 = 566
    //   (80 + 566) % 1000 = 646
    let third = iter.next();
    assert_eq!(third, Some(646), "Third hash should be 646");

    // Should be exhausted
    assert_eq!(
        iter.next(),
        None,
        "Iterator should be exhausted after k iterations"
    );
}

#[test]
fn clone_iterator_checkpoint() {
    use hash_iter::Hashes;

    // Test 1: Clone at the start
    let hash1: u64 = 12345;
    let hash2: u64 = 67890;
    let n: u64 = 10000;
    let k: u64 = 5;

    let iter = Hashes::<u64>::new(hash1, hash2, n, k);
    let clone_at_start = iter.clone();

    // Both should produce identical sequences
    let original_values: Vec<_> = iter.collect();
    let cloned_values: Vec<_> = clone_at_start.collect();

    assert_eq!(
        original_values, cloned_values,
        "Clone at start should produce identical sequence"
    );
    assert_eq!(original_values.len(), 5, "Should produce 5 values");

    // Test 2: Clone mid-iteration
    let mut iter = Hashes::<u64>::new(hash1, hash2, n, k);

    // Consume first 2 values
    let _first = iter.next();
    let _second = iter.next();

    // Clone the iterator at this checkpoint
    let checkpoint = iter.clone();

    // Both original and checkpoint should produce the same remaining values
    let remaining_original: Vec<_> = iter.collect();
    let remaining_checkpoint: Vec<_> = checkpoint.collect();

    assert_eq!(
        remaining_original, remaining_checkpoint,
        "Clone mid-iteration should produce identical remaining sequence"
    );
    assert_eq!(
        remaining_original.len(),
        3,
        "Should have 3 remaining values after consuming 2"
    );

    // Test 3: Verify independence - advancing one doesn't affect the other
    let mut iter1 = Hashes::<u64>::new(hash1, hash2, n, k);
    let mut iter2 = iter1.clone();

    // Advance iter1 by 1
    let val1_from_iter1 = iter1.next();

    // iter2 should still start from the beginning
    let val1_from_iter2 = iter2.next();

    assert_eq!(
        val1_from_iter1, val1_from_iter2,
        "First values should match"
    );

    // Advance iter1 further
    iter1.next();
    iter1.next();

    // iter2 should be independent and only advanced once
    let val2_from_iter2 = iter2.next();

    // Verify iter2's second value matches a fresh iterator's second value
    let mut fresh = Hashes::<u64>::new(hash1, hash2, n, k);
    fresh.next();
    let val2_from_fresh = fresh.next();

    assert_eq!(
        val2_from_iter2, val2_from_fresh,
        "Cloned iterator should be independent of original"
    );

    // Test 4: Clone with Copy trait (not only Clone, but also Copy)
    let iter = Hashes::<u64>::new(hash1, hash2, n, 3);
    let copy_of_iter = iter; // This is a copy operation since Hashes derives Copy

    // Both should be usable independently
    let from_original: Vec<_> = iter.collect();
    let from_copy: Vec<_> = copy_of_iter.collect();

    assert_eq!(
        from_original, from_copy,
        "Copy should work the same as Clone for iterator state"
    );

    // Test 5: Multiple checkpoints at different positions
    let mut iter = Hashes::<u64>::new(hash1, hash2, n, 10);

    let checkpoint_0 = iter.clone();
    iter.next();
    iter.next();

    let checkpoint_2 = iter.clone();
    iter.next();
    iter.next();

    let checkpoint_4 = iter.clone();

    // Each checkpoint should have the correct number of remaining elements
    assert_eq!(checkpoint_0.count(), 10, "Checkpoint at position 0");
    assert_eq!(checkpoint_2.count(), 8, "Checkpoint at position 2");
    assert_eq!(checkpoint_4.count(), 6, "Checkpoint at position 4");
}

#[test]
fn test_u32_type() {
    let hasher = DoubleHashBuilder::<u32>::new().build_hash_iter_hasher();
    let hashes: Vec<u32> = hasher.hash_iter(&"test", 20).collect();

    assert_eq!(hashes.len(), 20);
    // All values should be valid u32
    for &hash in &hashes {
        assert!(hash <= u32::MAX);
    }
}

#[test]
fn test_u64_type() {
    let hasher = DoubleHashBuilder::<u64>::new().build_hash_iter_hasher();
    let hashes: Vec<u64> = hasher.hash_iter(&"test", 20).collect();

    assert_eq!(hashes.len(), 20);
    // All values should be valid u64
    for &hash in &hashes {
        assert!(hash <= u64::MAX);
    }
}

#[test]
fn test_u128_type() {
    let hasher = DoubleHashBuilder::<u128>::new().build_hash_iter_hasher();
    let hashes: Vec<u128> = hasher.hash_iter(&"test", 15).collect();

    assert_eq!(hashes.len(), 15);
    // All values should be valid (u128 is largest)
    for &hash in &hashes {
        assert!(hash <= u128::MAX);
    }
}

#[test]
fn test_u32_with_custom_seeds() {
    let hasher = DoubleHashBuilder::<u32>::new()
        .with_seed1(11111)
        .with_seed2(22222)
        .build_hash_iter_hasher();

    let hashes: Vec<u32> = hasher.hash_iter(&"custom_seeds", 10).collect();

    assert_eq!(hashes.len(), 10);
    // Verify deterministic behavior
    let hashes2: Vec<u32> = hasher.hash_iter(&"custom_seeds", 10).collect();
    assert_eq!(hashes, hashes2);
}

#[test]
fn test_different_types_produce_different_results() {
    // Same key and count, but different output types will produce
    // different sequences due to hash truncation
    let key = "same_key";
    let count = 10;

    let hasher_u32 = DoubleHashBuilder::<u32>::new()
        .with_n(1000)
        .build_hash_iter_hasher();
    let hasher_u64 = DoubleHashBuilder::<u64>::new()
        .with_n(1000)
        .build_hash_iter_hasher();

    let hashes_u32: Vec<u32> = hasher_u32.hash_iter(&key, count).collect();
    let hashes_u64: Vec<u64> = hasher_u64.hash_iter(&key, count).collect();

    // Both should have same length
    assert_eq!(hashes_u32.len(), count);
    assert_eq!(hashes_u64.len(), count);

    // All values should be within range [0, 1000)
    for &hash in &hashes_u32 {
        assert!(hash < 1000);
    }
    for &hash in &hashes_u64 {
        assert!(hash < 1000);
    }

    // Values will be different due to u64->u32 truncation before modulo operation
    // This is expected behavior
    assert_ne!(hashes_u32[0] as u64, hashes_u64[0]);
}

#[test]
fn test_custom_hash_builders_with_u32() {
    use xxhash_rust::xxh3::Xxh3Builder;

    let hasher = DoubleHashHasher::<u32, _, _>::with_hash_builders(
        Xxh3Builder::new().with_seed(42),
        Xxh3Builder::new().with_seed(84),
        u32::MAX,
    );

    let hashes: Vec<u32> = hasher.hash_iter(&"custom_builder", 8).collect();

    assert_eq!(hashes.len(), 8);
    // Verify deterministic behavior
    let hashes2: Vec<u32> = hasher.hash_iter(&"custom_builder", 8).collect();
    assert_eq!(hashes, hashes2);
}

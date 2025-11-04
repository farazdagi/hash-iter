use hash_iter::{BuildHashIterHasher, DoubleHashBuilder, DoubleHashHasher, HashIterHasher};

#[test]
fn default_config() {
    {
        // Implicit builder object.
        let hasher = DoubleHashHasher::new();
        let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();

        assert_eq!(hashes, vec![
            10179864958193109059,
            16936771314159985077,
            5246933596417309481
        ]);
    }

    {
        // Explicit builder object.
        let builder: DoubleHashBuilder = DoubleHashBuilder::new();
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
    let hasher = DoubleHashBuilder::new()
        .with_seed1(12345)
        .with_seed2(67890)
        .with_n(usize::MAX)
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
    use xxhash_rust::xxh3::Xxh3Builder;

    let hasher = DoubleHashHasher::with_hash_builders(
        Xxh3Builder::new().with_seed(12345),
        Xxh3Builder::new().with_seed(67890),
        usize::MAX,
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
            hasher: DoubleHashHasher,
        }

        impl Foo {
            fn new() -> Self {
                Self {
                    hasher: DoubleHashHasher::new(),
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
        use xxhash_rust::xxh3::Xxh3Builder;
        struct Foo {
            hasher: DoubleHashHasher<u64, Xxh3Builder, Xxh3Builder>,
        }

        impl Foo {
            fn new() -> Self {
                Self {
                    hasher: DoubleHashHasher::new(),
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

    let mut iter = Hashes::new(hash1, hash2, n, k);

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

    let iter = Hashes::new(hash1, hash2, n, k);
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
    let mut iter = Hashes::new(hash1, hash2, n, k);

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
    let mut iter1 = Hashes::new(hash1, hash2, n, k);
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
    let mut fresh = Hashes::new(hash1, hash2, n, k);
    fresh.next();
    let val2_from_fresh = fresh.next();

    assert_eq!(
        val2_from_iter2, val2_from_fresh,
        "Cloned iterator should be independent of original"
    );

    // Test 4: Clone with Copy trait (not only Clone, but also Copy)
    let iter = Hashes::new(hash1, hash2, n, 3);
    let copy_of_iter = iter; // This is a copy operation since Hashes derives Copy

    // Both should be usable independently
    let from_original: Vec<_> = iter.collect();
    let from_copy: Vec<_> = copy_of_iter.collect();

    assert_eq!(
        from_original, from_copy,
        "Copy should work the same as Clone for iterator state"
    );

    // Test 5: Multiple checkpoints at different positions
    let mut iter = Hashes::new(hash1, hash2, n, 10);

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

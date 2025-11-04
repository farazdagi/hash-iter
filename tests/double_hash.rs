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

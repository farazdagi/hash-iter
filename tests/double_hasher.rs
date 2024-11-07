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
            5246933596417309480
        ]);
    }

    {
        // Explicit builder object.
        let builder = DoubleHashBuilder::new();
        let hasher = builder.build_hash_iter_hasher();
        let hashes = hasher.hash_iter(&"hello", 3).collect::<Vec<_>>();

        assert_eq!(hashes, vec![
            10179864958193109059,
            16936771314159985077,
            5246933596417309480
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
        5246933596417309480
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
        5246933596417309480
    ]);
}

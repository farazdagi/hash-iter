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
            7899665402360655699
        ]);
    }
    {
        // Explicit builder types.
        use xxhash_rust::xxh3::Xxh3Builder;
        struct Foo {
            hasher: DoubleHashHasher<Xxh3Builder, Xxh3Builder>,
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
            7899665402360655699
        ]);
    }
}

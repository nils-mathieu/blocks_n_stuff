use crate::{utility, Rng};

/// A general-purpose pseudo-random number generator.
///
/// This number generator is based on the [xoshiro256++][source].
///
/// [source]: https://prng.di.unimi.it/xoshiro256plusplus.c
#[derive(Debug, Clone)]
pub struct Xoshiro256PlusPlus {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl Rng for Xoshiro256PlusPlus {
    fn from_seed(seed: u64) -> Self
    where
        Self: Sized,
    {
        let a = utility::splitmix64(seed);
        let b = utility::splitmix64(a);
        let c = utility::splitmix64(b);
        let d = utility::splitmix64(c);

        Self { a, b, c, d }
    }

    fn next_u64(&mut self) -> u64 {
        let ret = self
            .a
            .wrapping_add(self.c)
            .rotate_left(23)
            .wrapping_add(self.a);

        let t = self.b << 17;

        self.c ^= self.a;
        self.d ^= self.b;
        self.b ^= self.c;
        self.a ^= self.d;

        self.c ^= t;

        self.d = self.d.rotate_left(45);

        ret
    }
}

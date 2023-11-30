use std::ops::BitXor;

use crate::{FromRng, Rng};

/// Contains the state required to mix `N` numbers.
#[derive(Debug, Clone)]
pub struct Mixer<const N: usize> {
    /// The initial value used to hash the input numbers.
    pub init: u64,
    /// A bunch of prime numbers used to multiply the
    /// input numbers with.
    pub primes: [u64; N],
}

impl<const N: usize> FromRng for Mixer<N> {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            init: rng.next_u64(),
            primes: std::array::from_fn(|_| crate::utility::generate_prime(rng)),
        }
    }
}

impl<const N: usize> Mixer<N> {
    /// Mixes the provided input numbers into a single one.
    pub fn mix_u64(&self, input: [u64; N]) -> u64 {
        self.mix_impl(input.into_iter())
    }

    /// Mixes the provided input numbers into a single one.
    pub fn mix_u32(&self, input: [u32; N]) -> u32 {
        self.mix_impl(input.into_iter().map(|x| x as u64)) as u32
    }

    /// Mixes the provided input numbers into a single one.
    pub fn mix_i32(&self, input: [i32; N]) -> u32 {
        self.mix_impl(input.into_iter().map(|x| x as u32 as u64)) as u32
    }

    fn mix_impl(&self, input: impl Iterator<Item = u64>) -> u64 {
        let mut ret = self.init;
        for (t, p) in input.zip(self.primes) {
            ret = ret.rotate_left(5).bitxor(t).wrapping_mul(p);
        }
        ret
    }
}

use std::ops::BitXor;

use crate::{FromRng, Noise, Rng};

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

impl<const N: usize> Noise<[u64; N]> for Mixer<N> {
    type Output = u64;

    fn sample(&self, input: [u64; N]) -> Self::Output {
        let mut ret = self.init;
        for (t, p) in input.into_iter().zip(self.primes) {
            ret = ret.rotate_left(5).bitxor(t).wrapping_mul(p);
        }
        ret
    }
}

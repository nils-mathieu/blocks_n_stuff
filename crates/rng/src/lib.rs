//! A pseudo-random number generation library.

pub mod utility;

mod mixer;
pub use mixer::*;

pub mod noises;
pub mod rngs;

pub use bns_rng_derive::FromRng;

/// The default pseudo-random number generator.
///
/// This general purpose RNG should be sufficient in a vast majority of cases.
pub type DefaultRng = rngs::Xoshiro256PlusPlus;

/// A seeded pseudo-random number generator.
pub trait Rng {
    /// Creates a new [`Rng`] instance from the provided seed.
    fn from_seed(seed: u64) -> Self
    where
        Self: Sized;

    /// Generates a pseudo-random `u64` value.
    fn next_u64(&mut self) -> u64;

    /// Generates a random `u32` value.
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Generates a random `f32` value in the range `[0.0, 1.0]`.
    #[inline(always)]
    fn next_f32_01(&mut self) -> f32 {
        utility::f32_from_u32_01(self.next_u32())
    }

    /// Generates a random `f32` value in the range `[-1.0, 1.0]`.
    #[inline(always)]
    fn next_f32_11(&mut self) -> f32 {
        utility::f32_from_u32_11(self.next_u32())
    }
}

/// A trait for types that can be generated from a random number generator.
pub trait FromRng {
    /// Generates a new instance of `Self` from the provided random number generator.
    fn from_rng(rng: &mut impl Rng) -> Self;
}

impl FromRng for u32 {
    #[inline]
    fn from_rng(rng: &mut impl Rng) -> Self {
        rng.next_u32()
    }
}

impl FromRng for u64 {
    #[inline]
    fn from_rng(rng: &mut impl Rng) -> Self {
        rng.next_u64()
    }
}

impl FromRng for f32 {
    #[inline]
    fn from_rng(rng: &mut impl Rng) -> Self {
        rng.next_f32_01()
    }
}

/// A trait for types that can map an input to a (usually continuous) pseudorandom output.
pub trait Noise<I> {
    /// The output of this noise.
    type Output;

    /// Samples the provided input.
    fn sample(&self, input: I) -> Self::Output;
}

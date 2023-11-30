/// A simple implementation of the [SplitMix64] algorithm.
///
/// This is mainly used to turn a 64-bit seed into a sequence of 64-bit numbers to use as
/// the base state for the main random number generator.
///
/// [SplitMix64]: http://prng.di.unimi.it/splitmix64.c
pub fn splitmix64(st: u64) -> u64 {
    let mut t = st.wrapping_add(0x9e3779b97f4a7c15);
    t = (t ^ (t >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    t = (t ^ (t >> 27)).wrapping_mul(0x94d049bb133111eb);
    t = t ^ (t >> 31);
    t
}

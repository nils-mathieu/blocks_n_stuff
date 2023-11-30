//! Utilities to generate prime numbers.

use crate::Rng;

/// The first few prime numbers, used for primality testing.
const FIRST_PRIMES: [u64; 200] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307,
    311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421,
    431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541, 547,
    557, 563, 569, 571, 577, 587, 593, 599, 601, 607, 613, 617, 619, 631, 641, 643, 647, 653, 659,
    661, 673, 677, 683, 691, 701, 709, 719, 727, 733, 739, 743, 751, 757, 761, 769, 773, 787, 797,
    809, 811, 821, 823, 827, 829, 839, 853, 857, 859, 863, 877, 881, 883, 887, 907, 911, 919, 929,
    937, 941, 947, 953, 967, 971, 977, 983, 991, 997, 1009, 1013, 1019, 1021, 1031, 1033, 1039,
    1049, 1051, 1061, 1063, 1069, 1087, 1091, 1093, 1097, 1103, 1109, 1117, 1123, 1129, 1151, 1153,
    1163, 1171, 1181, 1187, 1193, 1201, 1213, 1217, 1223,
];

/// Computes `a * b (mod m)`.
#[inline]
fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

/// Computes `base ^ 2 (mod m)`.
fn expmod(base: u64, mut e: u64, m: u64) -> u64 {
    // Use the binary exponentiation algorithm.
    let mut result = 1;
    let mut base = base % m;
    while e > 0 {
        if e % 2 == 1 {
            result = mulmod(result, base, m);
        }

        e >>= 1;
        base = mulmod(base, base, m);
    }
    result
}

/// Returns whether `n` is likely to be a prime number.
pub fn is_prime(n: u64, rng: &mut impl Rng) -> bool {
    // Implementation taken from:
    //  https://www.geeksforgeeks.org/how-to-generate-large-prime-numbers-for-rsa-algorithm/

    for &p in FIRST_PRIMES.iter() {
        if n % p == 0 {
            return false;
        }
    }

    // TODO:
    //  Find a sweet spot for the number of iterations.

    /// The number of iterations to perform for the Miller-Rabin primality test.
    const ITERATIONS: usize = 10;

    let mut max_divisions_by_two = 0;
    let mut even_component = n - 1;

    while even_component % 2 == 0 {
        even_component /= 2;
        max_divisions_by_two += 1;
    }

    let trial_composite = move |round_tester: u64| {
        if expmod(round_tester, even_component, n) == 1 {
            return false;
        }

        for i in 0..max_divisions_by_two {
            if expmod(round_tester, (1 << i) * even_component, n) == n - 1 {
                return false;
            }
        }

        true
    };

    for _ in 0..ITERATIONS {
        let round_tester = rng.next_u64() % (n - 2) + 2;
        if trial_composite(round_tester) {
            return false;
        }
    }

    true
}

/// Generates a random prime number.
///
/// # Remarks
///
/// The returned number is only *likely* to be prime, but it is not strictly guaranteed. For all
/// practical purposes, this is good enough.
pub fn generate_prime(rng: &mut impl Rng) -> u64 {
    loop {
        let n = rng.next_u64();

        if is_prime(n, rng) {
            return n;
        }
    }
}

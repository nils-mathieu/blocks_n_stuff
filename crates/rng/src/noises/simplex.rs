use crate::{FromRng, Noise, Rng};

const UNSKEW_2D: f32 = -0.21132486f32;
const RSQUARED_2D: f32 = 2.0f32 / 3.0f32;
const HASH_MULTIPLIER: u64 = 0x53A3F72DEEC546F5;
const N_GRADS_2D_EXPONENT: u32 = 7;
const N_GRADS_2D: u32 = 1 << N_GRADS_2D_EXPONENT;
const SKEW_2D: f32 = 0.3660254f32;

/// A 2D simplex noise implementation.
#[derive(Debug, Clone)]
pub struct Simplex2 {
    seed: u64,
    prime_x: u64,
    prime_y: u64,
}

impl FromRng for Simplex2 {
    fn from_rng(rng: &mut impl Rng) -> Self {
        Self {
            seed: rng.next_u64(),
            prime_x: crate::utility::generate_prime(rng),
            prime_y: crate::utility::generate_prime(rng),
        }
    }
}

impl Noise<[f32; 2]> for Simplex2 {
    type Output = f32;

    /// Samples the provided position in the noise field.
    ///
    /// # Returns
    ///
    /// This function returns the sampled value in the range `[-1.0, 1.0]`.
    fn sample(&self, [x, y]: [f32; 2]) -> f32 {
        let seed = self.seed;

        let s = SKEW_2D * (x + y);
        let xs = x + s;
        let ys = y + s;

        // Get base points and offsets.
        let xsb = crate::utility::floor_i32(xs);
        let ysb = crate::utility::floor_i32(ys);

        let xi = xs - xsb as f32;
        let yi = ys - ysb as f32;

        // Prime pre-multiplication for hash.
        let xsbp = (xsb as u64).wrapping_mul(self.prime_x);
        let ysbp = (ysb as u64).wrapping_mul(self.prime_y);

        // Unskew.
        let t = (xi + yi) * UNSKEW_2D;
        let dx0 = xi + t;
        let dy0 = yi + t;

        // First vertex.
        let a0 = RSQUARED_2D - dx0 * dx0 - dy0 * dy0;
        let mut value = (a0 * a0) * (a0 * a0) * grad(seed, xsbp, ysbp, dx0, dy0);

        // Second vertex.
        let a1 = (2.0 * (1.0 + 2.0 * UNSKEW_2D) * (1.0 / UNSKEW_2D + 2.0)) * t
            + ((-2.0 * (1.0 + 2.0 * UNSKEW_2D) * (1.0 + 2.0 * UNSKEW_2D)) + a0);
        let dx1 = dx0 - (1.0 + 2.0 * UNSKEW_2D);
        let dy1 = dy0 - (1.0 + 2.0 * UNSKEW_2D);
        value += (a1 * a1)
            * (a1 * a1)
            * grad(
                seed,
                xsbp.wrapping_add(self.prime_x),
                ysbp.wrapping_add(self.prime_y),
                dx1,
                dy1,
            );

        // Third and fourth vertices.
        // Nested conditionals were faster than compact bit logic/arithmetic.
        let xmyi = xi - yi;
        if t < UNSKEW_2D {
            if xi + xmyi > 1.0 {
                let dx2 = dx0 - (3.0 * UNSKEW_2D + 2.0);
                let dy2 = dy0 - (3.0 * UNSKEW_2D + 1.0);
                let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
                if a2 > 0.0 {
                    value += (a2 * a2)
                        * (a2 * a2)
                        * grad(
                            seed,
                            xsbp.wrapping_add(self.prime_x << 1),
                            ysbp.wrapping_add(self.prime_y),
                            dx2,
                            dy2,
                        );
                }
            } else {
                let dx2 = dx0 - UNSKEW_2D;
                let dy2 = dy0 - (UNSKEW_2D + 1.0);
                let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
                if a2 > 0.0 {
                    value += (a2 * a2)
                        * (a2 * a2)
                        * grad(seed, xsbp, ysbp.wrapping_add(self.prime_y), dx2, dy2);
                }
            }

            if yi - xmyi > 1.0 {
                let dx3 = dx0 - (3.0 * UNSKEW_2D + 1.0);
                let dy3 = dy0 - (3.0 * UNSKEW_2D + 2.0);
                let a3 = RSQUARED_2D - dx3 * dx3 - dy3 * dy3;
                if a3 > 0.0 {
                    value += (a3 * a3)
                        * (a3 * a3)
                        * grad(
                            seed,
                            xsbp.wrapping_add(self.prime_x),
                            ysbp.wrapping_add(self.prime_y << 1),
                            dx3,
                            dy3,
                        );
                }
            } else {
                let dx3 = dx0 - (UNSKEW_2D + 1.0);
                let dy3 = dy0 - UNSKEW_2D;
                let a3 = RSQUARED_2D - dx3 * dx3 - dy3 * dy3;
                if a3 > 0.0 {
                    value += (a3 * a3)
                        * (a3 * a3)
                        * grad(seed, xsbp.wrapping_add(self.prime_x), ysbp, dx3, dy3);
                }
            }
        } else {
            if xi + xmyi < 0.0 {
                let dx2 = dx0 + (1.0 + UNSKEW_2D);
                let dy2 = dy0 + UNSKEW_2D;
                let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
                if a2 > 0.0 {
                    value += (a2 * a2)
                        * (a2 * a2)
                        * grad(seed, xsbp.wrapping_sub(self.prime_x), ysbp, dx2, dy2);
                }
            } else {
                let dx2 = dx0 - (UNSKEW_2D + 1.0);
                let dy2 = dy0 - UNSKEW_2D;
                let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
                if a2 > 0.0 {
                    value += (a2 * a2)
                        * (a2 * a2)
                        * grad(seed, xsbp.wrapping_add(self.prime_x), ysbp, dx2, dy2);
                }
            }

            if yi < xmyi {
                let dx2 = dx0 + UNSKEW_2D;
                let dy2 = dy0 + (UNSKEW_2D + 1.0);
                let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
                if a2 > 0.0 {
                    value += (a2 * a2)
                        * (a2 * a2)
                        * grad(seed, xsbp, ysbp.wrapping_sub(self.prime_y), dx2, dy2);
                }
            } else {
                let dx2 = dx0 - UNSKEW_2D;
                let dy2 = dy0 - (UNSKEW_2D + 1.0);
                let a2 = RSQUARED_2D - dx2 * dx2 - dy2 * dy2;
                if a2 > 0.0 {
                    value += (a2 * a2)
                        * (a2 * a2)
                        * grad(seed, xsbp, ysbp.wrapping_add(self.prime_y), dx2, dy2);
                }
            }
        }

        value
    }
}

fn grad(seed: u64, xsvp: u64, ysvp: u64, dx: f32, dy: f32) -> f32 {
    let mut hash = seed ^ xsvp ^ ysvp;
    hash = hash.wrapping_mul(HASH_MULTIPLIER);
    hash ^= hash >> (64 - N_GRADS_2D_EXPONENT as u64 + 1);
    let gi = hash & ((N_GRADS_2D as u64 - 1) << 1);
    GRADIENTS_2D[gi as usize] * dx + GRADIENTS_2D[(gi | 1) as usize] * dy
}

#[rustfmt::skip]
const GRADIENTS_2D: [f32; N_GRADS_2D as usize * 2] = [
    6.9808965,
    16.853374,
    16.853374,
    6.9808965,
    16.853374,
    -6.9808965,
    6.9808965,
    -16.853374,
    -6.9808965,
    -16.853374,
    -16.853374,
    -6.9808965,
    -16.853374,
    6.9808965,
    -6.9808965,
    16.853374,
    2.3810537,
    18.0859,
    11.105003,
    14.4723215,
    14.4723215,
    11.105003,
    18.0859,
    2.3810537,
    18.0859,
    -2.3810537,
    14.4723215,
    -11.105003,
    11.105003,
    -14.4723215,
    2.3810537,
    -18.0859,
    -2.3810537,
    -18.0859,
    -11.105003,
    -14.4723215,
    -14.4723215,
    -11.105003,
    -18.0859,
    -2.3810537,
    -18.0859,
    2.3810537,
    -14.4723215,
    11.105003,
    -11.105003,
    14.4723215,
    -2.3810537,
    18.0859,
    6.9808965,
    16.853374,
    16.853374,
    6.9808965,
    16.853374,
    -6.9808965,
    6.9808965,
    -16.853374,
    -6.9808965,
    -16.853374,
    -16.853374,
    -6.9808965,
    -16.853374,
    6.9808965,
    -6.9808965,
    16.853374,
    2.3810537,
    18.0859,
    11.105003,
    14.4723215,
    14.4723215,
    11.105003,
    18.0859,
    2.3810537,
    18.0859,
    -2.3810537,
    14.4723215,
    -11.105003,
    11.105003,
    -14.4723215,
    2.3810537,
    -18.0859,
    -2.3810537,
    -18.0859,
    -11.105003,
    -14.4723215,
    -14.4723215,
    -11.105003,
    -18.0859,
    -2.3810537,
    -18.0859,
    2.3810537,
    -14.4723215,
    11.105003,
    -11.105003,
    14.4723215,
    -2.3810537,
    18.0859,
    6.9808965,
    16.853374,
    16.853374,
    6.9808965,
    16.853374,
    -6.9808965,
    6.9808965,
    -16.853374,
    -6.9808965,
    -16.853374,
    -16.853374,
    -6.9808965,
    -16.853374,
    6.9808965,
    -6.9808965,
    16.853374,
    2.3810537,
    18.0859,
    11.105003,
    14.4723215,
    14.4723215,
    11.105003,
    18.0859,
    2.3810537,
    18.0859,
    -2.3810537,
    14.4723215,
    -11.105003,
    11.105003,
    -14.4723215,
    2.3810537,
    -18.0859,
    -2.3810537,
    -18.0859,
    -11.105003,
    -14.4723215,
    -14.4723215,
    -11.105003,
    -18.0859,
    -2.3810537,
    -18.0859,
    2.3810537,
    -14.4723215,
    11.105003,
    -11.105003,
    14.4723215,
    -2.3810537,
    18.0859,
    6.9808965,
    16.853374,
    16.853374,
    6.9808965,
    16.853374,
    -6.9808965,
    6.9808965,
    -16.853374,
    -6.9808965,
    -16.853374,
    -16.853374,
    -6.9808965,
    -16.853374,
    6.9808965,
    -6.9808965,
    16.853374,
    2.3810537,
    18.0859,
    11.105003,
    14.4723215,
    14.4723215,
    11.105003,
    18.0859,
    2.3810537,
    18.0859,
    -2.3810537,
    14.4723215,
    -11.105003,
    11.105003,
    -14.4723215,
    2.3810537,
    -18.0859,
    -2.3810537,
    -18.0859,
    -11.105003,
    -14.4723215,
    -14.4723215,
    -11.105003,
    -18.0859,
    -2.3810537,
    -18.0859,
    2.3810537,
    -14.4723215,
    11.105003,
    -11.105003,
    14.4723215,
    -2.3810537,
    18.0859,
    6.9808965,
    16.853374,
    16.853374,
    6.9808965,
    16.853374,
    -6.9808965,
    6.9808965,
    -16.853374,
    -6.9808965,
    -16.853374,
    -16.853374,
    -6.9808965,
    -16.853374,
    6.9808965,
    -6.9808965,
    16.853374,
    2.3810537,
    18.0859,
    11.105003,
    14.4723215,
    14.4723215,
    11.105003,
    18.0859,
    2.3810537,
    18.0859,
    -2.3810537,
    14.4723215,
    -11.105003,
    11.105003,
    -14.4723215,
    2.3810537,
    -18.0859,
    -2.3810537,
    -18.0859,
    -11.105003,
    -14.4723215,
    -14.4723215,
    -11.105003,
    -18.0859,
    -2.3810537,
    -18.0859,
    2.3810537,
    -14.4723215,
    11.105003,
    -11.105003,
    14.4723215,
    -2.3810537,
    18.0859,
    6.9808965,
    16.853374,
    16.853374,
    6.9808965,
    16.853374,
    -6.9808965,
    6.9808965,
    -16.853374,
    -6.9808965,
    -16.853374,
    -16.853374,
    -6.9808965,
    -16.853374,
    6.9808965,
    -6.9808965,
    16.853374,
];

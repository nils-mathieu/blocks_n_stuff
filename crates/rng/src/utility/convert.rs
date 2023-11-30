/// Converts a `u32` value into a `f32` value in the range `[0.0, 1.0]`.
#[inline]
pub fn f32_from_u32_01(x: u32) -> f32 {
    // f32::from_bits(0x3F80_0000 | (x >> 9)) - 1.0
    (x & 0xFFFFFF) as f32 * (1.0 / 0xFFFFFF as f32)
}

/// Converts a `u32` value into a `f32` value in the range `[-1.0, 1.0]`.
#[inline]
pub fn f32_from_u32_11(x: u32) -> f32 {
    // `f32_from_u32_01` does not use the most significant bit of `x`, meaning we can use it for
    // the sign bit.
    if x & 0x1000_0000 != 0 {
        -f32_from_u32_01(x)
    } else {
        f32_from_u32_01(x)
    }
}

/// Returns the largest `i32` value that is less than or equal to `x`.
#[inline]
pub fn floor_i32(x: f32) -> i32 {
    if x >= 0.0 {
        x as i32
    } else {
        x as i32 - 1
    }
}

//! Provide entropy from the operating system.

/// Returns a random `u64` value.
pub fn entropy() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        let bytes: [u8; 8] = std::array::from_fn(|_| (js_sys::Math::random() * 256.0) as u8);
        u64::from_ne_bytes(bytes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut bytes = [0u8; 8];
        let _ = getrandom::getrandom(&mut bytes);
        u64::from_ne_bytes(bytes)
    }
}

#[path = "rand/chacha.rs"]
mod chacha;
#[path = "rand/system.rs"]
pub mod system;
#[path = "rand/value.rs"]
mod value;

pub use system::SystemRandom;
pub use value::RandomValue;

/// 10,000 u64 values per buffer, two buffers in total.
pub const RAND_CACHE_BYTES: usize = 10_000 * 8;
/// Development-only reproducible seed. Not for secrets or security tokens.
pub const FIXED_SEED: [u8; 32] = [42; 32];

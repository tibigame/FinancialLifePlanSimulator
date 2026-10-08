#[path = "../../../src-tauri/src/rand.rs"]
pub mod rand;

use chacha20::{
    ChaCha12Rng,
    rand_core::{Rng, SeedableRng},
};
use rand::{FIXED_SEED, RAND_CACHE_BYTES, SystemRandom};
use std::{hint::black_box, io, time::Instant};

const COUNT: usize = 10_000;
const SAMPLES: usize = 11;
const REPETITIONS: usize = 1_000;

fn measure(mut operation: impl FnMut() -> io::Result<()>) -> io::Result<f64> {
    for _ in 0..100 {
        operation()?;
    }
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let start = Instant::now();
        for _ in 0..REPETITIONS {
            operation()?;
        }
        samples.push(start.elapsed().as_secs_f64() * 1e6 / REPETITIONS as f64);
    }
    samples.sort_by(f64::total_cmp);
    Ok(samples[SAMPLES / 2])
}

fn report(name: &str, microseconds: f64) {
    println!(
        "{name},{microseconds:.3},{:.1}",
        RAND_CACHE_BYTES as f64 / microseconds
    );
}

fn main() -> io::Result<()> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    println!(
        "cpu,sse2={},avx2={},avx512f={},avx512vl={},avx512bw={}",
        std::is_x86_feature_detected!("sse2"),
        std::is_x86_feature_detected!("avx2"),
        std::is_x86_feature_detected!("avx512f"),
        std::is_x86_feature_detected!("avx512vl"),
        std::is_x86_feature_detected!("avx512bw")
    );
    println!(
        "batch,u64_count={COUNT},bytes={RAND_CACHE_BYTES},samples={SAMPLES},repetitions={REPETITIONS}"
    );

    // Verify the complete stream across multiple buffers, then emit a stable
    // fingerprint that the runner also compares between backend builds.
    let mut reference = vec![0; RAND_CACHE_BYTES * 4];
    ChaCha12Rng::from_seed(FIXED_SEED).fill_bytes(&mut reference);
    let mut random = SystemRandom::from_seed(FIXED_SEED)?;
    if random.get::<u8>(reference.len())? != reference {
        return Err(io::Error::other("Buffered stream differs from reference"));
    }
    let fingerprint = reference.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    println!("fingerprint,{fingerprint:016x}");
    drop(random);

    println!("method,median_us_per_10000_u64,MB_per_second");
    let mut bytes = vec![0; RAND_CACHE_BYTES];
    let mut direct = ChaCha12Rng::from_seed(FIXED_SEED);
    report(
        "direct_fill_bytes",
        measure(|| {
            direct.fill_bytes(black_box(&mut bytes));
            black_box(&bytes);
            Ok(())
        })?,
    );

    let mut random = SystemRandom::from_seed(FIXED_SEED)?;
    report(
        "double_buffer_fill_bytes",
        measure(|| {
            random.fill_bytes(black_box(&mut bytes))?;
            black_box(&bytes);
            Ok(())
        })?,
    );
    drop(random);

    let mut random = SystemRandom::from_seed(FIXED_SEED)?;
    let mut values = vec![0u64; COUNT];
    report(
        "double_buffer_fill_u64",
        measure(|| {
            random.fill(black_box(&mut values))?;
            black_box(&values);
            Ok(())
        })?,
    );
    drop(random);

    let mut random = SystemRandom::from_seed(FIXED_SEED)?;
    report(
        "double_buffer_get_u64_alloc",
        measure(|| {
            black_box(random.get::<u64>(black_box(COUNT))?);
            Ok(())
        })?,
    );
    drop(random);

    let mut cold = Vec::with_capacity(101);
    for _ in 0..101 {
        let start = Instant::now();
        let mut random = SystemRandom::new(true)?;
        black_box(random.get::<u64>(COUNT)?);
        cold.push(start.elapsed().as_secs_f64() * 1e6);
        drop(random); // Exclude shutdown; include seed selection, spawn and first fill.
    }
    cold.sort_by(f64::total_cmp);
    report("startup_and_first_get_fixed_seed", cold[50]);
    Ok(())
}

#[cfg(test)]
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[cfg(test)]
#[path = "../../../src-tauri/tests/unit/random.rs"]
mod random_tests;

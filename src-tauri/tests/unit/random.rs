use super::TestResult;
use crate::rand::{FIXED_SEED, RAND_CACHE_BYTES, SystemRandom};
use chacha20::{
    ChaCha12Rng,
    rand_core::{Rng, SeedableRng},
};

#[test]
fn mixed_reads_preserve_every_byte_across_buffer_boundaries() -> TestResult {
    let mut random = SystemRandom::new(true)?;
    let length = RAND_CACHE_BYTES * 4 + 137;
    let mut expected = vec![0; length];
    ChaCha12Rng::from_seed(FIXED_SEED).fill_bytes(&mut expected);
    let mut actual = random.get::<u8>(RAND_CACHE_BYTES - 1)?;
    for value in random.get::<u16>(10)? {
        actual.extend(value.to_le_bytes());
    }
    for value in random.get::<u64>(10_000)? {
        actual.extend(value.to_le_bytes());
    }
    for value in random.get::<i128>(13)? {
        actual.extend(value.to_le_bytes());
    }
    let remaining = length - actual.len();
    actual.extend(random.get::<u8>(remaining)?);
    assert_eq!(actual, expected);
    assert_eq!(random.seed(), FIXED_SEED);
    Ok(())
}

#[test]
fn request_partition_and_scheduling_do_not_change_output() -> TestResult {
    let mut one = SystemRandom::new(true)?;
    let mut split = SystemRandom::new(true)?;
    let expected = one.get::<u8>(RAND_CACHE_BYTES * 3 + 15)?;
    let mut actual = Vec::new();
    for count in [1, 3, 7, RAND_CACHE_BYTES, 4, RAND_CACHE_BYTES * 2] {
        std::thread::yield_now();
        actual.extend(split.get::<u8>(count)?);
    }
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn empty_and_rejected_requests_do_not_consume_bytes() -> TestResult {
    let mut random = SystemRandom::new(true)?;
    assert!(random.get::<u64>(0)?.is_empty());
    random.fill_bytes(&mut [])?;
    assert!(random.get::<u128>(usize::MAX).is_err());
    let mut reference = SystemRandom::new(true)?;
    assert_eq!(random.get::<u32>(20)?, reference.get::<u32>(20)?);
    Ok(())
}

#[test]
fn system_seed_can_be_replayed() -> TestResult {
    let mut random = SystemRandom::new(false)?;
    let mut replay = SystemRandom::from_seed(random.seed())?;
    assert_eq!(random.get::<u64>(10_000)?, replay.get::<u64>(10_000)?);
    Ok(())
}

#[test]
fn dropping_at_each_producer_stage_finishes() -> TestResult {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || -> std::io::Result<()> {
        for count in [
            0,
            1,
            RAND_CACHE_BYTES,
            RAND_CACHE_BYTES + 1,
            RAND_CACHE_BYTES * 3,
        ] {
            let mut random = SystemRandom::new(true)?;
            random.get::<u8>(count)?;
            drop(random);
        }
        done_tx.send(()).map_err(std::io::Error::other)
    });
    done_rx.recv_timeout(std::time::Duration::from_secs(10))?;
    worker.join().map_err(|_| "Drop test worker panicked")??;
    Ok(())
}

use super::RAND_CACHE_BYTES;
use chacha20::{
    ChaCha12Rng,
    rand_core::{Rng, SeedableRng},
};
use std::{
    io,
    sync::mpsc::{self, Receiver, SyncSender},
    thread::{self, JoinHandle},
};

type Producer = (Receiver<Vec<u8>>, SyncSender<Vec<u8>>, JoinHandle<()>);

pub(super) fn spawn(seed: [u8; 32]) -> io::Result<Producer> {
    let allocate = || -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(RAND_CACHE_BYTES)
            .map_err(io::Error::other)?;
        buffer.resize(RAND_CACHE_BYTES, 0);
        Ok(buffer)
    };
    let initial = [allocate()?, allocate()?];
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (recycle_tx, recycle_rx) = mpsc::sync_channel::<Vec<u8>>(1);
    let worker = thread::Builder::new()
        .name("simulation-random".into())
        .spawn(move || {
            let mut rng = ChaCha12Rng::from_seed(seed);
            // Always finish and publish blocks in stream order. Never reseed on refill.
            for mut buffer in initial {
                rng.fill_bytes(&mut buffer);
                if ready_tx.send(buffer).is_err() {
                    return;
                }
            }
            while let Ok(mut buffer) = recycle_rx.recv() {
                rng.fill_bytes(&mut buffer);
                if ready_tx.send(buffer).is_err() {
                    return;
                }
            }
        })?;
    Ok((ready_rx, recycle_tx, worker))
}

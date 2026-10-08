use super::{FIXED_SEED, RAND_CACHE_BYTES, RandomValue, chacha};
use ::rand::{TryRng, rngs::SysRng};
use std::{
    io,
    sync::mpsc::{Receiver, SyncSender},
    thread::JoinHandle,
};

/// A single consumer owns this byte stream; only the producer owns ChaCha12Rng.
/// Reads consume exactly the requested bytes, independent of request boundaries.
pub struct SystemRandom {
    current: Vec<u8>,
    cursor: usize,
    consumed_buffers: usize,
    ready: Option<Receiver<Vec<u8>>>,
    recycle: Option<SyncSender<Vec<u8>>>,
    worker: Option<JoinHandle<()>>,
    seed: [u8; 32],
}

impl SystemRandom {
    pub fn new(fixed_seed: bool) -> io::Result<Self> {
        let mut seed = FIXED_SEED;
        if !fixed_seed {
            SysRng.try_fill_bytes(&mut seed).map_err(io::Error::other)?;
        }
        Self::from_seed(seed)
    }

    pub fn from_seed(seed: [u8; 32]) -> io::Result<Self> {
        let (ready, recycle, worker) = chacha::spawn(seed)?;
        Ok(Self {
            current: Vec::new(),
            cursor: 0,
            consumed_buffers: 0,
            ready: Some(ready),
            recycle: Some(recycle),
            worker: Some(worker),
            seed,
        })
    }

    /// Keep this with a simulation result to replay the same byte stream.
    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    /// Number of buffers fully consumed, excluding partially read or prefetched buffers.
    pub fn consumed_buffers(&self) -> usize {
        self.consumed_buffers
    }

    fn advance_cursor(&mut self, count: usize) {
        self.cursor += count;
        if self.cursor == self.current.len() {
            self.consumed_buffers += 1;
        }
    }

    fn ensure_buffer(&mut self) -> io::Result<()> {
        if self.cursor < self.current.len() {
            return Ok(());
        }
        if !self.current.is_empty() {
            let consumed = std::mem::take(&mut self.current);
            self.recycle
                .as_ref()
                .ok_or_else(disconnected)?
                .send(consumed)
                .map_err(|_| disconnected())?;
        }
        self.current = self
            .ready
            .as_ref()
            .ok_or_else(disconnected)?
            .recv()
            .map_err(|_| disconnected())?;
        self.cursor = 0;
        debug_assert_eq!(self.current.len(), RAND_CACHE_BYTES);
        Ok(())
    }

    /// On a producer failure, the destination may have been partially filled.
    pub fn fill_bytes(&mut self, mut destination: &mut [u8]) -> io::Result<()> {
        while !destination.is_empty() {
            self.ensure_buffer()?;
            let count = destination.len().min(self.current.len() - self.cursor);
            destination[..count].copy_from_slice(&self.current[self.cursor..self.cursor + count]);
            self.advance_cursor(count);
            destination = &mut destination[count..];
        }
        Ok(())
    }

    /// Fill an existing integer slice without allocating. Integers use little endian.
    /// bool, floating point and platform-sized integers are intentionally excluded.
    pub fn fill<T: RandomValue>(&mut self, mut destination: &mut [T]) -> io::Result<()> {
        while !destination.is_empty() {
            self.ensure_buffer()?;
            let count = destination
                .len()
                .min((self.current.len() - self.cursor) / T::WIDTH);
            if count == 0 {
                let mut bytes = [0; 16];
                self.fill_bytes(&mut bytes[..T::WIDTH])?;
                destination[0] = T::decode(&bytes[..T::WIDTH]);
                destination = &mut destination[1..];
            } else {
                let end = self.cursor + count * T::WIDTH;
                for (value, bytes) in destination[..count]
                    .iter_mut()
                    .zip(self.current[self.cursor..end].chunks_exact(T::WIDTH))
                {
                    *value = T::decode(bytes);
                }
                self.advance_cursor(count * T::WIDTH);
                destination = &mut destination[count..];
            }
        }
        Ok(())
    }

    /// For repeated requests, prefer `fill` to reuse the caller's allocation.
    pub fn get<T: RandomValue>(&mut self, count: usize) -> io::Result<Vec<T>> {
        count.checked_mul(T::WIDTH).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Random request is too large")
        })?;
        let mut values = Vec::new();
        values.try_reserve_exact(count).map_err(io::Error::other)?;
        values.resize(count, T::default());
        self.fill(&mut values)?;
        Ok(values)
    }
}

fn disconnected() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "Random producer stopped")
}

impl Drop for SystemRandom {
    fn drop(&mut self) {
        // Disconnect both channels before joining: the producer may be blocked
        // either sending a ready buffer or waiting for a recycled buffer.
        drop(self.ready.take());
        drop(self.recycle.take());
        if let Some(worker) = self.worker.take() {
            // Drop cannot return Result. Reads already report producer failure.
            if worker.join().is_err() {
                eprintln!("Random producer panicked");
            }
        }
    }
}

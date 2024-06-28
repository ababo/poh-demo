use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
    time::interval,
};

/// Duration between hash batches.
pub const HASH_BATCH_PERIOD: Duration = Duration::from_millis(100);

/// Number of hashes per batch.
pub const HASH_BATCH_LENGTH: usize = 100;

/// Number of hash batches between two subsequent ticks.
pub const TICK_HASH_BATCHES: u64 = 10;

/// Sha256 hash alias.
pub type Hash = [u8; 32];

/// PohService output event.
#[derive(Debug)]
pub enum Event {
    Hash { state: Hash, batch: Vec<Hash> },
    Message { state: Hash, message: Vec<u8> },
    Tick { state: Hash, tick: u64 },
}

/// Proof-of-History service.
#[derive(Debug)]
pub struct PohService {
    state: Hash,
    tick: u64,
}

impl PohService {
    /// Create a new PohService instance.
    pub fn new(state: Hash) -> Self {
        Self { state, tick: 0 }
    }

    /// Process input messages and emit output events.
    pub async fn process(&mut self, mut messages: Receiver<Vec<u8>>, events: Sender<Event>) {
        let mut interval = interval(HASH_BATCH_PERIOD);
        let mut batches = 0;

        loop {
            select! {
                maybe_message = messages.recv() => {
                    let Some(message) = maybe_message else {
                        break;
                    };
                    self.hash(&message);
                    events.send(Event::Message {state: self.state, message}).await.unwrap();
                },
                _ = interval.tick() => {
                    let mut batch = vec![Hash::default(); HASH_BATCH_LENGTH];
                    for hash in batch.iter_mut() {
                        self.hash(&[]);
                        *hash = self.state;
                    }
                    events.send(Event::Hash {state: self.state, batch}).await.unwrap();
                    batches += 1;

                    if batches > 0 && batches % TICK_HASH_BATCHES == 0 {
                        self.hash(&[]);
                        self.tick += 1;
                        events.send(Event::Tick {state: self.state, tick: self.tick}).await.unwrap();
                    }
                }
            }
        }
    }

    fn hash(&mut self, message: &[u8]) {
        let mut sha256 = Sha256::new();
        sha256.update(self.state);
        sha256.update(message);
        self.state = sha256.finalize().into();
    }
}

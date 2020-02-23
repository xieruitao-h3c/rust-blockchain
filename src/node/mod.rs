mod server;
mod miner;

use std::sync::{Mutex, Arc, mpsc};
use crate::blockchain::Blockchain;
use crate::mempool::Mempool;
use crate::config::*;

pub fn start() {
    let min_tx_per_block = SETTINGS.get::<usize>("min_tx_per_block").unwrap();
    let difficulty = SETTINGS.get::<usize>("difficulty").unwrap();
    let concurrent_hashes = SETTINGS.get::<u64>("concurrent_hashes").unwrap();

    let blockchain = Arc::new(Mutex::new(Blockchain::new(min_tx_per_block, difficulty, concurrent_hashes)));
    let mempool = Arc::new(Mutex::new(Mempool::new()));

    let mut threads = vec![];
    let (tx, rx) = mpsc::channel();

    threads.push(server::start(tx, Arc::clone(&blockchain), Arc::clone(&mempool)));
    threads.push(miner::start(rx, Arc::clone(&blockchain), Arc::clone(&mempool)));

    for t in threads {
        let _ = t.join();
    }
}

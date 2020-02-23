use std::time::SystemTime;
use std::net::SocketAddr;
use rayon::prelude::*;
use crate::types::*;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use crate::config::*;

pub struct Blockchain {
    chain: Vec<Block>,
    min_tx_per_block: usize,
    difficulty: usize,
    concurrent_hashes: u64,
}

impl Blockchain {
    pub fn new(
        min_tx_per_block: usize,
        difficulty: usize,
        concurrent_hashes: u64,
    ) -> Self {
        Blockchain {
            chain: vec![],
            min_tx_per_block,
            difficulty,
            concurrent_hashes,
        }
    }

    pub fn get_all(&self) -> &Vec<Block> {
        &self.chain
    }

    pub fn get_concurrent_hashes(&self) -> u64 {
        self.concurrent_hashes
    }

    pub fn ok_to_mine(&self, txs: &[Tx]) -> bool {
        txs.len() >= self.min_tx_per_block
    }

    pub fn mine(
        &self,
        id: SocketAddr,
        nonce: u64,
        time: SystemTime,
        txs: Vec<Tx>,
    ) -> Option<Block> {
        let target = "0".repeat(self.difficulty);

        let mut nonces = vec![];
        for i in 0..self.concurrent_hashes {
            nonces.push(nonce + i);
        }

        let prev = match self.chain.len() {
            0 => String::new(),
            _ => self.chain[self.chain.len() - 1].hash.clone(),
        };

        nonces
            .par_iter()
            .find_map_any(move |&nonce| {
                let mut block = Block::new(
                    id,
                    prev.clone(),
                    txs.clone(),
                    nonce,
                    time.elapsed().unwrap().as_millis(),
                );

                let hash = block.generate_hash();
                if hash.starts_with(&target) {
                    block.hash = hash;
                    return Some(block);
                }

                None
            })
    }

    pub fn add(&mut self, block: Block) {
        let prev_s = if !block.prev.is_empty() {
            format!(" (prev {})", &block.prev[..8])
        } else {
            "".to_string()
        };
        let time_s = if SETTINGS.get::<bool>("debug_perf").unwrap() {
            format!(" -> {:.3}s", block.ms as f64 / 1000.0)
        } else {
            "".to_string()
        };
        println!(
            "new block from {} -- {} tx(s) @ {}:<{}>{}{}",
            block.id,
            block.len(),
            &block.hash[..8],
            block.nonce,
            prev_s,
            time_s,
        );

        self.chain.push(block);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Block {
    pub id: SocketAddr,
    pub nonce: u64,
    pub hash: String,
    pub prev: String,
    pub txs: Vec<Tx>,
    pub ms: u128,
}

impl Block {
    pub fn new(id: SocketAddr, prev: String, txs: Vec<Tx>, nonce: u64, ms: u128) -> Self {
        Block {
            id,
            nonce,
            hash: String::new(),
            prev,
            txs,
            ms,
        }
    }

    pub fn generate_hash(&self) -> String {
        let mut block = self.clone();
        block.hash = "".to_string();

        if let Ok(serialized) = serde_json::to_string(&block) {
            let mut hasher = Sha256::default();
            hasher.input(serialized);

            let mut hash = String::new();
            for h in hasher.result() {
                hash.push_str(&format!("{:x?}", h));
            }

            return hash;
        }

        String::new()
    }

    pub fn get_all(&self) -> &Vec<Tx> {
        &self.txs
    }

    pub fn len(&self) -> usize {
        self.txs.len()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use crate::blockchain::*;
    use crate::mempool::Mempool;
    use crate::config::SETTINGS;

    #[test]
    fn test_blockchain_mine() {
        let mut mp = Mempool::new();
        let whoami = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        let min_tx_per_block = SETTINGS.get::<usize>("min_tx_per_block").unwrap();
        let difficulty = SETTINGS.get::<usize>("difficulty").unwrap();
        let concurrent_hashes = SETTINGS.get::<u64>("concurrent_hashes").unwrap();

        for i in 0..min_tx_per_block {
            mp.add(Tx {
                from: 'A',
                to: std::char::from_digit(i as u32, 10).unwrap(),
                amount: 1,
                fee: i as f32 * 0.1
            });
        }

        let bc = Blockchain::new(min_tx_per_block, difficulty, concurrent_hashes);
        let mut nonce: u64 = 0;

        loop {
            nonce += concurrent_hashes;
            if let Some(block) = bc.mine(whoami, nonce, SystemTime::now(), mp.get_all().to_vec()) {
                assert_eq!(block.hash[..difficulty], "0".repeat(difficulty));
                assert_eq!(block.len(), min_tx_per_block);

                break;
            }
        }
    }

    #[test]
    fn test_block_generate_hash() {
        let whoami = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
        let block = Block::new(whoami, "".to_string(), vec![], 0, 0);
        assert_eq!(block.generate_hash()[..6], "156320".to_string());
    }
}

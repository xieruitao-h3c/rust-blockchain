use std::time::SystemTime;
use std::thread;
use std::sync::{Mutex, Arc, mpsc};
use crate::mempool::Mempool;
use crate::blockchain::*;
use crate::utils::*;
use crate::types::*;

pub fn start(
    rx: mpsc::Receiver<u16>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        println!("> miner started mining...");
        let local_port: u16 = rx.recv().unwrap();

        let mut time = SystemTime::now();
        let mut nonce: u64 = 0;
        let mut txs;

        let concurrent_hashes = {
            let bc = blockchain.lock().unwrap();
            bc.get_concurrent_hashes()
        };

        loop {
            // fetch transactions from mempool
            txs = {
                let mut mp = mempool.lock().unwrap();
                mp.get_all().to_vec()
            };

            let output = {
                let bc = blockchain.lock().unwrap();
                if bc.ok_to_mine(&txs) {
                    // reset timer if starting over
                    if nonce == 0 {
                        time = SystemTime::now();
                    }

                    // attempt to mine a block
                    let ret = bc.mine(local_port, nonce, time, txs);

                    // bump nonce
                    nonce = if nonce >= u64::max_value() - concurrent_hashes {
                        1
                    } else {
                        nonce + concurrent_hashes
                    };

                    ret
                } else {
                    // another node might've mined faster; reset timer
                    nonce = 0;

                    None
                }
            };

            if let Some(block) = output {
                // remove mined tx from mempool
                {
                    let mut mp = mempool.lock().unwrap();
                    for tx in block.get_all() {
                        mp.remove(&tx);
                    }
                }

                // broadcast the new block
                broadcast::<Block>(
                    ActionType::Broadcast(ObjectType::Block),
                    &block,
                    &[],
                    local_port,
                ).unwrap();

                // add new block to chain
                {
                    let mut bc = blockchain.lock().unwrap();
                    bc.add(block);
                }

                // reset on successful block (easier to debug)
                nonce = 0;
            }
        }
    })
}

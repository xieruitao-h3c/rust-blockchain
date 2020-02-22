use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::io;
use std::sync::{Mutex, Arc, mpsc};

use crate::types::*;
use crate::utils::*;
use crate::mempool::Mempool;
use crate::blockchain::*;

fn handler(
    stream: TcpStream,
    local_port: u16,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
) -> io::Result<()> {
    let mut rdr = io::BufReader::new(stream);
    let mut text = String::new();
    rdr.read_line(&mut text)?;
    if text.trim().is_empty() {
        return Ok(())
    }

    // received a request to fast-sync either blocks or txs
    if let Ok(command) = serde_json::from_str::<Command<SyncRequest>>(&text) {
        let payload = &command.payload;

        match &command.action {
            ActionType::SyncRequest(ObjectType::Block) => {
                let blocks = {
                    let bc = blockchain.lock().unwrap();
                    bc.get_all().to_vec()
                };

                let _ = broadcast::<SyncResponse<Block>>(
                    ActionType::SyncResponse(ObjectType::Block),
                    &SyncResponse::<Block> { data: blocks },
                    &[payload.port],
                    0,
                );
            },
            ActionType::SyncRequest(ObjectType::Tx) => {
                let txs = {
                    let mut mp = mempool.lock().unwrap();
                    mp.get_all().to_vec()
                };

                let _ = broadcast::<SyncResponse<Tx>>(
                    ActionType::SyncResponse(ObjectType::Tx),
                    &SyncResponse::<Tx> { data: txs },
                    &[payload.port],
                    0,
                );
            },
            _ => (),
        }

        return Ok(());
    }

    // received a blocks response from requested fast-sync
    if let Ok(command) = serde_json::from_str::<Command<SyncResponse<Block>>>(&text) {
        let payload = &command.payload;
        let mut bc = blockchain.lock().unwrap();

        for block in &payload.data {
            bc.add(block.clone());
        }

        return Ok(());
    }

    // received a txs response from a requested fast-sync
    if let Ok(command) = serde_json::from_str::<Command<SyncResponse<Tx>>>(&text) {
        let payload = &command.payload;
        let mut mp = mempool.lock().unwrap();

        for tx in &payload.data {
            let _ = mp.add(tx.clone());
        }

        return Ok(());
    }

    // received a block
    if let Ok(command) = serde_json::from_str::<Command<Block>>(&text) {
        let block = &command.payload;

        // remove mined tx from mempool
        {
            let mut mp = mempool.lock().unwrap();
            for tx in block.get_all() {
                mp.remove(&tx);
            }
        }

        // add the new block
        {
            let mut bc = blockchain.lock().unwrap();
            bc.add(block.clone());
        }

        return Ok(());
    }

    // received a transaction
    if let Ok(command) = serde_json::from_str::<Command<Tx>>(&text) {
        let tx = &command.payload;

        let (added, mp_count) = {
            let mut mp = mempool.lock().unwrap();
            (mp.add(tx.clone()), mp.len())
        };

        if added {
            println!("added {:?} to mempool ({} total)", tx, mp_count);
            let _ = broadcast::<Tx>(
                ActionType::Broadcast(ObjectType::Tx),
                tx,
                &[],
                local_port,
            );
        }

        return Ok(());
    }

    Ok(())
}

#[warn(unreachable_code)]
pub fn start(
    tx: mpsc::Sender<u16>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let addrs = get_all_peers(&[], 0);
        let listener = TcpListener::bind(&addrs[..]).expect("could not bind");

        if let Ok(addr) = listener.local_addr() {
            println!("> listening on {}...", addr);
            let local_port = addr.port();

            // let the miner know what port we're listening on
            tx.send(local_port).unwrap();

            // send a fast-sync request for any missed blocks & txs
            let peers = get_live_peers(&[], local_port);
            if !peers.is_empty() {
                for action in &[
                    ActionType::SyncRequest(ObjectType::Tx),
                    ActionType::SyncRequest(ObjectType::Block),
                ] {
                    let _ = broadcast::<SyncRequest>(
                        action.clone(),
                        &SyncRequest { port: local_port },
                        &[peers[0].port()],
                        0,
                    );
                }
            }

            // start handling requests
            loop {
                if let Ok((stream, _)) = listener.accept() {
                    if let Err(e) = handler(
                        stream,
                        local_port,
                        &blockchain,
                        &mempool,
                    ) {
                        println!("handler failed, {}", e);
                    }
                }
            }
        }
    })
}

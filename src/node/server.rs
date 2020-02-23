use std::thread;
use std::net::{TcpListener, TcpStream, SocketAddr, Ipv4Addr};
use std::io::{prelude::*, BufReader};
use std::io;
use std::sync::{Mutex, Arc, mpsc};
use crate::types::*;
use crate::utils::*;
use crate::mempool::Mempool;
use crate::blockchain::*;
use crate::config::*;

fn handler(
    stream: TcpStream,
    whoami: SocketAddr,
    blockchain: &Arc<Mutex<Blockchain>>,
    mempool: &Arc<Mutex<Mempool>>,
    peers: &[SocketAddr],
) -> io::Result<()> {
    let mut rdr = BufReader::new(stream);
    let mut text = String::new();
    rdr.read_line(&mut text)?;
    if text.trim().is_empty() {
        return Ok(())
    }

    let debug_broadcast = SETTINGS.get::<bool>("debug_broadcast").unwrap();
    if debug_broadcast {
        println!("received: {}", text.trim());
    }

    // received a request to sync either blocks or txs
    if let Ok(command) = serde_json::from_str::<Command<SyncRequest>>(&text) {
        let payload = &command.payload;
        let fwd_peers = &[payload.peer];

        match &command.action {
            ActionType::SyncRequest(ObjectType::Block) => {
                let blocks = {
                    let bc = blockchain.lock().unwrap();
                    bc.get_all().to_vec()
                };

                let _ = broadcast::<SyncResponse<Block>>(
                    ActionType::SyncResponse(ObjectType::Block),
                    &SyncResponse::<Block> { data: blocks },
                    if peers.is_empty() { fwd_peers } else { peers },
                    None,
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
                    if peers.is_empty() { fwd_peers } else { peers },
                    None,
                );
            },
            _ => (),
        }

        return Ok(());
    }

    // received a blocks response from requested sync
    if let Ok(command) = serde_json::from_str::<Command<SyncResponse<Block>>>(&text) {
        let payload = &command.payload;
        let mut bc = blockchain.lock().unwrap();

        for block in &payload.data {
            bc.add(block.clone());
        }

        return Ok(());
    }

    // received a txs response from a requested sync
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
                peers,
                Some(whoami),
            );
        }

        return Ok(());
    }

    Ok(())
}

#[warn(unreachable_code)]
pub fn start(
    tx: mpsc::Sender<SocketAddr>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    peers: Vec<SocketAddr>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let ipv4 = if peers.is_empty() { Ipv4Addr::new(127, 0, 0, 1) } else { Ipv4Addr::new(0, 0, 0, 0) };
        let addrs = get_all_peers(&[], None, Some(ipv4));
        let listener = TcpListener::bind(&addrs[..]).expect("could not bind");

        if let Ok(addr) = listener.local_addr() {
            println!("> listening on {}...", addr);

            // let the miner know what our own address is
            tx.send(addr).unwrap();

            // send a sync request for any missed blocks & txs
            let peers = get_live_peers(&peers, Some(addr), None);
            if !peers.is_empty() {
                for action in &[
                    ActionType::SyncRequest(ObjectType::Tx),
                    ActionType::SyncRequest(ObjectType::Block),
                ] {
                    let _ = broadcast::<SyncRequest>(
                        action.clone(),
                        &SyncRequest { peer: addr },
                        &[peers[0]],
                        None,
                    );
                }
            }

            // start handling requests
            loop {
                if let Ok((stream, _)) = listener.accept() {
                    if let Err(e) = handler(
                        stream,
                        addr,
                        &blockchain,
                        &mempool,
                        &peers,
                    ) {
                        println!("handler failed, {}", e);
                    }
                }
            }
        }
    })
}

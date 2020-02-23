use std::thread;
use std::time::Duration;
use std::net::SocketAddr;
use std::sync::mpsc::channel;
use rand::prelude::*;
use crate::types::*;
use crate::utils::broadcast;
use crate::config::*;

fn get_wallets(randomize: bool, i: usize) -> (char, char) {
    let mut alphabet: Vec<char> = "ABCDEFGHIJKLM".chars().collect();
    if randomize {
        alphabet.shuffle(&mut thread_rng());
    }

    (
        alphabet[i % alphabet.len()],
        alphabet[(i + 1) % alphabet.len()],
    )
}

pub fn generate(peers: Vec<SocketAddr>) {
    let broadcast_random = SETTINGS.get::<bool>("broadcast_random").unwrap();

    let mut txs: Vec<Tx> = vec![];
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut i: usize = 0;

        loop {
            let (from, to) = get_wallets(broadcast_random, i);
            let amount = if broadcast_random { rng.gen_range(1, 20) } else { 1 };
            let fee = if broadcast_random { rng.gen_range(0.1, 1.0) } else { 0.1 };

            let tx = Tx {
                from,
                to,
                amount,
                fee: (fee as f32 * 100.0).round() / 100.0,
            };

            if sender.send(tx).is_err() {
                println!("could not send transaction");
            }

            let secs: u64 = if broadcast_random { rng.gen_range(1, 5) } else { 3 };
            thread::sleep(Duration::from_secs(secs));

            i += 1;
        }
    });

    for tx in receiver {
        println!("{:?}", tx);
        broadcast::<Tx>(
            ActionType::Broadcast(ObjectType::Tx),
            &tx,
            &peers,
            None,
        ).expect("could not broadcast");
        txs.push(tx);
    }
}

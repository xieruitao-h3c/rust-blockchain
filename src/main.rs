#[macro_use]
extern crate lazy_static;

use std::net::SocketAddr;

mod config;
mod args;
mod types;
mod utils;
mod tx;
mod mempool;
mod node;
mod blockchain;

fn main() {
    let args = args::get();
    match args {
        args::Args { cmd_broadcast: true, .. } => {
            let peers: Vec<SocketAddr> = args.arg_peers;
            if peers.is_empty() {
                println!("broadcasting to all:");
            } else {
                let peers_joined = peers
                    .iter()
                    .map(|&p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                println!("broadcasting to {}:", peers_joined);
            }
            tx::generate(peers);
        },
        args::Args { cmd_mine: true, .. } => {
            println!("starting the node:");
            node::start(args.arg_peers);
        },
        _ => (),
    }
}

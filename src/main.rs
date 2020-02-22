#[macro_use]
extern crate lazy_static;

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
            let ports: Vec<u16> = args.arg_ports;
            if ports.is_empty() {
                println!("broadcasting to all:");
            } else {
                let ports_joined = ports
                    .iter()
                    .map(|&p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                println!("broadcasting to {}:", ports_joined);
            }
            tx::generate(ports);
        },
        args::Args { cmd_mine: true, .. } => {
            println!("starting the node:");
            node::start();
        },
        _ => (),
    }
}

use std::net::SocketAddr;
use serde::Deserialize;
use docopt::Docopt;

const USAGE: &str = "
A simple blockchain written in Rust.

Usage:
  blockchain broadcast [--peers=<peers>]
  blockchain mine [--peers=<peers>]
  blockchain (-h | --help)

Options:
  -h --help        Show this screen.
  --peers=<peers>  A comma-separated list of peers to broadcast to.
";

#[derive(Debug, Deserialize)]
pub struct Args {
    pub flag_peers: Vec<String>,
    pub arg_peers: Vec<SocketAddr>,
    pub cmd_broadcast: bool,
    pub cmd_mine: bool,
}

pub fn get() -> Args {
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if let Some(peer) = args.flag_peers.first() {
        for p in peer.split(',').collect::<Vec<_>>() {
            args.arg_peers.push(p.parse().unwrap());
        }
    }

    args
}

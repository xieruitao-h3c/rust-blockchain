use serde::Deserialize;
use docopt::Docopt;

const USAGE: &str = "
A simple blockchain written in Rust.

Usage:
  node broadcast [--peers=<ports>]
  node mine
  node (-h | --help)

Options:
  -h --help        Show this screen.
  --peers=<ports>  Optionally limit the ports to broadcast to.
";

#[derive(Debug, Deserialize)]
pub struct Args {
    pub flag_peers: Vec<String>,
    pub arg_ports: Vec<u16>,
    pub cmd_broadcast: bool,
    pub cmd_mine: bool,
}

pub fn get() -> Args {
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if let Some(port) = args.flag_peers.first() {
        let mut ports: Vec<String> = vec![];
        for p in port.split(',').collect::<Vec<_>>() {
            ports.push(String::from(p));
        }
        args.flag_peers = ports;
    }

    for p in &args.flag_peers {
        args.arg_ports.push(p.parse().unwrap());
    }

    args
}

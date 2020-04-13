use std::io::prelude::*;
use std::net::{TcpStream, SocketAddr};
use crate::types::*;
use serde_json::Result;
use std::clone::Clone;
use serde::ser::Serialize;
use crate::config::*;

pub fn get_all_peers(ports: &[u16], local_port: u16) -> Vec<SocketAddr> {
    let mut pool: Vec<SocketAddr> = vec![];

    let default_ports = &[4000 as u16, 4001, 4002, 4003, 4004];
    let range = if ports.is_empty() { default_ports } else { ports };

    for port in range.iter().filter(|&p| *p != local_port) {
        pool.push(SocketAddr::from(([127, 0, 0, 1], *port)));
    }

    pool
}

pub fn get_live_peers(ports: &[u16], local_port: u16) -> Vec<SocketAddr> {
    let mut peers: Vec<SocketAddr> = vec![];

    for addr in get_all_peers(ports, local_port) {
        if TcpStream::connect(&addr).is_ok() {
            peers.push(addr);
        }
    }

    peers
}

pub fn broadcast<T>(
    action: ActionType,
    payload: &T,
    peers: &[u16],
    local_port: u16,
) -> Result<()>
    where T: Clone + Serialize
{
    let pool = get_live_peers(peers, local_port);
    if pool.is_empty() {
        return Ok(())
    }

    let debug_broadcast = SETTINGS.get::<bool>("debug_broadcast").unwrap();
    if debug_broadcast {
        let pool_str = pool
            .iter()
            .map(|&sa| sa.port().to_string())
            .collect::<Vec<_>>()
            .join(",");

        println!("broadcasting to {}...", pool_str);
    }

    for peer in pool {
        if let Ok(mut stream) = TcpStream::connect(&peer) {
            let command = Command::<T>::new(action.clone(), (*payload).clone());
            let msg = serde_json::to_string::<Command<T>>(&command)?;
            writeln!(stream, "{}", msg).expect("could not broadcast");
        }
    }

    Ok(())
}

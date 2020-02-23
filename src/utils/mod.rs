use std::io::prelude::*;
use std::clone::Clone;
use std::net::{TcpStream, SocketAddr, IpAddr, Ipv4Addr};
use crate::types::*;
use serde_json::Result;
use serde::ser::Serialize;
use crate::config::*;

pub fn get_all_peers(
    peers: &[SocketAddr],
    mut whoami: Option<SocketAddr>,
    mut ipv4: Option<Ipv4Addr>,
) -> Vec<SocketAddr> {
    let mut pool: Vec<SocketAddr> = vec![];

    if whoami.is_none() {
        whoami = Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0));
    }

    if ipv4.is_none() {
        ipv4 = Some(Ipv4Addr::new(127, 0, 0, 1));
    }

    let ip = IpAddr::V4(ipv4.unwrap());
    let defaults = &[
        SocketAddr::new(ip, 4000),
        SocketAddr::new(ip, 4001),
        SocketAddr::new(ip, 4002),
        SocketAddr::new(ip, 4003),
        SocketAddr::new(ip, 4004),
    ];

    let range = if peers.is_empty() { defaults } else { peers };
    for peer in range.iter().filter(|&p| *p != whoami.unwrap()) {
        pool.push(*peer);
    }

    pool
}

pub fn get_live_peers(
    peers: &[SocketAddr],
    whoami: Option<SocketAddr>,
    ipv4: Option<Ipv4Addr>,
) -> Vec<SocketAddr> {
    let mut ret: Vec<SocketAddr> = vec![];

    for addr in get_all_peers(peers, whoami, ipv4) {
        if TcpStream::connect(&addr).is_ok() {
            ret.push(addr);
        }
    }

    ret
}

pub fn broadcast<T>(
    action: ActionType,
    payload: &T,
    peers: &[SocketAddr],
    whoami: Option<SocketAddr>,
) -> Result<()>
    where T: Clone + Serialize
{
    let pool = get_live_peers(peers, whoami, None);
    if pool.is_empty() {
        return Ok(())
    }

    let debug_broadcast = SETTINGS.get::<bool>("debug_broadcast").unwrap();
    if debug_broadcast {
        let pool_str = pool
            .iter()
            .map(|&sa| sa.to_string())
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

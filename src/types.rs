use std::net::SocketAddr;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ObjectType {
    Block,
    Tx,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ActionType {
    // used for syncing blocks and tx
    SyncRequest(ObjectType),
    SyncResponse(ObjectType),

    // let other nodes know of a newly mined block or tx
    Broadcast(ObjectType),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command<T> {
    pub action: ActionType,
    pub payload: T,
}

impl<T> Command<T> {
    pub fn new(action: ActionType, payload: T) -> Self {
        Command {
            action,
            payload,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncRequest {
    pub peer: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncResponse<T> {
    pub data: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tx {
    pub from: char,
    pub to: char,
    pub amount: i32,
    pub fee: f32,
}

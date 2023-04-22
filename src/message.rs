use crate::actor::ActorID;
use serde::{Deserialize, Serialize};

/// Message IDs should be unique on the node which sent them. For instance, each node can use a monotonically increasing integer as their source of message IDs.
pub type MessageID = u64;

/// A request from the Maelstrom system
#[derive(Serialize, Deserialize, Debug)]
pub struct Message<T> {
    /// Source of the request
    pub src: ActorID,
    /// Destination of the request
    pub dest: ActorID,
    /// Body, composed of JSON values
    pub body: T,
}

impl<T> Message<T> {
    pub fn new_reply_to<U>(msg: &Message<T>, body: U) -> Message<U> {
        Message {
            src: msg.dest.to_owned(),
            dest: msg.src.to_owned(),
            body,
        }
    }
}

impl<T: Serialize> Message<T> {
    pub fn serialize(self) -> String {
        serde_json::to_string(&self).expect("expected response to marshall to json")
    }
}

impl<T: for<'de> Deserialize<'de>> Message<T> {
    pub fn deserialize(buffer: &String) -> Self {
        serde_json::from_slice(buffer.as_bytes()).expect("expected valid payload")
    }
}

//! The actor trait
use std::sync::mpsc::Sender;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::Error;
use crate::message::Message;

pub type ActorID = String;

/// The Actor trait that you need to implement
pub trait Actor {
    type MessagePayload: Serialize + DeserializeOwned + Send;

    /// Initiate node with a name and a topology
    fn init(
        &mut self,
        tx: Sender<Message<Self::MessagePayload>>,
        node_id: String,
        peers: Vec<String>,
    ) -> Result<(), Error>;

    /// Receive a request. Will answer with a Vec of messages.
    fn receive(
        &mut self,
        request: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error>;
}

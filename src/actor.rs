//! The actor trait
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::Error;
use crate::message::Message;

pub type ActorID = String;

/// The Actor trait that you need to implement
pub trait Actor {
    type MessagePayload: Serialize + DeserializeOwned;

    /// Initiate node with a name and a topology
    fn init(&mut self, node_id: String, peers: Vec<String>) -> Result<(), Error>;

    /// Receive a request. Will answer with a Vec of messages.
    fn receive(
        &mut self,
        request: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error>;
}

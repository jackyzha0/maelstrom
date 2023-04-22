use maelstrom::{
    actor::Actor,
    errors::Error,
    message::{Message, MessageID},
    runtime::Runtime,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

fn main() {
    let mut runtime = Runtime::<BroadcastActor>::new();
    runtime.start();
}

#[derive(Default)]
struct BroadcastActor {
    node_id: Option<String>,
    peers: Vec<String>,
    messages: Vec<serde_json::value::Value>,
    // map from message id -> set of acks (node id)
    // TODO: remove acks when all nodes have acked
    acks: HashMap<u64, HashSet<String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Payload {
    Topology {
        #[serde(rename = "type")]
        message_type: String,
        msg_id: MessageID,
        topology: HashMap<String, Vec<String>>,
    },
    TopologyAck {
        #[serde(rename = "type")]
        message_type: String,
        in_reply_to: MessageID,
    },
    Broadcast {
        #[serde(rename = "type")]
        message_type: String,
        msg_id: MessageID,
        message: Value,
    },
    BroadcastAck {
        #[serde(rename = "type")]
        message_type: String,
        in_reply_to: MessageID,
    },
    Read {
        #[serde(rename = "type")]
        message_type: String,
        msg_id: MessageID,
    },
    ReadAck {
        #[serde(rename = "type")]
        message_type: String,
        messages: Vec<Value>,
        in_reply_to: MessageID,
    },
}

impl Actor for BroadcastActor {
    type MessagePayload = Payload;

    fn init(&mut self, node_id: String, _node_ids: Vec<String>) -> Result<(), Error> {
        eprintln!("Initialized node {}", node_id);
        self.node_id = Some(node_id);
        Ok(())
    }

    fn receive(
        &mut self,
        message: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error> {
        match &message.body {
            Payload::Topology {
                message_type,
                msg_id,
                topology,
            } if message_type == "topology" => {
                let peers = topology.get(self.node_id.as_ref().unwrap()).unwrap();
                self.peers = peers.to_owned();
                let ack = Payload::TopologyAck {
                    message_type: "topology_ok".to_owned(),
                    in_reply_to: *msg_id,
                };
                Ok(vec![Message::new_reply_to(message, ack)])
            }
            Payload::TopologyAck {
                message_type,
                in_reply_to: _,
            } if message_type == "topology_ok" => Ok(vec![]),
            Payload::Broadcast {
                message_type,
                msg_id,
                message: payload,
            } if message_type == "broadcast" => {
                // setup acks if it doesnt exist and add sender to set of acks
                let acks = self.acks.entry(*msg_id).or_default();
                acks.insert(message.src.to_owned());

                let node_id = self.node_id.as_ref().unwrap();
                if !acks.contains(node_id) {
                    // only unwrap and add if we haven't seen it before
                    self.messages.push(payload.to_owned());
                    // set ourselves as acked
                    acks.insert(node_id.clone());
                }

                // ack broadcast
                let ack = Payload::BroadcastAck {
                    message_type: "broadcast_ok".to_string(),
                    in_reply_to: *msg_id,
                };
                let response = Message::new_reply_to(message, ack);
                let mut responses: Vec<Message<Payload>> = vec![response];

                // send to everyone that hasn't acked
                responses.extend(self.peers.iter().filter(|peer| !acks.contains(*peer)).map(
                    |peer| Message {
                        src: self.node_id.as_ref().unwrap().to_owned(),
                        dest: peer.to_string(),
                        body: Payload::Broadcast {
                            message_type: "broadcast".to_string(),
                            msg_id: *msg_id,
                            message: payload.clone(),
                        },
                    },
                ));
                Ok(responses)
            }
            Payload::BroadcastAck {
                message_type,
                in_reply_to,
            } if message_type == "broadcast_ok" => {
                // setup acks if it doesnt exist and add receiver to set of acks
                let acks = self.acks.entry(*in_reply_to).or_default();
                acks.insert(message.src.to_string());
                Ok(vec![])
            }
            Payload::Read {
                message_type,
                msg_id,
            } if message_type == "read" => {
                let ack = Payload::ReadAck {
                    message_type: "read_ok".to_string(),
                    messages: self.messages.clone(),
                    in_reply_to: *msg_id,
                };
                Ok(vec![Message::new_reply_to(message, ack)])
            }
            Payload::ReadAck {
                message_type,
                messages: _,
                in_reply_to: _,
            } if message_type == "read_ok" => Ok(vec![]),
            _ => Err(Error::NotSupported),
        }
    }
}

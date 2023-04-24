use maelstrom::{
    actor::{Actor, ActorID},
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

type UniqueMessageID = (ActorID, MessageID);

#[derive(Default)]
struct BroadcastActor {
    node_id: Option<ActorID>,
    peers: Vec<ActorID>,
    messages: Vec<(UniqueMessageID, serde_json::value::Value)>,
    known: HashMap<ActorID, HashSet<UniqueMessageID>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Topology {
        msg_id: MessageID,
        topology: HashMap<ActorID, Vec<ActorID>>,
    },
    TopologyOk {
        in_reply_to: MessageID,
    },
    Broadcast {
        msg_id: MessageID,
        message: Value,
    },
    BroadcastOk {
        in_reply_to: MessageID,
    },
    Gossip {
        payload: Vec<(UniqueMessageID, Value)>,
    },
    GossipOk {
        seen: HashSet<UniqueMessageID>,
    },
    Read {
        msg_id: MessageID,
    },
    ReadOk {
        messages: Vec<Value>,
        in_reply_to: MessageID,
    },
}

impl BroadcastActor {
    #[inline(always)]
    pub fn node_id(&self) -> String {
        self.node_id.as_ref().unwrap().to_owned()
    }
}

impl Actor for BroadcastActor {
    type MessagePayload = Payload;

    fn init(&mut self, node_id: ActorID, _node_ids: Vec<ActorID>) -> Result<(), Error> {
        eprintln!("Initialized node {}", node_id);
        self.node_id = Some(node_id);
        Ok(())
    }

    fn receive(
        &mut self,
        message: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error> {
        match &message.body {
            Payload::Topology { msg_id, topology } => {
                let peers = topology.get(&self.node_id()).unwrap();
                self.peers = peers.to_owned();
                let ack = Payload::TopologyOk {
                    in_reply_to: *msg_id,
                };
                Ok(vec![Message::new_reply_to(message, ack)])
            }
            Payload::Broadcast {
                msg_id,
                message: payload,
            } => {
                let unique_id: UniqueMessageID = (message.src.to_string(), *msg_id);
                let mut messages = self.receive(&Message {
                    src: self.node_id(),
                    dest: self.node_id(),
                    body: Payload::Gossip {
                        payload: vec![(unique_id, payload.to_owned())],
                    },
                })?;
                messages.push(Message::new_reply_to(
                    message,
                    Payload::BroadcastOk {
                        in_reply_to: *msg_id,
                    },
                ));
                Ok(messages)
            }
            Payload::Gossip { payload } => {
                let our_id = self.node_id();
                let our_known = self.known.entry(our_id.to_owned()).or_default();
                for (msg_id, value) in payload {
                    if !our_known.contains(msg_id) {
                        self.messages.push((msg_id.to_owned(), value.to_owned()));
                        our_known.insert(msg_id.to_owned());
                    }
                }

                // ack
                let mut responses: Vec<Message<Self::MessagePayload>> =
                    vec![Message::new_reply_to(
                        message,
                        Payload::GossipOk { seen: our_known.to_owned() },
                    )];

                // construct gossip messages
                responses.extend(self.peers.iter().filter_map(|peer| {
                    let known = self.known.entry(peer.to_owned()).or_default();
                    let msgs_to_send: Vec<(UniqueMessageID, Value)> = self
                        .messages
                        .iter()
                        .cloned()
                        .filter(|(msg_id, _)| !known.contains(msg_id))
                        .collect();
                    if msgs_to_send.is_empty() {
                        None
                    } else {
                        Some(Message {
                            src: our_id.clone(),
                            dest: peer.to_owned(),
                            body: Payload::Gossip {
                                payload: msgs_to_send,
                            },
                        })
                    }
                }));
                Ok(responses)
            }
            Payload::GossipOk { seen } => {
                let their_known = self.known.entry(message.src.to_owned()).or_default();
                their_known.extend(seen.iter().cloned());
                Ok(vec![])
            }
            Payload::Read { msg_id } => {
                let ack = Payload::ReadOk {
                    messages: self.messages.iter().map(|m| m.1.to_owned()).collect(),
                    in_reply_to: *msg_id,
                };
                Ok(vec![Message::new_reply_to(message, ack)])
            }
            Payload::BroadcastOk { .. } | Payload::ReadOk { .. } | Payload::TopologyOk { .. } => {
                Ok(vec![])
            }
        }
    }
}

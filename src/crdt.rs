use crate::{
    actor::ActorID,
    message::{Message, MessageID},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

type UniqueMessageID = (ActorID, MessageID);

#[derive(Default)]
pub struct CrdtBase<T> {
    pub node_id: Option<ActorID>,
    pub peers: Vec<ActorID>,
    pub messages: Vec<(UniqueMessageID, T)>,
    pub known: HashMap<ActorID, HashSet<UniqueMessageID>>,
}

/// T is the individual message type
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload<T> {
    Add {
        msg_id: MessageID,
        delta: T,
    },
    AddOk {
        in_reply_to: MessageID,
    },
    Read {
        msg_id: MessageID,
    },
    ReadOk {
        in_reply_to: MessageID,
        value: Value,
    },
    StartGossip,
    Gossip {
        payload: Vec<(UniqueMessageID, T)>,
    },
    GossipOk {
        seen: HashSet<UniqueMessageID>,
    },
}

pub enum CrdtMessageResponse<T> {
    Responses(Vec<Message<Payload<T>>>),
    ReadRequest(MessageID),
}

impl<T: Send + Clone + 'static> CrdtBase<T> {
    #[inline(always)]
    pub fn node_id(&self) -> String {
        self.node_id.as_ref().unwrap().to_owned()
    }

    pub fn spawn_gossip_thread(&mut self, tx: Sender<Message<Payload<T>>>, interval: Duration) {
        let node_id = self.node_id();
        thread::spawn(move || loop {
            let node_id = node_id.clone();
            std::thread::sleep(interval);
            match tx.send(Message {
                src: node_id.clone(),
                dest: node_id.clone(),
                body: Payload::StartGossip,
            }) {
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("error while sending gossip signal: {}", e);
                    break;
                }
            }
        });
    }

    pub fn process_crdt_payload(
        &mut self,
        message: &Message<Payload<T>>,
    ) -> CrdtMessageResponse<T> {
        match &message.body {
            Payload::Add { msg_id, delta: value } => {
                let unique_id: UniqueMessageID = (message.src.to_string(), *msg_id);
                let our_known = self.known.entry(self.node_id()).or_default();
                self.messages.push((unique_id.to_owned(), value.to_owned()));
                our_known.insert(unique_id);
                CrdtMessageResponse::Responses(vec![Message::new_reply_to(
                    message,
                    Payload::AddOk { in_reply_to: *msg_id },
                )])
            }
            Payload::Read { msg_id } => CrdtMessageResponse::ReadRequest(*msg_id),
            Payload::StartGossip => {
                let node_id = self.node_id();
                let responses = self
                    .peers
                    .iter()
                    .filter_map(|peer| {
                        let known = self.known.entry(peer.to_owned()).or_default();
                        let msgs_to_send: Vec<(UniqueMessageID, T)> = self
                            .messages
                            .iter()
                            .cloned()
                            .filter(|(msg_id, _)| !known.contains(msg_id))
                            .collect();
                        if msgs_to_send.is_empty() {
                            None
                        } else {
                            Some(Message {
                                src: node_id.clone(),
                                dest: peer.to_owned(),
                                body: Payload::Gossip {
                                    payload: msgs_to_send,
                                },
                            })
                        }
                    })
                    .collect();
                CrdtMessageResponse::Responses(responses)
            }
            Payload::Gossip { payload } => {
                let our_id = self.node_id();
                let our_known = self.known.entry(our_id).or_default();
                for (msg_id, value) in payload {
                    if !our_known.contains(msg_id) {
                        self.messages.push((msg_id.to_owned(), value.to_owned()));
                        our_known.insert(msg_id.to_owned());
                    }
                }
                CrdtMessageResponse::Responses(vec![Message::new_reply_to(
                    message,
                    Payload::GossipOk {
                        seen: our_known.to_owned(),
                    },
                )])
            }
            Payload::GossipOk { seen } => {
                let their_known = self.known.entry(message.src.to_owned()).or_default();
                their_known.extend(seen.iter().cloned());
                CrdtMessageResponse::Responses(vec![])
            }
            Payload::AddOk { .. } | Payload::ReadOk { .. } => {
                CrdtMessageResponse::Responses(vec![])
            }
        }
    }
}

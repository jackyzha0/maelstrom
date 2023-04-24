use std::sync::mpsc::Sender;

use maelstrom::{
    actor::Actor,
    errors::Error,
    message::{Message, MessageID},
    runtime::Runtime,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn main() {
    let mut runtime: Runtime<IDActor> = Runtime::new();
    runtime.start();
}

#[derive(Default)]
struct IDActor {
    node_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Payload {
    Generate {
        #[serde(rename = "type")]
        message_type: String,
        msg_id: MessageID,
    },
    GenerateAck {
        #[serde(rename = "type")]
        message_type: String,
        in_reply_to: MessageID,
        id: String,
    },
}

impl Actor for IDActor {
    type MessagePayload = Payload;

    fn init(
        &mut self,
        _tx: Sender<Message<Self::MessagePayload>>,
        node_id: String,
        _peers: Vec<String>,
    ) -> Result<(), Error> {
        eprintln!("Initialized node {}", node_id);
        self.node_id = Some(node_id);
        Ok(())
    }

    fn receive(
        &mut self,
        message: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error> {
        match &message.body {
            Payload::Generate {
                msg_id,
                message_type,
            } if message_type == "generate" => {
                let uuid = Uuid::new_v4().to_string();
                let ack = Payload::GenerateAck {
                    message_type: "generate_ok".to_owned(),
                    in_reply_to: *msg_id,
                    id: uuid,
                };
                Ok(vec![Message::new_reply_to(message, ack)])
            }
            Payload::GenerateAck {
                message_type: _,
                in_reply_to: _,
                id: _,
            } => Ok(vec![]),
            _ => Err(Error::NotSupported),
        }
    }
}

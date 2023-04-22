use maelstrom::{
    actor::Actor,
    errors::Error,
    message::{Message, MessageID},
    runtime::Runtime,
};
use serde::{Deserialize, Serialize};

fn main() {
    let mut runtime: Runtime<EchoActor> = Runtime::new();
    runtime.start();
}

#[derive(Default)]
struct EchoActor {
    node_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Payload {
    Echo {
        #[serde(rename = "type")]
        message_type: String,
        msg_id: MessageID,
        echo: String,
    },
    EchoAck {
        #[serde(rename = "type")]
        message_type: String,
        in_reply_to: MessageID,
        echo: String,
    },
}

impl Actor for EchoActor {
    type MessagePayload = Payload;

    fn init(&mut self, node_id: String, _peers: Vec<String>) -> Result<(), Error> {
        eprintln!("Initialized node {}", node_id);
        self.node_id = Some(node_id);
        Ok(())
    }

    fn receive(
        &mut self,
        message: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error> {
        match &message.body {
            Payload::Echo {
                message_type,
                msg_id,
                echo
            } if message_type == "echo" => {
                let ack = Payload::EchoAck {
                    message_type: "echo_ok".to_owned(),
                    in_reply_to: *msg_id,
                    echo: echo.to_owned() 
                };
                Ok(vec![Message::new_reply_to(message, ack)])
            }
            Payload::EchoAck {
                message_type: _,
                in_reply_to: _,
                echo: _,
            } => Ok(vec![]),
            _ => Err(Error::NotSupported),
        }
    }
}

use crate::{
    actor::Actor,
    message::{Message, MessageID},
};
use serde::{Deserialize, Serialize};

pub struct Runtime<T: Actor + Default> {
    node: T,
}

impl<T: Actor + Default> Default for Runtime<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct InitMsg {
    msg_id: MessageID,
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Serialize)]
struct InitAckMsg {
    #[serde(rename = "type")]
    message_type: String,
    in_reply_to: MessageID,
}

impl<T: Actor + Default> Runtime<T> {
    pub fn new() -> Self {
        Self {
            node: Default::default(),
        }
    }

    fn init(&mut self) {
        let mut buffer = String::new();
        // read an init message
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("could not read stdin");

        // initialize node
        let init_msg: Message<InitMsg> = Message::deserialize(&buffer);
        self.node
            .init(
                init_msg.body.node_id.to_owned(),
                init_msg.body.node_ids.to_owned(),
            )
            .expect("initialization to not error");

        // ack
        let ack = InitAckMsg {
            message_type: "init_ok".to_owned(),
            in_reply_to: init_msg.body.msg_id,
        };
        println!("{}", Message::new_reply_to(&init_msg, ack).serialize());
    }

    pub fn start(&mut self) -> ! {
        let mut buffer = String::new();
        self.init();
        loop {
            std::io::stdin()
                .read_line(&mut buffer)
                .expect("could not read stdin");
            let msg: Message<T::MessagePayload> = Message::deserialize(&buffer);
            match self.node.receive(&msg) {
                Ok(responses) => {
                    for resp in responses {
                        println!("{}", resp.serialize());
                    }
                }
                Err(e) => {
                    eprintln!("errored while handling message: {:?}", e);
                    continue;
                }
            }
            buffer.clear();
        }
    }
}

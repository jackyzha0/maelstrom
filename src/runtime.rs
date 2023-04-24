use crate::{
    actor::Actor,
    message::{Message, MessageID},
};
use serde::{Deserialize, Serialize};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub struct Runtime<T: Actor + Default + Send> {
    node: T,
    rx: Receiver<Message<T::MessagePayload>>,
    pub tx: Sender<Message<T::MessagePayload>>,
}

impl<T: Actor + Default + Send + 'static> Default for Runtime<T> {
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

impl<T: Actor + Default + Send + 'static> Runtime<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Message<T::MessagePayload>>();
        Self {
            node: Default::default(),
            rx,
            tx,
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
                self.tx.clone(),
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

    pub fn start(&mut self) {
        self.init();

        let tx = self.tx.clone();
        let jh = thread::spawn(move || loop {
            for raw_line in std::io::stdin().lines() {
                let line = raw_line.unwrap();
                let msg: Message<T::MessagePayload> = Message::deserialize(&line);
                tx.send(msg).expect("sending message through tx");
            }
        });

        for msg in self.rx.iter() {
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
        }

        match jh.join() {
            Ok(_) => {},
            Err(e) => eprintln!("panicked on joining thread: {:?}", e),
        }
    }
}

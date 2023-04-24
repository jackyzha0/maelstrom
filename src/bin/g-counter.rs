use std::{sync::mpsc::Sender, time::Duration};
use maelstrom::{actor::Actor, crdt::{CrdtBase, Payload, CrdtMessageResponse}, message::Message, errors::Error, runtime::Runtime};

#[derive(Default)]
struct GCounter(CrdtBase<u64>);
impl Actor for GCounter {
    type MessagePayload = Payload<u64>;

    fn init(
        &mut self,
        tx: Sender<Message<Self::MessagePayload>>,
        node_id: String,
        peers: Vec<String>,
    ) -> Result<(), Error> {
        eprintln!("Initialized node {}", node_id);
        self.0.node_id = Some(node_id);
        self.0.peers = peers;
        self.0.spawn_gossip_thread(tx, Duration::from_millis(150));
        Ok(())
    }

    fn receive(
        &mut self,
        request: &Message<Self::MessagePayload>,
    ) -> Result<Vec<Message<Self::MessagePayload>>, Error> {
        match self.0.process_crdt_payload(request) {
            CrdtMessageResponse::Responses(responses) => Ok(responses),
            CrdtMessageResponse::ReadRequest(seq) => {
                let val: u64 = self.0.messages.iter().map(|m| m.1).sum();
                Ok(vec![Message::new_reply_to(request, Payload::ReadOk { in_reply_to: seq, value: val.into() })])
            },
        }
    }
}

fn main() {
    let mut runtime = Runtime::<GCounter>::new();
    runtime.start();
}

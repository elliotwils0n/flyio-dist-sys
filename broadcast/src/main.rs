use proto::{
    BroadcastBody, BroadcastEvent, BroadcastMessage, Message, MessageHandler, MessageId, Result,
};
use std::collections::{HashMap, HashSet};

struct BroadcastNode {
    id: u64,
    node_id: String,
    links: Vec<String>,
    seen: HashMap<String, HashSet<BroadcastMessage>>,
}

impl BroadcastNode {
    fn new(sender: std::sync::mpsc::Sender<BroadcastEvent>) -> Self {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if sender.send(BroadcastEvent::Gossip).is_err() {
                    break;
                }
            }
        });

        Self {
            id: 0,
            node_id: String::default(),
            links: Vec::default(),
            seen: HashMap::default(),
        }
    }
}

impl MessageHandler<BroadcastBody, BroadcastEvent> for BroadcastNode {
    fn identifier(&self) -> &str {
        &self.node_id
    }

    fn handle(
        &mut self,
        event: BroadcastEvent,
    ) -> impl IntoIterator<Item = Message<BroadcastBody>> {
        match event {
            BroadcastEvent::Message(message) => match message.body {
                BroadcastBody::Init {
                    msg_id,
                    node_id: init_node_id,
                    node_ids: _,
                } => {
                    self.node_id = init_node_id;
                    let response = Message::new(
                        message.dest,
                        message.src,
                        BroadcastBody::InitOk {
                            in_reply_to: msg_id,
                        },
                    );
                    vec![response]
                }
                BroadcastBody::Broadcast {
                    msg_id,
                    message: broadcast_message,
                } => {
                    self.seen
                        .entry(message.src.clone())
                        .or_default()
                        .insert(broadcast_message);
                    self.seen
                        .entry(self.node_id.clone())
                        .or_default()
                        .insert(broadcast_message);

                    let response = Message::new(
                        message.dest,
                        message.src,
                        BroadcastBody::BroadcastOk {
                            msg_id: MessageId(self.id),
                            in_reply_to: msg_id,
                        },
                    );
                    self.id += 1;
                    vec![response]
                }
                BroadcastBody::Read { msg_id } => {
                    let messages = self
                        .seen
                        .entry(self.node_id.clone())
                        .or_default()
                        .iter()
                        .cloned()
                        .collect();

                    let response = Message::new(
                        message.dest,
                        message.src,
                        BroadcastBody::ReadOk {
                            messages,
                            msg_id: MessageId(self.id),
                            in_reply_to: msg_id,
                        },
                    );
                    self.id += 1;
                    vec![response]
                }
                BroadcastBody::Topology {
                    msg_id,
                    mut topology,
                } => {
                    self.links = topology
                        .remove(&self.node_id)
                        .expect("Node id expected in network topology");

                    let response = Message::new(
                        message.dest,
                        message.src,
                        BroadcastBody::TopologyOk {
                            msg_id: MessageId(self.id),
                            in_reply_to: msg_id,
                        },
                    );
                    self.id += 1;
                    vec![response]
                }
                BroadcastBody::InitOk { .. }
                | BroadcastBody::BroadcastOk { .. }
                | BroadcastBody::ReadOk { .. }
                | BroadcastBody::TopologyOk { .. }
                | BroadcastBody::Error { .. } => vec![],
            },
            BroadcastEvent::Gossip => {
                let mut messages = Vec::new();
                let _ = self.seen.entry(self.node_id.clone()).or_default();
                for link in self.links.iter() {
                    let _ = self.seen.entry(link.clone()).or_default();
                    let link_messages = self.seen.get(link).unwrap();
                    let self_messages = self.seen.get(&self.node_id).unwrap();

                    let diff = self_messages
                        .difference(link_messages)
                        .cloned()
                        .collect::<Vec<BroadcastMessage>>();
                    for msg in diff {
                        messages.push(Message::new(
                            self.node_id.clone(),
                            link.clone(),
                            BroadcastBody::Broadcast {
                                msg_id: MessageId(self.id),
                                message: msg,
                            },
                        ));
                        self.id += 1
                    }
                }
                messages
            }
        }
    }
}

fn main() -> Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let node = BroadcastNode::new(sender.clone());
    proto::run(node, sender, receiver)
}

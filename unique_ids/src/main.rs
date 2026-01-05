use proto::{Message, MessageHandler, MessageId, Result, UniqueIdsBody, UniqueIdsEvent};

#[derive(Default)]
struct UniqueIdsNode {
    id: u64,
    node_id: String,
}

impl MessageHandler<UniqueIdsBody, UniqueIdsEvent> for UniqueIdsNode {
    fn identifier(&self) -> &str {
        &self.node_id
    }

    fn handle(
        &mut self,
        event: UniqueIdsEvent,
    ) -> impl IntoIterator<Item = Message<UniqueIdsBody>> {
        match event {
            UniqueIdsEvent::Message(message) => match message.body {
                UniqueIdsBody::Init {
                    msg_id,
                    node_id: req_node_id,
                    node_ids: _,
                } => {
                    self.node_id = req_node_id;
                    let response = Message::new(
                        message.dest,
                        message.src,
                        UniqueIdsBody::InitOk {
                            in_reply_to: msg_id,
                        },
                    );
                    Some(response)
                }
                UniqueIdsBody::Generate { msg_id } => {
                    let gen_id = MessageId(self.id);
                    let response = Message::new(
                        message.dest,
                        message.src,
                        UniqueIdsBody::GenerateOk {
                            msg_id: gen_id,
                            in_reply_to: msg_id,
                            id: format!("{}-{}", self.node_id, gen_id.0),
                        },
                    );
                    self.id += 1;
                    Some(response)
                }
                UniqueIdsBody::InitOk { .. }
                | UniqueIdsBody::GenerateOk { .. }
                | UniqueIdsBody::Error { .. } => None,
            },
        }
    }
}

fn main() -> Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    proto::run(UniqueIdsNode::default(), sender, receiver)
}

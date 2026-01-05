use proto::{EchoBody, EchoEvent, Message, MessageHandler, MessageId, Result};

#[derive(Default)]
struct EchoNode {
    id: u64,
    node_id: String,
}

impl MessageHandler<EchoBody, EchoEvent> for EchoNode {
    fn identifier(&self) -> &str {
        &self.node_id
    }

    fn handle(&mut self, event: EchoEvent) -> impl IntoIterator<Item = Message<EchoBody>> {
        match event {
            EchoEvent::Message(message) => match message.body {
                EchoBody::Init {
                    msg_id,
                    node_id: req_node_id,
                    node_ids: _,
                } => {
                    self.node_id = req_node_id;
                    let response = Message::new(
                        message.dest,
                        message.src,
                        EchoBody::InitOk {
                            in_reply_to: msg_id,
                        },
                    );
                    Some(response)
                }
                EchoBody::Echo { msg_id, echo } => {
                    let response = Message::new(
                        message.dest,
                        message.src,
                        EchoBody::EchoOk {
                            msg_id: MessageId(self.id),
                            in_reply_to: msg_id,
                            echo,
                        },
                    );
                    self.id += 1;
                    Some(response)
                }
                EchoBody::InitOk { .. } | EchoBody::EchoOk { .. } | EchoBody::Error { .. } => None,
            },
        }
    }
}

fn main() -> Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel();
    proto::run(EchoNode::default(), sender, receiver)
}

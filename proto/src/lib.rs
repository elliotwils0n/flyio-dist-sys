use std::{
    collections::{HashMap, HashSet},
    io::Write as _,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Parsing(serde_json::Error),
    Io(std::io::Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct MessageId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct BroadcastMessage(pub u64);

#[derive(Clone, Debug)]
pub enum EchoEvent {
    Message(Message<EchoBody>),
}

#[derive(Clone, Debug)]
pub enum UniqueIdsEvent {
    Message(Message<UniqueIdsBody>),
}

#[derive(Clone, Debug)]
pub enum BroadcastEvent {
    Message(Message<BroadcastBody>),
    Gossip,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Message<B> {
    pub src: String,
    pub dest: String,
    pub body: B,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoBody {
    Init {
        msg_id: MessageId,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: MessageId,
    },
    Echo {
        msg_id: MessageId,
        echo: String,
    },
    EchoOk {
        msg_id: MessageId,
        in_reply_to: MessageId,
        echo: String,
    },
    Error {
        in_reply_to: MessageId,
        code: u64,
        text: String,
    },
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UniqueIdsBody {
    Init {
        msg_id: MessageId,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: MessageId,
    },
    Generate {
        msg_id: MessageId,
    },
    GenerateOk {
        msg_id: MessageId,
        in_reply_to: MessageId,
        id: String,
    },
    Error {
        in_reply_to: MessageId,
        code: u64,
        text: String,
    },
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastBody {
    Init {
        msg_id: MessageId,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        in_reply_to: MessageId,
    },
    Broadcast {
        msg_id: MessageId,
        message: BroadcastMessage,
    },
    BroadcastOk {
        msg_id: MessageId,
        in_reply_to: MessageId,
    },
    Read {
        msg_id: MessageId,
    },
    ReadOk {
        msg_id: MessageId,
        in_reply_to: MessageId,
        messages: HashSet<BroadcastMessage>,
    },
    Topology {
        msg_id: MessageId,
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {
        msg_id: MessageId,
        in_reply_to: MessageId,
    },
    Error {
        in_reply_to: MessageId,
        code: u64,
        text: String,
    },
}

pub trait MessageHandler<B, E> {
    fn identifier(&self) -> &str;
    fn handle(&mut self, event: E) -> impl IntoIterator<Item = Message<B>>;
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Parsing(value)
    }
}

impl From<Message<EchoBody>> for EchoEvent {
    fn from(value: Message<EchoBody>) -> Self {
        EchoEvent::Message(value)
    }
}

impl From<Message<UniqueIdsBody>> for UniqueIdsEvent {
    fn from(value: Message<UniqueIdsBody>) -> Self {
        UniqueIdsEvent::Message(value)
    }
}

impl From<Message<BroadcastBody>> for BroadcastEvent {
    fn from(value: Message<BroadcastBody>) -> Self {
        BroadcastEvent::Message(value)
    }
}

impl<B> Message<B> {
    pub fn new(src: String, dest: String, body: B) -> Self {
        Self { src, dest, body }
    }
}

pub fn run<B, E>(
    mut handler: impl MessageHandler<B, E>,
    sender: std::sync::mpsc::Sender<E>,
    receiver: std::sync::mpsc::Receiver<E>,
) -> Result<()>
where
    B: serde::de::DeserializeOwned + serde::Serialize + std::fmt::Debug,
    E: From<Message<B>> + std::fmt::Debug + std::marker::Send + 'static,
{
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let _ = spawn_background_reader(sender);

    while let Ok(event) = receiver.recv() {
        let _ = writeln!(
            stderr,
            "[node: {}] Received event: {:?}",
            handler.identifier(),
            event
        );
        let replies = handler.handle(event).into_iter();
        for reply in replies {
            let _ = writeln!(
                stderr,
                "[node: {}] Sending reply message: {:?}",
                reply.src, reply
            );
            let reply_str = serde_json::to_string(&reply).map_err(Error::Parsing)?;
            writeln!(stdout, "{}", reply_str)?;
        }
        let _ = stderr.flush();
        stdout.flush()?;
    }
    let _ = stderr.flush();
    stdout.flush()?;

    Ok(())
}

fn spawn_background_reader<B, E>(sender: std::sync::mpsc::Sender<E>) -> std::thread::JoinHandle<()>
where
    B: serde::de::DeserializeOwned,
    E: From<Message<B>> + std::marker::Send + 'static,
{
    std::thread::spawn(move || {
        let stdin = std::io::stdin().lock();
        let message_iter = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<B>>();

        for message in message_iter {
            if let Ok(message) = message
                && sender.send(E::from(message)).is_err()
            {
                break;
            }
        }
    })
}

#[derive(Clone, Debug)]
pub enum Message {
    Connection(uuid::Uuid),
    Disconnection(uuid::Uuid),
    Message(uuid::Uuid, String),
}


impl Message {
    pub fn connection_id(&self) -> uuid::Uuid {
        match self {
            Message::Connection(id) => *id,
            Message::Disconnection(id) => *id,
            Message::Message(id, _) => *id,
        }
    }
}
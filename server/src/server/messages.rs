// pub enum StateChangeKind {
//     Connection,
//     Disconnection,
// }

// pub struct Message {
//     pub change_type: StateChangeKind,
//     pub user_id: u64,
// }

#[derive(Clone)]
pub enum Message {
    Connection(u64),
    Disconnection(u64),
    Message(u64, String),
}

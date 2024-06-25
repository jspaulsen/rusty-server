use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use tokio::{
    net::TcpStream, 
    sync::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
    },
};
use tokio_tungstenite::WebSocketStream;

use crate::server::Message;

use super::types::BoxedError;


pub struct Connection {
    uid: uuid::Uuid,
    stream: WebSocketStream<TcpStream>,
    recv: UnboundedReceiver<Message>,
    send: UnboundedSender<Message>,
}


type MessageSink = SplitSink<WebSocketStream<TcpStream>, tokio_tungstenite::tungstenite::Message>;
type MessageStream = SplitStream<WebSocketStream<TcpStream>>;


impl Connection {
    pub fn new(uid: uuid::Uuid, stream: WebSocketStream<TcpStream>, recv: UnboundedReceiver<Message>, send: UnboundedSender<Message>) -> Self {
        Self {
            uid,
            stream,
            recv,
            send,
        }
    }

    /// Pushes incoming messages from the websocket stream to the message queue
    // TODO: Finish this, it should return a Result
    async fn stream_recv(uid: uuid::Uuid, mut stream: MessageStream, send: UnboundedSender<Message>) -> Result<(), BoxedError> {
        while let Some(msg) = stream.next().await {
            let msg = msg.expect("Failed to get message");
            let incoming = Message::Message(
                uid,
                msg.to_string(),
            );

            println!("Received message in Connection.stream_recv: {:?}", msg);

            if msg.is_text() || msg.is_binary() {
                send
                    .send(incoming)
                    .expect("Failed to send message - stream_recv");
            }
        }

        Ok(())
    }

    /// Sends messages from the message queue to the websocket stream
    // TODO: Finish this, it should return a Result
    async fn send_process(mut sink: MessageSink, mut recv: UnboundedReceiver<Message>) -> Result<(), BoxedError> {
        while let Some(msg) = recv.recv().await {
            match msg {
                Message::Message(_, text) => {
                    println!("Sending message: {:?}", text);

                    sink.send(tokio_tungstenite::tungstenite::Message::Text(text)).await.expect("Failed to send message");
                },
                _ => continue,
            };
        }

        Ok(())
    }

    // async fn

    pub async fn run(self) -> Result<(), BoxedError> {
        let (send, recv) = self
            .stream
            .split();

        let result = tokio::try_join! {
            Self::stream_recv(self.uid, recv, self.send),
            Self::send_process(send, self.recv),
        };

        result.map(|_| ())
    }
}
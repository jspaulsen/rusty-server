use futures_util::{
    stream::{
        SplitSink, 
        SplitStream
    }, 
    SinkExt, 
    StreamExt,
};
use tokio::{
    net::TcpStream, 
    sync::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
    },
};

use tokio_tungstenite::{
    tungstenite::{
        error::Error as TungsteniteError,
        Message as TungsteniteMessage,
    },
    WebSocketStream,
};

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
    async fn stream_recv(uid: uuid::Uuid, mut stream: MessageStream, send: UnboundedSender<Message>) -> Result<(), BoxedError> {
        while let Some(msg) = stream.next().await {
            let message = match msg {
                Ok(msg) => msg,
                Err(e) => {
                    match e {
                        TungsteniteError::AlreadyClosed | TungsteniteError::ConnectionClosed | TungsteniteError::Protocol(_) => {
                            send
                                .send(Message::Disconnection(uid))
                                .expect("Failed to send disconnection message - stream_recv. This will only happen if the process has panicked");

                            return Ok(());
                        },
                        _ => {
                            send
                                .send(Message::Disconnection(uid))
                                .expect("Failed to send disconnection message - stream_recv. This will only happen if the process has panicked");

                            return Err(Box::new(e));
                        },
                    }
                }
            };

            match message { // TODO: Handle binary format
                TungsteniteMessage::Text(text) => {
                    send
                        .send(Message::Message(uid, text))
                        .expect("Failed to send message - stream_recv. This will only happen if the process has panicked");
                },
                TungsteniteMessage::Close(_) => {
                    send
                        .send(Message::Disconnection(uid))
                        .expect("Failed to send disconnection message - stream_recv. This will only happen if the process has panicked");

                    return Ok(()); // Connection closed
                },
                _ => continue,
            }
        }
        
        Ok(())
    }

    /// Sends messages from the message queue to the websocket stream
    async fn send_process(mut sink: MessageSink, mut recv: UnboundedReceiver<Message>) -> Result<(), BoxedError> {
        while let Some(msg) = recv.recv().await {
            match msg {
                Message::Message(_, text) => { // TODO: Handle binary format
                    sink.send(TungsteniteMessage::Text(text)).await.expect("Failed to send message");
                },
                _ => continue,
            };
        }

        Ok(())
    }

    pub async fn run(self) -> Result<(), BoxedError> {
        let (send, recv) = self
            .stream
            .split();

        // Publish a connection message to the process as soon as the connection is established
        self.send
            .send(Message::Connection(self.uid))
            .expect("Failed to send connection message! This will only happen if the process has panicked");
    
        let result = tokio::select! {
            r = Self::stream_recv(self.uid, recv, self.send) => r,
            r = Self::send_process(send, self.recv) => r,
        };

        result.map(|_| ())
    }
}
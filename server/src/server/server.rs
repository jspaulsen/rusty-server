use tokio::{
    net::{
        TcpListener,
        TcpStream,
    },
    runtime::Builder,
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Result},
};
use futures_util::{StreamExt, SinkExt};
use crate::queue::MessageQueue;
use crate::server::Message;


pub struct WebsocketServer {
    incoming: MessageQueue<Message>, // From clients
    outgoing: MessageQueue<Message>, // To clients

    // outgoing: MessageQueue<StateChangeMessage>,
    
    // todo: need host and port
    // todo: need to keep track of connections
}

// TODO: Need flag for stopping the server
impl WebsocketServer {
    pub fn new() -> Self { // TODO: Will change
        Self {
            incoming: MessageQueue::new(),
            outgoing: MessageQueue::new(),
        }
    }

    pub fn incoming_queue(&self) -> MessageQueue<Message> {
        self.incoming
            .clone()
    }

    pub fn outgoing_queue(&self) -> MessageQueue<Message> {
        self.outgoing
            .clone()
    }

    // TODO: pass in queue
    async fn _handle_connection(user_id: u64, connection: TcpStream, incoming_queue: MessageQueue<Message>) -> Result<()> {
        let mut stream = accept_async(connection)
            .await?;

        loop {
            let msg = stream
                .next()
            .await;

            match msg {
                Some(Ok(msg)) => {
                    let msg = msg.to_string();
                    let message = Message::Message(user_id, msg);

                    incoming_queue
                        .push(message)
                        .await;
                },
                Some(Err(e)) => {
                    return Err(e.into());
                },
                None => {
                    return Ok(());
                },
            }
        }
    }

    async fn handle_connection(connection: TcpStream) {
        let mut ws_stream = accept_async(connection)
            .await
            .expect("Failed to accept");
    
        while let Some(msg) = ws_stream.next().await {
            let msg = msg.expect("Failed to get message");
            if msg.is_text() || msg.is_binary() {
                ws_stream.send(msg).await.expect("Failed to send message");
            }
        }
        // TODO: actually handle connectio
    }
    
    async fn server_loop(&self) {
        let listener = TcpListener::bind("localhost:3000")
            .await
            .expect("Failed to bind to port 3000");
        
        // TODO: We also need to keep a list of connections by id or something
        // and push that id into the function managing the connection
        // TODO: change me to a match or something? 
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn( async move {
                Self::handle_connection(stream).await;
            });
        }
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()?;

        // This allows us to run tokio::spawn
        // in functions running within the runtime we've created
        let _guard = runtime.enter();

        // TODO: This changes with error returns
        runtime.block_on(self.server_loop());

        Ok(())
    }
}

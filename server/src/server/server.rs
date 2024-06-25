use std::sync::Arc;

use tokio::{
    net::TcpListener,
    runtime::Builder,
    sync::{
        mpsc::{
            unbounded_channel,
            UnboundedReceiver,
            UnboundedSender,
        }, 
        RwLock,
    },
};

use crate::server::{
    connection::Connection,
    Message,
};

use super::types::BoxedError;


type ThreadHandle = std::thread::JoinHandle<Result<(), BoxedError>>;
type ConnectionMap = std::collections::HashMap<uuid::Uuid, UnboundedSender<Message>>;
type SharedConnectionMap = Arc<RwLock<ConnectionMap>>;

struct InternalServer {
    bound: String,
    send: UnboundedSender<Message>,
    recv: Option<UnboundedReceiver<Message>>,

    // TODO: Flag
    // TODO: Track and manage connections; these can be stored in a HashMap<u64, TcpStream>
    // with an incrementing id for each connection

    // TODO: maybe we want to route messages to each connection instead of just having a send loop which
    // sends to all connections
    position: u64,
    connections: SharedConnectionMap,
}


// Hosts the logic for the server
pub struct WebsocketServer {
    bound: String,

    flag: bool, // TODO: this deffo changes

    handle: Option<ThreadHandle>,
    recv: Option<UnboundedReceiver<Message>>,

    send: Option<UnboundedSender<Message>>,
    // sender: UnboundedSender<Message>,
}



impl InternalServer {
    fn run_server_thread(&mut self) -> Result<(), BoxedError> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()?;

        // This allows us to run tokio::spawn
        // in functions running within the runtime we've created
        let _guard = runtime.enter();

        // Run the server loop until it completes
        runtime.block_on(self.server_loop())
    }

    /// Processes messages received from the server; this will push messages to the relevant connection.
    /// by looking up the connection id in the connection map
    async fn process_messages(mut recv: UnboundedReceiver<Message>, mapping: SharedConnectionMap) -> Result<(), BoxedError> {
        while let Some(msg) = recv.recv().await {
            println!("Received message in process_messages: {:?}", msg);
            
            let rw = mapping
                .read()
                .await;

            let connection = rw.get(&msg.connection_id());

            // TODO: We shouldn't panic here, just remove the connection from the map
            // as the connection has been closed
            if let Some(connection) = connection {
                connection
                    .send(msg)
                    .expect("Failed to send message - process_messages");
            }
        }

        Ok(())
    }

    async fn process_connections(listener: TcpListener, to_process: UnboundedSender<Message>, mapping: SharedConnectionMap) -> Result<(), BoxedError> {
        loop {
            let (stream, _) = listener.accept().await?;
            let ws_stream = tokio_tungstenite::accept_async(stream).await?;
            let uid = uuid::Uuid::new_v4();

            let (send, recv) = unbounded_channel::<Message>();
            let connection = Connection::new(uid, ws_stream, recv, to_process.clone());
            
            // Create a new connection and insert it into the connection map
            mapping
                .write()
                .await
                .insert(uid, send);
            
            tokio::spawn(async move {
                connection.run().await
            });
            
            // TODO: tokio::spawn
        }
    }

    // TODO:
    async fn server_loop(&mut self) -> Result<(), BoxedError> {
        let listener = TcpListener::bind(&self.bound)
            .await
            .expect("Failed to bind to port 3000");

        // PANIC: This should never happen
        // TODO: This should select! against a "stop" flag
        tokio::try_join! {
            Self::process_messages(
                self.recv
                    .take()
                    .expect("Failed to take receiver! This should never happen"),
                self.connections.clone()
            ),
            Self::process_connections(
                listener,
                self.send.clone(),
                self.connections.clone()
            )
        }.map(|_| ())
    }
}


impl WebsocketServer {
    pub fn new(host: &str, port: u16) -> Self {
        let bound = format!("{}:{}", host, port);

        Self {
            bound,
            handle: None,
            recv: None,
            send: None,
            flag: false,
        }
    }

    pub fn stop(&self) {
        unimplemented!("stop not implemented")
    }

    ///
    /// Panics if the server is not running
    pub fn recv_next(&mut self) -> Option<Message> {
        let recv = self
            .recv
            .as_mut()
            .expect("Server is not running!");

        let next = recv
            .try_recv();

        match next {
            Ok(msg) => Some(msg),
            Err(err) => {
                match err { // TODO: We should clean this up, but for now this is fine
                    tokio::sync::mpsc::error::TryRecvError::Empty => None,
                    tokio::sync::mpsc::error::TryRecvError::Disconnected => panic!("Websocket server thread has panicked"),
                }
            }
        }
    }

    pub fn send(&self, message: Message) {
        let send = self
            .send
            .as_ref()
            .expect("Server is not running!");

        send
            .send(message)
            .expect("Failed to send message! Server may have panicked");
    }

    // TODO: Maybe don't return self? Maybe return a handle to the thread?
    ///
    /// Panics if the server is already running
    pub fn run(mut self) -> Self {
        let (from_server, to_process) = unbounded_channel::<Message>();
        let (from_process, to_server) = unbounded_channel::<Message>();
        let bound = self.bound.clone();

        self.send = Some(from_process);
        self.recv = Some(to_process);

        // PANIC: Server is already running
        if self.handle.is_some() {
            panic!("Server is already running");
        }

        self.handle = Some(
            std::thread::spawn(move || {
                let mut server = InternalServer {
                    bound,
                    position: 0,
                    send: from_server,
                    recv: Some(to_server),
                    connections: Arc::new(
                        RwLock::new(
                            ConnectionMap::new()
                        )
                    )
                };

                server.run_server_thread()
            })
        );

        self
    }
}
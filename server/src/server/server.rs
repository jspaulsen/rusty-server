use std::sync::Arc;

use crossbeam_channel::{
    unbounded,
    Receiver,
    Sender, TryIter,
};
use futures_util::FutureExt;
use tokio::{
    net::TcpListener,
    runtime::Builder,
    sync::mpsc::{
        unbounded_channel,
        UnboundedReceiver,
        UnboundedSender,
    },

};
use tokio_util::sync::CancellationToken;

use crate::server::{
    connection::Connection,
    Message,
};

use super::types::BoxedError;


type ThreadHandle = std::thread::JoinHandle<Result<(), BoxedError>>;
type ConnectionMap = papaya::HashMap<uuid::Uuid, UnboundedSender<Message>>;
type SharedConnectionMap = Arc<ConnectionMap>;

type ToProcess = Sender<Message>;
type FromServer = Receiver<Message>;
type FromProcess = UnboundedReceiver<Message>;

type InternalToProcess = UnboundedSender<Message>;
type InternalFromServer = UnboundedReceiver<Message>;


struct InternalServer {
    bound: String,

    /// Messages from the server to be sent to the process
    to_process: ToProcess,

    /// Messages from process to be sent to the server
    from_process: Option<FromProcess>,

    /// Messages from the server to be sent to the process by intermediary step
    internal_to_process: InternalToProcess,

    /// Messages from the process to be sent to the server by intermediary step
    internal_from_server: Option<InternalFromServer>,

    /// Token to cancel the server
    token: CancellationToken,

    // TODO: maybe we want to route messages to each connection instead of just having a send loop which
    // sends to all connections
    connections: SharedConnectionMap,
}


// Hosts the logic for the server
pub struct WebsocketServer {
    bound: String,

    token: CancellationToken,
    handle: Option<ThreadHandle>,

    recv: Option<FromServer>,
    send: Option<UnboundedSender<Message>>,
}


impl InternalServer {
    fn new(bound: String, token: CancellationToken, send: ToProcess, recv: UnboundedReceiver<Message>) -> Self {
        let (internal_send, internal_recv) = unbounded_channel::<Message>();
        let connections = Arc::new(ConnectionMap::new());

        Self {
            bound,
            token,
            to_process: send,
            from_process: Some(recv),
            connections,
            internal_to_process: internal_send,
            internal_from_server: Some(internal_recv),
        }
    }
    
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

    /// Processes messages coming from the websocket connections and sends them to the process
    async fn process_incoming_messages(mut recv: UnboundedReceiver<Message>, to_process: ToProcess, mapping: SharedConnectionMap) -> Result<(), BoxedError> {
        while let Some(msg) = recv.recv().await {
            println!("Received incoming message in process_incoming_messages: {:?}", msg);
            
            // Remove the connection from the map if it's a disconnection message
            if let Message::Disconnection(uid) = msg {
                mapping
                    .pin()
                    .remove(&uid);
            }

            to_process
                .try_send(msg)
                .expect("Failed to send message - process_incoming_messages. This will only happen if the server has panicked");
        }

        Ok(())
    }

    /// Processes messages received from the process; this will push messages to the relevant connection.
    /// by looking up the connection id in the connection map
    async fn process_outgoing_messages(mut recv: UnboundedReceiver<Message>, mapping: SharedConnectionMap) {
        while let Some(msg) = recv.recv().await {
            let map = mapping.pin();
            println!("Received outgoing message in process_outgoing_messages: {:?}", msg);
            
            let connection_id = &msg.connection_id();
            let connection = map.get(connection_id);

            if let Some(connection) = connection {
                let result = connection
                    .send(msg);
                
                // If the connection is dead, remove it from the map
                if let Err(_) = result {
                    map.remove(connection_id);
                }
            }
        }
    }

    async fn process_connections(listener: TcpListener, to_process: InternalToProcess, mapping: SharedConnectionMap) -> Result<(), BoxedError> {
        loop {
            let (stream, _) = listener.accept().await?;
            let ws_stream = tokio_tungstenite::accept_async(stream).await?;
            let uid = uuid::Uuid::new_v4();

            let (send, recv) = unbounded_channel::<Message>();
            let connection = Connection::new(
                uid, 
                ws_stream, 
                recv, 
                to_process.clone(),
            );
            
            mapping
                .pin()
                .insert(uid, send);

            tokio::spawn(async move { connection.run().await });
        }
    }

    // TODO:
    async fn server_loop(&mut self) -> Result<(), BoxedError> {
        let listener = TcpListener::bind(&self.bound)
            .await
            .expect(format!("Failed to bind to {}", self.bound).as_str());

        let process_outgoing_messages = Self::process_outgoing_messages(
            self.from_process
                .take()
                .expect("Failed to take receiver! This can only happen if server_loop is called more than once, which should never happen"),
            self.connections.clone()
        ).map(|_| Ok(()));

        let process_incoming_messages = Self::process_incoming_messages(
            self.internal_from_server
                .take()
                .expect("Failed to take receiver! This can only happen if server_loop is called more than once, which should never happen"),
            self.to_process.clone(),
            self.connections.clone()
        ).map(|_| Ok(()));

        let process_connections = Self::process_connections(
            listener,
            self.internal_to_process.clone(),
            self.connections.clone()
        );

        tokio::select! {
            r = process_outgoing_messages => r,
            r = process_incoming_messages => r,
            r = process_connections => r,
            _ = self.token.cancelled() => Ok(()),
        }
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
            token: CancellationToken::new(),
        }
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.token.cancel();
    }

    /// Panics if the server is not running
    pub fn recv_next(&self) -> TryIter<Message> {
        let recv = self
            .recv
            .as_ref()
            .expect("Server is not running!");

        recv
            .try_iter()
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

    ///
    /// Panics if the server is already running
    pub fn run(mut self) -> Self {
        let (from_server, to_process) = unbounded::<Message>();
        let (from_process, to_server) = unbounded_channel::<Message>();
        let bound = self.bound.clone();
        let token = self.token.clone();

        self.send = Some(from_process);
        self.recv = Some(to_process);

        // PANIC: Server is already running
        if self.handle.is_some() {
            panic!("Server is already running");
        }

        self.handle = Some(
            std::thread::spawn(move || {
                let mut server = InternalServer::new(
                    bound,
                    token,
                    from_server,
                    to_server,
                );

                server.run_server_thread()
            })
        );

        self // TODO: Maybe don't return self? Maybe return a handle to the thread?
    }
}
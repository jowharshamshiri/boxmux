use crate::socket_handler::BoxMuxSocketHandler;
use rust_janus::{JanusServer, ServerConfig};
use std::sync::mpsc::Sender;
use uuid::Uuid;

/// Standalone socket server using RustJanus high-level APIs
pub struct BoxMuxSocketServer {
    config: ServerConfig,
    handler: BoxMuxSocketHandler,
}

impl BoxMuxSocketServer {
    pub fn new(message_sender: Sender<(Uuid, crate::thread_manager::Message)>, sender_uuid: Uuid) -> Self {
        let config = ServerConfig {
            socket_path: "/tmp/boxmux.sock".to_string(),
            max_connections: 100,
            default_timeout: 30,
            max_message_size: 10_000_000, // 10MB
            cleanup_on_start: true,
            cleanup_on_shutdown: true,
        };

        let handler = BoxMuxSocketHandler::new(message_sender, sender_uuid);

        Self { config, handler }
    }

    /// Start the socket server (blocks until stopped)
    /// This should be called from a dedicated thread or async context
    pub async fn start(&self) -> Result<(), rust_janus::JSONRPCError> {
        let mut server = JanusServer::new(self.config.clone());
        
        // Register all BoxMux command handlers
        self.handler.register_handlers(&mut server).await?;
        
        log::info!("Starting BoxMux socket server on: {}", self.config.socket_path);
        
        // Start listening - this blocks until the server is stopped
        server.start_listening().await?;
        
        log::info!("Socket server stopped");
        Ok(())
    }
}
use crate::socket_handler::BoxMuxSocketHandler;
use crate::thread_manager::Runnable;
use crate::{AppContext, FieldUpdate};
use rust_janus::{JanusServer, ServerConfig};
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

use crate::thread_manager::*;

create_runnable!(
    SocketLoop,
    |_inner: &mut RunnableImpl, _app_context: AppContext, _messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     _messages: Vec<Message>|
     -> (bool, AppContext) {
        let socket_path = "/tmp/boxmux.sock";
        
        // Create server configuration
        let config = ServerConfig {
            socket_path: socket_path.to_string(),
            max_connections: 100,
            default_timeout: 30,
            max_message_size: 10_000_000, // 10MB
            cleanup_on_start: true,
            cleanup_on_shutdown: true,
        };

        // Create message handler with reference to thread manager
        let message_sender = inner.get_message_sender().as_ref().unwrap().clone();
        let sender_uuid = inner.get_uuid();
        let handler = BoxMuxSocketHandler::new(message_sender, sender_uuid);

        log::info!("Starting BoxMux socket server on: {}", socket_path);

        // Initialize client and start listening
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create async runtime: {}", e);
                return (false, app_context);
            }
        };
        
        if let Err(e) = rt.block_on(async {
            let mut server = JanusServer::new(config);
            
            // Register command handlers
            handler.register_handlers(&mut server).await?;
            
            // Start listening for commands (this will block)
            server.start_listening().await?;
            
            // Keep server running
            while server.is_running() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            Ok::<(), rust_janus::JSONRPCError>(())
        }) {
            log::error!("Socket server error: {}", e);
            return (false, app_context);
        }

        log::info!("Socket server stopped");
        (true, app_context)
    }
);

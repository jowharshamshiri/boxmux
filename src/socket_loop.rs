use crate::socket_handler::BoxMuxSocketHandler;
use crate::thread_manager::Runnable;
use crate::{AppContext, FieldUpdate};
use rust_janus::{UnixDatagramServer, ApiSpecification, JanusClientConfig};
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
        
        // Load API specification
        let _api_spec = match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                match rt.block_on(ApiSpecification::from_file("api-spec.json")) {
                    Ok(spec) => spec,
                    Err(e) => {
                        log::error!("Failed to load API specification: {}", e);
                        return (false, app_context);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create async runtime: {}", e);
                return (false, app_context);
            }
        };
        
        // Create client configuration
        let _config = JanusClientConfig {
            max_concurrent_connections: 10,
            max_message_size: 10_000_000, // 10MB
            connection_timeout: Duration::from_secs(30),
            max_pending_commands: 100,
            max_command_handlers: 50,
            enable_resource_monitoring: true,
            max_channel_name_length: 256,
            max_command_name_length: 256,
            max_args_data_size: 5_000_000, // 5MB
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
            let mut server = UnixDatagramServer::new();
            
            // Register command handlers
            handler.register_handlers(&mut server).await?;
            
            // Start listening for commands (this will block)
            server.start_listening(socket_path).await?;
            
            // Keep server running
            while server.is_running() {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            Ok::<(), rust_janus::JanusError>(())
        }) {
            log::error!("Socket server error: {}", e);
            return (false, app_context);
        }

        log::info!("Socket server stopped");
        (true, app_context)
    }
);

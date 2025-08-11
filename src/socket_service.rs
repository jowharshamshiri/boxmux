use crate::socket_server::BoxMuxSocketServer;
use crate::thread_manager::Runnable;
use crate::{AppContext, FieldUpdate};
use std::sync::mpsc;
use uuid::Uuid;

use crate::thread_manager::*;

create_runnable!(
    SocketService,
    |_inner: &mut RunnableImpl, _app_context: AppContext, _messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     _messages: Vec<Message>|
     -> (bool, AppContext) {
        // Get thread communication channels
        let message_sender = inner.get_message_sender().as_ref().unwrap().clone();
        let sender_uuid = inner.get_uuid();
        
        // Create the socket server using RustJanus high-level APIs
        let server = BoxMuxSocketServer::new(message_sender, sender_uuid);
        
        // Create a tokio runtime for the async server
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Failed to create async runtime: {}", e);
                return (false, app_context);
            }
        };
        
        // Start the server (blocks until stopped)
        if let Err(e) = rt.block_on(server.start()) {
            log::error!("Socket server error: {}", e);
            return (false, app_context);
        }

        (true, app_context)
    }
);
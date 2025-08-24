use crate::thread_manager::Runnable;
use crate::{AppContext, FieldUpdate};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::mpsc;

use crate::thread_manager::*;

use uuid::Uuid;

create_runnable!(
    SocketLoop,
    |_inner: &mut RunnableImpl, _app_context: AppContext, _messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     _messages: Vec<Message>|
     -> (bool, AppContext) {
        let socket_path = "/tmp/boxmux.sock";
        // Remove the stale socket file if it exists
        if std::path::Path::new(socket_path).exists() {
            let _ = fs::remove_file(socket_path);
        }

        let listener = match UnixListener::bind(socket_path) {
            Ok(listener) => {
                log::info!("Listening on socket: {}", socket_path);
                listener
            }
            Err(err) => {
                log::error!("Failed to bind to socket {}: {}", socket_path, err);
                return (false, app_context);
            }
        };

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = String::new();
                    match stream.read_to_string(&mut buffer) {
                        Ok(_size) => {
                            let trimmed_message = buffer.trim();
                            log::debug!("Received message: {}", trimmed_message);

                            // Send the message to the thread manager for processing
                            if !trimmed_message.is_empty() {
                                inner.send_message(Message::ExternalMessage(
                                    trimmed_message.to_string(),
                                ));
                            }

                            // Send acknowledgment back to client
                            if let Err(err) = stream.write_all(b"Message Received.") {
                                log::error!("Error sending response: {}", err);
                            }
                        }
                        Err(err) => {
                            log::error!("Error receiving message: {}", err);
                        }
                    }
                }
                Err(err) => {
                    log::error!("Error accepting connection: {}", err);
                }
            }
        }

        (true, app_context)
    }
);

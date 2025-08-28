use crate::model::common::{run_socket_function, SocketFunction};
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
                            log::debug!("Received socket message: {}", trimmed_message);

                            // Parse JSON message as SocketFunction and execute directly
                            if !trimmed_message.is_empty() {
                                match serde_json::from_str::<SocketFunction>(trimmed_message) {
                                    Ok(socket_function) => {
                                        log::debug!(
                                            "Parsed socket function: {:?}",
                                            socket_function
                                        );

                                        // Execute socket function and send resulting messages
                                        match run_socket_function(socket_function, &app_context) {
                                            Ok((_updated_context, messages)) => {
                                                // Update app_context if it was modified
                                                // Note: app_context is typically not modified by socket functions
                                                // but we maintain the pattern for consistency

                                                // Send all resulting messages to the thread manager
                                                for message in messages {
                                                    inner.send_message(message);
                                                }

                                                // Send success acknowledgment
                                                if let Err(err) = stream.write_all(
                                                    b"Socket function executed successfully.",
                                                ) {
                                                    log::error!(
                                                        "Error sending success response: {}",
                                                        err
                                                    );
                                                }
                                            }
                                            Err(err) => {
                                                let error_msg = format!(
                                                    "Socket function execution failed: {}",
                                                    err
                                                );
                                                log::error!("{}", error_msg);

                                                // Send error response to client
                                                if let Err(write_err) =
                                                    stream.write_all(error_msg.as_bytes())
                                                {
                                                    log::error!(
                                                        "Error sending error response: {}",
                                                        write_err
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Err(parse_err) => {
                                        let error_msg =
                                            format!("Invalid socket function JSON: {}", parse_err);
                                        log::error!("{}", error_msg);

                                        // Send parse error response to client
                                        if let Err(write_err) =
                                            stream.write_all(error_msg.as_bytes())
                                        {
                                            log::error!(
                                                "Error sending parse error response: {}",
                                                write_err
                                            );
                                        }
                                    }
                                }
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

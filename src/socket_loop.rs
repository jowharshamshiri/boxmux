use crate::thread_manager::Runnable;
use crate::{AppContext, Config, FieldUpdate};
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{mpsc, Arc, Mutex};

use crate::thread_manager::*;

use uuid::Uuid;

create_runnable!(
    SocketLoop,
    |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     messages: Vec<Message>|
     -> (bool, AppContext) {
        let socket_path = "/tmp/boxmux.sock";
        // Remove the stale socket file if it exists
        if std::path::Path::new(socket_path).exists() {
            let _ = fs::remove_file(socket_path);
        }
        let listener = UnixListener::bind(socket_path).expect("Failed to bind to socket");
        log::info!("Listening on socket: {}", socket_path);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = String::new();
                    match stream.read_to_string(&mut buffer) {
                        Ok(size) => {
                            let data = &buffer[..size];
                            log::info!("Received message: {}", buffer);
                            inner.send_message(Message::ExternalMessage(buffer.trim().to_string()));
                            stream.write_all(b"Message Received.").unwrap();
                        }
                        Err(err) => {
                            log::error!("Error Receiving Message: {}", err);
                        }
                    }
                }
                Err(err) => {
                    log::error!("Error Accepting Connection: {}", err);
                }
            }
        }

        (true, app_context)
    }
);

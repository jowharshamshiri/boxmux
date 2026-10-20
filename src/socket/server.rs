use super::protocol::{Reply, SocketFunction};
use crate::runtime::event::Event;
use crossbeam_channel::Sender;
use interprocess::local_socket::{prelude::*, ListenerOptions, Stream};
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

/// Requests larger than this are rejected rather than read into memory.
const MAX_REQUEST_BYTES: u64 = 16 * 1024 * 1024;
/// How long a request may wait for the runtime to answer before the client gets an error.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// A listening control socket. Dropping it removes the socket file (on Unix).
pub struct SocketServer {
    name: String,
}

impl SocketServer {
    /// Binds `name` and starts accepting connections on a background thread. Fails when
    /// another live boxmux instance already listens there; a socket file left behind by an
    /// instance that crashed is replaced.
    pub fn start(name: &str, events: Sender<Event>) -> Result<SocketServer, String> {
        if super::client::send(
            name,
            &SocketFunction::QueryPtyStatus {
                box_id: String::new(),
            },
        )
        .is_ok()
        {
            return Err(format!(
                "another boxmux instance is already listening on '{}'; start this one with --socket <name>",
                name
            ));
        }
        let listener_name = super::local_name(name)
            .map_err(|e| format!("'{}' is not a valid socket name: {}", name, e))?;
        // Nothing answered above, so a leftover socket file is stale and may be replaced.
        let listener = ListenerOptions::new()
            .name(listener_name)
            .try_overwrite(true)
            .create_sync()
            .map_err(|e| format!("cannot listen on '{}': {}", name, e))?;
        std::thread::Builder::new()
            .name("socket".into())
            .spawn(move || {
                for connection in listener.incoming() {
                    match connection {
                        Ok(stream) => {
                            let events = events.clone();
                            let spawned = std::thread::Builder::new()
                                .name("socket-client".into())
                                .spawn(move || serve(stream, events));
                            if let Err(e) = spawned {
                                log::error!("cannot serve a control socket client: {}", e);
                            }
                        }
                        Err(e) => log::warn!("control socket accept failed: {}", e),
                    }
                }
            })
            .map_err(|e| format!("cannot start the control socket thread: {}", e))?;
        Ok(SocketServer {
            name: name.to_string(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for SocketServer {
    fn drop(&mut self) {
        // The listener thread is blocked in accept and owns the listener; remove the socket
        // file directly so nothing can connect after boxmux exits.
        if cfg!(unix) {
            let _ = std::fs::remove_file(&self.name);
        }
    }
}

fn serve(stream: Stream, events: Sender<Event>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let reply = match reader.by_ref().take(MAX_REQUEST_BYTES).read_line(&mut line) {
        Ok(0) => return,
        Ok(_) if !line.ends_with('\n') => Reply::error(format!(
            "requests must be a single line of at most {} bytes",
            MAX_REQUEST_BYTES
        )),
        Ok(_) => match serde_json::from_str::<SocketFunction>(&line) {
            Ok(function) => dispatch(function, &events),
            Err(e) => Reply::error(format!("invalid request: {}", e)),
        },
        Err(e) => Reply::error(format!("cannot read the request: {}", e)),
    };
    let mut text = serde_json::to_string(&reply).expect("replies always serialize");
    text.push('\n');
    let stream = reader.get_mut();
    if let Err(e) = stream
        .write_all(text.as_bytes())
        .and_then(|_| stream.flush())
    {
        log::warn!("cannot send a control socket reply: {}", e);
    }
}

fn dispatch(function: SocketFunction, events: &Sender<Event>) -> Reply {
    let (reply_tx, reply_rx) = crossbeam_channel::bounded(1);
    if events
        .send(Event::Socket {
            function,
            reply: reply_tx,
        })
        .is_err()
    {
        return Reply::error("boxmux is shutting down");
    }
    reply_rx
        .recv_timeout(REPLY_TIMEOUT)
        .unwrap_or_else(|_| Reply::error("boxmux did not answer in time"))
}

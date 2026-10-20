use super::protocol::{Reply, SocketFunction};
use interprocess::local_socket::{prelude::*, Stream};
use std::io::{BufRead, BufReader, Write};

/// Sends one command to the boxmux instance listening on `socket` and returns its reply.
pub fn send(socket: &str, function: &SocketFunction) -> Result<Reply, String> {
    let name = super::local_name(socket)
        .map_err(|e| format!("'{}' is not a valid socket name: {}", socket, e))?;
    let stream = Stream::connect(name).map_err(|e| {
        format!(
            "cannot reach a boxmux instance at '{}': {} (is boxmux running, and is BOXMUX_SOCKET or --socket set to its socket?)",
            socket, e
        )
    })?;
    let mut request =
        serde_json::to_string(function).map_err(|e| format!("cannot encode the command: {}", e))?;
    request.push('\n');
    let mut reader = BufReader::new(stream);
    reader
        .get_mut()
        .write_all(request.as_bytes())
        .and_then(|_| reader.get_mut().flush())
        .map_err(|e| format!("cannot send the command: {}", e))?;
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("cannot read the reply: {}", e))?;
    if line.is_empty() {
        return Err("boxmux closed the connection without replying".to_string());
    }
    serde_json::from_str(&line).map_err(|e| format!("malformed reply '{}': {}", line.trim_end(), e))
}

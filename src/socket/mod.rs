//! The control socket: a local socket (a Unix socket file, or a named pipe on Windows) through
//! which other processes, including boxmux's own CLI and the scripts it runs, control a running
//! instance.

pub mod client;
pub mod protocol;
pub mod server;

use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, ToFsName, ToNsName};

/// Converts a socket name to an `interprocess` name: a path on Unix, a pipe name on Windows.
pub(crate) fn local_name(name: &str) -> std::io::Result<Name<'_>> {
    if cfg!(windows) {
        name.to_ns_name::<GenericNamespaced>()
    } else {
        name.to_fs_name::<GenericFilePath>()
    }
}

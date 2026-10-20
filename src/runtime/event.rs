//! The events every background thread sends to the runtime loop, which is the only owner of
//! application state.

use crate::model::stream::RunId;
use crate::platform::ProcessSignal;
use crate::socket::protocol::{Reply, SocketFunction};
use crossbeam_channel::Sender;

#[derive(Debug)]
pub enum Event {
    /// A terminal event read by the input thread.
    Input(crossterm::event::Event),
    /// The input thread stopped because reading the terminal failed.
    InputFailed(String),
    /// A process signal (or the Windows console equivalent).
    Signal(ProcessSignal),
    /// A command from the control socket, answered through `reply`.
    Socket {
        function: SocketFunction,
        reply: Sender<Reply>,
    },
    /// Progress of a running script or PTY session.
    Run(RunEvent),
}

#[derive(Debug)]
pub enum RunEvent {
    /// Complete lines of output from a `Thread` run, as they arrive.
    Output { run: RunId, text: String },
    /// The screen of a PTY session changed.
    TerminalUpdated { run: RunId },
    /// The run ended. `output` holds the whole output of an `Immediate` run.
    Finished {
        run: RunId,
        output: Option<String>,
        outcome: RunOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// The process exited with this code.
    Exited(i32),
    /// The process was ended by this signal.
    Signaled(i32),
    /// The process could not be started or waited for.
    Failed(String),
}

use crate::thread_manager::Runnable;
use crate::AppContext;
use crate::FieldUpdate;
use crate::Message;
use signal_hook::consts::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};

use crate::thread_manager::*;

use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

create_runnable!(
    ResizeLoop,
    |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     messages: Vec<Message>|
     -> (bool, AppContext) {
        let mut signals =
            Signals::new([SIGWINCH, SIGINT, SIGTERM, SIGHUP, SIGQUIT, SIGTSTP, SIGCONT]).unwrap();

        for signal in signals.forever() {
            match signal {
                SIGWINCH => inner.send_message(Message::Resize),
                SIGINT | SIGTERM | SIGHUP | SIGQUIT => {
                    inner.send_message(Message::Terminate);
                    return (false, app_context);
                }
                SIGTSTP => inner.send_message(Message::Pause),
                SIGCONT => inner.send_message(Message::Start),
                _ => {}
            }
        }

        std::thread::sleep(Duration::from_millis(100));

        (true, app_context)
    }
);

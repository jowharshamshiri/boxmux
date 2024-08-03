use crate::model::app::AppContext;
use crate::{FieldUpdate, Panel, Updatable};
use bincode;
use log::{error, info};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{self, Sender};
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Exit,
    Terminate,
    Pause,
    Start,
    NextPanel(),
    PreviousPanel(),
    ScrollPanelDown(),
    ScrollPanelUp(),
    ScrollPanelLeft(),
    ScrollPanelRight(),
    Resize,
    RedrawPanel(String),
    RedrawApp,
    PanelEventRefresh(String),
    PanelOutputUpdate(String, bool, String),
    PanelScriptUpdate(String, Vec<String>),
    ReplacePanel(String, Panel),
    StopPanelRefresh(String),
    StartPanelRefresh(String),
    SwitchActiveLayout(String),
    KeyPress(String),
    ExternalMessage(String),
    AddPanel(String, Panel),
    RemovePanel(String),
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Message::Exit => "exit".hash(state),
            Message::Terminate => "terminate".hash(state),
            Message::NextPanel() => "next_panel".hash(state),
            Message::PreviousPanel() => "previous_panel".hash(state),
            Message::Resize => "resize".hash(state),
            Message::RedrawPanel(panel_id) => {
                "redraw_panel".hash(state);
                panel_id.hash(state);
            }
            Message::RedrawApp => "redraw_app".hash(state),
            Message::SwitchActiveLayout(layout_id) => {
                "switch_active_layout".hash(state);
                layout_id.hash(state);
            }
            Message::PanelEventRefresh(panel_id) => {
                "panel_event_refresh".hash(state);
                panel_id.hash(state);
            }
            Message::ScrollPanelDown() => "scroll_panel_down".hash(state),
            Message::ScrollPanelUp() => "scroll_panel_up".hash(state),
            Message::ScrollPanelLeft() => "scroll_panel_left".hash(state),
            Message::ScrollPanelRight() => "scroll_panel_right".hash(state),
            Message::PanelOutputUpdate(panel_id, success, output) => {
                "panel_output_update".hash(state);
                panel_id.hash(state);
                success.hash(state);
                output.hash(state);
            }
            Message::PanelScriptUpdate(panel_id, script) => {
                "panel_script_update".hash(state);
                panel_id.hash(state);
                script.hash(state);
            }
            Message::ReplacePanel(panel_id, panel) => {
                "replace_panel".hash(state);
                panel_id.hash(state);
                panel.hash(state);
            }
            Message::KeyPress(pressed_key) => {
                "key_press".hash(state);
                pressed_key.hash(state);
            }
            Message::Pause => "pause".hash(state),
            Message::ExternalMessage(msg) => {
                "external_message".hash(state);
                msg.hash(state);
            }
            Message::Start => "start".hash(state),
            Message::StopPanelRefresh(panel_id) => {
                "stop_panel_refresh".hash(state);
                panel_id.hash(state);
            }
            Message::StartPanelRefresh(panel_id) => {
                "start_panel_refresh".hash(state);
                panel_id.hash(state);
            }
            Message::AddPanel(panel_id, panel) => {
                "add_panel".hash(state);
                panel_id.hash(state);
                panel.hash(state);
            }
            Message::RemovePanel(panel_id) => {
                "remove_panel".hash(state);
                panel_id.hash(state);
            }
        }
    }
}

pub trait Runnable: Send + 'static {
    fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>>;
    fn receive_updates(&mut self) -> (AppContext, Vec<Message>);
    fn process(&mut self, app_context: AppContext, messages: Vec<Message>);

    fn update_app_context(&mut self, app_context: AppContext);
    fn set_uuid(&mut self, uuid: Uuid);
    fn get_uuid(&self) -> Uuid;
    fn set_app_context_sender(
        &mut self,
        app_context_sender: mpsc::Sender<(Uuid, Vec<FieldUpdate>)>,
    );
    fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>);
    fn set_app_context_receiver(
        &mut self,
        app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
    );
    fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>);
    fn get_app_context(&self) -> &AppContext;
    fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>>;
    fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>>;

    fn send_app_context_update(&self, old_app_context: AppContext);
    fn send_message(&self, msg: Message);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnableState {
    Created,
    Running,
    Paused,
    Terminated,
}

pub struct RunnableImpl {
    pub app_context: AppContext,
    uuid: Uuid,
    running_state: RunnableState,
    app_context_sender: Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>>,
    message_sender: Option<mpsc::Sender<(Uuid, Message)>>,
    app_context_receiver: Option<mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>>,
    message_receiver: Option<mpsc::Receiver<(Uuid, Message)>>,
}

impl RunnableImpl {
    pub fn new(app_context: AppContext) -> Self {
        RunnableImpl {
            app_context,
            uuid: Uuid::new_v4(),
            running_state: RunnableState::Created,
            app_context_sender: None,
            message_sender: None,
            app_context_receiver: None,
            message_receiver: None,
        }
    }

    pub fn _run(
        &mut self,
        process_fn: &mut dyn FnMut(&mut Self, AppContext, Vec<Message>) -> (bool, AppContext),
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (updated_app_context, new_messages) = self.receive_updates();
        let original_app_context = updated_app_context.clone();
        let mut should_continue = true;
        for message in new_messages.iter() {
            match message {
                Message::Exit => {
                    self.running_state = RunnableState::Terminated;
                    return Ok(false);
                }
                Message::Terminate => {
                    self.running_state = RunnableState::Terminated;
                    return Ok(false);
                }
                Message::Pause => {
                    self.running_state = RunnableState::Paused;
                }
                Message::Start => {
                    self.running_state = RunnableState::Running;
                }
                _ => {}
            }
        }

        //keep app_context in sync even if not running
        if updated_app_context != self.app_context {
            self.app_context = updated_app_context;
        }

        if self.running_state == RunnableState::Running {
            let (process_should_continue, result_app_context) =
                process_fn(self, self.app_context.clone(), new_messages);
            if result_app_context != original_app_context {
                self.app_context = result_app_context;
                self.send_app_context_update(original_app_context);
            }
            should_continue = process_should_continue;
        }

        if !should_continue {
            return Ok(false);
        }
        Ok(true)
    }
}

impl Runnable for RunnableImpl {
    fn receive_updates(&mut self) -> (AppContext, Vec<Message>) {
        let mut app_context_updates = Vec::new();
        let mut new_messages = Vec::new();

        if let Some(ref app_context_receiver) = self.app_context_receiver {
            while let Ok((_, received_field_updates)) = app_context_receiver.try_recv() {
                if !received_field_updates.is_empty() {
                    log::info!(
                        "Received app_context update: {:?} in thread {}",
                        received_field_updates,
                        self.uuid
                    );
                    app_context_updates = received_field_updates;
                }
            }
        }

        let mut updated_app_context = self.app_context.clone();
        updated_app_context.apply_updates(app_context_updates);

        if let Some(ref message_receiver) = self.message_receiver {
            while let Ok((_, message)) = message_receiver.try_recv() {
                new_messages.push(message);
            }
        }

        (updated_app_context, new_messages)
    }

    fn process(&mut self, _app_context: AppContext, _messages: Vec<Message>) {
        todo!()
    }

    fn update_app_context(&mut self, app_context: AppContext) {
        let old_app_context = self.app_context.clone();
        self.app_context = app_context;
        self.send_app_context_update(old_app_context);
    }

    fn set_uuid(&mut self, uuid: Uuid) {
        self.uuid = uuid;
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    fn set_app_context_sender(&mut self, app_context_sender: Sender<(Uuid, Vec<FieldUpdate>)>) {
        self.app_context_sender = Some(app_context_sender);
    }

    fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>) {
        self.message_sender = Some(message_sender);
    }

    fn set_app_context_receiver(
        &mut self,
        app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
    ) {
        self.app_context_receiver = Some(app_context_receiver);
    }

    fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>) {
        self.message_receiver = Some(message_receiver);
    }

    fn get_app_context(&self) -> &AppContext {
        &self.app_context
    }

    fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>> {
        &self.app_context_sender
    }

    fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
        &self.message_sender
    }

    fn send_app_context_update(&self, old_app_context: AppContext) {
        if let Some(ref app_context_sender) = self.get_app_context_sender() {
            if let Err(e) = app_context_sender.send((
                self.get_uuid(),
                self.get_app_context().generate_diff(&old_app_context),
            )) {
                error!("Failed to send update to main thread: {}", e);
            }
        }
    }

    fn send_message(&self, msg: Message) {
        if let Some(ref message_sender) = self.get_message_sender() {
            if let Err(e) = message_sender.send((self.get_uuid(), msg)) {
                error!("Failed to send message to main thread: {}", e);
            }
        }
    }

    fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        todo!()
    }
}

pub struct ThreadManager {
    threads: HashMap<Uuid, thread::JoinHandle<()>>,
    app_context_senders: HashMap<Uuid, mpsc::Sender<(Uuid, Vec<FieldUpdate>)>>,
    app_context_receivers: HashMap<Uuid, mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>>,
    message_senders: HashMap<Uuid, mpsc::Sender<(Uuid, Message)>>,
    message_receivers: HashMap<Uuid, mpsc::Receiver<(Uuid, Message)>>,
    app_context: AppContext,
}

impl ThreadManager {
    pub fn new(app_context: AppContext) -> Self {
        ThreadManager {
            threads: HashMap::new(),
            app_context_senders: HashMap::new(),
            message_senders: HashMap::new(),
            app_context_receivers: HashMap::new(),
            message_receivers: HashMap::new(),
            app_context: app_context,
        }
    }

    pub fn stop(&self) {
        self.send_message_to_all_threads((Uuid::new_v4(), Message::Exit));
    }

    pub fn pause(&self) {
        self.send_message_to_all_threads((Uuid::new_v4(), Message::Pause));
    }

    pub fn run(&mut self) {
        for message_sender in self.message_senders.values() {
            if let Err(e) = message_sender.send((Uuid::new_v4(), Message::Start)) {
                error!("Failed to send start message to thread: {}", e);
            }
        }
        let mut should_continue: bool = true;
        while should_continue {
            let mut has_updates = false;

            // Handle app_context updates
            for reciever in self.app_context_receivers.values() {
                if let Ok((uuid, app_context_updates)) = reciever.try_recv() {
                    if app_context_updates.is_empty() {
                        // log::info!("No updates received from thread {}", uuid);
                        continue;
                    } else {
                        let app_context_updates_size_in_bytes =
                            bincode::serialize(&app_context_updates)
                                .unwrap_or_default()
                                .len();
                        log::info!(
                            "Received {} updates from thread {} with total size {} bytes. Will relay to all other threads.",
                            app_context_updates.len(),
                            uuid,
                            app_context_updates_size_in_bytes
                        );
                    }

                    let original_app_context = self.app_context.clone();

                    // log::info!(
                    //     "Sending app_context update to all threads: {:?}",
                    //     app_context_updates
                    // );
                    self.app_context.apply_updates(app_context_updates);
                    self.send_app_context_update_to_all_threads((
                        uuid,
                        self.app_context.generate_diff(&original_app_context),
                    ));
                    has_updates = true;
                }
            }

            // Handle messages
            for reciever in self.message_receivers.values() {
                if let Ok((uuid, received_msg)) = reciever.try_recv() {
                    // log::info!("Received message from thread {}: {:?}", uuid, received_msg);
                    if received_msg == Message::Exit {
                        self.send_message_to_all_threads((Uuid::new_v4(), Message::Terminate));
                        should_continue = false;
                    } else {
                        self.send_message_to_all_threads((uuid, received_msg));
                        has_updates = true;
                    }
                }
            }

            // Sleep only if there were no updates to process
            if !has_updates {
                std::thread::sleep(std::time::Duration::from_millis(
                    self.app_context.config.frame_delay,
                ));
            }
        }
    }

    pub fn spawn_thread<R: Runnable + 'static>(&mut self, mut runnable: R) -> Uuid {
        let uuid = Uuid::new_v4();
        let (s_tm_t_s, s_tm_t_r) = mpsc::channel::<(Uuid, Vec<FieldUpdate>)>();
        let (s_t_tm_s, s_t_tm_r) = mpsc::channel::<(Uuid, Vec<FieldUpdate>)>();
        let (m_tm_t_s, m_tm_t_r) = mpsc::channel::<(Uuid, Message)>();
        let (m_t_tm_s, m_t_tm_r) = mpsc::channel::<(Uuid, Message)>();

        runnable.set_uuid(uuid);
        runnable.set_app_context_sender(s_t_tm_s);
        runnable.set_message_sender(m_t_tm_s);
        runnable.set_app_context_receiver(s_tm_t_r);
        runnable.set_message_receiver(m_tm_t_r);

        self.app_context_senders.insert(uuid, s_tm_t_s);
        self.message_senders.insert(uuid, m_tm_t_s);
        self.app_context_receivers.insert(uuid, s_t_tm_r);
        self.message_receivers.insert(uuid, m_t_tm_r);

        let runnable_class_name = std::any::type_name::<R>();
        let thread_name = format!("{}_{}", runnable_class_name, uuid);

        let handle = thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                let continue_running = true;
                while continue_running {
                    let result = runnable.run();
                    if let Err(e) = result {
                        error!("Runnable encountered an error: {}", e);
                    } else if let Ok(continue_running) = result {
                        if !continue_running {
                            info!("Stopping thread as directed by run method");
                            break;
                        }
                    }
                }
            })
            .unwrap();

        self.threads.insert(uuid, handle);

        log::info!("Thread spawned: {}", uuid);

        uuid
    }

    pub fn send_app_context_update_to_thread(&self, field_updates: Vec<FieldUpdate>, uuid: Uuid) {
        if let Some(sender) = self.app_context_senders.get(&uuid) {
            if let Err(e) = sender.send((uuid, field_updates)) {
                error!("Failed to send data to thread: {}", e);
            }
        }
    }

    pub fn send_app_context_update_to_all_threads(&self, field_updates: (Uuid, Vec<FieldUpdate>)) {
        for (&uuid, sender) in &self.app_context_senders {
            if uuid != field_updates.0 {
                if let Err(e) = sender.send(field_updates.clone()) {
                    error!("Failed to send update to thread: {}", e);
                }
            } else {
                log::info!("Skipping sending update to thread: {}", uuid);
            }
        }
    }

    pub fn send_message_to_thread(&self, msg: (Uuid, Message), uuid: Uuid) {
        if let Some(sender) = self.message_senders.get(&uuid) {
            if let Err(e) = sender.send(msg) {
                error!("Failed to send message to thread: {}", e);
            }
        }
    }

    pub fn send_message_to_all_threads(&self, msg: (Uuid, Message)) {
        for (&uuid, sender) in &self.message_senders {
            if uuid != msg.0 {
                if let Err(e) = sender.send(msg.clone()) {
                    error!("Failed to send message to thread: {}", e);
                }
            }
        }
    }

    pub fn join_threads(&mut self) {
        for handle in self.threads.drain() {
            if let Err(e) = handle.1.join() {
                error!("Failed to join thread: {:?}", e);
            }
        }
    }

    pub fn get_hash<T: Hash>(&self, t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    pub fn remove_thread(&mut self, uuid: Uuid) {
        if let Some(handle) = self.threads.remove(&uuid) {
            if let Err(e) = handle.join() {
                error!("Failed to join thread: {:?}", e);
            }
        }
        let msg = (Uuid::new_v4(), Message::Exit);
        self.send_message_to_thread(msg, uuid);
        self.app_context_senders.remove(&uuid);
        self.message_senders.remove(&uuid);
    }
}

#[macro_export]
macro_rules! create_runnable {
    ($name:ident, $init_body:expr, $process_body:expr) => {
        pub struct $name {
            inner: RunnableImpl,
        }

        impl $name {
            pub fn new(app_context: AppContext) -> Self {
                $name {
                    inner: RunnableImpl::new(app_context),
                }
            }
        }

        impl Runnable for $name {
            fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
                // Call the init block before the loop
                {
                    let inner = &mut self.inner;
                    let app_context = inner.app_context.clone();
                    let messages = Vec::new();
                    let init_result = $init_body(inner, app_context, messages);
                    if !init_result {
                        return Ok(false);
                    }
                }
                self.inner._run(&mut |inner, app_context, messages| {
                    $process_body(inner, app_context, messages)
                })
            }

            fn receive_updates(&mut self) -> (AppContext, Vec<Message>) {
                self.inner.receive_updates()
            }

            fn process(&mut self, app_context: AppContext, messages: Vec<Message>) {
                self.inner.process(app_context.clone(), messages.clone());
            }

            fn update_app_context(&mut self, app_context: AppContext) {
                self.inner.update_app_context(app_context)
            }

            fn set_uuid(&mut self, uuid: Uuid) {
                self.inner.set_uuid(uuid)
            }

            fn get_uuid(&self) -> Uuid {
                self.inner.get_uuid()
            }

            fn set_app_context_sender(
                &mut self,
                app_context_sender: mpsc::Sender<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_sender(app_context_sender)
            }

            fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>) {
                self.inner.set_message_sender(message_sender)
            }

            fn set_app_context_receiver(
                &mut self,
                app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_receiver(app_context_receiver)
            }

            fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>) {
                self.inner.set_message_receiver(message_receiver)
            }

            fn get_app_context(&self) -> &AppContext {
                self.inner.get_app_context()
            }

            fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>> {
                self.inner.get_app_context_sender()
            }

            fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
                self.inner.get_message_sender()
            }

            fn send_app_context_update(&self, old_app_context: AppContext) {
                self.inner.send_app_context_update(old_app_context)
            }

            fn send_message(&self, msg: Message) {
                self.inner.send_message(msg)
            }
        }
    };
}

#[macro_export]
macro_rules! create_runnable_with_dynamic_input {
    ($name:ident, $vec_fn:expr, $init_body:expr, $process_body:expr) => {
        pub struct $name {
            inner: RunnableImpl,
            vec_fn: Box<dyn Fn() -> Vec<String> + Send>,
        }

        impl $name {
            pub fn new(
                app_context: AppContext,
                vec_fn: Box<dyn Fn() -> Vec<String> + Send>,
            ) -> Self {
                $name {
                    inner: RunnableImpl::new(app_context),
                    vec_fn,
                }
            }
        }

        impl Runnable for $name {
            fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
                // Call the init block before the loop
                {
                    let inner = &mut self.inner;
                    let app_context = inner.app_context.clone();
                    let messages = Vec::new();
                    let vec = (self.vec_fn)();
                    let init_result = $init_body(inner, app_context, messages, vec);
                    if !init_result {
                        return Ok(false);
                    }
                }
                self.inner._run(&mut |inner, app_context, messages| {
                    let vec = (self.vec_fn)();
                    $process_body(inner, app_context, messages, vec)
                })
            }

            fn receive_updates(&mut self) -> (AppContext, Vec<Message>) {
                self.inner.receive_updates()
            }

            fn process(&mut self, app_context: AppContext, messages: Vec<Message>) {
                self.inner.process(app_context.clone(), messages.clone());
            }

            fn update_app_context(&mut self, app_context: AppContext) {
                self.inner.update_app_context(app_context)
            }

            fn set_uuid(&mut self, uuid: Uuid) {
                self.inner.set_uuid(uuid)
            }

            fn get_uuid(&self) -> Uuid {
                self.inner.get_uuid()
            }

            fn set_app_context_sender(
                &mut self,
                app_context_sender: mpsc::Sender<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_sender(app_context_sender)
            }

            fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>) {
                self.inner.set_message_sender(message_sender)
            }

            fn set_app_context_receiver(
                &mut self,
                app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_receiver(app_context_receiver)
            }

            fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>) {
                self.inner.set_message_receiver(message_receiver)
            }

            fn get_app_context(&self) -> &AppContext {
                self.inner.get_app_context()
            }

            fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>> {
                self.inner.get_app_context_sender()
            }

            fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
                self.inner.get_message_sender()
            }

            fn send_app_context_update(&self, old_app_context: AppContext) {
                self.inner.send_app_context_update(old_app_context)
            }

            fn send_message(&self, msg: Message) {
                self.inner.send_message(msg)
            }
        }
    };
}

// create_runnable!(
//     ExampleRunnable,z
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
//         // Initialization block
//         info!("Initializing ExampleRunnable with app_context: {:?}", app_context);
//         inner.update_app_context(app_context);
//         true // Initialization complete, continue running
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
//         // Processing block
//         info!("Processing in ExampleRunnable with app_context: {:?} and messages: {:?}", app_context, messages);

//         for message in messages {
//             match message {
//                 Message::Exit => return false, // Stop running
//                 Message::NextPanel(panel_id) => {
//                     info!("Next panel: {}", panel_id);
//                     // Handle NextPanel logic
//                 },
//                 Message::PreviousPanel(panel_id) => {
//                     info!("Previous panel: {}", panel_id);
//                     // Handle PreviousPanel logic
//                 },
//                 _ => {
//                     info!("Unhandled message: {:?}", message);
//                     // Handle other messages
//                 },
//             }
//         }

//         true // Continue running
//     }
// );

// create_runnable!(
//     TestRunnableOne,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne initialization");
//         true  // Assuming initialization is always successful
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableOne received message: {:?}", message);
//         }
//         info!("TestRunnableOne running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::RedrawApp);
//         false  // Intentionally stopping the thread for demonstration
//     }
// );

// create_runnable!(
//     TestRunnableTwo,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne initialization");
//         true  // Assuming initialization is always successful
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableOne received message: {:?}", message);
//         }
//         info!("TestRunnableOne running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::RedrawApp);
//         false  // Intentionally stopping the thread for demonstration
//     }
// );

// create_runnable!(
//     TestRunnableThree,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne initialization");
//         true  // Assuming initialization is always successful
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableOne received message: {:?}", message);
//         }
//         info!("TestRunnableOne running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::RedrawApp);
//         false  // Intentionally stopping the thread for demonstration
//     }
// );

// create_runnable!(
//     TestRunnableTwo,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableTwo initialization");
//         true  // Initialization success
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message)| {
//         info!("TestRunnableTwo received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableTwo received message: {:?}", message);
//         }
//         info!("TestRunnableTwo running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::ReplacePanel("Panel2".to_string()));
//         true  // Continue running
//     }
// );

// create_runnable!(
//     TestRunnableThree,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message)| {
//         info!("TestRunnableThree initialization");
//         true  // Initialization success
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message)| {
//         info!("TestRunnableThree received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableThree received message: {:?}", message);
//         }
//         info!("TestRunnableThree running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::PanelEventEnter("Panel3".to_string()));
//         true  // Continue running
//     }
// );

// #[cfg(test)]
// mod tests {
//     use crate::App;

//     use super::*;
//     use std::sync::mpsc::TryRecvError;

//     #[test]
//     fn test_message_delivery() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);
//         manager.send_app_context_update_to_thread(data.clone(), uuid2);
//         manager.send_app_context_update_to_thread(data.clone(), uuid3);

//         manager.send_message_to_all_threads((uuid1, Message::NextPanel("Panel1".to_string())));

//         // Run the manager's loop in a separate thread to allow message handling
//         let manager = manager;
//         // let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the messages
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that each runnable received the message
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (_, messages) = runnable.receive_updates();
//             assert!(messages.iter().any(|msg| matches!(msg, Message::NextPanel(panel_id) if panel_id == "Panel1")));
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }

//     #[test]
//     fn test_state_update_propagation() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);

//         // Run the manager's loop in a separate thread to allow app_context handling
//         let manager = Arc::new(manager);
//         let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the app_context update
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that the app_context was propagated to all runnables
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (app_context, _) = runnable.receive_updates();
//             assert_eq!(app_context, data);
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }

//     #[test]
//     fn test_concurrent_message_handling() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);
//         manager.send_app_context_update_to_thread(data.clone(), uuid2);
//         manager.send_app_context_update_to_thread(data.clone(), uuid3);

//         manager.send_message_to_all_threads((uuid1, Message::RedrawApp));
//         manager.send_message_to_all_threads((uuid2, Message::ReplacePanel("Panel2".to_string())));
//         manager.send_message_to_all_threads((uuid3, Message::PanelEventEnter("Panel3".to_string())));

//         // Run the manager's loop in a separate thread to allow message handling
//         let manager = Arc::new(manager);
//         let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the messages
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that each runnable received the messages
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (_, messages) = runnable.receive_updates();
//             assert!(messages.iter().any(|msg| matches!(msg, Message::RedrawApp)));
//             assert!(messages.iter().any(|msg| matches!(msg, Message::ReplacePanel(panel_id) if panel_id == "Panel2")));
//             assert!(messages.iter().any(|msg| matches!(msg, Message::PanelEventEnter(panel_id) if panel_id == "Panel3")));
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }

//     #[test]
//     fn test_message_delivery_once() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);
//         manager.send_app_context_update_to_thread(data.clone(), uuid2);
//         manager.send_app_context_update_to_thread(data.clone(), uuid3);

//         manager.send_message_to_all_threads((uuid1, Message::RedrawApp));

//         // Run the manager's loop in a separate thread to allow message handling
//         let manager = Arc::new(manager);
//         let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the messages
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that the message was delivered only once
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (_, messages) = runnable.receive_updates();
//             let count = messages.iter().filter(|msg| matches!(msg, Message::RedrawApp)).count();
//             assert_eq!(count, 1);
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }
// }

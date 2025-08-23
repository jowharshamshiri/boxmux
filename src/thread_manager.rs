use crate::model::app::AppContext;
use crate::{run_script, FieldUpdate, Panel, Updatable};
use bincode;
use log::{error};
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
    ScrollPanelPageUp(),
    ScrollPanelPageDown(),
    ScrollPanelPageLeft(),
    ScrollPanelPageRight(),
    CopyFocusedPanelContent(),
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
    ExecuteHotKeyChoice(String),
    MouseClick(u16, u16), // x, y coordinates
    PTYInput(String, String), // panel_id, input_text
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
            Message::ScrollPanelPageUp() => "scroll_panel_page_up".hash(state),
            Message::ScrollPanelPageDown() => "scroll_panel_page_down".hash(state),
            Message::ScrollPanelPageLeft() => "scroll_panel_page_left".hash(state),
            Message::ScrollPanelPageRight() => "scroll_panel_page_right".hash(state),
            Message::CopyFocusedPanelContent() => "copy_focused_panel_content".hash(state),
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
            Message::ExecuteHotKeyChoice(choice_id) => {
                "execute_hot_key_choice".hash(state);
                choice_id.hash(state);
            }
            Message::MouseClick(x, y) => {
                "mouse_click".hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::PTYInput(panel_id, input) => {
                "pty_input".hash(state);
                panel_id.hash(state);
                input.hash(state);
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
                    log::trace!(
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

    fn process(&mut self, app_context: AppContext, messages: Vec<Message>) {
        // Default implementation: update app context and handle basic messages
        self.update_app_context(app_context);
        
        // Process any messages that need handling
        for message in messages {
            match message {
                Message::Terminate => {
                    self.running_state = RunnableState::Terminated;
                }
                Message::Pause => {
                    self.running_state = RunnableState::Paused;
                }
                Message::Start => {
                    self.running_state = RunnableState::Running;
                }
                _ => {
                    // Other messages are handled by specific implementations
                }
            }
        }
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

#[derive(Debug)]
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
            app_context,
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
                        // log::trace!("No updates received from thread {}", uuid);
                        continue;
                    } else {
                        let app_context_updates_size_in_bytes =
                            bincode::serialize(&app_context_updates)
                                .unwrap_or_default()
                                .len();
                        log::trace!(
                            "Received {} updates from thread {} with total size {} bytes. Will relay to all other threads.",
                            app_context_updates.len(),
                            uuid,
                            app_context_updates_size_in_bytes
                        );
                    }

                    let original_app_context = self.app_context.clone();

                    // log::trace!(
                    //     "Sending app_context update to all threads: {:?}",
                    //     app_context_updates
                    // );
                    self.app_context.app.apply_updates(app_context_updates);
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
                            log::trace!("Stopping thread as directed by run method");
                            break;
                        }
                    }
                }
            })
            .unwrap();

        self.threads.insert(uuid, handle);

        log::trace!("Thread spawned: {}", uuid);

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
                log::trace!("Skipping sending update to thread: {}", uuid);
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

pub fn run_script_in_thread(
    app_context: AppContext,
    manager: &mut ThreadManager,
    panel_id: String,
    choice_id: String,
) -> Uuid {
    let vec_fn = move || vec![panel_id.clone(), choice_id.clone()];

    create_runnable_with_dynamic_input!(
        ChoiceScriptRunner,
        Box::new(vec_fn),
        |_inner: &mut RunnableImpl,
         _app_context: AppContext,
         _messages: Vec<Message>,
         _vec: Vec<String>|
         -> bool { true },
        |inner: &mut RunnableImpl,
         app_context: AppContext,
         _messages: Vec<Message>,
         vec: Vec<String>|
         -> (bool, AppContext) {
            let mut app_context_unwrapped = app_context.clone();
            let app_graph = app_context_unwrapped.app.generate_graph();
            let libs = app_context_unwrapped.app.libs.clone();
            let panel = app_context_unwrapped
                .app
                .get_panel_by_id_mut(&vec[0])
                .unwrap();
            let choice = panel
                .choices
                .as_mut()
                .unwrap()
                .iter_mut()
                .find(|c| c.id == vec[1])
                .unwrap();
            // Check if choice should use PTY
            let use_pty = crate::utils::should_use_pty_for_choice(choice);
            let pty_manager = app_context_unwrapped.pty_manager.as_ref();
            let message_sender = Some((inner.get_message_sender().as_ref().unwrap().clone(), inner.get_uuid()));
            
            match crate::utils::run_script_with_pty(
                libs, 
                choice.script.clone().unwrap().as_ref(),
                use_pty,
                pty_manager.map(|arc| arc.as_ref()),
                Some(choice.id.clone()),
                message_sender
            ) {
                Ok(output) => {
                    inner.send_message(Message::PanelOutputUpdate(choice.id.clone(), true, output))
                }
                Err(e) => inner.send_message(Message::PanelOutputUpdate(
                    choice.id.clone(),
                    false,
                    e.to_string(),
                )),
            }
            std::thread::sleep(std::time::Duration::from_millis(
                panel.calc_refresh_interval(&app_context, &app_graph),
            ));
            (false, app_context_unwrapped)
        }
    );

    let choice_refresh_loop = ChoiceScriptRunner::new(app_context.clone(), Box::new(vec_fn));
    manager.spawn_thread(choice_refresh_loop)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::app::App;
    use crate::model::common::{Config, FieldUpdate, EntityType};
    use crate::model::layout::Layout;
    use crate::model::panel::Panel;
    use serde_json::Value;
    use std::sync::mpsc;

    // Helper function to create test AppContext
    fn create_test_app_context() -> AppContext {
        let mut app = App::new();
        let layout = create_test_layout("test_layout");
        app.layouts.push(layout);
        let config = Config::default();
        AppContext::new(app, config)
    }

    // Helper function to create test Panel
    fn create_test_panel(id: &str) -> Panel {
        Panel {
            id: id.to_string(),
            position: crate::model::common::InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            ..Default::default()
        }
    }

    // Helper function to create test Layout
    fn create_test_layout(id: &str) -> Layout {
        Layout {
            id: id.to_string(),
            ..Default::default()
        }
    }

    // Helper function to create test FieldUpdate
    fn create_test_field_update(entity_type: EntityType, entity_id: &str, field_name: &str, value: Value) -> FieldUpdate {
        FieldUpdate {
            entity_type,
            entity_id: Some(entity_id.to_string()),
            field_name: field_name.to_string(),
            new_value: value,
        }
    }

    /// Tests that Message enum implements Hash correctly for different variants.
    /// This test demonstrates the message hashing feature.
    #[test]
    fn test_message_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        
        let msg1 = Message::Exit;
        let msg2 = Message::Exit;
        msg1.hash(&mut hasher1);
        msg2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
        
        let msg3 = Message::RedrawPanel("panel1".to_string());
        let msg4 = Message::RedrawPanel("panel1".to_string());
        let mut hasher3 = DefaultHasher::new();
        let mut hasher4 = DefaultHasher::new();
        msg3.hash(&mut hasher3);
        msg4.hash(&mut hasher4);
        assert_eq!(hasher3.finish(), hasher4.finish());
    }

    /// Tests that Message enum implements PartialEq correctly.
    /// This test demonstrates the message equality comparison feature.
    #[test]
    fn test_message_equality() {
        assert_eq!(Message::Exit, Message::Exit);
        assert_eq!(Message::Terminate, Message::Terminate);
        assert_eq!(Message::RedrawPanel("panel1".to_string()), Message::RedrawPanel("panel1".to_string()));
        assert_ne!(Message::RedrawPanel("panel1".to_string()), Message::RedrawPanel("panel2".to_string()));
        assert_ne!(Message::Exit, Message::Terminate);
    }

    /// Tests that Message enum correctly hashes different message types.
    /// This test demonstrates the message type differentiation feature.
    #[test]
    fn test_message_hash_different_types() {
        let msg1 = Message::KeyPress("a".to_string());
        let msg2 = Message::KeyPress("a".to_string());
        let msg3 = Message::KeyPress("b".to_string());
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();
        
        msg1.hash(&mut hasher1);
        msg2.hash(&mut hasher2);
        msg3.hash(&mut hasher3);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    /// Tests that RunnableState enum implements Clone and PartialEq.
    /// This test demonstrates the runnable state management feature.
    #[test]
    fn test_runnable_state() {
        let state1 = RunnableState::Created;
        let state2 = state1.clone();
        assert_eq!(state1, state2);
        
        let state3 = RunnableState::Running;
        assert_ne!(state1, state3);
        
        let state4 = RunnableState::Paused;
        let state5 = RunnableState::Terminated;
        assert_ne!(state4, state5);
    }

    /// Tests that RunnableImpl can be created with an AppContext.
    /// This test demonstrates the runnable implementation creation feature.
    #[test]
    fn test_runnable_impl_new() {
        let app_context = create_test_app_context();
        let runnable = RunnableImpl::new(app_context.clone());
        
        assert_eq!(runnable.get_app_context(), &app_context);
        assert_eq!(runnable.running_state, RunnableState::Created);
        assert!(runnable.app_context_sender.is_none());
        assert!(runnable.message_sender.is_none());
        assert!(runnable.app_context_receiver.is_none());
        assert!(runnable.message_receiver.is_none());
    }

    /// Tests that RunnableImpl can set and get UUID.
    /// This test demonstrates the runnable UUID management feature.
    #[test]
    fn test_runnable_impl_uuid() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);
        
        let original_uuid = runnable.get_uuid();
        let new_uuid = Uuid::new_v4();
        runnable.set_uuid(new_uuid);
        
        assert_eq!(runnable.get_uuid(), new_uuid);
        assert_ne!(runnable.get_uuid(), original_uuid);
    }

    /// Tests that RunnableImpl can set and get senders and receivers.
    /// This test demonstrates the runnable channel management feature.
    #[test]
    fn test_runnable_impl_channels() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);
        
        let (app_context_sender, app_context_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();
        
        runnable.set_app_context_sender(app_context_sender);
        runnable.set_message_sender(message_sender);
        runnable.set_app_context_receiver(app_context_receiver);
        runnable.set_message_receiver(message_receiver);
        
        assert!(runnable.get_app_context_sender().is_some());
        assert!(runnable.get_message_sender().is_some());
        assert!(runnable.app_context_receiver.is_some());
        assert!(runnable.message_receiver.is_some());
    }

    /// Tests that RunnableImpl can update app context.
    /// This test demonstrates the app context update feature.
    #[test]
    fn test_runnable_impl_update_app_context() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);
        
        let mut new_app_context = create_test_app_context();
        new_app_context.config.frame_delay = 100;
        
        runnable.update_app_context(new_app_context.clone());
        assert_eq!(runnable.get_app_context(), &new_app_context);
    }

    /// Tests that RunnableImpl can receive updates from channels.
    /// This test demonstrates the message receiving feature.
    #[test]
    fn test_runnable_impl_receive_updates() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);
        
        let (app_context_sender, app_context_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();
        
        runnable.set_app_context_receiver(app_context_receiver);
        runnable.set_message_receiver(message_receiver);
        
        // Send test data
        let field_update = create_test_field_update(EntityType::App, "test", "field", Value::String("value".to_string()));
        app_context_sender.send((Uuid::new_v4(), vec![field_update])).unwrap();
        message_sender.send((Uuid::new_v4(), Message::RedrawApp)).unwrap();
        
        let (updated_app_context, messages) = runnable.receive_updates();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], Message::RedrawApp);
    }

    /// Tests that RunnableImpl processes messages correctly.
    /// This test demonstrates the message processing feature.
    #[test]
    fn test_runnable_impl_process_messages() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context.clone());
        
        let messages = vec![
            Message::Start,
            Message::Pause,
            Message::Terminate,
        ];
        
        runnable.process(app_context.clone(), vec![Message::Start]);
        assert_eq!(runnable.running_state, RunnableState::Running);
        
        runnable.process(app_context.clone(), vec![Message::Pause]);
        assert_eq!(runnable.running_state, RunnableState::Paused);
        
        runnable.process(app_context.clone(), vec![Message::Terminate]);
        assert_eq!(runnable.running_state, RunnableState::Terminated);
    }

    /// Tests that RunnableImpl can send messages through channels.
    /// This test demonstrates the message sending feature.
    #[test]
    fn test_runnable_impl_send_message() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);
        
        let (message_sender, message_receiver) = mpsc::channel();
        runnable.set_message_sender(message_sender);
        
        runnable.send_message(Message::RedrawApp);
        
        let (uuid, received_message) = message_receiver.recv().unwrap();
        assert_eq!(received_message, Message::RedrawApp);
        assert_eq!(uuid, runnable.get_uuid());
    }

    /// Tests that ThreadManager can be created with an AppContext.
    /// This test demonstrates the thread manager creation feature.
    #[test]
    fn test_thread_manager_new() {
        let app_context = create_test_app_context();
        let manager = ThreadManager::new(app_context.clone());
        
        assert_eq!(manager.app_context, app_context);
        assert!(manager.threads.is_empty());
        assert!(manager.app_context_senders.is_empty());
        assert!(manager.message_senders.is_empty());
        assert!(manager.app_context_receivers.is_empty());
        assert!(manager.message_receivers.is_empty());
    }

    /// Tests that ThreadManager can spawn threads.
    /// This test demonstrates the thread spawning feature.
    #[test]
    fn test_thread_manager_spawn_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        
        assert!(manager.threads.contains_key(&uuid));
        assert!(manager.app_context_senders.contains_key(&uuid));
        assert!(manager.message_senders.contains_key(&uuid));
        assert!(manager.app_context_receivers.contains_key(&uuid));
        assert!(manager.message_receivers.contains_key(&uuid));
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can remove threads.
    /// This test demonstrates the thread removal feature.
    #[test]
    fn test_thread_manager_remove_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        assert!(manager.threads.contains_key(&uuid));
        
        manager.remove_thread(uuid);
        assert!(!manager.threads.contains_key(&uuid));
        assert!(!manager.app_context_senders.contains_key(&uuid));
        assert!(!manager.message_senders.contains_key(&uuid));
    }

    /// Tests that ThreadManager can send messages to specific threads.
    /// This test demonstrates the targeted message sending feature.
    #[test]
    fn test_thread_manager_send_message_to_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        let message = (Uuid::new_v4(), Message::RedrawApp);
        
        manager.send_message_to_thread(message, uuid);
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can send messages to all threads.
    /// This test demonstrates the broadcast message sending feature.
    #[test]
    fn test_thread_manager_send_message_to_all_threads() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable1 = RunnableImpl::new(app_context.clone());
        let runnable2 = RunnableImpl::new(app_context.clone());
        
        let uuid1 = manager.spawn_thread(runnable1);
        let uuid2 = manager.spawn_thread(runnable2);
        
        let message = (Uuid::new_v4(), Message::RedrawApp);
        manager.send_message_to_all_threads(message);
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can send app context updates to specific threads.
    /// This test demonstrates the targeted app context update feature.
    #[test]
    fn test_thread_manager_send_app_context_update_to_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        let field_update = create_test_field_update(EntityType::App, "test", "field", Value::String("value".to_string()));
        
        manager.send_app_context_update_to_thread(vec![field_update], uuid);
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can send app context updates to all threads.
    /// This test demonstrates the broadcast app context update feature.
    #[test]
    fn test_thread_manager_send_app_context_update_to_all_threads() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable1 = RunnableImpl::new(app_context.clone());
        let runnable2 = RunnableImpl::new(app_context.clone());
        
        let uuid1 = manager.spawn_thread(runnable1);
        let uuid2 = manager.spawn_thread(runnable2);
        
        let field_update = create_test_field_update(EntityType::App, "test", "field", Value::String("value".to_string()));
        let sender_uuid = Uuid::new_v4();
        
        manager.send_app_context_update_to_all_threads((sender_uuid, vec![field_update]));
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can stop all threads.
    /// This test demonstrates the thread stopping feature.
    #[test]
    fn test_thread_manager_stop() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        assert!(manager.threads.contains_key(&uuid));
        
        manager.stop();
        manager.join_threads();
        
        assert!(manager.threads.is_empty());
    }

    /// Tests that ThreadManager can pause all threads.
    /// This test demonstrates the thread pausing feature.
    #[test]
    fn test_thread_manager_pause() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        manager.pause();
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can calculate hash values.
    /// This test demonstrates the hash calculation feature.
    #[test]
    fn test_thread_manager_get_hash() {
        let app_context = create_test_app_context();
        let manager = ThreadManager::new(app_context);
        
        let test_string = "test";
        let hash1 = manager.get_hash(&test_string);
        let hash2 = manager.get_hash(&test_string);
        assert_eq!(hash1, hash2);
        
        let test_string2 = "different";
        let hash3 = manager.get_hash(&test_string2);
        assert_ne!(hash1, hash3);
    }

    /// Tests that ThreadManager can join all threads.
    /// This test demonstrates the thread joining feature.
    #[test]
    fn test_thread_manager_join_threads() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);
        
        let uuid = manager.spawn_thread(runnable);
        assert!(manager.threads.contains_key(&uuid));
        
        manager.stop();
        manager.join_threads();
        
        assert!(manager.threads.is_empty());
    }

    /// Tests that run_script_in_thread spawns a thread correctly.
    /// This test demonstrates the script thread spawning feature.
    #[test]
    fn test_run_script_in_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        
        let panel_id = "test_panel".to_string();
        let choice_id = "test_choice".to_string();
        
        let uuid = run_script_in_thread(app_context, &mut manager, panel_id, choice_id);
        
        assert!(manager.threads.contains_key(&uuid));
        assert!(manager.app_context_senders.contains_key(&uuid));
        assert!(manager.message_senders.contains_key(&uuid));
        
        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that Message::PanelOutputUpdate contains correct data.
    /// This test demonstrates the panel output update message feature.
    #[test]
    fn test_message_panel_output_update() {
        let panel_id = "test_panel".to_string();
        let success = true;
        let output = "test output".to_string();
        
        let message = Message::PanelOutputUpdate(panel_id.clone(), success, output.clone());
        
        match message {
            Message::PanelOutputUpdate(id, success_flag, content) => {
                assert_eq!(id, panel_id);
                assert_eq!(success_flag, success);
                assert_eq!(content, output);
            }
            _ => panic!("Expected PanelOutputUpdate message"),
        }
    }

    /// Tests that Message::PanelScriptUpdate contains correct data.
    /// This test demonstrates the panel script update message feature.
    #[test]
    fn test_message_panel_script_update() {
        let panel_id = "test_panel".to_string();
        let script = vec!["echo 'test'".to_string(), "ls".to_string()];
        
        let message = Message::PanelScriptUpdate(panel_id.clone(), script.clone());
        
        match message {
            Message::PanelScriptUpdate(id, script_content) => {
                assert_eq!(id, panel_id);
                assert_eq!(script_content, script);
            }
            _ => panic!("Expected PanelScriptUpdate message"),
        }
    }

    /// Tests that Message::ReplacePanel contains correct data.
    /// This test demonstrates the panel replacement message feature.
    #[test]
    fn test_message_replace_panel() {
        let panel_id = "test_panel".to_string();
        let panel = create_test_panel("new_panel");
        
        let message = Message::ReplacePanel(panel_id.clone(), panel.clone());
        
        match message {
            Message::ReplacePanel(id, new_panel) => {
                assert_eq!(id, panel_id);
                assert_eq!(new_panel.id, panel.id);
            }
            _ => panic!("Expected ReplacePanel message"),
        }
    }

    /// Tests that Message::SwitchActiveLayout contains correct data.
    /// This test demonstrates the layout switching message feature.
    #[test]
    fn test_message_switch_active_layout() {
        let layout_id = "test_layout".to_string();
        let message = Message::SwitchActiveLayout(layout_id.clone());
        
        match message {
            Message::SwitchActiveLayout(id) => {
                assert_eq!(id, layout_id);
            }
            _ => panic!("Expected SwitchActiveLayout message"),
        }
    }

    /// Tests that Message::KeyPress contains correct data.
    /// This test demonstrates the key press message feature.
    #[test]
    fn test_message_key_press() {
        let key = "ctrl+c".to_string();
        let message = Message::KeyPress(key.clone());
        
        match message {
            Message::KeyPress(pressed_key) => {
                assert_eq!(pressed_key, key);
            }
            _ => panic!("Expected KeyPress message"),
        }
    }

    /// Tests that Message::ExternalMessage contains correct data.
    /// This test demonstrates the external message feature.
    #[test]
    fn test_message_external_message() {
        let external_msg = "external command".to_string();
        let message = Message::ExternalMessage(external_msg.clone());
        
        match message {
            Message::ExternalMessage(msg) => {
                assert_eq!(msg, external_msg);
            }
            _ => panic!("Expected ExternalMessage message"),
        }
    }

    /// Tests that Message::AddPanel contains correct data.
    /// This test demonstrates the panel addition message feature.
    #[test]
    fn test_message_add_panel() {
        let panel_id = "test_panel".to_string();
        let panel = create_test_panel("new_panel");
        
        let message = Message::AddPanel(panel_id.clone(), panel.clone());
        
        match message {
            Message::AddPanel(id, new_panel) => {
                assert_eq!(id, panel_id);
                assert_eq!(new_panel.id, panel.id);
            }
            _ => panic!("Expected AddPanel message"),
        }
    }

    /// Tests that Message::RemovePanel contains correct data.
    /// This test demonstrates the panel removal message feature.
    #[test]
    fn test_message_remove_panel() {
        let panel_id = "test_panel".to_string();
        let message = Message::RemovePanel(panel_id.clone());
        
        match message {
            Message::RemovePanel(id) => {
                assert_eq!(id, panel_id);
            }
            _ => panic!("Expected RemovePanel message"),
        }
    }

    /// Tests that simple messages are created correctly.
    /// This test demonstrates the simple message creation feature.
    #[test]
    fn test_simple_messages() {
        let exit_msg = Message::Exit;
        let terminate_msg = Message::Terminate;
        let pause_msg = Message::Pause;
        let start_msg = Message::Start;
        let resize_msg = Message::Resize;
        let redraw_app_msg = Message::RedrawApp;
        
        // Test that they can be created and compared
        assert_eq!(exit_msg, Message::Exit);
        assert_eq!(terminate_msg, Message::Terminate);
        assert_eq!(pause_msg, Message::Pause);
        assert_eq!(start_msg, Message::Start);
        assert_eq!(resize_msg, Message::Resize);
        assert_eq!(redraw_app_msg, Message::RedrawApp);
        
        // Test that they are different from each other
        assert_ne!(exit_msg, terminate_msg);
        assert_ne!(pause_msg, start_msg);
        assert_ne!(resize_msg, redraw_app_msg);
    }

    /// Tests that scroll messages are created correctly.
    /// This test demonstrates the scroll message creation feature.
    #[test]
    fn test_scroll_messages() {
        let scroll_down = Message::ScrollPanelDown();
        let scroll_up = Message::ScrollPanelUp();
        let scroll_left = Message::ScrollPanelLeft();
        let scroll_right = Message::ScrollPanelRight();
        let scroll_page_up = Message::ScrollPanelPageUp();
        let scroll_page_down = Message::ScrollPanelPageDown();
        
        assert_eq!(scroll_down, Message::ScrollPanelDown());
        assert_eq!(scroll_up, Message::ScrollPanelUp());
        assert_eq!(scroll_left, Message::ScrollPanelLeft());
        assert_eq!(scroll_right, Message::ScrollPanelRight());
        assert_eq!(scroll_page_up, Message::ScrollPanelPageUp());
        assert_eq!(scroll_page_down, Message::ScrollPanelPageDown());
        
        assert_ne!(scroll_down, scroll_up);
        assert_ne!(scroll_left, scroll_right);
    }

    /// Tests that navigation messages are created correctly.
    /// This test demonstrates the navigation message creation feature.
    #[test]
    fn test_navigation_messages() {
        let next_panel = Message::NextPanel();
        let previous_panel = Message::PreviousPanel();
        
        assert_eq!(next_panel, Message::NextPanel());
        assert_eq!(previous_panel, Message::PreviousPanel());
        assert_ne!(next_panel, previous_panel);
    }

    /// Tests that panel refresh messages are created correctly.
    /// This test demonstrates the panel refresh message feature.
    #[test]
    fn test_panel_refresh_messages() {
        let panel_id = "test_panel".to_string();
        let start_refresh = Message::StartPanelRefresh(panel_id.clone());
        let stop_refresh = Message::StopPanelRefresh(panel_id.clone());
        let event_refresh = Message::PanelEventRefresh(panel_id.clone());
        
        match start_refresh {
            Message::StartPanelRefresh(id) => assert_eq!(id, panel_id),
            _ => panic!("Expected StartPanelRefresh"),
        }
        
        match stop_refresh {
            Message::StopPanelRefresh(id) => assert_eq!(id, panel_id),
            _ => panic!("Expected StopPanelRefresh"),
        }
        
        match event_refresh {
            Message::PanelEventRefresh(id) => assert_eq!(id, panel_id),
            _ => panic!("Expected PanelEventRefresh"),
        }
    }

    /// Tests that RedrawPanel message is created correctly.
    /// This test demonstrates the panel redraw message feature.
    #[test]
    fn test_redraw_panel_message() {
        let panel_id = "test_panel".to_string();
        let redraw_msg = Message::RedrawPanel(panel_id.clone());
        
        match redraw_msg {
            Message::RedrawPanel(id) => assert_eq!(id, panel_id),
            _ => panic!("Expected RedrawPanel message"),
        }
    }
}

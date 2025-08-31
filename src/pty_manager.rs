use anyhow::Result;
use log::{debug, error, warn};
// Use log crate for debugging
use crate::ansi_processor::AnsiProcessor;
use crate::circular_buffer::CircularBuffer;
use portable_pty::{CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct PtyProcess {
    pub muxbox_id: String,
    pub process_id: Option<u32>,
    pub status: PtyStatus,
    pub master_pty: Option<Arc<Mutex<Box<dyn MasterPty + Send>>>>,
    pub can_kill: bool, // Indicates if we can kill the process
    pub output_buffer: Arc<Mutex<CircularBuffer>>, // Scrollback buffer for PTY output
    pub stream_id: String, // Unique stream ID for this PTY process
}

impl std::fmt::Debug for PtyProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyProcess")
            .field("muxbox_id", &self.muxbox_id)
            .field("process_id", &self.process_id)
            .field("status", &self.status)
            .field("master_pty", &self.master_pty.is_some())
            .field("can_kill", &self.can_kill)
            .field(
                "output_buffer_size",
                &self.output_buffer.lock().unwrap().len(),
            )
            .field("stream_id", &self.stream_id)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PtyStatus {
    Starting,
    Running,
    Finished(i32), // exit code
    Error(String),
    FailedFallback, // PTY failed, fell back to regular execution
    Dead(String),   // PTY process died unexpectedly with reason
}

// F0122: PTY Thread Integration - Now thread-safe by creating PTY system on-demand
unsafe impl Send for PtyManager {}
unsafe impl Sync for PtyManager {}

#[derive(Debug, Clone)]
pub struct PtyManager {
    active_ptys: Arc<Mutex<HashMap<String, PtyProcess>>>,
    pty_failures: Arc<Mutex<HashMap<String, Vec<String>>>>, // Track failure reasons per muxbox
}

impl PtyManager {
    pub fn new() -> Result<Self> {
        Ok(PtyManager {
            active_ptys: Arc::new(Mutex::new(HashMap::new())),
            pty_failures: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Handle ExecuteScript message for PTY execution
    pub fn handle_execute_script(
        &self,
        execute_script: &crate::model::common::ExecuteScript,
        sender: std::sync::mpsc::Sender<(uuid::Uuid, crate::thread_manager::Message)>,
        thread_uuid: uuid::Uuid,
    ) -> Result<()> {
        log::info!(
            "PTYManager handling ExecuteScript for target_box_id: {}, stream_id: {}",
            execute_script.target_box_id,
            execute_script.stream_id
        );

        let libs = if execute_script.libs.is_empty() {
            None
        } else {
            Some(execute_script.libs.clone())
        };

        self.spawn_pty_script_with_redirect(
            execute_script.target_box_id.clone(),
            &execute_script.script,
            libs,
            sender,
            thread_uuid,
            execute_script.redirect_output.clone(),
            Some(execute_script.stream_id.clone()),
        )
    }

    /// Spawn a script in a PTY for the given muxbox
    pub fn spawn_pty_script(
        &self,
        muxbox_id: String,
        script_commands: &[String],
        libs: Option<Vec<String>>,
        sender: std::sync::mpsc::Sender<(uuid::Uuid, crate::thread_manager::Message)>,
        thread_uuid: uuid::Uuid,
        stream_id: Option<String>,
    ) -> Result<()> {
        self.spawn_pty_script_with_redirect(
            muxbox_id,
            script_commands,
            libs,
            sender,
            thread_uuid,
            None,
            stream_id, // Pass provided stream_id
        )
    }

    /// Spawn a script in a PTY for the given muxbox with optional output redirection
    pub fn spawn_pty_script_with_redirect(
        &self,
        muxbox_id: String,
        script_commands: &[String],
        libs: Option<Vec<String>>,
        sender: std::sync::mpsc::Sender<(uuid::Uuid, crate::thread_manager::Message)>,
        thread_uuid: uuid::Uuid,
        redirect_target: Option<String>,
        stream_id: Option<String>, // Custom stream ID to use for all output
    ) -> Result<()> {
        // SOURCE OBJECT ARCHITECTURE: stream_id must be provided from source object - no fallbacks
        let pty_stream_id = stream_id.expect("PTY execution requires stream_id from source object - architectural issue if None");
        
        log::info!(
            "Starting PTY script execution for muxbox: {}, redirect: {:?}, script_lines: {}, stream_id: {}",
            muxbox_id,
            redirect_target,
            script_commands.len(),
            pty_stream_id
        );

        // Create PTY with appropriate size (will be resized when muxbox bounds are known)
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        log::debug!("PTY size configured: {}x{}", pty_size.cols, pty_size.rows);

        // Create PTY system on-demand for thread safety
        let pty_system = portable_pty::native_pty_system();
        log::info!("PTY system created, attempting to allocate PTY pair");

        let pty_pair = match pty_system.openpty(pty_size) {
            Ok(pair) => {
                log::info!("PTY pair allocated successfully");
                pair
            }
            Err(e) => {
                let error_msg = format!("PTY allocation failed: {}", e);
                log::error!("{}", error_msg);

                // Track this failure for recovery purposes
                self.record_pty_failure(muxbox_id.clone(), error_msg.clone());

                return Err(e.into());
            }
        };
        let reader = pty_pair.master;
        let writer = pty_pair.slave;

        // Build the script content
        let mut script_content = String::new();
        if let Some(paths) = libs {
            for lib in paths {
                script_content.push_str(&format!("source {}\n", lib));
            }
        }

        for command in script_commands {
            script_content.push_str(&format!("{}\n", command));
        }

        // Create command to run in PTY
        let mut cmd = CommandBuilder::new("bash");
        cmd.arg("-c");
        cmd.arg(&script_content);

        // Spawn the process
        let mut child = match writer.spawn_command(cmd) {
            Ok(child) => {
                log::info!(
                    "PTY process spawned successfully - PID: {:?}",
                    child.process_id()
                );
                child
            }
            Err(e) => {
                let error_msg = format!("PTY process spawn failed: {}", e);
                log::error!("{}", error_msg);

                // Track this failure for recovery purposes
                self.record_pty_failure(muxbox_id.clone(), error_msg.clone());

                return Err(e.into());
            }
        };
        let process_id = child.process_id();

        // Store master PTY for resize operations using Arc<Mutex<>> for thread-safe sharing
        let master_pty_handle = Arc::new(Mutex::new(reader));
        let master_pty_clone = master_pty_handle.clone();

        // Create circular buffer for PTY output with configurable size (default 10,000 lines)
        let output_buffer = Arc::new(Mutex::new(CircularBuffer::new(10000)));
        let buffer_clone = output_buffer.clone();

        // Create PTY process record - use the stream_id determined at the start of function
        
        let pty_process = PtyProcess {
            muxbox_id: muxbox_id.clone(),
            process_id,
            status: PtyStatus::Running,
            master_pty: Some(master_pty_handle),
            can_kill: true, // Process can be killed
            output_buffer,
            stream_id: pty_stream_id.clone(),
        };

        // Store in active PTYs
        {
            let mut active_ptys = self.active_ptys.lock().unwrap();
            active_ptys.insert(muxbox_id.clone(), pty_process);
        }

        // Determine target muxbox for output (original muxbox or redirect target)
        let output_target = redirect_target.clone().unwrap_or_else(|| muxbox_id.clone());

        // Spawn reader thread for PTY output
        let active_ptys_clone = self.active_ptys.clone();
        let muxbox_id_clone = muxbox_id.clone();
        let _pty_stream_id_clone = pty_stream_id.clone();
        let thread_uuid_clone = thread_uuid; // Pass correct UUID to PTY reader thread

        thread::spawn(move || {
            log::info!(
                "PTY reader thread started for muxbox: {}, output_target: {}",
                muxbox_id_clone,
                output_target
            );

            let mut buffer = [0u8; 4096];
            let mut ansi_processor = AnsiProcessor::with_screen_size(80, 24);
            // Start in line mode - will auto-detect and switch to screen mode if needed
            ansi_processor.set_screen_mode(false);
            let mut bytes_processed = 0u64;
            let mut _messages_sent = 0u32;

            // Create reader once outside the loop using cloned master PTY handle
            let mut pty_reader = match master_pty_clone.lock().unwrap().try_clone_reader() {
                Ok(reader) => {
                    log::info!("PTY reader cloned successfully");
                    reader
                }
                Err(e) => {
                    log::error!("Failed to clone PTY reader: {}", e);
                    return;
                }
            };

            loop {
                // Use non-blocking read with timeout to enable live streaming
                match pty_reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - process has ended
                        debug!("PTY EOF for muxbox: {}", muxbox_id_clone);

                        // Wait for child process and get exit status
                        let exit_code = match child.wait() {
                            Ok(status) => status.exit_code(),
                            Err(_) => 1,
                        };

                        // Update PTY status
                        {
                            let mut active_ptys = active_ptys_clone.lock().unwrap();
                            if let Some(pty_proc) = active_ptys.get_mut(&muxbox_id_clone) {
                                pty_proc.status = PtyStatus::Finished(exit_code as i32);
                            }
                        }

                        // Send remaining terminal screen content for stream integration
                        let remaining_text = ansi_processor.get_screen_content_for_stream();
                        if !remaining_text.is_empty() {
                            let pty_reader_uuid = uuid::Uuid::new_v4();
                            if let Err(e) = sender.send((
                                pty_reader_uuid,
                                // T0328: Replace MuxBoxOutputUpdate with StreamUpdateMessage
                                crate::thread_manager::Message::StreamUpdateMessage(crate::model::common::StreamUpdate {
                                    stream_id: pty_stream_id.clone(),
                                    target_box_id: output_target.clone(),
                                    content_update: format!("{}\n", remaining_text), // Add newline for final output
                                    source_state: crate::model::common::SourceState::Pty(
                                        crate::model::common::PtySourceState {
                                            process_id: 0,
                                            runtime: std::time::Duration::from_millis(0),
                                            exit_code: None,
                                            status: crate::model::common::ExecutionPtyStatus::Completed,
                                        }
                                    ),
                                    execution_mode: crate::model::common::ExecutionMode::Pty,
                                }),
                            )) {
                                error!("Failed to send final PTY output: {}", e);
                            }
                        }

                        // Send final message indicating completion
                        if let Err(e) = sender.send((
                            thread_uuid_clone,
                            crate::thread_manager::Message::StreamUpdateMessage(crate::model::common::StreamUpdate {
                                stream_id: pty_stream_id.clone(),
                                target_box_id: output_target.clone(),
                                content_update: format!("\n[Process exited with code {}]\n", exit_code),
                                source_state: crate::model::common::SourceState::Pty(
                                    crate::model::common::PtySourceState {
                                        process_id: 0,
                                        runtime: std::time::Duration::from_millis(0),
                                        exit_code: Some(exit_code as i32),
                                        status: crate::model::common::ExecutionPtyStatus::Completed,
                                    }
                                ),
                                execution_mode: crate::model::common::ExecutionMode::Pty,
                            }),
                        )) {
                            error!("Failed to send PTY completion message: {}", e);
                        }
                        break;
                    }
                    Ok(bytes_read) => {
                        bytes_processed += bytes_read as u64;
                        log::debug!(
                            "Read {} bytes from PTY (total: {})",
                            bytes_read,
                            bytes_processed
                        );

                        // Process raw bytes through ANSI processor
                        ansi_processor.process_bytes(&buffer[..bytes_read]);

                        // Get terminal screen content for stream integration
                        // This feeds PTY TerminalScreenBuffer into BoxMux's main differential drawing system
                        let content_to_send = ansi_processor.get_screen_content_for_stream();
                        let should_replace = ansi_processor.should_replace_content();
                        
                        if !content_to_send.trim().is_empty() {
                            if should_replace {
                                log::info!("Full-screen program detected - sending terminal screen content ({} chars) to {}", 
                                          content_to_send.len(), output_target);
                            } else {
                                log::info!("Line-based program - sending processed content ({} chars) to {}", 
                                          content_to_send.len(), output_target);
                            }
                        } else {
                            continue; // No content to send
                        }
                        
                        // Send content if we have any
                        if !content_to_send.trim().is_empty() {
                            // Store content in circular buffer for scrollback
                            if let Ok(mut buffer) = buffer_clone.lock() {
                                buffer.push(content_to_send.clone());
                                debug!("Added content to PTY buffer (total: {})", buffer.len());
                            }

                            // Create appropriate content update based on program behavior
                            let content_update = if should_replace {
                                format!("REPLACE:{}", content_to_send) // Signal to replace, not append
                            } else {
                                content_to_send // Normal append behavior
                            };
                            
                            let message = crate::thread_manager::Message::StreamUpdateMessage(crate::model::common::StreamUpdate {
                                stream_id: pty_stream_id.clone(),
                                target_box_id: output_target.clone(),
                                content_update,
                                source_state: crate::model::common::SourceState::Pty(
                                    crate::model::common::PtySourceState {
                                        process_id: 0,
                                        runtime: std::time::Duration::from_millis(0),
                                        exit_code: None,
                                        status: crate::model::common::ExecutionPtyStatus::Running,
                                    }
                                ),
                                execution_mode: crate::model::common::ExecutionMode::Pty,
                            });
                            
                            // Use correct thread_uuid for ThreadManager message routing
                            debug!(
                                "About to send PTY message via channel - thread_uuid: {:?}",
                                thread_uuid_clone
                            );
                            if let Err(e) = sender.send((thread_uuid_clone, message)) {
                                error!("PTY message send failed - channel disconnected or full: {}", e);
                                break;
                            } else {
                                debug!("PTY message sent successfully via channel");
                            }
                            _messages_sent += 1;
                        }
                    }
                    Err(e) => {
                        error!("Error reading from PTY: {}", e);

                        // Update PTY status to error
                        {
                            let mut active_ptys = active_ptys_clone.lock().unwrap();
                            if let Some(pty_proc) = active_ptys.get_mut(&muxbox_id_clone) {
                                pty_proc.status = PtyStatus::Error(e.to_string());
                            }
                        }

                        // Send error message
                        if let Err(e) = sender.send((
                            thread_uuid_clone,
                            crate::thread_manager::Message::StreamUpdateMessage(crate::model::common::StreamUpdate {
                                stream_id: pty_stream_id.clone(),
                                target_box_id: muxbox_id_clone.clone(),
                                content_update: format!("[PTY Error: {}]", e),
                                source_state: crate::model::common::SourceState::Pty(
                                    crate::model::common::PtySourceState {
                                        process_id: 0,
                                        runtime: std::time::Duration::from_millis(0),
                                        exit_code: Some(1),
                                        status: crate::model::common::ExecutionPtyStatus::Failed("PTY error".to_string()),
                                    }
                                ),
                                execution_mode: crate::model::common::ExecutionMode::Pty,
                            }),
                        )) {
                            error!("Failed to send PTY error message: {}", e);
                        }
                        break;
                    }
                }

                // Small delay to prevent busy waiting
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Ok(())
    }

    /// Send input to a PTY
    pub fn send_input(&self, muxbox_id: &str, input: &str) -> Result<()> {
        debug!("PTY input for muxbox {}: {}", muxbox_id, input);

        let active_ptys = self.active_ptys.lock().unwrap();

        if let Some(pty_process) = active_ptys.get(muxbox_id) {
            // Check if PTY is running and can accept input
            match pty_process.status {
                PtyStatus::Running | PtyStatus::Starting => {
                    if let Some(master_pty_handle) = &pty_process.master_pty {
                        // Real PTY - write to actual PTY by getting a writer
                        let master_pty = master_pty_handle.lock().unwrap();

                        // Get writer from master PTY
                        let mut writer = master_pty
                            .take_writer()
                            .map_err(|e| anyhow::anyhow!("Failed to get PTY writer: {}", e))?;

                        // Write input to PTY
                        use std::io::Write;
                        writer
                            .write_all(input.as_bytes())
                            .map_err(|e| anyhow::anyhow!("Failed to write to PTY: {}", e))?;

                        writer
                            .flush()
                            .map_err(|e| anyhow::anyhow!("Failed to flush PTY writer: {}", e))?;

                        debug!(
                            "Successfully sent {} bytes to real PTY {}",
                            input.len(),
                            muxbox_id
                        );
                        Ok(())
                    } else {
                        // Test PTY or PTY without master handle - simulate successful input
                        #[cfg(test)]
                        {
                            debug!(
                                "Test PTY - simulating successful input send for {}",
                                muxbox_id
                            );
                            Ok(())
                        }
                        #[cfg(not(test))]
                        {
                            Err(anyhow::anyhow!(
                                "No master PTY handle available for muxbox: {}",
                                muxbox_id
                            ))
                        }
                    }
                }
                PtyStatus::Finished(_) => Err(anyhow::anyhow!(
                    "PTY process {} has finished and cannot accept input",
                    muxbox_id
                )),
                PtyStatus::Error(ref err) => Err(anyhow::anyhow!(
                    "PTY process {} is in error state: {}",
                    muxbox_id,
                    err
                )),
                PtyStatus::FailedFallback => Err(anyhow::anyhow!(
                    "PTY process {} failed and fell back to regular execution",
                    muxbox_id
                )),
                PtyStatus::Dead(ref reason) => Err(anyhow::anyhow!(
                    "PTY process {} is dead: {}",
                    muxbox_id,
                    reason
                )),
            }
        } else {
            Err(anyhow::anyhow!(
                "No PTY process found for muxbox: {}",
                muxbox_id
            ))
        }
    }

    /// Resize a PTY to match muxbox dimensions
    pub fn resize_pty(&mut self, muxbox_id: &str, rows: u16, cols: u16) -> Result<()> {
        debug!("Resizing PTY for muxbox {} to {}x{}", muxbox_id, cols, rows);

        let active_ptys = self.active_ptys.lock().unwrap();

        if let Some(pty_process) = active_ptys.get(muxbox_id) {
            if let Some(master_pty_handle) = &pty_process.master_pty {
                let pty_size = PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                };

                match master_pty_handle.lock().unwrap().resize(pty_size) {
                    Ok(_) => {
                        debug!(
                            "PTY successfully resized for muxbox {} to {}x{}",
                            muxbox_id, cols, rows
                        );
                    }
                    Err(e) => {
                        warn!("PTY resize failed for muxbox {}: {}", muxbox_id, e);
                        return Err(e.into());
                    }
                }
            } else {
                debug!("No master PTY handle available for muxbox: {}", muxbox_id);
            }
        } else {
            debug!("PTY not found for muxbox: {}", muxbox_id);
        }

        Ok(())
    }

    /// Kill a PTY process
    pub fn kill_pty(&mut self, muxbox_id: &str) -> Result<()> {
        let mut active_ptys = self.active_ptys.lock().unwrap();

        if let Some(pty_process) = active_ptys.get_mut(muxbox_id) {
            debug!("Killing PTY process for muxbox: {}", muxbox_id);

            if pty_process.can_kill {
                if let Some(pid) = pty_process.process_id {
                    // Use system kill command for cross-platform killing
                    #[cfg(unix)]
                    {
                        use std::process::Command;
                        match Command::new("kill").arg("-9").arg(pid.to_string()).output() {
                            Ok(_) => {
                                debug!(
                                    "Successfully killed PTY process {} for muxbox: {}",
                                    pid, muxbox_id
                                );
                                pty_process.status = PtyStatus::Finished(-9); // SIGKILL
                                pty_process.can_kill = false; // Already killed
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to kill PTY process {} for muxbox {}: {}",
                                    pid, muxbox_id, e
                                );
                                pty_process.status =
                                    PtyStatus::Error(format!("Kill failed: {}", e));
                            }
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        warn!(
                            "Process killing not supported on this platform for muxbox: {}",
                            muxbox_id
                        );
                        pty_process.status = PtyStatus::Error("Kill not supported".to_string());
                    }
                } else {
                    warn!("No process ID available for muxbox: {}", muxbox_id);
                    pty_process.status = PtyStatus::Error("No process ID".to_string());
                }
            } else {
                debug!(
                    "Process for muxbox {} already killed or cannot be killed",
                    muxbox_id
                );
            }

            // Drop the PTY handle to clean up resources
            pty_process.master_pty = None;
        }

        Ok(())
    }

    /// Get status of a PTY process
    pub fn get_pty_status(&self, muxbox_id: &str) -> Option<PtyStatus> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).map(|p| p.status.clone())
    }

    /// Clean up finished PTY processes
    pub fn cleanup_finished(&mut self) {
        let mut active_ptys = self.active_ptys.lock().unwrap();

        active_ptys.retain(|muxbox_id, pty_process| {
            match &pty_process.status {
                PtyStatus::Finished(_) | PtyStatus::Error(_) => {
                    debug!("Cleaning up finished PTY for muxbox: {}", muxbox_id);
                    false // Remove from map
                }
                _ => true, // Keep running PTYs
            }
        });
    }

    /// Get list of active PTY muxbox IDs
    pub fn get_active_pty_muxboxes(&self) -> Vec<String> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.keys().cloned().collect()
    }

    /// Send signal to a PTY process (future enhancement)
    pub fn send_signal(&mut self, muxbox_id: &str, _signal: i32) -> Result<()> {
        debug!(
            "Signal send request for muxbox: {} (not yet implemented)",
            muxbox_id
        );
        // TODO: Implement signal sending when portable_pty supports it
        // For now, only kill() is supported through kill_pty()
        Ok(())
    }

    /// Get process information for a PTY
    pub fn get_process_info(&self, muxbox_id: &str) -> Option<(u32, PtyStatus)> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys
            .get(muxbox_id)
            .and_then(|p| p.process_id.map(|pid| (pid, p.status.clone())))
    }

    /// Check if a PTY process is still running
    pub fn is_process_running(&self, muxbox_id: &str) -> bool {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys
            .get(muxbox_id)
            .map(|p| matches!(p.status, PtyStatus::Running | PtyStatus::Starting))
            .unwrap_or(false)
    }

    /// Record a PTY failure for error recovery tracking
    fn record_pty_failure(&self, muxbox_id: String, error_msg: String) {
        let mut pty_failures = self.pty_failures.lock().unwrap();
        pty_failures
            .entry(muxbox_id)
            .or_insert_with(Vec::new)
            .push(error_msg);
    }

    /// Get PTY failure history for a muxbox
    pub fn get_pty_failure_history(&self, muxbox_id: &str) -> Vec<String> {
        let pty_failures = self.pty_failures.lock().unwrap();
        pty_failures.get(muxbox_id).cloned().unwrap_or_default()
    }

    /// Check if a muxbox has had recent PTY failures (within last few attempts)
    pub fn has_recent_pty_failures(&self, muxbox_id: &str, threshold: usize) -> bool {
        let pty_failures = self.pty_failures.lock().unwrap();
        pty_failures
            .get(muxbox_id)
            .map(|failures| failures.len() >= threshold)
            .unwrap_or(false)
    }

    /// Clear PTY failure history for a muxbox (useful after successful execution)
    pub fn clear_pty_failures(&self, muxbox_id: &str) {
        let mut pty_failures = self.pty_failures.lock().unwrap();
        pty_failures.remove(muxbox_id);
    }

    /// Check if PTY should be avoided for this muxbox due to repeated failures
    pub fn should_avoid_pty(&self, muxbox_id: &str) -> bool {
        self.has_recent_pty_failures(muxbox_id, 3) // Avoid PTY after 3 consecutive failures
    }

    /// Reset PTY failure tracking (useful for testing or reset operations)
    pub fn reset_failure_tracking(&self) {
        let mut pty_failures = self.pty_failures.lock().unwrap();
        pty_failures.clear();
    }

    /// Get the circular buffer for a PTY muxbox (for scrollback access)
    pub fn get_output_buffer(&self, muxbox_id: &str) -> Option<Arc<Mutex<CircularBuffer>>> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).map(|p| p.output_buffer.clone())
    }

    /// Get scrollback content for a PTY muxbox
    pub fn get_scrollback_content(&self, muxbox_id: &str) -> Option<String> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).and_then(|p| {
            p.output_buffer
                .lock()
                .ok()
                .map(|buffer| buffer.get_content())
        })
    }

    /// Get recent lines from a PTY muxbox's buffer
    pub fn get_recent_lines(&self, muxbox_id: &str, line_count: usize) -> Option<Vec<String>> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).and_then(|p| {
            p.output_buffer
                .lock()
                .ok()
                .map(|buffer| buffer.get_last_lines(line_count))
        })
    }

    /// Search for text in a PTY muxbox's buffer
    pub fn search_buffer(&self, muxbox_id: &str, query: &str) -> Option<Vec<(usize, String)>> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).and_then(|p| {
            p.output_buffer
                .lock()
                .ok()
                .map(|buffer| buffer.search(query))
        })
    }

    /// Get buffer statistics for a PTY muxbox
    pub fn get_buffer_stats(&self, muxbox_id: &str) -> Option<crate::circular_buffer::BufferStats> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys
            .get(muxbox_id)
            .and_then(|p| p.output_buffer.lock().ok().map(|buffer| buffer.get_stats()))
    }

    /// Configure buffer size for a PTY muxbox (can be called before or during execution)
    pub fn set_buffer_size(&self, muxbox_id: &str, max_size: usize) -> Result<()> {
        let active_ptys = self.active_ptys.lock().unwrap();
        if let Some(pty_process) = active_ptys.get(muxbox_id) {
            if let Ok(mut buffer) = pty_process.output_buffer.lock() {
                buffer.set_max_size(max_size);
                debug!("Set buffer size for muxbox {} to {}", muxbox_id, max_size);
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Failed to lock buffer for muxbox: {}",
                    muxbox_id
                ))
            }
        } else {
            Err(anyhow::anyhow!("PTY not found for muxbox: {}", muxbox_id))
        }
    }

    /// Clear the buffer for a PTY muxbox
    pub fn clear_buffer(&self, muxbox_id: &str) -> Result<()> {
        let active_ptys = self.active_ptys.lock().unwrap();
        if let Some(pty_process) = active_ptys.get(muxbox_id) {
            if let Ok(mut buffer) = pty_process.output_buffer.lock() {
                buffer.clear();
                debug!("Cleared buffer for muxbox {}", muxbox_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Failed to lock buffer for muxbox: {}",
                    muxbox_id
                ))
            }
        } else {
            Err(anyhow::anyhow!("PTY not found for muxbox: {}", muxbox_id))
        }
    }

    /// Get buffer content within a specific range (for pagination/scrolling)
    pub fn get_buffer_range(
        &self,
        muxbox_id: &str,
        start: usize,
        count: usize,
    ) -> Option<Vec<String>> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).and_then(|p| {
            p.output_buffer
                .lock()
                .ok()
                .map(|buffer| buffer.get_lines_range(start, count))
        })
    }

    /// Get the stream ID for a PTY process
    pub fn get_stream_id(&self, muxbox_id: &str) -> Option<String> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).map(|pty| pty.stream_id.clone())
    }

    /// Get detailed process information for display purposes
    /// F0132: PTY Process Info - Enhanced process details for status display
    pub fn get_detailed_process_info(&self, muxbox_id: &str) -> Option<ProcessInfo> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(muxbox_id).map(|p| ProcessInfo {
            muxbox_id: p.muxbox_id.clone(),
            process_id: p.process_id,
            status: p.status.clone(),
            can_kill: p.can_kill,
            buffer_lines: p.output_buffer.lock().map(|buf| buf.len()).unwrap_or(0),
            is_running: matches!(p.status, PtyStatus::Running | PtyStatus::Starting),
        })
    }

    /// Get process status summary for compact display
    /// F0132: PTY Process Info - Compact status for title display
    /// F0135: PTY Error States - Enhanced with dead process indication
    pub fn get_process_status_summary(&self, muxbox_id: &str) -> Option<String> {
        self.get_process_info(muxbox_id).map(|(pid, status)| {
            let status_text = match status {
                PtyStatus::Starting => "Starting".to_string(),
                PtyStatus::Running => "Running".to_string(),
                PtyStatus::Finished(code) => {
                    if code == 0 {
                        "Done".to_string()
                    } else {
                        format!("Exit:{}", code)
                    }
                }
                PtyStatus::Error(ref msg) => {
                    if msg.len() > 10 {
                        format!("Error:{}", &msg[..7])
                    } else {
                        format!("Error:{}", msg)
                    }
                }
                PtyStatus::FailedFallback => "Fallback".to_string(),
                PtyStatus::Dead(ref reason) => {
                    if reason.len() > 8 {
                        format!("Dead:{}", &reason[..5])
                    } else {
                        format!("Dead:{}", reason)
                    }
                }
            };
            format!("PID:{} {}", pid, status_text)
        })
    }

    /// Check if a PTY process is in an error state
    /// F0135: PTY Error States - Detect processes requiring visual error indication
    pub fn is_pty_in_error_state(&self, muxbox_id: &str) -> bool {
        if let Some((_, status)) = self.get_process_info(muxbox_id) {
            matches!(
                status,
                PtyStatus::Error(_) | PtyStatus::Dead(_) | PtyStatus::FailedFallback
            )
        } else {
            false
        }
    }

    /// Check if a PTY process is dead and needs recovery
    /// F0135: PTY Error States - Detect dead processes needing restart
    pub fn is_pty_dead(&self, muxbox_id: &str) -> bool {
        if let Some((_, status)) = self.get_process_info(muxbox_id) {
            matches!(status, PtyStatus::Dead(_))
        } else {
            false
        }
    }

    /// Get error state details for recovery UI
    /// F0135: PTY Error States - Provide error context for recovery actions
    pub fn get_error_state_info(&self, muxbox_id: &str) -> Option<ErrorStateInfo> {
        if let Some((pid, status)) = self.get_process_info(muxbox_id) {
            match status {
                PtyStatus::Error(msg) => Some(ErrorStateInfo {
                    muxbox_id: muxbox_id.to_string(),
                    error_type: ErrorType::ExecutionError,
                    message: msg,
                    pid: Some(pid),
                    can_retry: true,
                    suggested_action: "Retry PTY execution".to_string(),
                }),
                PtyStatus::Dead(reason) => Some(ErrorStateInfo {
                    muxbox_id: muxbox_id.to_string(),
                    error_type: ErrorType::ProcessDied,
                    message: reason,
                    pid: Some(pid),
                    can_retry: true,
                    suggested_action: "Restart PTY process".to_string(),
                }),
                PtyStatus::FailedFallback => Some(ErrorStateInfo {
                    muxbox_id: muxbox_id.to_string(),
                    error_type: ErrorType::FallbackUsed,
                    message: "PTY failed, using regular execution".to_string(),
                    pid: Some(pid),
                    can_retry: true,
                    suggested_action: "Reset PTY and retry".to_string(),
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Mark a PTY process as dead with reason
    /// F0135: PTY Error States - Set dead status for error visualization
    pub fn mark_pty_dead(&self, muxbox_id: &str, reason: String) -> Result<(), anyhow::Error> {
        let mut active_ptys = self.active_ptys.lock().unwrap();
        if let Some(pty_process) = active_ptys.get_mut(muxbox_id) {
            pty_process.status = PtyStatus::Dead(reason);
            Ok(())
        } else {
            Err(anyhow::anyhow!("PTY not found for muxbox: {}", muxbox_id))
        }
    }

    /// Kill a PTY process via socket command
    /// F0137: Socket PTY Control - Terminate PTY process remotely
    pub fn kill_pty_process(&self, muxbox_id: &str) -> Result<(), anyhow::Error> {
        let mut active_ptys = self.active_ptys.lock().unwrap();
        if let Some(pty_process) = active_ptys.get_mut(muxbox_id) {
            if let Some(pid) = pty_process.process_id {
                // Use existing kill functionality
                if pty_process.can_kill {
                    // Kill the process using system kill command
                    let kill_result = if cfg!(target_os = "windows") {
                        std::process::Command::new("taskkill")
                            .args(&["/F", "/PID", &pid.to_string()])
                            .output()
                    } else {
                        std::process::Command::new("kill")
                            .args(&["-9", &pid.to_string()])
                            .output()
                    };

                    match kill_result {
                        Ok(output) => {
                            if output.status.success() {
                                pty_process.status = PtyStatus::Finished(-9); // Killed signal
                                log::info!("Killed PTY process {} for muxbox {}", pid, muxbox_id);
                                Ok(())
                            } else {
                                let error_msg = format!(
                                    "Kill command failed: {}",
                                    String::from_utf8_lossy(&output.stderr)
                                );
                                pty_process.status = PtyStatus::Error(error_msg.clone());
                                Err(anyhow::anyhow!(error_msg))
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to execute kill command: {}", e);
                            pty_process.status = PtyStatus::Error(error_msg.clone());
                            Err(anyhow::anyhow!(error_msg))
                        }
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Process {} for muxbox {} cannot be killed",
                        pid,
                        muxbox_id
                    ))
                }
            } else {
                Err(anyhow::anyhow!(
                    "No process ID available for muxbox {}",
                    muxbox_id
                ))
            }
        } else {
            Err(anyhow::anyhow!("PTY not found for muxbox: {}", muxbox_id))
        }
    }

    /// Restart a PTY process via socket command  
    /// F0137: Socket PTY Control - Restart PTY process after termination
    pub fn restart_pty_process(&self, muxbox_id: &str) -> Result<(), anyhow::Error> {
        // First, kill the existing process if it's still running
        if self.is_process_running(muxbox_id) {
            if let Err(e) = self.kill_pty_process(muxbox_id) {
                log::warn!("Failed to kill existing process during restart: {}", e);
            }
            // Give the process time to terminate
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // Clear the failure tracking for this muxbox to allow PTY retry
        self.clear_pty_failures(muxbox_id);

        // Mark the PTY as needing restart by setting it to Starting status
        let mut active_ptys = self.active_ptys.lock().unwrap();
        if let Some(pty_process) = active_ptys.get_mut(muxbox_id) {
            pty_process.status = PtyStatus::Starting;
            pty_process.process_id = None; // Clear old PID
            log::info!("Marked PTY process for restart on muxbox {}", muxbox_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("PTY not found for muxbox: {}", muxbox_id))
        }
    }

    /// Test helper - Add a PTY process for testing purposes
    #[cfg(test)]
    pub fn add_test_pty_process(&self, muxbox_id: String, buffer: Arc<Mutex<CircularBuffer>>) {
        let pty_process = PtyProcess {
            muxbox_id: muxbox_id.clone(),
            process_id: Some(12345),
            status: PtyStatus::Running,
            master_pty: None,
            can_kill: false,
            output_buffer: buffer,
            stream_id: format!("pty-test-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        };

        self.active_ptys
            .lock()
            .unwrap()
            .insert(muxbox_id, pty_process);
    }

    /// Test helper - Add a PTY process with custom status
    #[cfg(test)]
    pub fn add_test_pty_process_with_status(
        &self,
        muxbox_id: String,
        buffer: Arc<Mutex<CircularBuffer>>,
        status: PtyStatus,
        pid: u32,
    ) {
        let pty_process = PtyProcess {
            muxbox_id: muxbox_id.clone(),
            process_id: Some(pid),
            status,
            master_pty: None,
            can_kill: false,
            output_buffer: buffer,
            stream_id: format!("pty-test-{}", &uuid::Uuid::new_v4().to_string()[..8]),
        };

        self.active_ptys
            .lock()
            .unwrap()
            .insert(muxbox_id, pty_process);
    }

    /// Test helper - Set PTY process killability for testing
    #[cfg(test)]
    pub fn set_pty_killable(&self, muxbox_id: &str, can_kill: bool) {
        let mut active_ptys = self.active_ptys.lock().unwrap();
        if let Some(pty_process) = active_ptys.get_mut(muxbox_id) {
            pty_process.can_kill = can_kill;
        }
    }
}

/// F0132: PTY Process Info - Detailed process information structure
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub muxbox_id: String,
    pub process_id: Option<u32>,
    pub status: PtyStatus,
    pub can_kill: bool,
    pub buffer_lines: usize,
    pub is_running: bool,
}

/// F0135: PTY Error States - Error state information for recovery UI
#[derive(Debug, Clone)]
pub struct ErrorStateInfo {
    pub muxbox_id: String,
    pub error_type: ErrorType,
    pub message: String,
    pub pid: Option<u32>,
    pub can_retry: bool,
    pub suggested_action: String,
}

/// F0135: PTY Error States - Types of PTY errors
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    ExecutionError, // Script/command execution failed
    ProcessDied,    // PTY process died unexpectedly
    FallbackUsed,   // PTY failed, using regular execution
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PTY manager")
    }
}

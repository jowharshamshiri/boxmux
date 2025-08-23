use anyhow::Result;
use log::{debug, error, warn};
// Use log crate for debugging
use portable_pty::{CommandBuilder, PtySize, MasterPty};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::ansi_processor::AnsiProcessor;

pub struct PtyProcess {
    pub panel_id: String,
    pub process_id: Option<u32>,
    pub status: PtyStatus,
    pub master_pty: Option<Box<dyn MasterPty + Send>>,
}

impl std::fmt::Debug for PtyProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PtyProcess")
            .field("panel_id", &self.panel_id)
            .field("process_id", &self.process_id)
            .field("status", &self.status)
            .field("master_pty", &self.master_pty.is_some())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PtyStatus {
    Starting,
    Running,
    Finished(i32), // exit code
    Error(String),
}

// F0122: PTY Thread Integration - Now thread-safe by creating PTY system on-demand
unsafe impl Send for PtyManager {}
unsafe impl Sync for PtyManager {}

#[derive(Debug)]
pub struct PtyManager {
    active_ptys: Arc<Mutex<HashMap<String, PtyProcess>>>,
}

impl PtyManager {
    pub fn new() -> Result<Self> {        
        Ok(PtyManager {
            active_ptys: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Spawn a script in a PTY for the given panel
    pub fn spawn_pty_script(
        &self,
        panel_id: String,
        script_commands: &[String],
        libs: Option<Vec<String>>,
        sender: std::sync::mpsc::Sender<(uuid::Uuid, crate::thread_manager::Message)>,
        thread_uuid: uuid::Uuid,
    ) -> Result<()> {
        self.spawn_pty_script_with_redirect(panel_id, script_commands, libs, sender, thread_uuid, None)
    }

    /// Spawn a script in a PTY for the given panel with optional output redirection
    pub fn spawn_pty_script_with_redirect(
        &self,
        panel_id: String,
        script_commands: &[String],
        libs: Option<Vec<String>>,
        sender: std::sync::mpsc::Sender<(uuid::Uuid, crate::thread_manager::Message)>,
        thread_uuid: uuid::Uuid,
        redirect_target: Option<String>,
    ) -> Result<()> {
        log::info!("Starting PTY script execution for panel: {}, redirect: {:?}, script_lines: {}", 
               panel_id, redirect_target, script_commands.len());

        // Create PTY with appropriate size (will be resized when panel bounds are known)
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
            },
            Err(e) => {
                log::error!("PTY allocation failed: {}", e);
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
                log::info!("PTY process spawned successfully - PID: {:?}", child.process_id());
                child
            },
            Err(e) => {
                log::error!("PTY process spawn failed: {}", e);
                return Err(e.into());
            }
        };
        let process_id = child.process_id();

        // Store the reader for resize operations (we'll store it in the process record)
        // For now, we don't store the master_pty since we can't safely clone it
        // TODO: Implement a better approach for resize that doesn't require cloning

        // Create PTY process record
        let pty_process = PtyProcess {
            panel_id: panel_id.clone(),
            process_id,
            status: PtyStatus::Running,
            master_pty: None, // We'll implement resize differently
        };

        // Store in active PTYs
        {
            let mut active_ptys = self.active_ptys.lock().unwrap();
            active_ptys.insert(panel_id.clone(), pty_process);
        }

        // Determine target panel for output (original panel or redirect target)
        let output_target = redirect_target.clone().unwrap_or_else(|| panel_id.clone());

        // Spawn reader thread for PTY output
        let active_ptys_clone = self.active_ptys.clone();
        let panel_id_clone = panel_id.clone();
        
        thread::spawn(move || {
            log::info!("PTY reader thread started for panel: {}, output_target: {}", panel_id_clone, output_target);
            
            let mut buffer = [0u8; 4096];
            let mut ansi_processor = AnsiProcessor::new();
            let mut bytes_processed = 0u64;
            let mut messages_sent = 0u32;
            
            // Create reader once outside the loop
            let mut pty_reader = match reader.try_clone_reader() {
                Ok(reader) => {
                    log::info!("PTY reader cloned successfully");
                    reader
                },
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
                        debug!("PTY EOF for panel: {}", panel_id_clone);
                        
                        // Wait for child process and get exit status
                        let exit_code = match child.wait() {
                            Ok(status) => status.exit_code(),
                            Err(_) => 1,
                        };

                        // Update PTY status
                        {
                            let mut active_ptys = active_ptys_clone.lock().unwrap();
                                    if let Some(pty_proc) = active_ptys.get_mut(&panel_id_clone) {
                                        pty_proc.status = PtyStatus::Finished(exit_code as i32);
                                    }
                                }

                                // Send any remaining processed output
                                let remaining_text = ansi_processor.get_processed_text();
                                if !remaining_text.is_empty() {
                                    if let Err(e) = sender.send((thread_uuid, crate::thread_manager::Message::PanelOutputUpdate(
                                        output_target.clone(),
                                        true,
                                        remaining_text.to_string(),
                                    ))) {
                                        error!("Failed to send final PTY output: {}", e);
                                    }
                                }

                                // Send final message indicating completion
                                if let Err(e) = sender.send((thread_uuid, crate::thread_manager::Message::PanelOutputUpdate(
                                    output_target.clone(),
                                    exit_code == 0,
                                    format!("[Process exited with code {}]", exit_code),
                                ))) {
                                    error!("Failed to send PTY completion message: {}", e);
                                }
                                break;
                            }
                            Ok(bytes_read) => {
                                bytes_processed += bytes_read as u64;
                                log::debug!("Read {} bytes from PTY (total: {})", bytes_read, bytes_processed);
                                
                                // Process raw bytes through ANSI processor
                                ansi_processor.process_bytes(&buffer[..bytes_read]);
                                
                                // Get processed text and send line by line
                                let processed_output = ansi_processor.get_processed_text().to_string();
                                let mut lines_to_send = Vec::new();
                                
                                // Split by newlines and collect complete lines
                                let current_lines: Vec<&str> = processed_output.split('\n').collect();
                                
                                // If we have complete lines (ending with newline), send them
                                if processed_output.ends_with('\n') {
                                    lines_to_send.extend(current_lines.iter().map(|s| s.to_string()));
                                    ansi_processor.clear_processed_text();
                                } else if current_lines.len() > 1 {
                                    // Send all but the last incomplete line
                                    lines_to_send.extend(current_lines[..current_lines.len()-1].iter().map(|s| s.to_string()));
                                    
                                    // Keep the last incomplete line in processor
                                    ansi_processor.clear_processed_text();
                                    if let Some(last_line) = current_lines.last() {
                                        ansi_processor.process_string(last_line);
                                    }
                                }
                                
                                // Send complete lines
                                for line in lines_to_send {
                                    if !line.trim().is_empty() {
                                        log::info!("Sending PTY line (len: {}) to {}: {}", line.len(), output_target, 
                                               line.chars().take(50).collect::<String>());
                                        let message = crate::thread_manager::Message::PanelOutputUpdate(
                                            output_target.clone(),
                                            true,
                                            line,
                                        );
                                        debug!("About to send message via channel - thread_uuid: {:?}", thread_uuid);
                                        if let Err(e) = sender.send((thread_uuid, message)) {
                                            error!("PTY message send failed - channel disconnected or full: {}", e);
                                            break;
                                        } else {
                                            debug!("PTY message sent successfully via channel");
                                        }
                                        messages_sent += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error reading from PTY: {}", e);
                                
                                // Update PTY status to error
                                {
                                    let mut active_ptys = active_ptys_clone.lock().unwrap();
                                    if let Some(pty_proc) = active_ptys.get_mut(&panel_id_clone) {
                                        pty_proc.status = PtyStatus::Error(e.to_string());
                                    }
                                }
                                
                                // Send error message
                                if let Err(e) = sender.send((thread_uuid, crate::thread_manager::Message::PanelOutputUpdate(
                                    panel_id_clone.clone(),
                                    false,
                                    format!("[PTY Error: {}]", e),
                                ))) {
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
    pub fn send_input(&self, panel_id: &str, input: &str) -> Result<()> {
        debug!("PTY input for panel {}: {}", panel_id, input);
        
        // TODO: Implement actual PTY input sending
        // For now, just log that we received the input request
        log::info!("PTY input routing request for panel {}: {}", panel_id, input.chars().take(10).collect::<String>());
        
        Ok(())
    }

    /// Resize a PTY to match panel dimensions
    pub fn resize_pty(&mut self, panel_id: &str, rows: u16, cols: u16) -> Result<()> {
        debug!("Resizing PTY for panel {} to {}x{}", panel_id, cols, rows);
        
        let active_ptys = self.active_ptys.lock().unwrap();
        
        if let Some(_pty_process) = active_ptys.get(panel_id) {
            // For now, we log the resize request but don't perform the actual resize
            // This requires architectural changes to safely store master PTY handles
            debug!("PTY resize requested for panel {} to {}x{} (implementation pending)", panel_id, cols, rows);
            
            // TODO: Implement actual PTY resize by:
            // 1. Storing master PTY handles safely
            // 2. Calling resize on the stored handle
            // For now, this provides the API without the full implementation
        } else {
            debug!("PTY not found for panel: {}", panel_id);
        }
        
        Ok(())
    }

    /// Kill a PTY process
    pub fn kill_pty(&mut self, panel_id: &str) -> Result<()> {
        let mut active_ptys = self.active_ptys.lock().unwrap();
        
        if let Some(pty_process) = active_ptys.get_mut(panel_id) {
            debug!("Killing PTY process for panel: {}", panel_id);
            
            // Mark as finished (killed)
            pty_process.status = PtyStatus::Finished(-9); // SIGKILL
            pty_process.master_pty = None; // Drop the PTY handle
            
            // Note: We would need to store process handles to actually kill them
            // This will be implemented when we refactor to store more PTY state
            warn!("PTY kill not yet fully implemented for panel: {}", panel_id);
        }
        
        Ok(())
    }

    /// Get status of a PTY process
    pub fn get_pty_status(&self, panel_id: &str) -> Option<PtyStatus> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.get(panel_id).map(|p| p.status.clone())
    }

    /// Clean up finished PTY processes
    pub fn cleanup_finished(&mut self) {
        let mut active_ptys = self.active_ptys.lock().unwrap();
        
        active_ptys.retain(|panel_id, pty_process| {
            match &pty_process.status {
                PtyStatus::Finished(_) | PtyStatus::Error(_) => {
                    debug!("Cleaning up finished PTY for panel: {}", panel_id);
                    false // Remove from map
                }
                _ => true // Keep running PTYs
            }
        });
    }

    /// Get list of active PTY panel IDs
    pub fn get_active_pty_panels(&self) -> Vec<String> {
        let active_ptys = self.active_ptys.lock().unwrap();
        active_ptys.keys().cloned().collect()
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new().expect("Failed to create PTY manager")
    }
}
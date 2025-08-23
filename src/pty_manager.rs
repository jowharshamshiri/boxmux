use anyhow::Result;
use log::{debug, error, warn};
use portable_pty::{CommandBuilder, PtySize, PtySystem};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct PtyProcess {
    pub panel_id: String,
    pub process_id: Option<u32>,
    pub status: PtyStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PtyStatus {
    Starting,
    Running,
    Finished(i32), // exit code
    Error(String),
}

pub struct PtyManager {
    active_ptys: Arc<Mutex<HashMap<String, PtyProcess>>>,
    pty_system: Box<dyn PtySystem>,
}

impl PtyManager {
    pub fn new() -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();
        
        Ok(PtyManager {
            active_ptys: Arc::new(Mutex::new(HashMap::new())),
            pty_system,
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
        debug!("Spawning PTY for panel: {}", panel_id);

        // Create PTY with appropriate size (will be resized when panel bounds are known)
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pty_pair = self.pty_system.openpty(pty_size)?;
        let reader = pty_pair.master;
        let _writer = pty_pair.slave;

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
        let mut child = _writer.spawn_command(cmd)?;
        let process_id = child.process_id();

        // Create PTY process record
        let pty_process = PtyProcess {
            panel_id: panel_id.clone(),
            process_id,
            status: PtyStatus::Running,
        };

        // Store in active PTYs
        {
            let mut active_ptys = self.active_ptys.lock().unwrap();
            active_ptys.insert(panel_id.clone(), pty_process);
        }

        // Spawn reader thread for PTY output
        let active_ptys_clone = self.active_ptys.clone();
        let panel_id_clone = panel_id.clone();
        
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            let mut accumulated_output = String::new();

            loop {
                match reader.try_clone_reader() {
                    Ok(mut pty_reader) => {
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

                                // Send any remaining output
                                if !accumulated_output.is_empty() {
                                    if let Err(e) = sender.send((thread_uuid, crate::thread_manager::Message::PanelOutputUpdate(
                                        panel_id_clone.clone(),
                                        true,
                                        accumulated_output.clone(),
                                    ))) {
                                        error!("Failed to send final PTY output: {}", e);
                                    }
                                }

                                // Send final message indicating completion
                                if let Err(e) = sender.send((thread_uuid, crate::thread_manager::Message::PanelOutputUpdate(
                                    panel_id_clone.clone(),
                                    exit_code == 0,
                                    format!("[Process exited with code {}]", exit_code),
                                ))) {
                                    error!("Failed to send PTY completion message: {}", e);
                                }
                                break;
                            }
                            Ok(bytes_read) => {
                                // Convert bytes to string
                                if let Ok(output) = String::from_utf8(buffer[..bytes_read].to_vec()) {
                                    accumulated_output.push_str(&output);
                                    
                                    // Send output line by line
                                    while let Some(newline_pos) = accumulated_output.find('\n') {
                                        let line = accumulated_output[..newline_pos].to_string();
                                        accumulated_output = accumulated_output[newline_pos + 1..].to_string();
                                        
                                        if !line.trim().is_empty() {
                                            if let Err(e) = sender.send((thread_uuid, crate::thread_manager::Message::PanelOutputUpdate(
                                                panel_id_clone.clone(),
                                                true,
                                                line,
                                            ))) {
                                                error!("Failed to send PTY output message: {}", e);
                                                break;
                                            }
                                        }
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
                    }
                    Err(e) => {
                        error!("Failed to clone PTY reader: {}", e);
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
    pub fn send_input(&mut self, panel_id: &str, _input: &str) -> Result<()> {
        // This would require storing the master PTY handle
        // For now, we'll implement this later when we add input routing
        warn!("PTY input not yet implemented for panel: {}", panel_id);
        Ok(())
    }

    /// Resize a PTY to match panel dimensions
    pub fn resize_pty(&mut self, panel_id: &str, rows: u16, cols: u16) -> Result<()> {
        debug!("Resizing PTY for panel {} to {}x{}", panel_id, cols, rows);
        
        // This would require storing PTY handles for resizing
        // For now, we'll implement this later
        warn!("PTY resize not yet implemented for panel: {}", panel_id);
        Ok(())
    }

    /// Kill a PTY process
    pub fn kill_pty(&mut self, panel_id: &str) -> Result<()> {
        let mut active_ptys = self.active_ptys.lock().unwrap();
        
        if let Some(pty_process) = active_ptys.get_mut(panel_id) {
            debug!("Killing PTY process for panel: {}", panel_id);
            
            // Mark as finished (killed)
            pty_process.status = PtyStatus::Finished(-9); // SIGKILL
            
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
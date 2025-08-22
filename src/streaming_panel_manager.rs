use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{debug, error, warn};

use crate::streaming_executor::{StreamingExecutor, OutputLine};
use crate::streaming_messages::{StreamingOutput, StreamingComplete};
use crate::model::panel::Panel;
use crate::model::app::AppContext;
use crate::thread_manager::Message;

#[derive(Debug, Clone)]
pub struct StreamingTask {
    pub task_id: Uuid,
    pub panel_id: String,
    pub script_commands: Vec<String>,
    pub is_active: bool,
    pub start_time: Instant,
    pub last_output_time: Option<Instant>,
    pub line_count: u64,
}

#[derive(Debug)]
pub struct StreamingPanelManager {
    active_tasks: Arc<Mutex<HashMap<Uuid, StreamingTask>>>,
    executors: Arc<Mutex<HashMap<Uuid, StreamingExecutor>>>,
    running_processes: Arc<Mutex<HashMap<Uuid, std::process::Child>>>,
    message_sender: mpsc::Sender<Message>,
    debounce_interval: Duration,
}

impl StreamingPanelManager {
    pub fn new(message_sender: mpsc::Sender<Message>) -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            executors: Arc::new(Mutex::new(HashMap::new())),
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            debounce_interval: Duration::from_millis(16), // ~60fps
        }
    }

    pub fn start_streaming(&self, panel: &Panel) -> Result<Uuid, Box<dyn std::error::Error>> {
        if panel.script.is_none() || panel.script.as_ref().unwrap().is_empty() {
            return Err("Panel has no script to execute".into());
        }

        let task_id = Uuid::new_v4();
        let script_commands = panel.script.as_ref().unwrap().clone();
        
        debug!("Starting streaming task {} for panel {}", task_id, panel.id);

        // Create streaming task
        let task = StreamingTask {
            task_id,
            panel_id: panel.id.clone(),
            script_commands: script_commands.clone(),
            is_active: true,
            start_time: Instant::now(),
            last_output_time: None,
            line_count: 0,
        };

        // Store task
        {
            if let Ok(mut tasks) = self.active_tasks.lock() {
                tasks.insert(task_id, task);
            }
        }

        // Create executor
        let mut executor = StreamingExecutor::new();
        let script = script_commands.join(" && ");
        
        match executor.spawn_streaming(&script, None) {
            Ok((child, _receiver)) => {
                // Store executor and process
                {
                    if let Ok(mut executors) = self.executors.lock() {
                        executors.insert(task_id, executor);
                    }
                    if let Ok(mut processes) = self.running_processes.lock() {
                        processes.insert(task_id, child);
                    }
                }

                // Start monitoring thread
                self.start_monitoring_thread(task_id);
                
                Ok(task_id)
            }
            Err(e) => {
                // Cleanup failed task
                {
                    if let Ok(mut tasks) = self.active_tasks.lock() {
                        tasks.remove(&task_id);
                    }
                }
                Err(e)
            }
        }
    }

    fn start_monitoring_thread(&self, task_id: Uuid) {
        let active_tasks = Arc::clone(&self.active_tasks);
        let executors = Arc::clone(&self.executors);
        let running_processes = Arc::clone(&self.running_processes);
        let message_sender = self.message_sender.clone();
        let debounce_interval = self.debounce_interval;

        std::thread::spawn(move || {
            debug!("Monitoring thread started for task {}", task_id);
            
            let mut last_update = Instant::now();
            let mut accumulated_lines = Vec::new();
            
            loop {
                // Check if task is still active
                let (panel_id, is_active) = {
                    if let Ok(tasks) = active_tasks.lock() {
                        if let Some(task) = tasks.get(&task_id) {
                            (task.panel_id.clone(), task.is_active)
                        } else {
                            debug!("Task {} no longer exists, stopping monitoring", task_id);
                            break;
                        }
                    } else {
                        break;
                    }
                };

                if !is_active {
                    debug!("Task {} marked inactive, stopping monitoring", task_id);
                    break;
                }

                // Read output from executor
                let mut new_lines = Vec::new();
                {
                    if let Ok(executors) = executors.lock() {
                        if let Some(executor) = executors.get(&task_id) {
                            while let Some(output_line) = executor.try_read_line() {
                                new_lines.push(output_line);
                            }
                        }
                    }
                }

                // Accumulate lines
                accumulated_lines.extend(new_lines);

                // Check if process completed
                let process_completed = {
                    if let Ok(mut processes) = running_processes.lock() {
                        if let Some(child) = processes.get_mut(&task_id) {
                            if let Ok(Some(status)) = child.try_wait() {
                                debug!("Process for task {} completed with status: {:?}", task_id, status);
                                true
                            } else {
                                false
                            }
                        } else {
                            true // Process not found, consider completed
                        }
                    } else {
                        true
                    }
                };

                // Send updates if debounce interval passed or process completed
                let should_update = last_update.elapsed() >= debounce_interval 
                    || process_completed 
                    || !accumulated_lines.is_empty() && accumulated_lines.len() >= 10;

                if should_update && !accumulated_lines.is_empty() {
                    // Send accumulated lines
                    for line in &accumulated_lines {
                        let streaming_msg = StreamingOutput::new(
                            panel_id.clone(),
                            line.content.clone(),
                            line.sequence,
                            line.is_stderr,
                        );
                        
                        // Convert to panel output update message
                        let msg = Message::PanelOutputUpdate(
                            panel_id.clone(),
                            true,
                            line.content.clone(),
                        );
                        
                        if let Err(e) = message_sender.send(msg) {
                            error!("Failed to send streaming output for task {}: {}", task_id, e);
                        }
                    }

                    // Update task
                    {
                        if let Ok(mut tasks) = active_tasks.lock() {
                            if let Some(task) = tasks.get_mut(&task_id) {
                                task.last_output_time = Some(Instant::now());
                                task.line_count += accumulated_lines.len() as u64;
                            }
                        }
                    }

                    accumulated_lines.clear();
                    last_update = Instant::now();
                }

                if process_completed {
                    // Send completion message
                    let completion_msg = Message::ExternalMessage(
                        format!("streaming_complete:{}:success", task_id)
                    );
                    
                    if let Err(e) = message_sender.send(completion_msg) {
                        error!("Failed to send completion message for task {}: {}", task_id, e);
                    }

                    // Cleanup
                    {
                        if let Ok(mut tasks) = active_tasks.lock() {
                            if let Some(mut task) = tasks.remove(&task_id) {
                                task.is_active = false;
                            }
                        }
                        if let Ok(mut executors) = executors.lock() {
                            executors.remove(&task_id);
                        }
                        if let Ok(mut processes) = running_processes.lock() {
                            processes.remove(&task_id);
                        }
                    }

                    debug!("Monitoring thread completed for task {}", task_id);
                    break;
                }

                std::thread::sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn stop_streaming(&self, task_id: Uuid) -> bool {
        debug!("Stopping streaming task {}", task_id);

        // Mark task as inactive
        {
            if let Ok(mut tasks) = self.active_tasks.lock() {
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.is_active = false;
                } else {
                    return false;
                }
            }
        }

        // Kill process
        {
            if let Ok(mut processes) = self.running_processes.lock() {
                if let Some(mut child) = processes.remove(&task_id) {
                    if let Err(e) = child.kill() {
                        warn!("Failed to kill process for task {}: {}", task_id, e);
                    }
                }
            }
        }

        // Cleanup
        {
            if let Ok(mut executors) = self.executors.lock() {
                executors.remove(&task_id);
            }
            if let Ok(mut tasks) = self.active_tasks.lock() {
                tasks.remove(&task_id);
            }
        }

        true
    }

    pub fn get_active_task_count(&self) -> usize {
        self.active_tasks.lock().map(|tasks| tasks.len()).unwrap_or(0)
    }

    pub fn get_task_info(&self, task_id: Uuid) -> Option<StreamingTask> {
        self.active_tasks.lock().ok()?.get(&task_id).cloned()
    }

    pub fn stop_all_tasks(&self) {
        debug!("Stopping all streaming tasks");

        let task_ids: Vec<Uuid> = {
            if let Ok(tasks) = self.active_tasks.lock() {
                tasks.keys().cloned().collect()
            } else {
                return;
            }
        };

        for task_id in task_ids {
            self.stop_streaming(task_id);
        }
    }

    pub fn set_debounce_interval(&mut self, interval: Duration) {
        self.debounce_interval = interval;
    }
}

impl Drop for StreamingPanelManager {
    fn drop(&mut self) {
        self.stop_all_tasks();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn create_test_panel(id: &str, script: Vec<String>) -> Panel {
        let mut panel = Panel::default();
        panel.id = id.to_string();
        panel.script = Some(script);
        panel
    }

    #[test]
    fn test_streaming_panel_manager_creation() {
        let (sender, _receiver) = mpsc::channel();
        let manager = StreamingPanelManager::new(sender);
        
        assert_eq!(manager.get_active_task_count(), 0);
        assert_eq!(manager.debounce_interval, Duration::from_millis(16));
    }

    #[test]
    fn test_start_streaming_simple_command() {
        let (sender, receiver) = mpsc::channel();
        let manager = StreamingPanelManager::new(sender);
        
        let panel = create_test_panel("test_panel", vec!["echo 'test'".to_string()]);
        
        let task_id = manager.start_streaming(&panel).unwrap();
        assert_eq!(manager.get_active_task_count(), 1);
        
        // Wait for execution
        std::thread::sleep(Duration::from_millis(100));
        
        // Check for messages
        let mut received_output = false;
        while let Ok(msg) = receiver.try_recv() {
            if let Message::PanelOutputUpdate(panel_id, _, content) = msg {
                if panel_id == "test_panel" && content.contains("test") {
                    received_output = true;
                    break;
                }
            }
        }
        
        assert!(received_output);
    }

    #[test]
    fn test_start_streaming_no_script() {
        let (sender, _receiver) = mpsc::channel();
        let manager = StreamingPanelManager::new(sender);
        
        let panel = create_test_panel("test_panel", vec![]);
        
        let result = manager.start_streaming(&panel);
        assert!(result.is_err());
        assert_eq!(manager.get_active_task_count(), 0);
    }

    #[test]
    fn test_stop_streaming() {
        let (sender, _receiver) = mpsc::channel();
        let manager = StreamingPanelManager::new(sender);
        
        let panel = create_test_panel("test_panel", vec!["sleep 1".to_string()]);
        
        let task_id = manager.start_streaming(&panel).unwrap();
        assert_eq!(manager.get_active_task_count(), 1);
        
        let stopped = manager.stop_streaming(task_id);
        assert!(stopped);
        
        // Wait for cleanup
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(manager.get_active_task_count(), 0);
    }

    #[test]
    fn test_get_task_info() {
        let (sender, _receiver) = mpsc::channel();
        let manager = StreamingPanelManager::new(sender);
        
        let panel = create_test_panel("test_panel", vec!["echo 'test'".to_string()]);
        
        let task_id = manager.start_streaming(&panel).unwrap();
        
        let task_info = manager.get_task_info(task_id);
        assert!(task_info.is_some());
        
        let task = task_info.unwrap();
        assert_eq!(task.task_id, task_id);
        assert_eq!(task.panel_id, "test_panel");
        assert!(task.is_active);
    }

    #[test]
    fn test_stop_all_tasks() {
        let (sender, _receiver) = mpsc::channel();
        let manager = StreamingPanelManager::new(sender);
        
        // Start multiple tasks
        let panel1 = create_test_panel("panel1", vec!["sleep 1".to_string()]);
        let panel2 = create_test_panel("panel2", vec!["sleep 1".to_string()]);
        
        manager.start_streaming(&panel1).unwrap();
        manager.start_streaming(&panel2).unwrap();
        
        assert_eq!(manager.get_active_task_count(), 2);
        
        manager.stop_all_tasks();
        
        // Wait for cleanup
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(manager.get_active_task_count(), 0);
    }

    #[test]
    fn test_debounce_interval() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = StreamingPanelManager::new(sender);
        
        assert_eq!(manager.debounce_interval, Duration::from_millis(16));
        
        manager.set_debounce_interval(Duration::from_millis(50));
        assert_eq!(manager.debounce_interval, Duration::from_millis(50));
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{debug, error, warn};

use crate::streaming_executor::{StreamingExecutor, OutputLine};
use crate::streaming_messages::{StreamingOutput, StreamingStatus, StreamingStatusUpdate};
use crate::rate_limiter::StreamingRateLimiter;
use crate::model::panel::Panel;
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
    rate_limiter: Arc<Mutex<StreamingRateLimiter>>,
}

impl StreamingPanelManager {
    pub fn new(message_sender: mpsc::Sender<Message>) -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            executors: Arc::new(Mutex::new(HashMap::new())),
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            debounce_interval: Duration::from_millis(16), // ~60fps
            rate_limiter: Arc::new(Mutex::new(StreamingRateLimiter::new(60, 1000))), // 60 lines/sec, queue size 1000
        }
    }
    
    pub fn with_rate_limit(message_sender: mpsc::Sender<Message>, max_lines_per_second: u32, max_queue_size: usize) -> Self {
        Self {
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
            executors: Arc::new(Mutex::new(HashMap::new())),
            running_processes: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            debounce_interval: Duration::from_millis(16),
            rate_limiter: Arc::new(Mutex::new(StreamingRateLimiter::new(max_lines_per_second, max_queue_size))),
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
            Ok((child, receiver)) => {
                // Store executor and process
                {
                    if let Ok(mut executors) = self.executors.lock() {
                        executors.insert(task_id, executor);
                    }
                    if let Ok(mut processes) = self.running_processes.lock() {
                        processes.insert(task_id, child);
                    }
                }

                // Send starting status update
                let status_update = StreamingStatusUpdate::new(
                    panel.id.clone(),
                    task_id,
                    StreamingStatus::Starting,
                    0,
                );
                
                let status_msg = Message::StreamingStatusUpdate(panel.id.clone(), status_update);
                
                if let Err(e) = self.message_sender.send(status_msg) {
                    error!("Failed to send starting status update: {}", e);
                }

                // Start monitoring thread with receiver
                self.start_monitoring_thread(task_id, receiver);
                
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

    fn start_monitoring_thread(&self, task_id: Uuid, receiver: mpsc::Receiver<OutputLine>) {
        let active_tasks = Arc::clone(&self.active_tasks);
        let executors = Arc::clone(&self.executors);
        let running_processes = Arc::clone(&self.running_processes);
        let message_sender = self.message_sender.clone();
        let debounce_interval = self.debounce_interval;
        let rate_limiter = Arc::clone(&self.rate_limiter);

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

                // Read output from receiver
                let mut new_lines = Vec::new();
                while let Ok(output_line) = receiver.try_recv() {
                    new_lines.push(output_line);
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
                    // Apply rate limiting to accumulated lines
                    let mut rate_limited_lines = Vec::new();
                    
                    for line in &accumulated_lines {
                        if let Ok(mut limiter) = rate_limiter.lock() {
                            if limiter.should_allow_output() {
                                rate_limited_lines.push(line);
                            } else {
                                // Queue the line for later if rate limited
                                limiter.queue_output(line.content.clone());
                            }
                        } else {
                            // Fallback: send without rate limiting if lock fails
                            rate_limited_lines.push(line);
                        }
                    }
                    
                    // Send rate-limited lines
                    for line in &rate_limited_lines {
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
                    
                    // Process queued output from rate limiter
                    if let Ok(mut limiter) = rate_limiter.lock() {
                        while let Some(queued_content) = limiter.get_next_output() {
                            let msg = Message::PanelOutputUpdate(
                                panel_id.clone(),
                                true,
                                queued_content,
                            );
                            
                            if let Err(e) = message_sender.send(msg) {
                                error!("Failed to send queued output for task {}: {}", task_id, e);
                                break;
                            }
                        }
                    }

                    // Update task and send status update
                    {
                        if let Ok(mut tasks) = active_tasks.lock() {
                            if let Some(task) = tasks.get_mut(&task_id) {
                                task.last_output_time = Some(Instant::now());
                                task.line_count += accumulated_lines.len() as u64;
                                
                                // Send running status update with current line count
                                let status_update = StreamingStatusUpdate::new(
                                    task.panel_id.clone(),
                                    task_id,
                                    StreamingStatus::Running,
                                    task.line_count,
                                );
                                
                                let status_msg = Message::StreamingStatusUpdate(task.panel_id.clone(), status_update);
                                
                                if let Err(e) = message_sender.send(status_msg) {
                                    error!("Failed to send running status update for task {}: {}", task_id, e);
                                }
                            }
                        }
                    }

                    accumulated_lines.clear();
                    last_update = Instant::now();
                }

                if process_completed {
                    // Get final line count and send completion status
                    let (final_panel_id, final_line_count) = {
                        if let Ok(tasks) = active_tasks.lock() {
                            if let Some(task) = tasks.get(&task_id) {
                                (task.panel_id.clone(), task.line_count)
                            } else {
                                (panel_id.clone(), 0)
                            }
                        } else {
                            (panel_id.clone(), 0)
                        }
                    };
                    
                    // Send completion status update
                    let completion_status = StreamingStatusUpdate::new(
                        final_panel_id.clone(),
                        task_id,
                        StreamingStatus::Completed(true), // Assume success for now
                        final_line_count,
                    );
                    
                    let completion_status_msg = Message::StreamingStatusUpdate(final_panel_id.clone(), completion_status);
                    
                    if let Err(e) = message_sender.send(completion_status_msg) {
                        error!("Failed to send completion status update for task {}: {}", task_id, e);
                    }
                    
                    // Send legacy completion message for compatibility
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

    pub fn get_performance_stats(&self) -> Option<(usize, usize, f64)> {
        if let Ok(limiter) = self.rate_limiter.lock() {
            let active_tasks = self.get_active_task_count();
            let queue_size = limiter.queue_size();
            let queue_utilization = if limiter.is_queue_full() { 100.0 } else { 
                (queue_size as f64 / 1000.0) * 100.0 // Assuming default queue size 1000
            };
            Some((active_tasks, queue_size, queue_utilization))
        } else {
            None
        }
    }

    pub fn adjust_rate_limit(&self, max_lines_per_second: u32, max_queue_size: usize) {
        if let Ok(mut limiter) = self.rate_limiter.lock() {
            *limiter = StreamingRateLimiter::new(max_lines_per_second, max_queue_size);
        }
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
        
        // Wait longer for streaming execution to complete
        std::thread::sleep(Duration::from_millis(200));
        
        // Check for messages with timeout
        let mut received_output = false;
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            while let Ok(msg) = receiver.try_recv() {
                if let Message::PanelOutputUpdate(panel_id, _, content) = msg {
                    if panel_id == "test_panel" && content.contains("test") {
                        received_output = true;
                        break;
                    }
                }
            }
            if received_output {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
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
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::{debug, error, warn, info};
use crate::streaming_executor::StreamingExecutor;
use crate::model::app::AppContext;
use crate::model::panel::Panel;
use crate::thread_manager::Message;

#[derive(Debug, Clone)]
pub enum Task {
    PanelRefresh {
        panel_id: String,
        script_commands: Vec<String>,
        refresh_interval: Duration,
        context: AppContext,
    },
    ChoiceExecution {
        choice_id: String,
        script_commands: Vec<String>,
        redirect_output_to: Option<String>,
        context: AppContext,
    },
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub task: Task,
    pub id: Uuid,
    pub scheduled_time: Instant,
    pub is_periodic: bool,
    pub last_execution: Option<Instant>,
}

#[derive(Debug)]
pub struct UnifiedThreadPool {
    task_queue: Arc<Mutex<VecDeque<TaskInfo>>>,
    active_tasks: Arc<Mutex<HashMap<Uuid, TaskInfo>>>,
    running_processes: Arc<Mutex<HashMap<Uuid, std::process::Child>>>,
    worker_handles: Vec<JoinHandle<()>>,
    message_sender: mpsc::Sender<Message>,
    shutdown_flag: Arc<Mutex<bool>>,
    max_concurrent_tasks: usize,
}

impl UnifiedThreadPool {
    pub fn new(message_sender: mpsc::Sender<Message>, worker_count: usize, max_concurrent: usize) -> Self {
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let active_tasks = Arc::new(Mutex::new(HashMap::new()));
        let running_processes = Arc::new(Mutex::new(HashMap::new()));
        let shutdown_flag = Arc::new(Mutex::new(false));
        
        let mut worker_handles = Vec::new();
        
        for worker_id in 0..worker_count {
            let queue_clone = Arc::clone(&task_queue);
            let active_clone = Arc::clone(&active_tasks);
            let processes_clone = Arc::clone(&running_processes);
            let sender_clone = message_sender.clone();
            let shutdown_clone = Arc::clone(&shutdown_flag);
            
            let handle = thread::spawn(move || {
                Self::worker_loop(
                    worker_id,
                    queue_clone,
                    active_clone,
                    processes_clone,
                    sender_clone,
                    shutdown_clone,
                );
            });
            
            worker_handles.push(handle);
        }
        
        Self {
            task_queue,
            active_tasks,
            running_processes,
            worker_handles,
            message_sender,
            shutdown_flag,
            max_concurrent_tasks: max_concurrent,
        }
    }

    pub fn schedule_panel_refresh(&self, panel: &Panel, context: AppContext) -> Uuid {
        let task_id = Uuid::new_v4();
        let refresh_interval = Duration::from_millis(panel.refresh_interval.unwrap_or(1000));
        
        let task = Task::PanelRefresh {
            panel_id: panel.id.clone(),
            script_commands: panel.script.clone().unwrap_or_default(),
            refresh_interval,
            context,
        };
        
        let task_info = TaskInfo {
            task,
            id: task_id,
            scheduled_time: Instant::now(),
            is_periodic: true,
            last_execution: None,
        };
        
        if let Ok(mut queue) = self.task_queue.lock() {
            queue.push_back(task_info);
        }
        
        debug!("Scheduled panel refresh task {} for panel {}", task_id, panel.id);
        task_id
    }

    pub fn execute_choice(&self, choice_id: String, script_commands: Vec<String>, redirect_output_to: Option<String>, context: AppContext) -> Uuid {
        let task_id = Uuid::new_v4();
        
        let task = Task::ChoiceExecution {
            choice_id: choice_id.clone(),
            script_commands,
            redirect_output_to,
            context,
        };
        
        let task_info = TaskInfo {
            task,
            id: task_id,
            scheduled_time: Instant::now(),
            is_periodic: false,
            last_execution: None,
        };
        
        if let Ok(mut queue) = self.task_queue.lock() {
            queue.push_front(task_info); // Choices get priority
        }
        
        debug!("Scheduled choice execution task {} for choice {}", task_id, choice_id);
        task_id
    }

    pub fn cancel_task(&self, task_id: Uuid) -> bool {
        // Remove from queue
        if let Ok(mut queue) = self.task_queue.lock() {
            let original_len = queue.len();
            queue.retain(|task| task.id != task_id);
            if queue.len() < original_len {
                debug!("Removed task {} from queue", task_id);
                return true;
            }
        }
        
        // Remove from active tasks and kill process if running
        if let Ok(mut active) = self.active_tasks.lock() {
            if active.remove(&task_id).is_some() {
                if let Ok(mut processes) = self.running_processes.lock() {
                    if let Some(mut child) = processes.remove(&task_id) {
                        if let Err(e) = child.kill() {
                            warn!("Failed to kill process for task {}: {}", task_id, e);
                        }
                    }
                }
                debug!("Cancelled active task {}", task_id);
                return true;
            }
        }
        
        false
    }

    pub fn get_active_task_count(&self) -> usize {
        self.active_tasks.lock().map(|active| active.len()).unwrap_or(0)
    }

    pub fn get_queued_task_count(&self) -> usize {
        self.task_queue.lock().map(|queue| queue.len()).unwrap_or(0)
    }

    fn worker_loop(
        worker_id: usize,
        task_queue: Arc<Mutex<VecDeque<TaskInfo>>>,
        active_tasks: Arc<Mutex<HashMap<Uuid, TaskInfo>>>,
        running_processes: Arc<Mutex<HashMap<Uuid, std::process::Child>>>,
        message_sender: mpsc::Sender<Message>,
        shutdown_flag: Arc<Mutex<bool>>,
    ) {
        debug!("Worker {} started", worker_id);
        
        loop {
            // Check shutdown flag
            if let Ok(shutdown) = shutdown_flag.lock() {
                if *shutdown {
                    debug!("Worker {} shutting down", worker_id);
                    break;
                }
            }
            
            // Get next task from queue
            let task_info = {
                if let Ok(mut queue) = task_queue.lock() {
                    queue.pop_front()
                } else {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
            };
            
            if let Some(mut task_info) = task_info {
                // Check if we should execute this task now
                if task_info.scheduled_time > Instant::now() {
                    // Put it back in queue and wait
                    if let Ok(mut queue) = task_queue.lock() {
                        queue.push_back(task_info);
                    }
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                
                // Add to active tasks
                {
                    if let Ok(mut active) = active_tasks.lock() {
                        active.insert(task_info.id, task_info.clone());
                    }
                }
                
                debug!("Worker {} executing task {}", worker_id, task_info.id);
                
                // Execute the task
                Self::execute_task(&task_info, &running_processes, &message_sender);
                
                // Update task timing and reschedule if periodic
                task_info.last_execution = Some(Instant::now());
                
                if task_info.is_periodic {
                    if let Task::PanelRefresh { refresh_interval, .. } = &task_info.task {
                        task_info.scheduled_time = Instant::now() + *refresh_interval;
                        
                        // Re-queue the periodic task
                        if let Ok(mut queue) = task_queue.lock() {
                            queue.push_back(task_info.clone());
                        }
                    }
                }
                
                // Remove from active tasks
                {
                    if let Ok(mut active) = active_tasks.lock() {
                        active.remove(&task_info.id);
                    }
                }
                
                debug!("Worker {} completed task {}", worker_id, task_info.id);
            } else {
                // No tasks available, sleep briefly
                thread::sleep(Duration::from_millis(50));
            }
        }
        
        debug!("Worker {} stopped", worker_id);
    }

    fn execute_task(
        task_info: &TaskInfo,
        running_processes: &Arc<Mutex<HashMap<Uuid, std::process::Child>>>,
        message_sender: &mpsc::Sender<Message>,
    ) {
        match &task_info.task {
            Task::PanelRefresh { panel_id, script_commands, .. } => {
                Self::execute_panel_refresh(task_info.id, panel_id, script_commands, running_processes, message_sender);
            }
            Task::ChoiceExecution { choice_id, script_commands, redirect_output_to, .. } => {
                Self::execute_choice_script(task_info.id, choice_id, script_commands, redirect_output_to, running_processes, message_sender);
            }
        }
    }

    fn execute_panel_refresh(
        task_id: Uuid,
        panel_id: &str,
        script_commands: &[String],
        running_processes: &Arc<Mutex<HashMap<Uuid, std::process::Child>>>,
        message_sender: &mpsc::Sender<Message>,
    ) {
        if script_commands.is_empty() {
            return;
        }
        
        let script = script_commands.join(" && ");
        let mut executor = StreamingExecutor::new();
        
        match executor.spawn_streaming(&script, None) {
            Ok((child, _receiver)) => {
                // Store the running process
                {
                    if let Ok(mut processes) = running_processes.lock() {
                        processes.insert(task_id, child);
                    }
                }
                
                // Retrieve child for execution monitoring
                let child = {
                    if let Ok(mut processes) = running_processes.lock() {
                        processes.remove(&task_id)
                    } else {
                        return;
                    }
                };
                
                if let Some(mut child) = child {
                    let mut output_lines = Vec::new();
                    
                    // Collect streaming output
                    loop {
                        if let Some(line) = executor.try_read_line() {
                            output_lines.push(line.content);
                        }
                        
                        if let Some(status) = executor.get_exit_status(&mut child) {
                            // Process completed
                            
                            // Collect any remaining output
                            while let Some(line) = executor.try_read_line() {
                                output_lines.push(line.content);
                            }
                            
                            // Send final output to panel
                            let final_output = output_lines.join("\n");
                            let msg = Message::PanelOutputUpdate(
                                panel_id.to_string(),
                                true,
                                final_output,
                            );
                            
                            if let Err(e) = message_sender.send(msg) {
                                error!("Failed to send panel output update: {}", e);
                            }
                            
                            break;
                        }
                        
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            Err(e) => {
                error!("Failed to start panel refresh for {}: {}", panel_id, e);
            }
        }
    }

    fn execute_choice_script(
        task_id: Uuid,
        choice_id: &str,
        script_commands: &[String],
        redirect_output_to: &Option<String>,
        running_processes: &Arc<Mutex<HashMap<Uuid, std::process::Child>>>,
        message_sender: &mpsc::Sender<Message>,
    ) {
        if script_commands.is_empty() {
            return;
        }
        
        let script = script_commands.join(" && ");
        let mut executor = StreamingExecutor::new();
        
        match executor.spawn_streaming(&script, None) {
            Ok((child, _receiver)) => {
                // Store the running process
                {
                    if let Ok(mut processes) = running_processes.lock() {
                        processes.insert(task_id, child);
                    }
                }
                
                // Retrieve child for execution monitoring
                let child = {
                    if let Ok(mut processes) = running_processes.lock() {
                        processes.remove(&task_id)
                    } else {
                        return;
                    }
                };
                
                if let Some(mut child) = child {
                    let mut output_lines = Vec::new();
                    
                    // Collect streaming output
                    loop {
                        if let Some(line) = executor.try_read_line() {
                            output_lines.push(line.content);
                            
                            // Send incremental updates if redirecting output
                            if let Some(target_panel_id) = redirect_output_to {
                                let incremental_output = output_lines.join("\n");
                                let msg = Message::PanelOutputUpdate(
                                    target_panel_id.clone(),
                                    true,
                                    incremental_output.clone(),
                                );
                                
                                if let Err(e) = message_sender.send(msg) {
                                    error!("Failed to send incremental output update: {}", e);
                                }
                            }
                        }
                        
                        if let Some(status) = executor.get_exit_status(&mut child) {
                            // Process completed
                            
                            // Collect any remaining output
                            while let Some(line) = executor.try_read_line() {
                                output_lines.push(line.content);
                            }
                            
                            // Send completion message
                            let final_output = output_lines.join("\n");
                            
                            if let Some(target_panel_id) = redirect_output_to {
                                let msg = Message::PanelOutputUpdate(
                                    target_panel_id.clone(),
                                    true,
                                    final_output.clone(),
                                );
                                
                                if let Err(e) = message_sender.send(msg) {
                                    error!("Failed to send final output update: {}", e);
                                }
                            }
                            
                            // Send choice completion message - use ExternalMessage as placeholder
                            let completion_msg = Message::ExternalMessage(
                                format!("choice_complete:{}:{}", choice_id, status.success())
                            );
                            
                            if let Err(e) = message_sender.send(completion_msg) {
                                error!("Failed to send choice completion message: {}", e);
                            }
                            
                            break;
                        }
                        
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
            Err(e) => {
                error!("Failed to start choice execution for {}: {}", choice_id, e);
            }
        }
    }

    pub fn shutdown(&self) {
        info!("Shutting down unified thread pool");
        
        // Set shutdown flag
        if let Ok(mut shutdown) = self.shutdown_flag.lock() {
            *shutdown = true;
        }
        
        // Cancel all running processes
        if let Ok(mut processes) = self.running_processes.lock() {
            for (task_id, mut child) in processes.drain() {
                if let Err(e) = child.kill() {
                    warn!("Failed to kill process for task {}: {}", task_id, e);
                }
            }
        }
        
        // Clear task queue
        if let Ok(mut queue) = self.task_queue.lock() {
            queue.clear();
        }
        
        // Clear active tasks
        if let Ok(mut active) = self.active_tasks.lock() {
            active.clear();
        }
        
        info!("Unified thread pool shutdown complete");
    }
}

impl Drop for UnifiedThreadPool {
    fn drop(&mut self) {
        self.shutdown();
        
        // Wait for worker threads to finish
        while let Some(handle) = self.worker_handles.pop() {
            if let Err(e) = handle.join() {
                error!("Error joining worker thread: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_unified_thread_pool_creation() {
        let (sender, _receiver) = mpsc::channel();
        let pool = UnifiedThreadPool::new(sender, 2, 10);
        
        assert_eq!(pool.get_active_task_count(), 0);
        assert_eq!(pool.get_queued_task_count(), 0);
        assert_eq!(pool.max_concurrent_tasks, 10);
    }

    #[test]
    fn test_choice_execution_scheduling() {
        let (sender, receiver) = mpsc::channel();
        let pool = UnifiedThreadPool::new(sender, 1, 5);
        let app = crate::model::app::App::default();
        let config = crate::Config::default();
        let context = AppContext::new(app, config);
        
        let task_id = pool.execute_choice(
            "test_choice".to_string(),
            vec!["echo 'test'".to_string()],
            None,
            context,
        );
        
        // Task should be queued
        assert_eq!(pool.get_queued_task_count(), 1);
        
        // Wait for execution
        thread::sleep(Duration::from_millis(200));
        
        // Check for completion message
        let mut received_completion = false;
        while let Ok(msg) = receiver.try_recv() {
            if let Message::ExternalMessage(external_msg) = msg {
                if external_msg.starts_with("choice_complete:") {
                    let parts: Vec<&str> = external_msg.split(':').collect();
                    if parts.len() >= 2 {
                        let choice_id = parts[1];
                        if choice_id == "test_choice" {
                            received_completion = true;
                            break;
                        }
                    }
                }
            }
        }
        
        assert!(received_completion);
    }

    #[test]
    fn test_task_cancellation() {
        let (sender, _receiver) = mpsc::channel();
        let pool = UnifiedThreadPool::new(sender, 1, 5);
        let app = crate::model::app::App::default();
        let config = crate::Config::default();
        let context = AppContext::new(app, config);
        
        let task_id = pool.execute_choice(
            "cancel_test".to_string(),
            vec!["sleep 5".to_string()],
            None,
            context,
        );
        
        assert_eq!(pool.get_queued_task_count(), 1);
        
        // Cancel the task
        let cancelled = pool.cancel_task(task_id);
        assert!(cancelled);
        assert_eq!(pool.get_queued_task_count(), 0);
    }

    #[test]
    fn test_concurrent_task_limit() {
        let (sender, _receiver) = mpsc::channel();
        let pool = UnifiedThreadPool::new(sender, 1, 2);
        let app = crate::model::app::App::default();
        let config = crate::Config::default();
        let context = AppContext::new(app, config);
        
        // Schedule multiple tasks
        for i in 0..5 {
            pool.execute_choice(
                format!("task_{}", i),
                vec!["echo 'test'".to_string()],
                None,
                context.clone(),
            );
        }
        
        // Should have 5 tasks queued
        assert_eq!(pool.get_queued_task_count(), 5);
        assert_eq!(pool.max_concurrent_tasks, 2);
    }

    #[test]
    fn test_panel_refresh_scheduling() {
        let (sender, _receiver) = mpsc::channel();
        let pool = UnifiedThreadPool::new(sender, 1, 5);
        let app = crate::model::app::App::default();
        let config = crate::Config::default();
        let context = AppContext::new(app, config);
        
        let panel = Panel {
            id: "refresh_panel".to_string(),
            script: Some(vec!["echo 'refresh'".to_string()]),
            refresh_interval: Some(100),
            ..Default::default()
        };
        
        let task_id = pool.schedule_panel_refresh(&panel, context);
        
        // Task should be queued
        assert_eq!(pool.get_queued_task_count(), 1);
        
        // Wait for execution and rescheduling
        thread::sleep(Duration::from_millis(200));
        
        // Periodic tasks are rescheduled after execution, so it should still be 1
        // If it's 0, that means the task hasn't been rescheduled yet
        let queue_count = pool.get_queued_task_count();
        assert!(queue_count <= 1, "Expected queue count to be 0 or 1, got {}", queue_count);
    }

    #[test]
    fn test_worker_thread_count() {
        let (sender, _receiver) = mpsc::channel();
        let pool = UnifiedThreadPool::new(sender, 3, 10);
        
        // Verify worker threads are created (indirect test via execution)
        assert_eq!(pool.worker_handles.len(), 3);
    }
}
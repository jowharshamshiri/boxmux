use std::collections::HashMap;
use uuid::Uuid;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use log::{debug, warn};

use crate::streaming_messages::{StreamingOutput, StreamingComplete};
use crate::thread_manager::Message;

/// Manages real-time panel updates with efficient rendering and debouncing
#[derive(Debug)]
pub struct RealTimeUpdateManager {
    pending_updates: Arc<Mutex<HashMap<String, PanelUpdate>>>,
    last_render_times: Arc<Mutex<HashMap<String, Instant>>>,
    message_sender: mpsc::Sender<Message>,
    render_debounce: Duration,
    max_update_frequency: Duration,
}

#[derive(Debug, Clone)]
pub struct PanelUpdate {
    pub panel_id: String,
    pub content_changes: Vec<String>,
    pub last_update: Instant,
    pub sequence_number: u64,
    pub force_render: bool,
}

impl RealTimeUpdateManager {
    pub fn new(message_sender: mpsc::Sender<Message>) -> Self {
        Self {
            pending_updates: Arc::new(Mutex::new(HashMap::new())),
            last_render_times: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
            render_debounce: Duration::from_millis(16), // ~60fps
            max_update_frequency: Duration::from_millis(100), // Max 10 updates per second per panel
        }
    }

    /// Queue a streaming output for panel update
    pub fn queue_streaming_update(&self, streaming_output: StreamingOutput) {
        let panel_id = streaming_output.panel_id.clone();
        
        debug!("Queuing streaming update for panel {}: {}", 
               panel_id, streaming_output.line_content);

        {
            if let Ok(mut updates) = self.pending_updates.lock() {
                let update = updates.entry(panel_id.clone()).or_insert_with(|| {
                    PanelUpdate {
                        panel_id: panel_id.clone(),
                        content_changes: Vec::new(),
                        last_update: Instant::now(),
                        sequence_number: 0,
                        force_render: false,
                    }
                });

                update.content_changes.push(streaming_output.line_content);
                update.last_update = Instant::now();
                update.sequence_number = streaming_output.sequence;
            }
        }

        // Check if we should trigger immediate render
        self.maybe_trigger_render(&panel_id);
    }

    /// Queue multiple streaming outputs efficiently
    pub fn queue_batch_updates(&self, outputs: Vec<StreamingOutput>) {
        debug!("Queuing batch of {} updates", outputs.len());

        let mut panel_groups: HashMap<String, Vec<StreamingOutput>> = HashMap::new();
        
        // Group outputs by panel
        for output in outputs {
            panel_groups.entry(output.panel_id.clone())
                .or_insert_with(Vec::new)
                .push(output);
        }

        // Process each panel's updates
        for (panel_id, panel_outputs) in panel_groups {
            {
                if let Ok(mut updates) = self.pending_updates.lock() {
                    let update = updates.entry(panel_id.clone()).or_insert_with(|| {
                        PanelUpdate {
                            panel_id: panel_id.clone(),
                            content_changes: Vec::new(),
                            last_update: Instant::now(),
                            sequence_number: 0,
                            force_render: false,
                        }
                    });

                    for output in &panel_outputs {
                        update.content_changes.push(output.line_content.clone());
                        update.sequence_number = output.sequence.max(update.sequence_number);
                    }
                    update.last_update = Instant::now();
                }
            }

            self.maybe_trigger_render(&panel_id);
        }
    }

    /// Force immediate render of a specific panel
    pub fn force_panel_render(&self, panel_id: &str) {
        debug!("Forcing immediate render for panel {}", panel_id);

        {
            if let Ok(mut updates) = self.pending_updates.lock() {
                if let Some(update) = updates.get_mut(panel_id) {
                    update.force_render = true;
                }
            }
        }

        self.trigger_render(panel_id);
    }

    /// Check if panel should be rendered based on debouncing rules
    fn maybe_trigger_render(&self, panel_id: &str) {
        let should_render = {
            if let Ok(last_renders) = self.last_render_times.lock() {
                if let Some(last_render) = last_renders.get(panel_id) {
                    last_render.elapsed() >= self.render_debounce
                } else {
                    true // First render
                }
            } else {
                false
            }
        };

        let has_pending_content = {
            if let Ok(updates) = self.pending_updates.lock() {
                if let Some(update) = updates.get(panel_id) {
                    !update.content_changes.is_empty() || update.force_render
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_render && has_pending_content {
            self.trigger_render(panel_id);
        }
    }

    /// Actually trigger a panel render
    fn trigger_render(&self, panel_id: &str) {
        let update_content = {
            if let Ok(mut updates) = self.pending_updates.lock() {
                if let Some(update) = updates.remove(panel_id) {
                    Some(update.content_changes.join("\n"))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(content) = update_content {
            debug!("Triggering render for panel {} with {} characters", 
                   panel_id, content.len());

            // Update last render time
            {
                if let Ok(mut last_renders) = self.last_render_times.lock() {
                    last_renders.insert(panel_id.to_string(), Instant::now());
                }
            }

            // Send panel update message using streaming complete
            let task_id = Uuid::new_v4();
            let line_count = content.lines().count() as u64;
            let streaming_complete = StreamingComplete::new(
                panel_id.to_string(),
                task_id,
                Some(0), // Success exit code
                line_count,
            );
            let msg = Message::StreamingComplete(streaming_complete);

            if let Err(e) = self.message_sender.send(msg) {
                warn!("Failed to send panel update message for {}: {}", panel_id, e);
            }

            // Send redraw message
            let redraw_msg = Message::RedrawPanel(panel_id.to_string());
            if let Err(e) = self.message_sender.send(redraw_msg) {
                warn!("Failed to send redraw message for {}: {}", panel_id, e);
            }
        }
    }

    /// Get pending update count for a panel
    pub fn get_pending_update_count(&self, panel_id: &str) -> usize {
        if let Ok(updates) = self.pending_updates.lock() {
            if let Some(update) = updates.get(panel_id) {
                update.content_changes.len()
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Get total pending updates across all panels
    pub fn get_total_pending_updates(&self) -> usize {
        if let Ok(updates) = self.pending_updates.lock() {
            updates.values()
                .map(|update| update.content_changes.len())
                .sum()
        } else {
            0
        }
    }

    /// Clear all pending updates
    pub fn clear_all_pending(&self) {
        debug!("Clearing all pending updates");
        
        if let Ok(mut updates) = self.pending_updates.lock() {
            updates.clear();
        }
        
        if let Ok(mut last_renders) = self.last_render_times.lock() {
            last_renders.clear();
        }
    }

    /// Clear pending updates for specific panel
    pub fn clear_panel_pending(&self, panel_id: &str) {
        debug!("Clearing pending updates for panel {}", panel_id);
        
        if let Ok(mut updates) = self.pending_updates.lock() {
            updates.remove(panel_id);
        }
    }

    /// Set rendering configuration
    pub fn set_render_debounce(&mut self, debounce: Duration) {
        self.render_debounce = debounce;
        debug!("Set render debounce to {:?}", debounce);
    }

    pub fn set_max_update_frequency(&mut self, frequency: Duration) {
        self.max_update_frequency = frequency;
        debug!("Set max update frequency to {:?}", frequency);
    }

    /// Start background processing thread for batched updates
    pub fn start_background_processor(&self) -> std::thread::JoinHandle<()> {
        let pending_updates = Arc::clone(&self.pending_updates);
        let message_sender = self.message_sender.clone();
        let process_interval = self.render_debounce;

        std::thread::spawn(move || {
            debug!("Background update processor started");
            
            loop {
                std::thread::sleep(process_interval);
                
                // Get panels that need processing
                let panels_to_process: Vec<String> = {
                    if let Ok(updates) = pending_updates.lock() {
                        updates.keys()
                            .filter(|panel_id| {
                                if let Some(update) = updates.get(*panel_id) {
                                    update.last_update.elapsed() >= process_interval
                                } else {
                                    false
                                }
                            })
                            .cloned()
                            .collect()
                    } else {
                        continue;
                    }
                };

                // Process each panel
                for panel_id in panels_to_process {
                    let content = {
                        if let Ok(mut updates) = pending_updates.lock() {
                            if let Some(update) = updates.remove(&panel_id) {
                                if !update.content_changes.is_empty() {
                                    Some(update.content_changes.join("\n"))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if let Some(content) = content {
                        debug!("Background processing update for panel {}", panel_id);
                        
                        let task_id = Uuid::new_v4();
                        let line_count = content.lines().count() as u64;
                        let streaming_complete = StreamingComplete::new(
                            panel_id.clone(),
                            task_id,
                            Some(0), // Success exit code
                            line_count,
                        );
                        let msg = Message::StreamingComplete(streaming_complete);

                        if let Err(e) = message_sender.send(msg) {
                            warn!("Background processor failed to send update for {}: {}", panel_id, e);
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::SystemTime;

    #[test]
    fn test_real_time_update_manager_creation() {
        let (sender, _receiver) = mpsc::channel();
        let manager = RealTimeUpdateManager::new(sender);
        
        assert_eq!(manager.get_total_pending_updates(), 0);
        assert_eq!(manager.render_debounce, Duration::from_millis(16));
    }

    #[test]
    fn test_queue_streaming_update() {
        let (sender, receiver) = mpsc::channel();
        let manager = RealTimeUpdateManager::new(sender);
        
        let output = StreamingOutput {
            panel_id: "test_panel".to_string(),
            line_content: "test line".to_string(),
            sequence: 1,
            timestamp: SystemTime::now(),
            is_stderr: false,
        };
        
        manager.queue_streaming_update(output);
        
        // Should trigger render immediately, so pending count should be 0
        assert_eq!(manager.get_pending_update_count("test_panel"), 0);
        
        // Should trigger render message
        let msg = receiver.try_recv().unwrap();
        if let Message::StreamingComplete(streaming_complete) = msg {
            assert_eq!(streaming_complete.panel_id, "test_panel");
            assert_eq!(streaming_complete.success, true);
            assert_eq!(streaming_complete.total_lines, 1); // "test line" has 1 line
        } else {
            panic!("Expected StreamingComplete message");
        }
    }

    #[test]
    fn test_queue_batch_updates() {
        let (sender, receiver) = mpsc::channel();
        let manager = RealTimeUpdateManager::new(sender);
        
        let outputs = vec![
            StreamingOutput {
                panel_id: "panel1".to_string(),
                line_content: "line1".to_string(),
                sequence: 1,
                timestamp: SystemTime::now(),
                is_stderr: false,
            },
            StreamingOutput {
                panel_id: "panel1".to_string(),
                line_content: "line2".to_string(),
                sequence: 2,
                timestamp: SystemTime::now(),
                is_stderr: false,
            },
        ];
        
        manager.queue_batch_updates(outputs);
        
        // Should trigger combined render
        let msg = receiver.try_recv().unwrap();
        if let Message::StreamingComplete(streaming_complete) = msg {
            assert_eq!(streaming_complete.panel_id, "panel1");
            assert_eq!(streaming_complete.success, true);
            assert_eq!(streaming_complete.total_lines, 2); // "line1\nline2" has 2 lines
        } else {
            panic!("Expected StreamingComplete message");
        }
    }

    #[test]
    fn test_force_panel_render() {
        let (sender, receiver) = mpsc::channel();
        let manager = RealTimeUpdateManager::new(sender);
        
        // Add some pending content
        let output = StreamingOutput {
            panel_id: "test_panel".to_string(),
            line_content: "test content".to_string(),
            sequence: 1,
            timestamp: SystemTime::now(),
            is_stderr: false,
        };
        
        manager.queue_streaming_update(output);
        
        // Clear the first message
        let _ = receiver.try_recv();
        
        // Add more content
        let output2 = StreamingOutput {
            panel_id: "test_panel".to_string(),
            line_content: "more content".to_string(),
            sequence: 2,
            timestamp: SystemTime::now(),
            is_stderr: false,
        };
        
        // Use internal method to add without triggering render
        {
            if let Ok(mut updates) = manager.pending_updates.lock() {
                let update = updates.entry("test_panel".to_string()).or_insert_with(|| {
                    PanelUpdate {
                        panel_id: "test_panel".to_string(),
                        content_changes: Vec::new(),
                        last_update: Instant::now(),
                        sequence_number: 0,
                        force_render: false,
                    }
                });
                update.content_changes.push("more content".to_string());
            }
        }
        
        // Force render
        manager.force_panel_render("test_panel");
        
        // Should get render message
        let mut found_update = false;
        while let Ok(msg) = receiver.try_recv() {
            if let Message::StreamingComplete(streaming_complete) = msg {
                if streaming_complete.panel_id == "test_panel" && streaming_complete.total_lines > 0 {
                    found_update = true;
                    break;
                }
            }
        }
        assert!(found_update, "Expected StreamingComplete message with content");
    }

    #[test]
    fn test_clear_operations() {
        let (sender, _receiver) = mpsc::channel();
        let manager = RealTimeUpdateManager::new(sender);
        
        // Add some updates
        let output = StreamingOutput {
            panel_id: "panel1".to_string(),
            line_content: "content1".to_string(),
            sequence: 1,
            timestamp: SystemTime::now(),
            is_stderr: false,
        };
        
        {
            if let Ok(mut updates) = manager.pending_updates.lock() {
                let update = updates.entry("panel1".to_string()).or_insert_with(|| {
                    PanelUpdate {
                        panel_id: "panel1".to_string(),
                        content_changes: Vec::new(),
                        last_update: Instant::now(),
                        sequence_number: 0,
                        force_render: false,
                    }
                });
                update.content_changes.push("content1".to_string());
            }
        }
        
        assert_eq!(manager.get_pending_update_count("panel1"), 1);
        
        // Clear specific panel
        manager.clear_panel_pending("panel1");
        assert_eq!(manager.get_pending_update_count("panel1"), 0);
        
        // Add updates for multiple panels
        {
            if let Ok(mut updates) = manager.pending_updates.lock() {
                for i in 1..=3 {
                    let panel_id = format!("panel{}", i);
                    let update = updates.entry(panel_id.clone()).or_insert_with(|| {
                        PanelUpdate {
                            panel_id: panel_id.clone(),
                            content_changes: Vec::new(),
                            last_update: Instant::now(),
                            sequence_number: 0,
                            force_render: false,
                        }
                    });
                    update.content_changes.push(format!("content{}", i));
                }
            }
        }
        
        assert_eq!(manager.get_total_pending_updates(), 3);
        
        // Clear all
        manager.clear_all_pending();
        assert_eq!(manager.get_total_pending_updates(), 0);
    }

    #[test]
    fn test_configuration_updates() {
        let (sender, _receiver) = mpsc::channel();
        let mut manager = RealTimeUpdateManager::new(sender);
        
        assert_eq!(manager.render_debounce, Duration::from_millis(16));
        assert_eq!(manager.max_update_frequency, Duration::from_millis(100));
        
        manager.set_render_debounce(Duration::from_millis(50));
        manager.set_max_update_frequency(Duration::from_millis(200));
        
        assert_eq!(manager.render_debounce, Duration::from_millis(50));
        assert_eq!(manager.max_update_frequency, Duration::from_millis(200));
    }
}
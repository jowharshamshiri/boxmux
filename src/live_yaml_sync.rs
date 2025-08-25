// F0200: Complete Live YAML Synchronization System
// Treats YAML as live file with continuous async persistence of all state changes

use crate::model::app::AppContext;
use serde_yaml;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

/// File locking to prevent multiple BoxMux processes on same YAML
#[derive(Debug)]
pub struct YamlFileLock {
    lock_file: String,
    process_id: u32,
}

impl YamlFileLock {
    pub fn dummy() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            lock_file: String::new(),
            process_id: 0,
        })
    }
    
    pub fn acquire(yaml_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lock_file = format!("{}.lock", yaml_path);
        let process_id = std::process::id();
        
        // Check if lock exists
        if Path::new(&lock_file).exists() {
            // Check if process still running
            if let Ok(contents) = fs::read_to_string(&lock_file) {
                if let Ok(existing_pid) = contents.trim().parse::<u32>() {
                    // On Unix, check if process exists
                    #[cfg(unix)]
                    {
                        if unsafe { libc::kill(existing_pid as libc::pid_t, 0) } == 0 {
                            return Err(format!("YAML file {} already in use by process {}", yaml_path, existing_pid).into());
                        }
                    }
                }
            }
        }
        
        // Create lock file
        fs::write(&lock_file, process_id.to_string())?;
        
        Ok(Self {
            lock_file,
            process_id,
        })
    }
}

impl Drop for YamlFileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_file);
    }
}

/// Async YAML persistence manager
#[derive(Debug)]
pub struct LiveYamlSync {
    yaml_path: String,
    sender: mpsc::Sender<YamlSyncMessage>,
    _lock: YamlFileLock,
}

#[derive(Debug)]
enum YamlSyncMessage {
    SaveComplete(AppContext),
    SaveBounds(String, crate::model::common::InputBounds),
    SaveContent(String, String),
    SaveScroll(String, usize, usize),
    SaveActiveLayout(String),
    Shutdown,
}

impl LiveYamlSync {
    pub fn new(yaml_path: String, enable_sync: bool) -> Result<Self, Box<dyn std::error::Error>> {
        if !enable_sync {
            // Return a dummy sync that doesn't actually save
            let (sender, _receiver) = mpsc::channel::<YamlSyncMessage>();
            return Ok(Self {
                yaml_path,
                sender,
                _lock: YamlFileLock::dummy()?,
            });
        }
        // Acquire exclusive lock
        let lock = YamlFileLock::acquire(&yaml_path)?;
        
        let (sender, receiver) = mpsc::channel::<YamlSyncMessage>();
        let yaml_path_clone = yaml_path.clone();
        
        // Spawn background sync thread
        thread::spawn(move || {
            Self::sync_worker(yaml_path_clone, receiver);
        });
        
        Ok(Self {
            yaml_path,
            sender,
            _lock: lock,
        })
    }
    
    /// Background worker for async YAML writing
    fn sync_worker(yaml_path: String, receiver: mpsc::Receiver<YamlSyncMessage>) {
        let mut last_save = SystemTime::now();
        let mut pending_changes = Vec::new();
        
        while let Ok(message) = receiver.recv_timeout(Duration::from_millis(100)) {
            match message {
                YamlSyncMessage::Shutdown => break,
                _ => {
                    pending_changes.push(message);
                    
                    // Batch writes - save every 500ms or when 10 changes accumulated
                    let should_save = pending_changes.len() >= 10 || 
                        last_save.elapsed().unwrap_or(Duration::ZERO) > Duration::from_millis(500);
                    
                    if should_save {
                        if let Err(e) = Self::apply_changes(&yaml_path, &pending_changes) {
                            log::error!("Failed to sync YAML: {}", e);
                        } else {
                            log::debug!("Synced {} changes to YAML", pending_changes.len());
                        }
                        pending_changes.clear();
                        last_save = SystemTime::now();
                    }
                }
            }
        }
        
        // Final save on shutdown
        if !pending_changes.is_empty() {
            let _ = Self::apply_changes(&yaml_path, &pending_changes);
        }
    }
    
    /// Apply accumulated changes to YAML file atomically
    fn apply_changes(yaml_path: &str, changes: &[YamlSyncMessage]) -> Result<(), Box<dyn std::error::Error>> {
        // Load current YAML
        let yaml_content = fs::read_to_string(yaml_path)?;
        let mut yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
        
        // Apply all changes
        for change in changes {
            match change {
                YamlSyncMessage::SaveComplete(app_context) => {
                    // Complete state save - serialize entire app
                    let serialized = serde_yaml::to_value(app_context)?;
                    yaml_value = serialized;
                }
                YamlSyncMessage::SaveBounds(muxbox_id, bounds) => {
                    Self::update_muxbox_bounds(&mut yaml_value, muxbox_id, bounds)?;
                }
                YamlSyncMessage::SaveContent(muxbox_id, content) => {
                    Self::update_muxbox_content(&mut yaml_value, muxbox_id, content)?;
                }
                YamlSyncMessage::SaveScroll(muxbox_id, scroll_x, scroll_y) => {
                    Self::update_muxbox_scroll(&mut yaml_value, muxbox_id, *scroll_x, *scroll_y)?;
                }
                YamlSyncMessage::SaveActiveLayout(layout_id) => {
                    Self::update_active_layout(&mut yaml_value, layout_id)?;
                }
                YamlSyncMessage::Shutdown => break,
            }
        }
        
        // Atomic write
        let temp_path = format!("{}.tmp", yaml_path);
        let updated_yaml = serde_yaml::to_string(&yaml_value)?;
        fs::write(&temp_path, updated_yaml)?;
        fs::rename(&temp_path, yaml_path)?;
        
        Ok(())
    }
    
    /// Update muxbox bounds in YAML structure
    fn update_muxbox_bounds(yaml_value: &mut serde_yaml::Value, muxbox_id: &str, bounds: &crate::model::common::InputBounds) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(muxbox) = Self::find_muxbox_mut(yaml_value, muxbox_id) {
            if let serde_yaml::Value::Mapping(map) = muxbox {
                map.insert(
                    serde_yaml::Value::String("x1".to_string()),
                    serde_yaml::Value::String(bounds.x1.clone()),
                );
                map.insert(
                    serde_yaml::Value::String("y1".to_string()),
                    serde_yaml::Value::String(bounds.y1.clone()),
                );
                map.insert(
                    serde_yaml::Value::String("x2".to_string()),
                    serde_yaml::Value::String(bounds.x2.clone()),
                );
                map.insert(
                    serde_yaml::Value::String("y2".to_string()),
                    serde_yaml::Value::String(bounds.y2.clone()),
                );
            }
        }
        Ok(())
    }
    
    /// Update muxbox content in YAML structure  
    fn update_muxbox_content(yaml_value: &mut serde_yaml::Value, muxbox_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(muxbox) = Self::find_muxbox_mut(yaml_value, muxbox_id) {
            if let serde_yaml::Value::Mapping(map) = muxbox {
                map.insert(
                    serde_yaml::Value::String("output".to_string()),
                    serde_yaml::Value::String(content.to_string()),
                );
            }
        }
        Ok(())
    }
    
    /// Update muxbox scroll position in YAML
    fn update_muxbox_scroll(yaml_value: &mut serde_yaml::Value, muxbox_id: &str, scroll_x: usize, scroll_y: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(muxbox) = Self::find_muxbox_mut(yaml_value, muxbox_id) {
            if let serde_yaml::Value::Mapping(map) = muxbox {
                map.insert(
                    serde_yaml::Value::String("scroll_x".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(scroll_x)),
                );
                map.insert(
                    serde_yaml::Value::String("scroll_y".to_string()),
                    serde_yaml::Value::Number(serde_yaml::Number::from(scroll_y)),
                );
            }
        }
        Ok(())
    }
    
    /// Update active layout in YAML
    fn update_active_layout(yaml_value: &mut serde_yaml::Value, layout_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Set all layouts active=false first
        if let Some(layouts) = yaml_value.get_mut("app").and_then(|app| app.get_mut("layouts")) {
            if let serde_yaml::Value::Sequence(layout_list) = layouts {
                for layout in layout_list.iter_mut() {
                    if let serde_yaml::Value::Mapping(map) = layout {
                        // Set all to false first
                        map.insert(
                            serde_yaml::Value::String("active".to_string()),
                            serde_yaml::Value::Bool(false),
                        );
                        
                        // Set target layout to true
                        if let Some(id_val) = map.get(&serde_yaml::Value::String("id".to_string())) {
                            if let serde_yaml::Value::String(id) = id_val {
                                if id == layout_id {
                                    map.insert(
                                        serde_yaml::Value::String("active".to_string()),
                                        serde_yaml::Value::Bool(true),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Find mutable reference to muxbox in YAML structure
    fn find_muxbox_mut<'a>(yaml_value: &'a mut serde_yaml::Value, muxbox_id: &str) -> Option<&'a mut serde_yaml::Value> {
        // Search in layouts -> children
        if let Some(app) = yaml_value.get_mut("app") {
            if let Some(layouts) = app.get_mut("layouts") {
                if let serde_yaml::Value::Sequence(layout_list) = layouts {
                    for layout in layout_list.iter_mut() {
                        if let Some(children) = layout.get_mut("children") {
                            if let Some(found) = Self::find_muxbox_in_children_mut(children, muxbox_id) {
                                return Some(found);
                            }
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Recursive search in children array
    fn find_muxbox_in_children_mut<'a>(children: &'a mut serde_yaml::Value, muxbox_id: &str) -> Option<&'a mut serde_yaml::Value> {
        if let serde_yaml::Value::Sequence(child_list) = children {
            for child in child_list.iter_mut() {
                if let serde_yaml::Value::Mapping(map) = child {
                    // Check if this is the target muxbox
                    if let Some(id_val) = map.get(&serde_yaml::Value::String("id".to_string())) {
                        if let serde_yaml::Value::String(id) = id_val {
                            if id == muxbox_id {
                                return Some(child);
                            }
                        }
                    }
                    
                    // Search in nested children
                    if let Some(nested_children) = child.get_mut(&serde_yaml::Value::String("children".to_string())) {
                        if let Some(found) = Self::find_muxbox_in_children_mut(nested_children, muxbox_id) {
                            return Some(found);
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Public API for live sync operations
    pub fn save_complete_state(&self, app_context: &AppContext) {
        let _ = self.sender.send(YamlSyncMessage::SaveComplete(app_context.clone()));
    }
    
    pub fn save_bounds(&self, muxbox_id: &str, bounds: &crate::model::common::InputBounds) {
        let _ = self.sender.send(YamlSyncMessage::SaveBounds(muxbox_id.to_string(), bounds.clone()));
    }
    
    pub fn save_content(&self, muxbox_id: &str, content: &str) {
        let _ = self.sender.send(YamlSyncMessage::SaveContent(muxbox_id.to_string(), content.to_string()));
    }
    
    pub fn save_scroll(&self, muxbox_id: &str, scroll_x: usize, scroll_y: usize) {
        let _ = self.sender.send(YamlSyncMessage::SaveScroll(muxbox_id.to_string(), scroll_x, scroll_y));
    }
    
    pub fn save_active_layout(&self, layout_id: &str) {
        let _ = self.sender.send(YamlSyncMessage::SaveActiveLayout(layout_id.to_string()));
    }
}

impl Drop for LiveYamlSync {
    fn drop(&mut self) {
        let _ = self.sender.send(YamlSyncMessage::Shutdown);
        thread::sleep(Duration::from_millis(200)); // Allow final save
    }
}
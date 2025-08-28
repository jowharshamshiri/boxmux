use crate::model::common::Bounds;
use crate::AppContext;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: String,
    pub component_types: Vec<String>,
    pub dependencies: Vec<PluginDependency>,
    pub permissions: Vec<PluginPermission>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub required: bool,
}

/// Plugin permission types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginPermission {
    FileSystem { paths: Vec<String> },
    Network { hosts: Vec<String> },
    Process { commands: Vec<String> },
    Environment { variables: Vec<String> },
}

/// Plugin component definition
pub struct PluginComponent {
    pub component_type: String,
    pub render_fn: fn(&PluginContext, &ComponentConfig) -> Result<String, PluginError>,
    pub update_fn:
        Option<fn(&PluginContext, &ComponentConfig) -> Result<ComponentState, PluginError>>,
    pub event_handler: Option<fn(&PluginContext, &PluginEvent) -> Result<(), PluginError>>,
}

/// Plugin execution context
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub app_context: AppContext,
    pub muxbox_bounds: Bounds,
    pub plugin_data: HashMap<String, serde_json::Value>,
    pub permissions: Vec<PluginPermission>,
}

/// Component configuration from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub component_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub data_source: Option<String>,
    pub refresh_interval: Option<u64>,
}

/// Component state for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentState {
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub needs_refresh: bool,
}

/// Plugin events
#[derive(Debug, Clone)]
pub enum PluginEvent {
    KeyPress(String),
    MouseEvent {
        x: u16,
        y: u16,
        action: String,
    },
    Timer {
        interval: u64,
    },
    DataUpdate {
        source: String,
        data: serde_json::Value,
    },
    MuxBoxResize {
        new_bounds: Bounds,
    },
}

/// Plugin errors
#[derive(Debug, Clone)]
pub enum PluginError {
    InitializationFailed(String),
    RenderFailed(String),
    PermissionDenied(String),
    InvalidConfiguration(String),
    RuntimeError(String),
}

/// Plugin registry for managing loaded plugins
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: HashMap<String, LoadedPlugin>,
    component_types: HashMap<String, String>, // component_type -> plugin_name
    security_manager: PluginSecurityManager,
}

/// Loaded plugin instance
struct LoadedPlugin {
    manifest: PluginManifest,
    components: HashMap<String, PluginComponent>,
    library: Option<Library>,
    is_active: bool,
    load_time: std::time::SystemTime,
}

impl std::fmt::Debug for LoadedPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedPlugin")
            .field("manifest", &self.manifest)
            .field(
                "components",
                &format!("{} components", self.components.len()),
            )
            .field("library_loaded", &self.library.is_some())
            .field("is_active", &self.is_active)
            .field("load_time", &self.load_time)
            .finish()
    }
}

/// Security manager for plugin permissions
#[derive(Debug)]
struct PluginSecurityManager {
    allowed_paths: Vec<String>,
    allowed_hosts: Vec<String>,
    allowed_commands: Vec<String>,
    _sandbox_enabled: bool,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            component_types: HashMap::new(),
            security_manager: PluginSecurityManager::new(),
        }
    }

    /// Load a plugin from a directory
    pub fn load_plugin<P: AsRef<Path>>(&mut self, plugin_path: P) -> Result<(), PluginError> {
        let manifest_path = plugin_path.as_ref().join("plugin.toml");
        let manifest = self.load_manifest(&manifest_path)?;

        // Validate permissions
        self.security_manager
            .validate_permissions(&manifest.permissions)?;

        // Try to load dynamic library first, fall back to mock if not available
        let library_path = plugin_path.as_ref().join(&manifest.entry_point);
        let (library, components) = if library_path.exists() {
            self.load_dynamic_library(&library_path, &manifest)?
        } else {
            // Fall back to mock implementation for testing/development
            (None, self.load_mock_components(&manifest)?)
        };

        let loaded_plugin = LoadedPlugin {
            manifest: manifest.clone(),
            components,
            library,
            is_active: true,
            load_time: std::time::SystemTime::now(),
        };

        // Register component types
        for component_type in &manifest.component_types {
            self.component_types
                .insert(component_type.clone(), manifest.name.clone());
        }

        self.plugins.insert(manifest.name.clone(), loaded_plugin);
        Ok(())
    }

    /// Get a component by type
    pub fn get_component(&self, component_type: &str) -> Option<&PluginComponent> {
        if let Some(plugin_name) = self.component_types.get(component_type) {
            if let Some(plugin) = self.plugins.get(plugin_name) {
                return plugin.components.get(component_type);
            }
        }
        None
    }

    /// Render a plugin component
    pub fn render_component(
        &self,
        component_type: &str,
        context: &PluginContext,
        config: &ComponentConfig,
    ) -> Result<String, PluginError> {
        if let Some(component) = self.get_component(component_type) {
            (component.render_fn)(context, config)
        } else {
            Err(PluginError::InvalidConfiguration(format!(
                "Component type '{}' not found",
                component_type
            )))
        }
    }

    /// Handle plugin events
    pub fn handle_event(
        &self,
        component_type: &str,
        context: &PluginContext,
        event: &PluginEvent,
    ) -> Result<(), PluginError> {
        if let Some(component) = self.get_component(component_type) {
            if let Some(handler) = &component.event_handler {
                handler(context, event)
            } else {
                Ok(()) // No event handler defined
            }
        } else {
            Err(PluginError::InvalidConfiguration(format!(
                "Component type '{}' not found",
                component_type
            )))
        }
    }

    /// List loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginManifest> {
        self.plugins.values().map(|p| &p.manifest).collect()
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, plugin_name: &str) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.remove(plugin_name) {
            // Remove component type registrations
            for component_type in &plugin.manifest.component_types {
                self.component_types.remove(component_type);
            }
            Ok(())
        } else {
            Err(PluginError::InvalidConfiguration(format!(
                "Plugin '{}' not found",
                plugin_name
            )))
        }
    }

    pub fn load_manifest<P: AsRef<Path>>(
        &self,
        manifest_path: P,
    ) -> Result<PluginManifest, PluginError> {
        // Try to read actual TOML file, fall back to mock for testing
        if manifest_path.as_ref().exists() {
            let content = std::fs::read_to_string(manifest_path).map_err(|e| {
                PluginError::InitializationFailed(format!("Failed to read manifest: {}", e))
            })?;

            toml::from_str(&content).map_err(|e| {
                PluginError::InitializationFailed(format!("Failed to parse manifest: {}", e))
            })
        } else {
            // Return mock manifest for testing when no actual file exists
            Ok(PluginManifest {
                name: "test_plugin".to_string(),
                version: "1.0.0".to_string(),
                author: "BoxMux Team".to_string(),
                description: "Test plugin".to_string(),
                entry_point: "lib.so".to_string(),
                component_types: vec!["custom_chart".to_string()],
                dependencies: vec![],
                permissions: vec![],
            })
        }
    }

    /// Load dynamic library and extract components
    fn load_dynamic_library<P: AsRef<Path>>(
        &self,
        library_path: P,
        manifest: &PluginManifest,
    ) -> Result<(Option<Library>, HashMap<String, PluginComponent>), PluginError> {
        unsafe {
            let library = Library::new(library_path.as_ref()).map_err(|e| {
                PluginError::InitializationFailed(format!("Failed to load library: {}", e))
            })?;

            let mut components = HashMap::new();

            // For each component type, try to load the required functions
            for component_type in &manifest.component_types {
                let render_fn_name = format!("{}_render", component_type);
                let update_fn_name = format!("{}_update", component_type);
                let event_fn_name = format!("{}_event", component_type);

                // Load render function (required)
                let render_symbol: Symbol<
                    fn(&PluginContext, &ComponentConfig) -> Result<String, PluginError>,
                > = library.get(render_fn_name.as_bytes()).map_err(|e| {
                    PluginError::InitializationFailed(format!(
                        "Failed to load render function '{}': {}",
                        render_fn_name, e
                    ))
                })?;

                // Load update function (optional)
                let update_fn = library.get(update_fn_name.as_bytes()).ok().map(
                    |symbol: Symbol<
                        fn(&PluginContext, &ComponentConfig) -> Result<ComponentState, PluginError>,
                    >| { *symbol.into_raw() },
                );

                // Load event handler (optional)
                let event_handler = library.get(event_fn_name.as_bytes()).ok().map(
                    |symbol: Symbol<
                        fn(&PluginContext, &PluginEvent) -> Result<(), PluginError>,
                    >| { *symbol.into_raw() },
                );

                let component = PluginComponent {
                    component_type: component_type.clone(),
                    render_fn: *render_symbol.into_raw(),
                    update_fn,
                    event_handler,
                };

                components.insert(component_type.clone(), component);
            }

            Ok((Some(library), components))
        }
    }

    /// Load mock components for testing/fallback
    fn load_mock_components(
        &self,
        manifest: &PluginManifest,
    ) -> Result<HashMap<String, PluginComponent>, PluginError> {
        let mut components = HashMap::new();

        // Mock component loading for testing when no dynamic library available
        for component_type in &manifest.component_types {
            let component = PluginComponent {
                component_type: component_type.clone(),
                render_fn: mock_render_function,
                update_fn: Some(mock_update_function),
                event_handler: Some(mock_event_handler),
            };
            components.insert(component_type.clone(), component);
        }

        Ok(components)
    }
}

impl PluginSecurityManager {
    fn new() -> Self {
        Self {
            allowed_paths: vec!["/tmp".to_string(), "/var/log".to_string()],
            allowed_hosts: vec!["localhost".to_string()],
            allowed_commands: vec!["echo".to_string(), "date".to_string()],
            _sandbox_enabled: true,
        }
    }

    fn validate_permissions(&self, permissions: &[PluginPermission]) -> Result<(), PluginError> {
        for permission in permissions {
            match permission {
                PluginPermission::FileSystem { paths } => {
                    for path in paths {
                        if !self.is_path_allowed(path) {
                            return Err(PluginError::PermissionDenied(format!(
                                "File system access to '{}' not allowed",
                                path
                            )));
                        }
                    }
                }
                PluginPermission::Network { hosts } => {
                    for host in hosts {
                        if !self.is_host_allowed(host) {
                            return Err(PluginError::PermissionDenied(format!(
                                "Network access to '{}' not allowed",
                                host
                            )));
                        }
                    }
                }
                PluginPermission::Process { commands } => {
                    for command in commands {
                        if !self.is_command_allowed(command) {
                            return Err(PluginError::PermissionDenied(format!(
                                "Process execution of '{}' not allowed",
                                command
                            )));
                        }
                    }
                }
                PluginPermission::Environment { variables: _ } => {
                    // Environment variable access is generally allowed
                }
            }
        }
        Ok(())
    }

    fn is_path_allowed(&self, path: &str) -> bool {
        self.allowed_paths
            .iter()
            .any(|allowed| path.starts_with(allowed))
    }

    fn is_host_allowed(&self, host: &str) -> bool {
        self.allowed_hosts.contains(&host.to_string())
    }

    fn is_command_allowed(&self, command: &str) -> bool {
        self.allowed_commands.contains(&command.to_string())
    }
}

// Mock functions for testing - would be replaced by actual plugin code
fn mock_render_function(
    _context: &PluginContext,
    config: &ComponentConfig,
) -> Result<String, PluginError> {
    Ok(format!("Custom component: {}", config.component_type))
}

fn mock_update_function(
    _context: &PluginContext,
    _config: &ComponentConfig,
) -> Result<ComponentState, PluginError> {
    Ok(ComponentState {
        content: "Updated content".to_string(),
        metadata: HashMap::new(),
        needs_refresh: false,
    })
}

fn mock_event_handler(_context: &PluginContext, event: &PluginEvent) -> Result<(), PluginError> {
    match event {
        PluginEvent::KeyPress(key) => {
            log::debug!("Plugin received key press: {}", key);
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.plugins.len(), 0);
        assert_eq!(registry.component_types.len(), 0);
    }

    #[test]
    fn test_plugin_manifest_serialization() {
        let manifest = PluginManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            author: "test_author".to_string(),
            description: "Test plugin".to_string(),
            entry_point: "lib.so".to_string(),
            component_types: vec!["custom_type".to_string()],
            dependencies: vec![],
            permissions: vec![PluginPermission::FileSystem {
                paths: vec!["/tmp".to_string()],
            }],
        };

        let serialized = serde_json::to_string(&manifest).unwrap();
        let deserialized: PluginManifest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(manifest.name, deserialized.name);
        assert_eq!(manifest.version, deserialized.version);
    }

    #[test]
    fn test_security_manager_path_validation() {
        let security_manager = PluginSecurityManager::new();

        assert!(security_manager.is_path_allowed("/tmp/test"));
        assert!(!security_manager.is_path_allowed("/etc/passwd"));
    }

    #[test]
    fn test_component_config_parsing() {
        let config_json = r#"{
            "component_type": "custom_chart",
            "properties": {
                "title": "Test Chart",
                "data_source": "metrics"
            },
            "refresh_interval": 5000
        }"#;

        let config: ComponentConfig = serde_json::from_str(config_json).unwrap();
        assert_eq!(config.component_type, "custom_chart");
        assert_eq!(config.refresh_interval, Some(5000));
    }

    #[test]
    fn test_plugin_error_display() {
        let error = PluginError::PermissionDenied("Test error".to_string());
        assert!(format!("{:?}", error).contains("Test error"));
    }
}

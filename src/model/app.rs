use crate::live_yaml_sync::LiveYamlSync;
use crate::model::muxbox::*;
use crate::{model::layout::Layout, Bounds};
use crate::components::{ErrorDisplay, ErrorInfo, ErrorSeverity};

use std::fs::File;
use std::io::Read;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

// F0200: Serializable wrapper for complete app state
#[derive(Debug, Serialize)]
struct SerializableApp {
    app: App,
}

// Implement custom serialization that includes proper app wrapper
impl SerializableApp {
    fn to_yaml_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Create proper app structure for YAML
        let yaml_content = serde_yaml::to_string(&self)?;
        Ok(yaml_content)
    }
}
use serde_yaml;
use std::collections::HashMap;
use std::sync::Arc;

use crate::validation::SchemaValidator;
use crate::{calculate_bounds_map, Config, FieldUpdate, Updatable};
use core::hash::Hash;
use regex::Regex;
use std::env;
use std::hash::{DefaultHasher, Hasher};

/// Variable context system implementing correct hierarchical precedence:
/// Child MuxBox > Parent MuxBox > Layout > App Global > Environment > Default
#[derive(Debug, Clone)]
pub struct VariableContext {
    app_vars: HashMap<String, String>,
    layout_vars: HashMap<String, String>,
}

impl VariableContext {
    pub fn new(
        app_vars: Option<&HashMap<String, String>>,
        layout_vars: Option<&HashMap<String, String>>,
    ) -> Self {
        Self {
            app_vars: app_vars.cloned().unwrap_or_default(),
            layout_vars: layout_vars.cloned().unwrap_or_default(),
        }
    }

    /// Resolve variable with correct precedence order:
    /// MuxBox Hierarchy (child->parent) > Layout > App > Environment > Default
    /// This allows YAML-defined variables to override environment for granular control
    pub fn resolve_variable(
        &self,
        name: &str,
        default: &str,
        muxbox_hierarchy: &[&MuxBox],
    ) -> String {
        // Walk up muxbox hierarchy from most granular (child) to least granular (root parent)
        for muxbox in muxbox_hierarchy.iter() {
            if let Some(variables) = &muxbox.variables {
                if let Some(muxbox_val) = variables.get(name) {
                    return muxbox_val.clone();
                }
            }
        }

        // Layout-level variables
        if let Some(layout_val) = self.layout_vars.get(name) {
            return layout_val.clone();
        }

        // App-global variables
        if let Some(app_val) = self.app_vars.get(name) {
            return app_val.clone();
        }

        // Environment variables as fallback before defaults
        if let Ok(env_val) = env::var(name) {
            return env_val;
        }

        // Finally, use default value
        default.to_string()
    }

    /// Apply variable substitution to a string with hierarchical context
    pub fn substitute_in_string(
        &self,
        content: &str,
        muxbox_hierarchy: &[&MuxBox],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = content.to_string();

        // Check for nested variables and fail gracefully with location info
        if result.contains("${") {
            let nested_pattern = Regex::new(r"\$\{[^}]*\$\{[^}]*\}")?;
            if let Some(nested_match) = nested_pattern.find(&result) {
                let problematic_text = &result[nested_match.start()..nested_match.end()];
                return Err(format!(
                    "Nested variable substitution is not supported. Found: '{}' - Use simple variables only.",
                    problematic_text
                ).into());
            }
        }

        // Pattern for variable substitution: ${VAR_NAME} or ${VAR_NAME:default_value}
        // Updated to handle the case where default contains ${} more carefully
        let var_pattern = Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}")?;

        result = var_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_value = caps.get(2).map_or("", |m| m.as_str());

                // Additional check for malformed nested syntax
                if default_value.contains("${") && !default_value.ends_with("}") {
                    return format!(
                        "error: malformed nested variable in default for '{}'",
                        var_name
                    );
                }

                self.resolve_variable(var_name, default_value, muxbox_hierarchy)
            })
            .to_string();

        // Pattern for simple environment variables: $VAR_NAME
        let env_pattern = Regex::new(r"\$([A-Z_][A-Z0-9_]*)")?;

        result = env_pattern
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                self.resolve_variable(var_name, &format!("${}", var_name), muxbox_hierarchy)
            })
            .to_string();

        Ok(result)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TemplateRoot {
    pub app: App,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct App {
    pub layouts: Vec<Layout>,
    #[serde(default)]
    pub libs: Option<Vec<String>>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub hot_keys: Option<HashMap<String, String>>,
    #[serde(default)]
    pub variables: Option<HashMap<String, String>>,
    #[serde(skip)]
    app_graph: Option<AppGraph>,
    #[serde(skip)]
    pub adjusted_bounds: Option<HashMap<String, HashMap<String, Bounds>>>,
    #[serde(skip)]
    pub execution_sources: HashMap<String, crate::model::common::UnifiedExecutionSource>,
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.layouts == other.layouts
            && self.on_keypress == other.on_keypress
            && self.hot_keys == other.hot_keys
            && self.app_graph == other.app_graph
            && self.adjusted_bounds == other.adjusted_bounds
            && self.execution_sources == other.execution_sources
    }
}

impl Eq for App {}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            layouts: Vec::new(),
            libs: None,
            on_keypress: None,
            hot_keys: None,
            variables: None,
            app_graph: None,
            adjusted_bounds: None,
            execution_sources: HashMap::new(),
        }
    }

    pub fn get_adjusted_bounds(
        &mut self,
        force_readjust: Option<bool>,
    ) -> &HashMap<String, HashMap<String, Bounds>> {
        if self.adjusted_bounds.is_none() || force_readjust.unwrap_or(false) {
            self.adjusted_bounds = Some(self.calculate_bounds());
        }
        self.adjusted_bounds
            .as_ref()
            .expect("Failed to calculate adjusted bounds!")
    }

    pub fn get_adjusted_bounds_and_app_graph(
        &mut self,
        force_readjust: Option<bool>,
    ) -> (HashMap<String, HashMap<String, Bounds>>, AppGraph) {
        // First, get the adjusted bounds by cloning the content
        let adjusted_bounds = self.get_adjusted_bounds(force_readjust).clone();

        // Then, generate the app graph
        let app_graph = self.generate_graph();

        (adjusted_bounds, app_graph)
    }

    pub fn get_layout_by_id(&self, id: &str) -> Option<&Layout> {
        self.layouts.iter().find(|l| l.id == id)
    }

    pub fn get_layout_by_id_mut(&mut self, id: &str) -> Option<&mut Layout> {
        self.layouts.iter_mut().find(|l| l.id == id)
    }

    pub fn get_root_layout(&self) -> Option<&Layout> {
        let mut roots = self.layouts.iter().filter(|l| l.root.unwrap_or(false));
        match roots.clone().count() {
            1 => roots.next(),
            0 => None,
            _ => panic!("Multiple root layouts found, which is not allowed."),
        }
    }

    pub fn get_root_layout_mut(&mut self) -> Option<&mut Layout> {
        let mut roots: Vec<&mut Layout> = self
            .layouts
            .iter_mut()
            .filter(|l| l.root.unwrap_or(false))
            .collect();

        match roots.len() {
            1 => Some(roots.remove(0)),
            0 => None,
            _ => panic!("Multiple root layouts found, which is not allowed."),
        }
    }

    pub fn get_active_layout(&self) -> Option<&Layout> {
        let mut actives = self.layouts.iter().filter(|l| l.active.unwrap_or(false));
        match actives.clone().count() {
            1 => actives.next(),
            0 => None,
            _ => panic!("Multiple active layouts found, which is not allowed."),
        }
    }

    pub fn get_active_layout_mut(&mut self) -> Option<&mut Layout> {
        let mut actives: Vec<&mut Layout> = self
            .layouts
            .iter_mut()
            .filter(|l| l.active.unwrap_or(false))
            .collect();

        match actives.len() {
            1 => Some(actives.remove(0)),
            0 => None,
            _ => panic!("Multiple active layouts found, which is not allowed."),
        }
    }

    pub fn set_active_layout(&mut self, layout_id: &str) {
        // Track whether we found the layout with the given ID.
        let mut found_layout = false;

        // Iterate through the layouts to set the active and root status.
        for layout in &mut self.layouts {
            if layout.id == layout_id {
                // If the layout matches the requested ID, set it as active and root.
                layout.active = Some(true);
                found_layout = true;
            } else {
                // Otherwise, deactivate it and unset its root status.
                layout.active = Some(false);
            }
        }

        // Log an error if no layout with the given ID was found.
        if !found_layout {
            log::error!("Layout with ID '{}' not found.", layout_id);
        }
    }

    // F0200: Set active layout with YAML persistence
    pub fn set_active_layout_with_yaml_save(
        &mut self,
        layout_id: &str,
        yaml_path: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.set_active_layout(layout_id);

        // Save to YAML if path provided
        if let Some(path) = yaml_path {
            save_active_layout_to_yaml(path, layout_id)?;
        }

        Ok(())
    }

    pub fn get_muxbox_by_id(&self, id: &str) -> Option<&MuxBox> {
        for layout in &self.layouts {
            if let Some(muxbox) = layout.get_muxbox_by_id(id) {
                return Some(muxbox);
            }
        }
        None
    }

    pub fn get_muxbox_by_id_mut(&mut self, id: &str) -> Option<&mut MuxBox> {
        for layout in &mut self.layouts {
            if let Some(muxbox) = layout.get_muxbox_by_id_mut(id) {
                return Some(muxbox);
            }
        }
        None
    }

    pub fn validate(&mut self) {
        let mut validator = SchemaValidator::new();
        match validator.validate_app(self) {
            Ok(_) => {
                // Apply post-validation setup
                if let Err(e) = apply_post_validation_setup(self) {
                    panic!("Post-validation setup error: {}", e);
                }
            }
            Err(validation_errors) => {
                let error_messages: Vec<String> = validation_errors
                    .into_iter()
                    .map(|e| format!("{}", e))
                    .collect();
                let combined_message = error_messages.join("; ");
                panic!("Validation errors: {}", combined_message);
            }
        }
    }

    pub fn calculate_bounds(&mut self) -> HashMap<String, HashMap<String, Bounds>> {
        let mut calculated_bounds: HashMap<String, HashMap<String, Bounds>> = HashMap::new();

        let app_graph = self.generate_graph();

        for layout in &mut self.layouts {
            let calculated_layout_bounds = calculate_bounds_map(&app_graph, layout);
            calculated_bounds.insert(layout.id.clone(), calculated_layout_bounds);
        }

        calculated_bounds
    }

    pub fn generate_graph(&mut self) -> AppGraph {
        if let Some(app_graph) = self.app_graph.clone() {
            app_graph
        } else {
            let mut app_graph = AppGraph::new();

            for layout in &self.layouts {
                app_graph.add_layout(layout);
            }
            self.app_graph = Some(app_graph.clone());
            app_graph
        }
    }

    pub fn replace_muxbox(&mut self, muxbox: MuxBox) {
        for layout in &mut self.layouts {
            if let Some(replaced) = layout.replace_muxbox_recursive(&muxbox) {
                if replaced {
                    return;
                }
            }
        }
    }

    /// Register a new execution source and return its stream ID
    /// For periodic sources, reuses existing stream ID if source already exists
    pub fn register_execution_source(
        &mut self,
        source_type: crate::model::common::ExecutionSourceType,
        target_box_id: String,
    ) -> String {
        // Check if this source already exists (important for periodic refresh sources)
        if let Some(existing_source_id) = self.find_existing_source(&source_type, &target_box_id) {
            // Reuse existing stream ID for this source
            return self.execution_sources[&existing_source_id]
                .stream_id
                .clone();
        }

        // Create new source with fresh IDs
        let source_id = uuid::Uuid::new_v4().to_string();
        let stream_id = uuid::Uuid::new_v4().to_string();

        let source = crate::model::common::UnifiedExecutionSource {
            source_id: source_id.clone(),
            stream_id: stream_id.clone(),
            target_box_id,
            source_type,
            created_at: std::time::SystemTime::now(),
            status: crate::model::common::ExecutionSourceStatus::Pending,
        };

        self.execution_sources.insert(source_id.clone(), source);
        stream_id
    }

    /// Find existing source by type and target box (for source reuse)
    fn find_existing_source(
        &self,
        source_type: &crate::model::common::ExecutionSourceType,
        target_box_id: &str,
    ) -> Option<String> {
        for (source_id, source) in &self.execution_sources {
            if source.target_box_id == target_box_id {
                // For periodic sources, match by target box (each box has one periodic source)
                match (&source_type, &source.source_type) {
                    (
                        crate::model::common::ExecutionSourceType::PeriodicScript(_),
                        crate::model::common::ExecutionSourceType::PeriodicScript(_),
                    ) => {
                        return Some(source_id.clone());
                    }
                    // Other source types use exact matching if needed
                    _ if source.source_type == *source_type => {
                        return Some(source_id.clone());
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// Get execution source by source ID
    pub fn get_execution_source(
        &self,
        source_id: &str,
    ) -> Option<&crate::model::common::UnifiedExecutionSource> {
        self.execution_sources.get(source_id)
    }

    /// Get execution source by source ID (mutable)
    pub fn get_execution_source_mut(
        &mut self,
        source_id: &str,
    ) -> Option<&mut crate::model::common::UnifiedExecutionSource> {
        self.execution_sources.get_mut(source_id)
    }

    /// Update execution source status
    pub fn update_source_status(
        &mut self,
        source_id: &str,
        status: crate::model::common::ExecutionSourceStatus,
    ) {
        if let Some(source) = self.execution_sources.get_mut(source_id) {
            source.status = status;
        }
    }

    /// Remove execution source
    pub fn remove_execution_source(
        &mut self,
        source_id: &str,
    ) -> Option<crate::model::common::UnifiedExecutionSource> {
        self.execution_sources.remove(source_id)
    }

    /// Pre-register periodic refresh sources for all boxes with scripts during app initialization
    /// This ensures consistent stream IDs for periodic execution
    pub fn register_periodic_sources(&mut self) {
        // Collect all box data first to avoid borrow conflicts
        let mut boxes_to_register = Vec::new();

        for layout in &self.layouts {
            for muxbox in layout.get_all_muxboxes() {
                if let Some(script) = &muxbox.script {
                    boxes_to_register.push((muxbox.id.clone(), script.join(" ")));
                }
            }
        }

        // Now register sources without borrowing conflicts
        for (box_id, script_string) in boxes_to_register {
            let source_type =
                crate::model::common::ExecutionSourceType::PeriodicScript(script_string);
            let _stream_id = self.register_execution_source(source_type, box_id.clone());
            log::info!(
                "Pre-registered periodic source for box {} with stable stream ID: {}",
                box_id,
                _stream_id
            );
        }
    }

    /// Get all sources targeting a specific box
    pub fn get_sources_for_box(
        &self,
        box_id: &str,
    ) -> Vec<&crate::model::common::UnifiedExecutionSource> {
        self.execution_sources
            .values()
            .filter(|source| source.target_box_id == box_id)
            .collect()
    }

    /// Find source by stream ID
    pub fn find_source_by_stream_id(
        &self,
        stream_id: &str,
    ) -> Option<&crate::model::common::UnifiedExecutionSource> {
        self.execution_sources
            .values()
            .find(|source| source.stream_id == stream_id)
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            layouts: self.layouts.to_vec(),
            libs: self.libs.clone(),
            on_keypress: self.on_keypress.clone(),
            hot_keys: self.hot_keys.clone(),
            variables: self.variables.clone(),
            app_graph: self.app_graph.clone(),
            adjusted_bounds: self.adjusted_bounds.clone(),
            execution_sources: self.execution_sources.clone(),
        }
    }
}

// Implement Updatable for App
impl Updatable for App {
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();

        // Compare each layout
        for (self_layout, other_layout) in self.layouts.iter().zip(&other.layouts) {
            updates.extend(self_layout.generate_diff(other_layout));
        }

        // Compare on_keypress
        if self.on_keypress != other.on_keypress {
            updates.push(FieldUpdate {
                entity_type: crate::EntityType::App,
                entity_id: None,
                field_name: "on_keypress".to_string(),
                new_value: serde_json::to_value(&other.on_keypress).unwrap(),
            });
        }

        // Compare adjusted_bounds
        if self.adjusted_bounds != other.adjusted_bounds {
            updates.push(FieldUpdate {
                entity_type: crate::EntityType::App,
                entity_id: None,
                field_name: "adjusted_bounds".to_string(),
                new_value: serde_json::to_value(&other.adjusted_bounds).unwrap(),
            });
        }

        updates
    }

    fn apply_updates(&mut self, updates: Vec<FieldUpdate>) {
        let updates_for_layouts = updates.clone();
        for update in updates {
            if update.entity_id.is_some() {
                // Skip updates that are not for the top-level entity
                continue;
            }
            match update.field_name.as_str() {
                "on_keypress" => {
                    if let Ok(new_on_keypress) = serde_json::from_value::<
                        Option<HashMap<String, Vec<String>>>,
                    >(update.new_value.clone())
                    {
                        self.on_keypress = new_on_keypress;
                    }
                }
                "adjusted_bounds" => {
                    if let Ok(new_adjusted_bounds) = serde_json::from_value::<
                        Option<HashMap<String, HashMap<String, Bounds>>>,
                    >(update.new_value.clone())
                    {
                        self.adjusted_bounds = new_adjusted_bounds;
                    }
                }
                _ => {
                    log::warn!("Unknown field name for App: {}", update.field_name);
                }
            }
        }
        for layout in &mut self.layouts {
            layout.apply_updates(updates_for_layouts.clone());
        }
    }
}

#[derive(Debug)]
pub struct AppGraph {
    graphs: HashMap<String, DiGraph<MuxBox, ()>>,
    node_maps: HashMap<String, HashMap<String, NodeIndex>>,
}

impl Hash for AppGraph {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, graph) in &self.graphs {
            key.hash(state);
            for node in graph.node_indices() {
                graph.node_weight(node).unwrap().hash(state);
            }
        }
        for (key, node_map) in &self.node_maps {
            key.hash(state);
            for (node_key, node_index) in node_map {
                node_key.hash(state);
                node_index.hash(state);
            }
        }
    }
}

impl PartialEq for AppGraph {
    fn eq(&self, other: &Self) -> bool {
        //compare hashes
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        self.hash(&mut hasher1);
        other.hash(&mut hasher2);
        hasher1.finish() == hasher2.finish()
    }
}

impl Eq for AppGraph {}

impl Default for AppGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AppGraph {
    pub fn new() -> Self {
        AppGraph {
            graphs: HashMap::new(),
            node_maps: HashMap::new(),
        }
    }

    pub fn add_layout(&mut self, layout: &Layout) {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();

        if let Some(children) = &layout.children {
            for muxbox in children {
                self.add_muxbox_recursively(
                    &mut graph,
                    &mut node_map,
                    muxbox.clone(),
                    None,
                    &layout.id,
                );
            }
        }

        self.graphs.insert(layout.id.clone(), graph);
        self.node_maps.insert(layout.id.clone(), node_map);
    }

    fn add_muxbox_recursively(
        &self,
        graph: &mut DiGraph<MuxBox, ()>,
        node_map: &mut HashMap<String, NodeIndex>,
        mut muxbox: MuxBox,
        parent_id: Option<String>,
        parent_layout_id: &str,
    ) {
        muxbox.parent_layout_id = Some(parent_layout_id.to_string());
        let muxbox_id = muxbox.id.clone();
        let node_index = graph.add_node(muxbox.clone());
        node_map.insert(muxbox_id.clone(), node_index);

        if let Some(parent_id) = muxbox.parent_id.clone() {
            if let Some(&parent_index) = node_map.get(&parent_id) {
                graph.add_edge(parent_index, node_index, ());
            }
        } else if let Some(parent_id) = parent_id {
            if let Some(&parent_index) = node_map.get(&parent_id) {
                graph.add_edge(parent_index, node_index, ());
            }
        }

        if let Some(children) = muxbox.children {
            for mut child in children {
                child.parent_id = Some(muxbox_id.clone());
                self.add_muxbox_recursively(
                    graph,
                    node_map,
                    child,
                    Some(muxbox_id.clone()),
                    parent_layout_id,
                );
            }
        }
    }

    pub fn get_layout_muxbox_by_id(&self, layout_id: &str, muxbox_id: &str) -> Option<&MuxBox> {
        self.node_maps.get(layout_id).and_then(|node_map| {
            node_map.get(muxbox_id).and_then(|&index| {
                self.graphs
                    .get(layout_id)
                    .and_then(|graph| graph.node_weight(index))
            })
        })
    }

    pub fn get_muxbox_by_id(&self, muxbox_id: &str) -> Option<&MuxBox> {
        for (layout_id, node_map) in &self.node_maps {
            if let Some(&index) = node_map.get(muxbox_id) {
                return self
                    .graphs
                    .get(layout_id)
                    .and_then(|graph| graph.node_weight(index));
            }
        }
        None
    }

    pub fn get_children(&self, layout_id: &str, muxbox_id: &str) -> Vec<&MuxBox> {
        if let Some(node_map) = self.node_maps.get(layout_id) {
            if let Some(&index) = node_map.get(muxbox_id) {
                return self.graphs[layout_id]
                    .edges_directed(index, petgraph::Direction::Outgoing)
                    .map(|edge| self.graphs[layout_id].node_weight(edge.target()).unwrap())
                    .collect();
            }
        }
        Vec::new()
    }

    pub fn get_layout_children(&self, layout_id: &str) -> Vec<&MuxBox> {
        if let Some(node_map) = self.node_maps.get(layout_id) {
            let root_node = node_map.get(layout_id).unwrap();
            return self.graphs[layout_id]
                .edges_directed(*root_node, petgraph::Direction::Outgoing)
                .map(|edge| self.graphs[layout_id].node_weight(edge.target()).unwrap())
                .collect();
        }
        Vec::new()
    }

    pub fn get_parent(&self, layout_id: &str, muxbox_id: &str) -> Option<&MuxBox> {
        if let Some(node_map) = self.node_maps.get(layout_id) {
            if let Some(&index) = node_map.get(muxbox_id) {
                return self.graphs[layout_id]
                    .edges_directed(index, petgraph::Direction::Incoming)
                    .next()
                    .and_then(|edge| self.graphs[layout_id].node_weight(edge.source()));
            }
        }
        None
    }
}

#[derive(Debug, Serialize)]
pub struct AppContext {
    pub app: App,
    pub config: Config,
    #[serde(skip)]
    pub plugin_registry: std::sync::Arc<std::sync::Mutex<crate::plugin::PluginRegistry>>,
    #[serde(skip)]
    pub pty_manager: Option<std::sync::Arc<crate::pty_manager::PtyManager>>,
    pub yaml_file_path: Option<String>,
    #[serde(skip)]
    pub live_yaml_sync: Option<Arc<LiveYamlSync>>,
}

impl Updatable for AppContext {
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();

        // Compare app
        updates.extend(self.app.generate_diff(&other.app));

        // Compare config
        if self.config != other.config {
            updates.push(FieldUpdate {
                entity_type: crate::EntityType::AppContext,
                entity_id: None,
                field_name: "config".to_string(),
                new_value: serde_json::to_value(&other.config).unwrap(),
            });
        }

        updates
    }

    fn apply_updates(&mut self, updates: Vec<FieldUpdate>) {
        let updates_for_layouts = updates.clone();

        for update in updates {
            if update.entity_id.is_some() {
                // Skip updates that are not for the top-level entity
                continue;
            }
            match update.field_name.as_str() {
                "config" => {
                    if let Ok(new_config) =
                        serde_json::from_value::<Config>(update.new_value.clone())
                    {
                        self.config = new_config;
                    }
                }
                _ => log::warn!("Unknown field name for AppContext: {}", update.field_name),
            }
        }

        self.app.apply_updates(updates_for_layouts);
    }
}

impl PartialEq for AppContext {
    fn eq(&self, other: &Self) -> bool {
        self.app == other.app && self.config == other.config
    }
}

impl AppContext {
    pub fn new(app: App, config: Config) -> Self {
        // App is already validated in load_app_from_yaml
        AppContext {
            app,
            config,
            plugin_registry: std::sync::Arc::new(std::sync::Mutex::new(
                crate::plugin::PluginRegistry::new(),
            )),
            pty_manager: None,
            yaml_file_path: None,
            live_yaml_sync: None,
        }
    }

    pub fn new_with_pty(
        app: App,
        config: Config,
        pty_manager: std::sync::Arc<crate::pty_manager::PtyManager>,
    ) -> Self {
        AppContext {
            app,
            config,
            plugin_registry: std::sync::Arc::new(std::sync::Mutex::new(
                crate::plugin::PluginRegistry::new(),
            )),
            pty_manager: Some(pty_manager),
            yaml_file_path: None,
            live_yaml_sync: None,
        }
    }

    // F0190: Constructor with YAML file path for live updates
    pub fn new_with_yaml_path(app: App, config: Config, yaml_path: String) -> Self {
        Self::new_with_yaml_path_and_lock(app, config, yaml_path, false)
    }

    pub fn new_with_yaml_path_and_lock(
        app: App,
        config: Config,
        yaml_path: String,
        locked: bool,
    ) -> Self {
        let live_yaml_sync = if !locked {
            match LiveYamlSync::new(yaml_path.clone(), true) {
                Ok(sync) => {
                    log::info!("Live YAML sync initialized");
                    Some(Arc::new(sync))
                }
                Err(e) => {
                    log::error!("Failed to initialize live YAML sync: {}", e);
                    None
                }
            }
        } else {
            None
        };

        AppContext {
            app,
            config,
            plugin_registry: std::sync::Arc::new(std::sync::Mutex::new(
                crate::plugin::PluginRegistry::new(),
            )),
            pty_manager: None,
            yaml_file_path: Some(yaml_path),
            live_yaml_sync,
        }
    }

    pub fn new_with_pty_and_yaml(
        app: App,
        config: Config,
        pty_manager: std::sync::Arc<crate::pty_manager::PtyManager>,
        yaml_path: String,
    ) -> Self {
        Self::new_with_pty_and_yaml_and_lock(app, config, pty_manager, yaml_path, false)
    }

    pub fn new_with_pty_and_yaml_and_lock(
        app: App,
        config: Config,
        pty_manager: std::sync::Arc<crate::pty_manager::PtyManager>,
        yaml_path: String,
        locked: bool,
    ) -> Self {
        let live_yaml_sync = if !locked {
            match LiveYamlSync::new(yaml_path.clone(), true) {
                Ok(sync) => {
                    log::info!("Live YAML sync initialized with PTY");
                    Some(Arc::new(sync))
                }
                Err(e) => {
                    log::error!("Failed to initialize live YAML sync: {}", e);
                    None
                }
            }
        } else {
            None
        };

        AppContext {
            app,
            config,
            plugin_registry: std::sync::Arc::new(std::sync::Mutex::new(
                crate::plugin::PluginRegistry::new(),
            )),
            pty_manager: Some(pty_manager),
            yaml_file_path: Some(yaml_path),
            live_yaml_sync,
        }
    }
}

impl Clone for AppContext {
    fn clone(&self) -> Self {
        AppContext {
            app: self.app.clone(),
            config: self.config.clone(),
            plugin_registry: self.plugin_registry.clone(),
            pty_manager: self.pty_manager.clone(),
            yaml_file_path: self.yaml_file_path.clone(),
            live_yaml_sync: self.live_yaml_sync.clone(),
        }
    }
}

impl Hash for AppContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app.hash(state);
        self.config.hash(state);
        // Note: plugin_registry contains Mutex which doesn't implement Hash
    }
}

impl Hash for App {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for layout in &self.layouts {
            layout.hash(state);
        }
    }
}

impl Clone for AppGraph {
    fn clone(&self) -> Self {
        let mut new_graphs = HashMap::new();
        let mut new_node_maps = HashMap::new();

        for (key, graph) in &self.graphs {
            let new_graph = graph.clone();
            // Using unwrap here assumes there must always be a corresponding node_map for each graph.
            // This will panic if that invariant is broken, which is considered a critical and unexpected error.
            let new_node_map = self.node_maps.get(key).unwrap().clone();
            new_graphs.insert(key.clone(), new_graph);
            new_node_maps.insert(key.clone(), new_node_map);
        }

        AppGraph {
            graphs: new_graphs,
            node_maps: new_node_maps,
        }
    }
}

pub fn load_app_from_yaml(file_path: &str) -> Result<App, Box<dyn std::error::Error>> {
    load_app_from_yaml_with_lock(file_path, false)
}

pub fn load_app_from_yaml_with_lock(
    file_path: &str,
    _locked: bool,
) -> Result<App, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // First, perform JSON schema validation if schema files exist
    let schema_dir = "schemas";
    if std::path::Path::new(schema_dir).exists() {
        let mut validator = SchemaValidator::new();
        if let Err(schema_errors) = validator.validate_with_json_schema(&contents, schema_dir) {
            let error_messages: Vec<String> = schema_errors
                .into_iter()
                .map(|e| format!("{}", e))
                .collect();
            let combined_message = error_messages.join("\n");
            return Err(format!("JSON Schema validation failed:\n{}", combined_message).into());
        }
    }

    // Parse YAML first to extract variable definitions
    let root_result: Result<TemplateRoot, _> = serde_yaml::from_str(&contents);

    let mut app = match root_result {
        Ok(root) => root.app,
        Err(serde_error) => {
            // Enhanced error handling with Rust-style line/column display
            if let Some(location) = serde_error.location() {
                let line_num = location.line();
                let col_num = location.column();
                let error_display = ErrorDisplay::with_terminal_config("yaml_parser".to_string());
                let error_info = ErrorInfo {
                    message: format!("{}", serde_error),
                    file_path: file_path.to_string(),
                    line_number: line_num,
                    column_number: col_num,
                    severity: ErrorSeverity::Error,
                    help: Some("Check YAML syntax and structure".to_string()),
                    note: None,
                    caret_positioning: None,
                };
                let formatted_error = error_display.format_error(&error_info, &contents);
                return Err(formatted_error.into());
            }

            // Fallback: try to deserialize directly into App
            match serde_yaml::from_str::<App>(&contents) {
                Ok(app) => app,
                Err(app_error) => {
                    if let Some(location) = app_error.location() {
                        let line_num = location.line();
                        let col_num = location.column();
                        let error_display = ErrorDisplay::with_terminal_config("app_parser".to_string());
                        let error_info = ErrorInfo {
                            message: format!("{}", app_error),
                            file_path: file_path.to_string(),
                            line_number: line_num,
                            column_number: col_num,
                            severity: ErrorSeverity::Error,
                            help: Some("Verify configuration structure".to_string()),
                            note: None,
                            caret_positioning: None,
                        };
                        let formatted_error = error_display.format_error(&error_info, &contents);
                        return Err(formatted_error.into());
                    }
                    return Err(format!("YAML parsing error: {}", app_error).into());
                }
            }
        }
    };

    // Apply variable substitution AFTER parsing with hierarchical context
    apply_variable_substitution(&mut app)?;

    // Validate the app configuration using SchemaValidator
    let mut validator = SchemaValidator::new();
    match validator.validate_app(&app) {
        Ok(_) => {
            // Apply the old validation logic for setting up parent relationships and defaults
            apply_post_validation_setup(&mut app)?;

            // Pre-register periodic refresh sources for unified execution architecture
            app.register_periodic_sources();

            Ok(app)
        }
        Err(validation_errors) => {
            let error_messages: Vec<String> = validation_errors
                .into_iter()
                .map(|e| format!("{}", e))
                .collect();
            let combined_message = error_messages.join("; ");
            Err(format!("Configuration validation errors: {}", combined_message).into())
        }
    }
}


/// Apply variable substitution to all fields in the parsed App structure
fn apply_variable_substitution(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Create variable context with app-level variables
    let context = VariableContext::new(app.variables.as_ref(), None);

    // Apply substitution to all layouts
    for layout in &mut app.layouts {
        apply_layout_variable_substitution(layout, &context)?;
    }

    Ok(())
}

/// Apply variable substitution to a layout and all its muxboxes
fn apply_layout_variable_substitution(
    layout: &mut Layout,
    context: &VariableContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use the same context for layout (layout variables not yet implemented)
    let layout_context = context;

    // Apply to layout title
    if let Some(ref mut title) = layout.title {
        *title = layout_context
            .substitute_in_string(title, &[])
            .map_err(|e| format!("Error in layout '{}' title: {}", layout.id, e))?;
    }

    // Apply to all child muxboxes
    if let Some(ref mut children) = layout.children {
        for child in children {
            apply_muxbox_variable_substitution(child, &layout_context, &[])?;
        }
    }

    Ok(())
}

/// Apply variable substitution to a muxbox and its children with hierarchy context
fn apply_muxbox_variable_substitution(
    muxbox: &mut MuxBox,
    context: &VariableContext,
    parent_hierarchy: &[&MuxBox],
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a local variable context including this muxbox's variables
    let local_context = if let Some(ref muxbox_vars) = muxbox.variables {
        let mut combined_app_vars = context.app_vars.clone();
        // Add muxbox variables to context for this muxbox and children
        combined_app_vars.extend(muxbox_vars.clone());
        VariableContext::new(Some(&combined_app_vars), Some(&context.layout_vars))
    } else {
        context.clone()
    };

    // Build complete muxbox hierarchy for variable resolution
    let full_hierarchy = parent_hierarchy.to_vec();
    // Note: We can't add 'muxbox' to hierarchy due to borrowing issues
    // Instead, we've merged muxbox variables into the context above

    // Apply substitution to muxbox fields with error context
    if let Some(ref mut title) = muxbox.title {
        *title = local_context
            .substitute_in_string(title, &full_hierarchy)
            .map_err(|e| format!("Error in muxbox '{}' title: {}", muxbox.id, e))?;
    }

    if let Some(ref mut content) = muxbox.content {
        *content = local_context
            .substitute_in_string(content, &full_hierarchy)
            .map_err(|e| format!("Error in muxbox '{}' content: {}", muxbox.id, e))?;
    }

    if let Some(ref mut script) = muxbox.script {
        for (i, script_line) in script.iter_mut().enumerate() {
            *script_line = local_context
                .substitute_in_string(script_line, &full_hierarchy)
                .map_err(|e| {
                    format!(
                        "Error in muxbox '{}' script line {}: {}",
                        muxbox.id,
                        i + 1,
                        e
                    )
                })?;
        }
    }

    if let Some(ref mut redirect) = muxbox.redirect_output {
        *redirect = local_context
            .substitute_in_string(redirect, &full_hierarchy)
            .map_err(|e| format!("Error in muxbox '{}' redirect_output: {}", muxbox.id, e))?;
    }

    // Apply to choices if present
    if let Some(ref mut choices) = muxbox.choices {
        for choice in choices {
            if let Some(ref mut choice_content) = choice.content {
                *choice_content = local_context
                    .substitute_in_string(choice_content, &full_hierarchy)
                    .map_err(|e| {
                        format!(
                            "Error in muxbox '{}' choice '{}' content: {}",
                            muxbox.id, choice.id, e
                        )
                    })?;
            }

            if let Some(ref mut choice_script) = choice.script {
                for (i, script_line) in choice_script.iter_mut().enumerate() {
                    *script_line = local_context
                        .substitute_in_string(script_line, &full_hierarchy)
                        .map_err(|e| {
                            format!(
                                "Error in muxbox '{}' choice '{}' script line {}: {}",
                                muxbox.id,
                                choice.id,
                                i + 1,
                                e
                            )
                        })?;
                }
            }
        }
    }

    // Recursively apply to child muxboxes
    if let Some(ref mut children) = muxbox.children {
        for child in children {
            // For children, we can't include 'muxbox' in hierarchy due to borrowing
            // but the local_context already includes muxbox variables
            apply_muxbox_variable_substitution(child, &local_context, &full_hierarchy)?;
        }
    }

    Ok(())
}

/// Legacy function kept for backward compatibility (now unused in main flow)
pub fn substitute_variables(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // This function is deprecated in favor of the new hierarchical system
    // but kept for any external dependencies
    let context = VariableContext::new(None, None);
    context.substitute_in_string(content, &[])
}

fn apply_post_validation_setup(app: &mut App) -> Result<(), String> {
    // This function applies the setup logic that was previously in validate_app
    // after the SchemaValidator has already validated the structure

    fn set_parent_ids(muxbox: &mut MuxBox, parent_layout_id: &str, parent_id: Option<String>) {
        muxbox.parent_layout_id = Some(parent_layout_id.to_string());
        muxbox.parent_id = parent_id;

        if let Some(ref mut children) = muxbox.children {
            for child in children {
                set_parent_ids(child, parent_layout_id, Some(muxbox.id.clone()));
            }
        }
    }

    let mut root_layout_id: Option<String> = None;

    for layout in &mut app.layouts {
        let mut layout_clone = layout.clone();
        let muxboxes_in_tab_order = layout_clone.get_muxboxes_in_tab_order();

        // Identify root layout
        if layout.root.unwrap_or(false) {
            root_layout_id = Some(layout.id.clone());
        }

        if layout.children.is_none() {
            continue;
        }

        // Set up parent relationships and defaults
        for muxbox in layout.children.as_mut().unwrap() {
            set_parent_ids(muxbox, &layout.id, None);
            if !muxboxes_in_tab_order.is_empty() && muxbox.id == muxboxes_in_tab_order[0].id {
                muxbox.selected = Some(true);
            }
            if let Some(choices) = &mut muxbox.choices {
                if !choices.is_empty() {
                    choices[0].selected = true;
                }
            }

            // F0204-F0209: Initialize stream architecture for each muxbox
            muxbox.initialize_streams();

            // Initialize streams for children recursively
            fn initialize_child_streams(muxbox: &mut MuxBox) {
                muxbox.initialize_streams();
                if let Some(children) = &mut muxbox.children {
                    for child in children {
                        initialize_child_streams(child);
                    }
                }
            }

            if let Some(children) = &mut muxbox.children {
                for child in children {
                    initialize_child_streams(child);
                }
            }
        }
    }

    // Set default root layout if none specified
    if root_layout_id.is_none() {
        if let Some(first_layout) = app.layouts.first() {
            root_layout_id = Some(first_layout.id.clone());
        }
    }

    // Set the root layout as active
    if let Some(root_layout_id) = root_layout_id {
        if let Some(root_layout) = app.layouts.iter_mut().find(|l| l.id == root_layout_id) {
            root_layout.active = Some(true);
            root_layout.root = Some(true);

            // Set all other layouts as inactive
            for layout in &mut app.layouts {
                if layout.id != root_layout_id {
                    layout.active = Some(false);
                    layout.root = Some(false);
                }
            }
        }
    }

    Ok(())
}

// The old check_unique_ids and check_muxbox_ids functions are no longer needed
// because SchemaValidator handles ID uniqueness validation

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::InputBounds;
    use crate::model::layout::Layout;
    use crate::model::muxbox::MuxBox;
    use std::collections::HashMap;

    // === Helper Functions ===

    /// Creates a basic test muxbox with the given id.
    /// This helper demonstrates how to create a MuxBox for App testing.
    fn create_test_muxbox(id: &str) -> MuxBox {
        MuxBox {
            id: id.to_string(),
            title: Some(format!("Test MuxBox {}", id)),
            position: InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            tab_order: Some("1".to_string()),
            selected: Some(false),
            ..Default::default()
        }
    }

    /// Creates a test Layout with the given id and optional children.
    /// This helper demonstrates how to create a Layout for App testing.
    fn create_test_layout(id: &str, children: Option<Vec<MuxBox>>) -> Layout {
        Layout {
            id: id.to_string(),
            title: Some(format!("Test Layout {}", id)),
            children,
            root: Some(false),
            active: Some(false),
            ..Default::default()
        }
    }

    /// Creates a test App with basic layouts and muxboxes.
    /// This helper demonstrates how to create an App for testing.
    fn create_test_app() -> App {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout1 = create_test_layout("layout1", Some(vec![muxbox1, muxbox2]));

        let mut app = App::new();
        app.layouts.push(layout1);
        app
    }

    /// Creates a test AppContext with a basic app configuration.
    /// This helper demonstrates how to create an AppContext for testing.
    fn create_test_app_context() -> AppContext {
        let app = create_test_app();
        AppContext::new(app, Config::default())
    }

    fn load_test_app_context() -> AppContext {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let dashboard_path = current_dir.join("layouts/tests.yaml");
        let app = load_app_from_yaml(dashboard_path.to_str().unwrap()).expect("Failed to load app");
        AppContext::new(app, Config::default())
    }

    fn setup_app_context() -> AppContext {
        load_test_app_context()
    }

    // === App Default Tests ===

    /// Tests that App::new() creates an app with expected default values.
    /// This test demonstrates the default App construction behavior.
    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.layouts.len(), 0);
        assert_eq!(app.libs, None);
        assert_eq!(app.on_keypress, None);
        assert_eq!(app.app_graph, None);
        assert_eq!(app.adjusted_bounds, None);
    }

    /// Tests that App::default() creates an app with expected default values.
    /// This test demonstrates the default App construction behavior.
    #[test]
    fn test_app_default() {
        let app = App::default();
        assert_eq!(app.layouts.len(), 0);
        assert_eq!(app.libs, None);
        assert_eq!(app.on_keypress, None);
        assert_eq!(app.app_graph, None);
        assert_eq!(app.adjusted_bounds, None);
    }

    // === App Layout Management Tests ===

    /// Tests that App::get_layout_by_id() finds layouts correctly.
    /// This test demonstrates the layout retrieval feature.
    #[test]
    fn test_app_get_layout_by_id() {
        let app = create_test_app();

        let found_layout = app.get_layout_by_id("layout1");
        assert!(found_layout.is_some());
        assert_eq!(found_layout.unwrap().id, "layout1");

        let not_found = app.get_layout_by_id("nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that App::get_layout_by_id_mut() finds and allows modification.
    /// This test demonstrates the mutable layout retrieval feature.
    #[test]
    fn test_app_get_layout_by_id_mut() {
        let mut app = create_test_app();

        let found_layout = app.get_layout_by_id_mut("layout1");
        assert!(found_layout.is_some());

        // Modify the layout
        found_layout.unwrap().title = Some("Modified Layout".to_string());

        // Verify the modification
        let verified_layout = app.get_layout_by_id("layout1");
        assert_eq!(
            verified_layout.unwrap().title,
            Some("Modified Layout".to_string())
        );
    }

    /// Tests that App::get_layout_by_id_mut() handles empty app.
    /// This test demonstrates edge case handling in mutable layout retrieval.
    #[test]
    fn test_app_get_layout_by_id_mut_empty() {
        let mut app = App::new();

        let found_layout = app.get_layout_by_id_mut("nonexistent");
        assert!(found_layout.is_none());
    }

    // === App Root Layout Tests ===

    /// Tests that App::get_root_layout() finds the root layout correctly.
    /// This test demonstrates the root layout retrieval feature.
    #[test]
    fn test_app_get_root_layout() {
        let mut app = create_test_app();

        // Initially no root layout
        assert!(app.get_root_layout().is_none());

        // Set a layout as root
        app.layouts[0].root = Some(true);

        let root_layout = app.get_root_layout();
        assert!(root_layout.is_some());
        assert_eq!(root_layout.unwrap().id, "layout1");
    }

    /// Tests that App::get_root_layout() panics with multiple root layouts.
    /// This test demonstrates the root layout validation feature.
    #[test]
    #[should_panic(expected = "Multiple root layouts found, which is not allowed.")]
    fn test_app_get_root_layout_multiple_panics() {
        let mut app = create_test_app();

        // Add another layout and set both as root
        let layout2 = create_test_layout("layout2", None);
        app.layouts.push(layout2);
        app.layouts[0].root = Some(true);
        app.layouts[1].root = Some(true);

        app.get_root_layout();
    }

    /// Tests that App::get_root_layout_mut() finds and allows modification.
    /// This test demonstrates the mutable root layout retrieval feature.
    #[test]
    fn test_app_get_root_layout_mut() {
        let mut app = create_test_app();
        app.layouts[0].root = Some(true);

        let root_layout = app.get_root_layout_mut();
        assert!(root_layout.is_some());

        // Modify the root layout
        root_layout.unwrap().title = Some("Modified Root".to_string());

        // Verify the modification
        let verified_layout = app.get_root_layout();
        assert_eq!(
            verified_layout.unwrap().title,
            Some("Modified Root".to_string())
        );
    }

    // === App Active Layout Tests ===

    /// Tests that App::get_active_layout() finds the active layout correctly.
    /// This test demonstrates the active layout retrieval feature.
    #[test]
    fn test_app_get_active_layout() {
        let mut app = create_test_app();

        // Initially no active layout
        assert!(app.get_active_layout().is_none());

        // Set a layout as active
        app.layouts[0].active = Some(true);

        let active_layout = app.get_active_layout();
        assert!(active_layout.is_some());
        assert_eq!(active_layout.unwrap().id, "layout1");
    }

    /// Tests that App::get_active_layout() panics with multiple active layouts.
    /// This test demonstrates the active layout validation feature.
    #[test]
    #[should_panic(expected = "Multiple active layouts found, which is not allowed.")]
    fn test_app_get_active_layout_multiple_panics() {
        let mut app = create_test_app();

        // Add another layout and set both as active
        let layout2 = create_test_layout("layout2", None);
        app.layouts.push(layout2);
        app.layouts[0].active = Some(true);
        app.layouts[1].active = Some(true);

        app.get_active_layout();
    }

    /// Tests that App::get_active_layout_mut() finds and allows modification.
    /// This test demonstrates the mutable active layout retrieval feature.
    #[test]
    fn test_app_get_active_layout_mut() {
        let mut app = create_test_app();
        app.layouts[0].active = Some(true);

        let active_layout = app.get_active_layout_mut();
        assert!(active_layout.is_some());

        // Modify the active layout
        active_layout.unwrap().title = Some("Modified Active".to_string());

        // Verify the modification
        let verified_layout = app.get_active_layout();
        assert_eq!(
            verified_layout.unwrap().title,
            Some("Modified Active".to_string())
        );
    }

    /// Tests that App::set_active_layout() sets the correct layout as active.
    /// This test demonstrates the active layout setting feature.
    #[test]
    fn test_app_set_active_layout() {
        let mut app = create_test_app();

        // Add another layout
        let layout2 = create_test_layout("layout2", None);
        app.layouts.push(layout2);

        // Set layout2 as active
        app.set_active_layout("layout2");

        let active_layout = app.get_active_layout();
        assert!(active_layout.is_some());
        assert_eq!(active_layout.unwrap().id, "layout2");

        // Verify layout1 is not active
        let layout1 = app.get_layout_by_id("layout1").unwrap();
        assert_eq!(layout1.active, Some(false));
    }

    /// Tests that App::set_active_layout() logs error for nonexistent layout.
    /// This test demonstrates error handling in active layout setting.
    #[test]
    fn test_app_set_active_layout_nonexistent() {
        let mut app = create_test_app();

        // This should not panic but should log an error
        app.set_active_layout("nonexistent");

        // No layout should be active
        assert!(app.get_active_layout().is_none());
    }

    // === App MuxBox Management Tests ===

    /// Tests that App::get_muxbox_by_id() finds muxboxes across layouts.
    /// This test demonstrates the cross-layout muxbox retrieval feature.
    #[test]
    fn test_app_get_muxbox_by_id() {
        let app = create_test_app();

        let found_muxbox = app.get_muxbox_by_id("muxbox1");
        assert!(found_muxbox.is_some());
        assert_eq!(found_muxbox.unwrap().id, "muxbox1");

        let not_found = app.get_muxbox_by_id("nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that App::get_muxbox_by_id_mut() finds and allows modification.
    /// This test demonstrates the mutable cross-layout muxbox retrieval feature.
    #[test]
    fn test_app_get_muxbox_by_id_mut() {
        let mut app = create_test_app();

        let found_muxbox = app.get_muxbox_by_id_mut("muxbox1");
        assert!(found_muxbox.is_some());

        // Modify the muxbox
        found_muxbox.unwrap().title = Some("Modified MuxBox".to_string());

        // Verify the modification
        let verified_muxbox = app.get_muxbox_by_id("muxbox1");
        assert_eq!(
            verified_muxbox.unwrap().title,
            Some("Modified MuxBox".to_string())
        );
    }

    /// Tests that App::get_muxbox_by_id_mut() handles empty app.
    /// This test demonstrates edge case handling in mutable muxbox retrieval.
    #[test]
    fn test_app_get_muxbox_by_id_mut_empty() {
        let mut app = App::new();

        let found_muxbox = app.get_muxbox_by_id_mut("nonexistent");
        assert!(found_muxbox.is_none());
    }

    // === App Validation Tests ===

    /// Tests that App::validate() sets up parent relationships correctly.
    /// This test demonstrates the app validation feature.
    #[test]
    fn test_app_validate() {
        let mut app = create_test_app();

        // Before validation, parent relationships should not be set
        let muxbox = app.get_muxbox_by_id("muxbox1").unwrap();
        assert_eq!(muxbox.parent_layout_id, None);
        assert_eq!(muxbox.parent_id, None);

        app.validate();

        // After validation, parent relationships should be set
        let muxbox = app.get_muxbox_by_id("muxbox1").unwrap();
        assert_eq!(muxbox.parent_layout_id, Some("layout1".to_string()));
        assert_eq!(muxbox.parent_id, None); // Top-level muxbox has no parent muxbox
    }

    /// Tests that App::validate() sets root layout as active.
    /// This test demonstrates the root layout activation feature.
    #[test]
    fn test_app_validate_root_layout_activation() {
        let mut app = create_test_app();
        app.layouts[0].root = Some(true);

        app.validate();

        let layout = app.get_layout_by_id("layout1").unwrap();
        assert_eq!(layout.active, Some(true));
    }

    /// Tests that App::validate() defaults to first layout when no root.
    /// This test demonstrates the default root layout behavior.
    #[test]
    fn test_app_validate_default_root() {
        let mut app = create_test_app();

        // Add another layout
        let layout2 = create_test_layout("layout2", None);
        app.layouts.push(layout2);

        app.validate();

        // First layout should be set as root and active
        let layout1 = app.get_layout_by_id("layout1").unwrap();
        assert_eq!(layout1.root, Some(true));
        assert_eq!(layout1.active, Some(true));

        // Second layout should not be root or active
        let layout2 = app.get_layout_by_id("layout2").unwrap();
        assert_eq!(layout2.root, Some(false));
        assert_eq!(layout2.active, Some(false));
    }

    /// Tests that App::validate() panics with no layouts.
    /// This test demonstrates the empty app validation behavior.
    #[test]
    #[should_panic(expected = "Required field 'layouts' is missing")]
    fn test_app_validate_empty_panics() {
        let mut app = App::new();
        app.validate();
    }

    /// Tests that App::validate() panics with duplicate IDs.
    /// This test demonstrates the duplicate ID validation feature.
    #[test]
    #[should_panic(expected = "Duplicate ID 'muxbox1' found in muxboxes")]
    fn test_app_validate_duplicate_ids_panics() {
        let mut app = App::new();

        // Create two muxboxes with the same ID
        let muxbox1a = create_test_muxbox("muxbox1");
        let muxbox1b = create_test_muxbox("muxbox1"); // Duplicate ID

        let layout = create_test_layout("layout1", Some(vec![muxbox1a, muxbox1b]));
        app.layouts.push(layout);

        app.validate();
    }

    /// Tests that App::validate() panics with multiple root layouts.
    /// This test demonstrates the multiple root layout validation feature.
    #[test]
    #[should_panic(
        expected = "Schema structure error: Multiple root layouts detected. Only one layout can be marked as 'root: true'."
    )]
    fn test_app_validate_multiple_root_panics() {
        let mut app = create_test_app();

        // Add another layout and set both as root
        let mut layout2 = create_test_layout("layout2", None);
        layout2.root = Some(true);
        app.layouts.push(layout2);
        app.layouts[0].root = Some(true);

        app.validate();
    }

    // === App Bounds Calculation Tests ===

    /// Tests that App::calculate_bounds() calculates bounds for all layouts.
    /// This test demonstrates the bounds calculation feature.
    #[test]
    fn test_app_calculate_bounds() {
        let mut app = create_test_app();

        let bounds = app.calculate_bounds();
        assert!(bounds.contains_key("layout1"));

        let layout_bounds = bounds.get("layout1").unwrap();
        assert!(layout_bounds.contains_key("muxbox1"));
        assert!(layout_bounds.contains_key("muxbox2"));
    }

    /// Tests that App::get_adjusted_bounds() caches bounds correctly.
    /// This test demonstrates the bounds caching feature.
    #[test]
    fn test_app_get_adjusted_bounds() {
        let mut app = create_test_app();

        // First call should calculate bounds
        let bounds1 = app.get_adjusted_bounds(None).clone();
        assert!(bounds1.contains_key("layout1"));

        // Second call should return cached bounds
        let bounds2 = app.get_adjusted_bounds(None).clone();
        assert_eq!(bounds1, bounds2);

        // Force recalculation
        let bounds3 = app.get_adjusted_bounds(Some(true));
        assert!(bounds3.contains_key("layout1"));
    }

    /// Tests that App::get_adjusted_bounds_and_app_graph() returns both.
    /// This test demonstrates the combined bounds and graph retrieval feature.
    #[test]
    fn test_app_get_adjusted_bounds_and_app_graph() {
        let mut app = create_test_app();

        let (bounds, app_graph) = app.get_adjusted_bounds_and_app_graph(None);

        assert!(bounds.contains_key("layout1"));
        assert!(app_graph.graphs.contains_key("layout1"));
    }

    // === App Graph Generation Tests ===

    /// Tests that App::generate_graph() creates graph for all layouts.
    /// This test demonstrates the graph generation feature.
    #[test]
    fn test_app_generate_graph() {
        let mut app = create_test_app();

        let app_graph = app.generate_graph();
        assert!(app_graph.graphs.contains_key("layout1"));

        let graph = &app_graph.graphs["layout1"];
        assert_eq!(graph.node_count(), 2); // muxbox1 and muxbox2
    }

    /// Tests that App::generate_graph() caches graph correctly.
    /// This test demonstrates the graph caching feature.
    #[test]
    fn test_app_generate_graph_caching() {
        let mut app = create_test_app();

        // First call should generate graph
        let graph1 = app.generate_graph();

        // Second call should return cached graph
        let graph2 = app.generate_graph();
        assert_eq!(graph1, graph2);
    }

    // === App MuxBox Replacement Tests ===

    /// Tests that App::replace_muxbox() replaces muxboxes correctly.
    /// This test demonstrates the muxbox replacement feature.
    #[test]
    fn test_app_replace_muxbox() {
        let mut app = create_test_app();

        // Create a replacement muxbox
        let mut replacement_muxbox = create_test_muxbox("muxbox1");
        replacement_muxbox.title = Some("Replaced MuxBox".to_string());

        app.replace_muxbox(replacement_muxbox);

        // Verify the muxbox was replaced
        let replaced_muxbox = app.get_muxbox_by_id("muxbox1").unwrap();
        assert_eq!(replaced_muxbox.title, Some("Replaced MuxBox".to_string()));
    }

    /// Tests that App::replace_muxbox() handles nonexistent muxboxes.
    /// This test demonstrates edge case handling in muxbox replacement.
    #[test]
    fn test_app_replace_muxbox_nonexistent() {
        let mut app = create_test_app();

        // Create a replacement muxbox with nonexistent ID
        let replacement_muxbox = create_test_muxbox("nonexistent");

        // This should not panic
        app.replace_muxbox(replacement_muxbox);

        // Original muxboxes should be unchanged
        let original_muxbox = app.get_muxbox_by_id("muxbox1").unwrap();
        assert_eq!(
            original_muxbox.title,
            Some("Test MuxBox muxbox1".to_string())
        );
    }

    // === App Clone Tests ===

    /// Tests that App implements Clone correctly.
    /// This test demonstrates App cloning behavior.
    #[test]
    fn test_app_clone() {
        let app1 = create_test_app();
        let app2 = app1.clone();

        assert_eq!(app1.layouts.len(), app2.layouts.len());
        assert_eq!(app1.layouts[0].id, app2.layouts[0].id);
        assert_eq!(app1.libs, app2.libs);
        assert_eq!(app1.on_keypress, app2.on_keypress);
    }

    /// Tests that App cloning includes all nested structures.
    /// This test demonstrates comprehensive App cloning.
    #[test]
    fn test_app_clone_comprehensive() {
        let mut app1 = create_test_app();
        app1.libs = Some(vec!["lib1.sh".to_string(), "lib2.sh".to_string()]);

        let mut keypress_map = HashMap::new();
        keypress_map.insert("ctrl+c".to_string(), vec!["exit".to_string()]);
        app1.on_keypress = Some(keypress_map);

        let app2 = app1.clone();

        assert_eq!(app1.libs, app2.libs);
        assert_eq!(app1.on_keypress, app2.on_keypress);
        assert_eq!(
            app1.layouts[0].children.as_ref().unwrap().len(),
            app2.layouts[0].children.as_ref().unwrap().len()
        );
    }

    // === App Hash Tests ===

    /// Tests that App implements Hash correctly.
    /// This test demonstrates App hashing behavior.
    #[test]
    fn test_app_hash() {
        let app1 = create_test_app();
        let app2 = create_test_app();
        let mut app3 = create_test_app();
        app3.layouts[0].id = "different".to_string();

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        app1.hash(&mut hasher1);
        app2.hash(&mut hasher2);
        app3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === App PartialEq Tests ===

    /// Tests that App implements PartialEq correctly.
    /// This test demonstrates App equality comparison.
    #[test]
    fn test_app_equality() {
        let app1 = create_test_app();
        let app2 = create_test_app();
        let mut app3 = create_test_app();
        app3.layouts[0].id = "different".to_string();

        assert_eq!(app1, app2);
        assert_ne!(app1, app3);
    }

    // === AppContext Tests ===

    /// Tests that AppContext::new() creates context with validated app.
    /// This test demonstrates AppContext construction behavior.
    #[test]
    fn test_app_context_new() {
        let app = create_test_app();
        let config = Config::new(60);
        let app_context = AppContext::new(app, config);

        assert_eq!(app_context.config.frame_delay, 60);
        assert_eq!(app_context.app.layouts.len(), 1);
    }

    /// Tests that AppContext implements Clone correctly.
    /// This test demonstrates AppContext cloning behavior.
    #[test]
    fn test_app_context_clone() {
        let app_context1 = create_test_app_context();
        let app_context2 = app_context1.clone();

        assert_eq!(app_context1.config, app_context2.config);
        assert_eq!(app_context1.app, app_context2.app);
    }

    /// Tests that AppContext implements Hash correctly.
    /// This test demonstrates AppContext hashing behavior.
    #[test]
    fn test_app_context_hash() {
        let app_context1 = create_test_app_context();
        let app_context2 = create_test_app_context();

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        app_context1.hash(&mut hasher1);
        app_context2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    /// Tests that AppContext implements PartialEq correctly.
    /// This test demonstrates AppContext equality comparison.
    #[test]
    fn test_app_context_equality() {
        let app_context1 = create_test_app_context();
        let app_context2 = create_test_app_context();

        assert_eq!(app_context1, app_context2);
    }

    // === AppGraph Tests ===

    /// Tests that AppGraph::new() creates an empty graph.
    /// This test demonstrates AppGraph construction behavior.
    #[test]
    fn test_app_graph_new() {
        let app_graph = AppGraph::new();
        assert_eq!(app_graph.graphs.len(), 0);
        assert_eq!(app_graph.node_maps.len(), 0);
    }

    /// Tests that AppGraph::default() creates an empty graph.
    /// This test demonstrates AppGraph default behavior.
    #[test]
    fn test_app_graph_default() {
        let app_graph = AppGraph::default();
        assert_eq!(app_graph.graphs.len(), 0);
        assert_eq!(app_graph.node_maps.len(), 0);
    }

    /// Tests that AppGraph::add_layout() adds layout to graph.
    /// This test demonstrates the layout addition feature.
    #[test]
    fn test_app_graph_add_layout() {
        let layout = create_test_layout("test", Some(vec![create_test_muxbox("muxbox1")]));
        let mut app_graph = AppGraph::new();

        app_graph.add_layout(&layout);

        assert!(app_graph.graphs.contains_key("test"));
        assert!(app_graph.node_maps.contains_key("test"));
        assert_eq!(app_graph.graphs["test"].node_count(), 1);
    }

    /// Tests that AppGraph::get_layout_muxbox_by_id() finds muxboxes.
    /// This test demonstrates the layout-specific muxbox retrieval feature.
    #[test]
    fn test_app_graph_get_layout_muxbox_by_id() {
        let layout = create_test_layout("test", Some(vec![create_test_muxbox("muxbox1")]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout);

        let muxbox = app_graph.get_layout_muxbox_by_id("test", "muxbox1");
        assert!(muxbox.is_some());
        assert_eq!(muxbox.unwrap().id, "muxbox1");

        let not_found = app_graph.get_layout_muxbox_by_id("test", "nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that AppGraph::get_muxbox_by_id() finds muxboxes across layouts.
    /// This test demonstrates the cross-layout muxbox retrieval feature.
    #[test]
    fn test_app_graph_get_muxbox_by_id() {
        let layout1 = create_test_layout("layout1", Some(vec![create_test_muxbox("muxbox1")]));
        let layout2 = create_test_layout("layout2", Some(vec![create_test_muxbox("muxbox2")]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout1);
        app_graph.add_layout(&layout2);

        let muxbox1 = app_graph.get_muxbox_by_id("muxbox1");
        assert!(muxbox1.is_some());
        assert_eq!(muxbox1.unwrap().id, "muxbox1");

        let muxbox2 = app_graph.get_muxbox_by_id("muxbox2");
        assert!(muxbox2.is_some());
        assert_eq!(muxbox2.unwrap().id, "muxbox2");
    }

    /// Tests that AppGraph::get_children() returns child muxboxes.
    /// This test demonstrates the children retrieval feature.
    #[test]
    fn test_app_graph_get_children() {
        let child_muxbox = create_test_muxbox("child");
        let mut parent_muxbox = create_test_muxbox("parent");
        parent_muxbox.children = Some(vec![child_muxbox]);

        let layout = create_test_layout("test", Some(vec![parent_muxbox]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout);

        let children = app_graph.get_children("test", "parent");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child");
    }

    /// Tests that AppGraph::get_parent() returns parent muxboxes.
    /// This test demonstrates the parent retrieval feature.
    #[test]
    fn test_app_graph_get_parent() {
        let child_muxbox = create_test_muxbox("child");
        let mut parent_muxbox = create_test_muxbox("parent");
        parent_muxbox.children = Some(vec![child_muxbox]);

        let layout = create_test_layout("test", Some(vec![parent_muxbox]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout);

        let parent = app_graph.get_parent("test", "child");
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().id, "parent");
    }

    /// Tests that AppGraph implements Hash correctly.
    /// This test demonstrates AppGraph hashing behavior.
    #[test]
    fn test_app_graph_hash() {
        let layout = create_test_layout("test", Some(vec![create_test_muxbox("muxbox1")]));
        let mut app_graph1 = AppGraph::new();
        let mut app_graph2 = AppGraph::new();
        app_graph1.add_layout(&layout);
        app_graph2.add_layout(&layout);

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        app_graph1.hash(&mut hasher1);
        app_graph2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    /// Tests that AppGraph implements PartialEq correctly.
    /// This test demonstrates AppGraph equality comparison.
    #[test]
    fn test_app_graph_equality() {
        let layout = create_test_layout("test", Some(vec![create_test_muxbox("muxbox1")]));
        let mut app_graph1 = AppGraph::new();
        let mut app_graph2 = AppGraph::new();
        app_graph1.add_layout(&layout);
        app_graph2.add_layout(&layout);

        assert_eq!(app_graph1, app_graph2);
    }

    // === Integration Tests (from original test suite) ===

    #[test]
    fn test_layout_and_muxboxes_addition() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        assert!(app_graph.graphs.contains_key("dashboard"));
        let graph = &app_graph.graphs["dashboard"];
        assert_eq!(
            graph.node_count(),
            9,
            "Should include all muxboxes and sub-muxboxes"
        );
    }

    #[test]
    fn test_get_muxbox_by_id() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        let muxboxes = [
            "header",
            "title",
            "time",
            "cpu",
            "memory",
            "log",
            "log_input",
            "log_output",
            "footer",
        ];
        for &muxbox_id in muxboxes.iter() {
            let muxbox = app_graph.get_muxbox_by_id(muxbox_id);
            assert!(
                muxbox.is_some(),
                "MuxBox with ID {} should exist",
                muxbox_id
            );
        }
    }

    #[test]
    fn test_get_children() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        let children = app_graph.get_children("dashboard", "header");
        assert_eq!(children.len(), 2, "Header should have exactly 2 children");
        assert!(
            children.iter().any(|&p| p.id == "title"),
            "Title should be a child of header"
        );
        assert!(
            children.iter().any(|&p| p.id == "time"),
            "Time should be a child of header"
        );
    }

    #[test]
    fn test_get_parent() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        let parent = app_graph.get_parent("dashboard", "title");
        assert!(parent.is_some(), "Parent should exist for 'title'");
        assert_eq!(
            parent.unwrap().id,
            "header",
            "Parent of 'title' should be 'header'"
        );
    }

    #[test]
    fn test_app_graph_clone() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        let cloned_graph = app_graph.clone();
        assert_eq!(app_graph, cloned_graph);
    }

    // === Load App from YAML Tests ===

    /// Tests that load_app_from_yaml() loads app correctly.
    /// This test demonstrates the YAML loading feature.
    #[test]
    fn test_load_app_from_yaml() {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let dashboard_path = current_dir.join("layouts/tests.yaml");

        let result = load_app_from_yaml(dashboard_path.to_str().unwrap());
        assert!(result.is_ok());

        let app = result.unwrap();
        assert_eq!(app.layouts.len(), 1);
        assert_eq!(app.layouts[0].id, "dashboard");
    }

    /// Tests that load_app_from_yaml() handles invalid files.
    /// This test demonstrates error handling in YAML loading.
    #[test]
    fn test_load_app_from_yaml_invalid_file() {
        let result = load_app_from_yaml("nonexistent.yaml");
        assert!(result.is_err());
    }

    /// Tests SchemaValidator integration with load_app_from_yaml.
    /// This test demonstrates the comprehensive validation system integration.
    #[test]
    fn test_load_app_from_yaml_with_schema_validation() {
        use std::fs;
        use std::io::Write;

        // Create a temporary invalid YAML file for testing validation
        let temp_file = "/tmp/boxmux_test_invalid.yaml";
        let invalid_yaml_content = r#"
app:
  layouts:
    - id: 'layout1'
      root: true
      children:
        - id: 'muxbox1'
          position:
            x1: 0%
            y1: 0%
            x2: 50%
            y2: 50%
        - id: 'muxbox1'  # Duplicate ID - should fail validation
          position:
            x1: 50%
            y1: 0%
            x2: 100%
            y2: 50%
"#;

        // Write the invalid content to temp file
        let mut file = fs::File::create(temp_file).expect("Failed to create temp file");
        file.write_all(invalid_yaml_content.as_bytes())
            .expect("Failed to write temp file");

        // Test that SchemaValidator catches the duplicate ID
        let result = load_app_from_yaml(temp_file);
        assert!(
            result.is_err(),
            "SchemaValidator should catch duplicate IDs"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Duplicate ID 'muxbox1' found in muxboxes"),
            "Error message should mention duplicate ID: {}",
            error_msg
        );

        // Clean up
        let _ = fs::remove_file(temp_file);

        // Test valid YAML passes validation
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let valid_dashboard_path = current_dir.join("layouts/tests.yaml");

        let valid_result = load_app_from_yaml(valid_dashboard_path.to_str().unwrap());
        assert!(
            valid_result.is_ok(),
            "Valid YAML should pass SchemaValidator"
        );

        let app = valid_result.unwrap();
        assert_eq!(app.layouts.len(), 1);
        assert_eq!(app.layouts[0].id, "dashboard");
    }

    /// Tests SchemaValidator integration with multiple root layouts error.
    /// This test demonstrates schema validation for structural errors.
    #[test]
    fn test_load_app_from_yaml_multiple_root_layouts_error() {
        use std::fs;
        use std::io::Write;

        let temp_file = "/tmp/boxmux_test_multiple_roots.yaml";
        let multiple_roots_yaml = r#"
app:
  layouts:
    - id: 'layout1'
      root: true
      children:
        - id: 'muxbox1'
          position:
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 100%
    - id: 'layout2'
      root: true  # Second root - should fail validation
      children:
        - id: 'muxbox2'
          position:
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 100%
"#;

        let mut file = fs::File::create(temp_file).expect("Failed to create temp file");
        file.write_all(multiple_roots_yaml.as_bytes())
            .expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file);
        assert!(
            result.is_err(),
            "SchemaValidator should catch multiple root layouts"
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Multiple root layouts detected"),
            "Error message should mention multiple root layouts: {}",
            error_msg
        );

        // Clean up
        let _ = fs::remove_file(temp_file);
    }
}

// F0200: Complete YAML Persistence System - Live Synchronization

/// Save complete application state to YAML file
pub fn save_complete_state_to_yaml(
    yaml_path: &str,
    app_context: &AppContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    // Create a complete app structure for serialization
    let serializable_app = SerializableApp {
        app: app_context.app.clone(),
    };

    // Convert to YAML with proper formatting (skip wrapper)
    let yaml_content = serializable_app.to_yaml_string()?;

    // Atomic write - write to temp file then rename
    let temp_path = format!("{}.tmp", yaml_path);
    fs::write(&temp_path, yaml_content)?;
    fs::rename(&temp_path, yaml_path)?;

    log::debug!("Saved complete application state to YAML: {}", yaml_path);
    Ok(())
}

/// Save active layout state to YAML file
pub fn save_active_layout_to_yaml(
    yaml_path: &str,
    active_layout_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_yaml::Value;
    use std::fs;

    // Read current YAML content
    let yaml_content = fs::read_to_string(yaml_path)?;
    let mut yaml_value: Value = serde_yaml::from_str(&yaml_content)?;

    // Update active layout in all layouts
    if let Some(root_map) = yaml_value.as_mapping_mut() {
        if let Some(app_map) = root_map.get_mut(&Value::String("app".to_string())) {
            if let Value::Mapping(app_map) = app_map {
                if let Some(layouts_seq) = app_map.get_mut(&Value::String("layouts".to_string())) {
                    if let Value::Sequence(layouts_seq) = layouts_seq {
                        for layout_value in layouts_seq.iter_mut() {
                            if let Value::Mapping(layout_map) = layout_value {
                                if let Some(Value::String(layout_id)) =
                                    layout_map.get(&Value::String("id".to_string()))
                                {
                                    let is_active = layout_id == active_layout_id;
                                    layout_map.insert(
                                        Value::String("active".to_string()),
                                        Value::Bool(is_active),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Atomic write
    let temp_path = format!("{}.tmp", yaml_path);
    let updated_yaml = serde_yaml::to_string(&yaml_value)?;
    fs::write(&temp_path, updated_yaml)?;
    fs::rename(&temp_path, yaml_path)?;

    log::info!(
        "Updated active layout to '{}' in YAML: {}",
        active_layout_id,
        yaml_path
    );
    Ok(())
}

// F0190: YAML persistence functions for live muxbox resizing
pub fn save_muxbox_bounds_to_yaml(
    yaml_path: &str,
    muxbox_id: &str,
    new_bounds: &crate::InputBounds,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_yaml::{self, Value};
    use std::fs;

    // Read the current YAML file
    let yaml_content = fs::read_to_string(yaml_path)?;
    let mut yaml_value: Value = serde_yaml::from_str(&yaml_content)?;

    // Find and update the muxbox bounds
    update_muxbox_bounds_recursive(&mut yaml_value, muxbox_id, new_bounds)?;

    // Atomic write
    let temp_path = format!("{}.tmp", yaml_path);
    let updated_yaml = serde_yaml::to_string(&yaml_value)?;
    fs::write(&temp_path, updated_yaml)?;
    fs::rename(&temp_path, yaml_path)?;

    log::debug!("Updated muxbox {} bounds in YAML: {}", muxbox_id, yaml_path);
    Ok(())
}

/// Save muxbox content changes to YAML
pub fn save_muxbox_content_to_yaml(
    yaml_path: &str,
    muxbox_id: &str,
    new_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_yaml::Value;
    use std::fs;

    let yaml_content = fs::read_to_string(yaml_path)?;
    let mut yaml_value: Value = serde_yaml::from_str(&yaml_content)?;

    update_muxbox_field_recursive(
        &mut yaml_value,
        muxbox_id,
        "content",
        &Value::String(new_content.to_string()),
    )?;

    let temp_path = format!("{}.tmp", yaml_path);
    let updated_yaml = serde_yaml::to_string(&yaml_value)?;
    fs::write(&temp_path, updated_yaml)?;
    fs::rename(&temp_path, yaml_path)?;

    log::debug!(
        "Updated muxbox {} content in YAML: {}",
        muxbox_id,
        yaml_path
    );
    Ok(())
}

/// Save muxbox scroll position to YAML
pub fn save_muxbox_scroll_to_yaml(
    yaml_path: &str,
    muxbox_id: &str,
    scroll_x: usize,
    scroll_y: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use serde_yaml::Value;
    use std::fs;

    let yaml_content = fs::read_to_string(yaml_path)?;
    let mut yaml_value: Value = serde_yaml::from_str(&yaml_content)?;

    update_muxbox_field_recursive(
        &mut yaml_value,
        muxbox_id,
        "scroll_x",
        &Value::Number(serde_yaml::Number::from(scroll_x)),
    )?;
    update_muxbox_field_recursive(
        &mut yaml_value,
        muxbox_id,
        "scroll_y",
        &Value::Number(serde_yaml::Number::from(scroll_y)),
    )?;

    let temp_path = format!("{}.tmp", yaml_path);
    let updated_yaml = serde_yaml::to_string(&yaml_value)?;
    fs::write(&temp_path, updated_yaml)?;
    fs::rename(&temp_path, yaml_path)?;

    log::debug!(
        "Updated muxbox {} scroll position in YAML: {}",
        muxbox_id,
        yaml_path
    );
    Ok(())
}

/// Generic function to update any muxbox field in YAML
fn update_muxbox_field_recursive(
    value: &mut serde_yaml::Value,
    target_muxbox_id: &str,
    field_name: &str,
    new_value: &serde_yaml::Value,
) -> Result<bool, Box<dyn std::error::Error>> {
    use serde_yaml::Value;
    match value {
        Value::Mapping(map) => {
            // Check if this is the target muxbox
            if let Some(Value::String(id)) = map.get(&Value::String("id".to_string())) {
                if id == target_muxbox_id {
                    map.insert(Value::String(field_name.to_string()), new_value.clone());
                    return Ok(true);
                }
            }

            // Recursively search in all fields
            for (_, child_value) in map.iter_mut() {
                if update_muxbox_field_recursive(
                    child_value,
                    target_muxbox_id,
                    field_name,
                    new_value,
                )? {
                    return Ok(true);
                }
            }
        }
        Value::Sequence(seq) => {
            for child_value in seq.iter_mut() {
                if update_muxbox_field_recursive(
                    child_value,
                    target_muxbox_id,
                    field_name,
                    new_value,
                )? {
                    return Ok(true);
                }
            }
        }
        _ => {}
    }
    Ok(false)
}

pub fn update_muxbox_bounds_recursive(
    value: &mut serde_yaml::Value,
    target_muxbox_id: &str,
    new_bounds: &crate::InputBounds,
) -> Result<bool, Box<dyn std::error::Error>> {
    use serde_yaml::Value;
    match value {
        Value::Mapping(map) => {
            // Check if this is the muxbox we're looking for
            if let Some(Value::String(id)) = map.get(&Value::String("id".to_string())) {
                if id == target_muxbox_id {
                    // Update the bounds in the position field
                    if let Some(position_value) =
                        map.get_mut(&Value::String("position".to_string()))
                    {
                        if let Value::Mapping(position_map) = position_value {
                            position_map.insert(
                                Value::String("x1".to_string()),
                                Value::String(new_bounds.x1.clone()),
                            );
                            position_map.insert(
                                Value::String("y1".to_string()),
                                Value::String(new_bounds.y1.clone()),
                            );
                            position_map.insert(
                                Value::String("x2".to_string()),
                                Value::String(new_bounds.x2.clone()),
                            );
                            position_map.insert(
                                Value::String("y2".to_string()),
                                Value::String(new_bounds.y2.clone()),
                            );
                            return Ok(true);
                        }
                    }
                    // If no position field exists, create one
                    let mut position_map = serde_yaml::Mapping::new();
                    position_map.insert(
                        Value::String("x1".to_string()),
                        Value::String(new_bounds.x1.clone()),
                    );
                    position_map.insert(
                        Value::String("y1".to_string()),
                        Value::String(new_bounds.y1.clone()),
                    );
                    position_map.insert(
                        Value::String("x2".to_string()),
                        Value::String(new_bounds.x2.clone()),
                    );
                    position_map.insert(
                        Value::String("y2".to_string()),
                        Value::String(new_bounds.y2.clone()),
                    );
                    map.insert(
                        Value::String("position".to_string()),
                        Value::Mapping(position_map),
                    );
                    return Ok(true);
                }
            }

            // Recursively search in children and other mappings
            for (_, child_value) in map.iter_mut() {
                if update_muxbox_bounds_recursive(child_value, target_muxbox_id, new_bounds)? {
                    return Ok(true);
                }
            }
        }
        Value::Sequence(seq) => {
            // Search through sequences (like children arrays)
            for item in seq.iter_mut() {
                if update_muxbox_bounds_recursive(item, target_muxbox_id, new_bounds)? {
                    return Ok(true);
                }
            }
        }
        _ => {
            // Other value types don't contain muxboxes
        }
    }

    Ok(false)
}

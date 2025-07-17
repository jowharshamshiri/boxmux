use crate::model::panel::*;
use crate::{model::layout::Layout, Bounds};

use std::fs::File;
use std::io::Read;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::{calculate_bounds_map, Config, FieldUpdate, Updatable};
use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

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
    #[serde(skip)]
    app_graph: Option<AppGraph>,
    #[serde(skip)]
    pub adjusted_bounds: Option<HashMap<String, HashMap<String, Bounds>>>,
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.layouts == other.layouts
            && self.on_keypress == other.on_keypress
            && self.app_graph == other.app_graph
            && self.adjusted_bounds == other.adjusted_bounds
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
            app_graph: None,
            adjusted_bounds: None,
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

    pub fn get_panel_by_id(&self, id: &str) -> Option<&Panel> {
        for layout in &self.layouts {
            if let Some(panel) = layout.get_panel_by_id(id) {
                return Some(panel);
            }
        }
        None
    }

    pub fn get_panel_by_id_mut(&mut self, id: &str) -> Option<&mut Panel> {
        for layout in &mut self.layouts {
            if let Some(panel) = layout.get_panel_by_id_mut(id) {
                return Some(panel);
            }
        }
        None
    }

    pub fn validate(&mut self) {
        if let Err(e) = validate_app(self) {
            panic!("Validation error: {}", e);
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

    pub fn replace_panel(&mut self, panel: Panel) {
        for layout in &mut self.layouts {
            if let Some(replaced) = layout.replace_panel_recursive(&panel) {
                if replaced {
                    return;
                }
            }
        }
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            layouts: self.layouts.to_vec(),
            libs: self.libs.clone(),
            on_keypress: self.on_keypress.clone(),
            app_graph: self.app_graph.clone(),
            adjusted_bounds: self.adjusted_bounds.clone(),
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
    graphs: HashMap<String, DiGraph<Panel, ()>>,
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
            for panel in children {
                self.add_panel_recursively(
                    &mut graph,
                    &mut node_map,
                    panel.clone(),
                    None,
                    &layout.id,
                );
            }
        }

        self.graphs.insert(layout.id.clone(), graph);
        self.node_maps.insert(layout.id.clone(), node_map);
    }

    fn add_panel_recursively(
        &self,
        graph: &mut DiGraph<Panel, ()>,
        node_map: &mut HashMap<String, NodeIndex>,
        mut panel: Panel,
        parent_id: Option<String>,
        parent_layout_id: &str,
    ) {
        panel.parent_layout_id = Some(parent_layout_id.to_string());
        let panel_id = panel.id.clone();
        let node_index = graph.add_node(panel.clone());
        node_map.insert(panel_id.clone(), node_index);

        if let Some(parent_id) = panel.parent_id.clone() {
            if let Some(&parent_index) = node_map.get(&parent_id) {
                graph.add_edge(parent_index, node_index, ());
            }
        } else if let Some(parent_id) = parent_id {
            if let Some(&parent_index) = node_map.get(&parent_id) {
                graph.add_edge(parent_index, node_index, ());
            }
        }

        if let Some(children) = panel.children {
            for mut child in children {
                child.parent_id = Some(panel_id.clone());
                self.add_panel_recursively(
                    graph,
                    node_map,
                    child,
                    Some(panel_id.clone()),
                    parent_layout_id,
                );
            }
        }
    }

    pub fn get_layout_panel_by_id(&self, layout_id: &str, panel_id: &str) -> Option<&Panel> {
        self.node_maps.get(layout_id).and_then(|node_map| {
            node_map.get(panel_id).and_then(|&index| {
                self.graphs
                    .get(layout_id)
                    .and_then(|graph| graph.node_weight(index))
            })
        })
    }

    pub fn get_panel_by_id(&self, panel_id: &str) -> Option<&Panel> {
        for (layout_id, node_map) in &self.node_maps {
            if let Some(&index) = node_map.get(panel_id) {
                return self
                    .graphs
                    .get(layout_id)
                    .and_then(|graph| graph.node_weight(index));
            }
        }
        None
    }

    pub fn get_children(&self, layout_id: &str, panel_id: &str) -> Vec<&Panel> {
        if let Some(node_map) = self.node_maps.get(layout_id) {
            if let Some(&index) = node_map.get(panel_id) {
                return self.graphs[layout_id]
                    .edges_directed(index, petgraph::Direction::Outgoing)
                    .map(|edge| self.graphs[layout_id].node_weight(edge.target()).unwrap())
                    .collect();
            }
        }
        Vec::new()
    }

    pub fn get_layout_children(&self, layout_id: &str) -> Vec<&Panel> {
        if let Some(node_map) = self.node_maps.get(layout_id) {
            let root_node = node_map.get(layout_id).unwrap();
            return self.graphs[layout_id]
                .edges_directed(*root_node, petgraph::Direction::Outgoing)
                .map(|edge| self.graphs[layout_id].node_weight(edge.target()).unwrap())
                .collect();
        }
        Vec::new()
    }

    pub fn get_parent(&self, layout_id: &str, panel_id: &str) -> Option<&Panel> {
        if let Some(node_map) = self.node_maps.get(layout_id) {
            if let Some(&index) = node_map.get(panel_id) {
                return self.graphs[layout_id]
                    .edges_directed(index, petgraph::Direction::Incoming)
                    .next()
                    .and_then(|edge| self.graphs[layout_id].node_weight(edge.source()));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct AppContext {
    pub app: App,
    pub config: Config,
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
        AppContext { app, config }
    }
}

impl Clone for AppContext {
    fn clone(&self) -> Self {
        AppContext {
            app: self.app.clone(),
            config: self.config.clone(),
        }
    }
}

impl Hash for AppContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app.hash(state);
        self.config.hash(state);
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
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let root_result: Result<TemplateRoot, _> = serde_yaml::from_str(&contents);

    let mut app = match root_result {
        Ok(root) => root.app,
        Err(_) => {
            // If deserialization into Root fails, try to deserialize directly into App
            serde_yaml::from_str(&contents)?
        }
    };

    // Validate the app configuration
    match validate_app(&mut app) {
        Ok(_) => {
            // log::info!("Loaded app from file: {}", file_path);
            // log::debug!("App: {:#?}", app);
            Ok(app)
        }
        Err(validation_error) => {
            Err(format!("Configuration validation error: {}", validation_error).into())
        }
    }
}

fn validate_app(app: &mut App) -> Result<(), String> {
    if app.layouts.is_empty() {
        return Err("No layouts defined in the application. Please add at least one layout with panels.".to_string());
    }

    fn set_parent_ids(panel: &mut Panel, parent_layout_id: &str, parent_id: Option<String>) {
        panel.parent_layout_id = Some(parent_layout_id.to_string());
        panel.parent_id = parent_id;

        if let Some(ref mut children) = panel.children {
            for child in children {
                set_parent_ids(child, parent_layout_id, Some(panel.id.clone()));
            }
        }
    }

    let mut id_set = HashSet::new();
    let mut root_layout_id: Option<String> = None;
    let mut root_count = 0;

    for layout in &mut app.layouts {
        let mut layout_clone = layout.clone();
        let panels_in_tab_order = layout_clone.get_panels_in_tab_order();

        // Check for unique IDs
        if let Err(e) = check_unique_ids(layout, &mut id_set) {
            return Err(e);
        }

        // Check for multiple root layouts
        if layout.root.unwrap_or(false) {
            root_count += 1;
            if root_count > 1 {
                return Err("Multiple root layouts detected. Only one layout can be marked as 'root: true'.".to_string());
            }
            root_layout_id = Some(layout.id.clone());
        }

        if layout.children.is_none() {
            continue;
        }

        // Set up parent relationships and defaults
        for panel in layout.children.as_mut().unwrap() {
            set_parent_ids(panel, &layout.id, None);
            if !panels_in_tab_order.is_empty() && panel.id == panels_in_tab_order[0].id {
                panel.selected = Some(true);
            }
            if let Some(choices) = &mut panel.choices {
                if !choices.is_empty() {
                    choices[0].selected = true;
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

fn check_unique_ids(
    layout: &Layout,
    id_set: &mut HashSet<String>,
) -> Result<(), String> {
    if let Some(children) = &layout.children {
        for panel in children {
            check_panel_ids(panel, id_set)?;
        }
    }
    Ok(())
}

fn check_panel_ids(
    panel: &Panel,
    id_set: &mut HashSet<String>,
) -> Result<(), String> {
    if !id_set.insert(panel.id.clone()) {
        return Err(format!("Duplicate panel ID found: '{}'. All panel IDs must be unique across the entire application.", panel.id));
    }
    if let Some(children) = &panel.children {
        for child in children {
            check_panel_ids(child, id_set)?;
        }
    }
    if let Some(choices) = &panel.choices {
        for choice in choices {
            if !id_set.insert(choice.id.clone()) {
                return Err(format!("Duplicate choice ID found: '{}'. All choice IDs must be unique across the entire application.", choice.id));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::panel::Panel;
    use crate::model::layout::Layout;
    use crate::model::common::InputBounds;
    use std::collections::HashMap;

    // === Helper Functions ===

    /// Creates a basic test panel with the given id.
    /// This helper demonstrates how to create a Panel for App testing.
    fn create_test_panel(id: &str) -> Panel {
        Panel {
            id: id.to_string(),
            title: Some(format!("Test Panel {}", id)),
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
    fn create_test_layout(id: &str, children: Option<Vec<Panel>>) -> Layout {
        Layout {
            id: id.to_string(),
            title: Some(format!("Test Layout {}", id)),
            children,
            root: Some(false),
            active: Some(false),
            ..Default::default()
        }
    }

    /// Creates a test App with basic layouts and panels.
    /// This helper demonstrates how to create an App for testing.
    fn create_test_app() -> App {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout1 = create_test_layout("layout1", Some(vec![panel1, panel2]));
        
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
        assert_eq!(verified_layout.unwrap().title, Some("Modified Layout".to_string()));
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
        assert_eq!(verified_layout.unwrap().title, Some("Modified Root".to_string()));
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
        assert_eq!(verified_layout.unwrap().title, Some("Modified Active".to_string()));
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

    // === App Panel Management Tests ===

    /// Tests that App::get_panel_by_id() finds panels across layouts.
    /// This test demonstrates the cross-layout panel retrieval feature.
    #[test]
    fn test_app_get_panel_by_id() {
        let app = create_test_app();
        
        let found_panel = app.get_panel_by_id("panel1");
        assert!(found_panel.is_some());
        assert_eq!(found_panel.unwrap().id, "panel1");
        
        let not_found = app.get_panel_by_id("nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that App::get_panel_by_id_mut() finds and allows modification.
    /// This test demonstrates the mutable cross-layout panel retrieval feature.
    #[test]
    fn test_app_get_panel_by_id_mut() {
        let mut app = create_test_app();
        
        let found_panel = app.get_panel_by_id_mut("panel1");
        assert!(found_panel.is_some());
        
        // Modify the panel
        found_panel.unwrap().title = Some("Modified Panel".to_string());
        
        // Verify the modification
        let verified_panel = app.get_panel_by_id("panel1");
        assert_eq!(verified_panel.unwrap().title, Some("Modified Panel".to_string()));
    }

    /// Tests that App::get_panel_by_id_mut() handles empty app.
    /// This test demonstrates edge case handling in mutable panel retrieval.
    #[test]
    fn test_app_get_panel_by_id_mut_empty() {
        let mut app = App::new();
        
        let found_panel = app.get_panel_by_id_mut("nonexistent");
        assert!(found_panel.is_none());
    }

    // === App Validation Tests ===

    /// Tests that App::validate() sets up parent relationships correctly.
    /// This test demonstrates the app validation feature.
    #[test]
    fn test_app_validate() {
        let mut app = create_test_app();
        
        // Before validation, parent relationships should not be set
        let panel = app.get_panel_by_id("panel1").unwrap();
        assert_eq!(panel.parent_layout_id, None);
        assert_eq!(panel.parent_id, None);
        
        app.validate();
        
        // After validation, parent relationships should be set
        let panel = app.get_panel_by_id("panel1").unwrap();
        assert_eq!(panel.parent_layout_id, Some("layout1".to_string()));
        assert_eq!(panel.parent_id, None); // Top-level panel has no parent panel
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
    #[should_panic(expected = "No layouts defined in the application.")]
    fn test_app_validate_empty_panics() {
        let mut app = App::new();
        app.validate();
    }

    /// Tests that App::validate() panics with duplicate IDs.
    /// This test demonstrates the duplicate ID validation feature.
    #[test]
    #[should_panic(expected = "Duplicate ID found: panel1")]
    fn test_app_validate_duplicate_ids_panics() {
        let mut app = App::new();
        
        // Create two panels with the same ID
        let panel1a = create_test_panel("panel1");
        let panel1b = create_test_panel("panel1"); // Duplicate ID
        
        let layout = create_test_layout("layout1", Some(vec![panel1a, panel1b]));
        app.layouts.push(layout);
        
        app.validate();
    }

    /// Tests that App::validate() panics with multiple root layouts.
    /// This test demonstrates the multiple root layout validation feature.
    #[test]
    #[should_panic(expected = "Multiple root layouts detected, which is not allowed.")]
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
        assert!(layout_bounds.contains_key("panel1"));
        assert!(layout_bounds.contains_key("panel2"));
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
        assert_eq!(graph.node_count(), 2); // panel1 and panel2
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

    // === App Panel Replacement Tests ===

    /// Tests that App::replace_panel() replaces panels correctly.
    /// This test demonstrates the panel replacement feature.
    #[test]
    fn test_app_replace_panel() {
        let mut app = create_test_app();
        
        // Create a replacement panel
        let mut replacement_panel = create_test_panel("panel1");
        replacement_panel.title = Some("Replaced Panel".to_string());
        
        app.replace_panel(replacement_panel);
        
        // Verify the panel was replaced
        let replaced_panel = app.get_panel_by_id("panel1").unwrap();
        assert_eq!(replaced_panel.title, Some("Replaced Panel".to_string()));
    }

    /// Tests that App::replace_panel() handles nonexistent panels.
    /// This test demonstrates edge case handling in panel replacement.
    #[test]
    fn test_app_replace_panel_nonexistent() {
        let mut app = create_test_app();
        
        // Create a replacement panel with nonexistent ID
        let replacement_panel = create_test_panel("nonexistent");
        
        // This should not panic
        app.replace_panel(replacement_panel);
        
        // Original panels should be unchanged
        let original_panel = app.get_panel_by_id("panel1").unwrap();
        assert_eq!(original_panel.title, Some("Test Panel panel1".to_string()));
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
        assert_eq!(app1.layouts[0].children.as_ref().unwrap().len(),
                   app2.layouts[0].children.as_ref().unwrap().len());
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
        let layout = create_test_layout("test", Some(vec![create_test_panel("panel1")]));
        let mut app_graph = AppGraph::new();
        
        app_graph.add_layout(&layout);
        
        assert!(app_graph.graphs.contains_key("test"));
        assert!(app_graph.node_maps.contains_key("test"));
        assert_eq!(app_graph.graphs["test"].node_count(), 1);
    }

    /// Tests that AppGraph::get_layout_panel_by_id() finds panels.
    /// This test demonstrates the layout-specific panel retrieval feature.
    #[test]
    fn test_app_graph_get_layout_panel_by_id() {
        let layout = create_test_layout("test", Some(vec![create_test_panel("panel1")]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout);
        
        let panel = app_graph.get_layout_panel_by_id("test", "panel1");
        assert!(panel.is_some());
        assert_eq!(panel.unwrap().id, "panel1");
        
        let not_found = app_graph.get_layout_panel_by_id("test", "nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that AppGraph::get_panel_by_id() finds panels across layouts.
    /// This test demonstrates the cross-layout panel retrieval feature.
    #[test]
    fn test_app_graph_get_panel_by_id() {
        let layout1 = create_test_layout("layout1", Some(vec![create_test_panel("panel1")]));
        let layout2 = create_test_layout("layout2", Some(vec![create_test_panel("panel2")]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout1);
        app_graph.add_layout(&layout2);
        
        let panel1 = app_graph.get_panel_by_id("panel1");
        assert!(panel1.is_some());
        assert_eq!(panel1.unwrap().id, "panel1");
        
        let panel2 = app_graph.get_panel_by_id("panel2");
        assert!(panel2.is_some());
        assert_eq!(panel2.unwrap().id, "panel2");
    }

    /// Tests that AppGraph::get_children() returns child panels.
    /// This test demonstrates the children retrieval feature.
    #[test]
    fn test_app_graph_get_children() {
        let child_panel = create_test_panel("child");
        let mut parent_panel = create_test_panel("parent");
        parent_panel.children = Some(vec![child_panel]);
        
        let layout = create_test_layout("test", Some(vec![parent_panel]));
        let mut app_graph = AppGraph::new();
        app_graph.add_layout(&layout);
        
        let children = app_graph.get_children("test", "parent");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child");
    }

    /// Tests that AppGraph::get_parent() returns parent panels.
    /// This test demonstrates the parent retrieval feature.
    #[test]
    fn test_app_graph_get_parent() {
        let child_panel = create_test_panel("child");
        let mut parent_panel = create_test_panel("parent");
        parent_panel.children = Some(vec![child_panel]);
        
        let layout = create_test_layout("test", Some(vec![parent_panel]));
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
        let layout = create_test_layout("test", Some(vec![create_test_panel("panel1")]));
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
        let layout = create_test_layout("test", Some(vec![create_test_panel("panel1")]));
        let mut app_graph1 = AppGraph::new();
        let mut app_graph2 = AppGraph::new();
        app_graph1.add_layout(&layout);
        app_graph2.add_layout(&layout);
        
        assert_eq!(app_graph1, app_graph2);
    }

    // === Integration Tests (from original test suite) ===

    #[test]
    fn test_layout_and_panels_addition() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        assert!(app_graph.graphs.contains_key("dashboard"));
        let graph = &app_graph.graphs["dashboard"];
        assert_eq!(
            graph.node_count(),
            9,
            "Should include all panels and sub-panels"
        );
    }

    #[test]
    fn test_get_panel_by_id() {
        let mut app_context = setup_app_context();
        let app_graph = app_context.app.generate_graph();
        let panels = [
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
        for &panel_id in panels.iter() {
            let panel = app_graph.get_panel_by_id(panel_id);
            assert!(panel.is_some(), "Panel with ID {} should exist", panel_id);
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
}

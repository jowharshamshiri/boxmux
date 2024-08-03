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

use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

use crate::{calculate_bounds_map, Config, FieldUpdate, Updatable};

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

        for layout in &mut self.layouts {
            let result = check_unique_ids(layout, &mut id_set);
            if let Err(e) = result {
                panic!("Error: {}", e);
            }
            if layout.root.unwrap_or(false) {
                if root_layout_id.is_some() {
                    panic!("Multiple root layouts detected, which is not allowed.");
                }
                root_layout_id = Some(layout.id.clone());
            }
            if layout.children.is_none() {
                continue;
            }
            for panel in layout.children.as_mut().unwrap() {
                set_parent_ids(panel, &layout.id, None);
            }
        }

        if root_layout_id.is_none() {
            log::debug!("No root layout defined in the application, defaulting to first layout.");
            if let Some(first_layout) = self.layouts.first() {
                root_layout_id = Some(first_layout.id.clone());
            } else {
                panic!("No layouts defined in the application.");
            }
        }

        // Set the root layout as active
        if let Some(root_layout_id) = root_layout_id {
            if let Some(root_layout) = self.get_layout_by_id_mut(&root_layout_id) {
                if root_layout.active.is_none() || !root_layout.active.unwrap() {
                    log::debug!("Setting root layout '{}' as active", root_layout_id);
                    root_layout.active = Some(true);
                    root_layout.root = Some(true);

                    // Set all other layouts as inactive
                    for layout in &mut self.layouts {
                        if layout.id != root_layout_id {
                            layout.active = Some(false);
                            layout.root = Some(false);
                        }
                    }
                }
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
            return app_graph;
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
            let children = &mut layout.get_all_panels();
            for child in children {
                if child.id == panel.id {
                    *child = &panel.clone();
                    return;
                }
            }
        }
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        App {
            layouts: self.layouts.iter().map(|layout| layout.clone()).collect(),
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
    pub fn new(mut app: App, config: Config) -> Self {
        app.validate();

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

    app.validate();

    // log::info!("Loaded app from file: {}", file_path);
    // log::debug!("App: {:#?}", app);

    Ok(app)
}

fn check_unique_ids(
    layout: &Layout,
    id_set: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
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
) -> Result<(), Box<dyn std::error::Error>> {
    if !id_set.insert(panel.id.clone()) {
        return Err(format!("Duplicate ID found: {}", panel.id).into());
    }
    if let Some(children) = &panel.children {
        for child in children {
            check_panel_ids(child, id_set)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn load_test_app_context() -> AppContext {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let dashboard_path = current_dir.join("layouts/tests.yaml");
        let app = load_app_from_yaml(dashboard_path.to_str().unwrap()).expect("Failed to load app");
        AppContext::new(app, Config::default())
    }

    fn setup_app_context() -> AppContext {
        load_test_app_context()
    }

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
}

// app.rs
use lazy_static::lazy_static;
use uuid::Uuid;
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::screen::AlternateScreen;

use serde::{de, ser};

use crate::model::common::{ScreenBuffer};
use crate::model::layout::Layout;
use crate::model::panel::*;

use crate::{screen_height, screen_width};
use crate::thread_manager::{Runnable, ThreadManager};
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use serde::{
    de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;
use std::collections::HashSet;
use serde_yaml;
use petgraph::graph::{DiGraph, NodeIndex, Graph};
use std::collections::HashMap;
use petgraph::visit::EdgeRef;

use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct App {
    pub layouts: Vec<Layout>,
}

impl App {
    pub fn new() -> Self {
        App {
            layouts: Vec::new(),
        }
    }

    pub fn get_layout_by_id(&self, id: &str) -> Option<&Layout> {
        self.layouts.iter().find(|l| l.id == id)
    }

    pub fn get_layout_by_id_mut(&mut self, id: &str) -> Option<&mut Layout> {
        self.layouts.iter_mut().find(|l| l.id == id)
    }
}

impl App {
    pub fn deep_clone(&self) -> Self {
        App {
            layouts: self.layouts.iter().map(|layout| layout.deep_clone()).collect(),
        }
    }
}

#[derive(Debug)]
pub struct AppGraph {
    graphs: HashMap<String, DiGraph<Panel, ()>>,
    node_maps: HashMap<String, HashMap<String, NodeIndex>>,
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

        for panel in &layout.children {
            self.add_panel_recursively(&mut graph, &mut node_map, panel.clone(), None, &layout.id);
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
                self.add_panel_recursively(graph, node_map, child, Some(panel_id.clone()), parent_layout_id);
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
				return self.graphs
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

    pub fn get_layout_by_id<'a>(&self, app: &'a App, layout_id: &str) -> Option<&'a Layout> {
        app.layouts.iter().find(|layout| layout.id == layout_id)
    }
}

impl Hash for AppGraph {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, graph) in &self.graphs {
            key.hash(state);
            for node in graph.node_indices() {
                graph.node_weight(node).unwrap().hash(state);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppContext {
    pub app: App,
    pub app_graph: AppGraph,
	pub screen_buffer: ScreenBuffer,
}

impl PartialEq for AppContext {
	fn eq(&self, other: &Self) -> bool {
		self.app == other.app 
	}
}

impl AppContext {
	pub fn new(app: App) -> Self {
		let mut app_graph = AppGraph::new();
		let mut id_set = HashSet::new();
		let screen_buffer = ScreenBuffer::new(screen_width(), screen_height());

		for layout in &app.layouts {
			check_unique_ids(&layout, &mut id_set);
			app_graph.add_layout(layout);
		}

		AppContext { app, app_graph, screen_buffer}
	}

    pub fn deep_clone(&self) -> Self {
        AppContext {
            app: self.app.deep_clone(),
            app_graph: self.app_graph.clone(),
			screen_buffer: self.screen_buffer.clone(),
        }
    }
}

impl Hash for AppContext {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app.hash(state);
        self.app_graph.hash(state);
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

pub fn load_app_from_yaml(file_path: &str) -> Result<AppContext, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let app: App = serde_yaml::from_str(&contents)?;
    
	let app_context = AppContext::new(app);

    Ok(app_context)
}

fn check_unique_ids(layout: &Layout, id_set: &mut HashSet<String>) -> Result<(), Box<dyn std::error::Error>> {
    for panel in &layout.children {
        check_panel_ids(panel, id_set)?;
    }
    Ok(())
}

fn check_panel_ids(panel: &Panel, id_set: &mut HashSet<String>) -> Result<(), Box<dyn std::error::Error>> {
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

	#[test]
	fn test_new_app() {
		let app = App::new();
		assert_eq!(app.layouts.len(), 0);
	}

    // #[test]
    // fn test_load_app_from_yaml() {
    //     // let mut file = NamedTempFile::new().unwrap();
    //     // write!(file, "---\nlayouts: []").unwrap();
    //     // let path = file.path().to_str().unwrap();

	// 	let path="layouts/dashboard.yaml";

    //     let app = load_app_from_yaml(path);
    //     assert!(app.is_ok());
    //     assert!(app.unwrap().layouts.is_empty());
    
	// }
}

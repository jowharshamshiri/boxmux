//! Test utilities and data factories for BoxMux
//!
//! This module provides common test utilities, data factories, and helper functions
//! to make writing tests easier and more consistent across the codebase.

use crate::model::app::{App, AppContext};
use crate::model::common::{Anchor, Bounds, InputBounds, SocketFunction};
use crate::model::layout::Layout;
use crate::model::panel::Panel;
use crate::thread_manager::Message;
use crate::Config;
use std::collections::HashMap;
use uuid::Uuid;

/// Test data factory for creating consistent test objects
pub struct TestDataFactory;

impl TestDataFactory {
    /// Create a minimal test panel with required fields
    pub fn create_test_panel(id: &str) -> Panel {
        Panel {
            id: id.to_string(),
            title: Some("Test Panel".to_string()),
            position: InputBounds {
                x1: "0".to_string(),
                y1: "0".to_string(),
                x2: "100".to_string(),
                y2: "50".to_string(),
            },
            anchor: Anchor::default(),
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            overflow_behavior: None,
            refresh_interval: None,
            tab_order: None,
            next_focus_id: None,
            children: None,
            fill: None,
            fill_char: None,
            selected_fill_char: None,
            border: Some(true),
            border_color: None,
            selected_border_color: None,
            bg_color: None,
            selected_bg_color: None,
            fg_color: None,
            selected_fg_color: None,
            title_fg_color: None,
            title_bg_color: None,
            selected_title_bg_color: None,
            selected_title_fg_color: None,
            title_position: None,
            error_border_color: None,
            error_bg_color: None,
            error_fg_color: None,
            error_title_bg_color: None,
            error_title_fg_color: None,
            error_selected_border_color: None,
            error_selected_bg_color: None,
            error_selected_fg_color: None,
            error_selected_title_bg_color: None,
            error_selected_title_fg_color: None,
            choices: None,
            menu_fg_color: None,
            menu_bg_color: None,
            selected_menu_fg_color: None,
            selected_menu_bg_color: None,
            redirect_output: None,
            append_output: None,
            script: None,
            thread: None,
            on_keypress: None,
            variables: None,
            horizontal_scroll: None,
            vertical_scroll: None,
            selected: None,
            content: Some("Test content".to_string()),
            save_in_file: None,
            chart_type: None,
            chart_data: None,
            plugin_component: None,
            plugin_config: None,
            table_data: None,
            table_config: None,
            auto_scroll_bottom: None,
            pty: None,
            output: String::new(),
            parent_id: None,
            parent_layout_id: None,
            error_state: false,
        }
    }

    /// Create a panel with custom properties
    pub fn create_custom_panel(id: &str, content: &str) -> Panel {
        let mut panel = Self::create_test_panel(id);
        panel.content = Some(content.to_string());
        panel.title = Some(format!("{} Panel", id));
        panel
    }

    /// Create a test layout with panels
    pub fn create_test_layout(id: &str, panels: Option<Vec<Panel>>) -> Layout {
        Layout {
            id: id.to_string(),
            title: Some(format!("Test Layout {}", id)),
            refresh_interval: None,
            children: panels.or_else(|| Some(vec![Self::create_test_panel("default_panel")])),
            fill: None,
            fill_char: None,
            selected_fill_char: None,
            border: None,
            border_color: None,
            selected_border_color: None,
            bg_color: None,
            selected_bg_color: None,
            fg_color: None,
            selected_fg_color: None,
            title_fg_color: None,
            title_bg_color: None,
            title_position: None,
            selected_title_bg_color: None,
            selected_title_fg_color: None,
            menu_fg_color: None,
            menu_bg_color: None,
            selected_menu_fg_color: None,
            selected_menu_bg_color: None,
            error_border_color: None,
            error_bg_color: None,
            error_fg_color: None,
            error_title_bg_color: None,
            error_title_fg_color: None,
            error_selected_border_color: None,
            error_selected_bg_color: None,
            error_selected_fg_color: None,
            error_selected_title_bg_color: None,
            error_selected_title_fg_color: None,
            overflow_behavior: None,
            root: Some(false),
            on_keypress: None,
            active: None,
            panel_ids_in_tab_order: None,
        }
    }

    /// Create a test layout marked as root
    pub fn create_root_layout(id: &str, panels: Option<Vec<Panel>>) -> Layout {
        let mut layout = Self::create_test_layout(id, panels);
        layout.root = Some(true);
        layout
    }

    /// Create a test app with basic structure
    pub fn create_test_app() -> App {
        let panel = Self::create_test_panel("test_panel");
        let layout = Self::create_root_layout("test_layout", Some(vec![panel]));

        let mut app = App::new();
        app.layouts = vec![layout];
        app.libs = None;
        app.on_keypress = None;
        app.hot_keys = None;
        app
    }

    /// Create an app with multiple layouts for testing layout switching
    pub fn create_multi_layout_app() -> App {
        let panel1 = Self::create_test_panel("panel1");
        let panel2 = Self::create_test_panel("panel2");
        let panel3 = Self::create_test_panel("panel3");

        let layout1 = Self::create_root_layout("layout1", Some(vec![panel1]));
        let layout2 = Self::create_test_layout("layout2", Some(vec![panel2]));
        let layout3 = Self::create_test_layout("layout3", Some(vec![panel3]));

        let mut app = App::new();
        app.layouts = vec![layout1, layout2, layout3];
        app.libs = None;
        app.on_keypress = None;
        app
    }

    /// Create test bounds with specified dimensions
    pub fn create_bounds(x1: usize, y1: usize, x2: usize, y2: usize) -> Bounds {
        Bounds { x1, y1, x2, y2 }
    }

    /// Create test input bounds with string coordinates
    pub fn create_input_bounds(x1: &str, y1: &str, x2: &str, y2: &str) -> InputBounds {
        InputBounds {
            x1: x1.to_string(),
            y1: y1.to_string(),
            x2: x2.to_string(),
            y2: y2.to_string(),
        }
    }

    /// Create app context for testing
    pub fn create_test_app_context() -> AppContext {
        AppContext {
            app: Self::create_test_app(),
            config: Config::default(),
            plugin_registry: std::sync::Arc::new(std::sync::Mutex::new(
                crate::plugin::PluginRegistry::new(),
            )),
            pty_manager: None,
            yaml_file_path: None,
        }
    }

    /// Create socket function for testing
    pub fn create_socket_function_replace_content(
        panel_id: &str,
        content: &str,
        success: bool,
    ) -> SocketFunction {
        SocketFunction::ReplacePanelContent {
            panel_id: panel_id.to_string(),
            success,
            content: content.to_string(),
        }
    }

    /// Create socket function for testing script replacement
    pub fn create_socket_function_replace_script(
        panel_id: &str,
        script: Vec<String>,
    ) -> SocketFunction {
        SocketFunction::ReplacePanelScript {
            panel_id: panel_id.to_string(),
            script,
        }
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a panel exists in a layout with specific properties
    pub fn assert_panel_exists_in_layout(layout: &Layout, panel_id: &str) -> bool {
        layout
            .children
            .as_ref()
            .map_or(false, |panels| panels.iter().any(|p| p.id == panel_id))
    }

    /// Assert that a panel has specific content
    pub fn assert_panel_has_content(panel: &Panel, expected_content: &str) -> bool {
        panel.content.as_ref().map(|c| c.as_str()) == Some(expected_content)
    }

    /// Assert that an app has a specific active layout
    pub fn assert_app_active_layout(app: &App, expected_layout_id: &str) -> bool {
        // App doesn't have active_layout field - use get_active_layout() method
        app.get_active_layout().map(|layout| layout.id.as_str()) == Some(expected_layout_id)
    }

    /// Assert that a layout is marked as root
    pub fn assert_layout_is_root(layout: &Layout) -> bool {
        layout.root.unwrap_or(false)
    }

    /// Assert message type matches expected
    pub fn assert_message_type(message: &Message, expected_type: &str) -> bool {
        match (message, expected_type) {
            (Message::PanelOutputUpdate(_, _, _), "PanelOutputUpdate") => true,
            (Message::PanelScriptUpdate(_, _), "PanelScriptUpdate") => true,
            (Message::StopPanelRefresh(_), "StopPanelRefresh") => true,
            (Message::StartPanelRefresh(_), "StartPanelRefresh") => true,
            (Message::SwitchActiveLayout(_), "SwitchActiveLayout") => true,
            (Message::ReplacePanel(_, _), "ReplacePanel") => true,
            (Message::AddPanel(_, _), "AddPanel") => true,
            (Message::RemovePanel(_), "RemovePanel") => true,
            _ => false,
        }
    }
}

/// Performance testing utilities
pub struct PerformanceTestUtils;

impl PerformanceTestUtils {
    /// Benchmark a function and return the duration
    pub fn benchmark_function<F, R>(iterations: usize, mut func: F) -> std::time::Duration
    where
        F: FnMut() -> R,
    {
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _ = func();
        }
        start.elapsed()
    }

    /// Assert that a function meets performance requirements
    pub fn assert_performance<F, R>(
        func: F,
        iterations: usize,
        max_duration: std::time::Duration,
        operation_name: &str,
    ) where
        F: FnMut() -> R,
    {
        let duration = Self::benchmark_function(iterations, func);
        println!(
            "{}: {} iterations took {:?}",
            operation_name, iterations, duration
        );
        assert!(
            duration <= max_duration,
            "{} performance regression: {:?} > {:?}",
            operation_name,
            duration,
            max_duration
        );
    }

    /// Create large test data for performance testing
    pub fn create_large_panel_list(count: usize) -> Vec<Panel> {
        (0..count)
            .map(|i| TestDataFactory::create_test_panel(&format!("panel_{}", i)))
            .collect()
    }

    /// Create large test layout with many panels
    pub fn create_large_layout(panel_count: usize) -> Layout {
        let panels = Self::create_large_panel_list(panel_count);
        TestDataFactory::create_test_layout("large_layout", Some(panels))
    }
}

/// Mock and stub utilities for testing
pub struct MockUtils;

impl MockUtils {
    /// Create a mock UUID for consistent testing
    pub fn create_test_uuid() -> Uuid {
        Uuid::parse_str("12345678-1234-5678-9012-123456789012").unwrap()
    }

    /// Create a series of test UUIDs for multiple object testing
    pub fn create_test_uuids(count: usize) -> Vec<Uuid> {
        (0..count)
            .map(|i| {
                Uuid::parse_str(&format!("12345678-1234-5678-9012-12345678901{:01}", i % 10))
                    .unwrap()
            })
            .collect()
    }

    /// Create test script commands
    pub fn create_test_script_commands() -> Vec<String> {
        vec![
            "echo 'test output'".to_string(),
            "date".to_string(),
            "whoami".to_string(),
        ]
    }

    /// Create test key mappings
    pub fn create_test_key_mappings() -> HashMap<String, Vec<String>> {
        let mut mappings = HashMap::new();
        mappings.insert("Ctrl + C".to_string(), vec!["exit".to_string()]);
        mappings.insert("Ctrl + D".to_string(), vec!["quit".to_string()]);
        mappings.insert("Enter".to_string(), vec!["confirm".to_string()]);
        mappings.insert("Escape".to_string(), vec!["cancel".to_string()]);
        mappings.insert("Tab".to_string(), vec!["next_panel".to_string()]);
        mappings
    }
}

/// Integration test utilities
pub struct IntegrationTestUtils;

impl IntegrationTestUtils {
    /// Setup complete test environment
    pub fn setup_test_environment() -> (AppContext, std::sync::mpsc::Receiver<(Uuid, Message)>) {
        let app_context = TestDataFactory::create_test_app_context();
        let (tx, rx) = std::sync::mpsc::channel();
        // Additional setup could go here
        (app_context, rx)
    }

    /// Simulate complete workflow from socket command to app update
    pub fn simulate_socket_to_app_workflow(
        socket_function: SocketFunction,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let (tx, rx) = std::sync::mpsc::channel();
        let test_uuid = MockUtils::create_test_uuid();

        // This would normally go through the socket handler
        let boxmux_message = match socket_function {
            SocketFunction::ReplacePanelContent {
                panel_id,
                success,
                content,
            } => Message::PanelOutputUpdate(panel_id, success, content),
            SocketFunction::ReplacePanelScript { panel_id, script } => {
                Message::PanelScriptUpdate(panel_id, script)
            }
            SocketFunction::StopPanelRefresh { panel_id } => Message::StopPanelRefresh(panel_id),
            SocketFunction::StartPanelRefresh { panel_id } => Message::StartPanelRefresh(panel_id),
            SocketFunction::ReplacePanel {
                panel_id,
                new_panel,
            } => Message::ReplacePanel(panel_id, new_panel),
            SocketFunction::SwitchActiveLayout { layout_id } => {
                Message::SwitchActiveLayout(layout_id)
            }
            SocketFunction::AddPanel { layout_id, panel } => Message::AddPanel(layout_id, panel),
            SocketFunction::RemovePanel { panel_id } => Message::RemovePanel(panel_id),
            // F0137/F0138: Socket PTY Control and Query patterns
            SocketFunction::KillPtyProcess { panel_id } => {
                Message::PanelOutputUpdate(panel_id, true, "PTY process killed".to_string())
            }
            SocketFunction::RestartPtyProcess { panel_id } => {
                Message::PanelOutputUpdate(panel_id, true, "PTY process restarted".to_string())
            }
            SocketFunction::QueryPtyStatus { panel_id } => {
                Message::PanelOutputUpdate(panel_id, true, "PTY status queried".to_string())
            }
        };

        tx.send((test_uuid, boxmux_message))?;
        let (_, message) = rx.recv()?;
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creates_valid_panel() {
        let panel = TestDataFactory::create_test_panel("test");
        assert_eq!(panel.id, "test");
        // Panel doesn't have panel_type field - removed assertion
        assert!(panel.content.is_some());
    }

    #[test]
    fn test_factory_creates_valid_layout() {
        let layout = TestDataFactory::create_test_layout("test_layout", None);
        assert_eq!(layout.id, "test_layout");
        assert!(layout.children.is_some());
        let children = layout.children.as_ref().unwrap();
        assert!(!children.is_empty());
        assert_eq!(children[0].id, "default_panel");
    }

    #[test]
    fn test_factory_creates_valid_app() {
        let app = TestDataFactory::create_test_app();
        assert!(!app.layouts.is_empty());
        // App doesn't have active_layout field - checking root layout instead
        assert!(app.layouts[0].root.unwrap_or(false));
        assert!(TestAssertions::assert_layout_is_root(&app.layouts[0]));
    }

    #[test]
    fn test_assertions_work_correctly() {
        let panel = TestDataFactory::create_test_panel("test");
        assert!(TestAssertions::assert_panel_has_content(
            &panel,
            "Test content"
        ));
        assert!(!TestAssertions::assert_panel_has_content(
            &panel,
            "Wrong content"
        ));
    }

    #[test]
    fn test_performance_utils() {
        let duration = PerformanceTestUtils::benchmark_function(1000, || {
            let _ = TestDataFactory::create_test_panel("perf_test");
        });

        // Should be able to create 1000 panels very quickly
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_mock_utils() {
        let uuid = MockUtils::create_test_uuid();
        assert_eq!(uuid.to_string(), "12345678-1234-5678-9012-123456789012");

        let uuids = MockUtils::create_test_uuids(3);
        assert_eq!(uuids.len(), 3);
        assert_ne!(uuids[0], uuids[1]); // Should have different endings
    }

    #[test]
    fn test_integration_utils() {
        let socket_function = TestDataFactory::create_socket_function_replace_content(
            "test_panel",
            "new content",
            true,
        );

        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(socket_function);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(TestAssertions::assert_message_type(
            &message,
            "PanelOutputUpdate"
        ));
    }
}

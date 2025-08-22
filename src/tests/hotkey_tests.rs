//! F0081 - Hot Key Actions Tests
//! Test the global keyboard shortcuts to trigger specific choice actions directly

#[cfg(test)]
mod hotkey_tests {
    use crate::model::app::App;
    use crate::model::layout::Layout;
    use crate::model::panel::{Panel, Choice};
    use crate::model::common::{InputBounds, Anchor};
    use crate::thread_manager::Message;
    use std::collections::HashMap;

    /// Test that hot_keys field can be set on App struct
    #[test]
    fn test_app_hotkeys_field_creation() {
        let mut app = App::new();
        let mut hotkeys = HashMap::new();
        hotkeys.insert("F1".to_string(), "deploy".to_string());
        hotkeys.insert("Ctrl+r".to_string(), "restart".to_string());
        
        app.hot_keys = Some(hotkeys);
        
        assert!(app.hot_keys.is_some());
        let hotkeys = app.hot_keys.unwrap();
        assert_eq!(hotkeys.get("F1"), Some(&"deploy".to_string()));
        assert_eq!(hotkeys.get("Ctrl+r"), Some(&"restart".to_string()));
    }

    /// Test that ExecuteHotKeyChoice message can be created
    #[test] 
    fn test_execute_hotkey_choice_message() {
        let choice_id = "deploy".to_string();
        let message = Message::ExecuteHotKeyChoice(choice_id.clone());
        
        match message {
            Message::ExecuteHotKeyChoice(id) => {
                assert_eq!(id, choice_id);
            }
            _ => panic!("Expected ExecuteHotKeyChoice message"),
        }
    }

    /// Test that find_panel_with_choice method works correctly
    #[test]
    fn test_find_panel_with_choice() {
        use crate::tests::test_utils::TestDataFactory;

        let choice = Choice {
            id: "test_choice".to_string(),
            content: Some("Test Choice".to_string()),
            script: Some(vec!["echo test".to_string()]),
            thread: Some(false),
            redirect_output: None,
            append_output: None,
            selected: false,
            waiting: false,
        };

        let mut panel = TestDataFactory::create_test_panel("test_panel");
        panel.choices = Some(vec![choice]);

        let mut layout = TestDataFactory::create_test_layout("test_layout", Some(vec![panel]));
        layout.root = Some(true);

        // Test finding existing choice
        let found_panel = layout.find_panel_with_choice("test_choice");
        assert!(found_panel.is_some());
        assert_eq!(found_panel.unwrap().id, "test_panel");

        // Test not finding non-existent choice
        let not_found = layout.find_panel_with_choice("non_existent");
        assert!(not_found.is_none());
    }

    /// Test hot key configuration in App structure
    #[test]
    fn test_hotkey_configuration() {
        let mut app = App::new();
        
        // Test that hot_keys starts as None
        assert!(app.hot_keys.is_none());
        
        // Test setting hot keys
        let mut hotkeys = HashMap::new();
        hotkeys.insert("F1".to_string(), "build".to_string());
        hotkeys.insert("F2".to_string(), "test".to_string());
        hotkeys.insert("Ctrl+d".to_string(), "deploy".to_string());
        
        app.hot_keys = Some(hotkeys);
        
        assert!(app.hot_keys.is_some());
        let hotkeys = app.hot_keys.as_ref().unwrap();
        assert_eq!(hotkeys.len(), 3);
        assert_eq!(hotkeys.get("F1"), Some(&"build".to_string()));
        assert_eq!(hotkeys.get("F2"), Some(&"test".to_string()));
        assert_eq!(hotkeys.get("Ctrl+d"), Some(&"deploy".to_string()));
    }

    /// Test App Clone includes hot_keys
    #[test]
    fn test_app_clone_includes_hotkeys() {
        let mut app = App::new();
        let mut hotkeys = HashMap::new();
        hotkeys.insert("F5".to_string(), "refresh".to_string());
        app.hot_keys = Some(hotkeys);

        let cloned_app = app.clone();
        assert!(cloned_app.hot_keys.is_some());
        assert_eq!(
            cloned_app.hot_keys.as_ref().unwrap().get("F5"),
            Some(&"refresh".to_string())
        );
    }

    /// Test App PartialEq includes hot_keys
    #[test]
    fn test_app_equality_includes_hotkeys() {
        let mut app1 = App::new();
        let mut app2 = App::new();
        
        // Initially equal
        assert_eq!(app1, app2);
        
        // Add hot keys to app1
        let mut hotkeys = HashMap::new();
        hotkeys.insert("F1".to_string(), "test".to_string());
        app1.hot_keys = Some(hotkeys);
        
        // Now different
        assert_ne!(app1, app2);
        
        // Make app2 equal
        let mut hotkeys2 = HashMap::new();
        hotkeys2.insert("F1".to_string(), "test".to_string());
        app2.hot_keys = Some(hotkeys2);
        
        // Equal again
        assert_eq!(app1, app2);
    }
}
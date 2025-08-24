#[cfg(test)]
mod plugin_integration_tests {
    use crate::model::app::App;
    use crate::model::common::Bounds;
    use crate::model::panel::Panel;
    use crate::{AppContext, Config};
    use std::collections::HashMap;

    #[test]
    fn test_panel_plugin_content_generation() {
        let mut panel = Panel::default();
        panel.id = "test_plugin".to_string();
        panel.plugin_component = Some("custom_chart".to_string());

        let mut plugin_config = HashMap::new();
        plugin_config.insert(
            "title".to_string(),
            serde_json::Value::String("Test Chart".to_string()),
        );
        plugin_config.insert(
            "refresh_rate".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1000)),
        );
        panel.plugin_config = Some(plugin_config);

        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        let bounds = Bounds::new(0, 0, 30, 10);

        let plugin_content = panel.generate_plugin_content(&app_context, &bounds);

        assert!(plugin_content.is_some());
        let content = plugin_content.unwrap();
        assert!(content.contains("custom_chart"));
        assert!(content.contains("Test Chart"));
        assert!(content.contains("30x10")); // Bounds verification
    }

    #[test]
    fn test_panel_no_plugin_config() {
        let mut panel = Panel::default();
        panel.id = "test_no_plugin".to_string();
        // No plugin_component and no plugin_config

        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        let bounds = Bounds::new(0, 0, 20, 5);

        let plugin_content = panel.generate_plugin_content(&app_context, &bounds);

        assert!(plugin_content.is_none());
    }

    #[test]
    fn test_panel_plugin_without_component_type() {
        let mut panel = Panel::default();
        panel.id = "test_no_component".to_string();
        // No plugin_component

        let mut plugin_config = HashMap::new();
        plugin_config.insert(
            "data".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        panel.plugin_config = Some(plugin_config);

        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        let bounds = Bounds::new(0, 0, 20, 5);

        let plugin_content = panel.generate_plugin_content(&app_context, &bounds);

        assert!(plugin_content.is_none());
    }

    #[test]
    fn test_plugin_content_priority() {
        // Test that output overrides plugin content, plugin content overrides static content
        let mut panel = Panel::default();
        panel.id = "test_priority".to_string();
        panel.content = Some("Static content".to_string());
        panel.plugin_component = Some("custom_widget".to_string());

        let mut plugin_config = HashMap::new();
        plugin_config.insert(
            "mode".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        panel.plugin_config = Some(plugin_config);

        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        let bounds = Bounds::new(0, 0, 25, 8);

        // Plugin content should override static content
        let plugin_content = panel.generate_plugin_content(&app_context, &bounds);
        assert!(plugin_content.is_some());
        assert!(plugin_content.unwrap().contains("custom_widget"));

        // Output should override everything (tested in rendering)
        panel.output = "Script output".to_string();
        // In actual rendering, output would override plugin content
    }

    #[test]
    fn test_plugin_config_serialization() {
        let mut panel = Panel::default();
        panel.id = "test_serialization".to_string();
        panel.plugin_component = Some("data_table".to_string());

        let mut plugin_config = HashMap::new();
        plugin_config.insert(
            "columns".to_string(),
            serde_json::json!(["Name", "Value", "Status"]),
        );
        plugin_config.insert(
            "sort_by".to_string(),
            serde_json::Value::String("Name".to_string()),
        );
        plugin_config.insert(
            "page_size".to_string(),
            serde_json::Value::Number(serde_json::Number::from(25)),
        );
        panel.plugin_config = Some(plugin_config);

        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        let bounds = Bounds::new(0, 0, 50, 15);

        let plugin_content = panel.generate_plugin_content(&app_context, &bounds);

        assert!(plugin_content.is_some());
        let content = plugin_content.unwrap();
        assert!(content.contains("data_table"));
        assert!(content.contains("Name"));
        assert!(content.contains("25"));
        assert!(content.contains("50x15"));
    }

    #[test]
    fn test_plugin_registry_initialization() {
        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);

        // Verify plugin registry is initialized
        let registry = app_context.plugin_registry.lock().unwrap();
        assert_eq!(registry.list_plugins().len(), 0); // Empty by default
    }

    #[test]
    fn test_complex_plugin_config() {
        let mut panel = Panel::default();
        panel.id = "test_complex".to_string();
        panel.plugin_component = Some("dashboard_widget".to_string());
        panel.refresh_interval = Some(5000);

        let plugin_config = serde_json::json!({
            "widget_type": "metrics",
            "data_sources": ["cpu", "memory", "disk"],
            "display": {
                "chart_type": "line",
                "time_range": "1h",
                "colors": ["red", "green", "blue"]
            },
            "alerts": {
                "enabled": true,
                "thresholds": {
                    "cpu": 80,
                    "memory": 90,
                    "disk": 95
                }
            }
        });

        if let serde_json::Value::Object(map) = plugin_config {
            let hashmap: HashMap<String, serde_json::Value> = map.into_iter().collect();
            panel.plugin_config = Some(hashmap);
        }

        let app = App::default();
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        let bounds = Bounds::new(0, 0, 60, 20);

        let plugin_content = panel.generate_plugin_content(&app_context, &bounds);

        assert!(plugin_content.is_some());
        let content = plugin_content.unwrap();
        assert!(content.contains("dashboard_widget"));
        assert!(content.contains("metrics"));
        assert!(content.contains("60x20"));
    }
}

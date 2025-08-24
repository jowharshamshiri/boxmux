#[cfg(test)]
mod chart_integration_tests {
    use crate::model::common::Bounds;
    use crate::model::panel::Panel;

    #[test]
    fn test_panel_chart_content_generation() {
        let mut panel = Panel::default();
        panel.id = "test_chart".to_string();
        panel.chart_type = Some("bar".to_string());
        panel.chart_data = Some("Item1,10\nItem2,20\nItem3,15".to_string());

        let bounds = Bounds::new(0, 0, 30, 10);
        let chart_content = panel.generate_chart_content(&bounds);

        assert!(chart_content.is_some());
        let content = chart_content.unwrap();
        assert!(content.contains("Item1"));
        assert!(content.contains("█")); // Bar chart should contain bars
    }

    #[test]
    fn test_panel_no_chart_data() {
        let mut panel = Panel::default();
        panel.id = "test_no_chart".to_string();
        panel.chart_type = Some("bar".to_string());
        // No chart_data

        let bounds = Bounds::new(0, 0, 20, 5);
        let chart_content = panel.generate_chart_content(&bounds);

        assert!(chart_content.is_none());
    }

    #[test]
    fn test_panel_empty_chart_data() {
        let mut panel = Panel::default();
        panel.id = "test_empty_chart".to_string();
        panel.chart_type = Some("bar".to_string());
        panel.chart_data = Some("".to_string());

        let bounds = Bounds::new(0, 0, 20, 5);
        let chart_content = panel.generate_chart_content(&bounds);

        assert!(chart_content.is_some());
        assert_eq!(chart_content.unwrap(), "No chart data");
    }

    #[test]
    fn test_all_chart_types() {
        let chart_data = "A,10\nB,20\nC,15".to_string();
        let bounds = Bounds::new(0, 0, 25, 8);

        let chart_types = vec!["bar", "line", "histogram"];

        for chart_type in chart_types {
            let mut panel = Panel::default();
            panel.id = format!("test_{}", chart_type);
            panel.chart_type = Some(chart_type.to_string());
            panel.chart_data = Some(chart_data.clone());

            let chart_content = panel.generate_chart_content(&bounds);
            assert!(chart_content.is_some(), "Chart type {} failed", chart_type);

            let content = chart_content.unwrap();
            assert!(
                !content.is_empty(),
                "Chart type {} produced empty content",
                chart_type
            );
        }
    }

    #[test]
    fn test_chart_data_formats() {
        let bounds = Bounds::new(0, 0, 20, 5);

        // Test different data formats
        let formats = vec![
            ("csv", "A,10\nB,20"),
            ("colon", "A:10\nB:20"),
            ("space", "A 10\nB 20"),
        ];

        for (format_name, data) in formats {
            let mut panel = Panel::default();
            panel.id = format!("test_{}", format_name);
            panel.chart_type = Some("bar".to_string());
            panel.chart_data = Some(data.to_string());

            let chart_content = panel.generate_chart_content(&bounds);
            assert!(chart_content.is_some(), "Format {} failed", format_name);

            let content = chart_content.unwrap();
            assert!(
                !content.is_empty(),
                "Format {} produced empty content",
                format_name
            );
        }
    }

    #[test]
    fn test_invalid_chart_type() {
        let mut panel = Panel::default();
        panel.id = "test_invalid".to_string();
        panel.chart_type = Some("invalid_type".to_string());
        panel.chart_data = Some("A,10\nB,20".to_string());

        let bounds = Bounds::new(0, 0, 20, 5);
        let chart_content = panel.generate_chart_content(&bounds);

        // Should default to bar chart
        assert!(chart_content.is_some());
        let content = chart_content.unwrap();
        assert!(content.contains("│")); // Bar chart formatting
    }
}

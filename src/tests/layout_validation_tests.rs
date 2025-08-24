#[cfg(test)]
mod layout_validation_tests {
    use crate::model::app::{load_app_from_yaml, App};
    use crate::{AppContext, Config};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_all_layout_files_are_valid() {
        let layouts_dir = Path::new("layouts");

        // Verify layouts directory exists
        assert!(layouts_dir.exists(), "layouts directory should exist");

        // Get all YAML files in layouts directory
        let yaml_files = fs::read_dir(layouts_dir)
            .expect("Failed to read layouts directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "yaml" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Ensure we found some YAML files
        assert!(
            !yaml_files.is_empty(),
            "Should find at least one YAML file in layouts directory"
        );

        let mut successful_loads = 0;
        let mut total_files = 0;

        for yaml_file in yaml_files {
            total_files += 1;
            let file_name = yaml_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            println!("Testing layout file: {}", file_name);

            // Test 1: File can be read
            let _yaml_content = match fs::read_to_string(&yaml_file) {
                Ok(content) => content,
                Err(e) => {
                    panic!("Failed to read file {}: {}", file_name, e);
                }
            };

            // Test 2: YAML can be parsed and loaded into App
            let file_path_str = yaml_file.to_str().expect("Invalid file path");
            let app = match load_app_from_yaml(file_path_str) {
                Ok(app) => {
                    println!("  ✅ Successfully loaded {}", file_name);
                    app
                }
                Err(e) => {
                    panic!("Failed to load app from {}: {}", file_name, e);
                }
            };

            // Test 3: App validation passes
            let mut app_mut = app.clone();
            app_mut.validate(); // validate() doesn't return a Result
            println!("  ✅ Successfully validated {}", file_name);

            // Test 4: App graph can be generated (no circular dependencies)
            let mut app_for_graph = app.clone();
            let _graph = app_for_graph.generate_graph(); // generate_graph() doesn't return a Result
            println!("  ✅ Successfully generated graph for {}", file_name);

            // Test 5: AppContext can be created and basic operations work
            let config = Config::default();
            let app_context = AppContext::new(app, config);

            // Verify root layout exists
            assert!(
                app_context.app.get_root_layout().is_some(),
                "File {} should have a root layout",
                file_name
            );

            // Verify at least one layout exists
            assert!(
                !app_context.app.layouts.is_empty(),
                "File {} should have at least one layout",
                file_name
            );

            successful_loads += 1;
        }

        println!(
            "Layout validation summary: {}/{} files successfully loaded and validated",
            successful_loads, total_files
        );

        // Verify we successfully processed all files
        assert_eq!(
            successful_loads, total_files,
            "All layout files should load successfully"
        );

        // Ensure we tested a reasonable number of files
        assert!(
            total_files >= 10,
            "Expected at least 10 layout files, found {}",
            total_files
        );
    }

    #[test]
    fn test_specific_layout_files_functionality() {
        let test_cases = vec![
            ("chart_demo.yaml", "Should contain chart muxboxes"),
            ("table_demo.yaml", "Should contain table muxboxes"),
            ("plugin_demo.yaml", "Should contain plugin muxboxes"),
            (
                "system_monitor_pro.yaml",
                "Should contain system monitoring muxboxes",
            ),
            (
                "developer_workspace.yaml",
                "Should contain development tools",
            ),
        ];

        for (filename, description) in test_cases {
            let file_path = Path::new("layouts").join(filename);

            if !file_path.exists() {
                println!("⚠️  Optional layout file {} not found, skipping", filename);
                continue;
            }

            println!(
                "Testing specific functionality of {}: {}",
                filename, description
            );

            let file_path_str = file_path.to_str().expect("Invalid file path");
            let app = load_app_from_yaml(file_path_str)
                .unwrap_or_else(|e| panic!("Failed to load {}: {}", filename, e));

            // Verify basic structure
            assert!(!app.layouts.is_empty(), "{} should have layouts", filename);

            let root_layout = app
                .layouts
                .iter()
                .find(|l| l.root == Some(true))
                .unwrap_or_else(|| panic!("{} should have a root layout", filename));

            assert!(
                root_layout.children.is_some(),
                "{} root layout should have children",
                filename
            );

            let children = root_layout.children.as_ref().unwrap();
            assert!(
                !children.is_empty(),
                "{} should have at least one muxbox",
                filename
            );

            // File-specific validations
            match filename {
                "chart_demo.yaml" => {
                    let has_charts = children
                        .iter()
                        .any(|muxbox| muxbox.chart_type.is_some() || muxbox.chart_data.is_some());
                    assert!(has_charts, "chart_demo.yaml should contain chart muxboxes");
                }
                "table_demo.yaml" => {
                    let has_tables = children
                        .iter()
                        .any(|muxbox| muxbox.table_data.is_some() || muxbox.table_config.is_some());
                    assert!(has_tables, "table_demo.yaml should contain table muxboxes");
                }
                "plugin_demo.yaml" => {
                    let has_plugins = children
                        .iter()
                        .any(|muxbox| muxbox.plugin_component.is_some());
                    assert!(has_plugins, "plugin_demo.yaml should contain plugin muxboxes");
                }
                _ => {
                    // Generic validation for other files
                    println!("  ✅ Basic structure validation passed for {}", filename);
                }
            }

            println!(
                "  ✅ Specific functionality validation passed for {}",
                filename
            );
        }
    }

    #[test]
    fn test_layout_files_have_proper_structure() {
        let layouts_dir = Path::new("layouts");
        let yaml_files = fs::read_dir(layouts_dir)
            .expect("Failed to read layouts directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "yaml" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for yaml_file in yaml_files {
            let file_name = yaml_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            println!("Checking structure of {}", file_name);

            let file_path_str = yaml_file.to_str().expect("Invalid file path");
            let app = load_app_from_yaml(file_path_str)
                .unwrap_or_else(|e| panic!("Failed to load {}: {}", file_name, e));

            // Verify required structure elements
            assert!(
                !app.layouts.is_empty(),
                "{} must have at least one layout",
                file_name
            );

            // Check for root layout
            let root_count = app.layouts.iter().filter(|l| l.root == Some(true)).count();
            assert_eq!(
                root_count, 1,
                "{} must have exactly one root layout",
                file_name
            );

            // Verify layout IDs are unique
            let mut layout_ids = std::collections::HashSet::new();
            for layout in &app.layouts {
                assert!(
                    layout_ids.insert(&layout.id),
                    "{} has duplicate layout ID: {}",
                    file_name,
                    layout.id
                );
            }

            // Verify muxbox IDs are unique within each layout
            for layout in &app.layouts {
                if let Some(children) = &layout.children {
                    let mut muxbox_ids = std::collections::HashSet::new();
                    for muxbox in children {
                        assert!(
                            muxbox_ids.insert(&muxbox.id),
                            "{} layout '{}' has duplicate muxbox ID: {}",
                            file_name,
                            layout.id,
                            muxbox.id
                        );
                    }
                }
            }

            println!("  ✅ Structure validation passed for {}", file_name);
        }
    }

    #[test]
    fn test_layout_files_performance() {
        let layouts_dir = Path::new("layouts");
        let yaml_files = fs::read_dir(layouts_dir)
            .expect("Failed to read layouts directory")
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "yaml" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let start_time = std::time::Instant::now();

        for yaml_file in &yaml_files {
            let file_start = std::time::Instant::now();

            let file_path_str = yaml_file.to_str().expect("Invalid file path");
            let _app = load_app_from_yaml(file_path_str).expect("App should load successfully");

            let load_time = file_start.elapsed();

            // Each file should load within reasonable time (1 second)
            assert!(
                load_time.as_millis() < 1000,
                "File {:?} took too long to load: {}ms",
                yaml_file.file_name(),
                load_time.as_millis()
            );
        }

        let total_time = start_time.elapsed();

        println!(
            "Performance test: loaded {} layout files in {}ms",
            yaml_files.len(),
            total_time.as_millis()
        );

        // All files combined should load within reasonable time (5 seconds)
        assert!(
            total_time.as_secs() < 5,
            "Total loading time too long: {}ms",
            total_time.as_millis()
        );
    }
}

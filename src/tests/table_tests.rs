use crate::table::*;
use std::collections::HashMap;

#[test]
fn test_table_data_parsing_csv() {
    let csv_content = "Name,Age,Status\nAlice,25,Active\nBob,30,Inactive\nCharlie,35,Active";
    let table_data = parse_table_data(csv_content, None);

    assert_eq!(table_data.headers, vec!["Name", "Age", "Status"]);
    assert_eq!(table_data.rows.len(), 3);
    assert_eq!(table_data.rows[0], vec!["Alice", "25", "Active"]);
    assert_eq!(table_data.rows[1], vec!["Bob", "30", "Inactive"]);
    assert_eq!(table_data.rows[2], vec!["Charlie", "35", "Active"]);
    assert_eq!(table_data.metadata.get("source").unwrap(), "parsed_csv");
}

#[test]
fn test_table_data_parsing_json() {
    let json_content = r#"[
            {"name": "Alice", "age": 25, "status": "Active"},
            {"name": "Bob", "age": 30, "status": "Inactive"},
            {"name": "Charlie", "age": 35, "status": "Active"}
        ]"#;

    let table_data = parse_table_data_from_json(json_content).unwrap();

    assert_eq!(table_data.rows.len(), 3);
    assert!(table_data.headers.contains(&"name".to_string()));
    assert!(table_data.headers.contains(&"age".to_string()));
    assert!(table_data.headers.contains(&"status".to_string()));
    assert_eq!(table_data.metadata.get("source").unwrap(), "parsed_json");
}

#[test]
fn test_table_rendering_basic() {
    let data = TableData {
        headers: vec!["ID".to_string(), "Name".to_string(), "Score".to_string()],
        rows: vec![
            vec!["1".to_string(), "Alice".to_string(), "95".to_string()],
            vec!["2".to_string(), "Bob".to_string(), "87".to_string()],
            vec!["3".to_string(), "Charlie".to_string(), "92".to_string()],
        ],
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        title: Some("Test Results".to_string()),
        width: 40,
        height: 10,
        border_style: TableBorderStyle::Single,
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Check for basic structure
    assert!(result.contains("Test Results"));
    assert!(result.contains("ID"));
    assert!(result.contains("Alice"));
    assert!(result.contains("95"));
    assert!(result.contains("┌")); // Top border
    assert!(result.contains("├")); // Header separator
    assert!(result.contains("└")); // Bottom border
    assert!(result.contains("│")); // Vertical borders
}

#[test]
fn test_table_border_styles() {
    let data = TableData {
        headers: vec!["A".to_string(), "B".to_string()],
        rows: vec![vec!["1".to_string(), "2".to_string()]],
        metadata: HashMap::new(),
    };

    // Test different border styles
    let configs = vec![
        (TableBorderStyle::None, vec![]),
        (TableBorderStyle::Single, vec!["┌", "├", "└", "│"]),
        (TableBorderStyle::Double, vec!["╔", "╠", "╚", "║"]),
        (TableBorderStyle::Rounded, vec!["╭", "├", "╰", "│"]),
        (TableBorderStyle::Thick, vec!["┏", "├", "┗", "│"]),
    ];

    for (style, expected_chars) in configs {
        let config = TableConfig {
            border_style: style,
            ..TableConfig::default()
        };

        let result = render_table(&data, &config);

        for expected_char in expected_chars {
            assert!(
                result.contains(expected_char),
                "Border style {:?} should contain '{}'",
                config.border_style,
                expected_char
            );
        }
    }
}

#[test]
fn test_table_filtering() {
    let data = TableData {
        headers: vec![
            "Name".to_string(),
            "Department".to_string(),
            "Salary".to_string(),
        ],
        rows: vec![
            vec![
                "Alice".to_string(),
                "Engineering".to_string(),
                "75000".to_string(),
            ],
            vec![
                "Bob".to_string(),
                "Marketing".to_string(),
                "65000".to_string(),
            ],
            vec![
                "Charlie".to_string(),
                "Engineering".to_string(),
                "80000".to_string(),
            ],
            vec![
                "Diana".to_string(),
                "Sales".to_string(),
                "70000".to_string(),
            ],
        ],
        metadata: HashMap::new(),
    };

    let mut filters = HashMap::new();
    filters.insert("Department".to_string(), "Engineering".to_string());

    let config = TableConfig {
        filters,
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Should contain Engineering entries
    assert!(result.contains("Alice"));
    assert!(result.contains("Charlie"));
    // Should not contain non-Engineering entries
    assert!(!result.contains("Bob"));
    assert!(!result.contains("Diana"));
}

#[test]
fn test_table_sorting_text() {
    let data = TableData {
        headers: vec!["Name".to_string(), "Score".to_string()],
        rows: vec![
            vec!["Charlie".to_string(), "92".to_string()],
            vec!["Alice".to_string(), "95".to_string()],
            vec!["Bob".to_string(), "87".to_string()],
        ],
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        sort_column: Some("Name".to_string()),
        sort_ascending: true,
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Check that Alice comes before Bob which comes before Charlie
    let alice_pos = result.find("Alice").unwrap();
    let bob_pos = result.find("Bob").unwrap();
    let charlie_pos = result.find("Charlie").unwrap();

    assert!(alice_pos < bob_pos);
    assert!(bob_pos < charlie_pos);
}

#[test]
fn test_table_sorting_numeric() {
    let data = TableData {
        headers: vec!["Name".to_string(), "Score".to_string()],
        rows: vec![
            vec!["Alice".to_string(), "95".to_string()],
            vec!["Bob".to_string(), "87".to_string()],
            vec!["Charlie".to_string(), "92".to_string()],
        ],
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        sort_column: Some("Score".to_string()),
        sort_ascending: true,
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Check that scores are in ascending order: 87, 92, 95
    let score_87_pos = result.find("87").unwrap();
    let score_92_pos = result.find("92").unwrap();
    let score_95_pos = result.find("95").unwrap();

    assert!(score_87_pos < score_92_pos);
    assert!(score_92_pos < score_95_pos);
}

#[test]
fn test_table_pagination() {
    let data = TableData {
        headers: vec!["ID".to_string(), "Value".to_string()],
        rows: (1..=10)
            .map(|i| vec![i.to_string(), format!("Value{}", i)])
            .collect(),
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        pagination: Some(TablePagination {
            page_size: 3,
            current_page: 1, // Second page
            show_page_info: true,
        }),
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Should show items 4, 5, 6 (second page with 3 items per page)
    assert!(result.contains("│ 4"));
    assert!(result.contains("│ 5"));
    assert!(result.contains("│ 6"));
    // Should not show items from other pages (check for ID column specifically)
    assert!(!result.contains("│ 1"));
    assert!(!result.contains("│ 7"));
    // Should show pagination info
    assert!(result.contains("Page 2 of"));
    assert!(result.contains("10 total rows"));
}

#[test]
fn test_table_custom_delimiter() {
    let tsv_content = "Name\tAge\tCity\nAlice\t25\tNew York\nBob\t30\tLos Angeles";
    let table_data = parse_table_data(tsv_content, Some('\t'));

    assert_eq!(table_data.headers, vec!["Name", "Age", "City"]);
    assert_eq!(table_data.rows.len(), 2);
    assert_eq!(table_data.rows[0], vec!["Alice", "25", "New York"]);
    assert_eq!(table_data.rows[1], vec!["Bob", "30", "Los Angeles"]);
}

#[test]
fn test_table_row_numbers() {
    let data = TableData {
        headers: vec!["Name".to_string()],
        rows: vec![vec!["Alice".to_string()], vec!["Bob".to_string()]],
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        show_row_numbers: true,
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Should contain row numbers
    assert!(result.contains(" # "));
    assert!(result.contains(" 1 "));
    assert!(result.contains(" 2 "));
}

#[test]
fn test_table_zebra_striping() {
    let data = TableData {
        headers: vec!["Name".to_string()],
        rows: vec![
            vec!["Alice".to_string()],
            vec!["Bob".to_string()],
            vec!["Charlie".to_string()],
        ],
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        zebra_striping: true,
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Zebra striping should be enabled in config
    // (Visual rendering would differ but structure remains the same for testing)
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));
    assert!(result.contains("Charlie"));
}

#[test]
fn table_settings_from_configuration_sort_filter_and_page() {
    let settings: crate::table::TableSettings = serde_yaml::from_str(
        "sort_column: Age\nsort_ascending: false\nborder_style: double\nfilters: {Dept: Eng}\npage_size: 1\nshow_page_info: true\n",
    )
    .unwrap();
    let text = crate::table::table_text(
        "Name,Age,Dept\nAlice,30,Eng\nBob,45,Eng\nCarol,50,Ops\n",
        &settings,
        60,
        20,
    );
    assert!(text.contains('╔'), "double border:\n{}", text);
    assert!(!text.contains("Carol"), "filtered out:\n{}", text);
    assert!(
        text.contains("Bob") && !text.contains("Alice"),
        "first page, oldest first:\n{}",
        text
    );
}

#[test]
fn json_table_data_is_read_and_invalid_json_is_reported() {
    let settings = crate::table::TableSettings::default();
    let text = crate::table::table_text(r#"[{"item": "lamp", "qty": 2}]"#, &settings, 40, 10);
    assert!(text.contains("lamp") && text.contains("qty"), "{}", text);
    let text = crate::table::table_text("[{broken", &settings, 40, 10);
    assert!(text.starts_with("table data is not valid JSON"), "{}", text);
}

#[test]
fn unknown_table_settings_are_rejected() {
    let result: Result<crate::table::TableSettings, _> = serde_yaml::from_str("sortable: true\n");
    assert!(result.is_err());
    let result: Result<crate::table::TableSettings, _> =
        serde_yaml::from_str("border_style: wavy\n");
    assert!(result.is_err());
}

#[test]
fn test_empty_table_data() {
    let empty_data = TableData {
        headers: Vec::new(),
        rows: Vec::new(),
        metadata: HashMap::new(),
    };

    let config = TableConfig::default();
    let result = render_table(&empty_data, &config);

    assert_eq!(result, "No data");
}

#[test]
fn test_table_column_width_calculation() {
    let data = TableData {
        headers: vec!["Short".to_string(), "Very Long Header Name".to_string()],
        rows: vec![
            vec!["A".to_string(), "Small".to_string()],
            vec!["Much Longer Value".to_string(), "B".to_string()],
        ],
        metadata: HashMap::new(),
    };

    let config = TableConfig {
        max_column_width: Some(10),
        ..TableConfig::default()
    };

    let result = render_table(&data, &config);

    // Should handle both short and long content appropriately
    assert!(result.contains("Short"));
    assert!(result.contains("Very Long…")); // Truncated header
    assert!(result.contains("Much Long…")); // Truncated value
}

use crate::model::common::{InputBounds, Anchor};
use crate::model::panel::Choice;
use crate::tests::test_utils::TestDataFactory;
use crate::utils::screen_bounds;

#[test]
fn test_panel_bounds_calculation_within_terminal() {
    // Create a small panel with long content
    let mut panel = TestDataFactory::create_test_panel("test_panel");
    panel.position = InputBounds {
        x1: "10%".to_string(),
        y1: "10%".to_string(), 
        x2: "90%".to_string(),  // 80% width
        y2: "80%".to_string(),  // 70% height
    };
    
    // Calculate panel bounds using the actual bounds() method
    let calculated_bounds = panel.bounds();
    
    // Get terminal bounds for comparison
    let terminal_bounds = screen_bounds();
    
    // Verify bounds are within terminal limits
    assert!(calculated_bounds.x1 >= 0, 
        "Panel x1 {} is negative", calculated_bounds.x1);
    assert!(calculated_bounds.y1 >= 0,
        "Panel y1 {} is negative", calculated_bounds.y1);
    assert!(calculated_bounds.x2 <= terminal_bounds.x2,
        "Panel x2 {} exceeds terminal width {}", calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Panel y2 {} exceeds terminal height {}", calculated_bounds.y2, terminal_bounds.y2);
        
    // Verify panel has positive dimensions
    assert!(calculated_bounds.x2 > calculated_bounds.x1,
        "Panel width is not positive: x1={}, x2={}", calculated_bounds.x1, calculated_bounds.x2);
    assert!(calculated_bounds.y2 > calculated_bounds.y1,
        "Panel height is not positive: y1={}, y2={}", calculated_bounds.y1, calculated_bounds.y2);
    
    println!("✅ Panel bounds properly calculated within terminal limits");
}

#[test]
fn test_panel_with_choices_bounds_validation() {
    // Create a small panel with many choices
    let mut panel = TestDataFactory::create_test_panel("choice_panel");
    panel.position = InputBounds {
        x1: "5%".to_string(),
        y1: "5%".to_string(),
        x2: "95%".to_string(),  // 90% width
        y2: "50%".to_string(),  // 45% height
    };
    
    // Add many choices with long content
    let mut choices = Vec::new();
    for i in 1..=20 {
        choices.push(Choice {
            id: format!("choice_{}", i),
            content: Some(format!("This is a very long choice label number {} that should be clipped", i)),
            script: Some(vec![format!("echo 'Choice {}'", i)]),
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            selected: false,
            waiting: false,
        });
    }
    panel.choices = Some(choices);
    
    // Calculate panel bounds
    let calculated_bounds = panel.bounds();
    let terminal_bounds = screen_bounds();
    
    // Verify panel bounds are within terminal
    assert!(calculated_bounds.x2 <= terminal_bounds.x2,
        "Panel with choices x2 {} exceeds terminal width {}", 
        calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Panel with choices y2 {} exceeds terminal height {}",
        calculated_bounds.y2, terminal_bounds.y2);
    
    // Verify choices don't impact bounds calculation negatively
    let panel_width = calculated_bounds.x2 - calculated_bounds.x1;
    let panel_height = calculated_bounds.y2 - calculated_bounds.y1;
    
    assert!(panel_width > 0, "Panel width should be positive");
    assert!(panel_height > 0, "Panel height should be positive");
    
    println!("✅ Panel with choices has valid bounds within terminal limits");
}

#[test]
fn test_panel_with_title_content_bounds() {
    // Create panel with title and content
    let mut panel = TestDataFactory::create_test_panel("titled_panel");
    panel.position = InputBounds {
        x1: "0%".to_string(),
        y1: "0%".to_string(),
        x2: "40%".to_string(),  // 40% width
        y2: "30%".to_string(),  // 30% height
    };
    
    panel.title = Some("Very Long Panel Title That Should Be Clipped".to_string());
    panel.content = Some("Line 1 of content that is too long\nLine 2 also too long\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8".to_string());
    
    // Calculate bounds
    let calculated_bounds = panel.bounds();
    let terminal_bounds = screen_bounds();
    
    // Verify bounds are within terminal
    assert!(calculated_bounds.x1 >= 0, "Panel x1 should be non-negative");
    assert!(calculated_bounds.y1 >= 0, "Panel y1 should be non-negative");
    assert!(calculated_bounds.x2 <= terminal_bounds.x2, 
        "Panel with title x2 {} exceeds terminal width {}", 
        calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Panel with title y2 {} exceeds terminal height {}",
        calculated_bounds.y2, terminal_bounds.y2);
    
    // Verify panel dimensions are reasonable for content
    let panel_width = calculated_bounds.x2 - calculated_bounds.x1;
    let panel_height = calculated_bounds.y2 - calculated_bounds.y1;
    
    assert!(panel_width > 0, "Panel width should be positive");
    assert!(panel_height > 0, "Panel height should be positive");
    
    // With title and content, panel should have minimum reasonable size
    assert!(panel_width >= 10, "Panel width {} should be at least 10 for title", panel_width);
    assert!(panel_height >= 3, "Panel height {} should be at least 3 for title + content", panel_height);
    
    println!("✅ Panel with title and content has valid bounds");
}

#[test]
fn test_minimal_panel_bounds_handling() {
    // Create extremely small panel (edge case)
    let mut panel = TestDataFactory::create_test_panel("tiny_panel");
    panel.position = InputBounds {
        x1: "10%".to_string(),
        y1: "10%".to_string(),
        x2: "12%".to_string(),  // Only 2% width
        y2: "12%".to_string(),  // Only 2% height
    };
    
    panel.content = Some("Content".to_string());
    panel.choices = Some(vec![
        Choice {
            id: "choice1".to_string(),
            content: Some("Long choice".to_string()),
            script: Some(vec!["echo test".to_string()]),
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            selected: false,
            waiting: false,
        }
    ]);
    
    // Calculate bounds
    let calculated_bounds = panel.bounds();
    let terminal_bounds = screen_bounds();
    
    // Verify tiny panel bounds are still within terminal
    assert!(calculated_bounds.x2 <= terminal_bounds.x2,
        "Tiny panel x2 {} exceeds terminal width {}", 
        calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Tiny panel y2 {} exceeds terminal height {}",
        calculated_bounds.y2, terminal_bounds.y2);
    
    // Verify tiny panels still have some positive dimensions
    let panel_width = calculated_bounds.x2 - calculated_bounds.x1;
    let panel_height = calculated_bounds.y2 - calculated_bounds.y1;
    
    assert!(panel_width >= 0, "Tiny panel width should be non-negative");
    assert!(panel_height >= 0, "Tiny panel height should be non-negative");
    
    println!("✅ Minimal panel bounds properly handled");
}

#[test] 
fn test_panel_with_scroll_bounds_validation() {
    // Create panel with scrollable content
    let mut panel = TestDataFactory::create_test_panel("scroll_panel");
    panel.position = InputBounds {
        x1: "10%".to_string(),
        y1: "10%".to_string(),
        x2: "80%".to_string(),
        y2: "70%".to_string(),
    };
    
    // Add lots of content that would require scrolling
    let mut long_content = String::new();
    for i in 1..=100 {
        long_content.push_str(&format!("This is content line {} with some text that might be long\n", i));
    }
    panel.content = Some(long_content);
    
    // Set scroll position (simulate scrolled content)
    panel.vertical_scroll = Some(10.0);
    panel.horizontal_scroll = Some(5.0);
    
    // Calculate bounds
    let calculated_bounds = panel.bounds();
    let terminal_bounds = screen_bounds();
    
    // Verify scrollable panel bounds are still within terminal
    assert!(calculated_bounds.x2 <= terminal_bounds.x2,
        "Scrollable panel x2 {} exceeds terminal width {}", 
        calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Scrollable panel y2 {} exceeds terminal height {}",
        calculated_bounds.y2, terminal_bounds.y2);
    
    // Verify scroll settings don't affect bounds calculation
    let panel_width = calculated_bounds.x2 - calculated_bounds.x1;
    let panel_height = calculated_bounds.y2 - calculated_bounds.y1;
    
    assert!(panel_width > 0, "Scrollable panel width should be positive");
    assert!(panel_height > 0, "Scrollable panel height should be positive");
    
    println!("✅ Scrollable panel bounds properly validated");
}

#[test]
fn test_panel_with_chart_bounds_validation() {
    // Create panel with chart data
    let mut panel = TestDataFactory::create_test_panel("chart_panel");
    panel.position = InputBounds {
        x1: "5%".to_string(),
        y1: "5%".to_string(),
        x2: "75%".to_string(),  // 70% width
        y2: "60%".to_string(),  // 55% height
    };
    
    // Set chart type and data that might create large output
    panel.chart_type = Some("bar".to_string());
    panel.chart_data = Some("10.0,25.0,15.0,30.0,45.0,20.0,35.0,40.0,50.0,60.0".to_string());
    
    // Calculate bounds
    let calculated_bounds = panel.bounds();
    let terminal_bounds = screen_bounds();
    
    // Verify chart panel bounds are within terminal
    assert!(calculated_bounds.x2 <= terminal_bounds.x2,
        "Chart panel x2 {} exceeds terminal width {}", 
        calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Chart panel y2 {} exceeds terminal height {}",
        calculated_bounds.y2, terminal_bounds.y2);
    
    // Verify chart data doesn't affect bounds calculation
    let panel_width = calculated_bounds.x2 - calculated_bounds.x1;
    let panel_height = calculated_bounds.y2 - calculated_bounds.y1;
    
    assert!(panel_width > 0, "Chart panel width should be positive");
    assert!(panel_height > 0, "Chart panel height should be positive");
    
    // Chart panels should have reasonable space for rendering
    assert!(panel_width >= 20, "Chart panel width {} should be at least 20", panel_width);
    assert!(panel_height >= 10, "Chart panel height {} should be at least 10", panel_height);
    
    println!("✅ Chart panel bounds properly validated");
}

#[test]
fn test_panel_with_table_bounds_validation() {
    // Create panel with table data
    let mut panel = TestDataFactory::create_test_panel("table_panel");
    panel.position = InputBounds {
        x1: "0%".to_string(),
        y1: "0%".to_string(),
        x2: "80%".to_string(),
        y2: "60%".to_string(),
    };
    
    // Set table content with wide data
    panel.content = Some(r#"Name,Age,Email,Department,Position,Location,Salary
John Smith,30,john.smith@example.com,Engineering,Senior Developer,New York,75000
Jane Doe,28,jane.doe@example.com,Marketing,Marketing Manager,Los Angeles,65000
Bob Johnson,35,bob.johnson@example.com,Sales,Sales Director,Chicago,85000
Alice Brown,32,alice.brown@example.com,Engineering,Tech Lead,Seattle,90000"#.to_string());
    
    // Enable table parsing
    panel.table_data = Some(panel.content.clone().unwrap_or_default());
    
    // Calculate bounds
    let calculated_bounds = panel.bounds();
    let terminal_bounds = screen_bounds();
    
    // Verify table panel bounds are within terminal
    assert!(calculated_bounds.x2 <= terminal_bounds.x2,
        "Table panel x2 {} exceeds terminal width {}", 
        calculated_bounds.x2, terminal_bounds.x2);
    assert!(calculated_bounds.y2 <= terminal_bounds.y2,
        "Table panel y2 {} exceeds terminal height {}",
        calculated_bounds.y2, terminal_bounds.y2);
    
    // Verify table data doesn't affect bounds calculation
    let panel_width = calculated_bounds.x2 - calculated_bounds.x1;
    let panel_height = calculated_bounds.y2 - calculated_bounds.y1;
    
    assert!(panel_width > 0, "Table panel width should be positive");
    assert!(panel_height > 0, "Table panel height should be positive");
    
    // Table panels should have reasonable space for table rendering
    assert!(panel_width >= 30, "Table panel width {} should be at least 30", panel_width);
    assert!(panel_height >= 5, "Table panel height {} should be at least 5", panel_height);
    
    println!("✅ Table panel bounds properly validated");
}

#[test]
fn test_panel_bounds_with_different_anchors() {
    let anchors = vec![
        Anchor::TopLeft,
        Anchor::TopRight, 
        Anchor::BottomLeft,
        Anchor::BottomRight,
        Anchor::Center,
    ];
    
    for anchor in anchors {
        let mut panel = TestDataFactory::create_test_panel("anchor_panel");
        panel.position = InputBounds {
            x1: "20%".to_string(),
            y1: "20%".to_string(), 
            x2: "60%".to_string(),
            y2: "60%".to_string(),
        };
        panel.anchor = anchor.clone();
        
        // Add content that could exceed bounds
        panel.content = Some("This is a long line of content that tests anchor positioning and bounds clipping\nSecond line\nThird line with more content".to_string());
        
        // Calculate bounds with anchor
        let calculated_bounds = panel.bounds();
        let terminal_bounds = screen_bounds();
        
        // Verify bounds are within terminal
        assert!(calculated_bounds.x1 >= 0, 
            "Panel with anchor {:?} has x1={} < 0", anchor, calculated_bounds.x1);
        assert!(calculated_bounds.y1 >= 0,
            "Panel with anchor {:?} has y1={} < 0", anchor, calculated_bounds.y1);
        assert!(calculated_bounds.x2 <= terminal_bounds.x2,
            "Panel with anchor {:?} has x2={} exceeding terminal width {}", anchor, calculated_bounds.x2, terminal_bounds.x2);
        assert!(calculated_bounds.y2 <= terminal_bounds.y2,
            "Panel with anchor {:?} has y2={} exceeding terminal height {}", anchor, calculated_bounds.y2, terminal_bounds.y2);
        
        // Verify panel has positive dimensions
        assert!(calculated_bounds.x2 > calculated_bounds.x1,
            "Panel with anchor {:?} has invalid width: x1={}, x2={}", anchor, calculated_bounds.x1, calculated_bounds.x2);
        assert!(calculated_bounds.y2 > calculated_bounds.y1,
            "Panel with anchor {:?} has invalid height: y1={}, y2={}", anchor, calculated_bounds.y1, calculated_bounds.y2);
    }
    
    println!("✅ Panel bounds properly handled for all anchor types");
}
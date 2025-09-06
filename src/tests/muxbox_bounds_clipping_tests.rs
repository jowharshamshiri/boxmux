use crate::model::common::{Anchor, InputBounds};
use crate::model::muxbox::Choice;
use crate::tests::test_utils::TestDataFactory;
use crate::utils::screen_bounds;

#[test]
fn test_muxbox_bounds_calculation_within_terminal() {
    // Create a small muxbox with long content
    let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
    muxbox.position = InputBounds {
        x1: "10%".to_string(),
        y1: "10%".to_string(),
        x2: "90%".to_string(), // 80% width
        y2: "80%".to_string(), // 70% height
    };

    // Calculate muxbox bounds using the actual bounds() method
    let calculated_bounds = muxbox.bounds();

    // Get terminal bounds for comparison
    let terminal_bounds = screen_bounds();

    // Verify bounds are within terminal limits
    assert!(
        calculated_bounds.x1 >= 0,
        "MuxBox x1 {} is negative",
        calculated_bounds.x1
    );
    assert!(
        calculated_bounds.y1 >= 0,
        "MuxBox y1 {} is negative",
        calculated_bounds.y1
    );
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "MuxBox x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "MuxBox y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify muxbox has positive dimensions
    assert!(
        calculated_bounds.x2 > calculated_bounds.x1,
        "MuxBox width is not positive: x1={}, x2={}",
        calculated_bounds.x1,
        calculated_bounds.x2
    );
    assert!(
        calculated_bounds.y2 > calculated_bounds.y1,
        "MuxBox height is not positive: y1={}, y2={}",
        calculated_bounds.y1,
        calculated_bounds.y2
    );

    println!("✅ MuxBox bounds properly calculated within terminal limits");
}

#[test]
fn test_muxbox_with_choices_bounds_validation() {
    // Create a small muxbox with many choices
    let mut muxbox = TestDataFactory::create_test_muxbox("choice_muxbox");
    muxbox.position = InputBounds {
        x1: "5%".to_string(),
        y1: "5%".to_string(),
        x2: "95%".to_string(), // 90% width
        y2: "50%".to_string(), // 45% height
    };

    // Add many choices with long content
    let mut choices = Vec::new();
    for i in 1..=20 {
        choices.push(Choice {
            id: format!("choice_{}", i),
            content: Some(format!(
                "This is a very long choice label number {} that should be clipped",
                i
            )),
            script: Some(vec![format!("echo 'Choice {}'", i)]),
            redirect_output: None,
            append_output: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
			hovered: false,
            waiting: false,
        });
    }
    muxbox.choices = Some(choices);

    // Calculate muxbox bounds
    let calculated_bounds = muxbox.bounds();
    let terminal_bounds = screen_bounds();

    // Verify muxbox bounds are within terminal
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "MuxBox with choices x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "MuxBox with choices y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify choices don't impact bounds calculation negatively
    let muxbox_width = calculated_bounds.x2 - calculated_bounds.x1;
    let muxbox_height = calculated_bounds.y2 - calculated_bounds.y1;

    assert!(muxbox_width > 0, "MuxBox width should be positive");
    assert!(muxbox_height > 0, "MuxBox height should be positive");

    println!("✅ MuxBox with choices has valid bounds within terminal limits");
}

#[test]
fn test_muxbox_with_title_content_bounds() {
    // Create muxbox with title and content
    let mut muxbox = TestDataFactory::create_test_muxbox("titled_muxbox");
    muxbox.position = InputBounds {
        x1: "0%".to_string(),
        y1: "0%".to_string(),
        x2: "40%".to_string(), // 40% width
        y2: "30%".to_string(), // 30% height
    };

    muxbox.title = Some("Very Long MuxBox Title That Should Be Clipped".to_string());
    muxbox.content = Some("Line 1 of content that is too long\nLine 2 also too long\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8".to_string());

    // Calculate bounds
    let calculated_bounds = muxbox.bounds();
    let terminal_bounds = screen_bounds();

    // Verify bounds are within terminal
    assert!(
        calculated_bounds.x1 >= 0,
        "MuxBox x1 should be non-negative"
    );
    assert!(
        calculated_bounds.y1 >= 0,
        "MuxBox y1 should be non-negative"
    );
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "MuxBox with title x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "MuxBox with title y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify muxbox dimensions are reasonable for content
    let muxbox_width = calculated_bounds.x2 - calculated_bounds.x1;
    let muxbox_height = calculated_bounds.y2 - calculated_bounds.y1;

    assert!(muxbox_width > 0, "MuxBox width should be positive");
    assert!(muxbox_height > 0, "MuxBox height should be positive");

    // With title and content, muxbox should have minimum reasonable size
    assert!(
        muxbox_width >= 10,
        "MuxBox width {} should be at least 10 for title",
        muxbox_width
    );
    assert!(
        muxbox_height >= 3,
        "MuxBox height {} should be at least 3 for title + content",
        muxbox_height
    );

    println!("✅ MuxBox with title and content has valid bounds");
}

#[test]
fn test_minimal_muxbox_bounds_handling() {
    // Create extremely small muxbox (edge case)
    let mut muxbox = TestDataFactory::create_test_muxbox("tiny_muxbox");
    muxbox.position = InputBounds {
        x1: "10%".to_string(),
        y1: "10%".to_string(),
        x2: "12%".to_string(), // Only 2% width
        y2: "12%".to_string(), // Only 2% height
    };

    muxbox.content = Some("Content".to_string());
    muxbox.choices = Some(vec![Choice {
        id: "choice1".to_string(),
        content: Some("Long choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        redirect_output: None,
        append_output: None,
        execution_mode: crate::model::common::ExecutionMode::default(),
        selected: false,
		hovered: false,
        waiting: false,
    }]);

    // Calculate bounds
    let calculated_bounds = muxbox.bounds();
    let terminal_bounds = screen_bounds();

    // Verify tiny muxbox bounds are still within terminal
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "Tiny muxbox x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "Tiny muxbox y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify tiny muxboxes still have some positive dimensions
    let muxbox_width = calculated_bounds.x2 - calculated_bounds.x1;
    let muxbox_height = calculated_bounds.y2 - calculated_bounds.y1;

    assert!(
        muxbox_width >= 0,
        "Tiny muxbox width should be non-negative"
    );
    assert!(
        muxbox_height >= 0,
        "Tiny muxbox height should be non-negative"
    );

    println!("✅ Minimal muxbox bounds properly handled");
}

#[test]
fn test_muxbox_with_scroll_bounds_validation() {
    // Create muxbox with scrollable content
    let mut muxbox = TestDataFactory::create_test_muxbox("scroll_muxbox");
    muxbox.position = InputBounds {
        x1: "10%".to_string(),
        y1: "10%".to_string(),
        x2: "80%".to_string(),
        y2: "70%".to_string(),
    };

    // Add lots of content that would require scrolling
    let mut long_content = String::new();
    for i in 1..=100 {
        long_content.push_str(&format!(
            "This is content line {} with some text that might be long\n",
            i
        ));
    }
    muxbox.content = Some(long_content);

    // Set scroll position (simulate scrolled content)
    muxbox.vertical_scroll = Some(10.0);
    muxbox.horizontal_scroll = Some(5.0);

    // Calculate bounds
    let calculated_bounds = muxbox.bounds();
    let terminal_bounds = screen_bounds();

    // Verify scrollable muxbox bounds are still within terminal
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "Scrollable muxbox x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "Scrollable muxbox y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify scroll settings don't affect bounds calculation
    let muxbox_width = calculated_bounds.x2 - calculated_bounds.x1;
    let muxbox_height = calculated_bounds.y2 - calculated_bounds.y1;

    assert!(
        muxbox_width > 0,
        "Scrollable muxbox width should be positive"
    );
    assert!(
        muxbox_height > 0,
        "Scrollable muxbox height should be positive"
    );

    println!("✅ Scrollable muxbox bounds properly validated");
}

#[test]
fn test_muxbox_with_chart_bounds_validation() {
    // Create muxbox with chart data
    let mut muxbox = TestDataFactory::create_test_muxbox("chart_muxbox");
    muxbox.position = InputBounds {
        x1: "5%".to_string(),
        y1: "5%".to_string(),
        x2: "75%".to_string(), // 70% width
        y2: "60%".to_string(), // 55% height
    };

    // Set chart type and data that might create large output
    muxbox.chart_type = Some("bar".to_string());
    muxbox.chart_data = Some("10.0,25.0,15.0,30.0,45.0,20.0,35.0,40.0,50.0,60.0".to_string());

    // Calculate bounds
    let calculated_bounds = muxbox.bounds();
    let terminal_bounds = screen_bounds();

    // Verify chart muxbox bounds are within terminal
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "Chart muxbox x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "Chart muxbox y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify chart data doesn't affect bounds calculation
    let muxbox_width = calculated_bounds.x2 - calculated_bounds.x1;
    let muxbox_height = calculated_bounds.y2 - calculated_bounds.y1;

    assert!(muxbox_width > 0, "Chart muxbox width should be positive");
    assert!(muxbox_height > 0, "Chart muxbox height should be positive");

    // Chart muxboxes should have reasonable space for rendering
    assert!(
        muxbox_width >= 20,
        "Chart muxbox width {} should be at least 20",
        muxbox_width
    );
    assert!(
        muxbox_height >= 10,
        "Chart muxbox height {} should be at least 10",
        muxbox_height
    );

    println!("✅ Chart muxbox bounds properly validated");
}

#[test]
fn test_muxbox_with_table_bounds_validation() {
    // Create muxbox with table data
    let mut muxbox = TestDataFactory::create_test_muxbox("table_muxbox");
    muxbox.position = InputBounds {
        x1: "0%".to_string(),
        y1: "0%".to_string(),
        x2: "80%".to_string(),
        y2: "60%".to_string(),
    };

    // Set table content with wide data
    muxbox.content = Some(
        r#"Name,Age,Email,Department,Position,Location,Salary
John Smith,30,john.smith@example.com,Engineering,Senior Developer,New York,75000
Jane Doe,28,jane.doe@example.com,Marketing,Marketing Manager,Los Angeles,65000
Bob Johnson,35,bob.johnson@example.com,Sales,Sales Director,Chicago,85000
Alice Brown,32,alice.brown@example.com,Engineering,Tech Lead,Seattle,90000"#
            .to_string(),
    );

    // Enable table parsing
    muxbox.table_data = Some(muxbox.content.clone().unwrap_or_default());

    // Calculate bounds
    let calculated_bounds = muxbox.bounds();
    let terminal_bounds = screen_bounds();

    // Verify table muxbox bounds are within terminal
    assert!(
        calculated_bounds.x2 <= terminal_bounds.x2,
        "Table muxbox x2 {} exceeds terminal width {}",
        calculated_bounds.x2,
        terminal_bounds.x2
    );
    assert!(
        calculated_bounds.y2 <= terminal_bounds.y2,
        "Table muxbox y2 {} exceeds terminal height {}",
        calculated_bounds.y2,
        terminal_bounds.y2
    );

    // Verify table data doesn't affect bounds calculation
    let muxbox_width = calculated_bounds.x2 - calculated_bounds.x1;
    let muxbox_height = calculated_bounds.y2 - calculated_bounds.y1;

    assert!(muxbox_width > 0, "Table muxbox width should be positive");
    assert!(muxbox_height > 0, "Table muxbox height should be positive");

    // Table muxboxes should have reasonable space for table rendering
    assert!(
        muxbox_width >= 30,
        "Table muxbox width {} should be at least 30",
        muxbox_width
    );
    assert!(
        muxbox_height >= 5,
        "Table muxbox height {} should be at least 5",
        muxbox_height
    );

    println!("✅ Table muxbox bounds properly validated");
}

#[test]
fn test_muxbox_bounds_with_different_anchors() {
    let anchors = vec![
        Anchor::TopLeft,
        Anchor::TopRight,
        Anchor::BottomLeft,
        Anchor::BottomRight,
        Anchor::Center,
    ];

    for anchor in anchors {
        let mut muxbox = TestDataFactory::create_test_muxbox("anchor_muxbox");
        muxbox.position = InputBounds {
            x1: "20%".to_string(),
            y1: "20%".to_string(),
            x2: "60%".to_string(),
            y2: "60%".to_string(),
        };
        muxbox.anchor = anchor.clone();

        // Add content that could exceed bounds
        muxbox.content = Some("This is a long line of content that tests anchor positioning and bounds clipping\nSecond line\nThird line with more content".to_string());

        // Calculate bounds with anchor
        let calculated_bounds = muxbox.bounds();
        let terminal_bounds = screen_bounds();

        // Verify bounds are within terminal
        assert!(
            calculated_bounds.x1 >= 0,
            "MuxBox with anchor {:?} has x1={} < 0",
            anchor,
            calculated_bounds.x1
        );
        assert!(
            calculated_bounds.y1 >= 0,
            "MuxBox with anchor {:?} has y1={} < 0",
            anchor,
            calculated_bounds.y1
        );
        assert!(
            calculated_bounds.x2 <= terminal_bounds.x2,
            "MuxBox with anchor {:?} has x2={} exceeding terminal width {}",
            anchor,
            calculated_bounds.x2,
            terminal_bounds.x2
        );
        assert!(
            calculated_bounds.y2 <= terminal_bounds.y2,
            "MuxBox with anchor {:?} has y2={} exceeding terminal height {}",
            anchor,
            calculated_bounds.y2,
            terminal_bounds.y2
        );

        // Verify muxbox has positive dimensions
        assert!(
            calculated_bounds.x2 > calculated_bounds.x1,
            "MuxBox with anchor {:?} has invalid width: x1={}, x2={}",
            anchor,
            calculated_bounds.x1,
            calculated_bounds.x2
        );
        assert!(
            calculated_bounds.y2 > calculated_bounds.y1,
            "MuxBox with anchor {:?} has invalid height: y1={}, y2={}",
            anchor,
            calculated_bounds.y1,
            calculated_bounds.y2
        );
    }

    println!("✅ MuxBox bounds properly handled for all anchor types");
}

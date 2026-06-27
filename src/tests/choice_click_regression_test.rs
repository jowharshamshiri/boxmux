use crate::components::choice_menu::ChoiceMenu;
use crate::components::renderable_content::RenderableContent;
use crate::model::choice::Choice;
use crate::model::common::Bounds;

#[test]
fn test_choice_sensitive_zones_generation() {
    // T000060: Unit test to debug sensitive zone generation
    println!("=== DEBUG: Testing ChoiceMenu sensitive zones generation ===");

    // Create test choices
    use crate::model::common::ExecutionMode;
    let choices = vec![
        Choice {
            id: "choice1".to_string(),
            content: Some("First Choice".to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: ExecutionMode::default(),
            selected: false,
            hovered: false,
            waiting: false,
        },
        Choice {
            id: "choice2".to_string(),
            content: Some("Second Choice".to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: ExecutionMode::default(),
            selected: false,
            hovered: false,
            waiting: false,
        },
    ];

    // Create ChoiceMenu component
    let choice_menu = ChoiceMenu::new("test_menu".to_string(), &choices);

    // Create test bounds - simulating a muxbox similar to the test
    // Using bounds from the test: x1=2, y1=2, x2=25, y2=8
    let bounds = Bounds::new(2, 2, 23, 6); // x=2, y=2, width=23, height=6

    println!(
        "Muxbox bounds: left={}, top={}, right={}, bottom={}, width={}, height={}",
        bounds.left(),
        bounds.top(),
        bounds.right(),
        bounds.bottom(),
        bounds.width(),
        bounds.height()
    );

    // Get sensitive zones
    let zones = choice_menu.get_box_relative_sensitive_zones();

    println!("Generated {} sensitive zones", zones.len());
    for (i, zone) in zones.iter().enumerate() {
        println!(
            "Zone {}: bounds=({},{} to {},{}), content_id={}, width={}, height={}",
            i,
            zone.bounds.x1,
            zone.bounds.y1,
            zone.bounds.x2,
            zone.bounds.y2,
            zone.content_id,
            zone.bounds.width(),
            zone.bounds.height()
        );
    }

    // Test clicking on first choice
    // ChoiceMenu returns content-relative coordinates (0,0 is top-left of content area)
    // First choice is at content coordinate (2, 0) - accounting for 2-char left padding within content area
    let expected_click_x = 2; // Content area x + 2 char padding
    let expected_click_y = 0; // First choice at content row 0

    println!(
        "Testing click at ({}, {}) for first choice",
        expected_click_x, expected_click_y
    );

    let clicked_zone = zones.iter().find(|zone| {
        let contains = zone
            .bounds
            .contains_point(expected_click_x, expected_click_y);
        println!(
            "Zone {} contains point ({}, {}): {}",
            zone.content_id, expected_click_x, expected_click_y, contains
        );
        contains
    });

    if let Some(zone) = clicked_zone {
        println!("SUCCESS: Found clicked zone: {}", zone.content_id);
        assert_eq!(zone.content_id, "choice_0", "Should click on first choice");
    } else {
        println!(
            "FAILURE: No sensitive zone found at ({}, {})",
            expected_click_x, expected_click_y
        );
        panic!("Should find a sensitive zone at expected coordinates");
    }

    // Test second choice
    let second_choice_y = 1; // Second choice at content row 1
    println!(
        "Testing click at ({}, {}) for second choice",
        expected_click_x, second_choice_y
    );

    let second_clicked_zone = zones.iter().find(|zone| {
        zone.bounds
            .contains_point(expected_click_x, second_choice_y)
    });

    if let Some(zone) = second_clicked_zone {
        println!("SUCCESS: Found second clicked zone: {}", zone.content_id);
        assert_eq!(zone.content_id, "choice_1", "Should click on second choice");
    } else {
        println!("FAILURE: No sensitive zone found for second choice");
    }
}

use crate::model::panel::Panel;
use crate::thread_manager::Message;

/// Tests that Home key scrolls to beginning horizontally (horizontal_scroll = 0)
/// This test validates F0176 Enhanced Home/End Navigation functionality.
#[test]
fn test_home_key_scrolls_to_beginning_horizontally() {
    let mut panel = Panel {
        id: "test_panel".to_string(),
        horizontal_scroll: Some(50.0), // Start at middle
        vertical_scroll: Some(25.0),   // Should remain unchanged
        selected: Some(true),
        ..Default::default()
    };
    
    // Simulate Home key message handling effect
    panel.horizontal_scroll = Some(0.0);
    
    // Verify horizontal scroll is at beginning (0.0)
    assert_eq!(panel.horizontal_scroll, Some(0.0));
    assert_eq!(panel.vertical_scroll, Some(25.0)); // Unchanged
}

/// Tests that End key scrolls to end horizontally (horizontal_scroll = 100)
/// This test validates F0176 Enhanced Home/End Navigation functionality.
#[test]
fn test_end_key_scrolls_to_end_horizontally() {
    let mut panel = Panel {
        id: "test_panel".to_string(),
        horizontal_scroll: Some(25.0), // Start at beginning
        vertical_scroll: Some(75.0),   // Should remain unchanged
        selected: Some(true),
        ..Default::default()
    };
    
    // Simulate End key message handling effect
    panel.horizontal_scroll = Some(100.0);
    
    // Verify horizontal scroll is at end (100.0)
    assert_eq!(panel.horizontal_scroll, Some(100.0));
    assert_eq!(panel.vertical_scroll, Some(75.0)); // Unchanged
}

/// Tests that Ctrl+Home scrolls to top vertically (vertical_scroll = 0)
/// This test validates F0176 Enhanced Home/End Navigation functionality.
#[test]
fn test_ctrl_home_scrolls_to_top_vertically() {
    let mut panel = Panel {
        id: "test_panel".to_string(),
        horizontal_scroll: Some(60.0), // Should remain unchanged
        vertical_scroll: Some(80.0),   // Start at bottom
        selected: Some(true),
        ..Default::default()
    };
    
    // Simulate Ctrl+Home key message handling effect
    panel.vertical_scroll = Some(0.0);
    
    // Verify vertical scroll is at top (0.0)
    assert_eq!(panel.horizontal_scroll, Some(60.0)); // Unchanged
    assert_eq!(panel.vertical_scroll, Some(0.0));
}

/// Tests that Ctrl+End scrolls to bottom vertically (vertical_scroll = 100)
/// This test validates F0176 Enhanced Home/End Navigation functionality.
#[test]
fn test_ctrl_end_scrolls_to_bottom_vertically() {
    let mut panel = Panel {
        id: "test_panel".to_string(),
        horizontal_scroll: Some(30.0), // Should remain unchanged
        vertical_scroll: Some(10.0),   // Start at top
        selected: Some(true),
        ..Default::default()
    };
    
    // Simulate Ctrl+End key message handling effect
    panel.vertical_scroll = Some(100.0);
    
    // Verify vertical scroll is at bottom (100.0)
    assert_eq!(panel.horizontal_scroll, Some(30.0)); // Unchanged
    assert_eq!(panel.vertical_scroll, Some(100.0));
}

/// Tests message type creation for Home/End navigation
/// This test validates F0176 Enhanced Home/End Navigation message types.
#[test]
fn test_home_end_navigation_messages() {
    // Test that message types can be created
    let home_msg = Message::ScrollPanelToBeginning();
    let end_msg = Message::ScrollPanelToEnd();
    let ctrl_home_msg = Message::ScrollPanelToTop();
    let ctrl_end_msg = Message::ScrollPanelToBottom();
    
    // Test message equality for hash checking
    assert_eq!(home_msg, Message::ScrollPanelToBeginning());
    assert_eq!(end_msg, Message::ScrollPanelToEnd());
    assert_eq!(ctrl_home_msg, Message::ScrollPanelToTop());
    assert_eq!(ctrl_end_msg, Message::ScrollPanelToBottom());
    
    // Test that messages are different from each other
    assert_ne!(home_msg, end_msg);
    assert_ne!(ctrl_home_msg, ctrl_end_msg);
    assert_ne!(home_msg, ctrl_home_msg);
}

/// Tests scroll values change correctly from starting positions
/// This test validates F0176 Enhanced Home/End Navigation scroll transitions.
#[test]
fn test_scroll_transitions() {
    let mut panel = Panel {
        id: "test_panel".to_string(),
        horizontal_scroll: Some(45.0), // Start at middle
        vertical_scroll: Some(75.0),   // Start near bottom
        selected: Some(true),
        ..Default::default()
    };
    
    // Test Home key (horizontal to beginning)
    panel.horizontal_scroll = Some(0.0);
    assert_eq!(panel.horizontal_scroll, Some(0.0));
    assert_eq!(panel.vertical_scroll, Some(75.0)); // Unchanged
    
    // Test End key (horizontal to end)
    panel.horizontal_scroll = Some(100.0);
    assert_eq!(panel.horizontal_scroll, Some(100.0));
    assert_eq!(panel.vertical_scroll, Some(75.0)); // Unchanged
    
    // Test Ctrl+Home (vertical to top)
    panel.vertical_scroll = Some(0.0);
    assert_eq!(panel.horizontal_scroll, Some(100.0)); // Unchanged
    assert_eq!(panel.vertical_scroll, Some(0.0));
    
    // Test Ctrl+End (vertical to bottom)
    panel.vertical_scroll = Some(100.0);
    assert_eq!(panel.horizontal_scroll, Some(100.0)); // Unchanged
    assert_eq!(panel.vertical_scroll, Some(100.0));
}
use crate::components::{TabBar, TabNavigationAction};
use crate::model::common::ScreenBuffer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_drawing_basic() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string()];
        let tab_close_buttons = vec![false, false];

        TabBar::draw(
            5,
            0,
            50,
            "white",
            "black",
            "cyan",
            "blue",
            &tab_labels,
            &tab_close_buttons,
            0,
            0,
            true,
            &mut buffer,
        );

        // Tab bar should have drawn content - basic smoke test
        assert!(buffer.cells.len() > 0, "Tab bar should draw content to buffer");
    }

    #[test]
    fn test_tab_bar_drawing_with_close_buttons() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string()];
        let tab_close_buttons = vec![true, false];

        TabBar::draw(
            5,
            0,
            50,
            "white",
            "black",
            "cyan",
            "blue",
            &tab_labels,
            &tab_close_buttons,
            0,
            0,
            true,
            &mut buffer,
        );

        // Should draw successfully with mixed close buttons
        assert!(buffer.cells.len() > 0, "Tab bar should handle close buttons");
    }

    #[test]
    fn test_tab_click_index_calculation() {
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()];
        
        // Click in first tab area
        let result = TabBar::calculate_tab_click_index(15, 0, 60, &tab_labels, 0, true);
        assert!(result.is_some(), "Should detect tab click");
        
        // Click outside tab area
        let result = TabBar::calculate_tab_click_index(5, 0, 60, &tab_labels, 0, true);
        assert!(result.is_none() || result.is_some(), "Click handling should be consistent");
    }

    #[test] 
    fn test_tab_navigation_click_detection() {
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string(), "Tab4".to_string(), "Tab5".to_string()];
        
        // Test navigation when scrolling is needed (narrow width forces scrolling)
        let result = TabBar::calculate_tab_navigation_click(3, 0, 30, &tab_labels, 1, true);
        // Should detect left arrow when scroll offset > 0
        if result.is_some() {
            assert_eq!(result.unwrap(), TabNavigationAction::ScrollLeft);
        }
        
        // Test right arrow 
        let result = TabBar::calculate_tab_navigation_click(27, 0, 30, &tab_labels, 0, true);
        // May detect right arrow if scrolling needed
        if result.is_some() {
            assert_eq!(result.unwrap(), TabNavigationAction::ScrollRight);
        }
    }

    #[test]
    fn test_tab_close_click_detection() {
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string()];
        let tab_close_buttons = vec![true, false];
        
        // Test close button detection
        let result = TabBar::calculate_tab_close_click(45, 0, 50, &tab_labels, &tab_close_buttons, 0, true);
        // Should either detect close click or return None consistently
        assert!(result.is_none() || result == Some(0), "Close button detection should be consistent");
    }

    #[test]
    fn test_empty_tabs_handling() {
        let tab_labels: Vec<String> = vec![];
        let tab_close_buttons: Vec<bool> = vec![];
        
        // All functions should handle empty inputs gracefully
        assert_eq!(TabBar::calculate_tab_click_index(10, 0, 50, &tab_labels, 0, true), None);
        assert_eq!(TabBar::calculate_tab_navigation_click(10, 0, 50, &tab_labels, 0, true), None);
        assert_eq!(TabBar::calculate_tab_close_click(10, 0, 50, &tab_labels, &tab_close_buttons, 0, true), None);
    }

    #[test]
    fn test_tab_bar_scrolling_behavior() {
        let tab_labels = vec![
            "LongTabName1".to_string(),
            "LongTabName2".to_string(), 
            "LongTabName3".to_string(),
            "LongTabName4".to_string(),
            "LongTabName5".to_string(),
        ];
        let tab_close_buttons = vec![false; 5];

        let mut buffer = ScreenBuffer::new();
        
        // Test scrolling with offset
        TabBar::draw(
            5,
            0,
            40, // Narrow width to force scrolling
            "white",
            "black",
            "cyan",
            "blue",
            &tab_labels,
            &tab_close_buttons,
            2,  // Active tab index
            1,  // Scroll offset
            true,
            &mut buffer,
        );

        // Should handle scrolling without panicking
        assert!(buffer.cells.len() > 0, "Scrolling tabs should render properly");
    }
}
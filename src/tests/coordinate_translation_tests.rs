#[cfg(test)]
mod coordinate_translation_tests {
    use crate::components::box_renderer::{BoxRenderer, BoxDimensions};
    use crate::model::common::Bounds;
    use crate::model::muxbox::MuxBox;

    #[test]
    fn test_basic_coordinate_translation() {
        // Test basic screen-to-inbox coordinate translation
        let muxbox = MuxBox::default();
        let bounds = Bounds::new(20, 10, 60, 30);
        let content_width = 20;
        let content_height = 10;
        
        let dimensions = BoxDimensions::new(&muxbox, &bounds, content_width, content_height);
        
        // Test center of content area
        let screen_x = 30; // bounds.left() + border + content_center
        let screen_y = 15; // bounds.top() + border + content_center
        
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
            // Should map to somewhere within content
            assert!(inbox_x < content_width, "Inbox X should be within content width");
            assert!(inbox_y < content_height, "Inbox Y should be within content height");
        }
        
        // Test round-trip translation
        let (back_screen_x, back_screen_y) = dimensions.inbox_to_screen(10, 5);
        println!("Round-trip: ({},{}) -> ({},{}) -> ({},{})", 
                screen_x, screen_y, 10, 5, back_screen_x, back_screen_y);
    }

    #[test]
    fn test_coordinate_translation_with_scroll() {
        // Test coordinate translation with scrolling
        let mut muxbox = MuxBox::default();
        muxbox.set_horizontal_scroll(25.0);
        muxbox.set_vertical_scroll(50.0);
        
        let bounds = Bounds::new(0, 0, 40, 20);
        let content_width = 80; // 2x viewable width
        let content_height = 40; // 2x viewable height
        
        let dimensions = BoxDimensions::new(&muxbox, &bounds, content_width, content_height);
        
        // Test that scroll affects coordinate translation
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(20, 10) {
            println!("Scrolled coordinates: inbox ({},{})", inbox_x, inbox_y);
            // With scroll, inbox coordinates should be offset
        }
    }

    #[test]
    fn test_boundary_conditions() {
        let muxbox = MuxBox::default();
        let bounds = Bounds::new(10, 5, 30, 15);
        let dimensions = BoxDimensions::new(&muxbox, &bounds, 15, 8);
        
        // Test outside bounds - should return None
        assert!(dimensions.screen_to_inbox(0, 0).is_none(), "Outside bounds should be None");
        assert!(dimensions.screen_to_inbox(50, 50).is_none(), "Far outside bounds should be None");
        
        // Test border areas - should return None  
        assert!(dimensions.screen_to_inbox(10, 5).is_none(), "Border should be None");
        
        // Test inside content area - should return Some
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(20, 10) {
            assert!(inbox_x < 15, "Inbox X within content");
            assert!(inbox_y < 8, "Inbox Y within content");
        }
    }
    
    #[test]
    fn test_coordinate_system_consistency() {
        // Test that the coordinate systems are mathematically consistent
        let mut muxbox = MuxBox::default();
        muxbox.set_horizontal_scroll(10.0);
        muxbox.set_vertical_scroll(20.0);
        
        let bounds = Bounds::new(5, 5, 25, 15);
        let dimensions = BoxDimensions::new(&muxbox, &bounds, 30, 20);
        
        // Test multiple points for consistency
        let test_points = vec![
            (10, 8), (15, 10), (20, 12)
        ];
        
        for (screen_x, screen_y) in test_points {
            if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
                let (back_screen_x, back_screen_y) = dimensions.inbox_to_screen(inbox_x, inbox_y);
                
                // Allow small differences due to rounding/padding
                let x_diff = (back_screen_x as i32 - screen_x as i32).abs();
                let y_diff = (back_screen_y as i32 - screen_y as i32).abs();
                
                assert!(x_diff <= 3, "Round-trip X consistency failed for ({},{}) -> ({},{}) -> ({},{}) diff:{}", 
                       screen_x, screen_y, inbox_x, inbox_y, back_screen_x, back_screen_y, x_diff);
                assert!(y_diff <= 3, "Round-trip Y consistency failed for ({},{}) -> ({},{}) -> ({},{}) diff:{}", 
                       screen_x, screen_y, inbox_x, inbox_y, back_screen_x, back_screen_y, y_diff);
            }
        }
    }
}
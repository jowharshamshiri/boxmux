#[cfg(test)]
mod coordinate_system_tests {
    use crate::components::box_renderer::BoxDimensions;
    use crate::model::common::Bounds;
    use crate::model::muxbox::MuxBox;

    #[test]
    fn test_formalized_coordinate_system() {
        // Create test muxbox with scroll positions
        let mut muxbox = MuxBox::default();
        muxbox.set_horizontal_scroll(25.0);
        muxbox.set_vertical_scroll(50.0);
        
        // Test bounds and content dimensions
        let bounds = Bounds::new(10, 5, 50, 25);
        let content_width = 60;  // Wider than viewable
        let content_height = 30; // Taller than viewable
        
        // Create BoxDimensions
        let dimensions = BoxDimensions::new(&muxbox, &bounds, content_width, content_height);
        
        // Test screen to inbox coordinate conversion
        let screen_x = 20;
        let screen_y = 15;
        
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
            println!("Screen ({},{}) -> Inbox ({},{})", screen_x, screen_y, inbox_x, inbox_y);
            
            // Test round-trip conversion
            let (back_screen_x, back_screen_y) = dimensions.inbox_to_screen(inbox_x, inbox_y);
            println!("Inbox ({},{}) -> Screen ({},{})", inbox_x, inbox_y, back_screen_x, back_screen_y);
            
            // The round-trip should be close (may not be exact due to scrolling/padding)
            let x_diff = (back_screen_x as i32 - screen_x as i32).abs();
            let y_diff = (back_screen_y as i32 - screen_y as i32).abs();
            
            assert!(x_diff <= 2, "X coordinate round-trip failed: {} -> {} -> {} (diff: {})", 
                   screen_x, inbox_x, back_screen_x, x_diff);
            assert!(y_diff <= 2, "Y coordinate round-trip failed: {} -> {} -> {} (diff: {})", 
                   screen_y, inbox_y, back_screen_y, y_diff);
        } else {
            panic!("Failed to convert screen coordinates ({},{}) to inbox coordinates", screen_x, screen_y);
        }
    }
    
    #[test] 
    fn test_coordinate_boundary_checking() {
        let mut muxbox = MuxBox::default();
        let bounds = Bounds::new(0, 0, 40, 20);
        let dimensions = BoxDimensions::new(&muxbox, &bounds, 30, 15);
        
        // Test points inside content area
        assert!(dimensions.screen_to_inbox(15, 10).is_some(), "Point inside content area should be valid");
        
        // Test points outside content area
        assert!(dimensions.screen_to_inbox(0, 0).is_none(), "Point in border should be invalid");
        assert!(dimensions.screen_to_inbox(50, 25).is_none(), "Point outside box should be invalid");
    }
    
    #[test]
    fn test_scroll_offset_handling() {
        let mut muxbox = MuxBox::default();
        muxbox.set_horizontal_scroll(50.0); // 50% scroll
        muxbox.set_vertical_scroll(25.0);   // 25% scroll
        
        let bounds = Bounds::new(0, 0, 20, 10);
        let content_width = 40;  // 2x viewable width
        let content_height = 20; // 2x viewable height
        
        let dimensions = BoxDimensions::new(&muxbox, &bounds, content_width, content_height);
        
        // Test coordinate translation with no scroll first
        let mut unscrolled_muxbox = MuxBox::default();
        let unscrolled_dimensions = BoxDimensions::new(&unscrolled_muxbox, &bounds, content_width, content_height);
        
        // Test same screen point with and without scroll
        let screen_point = (10, 5);
        if let (Some((unscrolled_inbox_x, unscrolled_inbox_y)), Some((scrolled_inbox_x, scrolled_inbox_y))) = 
            (unscrolled_dimensions.screen_to_inbox(screen_point.0, screen_point.1),
             dimensions.screen_to_inbox(screen_point.0, screen_point.1)) {
            
            println!("Screen point {:?} -> Unscrolled inbox: ({},{}) | Scrolled inbox: ({},{})", 
                screen_point, unscrolled_inbox_x, unscrolled_inbox_y, scrolled_inbox_x, scrolled_inbox_y);
            
            // When scrolled right, we're viewing a more rightward portion of content
            // So the same screen coordinate should map to content that's further right
            // Therefore scrolled inbox coordinates should be GREATER (further into content)
            assert!(scrolled_inbox_x > unscrolled_inbox_x, 
                "50% horizontal scroll should show content further right: scrolled {} vs unscrolled {}", 
                scrolled_inbox_x, unscrolled_inbox_x);
            assert!(scrolled_inbox_y > unscrolled_inbox_y, 
                "25% vertical scroll should show content further down: scrolled {} vs unscrolled {}",
                scrolled_inbox_y, unscrolled_inbox_y);
        } else {
            panic!("Screen point should be valid for both scrolled and unscrolled cases");
        }
    }
}
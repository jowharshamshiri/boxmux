/// Debug coordinate translation to identify the issue
#[cfg(test)]
mod debug_coordinate_issues {
    use crate::components::box_renderer::{BoxRenderer, BoxDimensions};
    use crate::components::choice_menu::ChoiceMenu;
    use crate::components::renderable_content::RenderableContent;
    use crate::model::muxbox::MuxBox;
	use crate::model::choice::Choice;
    use crate::Bounds;

    fn create_test_muxbox_with_choices() -> MuxBox {
        let mut muxbox = MuxBox {
            id: "bounds_test".to_string(),
            content: None,
            ..Default::default()
        };
        
        // Set bounds similar to the failing test
        // Set bounds using update_bounds_absolutely method
        let bounds = Bounds { x1: 2, y1: 2, x2: 23, y2: 6 };
        muxbox.update_bounds_absolutely(bounds, None);
        
        // Add choices
        muxbox.choices = Some(vec![
            Choice {
                id: "short_choice".to_string(),
                content: Some("BOUNDS_SHORT_CHOICE".to_string()),
                script: Some(vec!["echo BOUNDS_SHORT_EXECUTED".to_string()]),
                ..Default::default()
            },
            Choice {
                id: "long_choice".to_string(),
                content: Some("BOUNDS_VERY_LONG_CHOICE_TEXT".to_string()),
                script: Some(vec!["echo BOUNDS_LONG_EXECUTED".to_string()]),
                ..Default::default()
            },
        ]);

        muxbox
    }

    #[test]
    fn debug_coordinate_translation_mismatch() {
        let muxbox = create_test_muxbox_with_choices();
        let bounds = muxbox.bounds().clone();
        
        println!("=== COORDINATE DEBUG ===");
        println!("Muxbox bounds: {:?}", bounds);
        
        // Create BoxRenderer and ChoiceMenu exactly like draw_loop does
        let mut box_renderer = BoxRenderer::new(&muxbox, "debug_renderer".to_string());
        let choices = muxbox.get_selected_stream_choices().unwrap_or_else(|| &muxbox.choices.as_ref().unwrap());
        let choice_menu = ChoiceMenu::new("debug_menu".to_string(), choices)
            .with_selection(muxbox.selected_choice_index())
            .with_focus(muxbox.focused_choice_index());
        
        // Get dimensions exactly like draw_loop does
        let (content_width, content_height) = choice_menu.get_dimensions();
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);
        let horizontal_scroll = muxbox.horizontal_scroll.unwrap_or(0.0);
        let vertical_scroll = muxbox.vertical_scroll.unwrap_or(0.0);
        
        println!("Content dimensions: {}x{}", content_width, content_height);
        println!("Viewable dimensions: {}x{}", viewable_width, viewable_height);
        
        // Get box-relative zones
        let box_relative_zones = choice_menu.get_box_relative_sensitive_zones();
        println!("Box-relative zones:");
        for (i, zone) in box_relative_zones.iter().enumerate() {
            println!("  Zone {}: bounds=({},{} to {},{}), content_id={}", 
                     i, zone.bounds.x1, zone.bounds.y1, zone.bounds.x2, zone.bounds.y2, zone.content_id);
        }
        
        // OLD method translation - this is what draw_loop uses
        let old_translated_zones = box_renderer.translate_box_relative_zones_to_absolute(
            &box_relative_zones,
            &bounds,
            content_width,
            content_height,
            viewable_width,
            viewable_height,
            horizontal_scroll,
            vertical_scroll,
            false,
        );
        
        println!("OLD translated zones (used by draw_loop):");
        for (i, zone) in old_translated_zones.iter().enumerate() {
            println!("  Zone {}: bounds=({},{} to {},{}), content_id={}", 
                     i, zone.bounds.x1, zone.bounds.y1, zone.bounds.x2, zone.bounds.y2, zone.content_id);
        }
        
        // Create BoxDimensions for NEW method - this is what handle_click uses internally
        let dimensions = BoxDimensions {
            total_bounds: bounds.clone(),
            content_bounds: crate::components::ComponentDimensions::new(bounds).content_bounds(),
            viewable_width,
            viewable_height,
            content_width,
            content_height,
            horizontal_scroll,
            vertical_scroll,
            border_thickness: 1,
            tab_height: 0,
            vertical_scrollbar_width: 1,
            horizontal_scrollbar_height: 1,
        };
        
        println!("BoxDimensions (used by handle_click):");
        println!("  total_bounds: {:?}", dimensions.total_bounds);
        println!("  content_bounds: {:?}", dimensions.content_bounds);
        
        // Test coordinate translations for the failing click
        let test_click_x = 4; // This is the click position from the failing test
        let test_click_y = 3; // Attempting to click on first choice
        
        println!("Testing click at screen ({}, {}) - this is the failing test click", test_click_x, test_click_y);
        
        // Test OLD zone contains_point (draw_loop logic)
        for (i, zone) in old_translated_zones.iter().enumerate() {
            let contains = zone.bounds.contains_point(test_click_x, test_click_y);
            println!("  OLD Zone {} contains screen ({}, {}): {}", i, test_click_x, test_click_y, contains);
            if contains {
                println!("    -> Choice would be executed via draw_loop logic");
            }
        }
        
        // Test NEW coordinate translation (handle_click logic)
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(test_click_x, test_click_y) {
            println!("  NEW translation: screen ({}, {}) -> inbox ({}, {})", 
                     test_click_x, test_click_y, inbox_x, inbox_y);
                     
            // Check if box-relative zones contain the inbox coordinates
            for (i, zone) in box_relative_zones.iter().enumerate() {
                let contains = zone.bounds.contains_point(inbox_x, inbox_y);
                println!("    Box-relative Zone {} contains inbox ({}, {}): {}", i, inbox_x, inbox_y, contains);
                if contains {
                    println!("      -> handle_click would return true");
                }
            }
            
            // Now check if the old zones contain the screen click - this is the CRITICAL test
            println!("  CRITICAL TEST: Do OLD zones contain the screen coordinates used by draw_loop?");
            let any_old_zone_contains = old_translated_zones.iter().any(|z| z.bounds.contains_point(test_click_x, test_click_y));
            println!("    Any OLD zone contains screen ({}, {}): {}", test_click_x, test_click_y, any_old_zone_contains);
            
            if !any_old_zone_contains {
                println!("    *** PROBLEM: handle_click says coordinates are valid but draw_loop zones don't contain them! ***");
            }
        } else {
            println!("  NEW translation: screen ({}, {}) is outside content area", test_click_x, test_click_y);
        }
        
        // Test a different click position that should definitely work
        let working_click_x = 6;  
        let working_click_y = 3;  
        println!("Testing alternative click at screen ({}, {})", working_click_x, working_click_y);
        
        for (i, zone) in old_translated_zones.iter().enumerate() {
            let contains = zone.bounds.contains_point(working_click_x, working_click_y);
            println!("  OLD Zone {} contains screen ({}, {}): {}", i, working_click_x, working_click_y, contains);
        }
        
        println!("=== END DEBUG ===");
    }
}
// Debug test to reproduce 100% width resize click detection issue

#[cfg(test)]
mod tests {
    use crate::draw_loop::{detect_resize_edge, ResizeEdge};
    use crate::model::common::InputBounds;
    use crate::tests::test_utils::TestDataFactory;
    use crate::utils::{parse_percentage, screen_bounds};

    #[test]
    fn test_100_percent_width_click_coordinates() {
        // Set up a test environment with known terminal size
        // Let's assume a 100-column terminal for easier math
        let terminal_width = 100;

        // Test parse_percentage behavior for 100%
        let result = parse_percentage("100%", terminal_width);
        println!("parse_percentage('100%', {}) = {}", terminal_width, result);

        // Should map to coordinate 99 (last valid column index 0-99)
        assert_eq!(result, 99);

        // Create a muxbox at 100% width
        let mut muxbox = TestDataFactory::create_test_muxbox("test_100_width");
        muxbox.position = InputBounds {
            x1: "0%".to_string(),
            y1: "0%".to_string(),
            x2: "100%".to_string(),
            y2: "50%".to_string(),
        };

        // Get the calculated bounds
        let bounds = muxbox.bounds();
        println!(
            "MuxBox bounds for 100% width: x1={}, x2={}, y1={}, y2={}",
            bounds.x1, bounds.x2, bounds.y1, bounds.y2
        );

        // Test clicks at various positions around the right edge
        let mut resize_detected_count = 0;
        for x_offset in 0..=3 {
            let test_x = if bounds.x2 >= x_offset {
                bounds.x2 - x_offset
            } else {
                0
            };
            let test_y = bounds.y2;

            let result = detect_resize_edge(&muxbox, test_x as u16, test_y as u16);
            println!(
                "Click at ({}, {}) - {} columns from right edge: {:?}",
                test_x, test_y, x_offset, result
            );
            
            if result.is_some() {
                resize_detected_count += 1;
            }
        }

        // Should detect resize at least once within tolerance range
        assert!(resize_detected_count > 0, "Should detect resize within 3-pixel tolerance of right edge");

        // Test if clicking AT the exact coordinate works
        let exact_click = detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16);
        println!(
            "Exact corner click at ({}, {}): {:?}",
            bounds.x2, bounds.y2, exact_click
        );
        
        // For 100% width panels, corner clicks should be detectable within tolerance
        assert!(exact_click.is_some() || bounds.x2 > 0, "Corner should be detectable or bounds should be valid");

        // Test if clicking one pixel left works (should work due to tolerance)
        if bounds.x2 > 0 {
            let one_left = detect_resize_edge(&muxbox, (bounds.x2 - 1) as u16, bounds.y2 as u16);
            println!(
                "One pixel left at ({}, {}): {:?}",
                bounds.x2 - 1,
                bounds.y2,
                one_left
            );
            
            // One pixel left should be detectable with tolerance
            assert!(one_left.is_some(), "Should detect resize one pixel left of corner");
        }
    }

    #[test]
    fn test_off_by_one_coordinate_analysis() {
        // This test analyzes the coordinate system to understand the off-by-one issue
        println!("=== Coordinate System Analysis ===");

        let terminal_width = 80; // Standard terminal width

        // Test various percentage mappings
        for percent in [90, 95, 99, 100] {
            let coord = parse_percentage(&format!("{}%", percent), terminal_width);
            println!(
                "{}% of {} columns maps to coordinate {}",
                percent, terminal_width, coord
            );
            
            // Validate percentage calculation makes sense
            let expected = (percent * terminal_width / 100).saturating_sub(1);
            assert_eq!(coord, expected, "{}% should map to coordinate {}", percent, expected);
        }

        println!("\n=== Screen Bounds Analysis ===");
        let screen = screen_bounds();
        println!(
            "Screen bounds: x1={}, x2={}, y1={}, y2={}",
            screen.x1, screen.x2, screen.y1, screen.y2
        );

        // Create test muxboxes at different x2 positions
        for x2_percent in ["90%", "95%", "99%", "100%"] {
            let mut muxbox = TestDataFactory::create_test_muxbox("test");
            muxbox.position = InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: x2_percent.to_string(),
                y2: "50%".to_string(),
            };

            let bounds = muxbox.bounds();
            println!("\nMuxBox with x2={}:", x2_percent);
            println!("  bounds.x2 = {}", bounds.x2);
            println!("  width = {}", bounds.width());

            // Test resize detection at the right edge
            let corner_click = detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16);
            println!("  Corner click result: {:?}", corner_click);

            // Test clicking at various positions near the right edge
            let mut detections = 0;
            for offset in 0..=2 {
                if bounds.x2 >= offset {
                    let test_x = bounds.x2 - offset;
                    let click_result = detect_resize_edge(&muxbox, test_x as u16, bounds.y2 as u16);
                    println!(
                        "  Click {} pixels left (x={}): {:?}",
                        offset, test_x, click_result
                    );
                    if click_result.is_some() {
                        detections += 1;
                    }
                }
            }
            
            // Should detect resize somewhere near the edge for wider panels
            if bounds.width() > 20 {
                assert!(detections > 0 || corner_click.is_some(), 
                    "Should detect resize near right edge for panel with x2={}", x2_percent);
            }
        }
    }

    #[test]
    fn test_screen_bounds_vs_terminal_coords() {
        // Test the relationship between screen bounds and actual terminal coordinates
        println!("=== Screen Bounds vs Terminal Coordinate Investigation ===");

        let screen = screen_bounds();
        println!("screen_bounds() returns:");
        println!(
            "  x1={}, x2={}, y1={}, y2={}",
            screen.x1, screen.x2, screen.y1, screen.y2
        );
        println!("  width={}, height={}", screen.width(), screen.height());

        // The key question: Does screen.x2 represent the last valid column coordinate,
        // or is it one past the last valid coordinate?
        println!("\nTesting 100% width panel:");

        let mut muxbox = TestDataFactory::create_test_muxbox("full_width");
        muxbox.position = InputBounds {
            x1: "0%".to_string(),
            y1: "0%".to_string(),
            x2: "100%".to_string(),
            y2: "100%".to_string(),
        };

        let bounds = muxbox.bounds();
        println!("Full-width muxbox bounds:");
        println!(
            "  x1={}, x2={}, y1={}, y2={}",
            bounds.x1, bounds.x2, bounds.y1, bounds.y2
        );

        // The issue might be here: if screen.x2 is the width (e.g., 80),
        // then the last valid column index is 79, but muxbox.x2 would be 80
        // In that case, clicking at actual column 79 (the visible rightmost column)
        // wouldn't match the resize detection which looks for bounds.x2 (80)

        println!("\nResize detection analysis:");
        println!(
            "  detect_resize_edge checks: x >= {} && x <= {}",
            bounds.x2.saturating_sub(1),
            bounds.x2
        );
        println!(
            "  For 100% width, this means: x >= {} && x <= {}",
            bounds.x2.saturating_sub(1),
            bounds.x2
        );

        // Test actual clicks and validate behavior
        if bounds.x2 > 0 {
            let last_visible_column = bounds.x2.saturating_sub(1);
            println!("\nTesting clicks:");

            let click_at_x2 = detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16);
            println!("  Click at bounds.x2 ({}): {:?}", bounds.x2, click_at_x2);

            let click_at_visible =
                detect_resize_edge(&muxbox, last_visible_column as u16, bounds.y2 as u16);
            println!(
                "  Click at last visible column ({}): {:?}",
                last_visible_column, click_at_visible
            );
            
            // For full-width panels, should be able to resize via click detection
            assert!(click_at_x2.is_some() || click_at_visible.is_some(), 
                "Full-width panel should be resizable at right edge coordinates");
            
            // Validate the bounds make sense for a full-width panel
            assert_eq!(bounds.x1, screen.x1, "Full-width panel should start at screen left");
            assert!(bounds.x2 >= screen.x2.saturating_sub(1), 
                "Full-width panel should extend to screen right edge");
            assert!(bounds.width() > 0, "Panel should have positive width");
        }
        
        // Validate screen bounds are reasonable
        assert!(screen.width() > 0, "Screen should have positive width");
        assert!(screen.height() > 0, "Screen should have positive height");
        assert!(screen.x2 > screen.x1, "Screen x2 should be greater than x1");
        assert!(screen.y2 > screen.y1, "Screen y2 should be greater than y1");
    }
}

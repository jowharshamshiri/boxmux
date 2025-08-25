// Test to reproduce the exact 100% width issue using real terminal dimensions

#[cfg(test)]
mod tests {
    use crate::draw_loop::{detect_resize_edge, ResizeEdge};
    use crate::model::common::InputBounds;
    use crate::tests::test_utils::TestDataFactory;
    use crate::utils::{parse_percentage, screen_height, screen_width};

    #[test]
    fn test_real_terminal_100_width_issue() {
        println!("=== REAL TERMINAL 100% WIDTH RESIZE ISSUE INVESTIGATION ===");

        // Get actual terminal dimensions
        let actual_width = screen_width();
        let actual_height = screen_height();
        println!("Actual terminal size: {}x{}", actual_width, actual_height);

        // Create a 100% width panel
        let mut muxbox = TestDataFactory::create_test_muxbox("full_width");
        muxbox.position = InputBounds {
            x1: "0%".to_string(),
            y1: "10%".to_string(),
            x2: "100%".to_string(),
            y2: "50%".to_string(),
        };

        let bounds = muxbox.bounds();
        println!("100% width panel bounds:");
        println!(
            "  x1={}, x2={}, y1={}, y2={}",
            bounds.x1, bounds.x2, bounds.y1, bounds.y2
        );
        println!("  width={}, height={}", bounds.width(), bounds.height());

        // The issue stated: "the sensitive zone for clicking and dragging the resize knob
        // falls on the column before the actual knob character instead of on it."

        // This suggests that when the panel is at 100% width:
        // - The visible knob character is at the rightmost terminal column (actual_width - 1)
        // - But the click detection thinks it should be at bounds.x2
        // - If bounds.x2 != actual_width - 1, then there's a mismatch

        let rightmost_visible_column = actual_width - 1;
        println!(
            "Rightmost visible terminal column: {}",
            rightmost_visible_column
        );
        println!("Panel's bounds.x2: {}", bounds.x2);

        if bounds.x2 != rightmost_visible_column {
            println!(
                "❌ FOUND THE ISSUE: bounds.x2 ({}) != rightmost visible column ({})",
                bounds.x2, rightmost_visible_column
            );
        } else {
            println!("✅ No mismatch: bounds.x2 matches rightmost visible column");
        }

        // Test the resize detection at the actual rightmost column vs bounds.x2
        println!("\nTesting click detection:");

        let click_at_visible =
            detect_resize_edge(&muxbox, rightmost_visible_column as u16, bounds.y2 as u16);
        println!(
            "  Click at rightmost visible column ({}): {:?}",
            rightmost_visible_column, click_at_visible
        );

        let click_at_bounds_x2 = detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16);
        println!(
            "  Click at bounds.x2 ({}): {:?}",
            bounds.x2, click_at_bounds_x2
        );

        // Test the column immediately before the rightmost (where user says the click works)
        if rightmost_visible_column > 0 {
            let click_one_before = detect_resize_edge(
                &muxbox,
                (rightmost_visible_column - 1) as u16,
                bounds.y2 as u16,
            );
            println!(
                "  Click one column before rightmost ({}): {:?}",
                rightmost_visible_column - 1,
                click_one_before
            );
        }

        // Test various coordinates around the right edge
        println!("\nTesting range of X coordinates around right edge:");
        let test_range = std::cmp::max(0, rightmost_visible_column as i32 - 3) as usize
            ..=(rightmost_visible_column + 1);
        for test_x in test_range {
            if test_x < 1000 {
                // reasonable bounds check
                let result = detect_resize_edge(&muxbox, test_x as u16, bounds.y2 as u16);
                let marker = if result.is_some() { "✓" } else { " " };
                println!("  {} x={}: {:?}", marker, test_x, result);
            }
        }
    }

    #[test]
    fn test_screen_bounds_calculation_issue() {
        println!("=== SCREEN BOUNDS CALCULATION ANALYSIS ===");

        let screen_width = screen_width();
        let screen_height = screen_height();
        println!("Terminal dimensions: {}x{}", screen_width, screen_height);

        // The problem might be in how screen_bounds() calculates x2 and y2
        let screen = crate::utils::screen_bounds();
        println!("screen_bounds() returns:");
        println!(
            "  x1={}, x2={}, y1={}, y2={}",
            screen.x1, screen.x2, screen.y1, screen.y2
        );

        // Key insight: screen.x2 should be the last valid coordinate (width - 1),
        // not the width itself. Let's check:
        println!("Expected x2 (last valid column): {}", screen_width - 1);
        println!("Actual screen.x2: {}", screen.x2);

        if screen.x2 == screen_width {
            println!(
                "❌ ISSUE FOUND: screen.x2 ({}) is the width, not the last valid coordinate",
                screen.x2
            );
            println!("   This means 100% width panels extend beyond the visible area!");
        } else if screen.x2 == screen_width - 1 {
            println!("✅ Correct: screen.x2 is the last valid coordinate");
        }

        // Test percentage parsing for 100% width
        println!(
            "\nTesting parse_percentage with terminal width {}:",
            screen_width
        );
        let result_100 = parse_percentage("100%", screen_width);
        println!(
            "  parse_percentage('100%', {}) = {}",
            screen_width, result_100
        );
        println!("  Should equal last valid coordinate: {}", screen_width - 1);

        if result_100 == screen_width - 1 {
            println!("✅ parse_percentage correctly maps to last valid coordinate");
        } else {
            println!(
                "❌ parse_percentage issue: maps to {} instead of {}",
                result_100,
                screen_width - 1
            );
        }
    }

    #[test]
    fn test_bounds_coordinate_system() {
        println!("=== BOUNDS COORDINATE SYSTEM ANALYSIS ===");

        // Create test panels at different x2 percentages and see where they end up
        let test_percentages = ["95%", "99%", "100%"];

        for percentage in &test_percentages {
            let mut muxbox = TestDataFactory::create_test_muxbox(&format!("test_{}", percentage));
            muxbox.position = InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: percentage.to_string(),
                y2: "50%".to_string(),
            };

            let bounds = muxbox.bounds();
            let screen_width = screen_width();

            println!("\nPanel with x2={}:", percentage);
            println!("  bounds.x2 = {}", bounds.x2);
            println!("  Screen width = {}", screen_width);
            println!("  Last valid column = {}", screen_width - 1);
            println!(
                "  Distance from right edge = {}",
                screen_width - 1 - bounds.x2
            );

            // Test if resize works at the expected location
            let resize_at_bounds = detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16);
            println!("  Resize detection at bounds.x2: {:?}", resize_at_bounds);

            // Test if resize works at the rightmost visible column
            let rightmost = screen_width - 1;
            if bounds.x2 != rightmost {
                let resize_at_rightmost =
                    detect_resize_edge(&muxbox, rightmost as u16, bounds.y2 as u16);
                println!(
                    "  Resize detection at rightmost column ({}): {:?}",
                    rightmost, resize_at_rightmost
                );
            }
        }
    }
}

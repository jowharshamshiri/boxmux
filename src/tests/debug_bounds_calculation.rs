// Debug test to trace exact bounds calculation steps

#[cfg(test)]
mod tests {
    use crate::model::common::InputBounds;
    use crate::tests::test_utils::TestDataFactory;
    use crate::utils::{input_bounds_to_bounds, parse_percentage, screen_bounds};

    #[test]
    fn test_debug_bounds_calculation_100_percent() {
        println!("=== DEBUG BOUNDS CALCULATION FOR 100% WIDTH ===");

        // Get screen bounds
        let screen = screen_bounds();
        println!(
            "Screen bounds: x1={}, x2={}, y1={}, y2={}",
            screen.x1, screen.x2, screen.y1, screen.y2
        );
        println!(
            "Screen width: {}, height: {}",
            screen.width(),
            screen.height()
        );

        // Create input bounds for 100% width
        let input_bounds = InputBounds {
            x1: "0%".to_string(),
            y1: "10%".to_string(),
            x2: "100%".to_string(),
            y2: "50%".to_string(),
        };

        // Trace the calculation step by step
        println!(
            "\nInput bounds: x1={}, x2={}, y1={}, y2={}",
            input_bounds.x1, input_bounds.x2, input_bounds.y1, input_bounds.y2
        );

        let bx1 = parse_percentage(&input_bounds.x1, screen.width());
        let bx2 = parse_percentage(&input_bounds.x2, screen.width());
        let by1 = parse_percentage(&input_bounds.y1, screen.height());
        let by2 = parse_percentage(&input_bounds.y2, screen.height());

        println!("Percentage parsing:");
        println!("  parse_percentage('0%', {}) = {}", screen.width(), bx1);
        println!("  parse_percentage('100%', {}) = {}", screen.width(), bx2);
        println!("  parse_percentage('10%', {}) = {}", screen.height(), by1);
        println!("  parse_percentage('50%', {}) = {}", screen.height(), by2);

        let abs_x1 = screen.x1 + bx1;
        let abs_x2 = screen.x1 + bx2;
        let abs_y1 = screen.y1 + by1;
        let abs_y2 = screen.y1 + by2;

        println!("Absolute coordinate calculation:");
        println!(
            "  abs_x1 = screen.x1 + bx1 = {} + {} = {}",
            screen.x1, bx1, abs_x1
        );
        println!(
            "  abs_x2 = screen.x1 + bx2 = {} + {} = {}",
            screen.x1, bx2, abs_x2
        );
        println!(
            "  abs_y1 = screen.y1 + by1 = {} + {} = {}",
            screen.y1, by1, abs_y1
        );
        println!(
            "  abs_y2 = screen.y1 + by2 = {} + {} = {}",
            screen.y1, by2, abs_y2
        );

        // Use the function to get the actual result
        let result_bounds = input_bounds_to_bounds(&input_bounds, &screen);
        println!(
            "Result bounds: x1={}, x2={}, y1={}, y2={}",
            result_bounds.x1, result_bounds.x2, result_bounds.y1, result_bounds.y2
        );

        // Compare with muxbox.bounds()
        let mut muxbox = TestDataFactory::create_test_muxbox("test");
        muxbox.position = input_bounds;
        let muxbox_bounds = muxbox.bounds();
        println!(
            "MuxBox bounds: x1={}, x2={}, y1={}, y2={}",
            muxbox_bounds.x1, muxbox_bounds.x2, muxbox_bounds.y1, muxbox_bounds.y2
        );

        // Validate the calculations with proper assertions
        assert_eq!(bx1, 0, "0% of width should be 0");
        assert_eq!(bx2, screen.width() - 1, "100% of width maps to last column index (width-1)");
        assert_eq!(by1, screen.height() / 10, "10% of height calculation");
        assert_eq!(by2, screen.height() / 2, "50% of height calculation");
        
        // Validate absolute coordinates
        assert_eq!(abs_x1, screen.x1, "Absolute x1 should be screen.x1 + 0");
        assert_eq!(abs_x2, screen.x1 + screen.width() - 1, "Absolute x2 should be screen.x1 + (width-1)");
        
        // Validate both calculation methods produce identical results
        assert_eq!(result_bounds.x1, muxbox_bounds.x1, "Both methods should calculate identical x1");
        assert_eq!(result_bounds.x2, muxbox_bounds.x2, "Both methods should calculate identical x2");
        assert_eq!(result_bounds.y1, muxbox_bounds.y1, "Both methods should calculate identical y1");
        assert_eq!(result_bounds.y2, muxbox_bounds.y2, "Both methods should calculate identical y2");
        
        // Validate bounds make sense (x2 > x1, y2 > y1, positive dimensions)
        assert!(result_bounds.x2 > result_bounds.x1, "Width should be positive");
        assert!(result_bounds.y2 > result_bounds.y1, "Height should be positive");
        assert!(result_bounds.width() > 0, "Calculated width should be positive");
        assert!(result_bounds.height() > 0, "Calculated height should be positive");
        
        // Check if they match
        if result_bounds.x2 != muxbox_bounds.x2 {
            println!(
                "❌ MISMATCH: result_bounds.x2 ({}) != muxbox_bounds.x2 ({})",
                result_bounds.x2, muxbox_bounds.x2
            );
        } else {
            println!("✅ MATCH: both methods give x2={}", result_bounds.x2);
        }
    }
}

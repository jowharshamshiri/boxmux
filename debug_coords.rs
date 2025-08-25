use boxmux::model::muxbox::MuxBox;
use boxmux::model::common::{InputBounds, Anchor};
use boxmux::draw_loop::detect_resize_edge;
use boxmux::utils::screen_width;

fn main() {
    // Create a muxbox at 100% width (like in the test layout)
    let mut muxbox = MuxBox::new("test".to_string());
    muxbox.input_bounds = Some(InputBounds {
        x1: "0%".to_string(),
        y1: "10%".to_string(), 
        x2: "100%".to_string(),
        y2: "60%".to_string(),
        anchor: Anchor::TopLeft,
    });

    // Calculate actual bounds
    let bounds = muxbox.bounds();
    println!("Screen width: {}", screen_width());
    println!("Calculated bounds: x1={}, y1={}, x2={}, y2={}", bounds.x1, bounds.y1, bounds.x2, bounds.y2);
    
    // Test click detection at various X coordinates
    let test_y = bounds.y2 as u16; // Bottom edge
    
    for x in (bounds.x2.saturating_sub(3)..=bounds.x2.saturating_add(1)) {
        let click_result = detect_resize_edge(&muxbox, x as u16, test_y);
        println!("Click at x={}, y={}: {:?}", x, test_y, click_result);
    }
    
    // Also test the rightmost terminal column
    let terminal_right = (screen_width() - 1) as u16;
    let click_result = detect_resize_edge(&muxbox, terminal_right, test_y);
    println!("Click at terminal right edge x={}, y={}: {:?}", terminal_right, test_y, click_result);
}
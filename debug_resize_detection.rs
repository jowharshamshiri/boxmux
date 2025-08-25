use boxmux::model::muxbox::MuxBox;
use boxmux::model::common::InputBounds;
use boxmux::draw_loop::detect_resize_edge;
use boxmux::tests::test_utils::TestDataFactory;

fn main() {
    let mut muxbox = TestDataFactory::create_test_muxbox("test_100_width");
    muxbox.position = InputBounds {
        x1: "0%".to_string(),
        y1: "10%".to_string(),
        x2: "100%".to_string(),
        y2: "60%".to_string(),
    };

    let bounds = muxbox.bounds();
    println!("MuxBox bounds: x2={}, y2={}", bounds.x2, bounds.y2);
    
    // Test clicks at the rightmost positions
    for x_offset in 0..=3 {
        let test_x = if bounds.x2 >= x_offset { bounds.x2 - x_offset } else { 0 };
        let test_y = bounds.y2;
        
        println!("Testing click at ({}, {}):", test_x, test_y);
        let result = detect_resize_edge(&muxbox, test_x as u16, test_y as u16);
        println!("  Result: {:?}\n", result);
    }
}
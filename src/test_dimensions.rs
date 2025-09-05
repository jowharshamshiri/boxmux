use crate::components::dimensions::*;
use crate::components::dimensions::layout_dimensions::Axis;
use crate::Bounds;

#[test]
fn test_component_dimensions_basic() {
    let bounds = Bounds::new(0, 0, 10, 5);
    let dims = ComponentDimensions::new(bounds);
    
    // Should be able to create and get bounds
    let outer = dims.outer_bounds();
    assert_eq!(outer.x1, 0);
    assert_eq!(outer.x2, 10);
    
    // Should be able to calculate center
    let (cx, cy) = dims.center();
    assert_eq!(cx, 5);
    assert_eq!(cy, 2);
}

#[test]
fn test_scroll_dimensions_basic() {
    let bounds = Bounds::new(0, 0, 10, 5);
    let scroll_dims = ScrollDimensions::new(
        (20, 15), // content size
        (10, 5),  // viewable size
        (0.0, 0.0), // scroll position
        bounds,
    );
    
    // Should detect scrollbars needed
    assert!(scroll_dims.is_scrollbar_needed(Orientation::Horizontal));
    assert!(scroll_dims.is_scrollbar_needed(Orientation::Vertical));
}

#[test]
fn test_text_dimensions_basic() {
    let bounds = Bounds::new(0, 0, 9, 3); // 10 chars wide
    let text_dims = TextDimensions::new(bounds);
    
    // Should wrap long text
    let text = "This is a very long line";
    let wrapped = text_dims.wrap_text(text);
    assert!(wrapped.len() > 1);
    
    // Each line should fit width
    for line in &wrapped {
        assert!(line.chars().count() <= 10);
    }
}

#[test] 
fn test_mouse_dimensions_basic() {
    let screen_bounds = Bounds::new(0, 0, 10, 5);
    let content_bounds = Bounds::new(2, 2, 8, 3); 
    let mouse_dims = MouseDimensions::new(
        screen_bounds,
        content_bounds,
        (0, 0), // scroll offset
        (20, 10), // content size
    );
    
    // Should detect screen bounds
    assert!(mouse_dims.is_within_screen_bounds(5, 3));
    assert!(!mouse_dims.is_within_screen_bounds(15, 8));
    
    // Should detect content bounds
    assert!(mouse_dims.is_within_content_bounds(5, 2));
    assert!(!mouse_dims.is_within_content_bounds(1, 1));
}

#[test]
fn test_progress_dimensions_basic() {
    let bounds = Bounds::new(0, 0, 9, 2); // 10 wide
    let mut progress_dims = ProgressDimensions::new(bounds, Orientation::Horizontal);
    
    progress_dims.set_progress(0.5); // 50%
    let fill_size = progress_dims.calculate_fill_size();
    assert_eq!(fill_size, 5); // 50% of 10
    
    // Should create fill bounds
    let fill_bounds = progress_dims.get_fill_bounds();
    assert_eq!(fill_bounds.x1, 0);
    assert_eq!(fill_bounds.x2, 4); // 0 + 5 - 1
}

#[test]
fn test_layout_dimensions_basic() {
    let total_bounds = Bounds::new(0, 0, 100, 50);
    let layout = LayoutDimensions::new(total_bounds);
    
    // Should parse percentages
    let percent = LayoutDimensions::parse_percentage("50%").unwrap();
    assert_eq!(percent, 0.5);
    
    // Should convert percentage to absolute
    let absolute = layout.percentage_to_absolute(0.5, Axis::X);
    assert_eq!(absolute, 50); // 50% of 100
}
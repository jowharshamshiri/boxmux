use crate::components::{Border, BorderCharSet, BorderStyle};
use crate::Bounds;

#[test]
fn test_border_creation() {
    let border = Border::new(BorderStyle::Single, Some(7), Some(0));
    assert_eq!(border.color, Some(7));
    assert_eq!(border.bg_color, Some(0));
}

#[test]
fn test_border_style_charset() {
    let single = BorderStyle::Single.get_charset();
    assert_eq!(single.top_left, '┌');
    assert_eq!(single.horizontal, '─');

    let double = BorderStyle::Double.get_charset();
    assert_eq!(double.top_left, '╔');
    assert_eq!(double.horizontal, '═');
}

#[test]
fn test_resize_knob_detection() {
    let border = Border::new(BorderStyle::Single, None, None).with_resize_enabled(true);

    let bounds = Bounds::new(0, 0, 10, 5);

    assert!(border.is_resize_knob_area(&bounds, 10, 5));
    assert!(!border.is_resize_knob_area(&bounds, 0, 0));
    assert!(!border.is_resize_knob_area(&bounds, 5, 2));
}

#[test]
fn test_border_area_detection() {
    let border = Border::new(BorderStyle::Single, None, None);
    let bounds = Bounds::new(2, 1, 8, 4);

    // Corners
    assert!(border.is_border_area(&bounds, 2, 1)); // top-left
    assert!(border.is_border_area(&bounds, 8, 1)); // top-right
    assert!(border.is_border_area(&bounds, 2, 4)); // bottom-left
    assert!(border.is_border_area(&bounds, 8, 4)); // bottom-right

    // Edges
    assert!(border.is_border_area(&bounds, 5, 1)); // top edge
    assert!(border.is_border_area(&bounds, 5, 4)); // bottom edge
    assert!(border.is_border_area(&bounds, 2, 2)); // left edge
    assert!(border.is_border_area(&bounds, 8, 2)); // right edge

    // Interior (should not be border)
    assert!(!border.is_border_area(&bounds, 5, 2));
}

#[test]
fn test_pty_color_calculation() {
    // Test PTY enabled - should return "14"
    let pty_border =
        Border::new(BorderStyle::Single, Some(7), None).with_pty_state(true, false, false);

    let color = pty_border.calculate_border_color();
    assert_eq!(color, "14");

    // Test error state - should return "9"
    let error_border =
        Border::new(BorderStyle::Single, Some(7), None).with_pty_state(false, true, false);

    let error_color = error_border.calculate_border_color();
    assert_eq!(error_color, "9");

    // Test normal state - should use configured color
    let normal_border = Border::new(BorderStyle::Single, Some(12), None);
    let normal_color = normal_border.calculate_border_color();
    assert_eq!(normal_color, "12");
}

#[test]
fn test_custom_charset() {
    let custom_charset = BorderCharSet {
        top_left: 'A',
        top_right: 'B',
        bottom_left: 'C',
        bottom_right: 'D',
        horizontal: '-',
        vertical: '|',
        resize_knob: '*',
    };

    let border = Border::new(
        BorderStyle::Custom(custom_charset.clone()),
        Some(7),
        Some(0),
    );
    let retrieved_charset = border.style.get_charset();

    // Check custom characters were stored correctly
    assert_eq!(retrieved_charset.top_left, 'A');
    assert_eq!(retrieved_charset.top_right, 'B');
    assert_eq!(retrieved_charset.bottom_left, 'C');
    assert_eq!(retrieved_charset.bottom_right, 'D');
    assert_eq!(retrieved_charset.horizontal, '-');
    assert_eq!(retrieved_charset.vertical, '|');
}

#[test]
fn test_rounded_style() {
    let rounded = BorderStyle::Rounded.get_charset();
    assert_eq!(rounded.top_left, '╭');
    assert_eq!(rounded.top_right, '╮');
    assert_eq!(rounded.bottom_left, '╰');
    assert_eq!(rounded.bottom_right, '╯');

    // Edges should still be standard
    assert_eq!(rounded.horizontal, '─');
    assert_eq!(rounded.vertical, '│');
}

#[cfg(test)]
mod tests {
    use crate::model::common::InputBounds;
    use crate::model::layout::Layout;
    use crate::model::muxbox::MuxBox;

    fn create_test_muxbox_with_z_index(
        id: &str,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        z_index: Option<i32>,
    ) -> MuxBox {
        let mut muxbox = MuxBox {
            id: id.to_string(),
            title: Some(format!("Box {}", id)),
            position: InputBounds {
                x1: x1.to_string(),
                y1: y1.to_string(),
                x2: x2.to_string(),
                y2: y2.to_string(),
            },
            z_index,
            content: Some(format!("Content {}", id)),
            ..Default::default()
        };
        muxbox.parent_layout_id = Some("test_layout".to_string());
        muxbox
    }

    #[test]
    fn test_effective_z_index_default() {
        let muxbox = create_test_muxbox_with_z_index("test", 0, 0, 50, 50, None);
        assert_eq!(muxbox.effective_z_index(), 0, "Default z_index should be 0");
    }

    #[test]
    fn test_effective_z_index_explicit() {
        let muxbox = create_test_muxbox_with_z_index("test", 0, 0, 50, 50, Some(5));
        assert_eq!(
            muxbox.effective_z_index(),
            5,
            "Explicit z_index should be returned"
        );
    }

    #[test]
    fn test_z_index_sorting_for_rendering() {
        let box1 = create_test_muxbox_with_z_index("box1", 0, 0, 50, 50, Some(10));
        let box2 = create_test_muxbox_with_z_index("box2", 0, 0, 50, 50, Some(5));
        let box3 = create_test_muxbox_with_z_index("box3", 0, 0, 50, 50, None); // z_index = 0

        let boxes = vec![box1, box2, box3];

        // Sort by z_index for rendering (lower z_index first)
        let mut sorted_boxes: Vec<&MuxBox> = boxes.iter().collect();
        sorted_boxes.sort_by_key(|muxbox| muxbox.effective_z_index());

        assert_eq!(
            sorted_boxes[0].id, "box3",
            "Box with z_index 0 should be first"
        );
        assert_eq!(
            sorted_boxes[1].id, "box2",
            "Box with z_index 5 should be second"
        );
        assert_eq!(
            sorted_boxes[2].id, "box1",
            "Box with z_index 10 should be last (on top)"
        );
    }

    #[test]
    fn test_z_index_click_detection_priority() {
        // Create overlapping boxes with different z_index values
        let box1 = create_test_muxbox_with_z_index("low", 10, 10, 50, 50, Some(1));
        let box2 = create_test_muxbox_with_z_index("high", 10, 10, 50, 50, Some(10));
        let box3 = create_test_muxbox_with_z_index("default", 10, 10, 50, 50, None);

        let mut layout = Layout {
            id: "test_layout".to_string(),
            title: Some("Test Layout".to_string()),
            children: Some(vec![box1, box2, box3]),
            ..Default::default()
        };

        // Click in the overlapping area (coordinates 25, 25)
        let clicked_box = layout.find_muxbox_at_coordinates(25, 25);

        assert!(
            clicked_box.is_some(),
            "Should find a box at click coordinates"
        );
        assert_eq!(
            clicked_box.unwrap().id,
            "high",
            "Should click the box with highest z_index (10)"
        );
    }

    #[test]
    fn test_z_index_with_nested_boxes() {
        let mut child1 = create_test_muxbox_with_z_index("child1", 20, 20, 40, 40, Some(5));
        let mut child2 = create_test_muxbox_with_z_index("child2", 20, 20, 40, 40, Some(15));
        child1.parent_id = Some("parent".to_string());
        child2.parent_id = Some("parent".to_string());

        let mut parent = create_test_muxbox_with_z_index("parent", 10, 10, 50, 50, Some(1));
        parent.children = Some(vec![child1, child2]);

        let mut layout = Layout {
            id: "test_layout".to_string(),
            title: Some("Test Layout".to_string()),
            children: Some(vec![parent]),
            ..Default::default()
        };

        // Click in the child overlap area
        let clicked_box = layout.find_muxbox_at_coordinates(30, 30);

        assert!(clicked_box.is_some(), "Should find a child box");
        assert_eq!(
            clicked_box.unwrap().id,
            "child2",
            "Should click child with highest z_index (15)"
        );
    }

    #[test]
    fn test_z_index_serialization() {
        let muxbox_with_z = create_test_muxbox_with_z_index("test", 0, 0, 50, 50, Some(42));
        let muxbox_without_z = create_test_muxbox_with_z_index("test2", 0, 0, 50, 50, None);

        // Test serialization to JSON
        let json_with_z =
            serde_json::to_string(&muxbox_with_z).expect("Should serialize with z_index");
        let json_without_z =
            serde_json::to_string(&muxbox_without_z).expect("Should serialize without z_index");

        assert!(
            json_with_z.contains("\"z_index\":42"),
            "JSON should include z_index field"
        );
        // Note: serde includes None Option fields by default, so we check for null value instead
        assert!(
            json_without_z.contains("\"z_index\":null") || !json_without_z.contains("\"z_index\""),
            "JSON should have z_index as null or omitted when None"
        );

        // Test deserialization from JSON
        let deserialized_with_z: MuxBox =
            serde_json::from_str(&json_with_z).expect("Should deserialize with z_index");
        let deserialized_without_z: MuxBox =
            serde_json::from_str(&json_without_z).expect("Should deserialize without z_index");

        assert_eq!(
            deserialized_with_z.z_index,
            Some(42),
            "Deserialized z_index should match"
        );
        assert_eq!(
            deserialized_without_z.z_index, None,
            "Deserialized z_index should be None"
        );
    }

    #[test]
    fn test_z_index_hash_equality() {
        let box1 = create_test_muxbox_with_z_index("test", 0, 0, 50, 50, Some(5));
        let box2 = create_test_muxbox_with_z_index("test", 0, 0, 50, 50, Some(5));
        let box3 = create_test_muxbox_with_z_index("test", 0, 0, 50, 50, Some(10));

        // Test equality
        assert_eq!(box1, box2, "Boxes with same z_index should be equal");
        assert_ne!(
            box1, box3,
            "Boxes with different z_index should not be equal"
        );

        // Test hashing
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        box1.hash(&mut hasher1);
        box2.hash(&mut hasher2);
        box3.hash(&mut hasher3);

        assert_eq!(
            hasher1.finish(),
            hasher2.finish(),
            "Boxes with same z_index should have same hash"
        );
        assert_ne!(
            hasher1.finish(),
            hasher3.finish(),
            "Boxes with different z_index should have different hash"
        );
    }

    #[test]
    fn test_negative_z_index() {
        let box_negative = create_test_muxbox_with_z_index("negative", 0, 0, 50, 50, Some(-5));
        let box_positive = create_test_muxbox_with_z_index("positive", 0, 0, 50, 50, Some(3));
        let box_default = create_test_muxbox_with_z_index("default", 0, 0, 50, 50, None);

        let boxes = vec![box_negative, box_positive, box_default];

        // Sort by z_index for rendering
        let mut sorted_boxes: Vec<&MuxBox> = boxes.iter().collect();
        sorted_boxes.sort_by_key(|muxbox| muxbox.effective_z_index());

        assert_eq!(
            sorted_boxes[0].id, "negative",
            "Negative z_index should render first (behind)"
        );
        assert_eq!(
            sorted_boxes[1].id, "default",
            "Default z_index (0) should render second"
        );
        assert_eq!(
            sorted_boxes[2].id, "positive",
            "Positive z_index should render last (on top)"
        );
    }
}

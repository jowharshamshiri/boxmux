use crate::model::app::*;
use crate::model::common::*;
use crate::model::muxbox::*;
use crate::model::layout::*;

/// Test that periodic refresh sources get stable stream IDs
#[test]
fn test_periodic_sources_get_stable_stream_ids() {
    // Create app with a muxbox that has a script
    let mut app = App::new();
    
    let mut muxbox = MuxBox {
        id: "test_box".to_string(),
        script: Some(vec!["echo".to_string(), "hello".to_string()]),
        refresh_interval: Some(1000),
        ..Default::default()
    };
    
    let mut layout = Layout::new();
    layout.id = "test_layout".to_string();
    layout.children = Some(vec![muxbox]);
    app.layouts.push(layout);
    
    // Pre-register periodic sources
    app.register_periodic_sources();
    
    // Get stream ID for the first time
    let source_type = ExecutionSourceType::PeriodicScript("echo hello".to_string());
    let stream_id_1 = app.register_execution_source(source_type.clone(), "test_box".to_string());
    
    // Get stream ID for the second time - should be the same
    let stream_id_2 = app.register_execution_source(source_type.clone(), "test_box".to_string());
    
    // Verify stream IDs are identical (stable)
    assert_eq!(stream_id_1, stream_id_2, "Periodic refresh sources should have stable stream IDs");
    
    // Verify we only have one source registered for this box
    let sources_for_box = app.get_sources_for_box("test_box");
    assert_eq!(sources_for_box.len(), 1, "Should have exactly one periodic source per box");
    
    // Verify the source has the correct type
    if let Some(source) = sources_for_box.first() {
        match &source.source_type {
            ExecutionSourceType::PeriodicScript(script) => {
                assert_eq!(script, "echo hello");
            },
            _ => panic!("Expected PeriodicScript source type"),
        }
    }
}

/// Test that different boxes get different stream IDs for their periodic sources
#[test]
fn test_different_boxes_get_different_periodic_stream_ids() {
    let mut app = App::new();
    
    // Create two boxes with scripts
    let mut muxbox1 = MuxBox {
        id: "box1".to_string(),
        script: Some(vec!["echo".to_string(), "box1".to_string()]),
        refresh_interval: Some(1000),
        ..Default::default()
    };
    
    let mut muxbox2 = MuxBox {
        id: "box2".to_string(),
        script: Some(vec!["echo".to_string(), "box2".to_string()]),
        refresh_interval: Some(2000),
        ..Default::default()
    };
    
    let mut layout = Layout::new();
    layout.id = "test_layout".to_string();
    layout.children = Some(vec![muxbox1, muxbox2]);
    app.layouts.push(layout);
    
    // Pre-register periodic sources
    app.register_periodic_sources();
    
    // Get stream IDs for both boxes
    let source_type1 = ExecutionSourceType::PeriodicScript("echo box1".to_string());
    let source_type2 = ExecutionSourceType::PeriodicScript("echo box2".to_string());
    
    let stream_id_1 = app.register_execution_source(source_type1, "box1".to_string());
    let stream_id_2 = app.register_execution_source(source_type2, "box2".to_string());
    
    // Verify stream IDs are different
    assert_ne!(stream_id_1, stream_id_2, "Different boxes should have different stream IDs");
    
    // Verify we have two sources registered
    assert_eq!(app.execution_sources.len(), 2, "Should have two registered sources");
}

/// Test that boxes without scripts don't get periodic sources
#[test]
fn test_boxes_without_scripts_get_no_periodic_sources() {
    let mut app = App::new();
    
    // Create box without script
    let mut muxbox = MuxBox {
        id: "no_script_box".to_string(),
        script: None,
        ..Default::default()
    };
    
    let mut layout = Layout::new();
    layout.id = "test_layout".to_string();
    layout.children = Some(vec![muxbox]);
    app.layouts.push(layout);
    
    // Pre-register periodic sources
    app.register_periodic_sources();
    
    // Verify no sources were registered
    assert_eq!(app.execution_sources.len(), 0, "Boxes without scripts should not get periodic sources");
    
    // Verify get_sources_for_box returns empty
    let sources_for_box = app.get_sources_for_box("no_script_box");
    assert_eq!(sources_for_box.len(), 0, "Box without script should have no sources");
}
use crate::components::{StatusIndicator, StatusType};
use crate::model::muxbox::MuxBox;
use crate::model::common::ExecutionMode;

#[test]
fn test_status_type_indicators() {
    assert_eq!(StatusType::PtyNormal.get_indicator(), "⚡");
    assert_eq!(StatusType::PtyDead.get_indicator(), "💀");
    assert_eq!(StatusType::PtyError.get_indicator(), "⚠️");
    assert_eq!(StatusType::ScriptRunning.get_indicator(), "▶️");
    assert_eq!(StatusType::ScriptCompleted.get_indicator(), "✅");
    assert_eq!(StatusType::ScriptFailed.get_indicator(), "❌");
    assert_eq!(StatusType::Custom("🚀".to_string()).get_indicator(), "🚀");
    assert_eq!(StatusType::None.get_indicator(), "");
}

#[test] 
fn test_status_type_visibility() {
    assert!(StatusType::PtyNormal.is_visible());
    assert!(StatusType::PtyDead.is_visible());
    assert!(StatusType::PtyError.is_visible());
    assert!(StatusType::ScriptRunning.is_visible());
    assert!(StatusType::ScriptCompleted.is_visible());
    assert!(StatusType::ScriptFailed.is_visible());
    assert!(StatusType::Custom("⭐".to_string()).is_visible());
    assert!(!StatusType::None.is_visible());
}

#[test]
fn test_custom_status_indicator_creation() {
    let indicator = StatusIndicator::new_custom("🎯".to_string(), Some("Target Acquired".to_string()));
    
    assert_eq!(indicator.get_indicator(), "🎯");
    assert_eq!(indicator.custom_text, Some("Target Acquired".to_string()));
    assert!(indicator.is_visible());
    assert!(matches!(indicator.indicator_type, StatusType::Custom(_)));
}

#[test]
fn test_status_indicator_with_process_info() {
    let indicator = StatusIndicator::new(StatusType::PtyNormal)
        .with_process_info("bash:1234".to_string());
    
    assert_eq!(indicator.get_indicator(), "⚡");
    assert_eq!(indicator.process_info, Some("bash:1234".to_string()));
    assert!(indicator.is_visible());
}

#[test]
fn test_status_indicator_chaining() {
    let indicator = StatusIndicator::new(StatusType::PtyError)
        .with_process_info("failed_process:5678".to_string())
        .with_custom_text("Connection timeout".to_string());
    
    assert_eq!(indicator.get_indicator(), "⚠️");
    assert_eq!(indicator.process_info, Some("failed_process:5678".to_string()));
    assert_eq!(indicator.custom_text, Some("Connection timeout".to_string()));
}

#[test] 
fn test_render_for_title_with_all_components() {
    let indicator = StatusIndicator::new(StatusType::PtyNormal)
        .with_process_info("bash:1234".to_string());
    
    let result = indicator.render_for_title(Some("My Terminal Session"));
    assert_eq!(result, Some("⚡ [bash:1234] My Terminal Session".to_string()));
}

#[test]
fn test_render_for_title_no_original_title() {
    let indicator = StatusIndicator::new(StatusType::PtyDead)
        .with_process_info("crashed_app:9999".to_string());
    
    let result = indicator.render_for_title(None);
    assert_eq!(result, Some("💀 [crashed_app:9999] PTY".to_string()));
}

#[test]
fn test_render_for_title_with_custom_status() {
    let indicator = StatusIndicator::new_custom("🔥".to_string(), Some("Hot Process".to_string()));
    
    let result = indicator.render_for_title(Some("Server Monitor"));
    assert_eq!(result, Some("🔥 Hot Process Server Monitor".to_string()));
}

#[test]
fn test_render_for_title_none_status() {
    let indicator = StatusIndicator::new(StatusType::None);
    
    let result = indicator.render_for_title(Some("Regular Box"));
    assert_eq!(result, Some("Regular Box".to_string()));
}

#[test]
fn test_render_for_title_none_status_no_original() {
    let indicator = StatusIndicator::new(StatusType::None);
    
    let result = indicator.render_for_title(None);
    assert_eq!(result, None);
}

#[test]
fn test_get_full_status_complete() {
    let indicator = StatusIndicator::new(StatusType::PtyError)
        .with_process_info("nginx:80".to_string())
        .with_custom_text("Port conflict".to_string());
    
    let result = indicator.get_full_status();
    assert_eq!(result, "⚠️ [nginx:80] Port conflict");
}

#[test]
fn test_get_full_status_minimal() {
    let indicator = StatusIndicator::new(StatusType::ScriptRunning);
    
    let result = indicator.get_full_status();
    assert_eq!(result, "▶️");
}

#[test]
fn test_get_full_status_none() {
    let indicator = StatusIndicator::new(StatusType::None);
    
    let result = indicator.get_full_status();
    assert_eq!(result, "");
}

#[test]
fn test_from_muxbox_pty_normal() {
    let mut muxbox = MuxBox::default();
    muxbox.id = "test_pty_1".to_string();
    muxbox.execution_mode = ExecutionMode::Pty;
    
    let indicator = StatusIndicator::from_muxbox(&muxbox, None);
    
    assert!(matches!(indicator.indicator_type, StatusType::PtyNormal));
    assert_eq!(indicator.get_indicator(), "⚡");
    assert!(indicator.is_visible());
}

#[test]
fn test_from_muxbox_non_pty() {
    let mut muxbox = MuxBox::default();
    muxbox.id = "test_thread_1".to_string();
    muxbox.execution_mode = ExecutionMode::Thread;
    
    let indicator = StatusIndicator::from_muxbox(&muxbox, None);
    
    assert!(matches!(indicator.indicator_type, StatusType::None));
    assert_eq!(indicator.get_indicator(), "");
    assert!(!indicator.is_visible());
}

#[test]
fn test_from_muxbox_immediate_mode() {
    let mut muxbox = MuxBox::default();
    muxbox.id = "test_immediate_1".to_string();
    muxbox.execution_mode = ExecutionMode::Immediate;
    
    let indicator = StatusIndicator::from_muxbox(&muxbox, None);
    
    assert!(matches!(indicator.indicator_type, StatusType::None));
    assert!(!indicator.is_visible());
}

#[test]
fn test_script_status_types() {
    let running = StatusIndicator::new(StatusType::ScriptRunning);
    let completed = StatusIndicator::new(StatusType::ScriptCompleted);
    let failed = StatusIndicator::new(StatusType::ScriptFailed);
    
    assert_eq!(running.get_indicator(), "▶️");
    assert_eq!(completed.get_indicator(), "✅");
    assert_eq!(failed.get_indicator(), "❌");
    
    assert!(running.is_visible());
    assert!(completed.is_visible());
    assert!(failed.is_visible());
}

#[test]
fn test_status_indicator_equality() {
    let indicator1 = StatusIndicator::new(StatusType::PtyNormal);
    let indicator2 = StatusIndicator::new(StatusType::PtyNormal);
    
    // Test that the status types are equal
    assert_eq!(indicator1.indicator_type, indicator2.indicator_type);
    assert_eq!(indicator1.get_indicator(), indicator2.get_indicator());
}

#[test]
fn test_custom_status_with_emoji_combinations() {
    let rocket = StatusIndicator::new_custom("🚀".to_string(), None);
    let fire = StatusIndicator::new_custom("🔥💻".to_string(), Some("Hot CPU".to_string()));
    let complex = StatusIndicator::new_custom("⚡🎯🚀".to_string(), Some("Multi-stage".to_string()));
    
    assert_eq!(rocket.get_indicator(), "🚀");
    assert_eq!(fire.get_indicator(), "🔥💻");
    assert_eq!(complex.get_indicator(), "⚡🎯🚀");
    
    assert_eq!(fire.get_full_status(), "🔥💻 Hot CPU");
    assert_eq!(complex.get_full_status(), "⚡🎯🚀 Multi-stage");
}
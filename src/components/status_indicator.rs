use crate::model::muxbox::MuxBox;
use crate::pty_manager::PtyManager;

/// Status indicator component for rendering process and execution status indicators
pub struct StatusIndicator {
    pub indicator_type: StatusType,
    pub process_info: Option<String>,
    pub custom_text: Option<String>,
}

/// Types of status indicators available
#[derive(Debug, Clone, PartialEq)]
pub enum StatusType {
    /// Normal PTY process (⚡)
    PtyNormal,
    /// Dead PTY process (💀)  
    PtyDead,
    /// PTY process in error state (⚠️)
    PtyError,
    /// Script execution in progress
    ScriptRunning,
    /// Script execution completed
    ScriptCompleted,
    /// Script execution failed
    ScriptFailed,
    /// Custom status with user-defined indicator
    Custom(String),
    /// No status indicator
    None,
}

impl StatusType {
    /// Get the character/emoji for this status type
    pub fn get_indicator(&self) -> &str {
        match self {
            StatusType::PtyNormal => "⚡",
            StatusType::PtyDead => "💀",
            StatusType::PtyError => "⚠️",
            StatusType::ScriptRunning => "▶️",
            StatusType::ScriptCompleted => "✅",
            StatusType::ScriptFailed => "❌",
            StatusType::Custom(indicator) => indicator,
            StatusType::None => "",
        }
    }

    /// Check if this status type should be displayed
    pub fn is_visible(&self) -> bool {
        !matches!(self, StatusType::None)
    }
}

impl StatusIndicator {
    /// Create status indicator from MuxBox and PTY manager state
    pub fn from_muxbox(muxbox: &MuxBox, pty_manager: Option<&PtyManager>) -> Self {
        if muxbox.execution_mode.is_pty() {
            let indicator_type = if let Some(pty_manager) = pty_manager {
                if pty_manager.is_pty_dead(&muxbox.id) {
                    StatusType::PtyDead
                } else if pty_manager.is_pty_in_error_state(&muxbox.id) {
                    StatusType::PtyError
                } else {
                    StatusType::PtyNormal
                }
            } else {
                StatusType::PtyNormal
            };

            let process_info = pty_manager.and_then(|pm| pm.get_process_status_summary(&muxbox.id));

            Self {
                indicator_type,
                process_info,
                custom_text: None,
            }
        } else {
            Self {
                indicator_type: StatusType::None,
                process_info: None,
                custom_text: None,
            }
        }
    }

    /// Create a custom status indicator
    pub fn new_custom(indicator: String, text: Option<String>) -> Self {
        Self {
            indicator_type: StatusType::Custom(indicator),
            process_info: None,
            custom_text: text,
        }
    }

    /// Create a specific status indicator type
    pub fn new(status_type: StatusType) -> Self {
        Self {
            indicator_type: status_type,
            process_info: None,
            custom_text: None,
        }
    }

    /// Set process information text
    pub fn with_process_info(mut self, info: String) -> Self {
        self.process_info = Some(info);
        self
    }

    /// Set custom text
    pub fn with_custom_text(mut self, text: String) -> Self {
        self.custom_text = Some(text);
        self
    }

    /// Render the status indicator as a string for title integration
    pub fn render_for_title(&self, original_title: Option<&str>) -> Option<String> {
        if !self.indicator_type.is_visible() {
            return original_title.map(|s| s.to_string());
        }

        let mut title_parts = vec![self.indicator_type.get_indicator().to_string()];

        // Add process info if available
        if let Some(ref info) = self.process_info {
            title_parts.push(format!("[{}]", info));
        }

        // Add custom text if available
        if let Some(ref text) = self.custom_text {
            title_parts.push(text.clone());
        }

        // Add original title if it exists
        if let Some(title) = original_title {
            title_parts.push(title.to_string());
        } else if matches!(
            self.indicator_type,
            StatusType::PtyNormal | StatusType::PtyDead | StatusType::PtyError
        ) {
            title_parts.push("PTY".to_string());
        }

        Some(title_parts.join(" "))
    }

    /// Get just the indicator character/emoji
    pub fn get_indicator(&self) -> &str {
        self.indicator_type.get_indicator()
    }

    /// Check if the status indicator should be displayed
    pub fn is_visible(&self) -> bool {
        self.indicator_type.is_visible()
    }

    /// Get the full status text (indicator + process info + custom text)
    pub fn get_full_status(&self) -> String {
        if !self.is_visible() {
            return String::new();
        }

        let mut parts = vec![self.get_indicator().to_string()];

        if let Some(ref info) = self.process_info {
            parts.push(format!("[{}]", info));
        }

        if let Some(ref text) = self.custom_text {
            parts.push(text.clone());
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(StatusType::Custom("⭐".to_string()).is_visible());
        assert!(!StatusType::None.is_visible());
    }

    #[test]
    fn test_custom_status_indicator() {
        let indicator =
            StatusIndicator::new_custom("🎯".to_string(), Some("Custom Status".to_string()));
        assert_eq!(indicator.get_indicator(), "🎯");
        assert_eq!(indicator.custom_text, Some("Custom Status".to_string()));
        assert!(indicator.is_visible());
    }

    #[test]
    fn test_status_indicator_with_process_info() {
        let indicator =
            StatusIndicator::new(StatusType::PtyNormal).with_process_info("bash:1234".to_string());

        assert_eq!(indicator.get_indicator(), "⚡");
        assert_eq!(indicator.process_info, Some("bash:1234".to_string()));
    }

    #[test]
    fn test_render_for_title() {
        let indicator =
            StatusIndicator::new(StatusType::PtyNormal).with_process_info("bash:1234".to_string());

        let result = indicator.render_for_title(Some("My Terminal"));
        assert_eq!(result, Some("⚡ [bash:1234] My Terminal".to_string()));
    }

    #[test]
    fn test_render_for_title_no_original() {
        let indicator =
            StatusIndicator::new(StatusType::PtyNormal).with_process_info("bash:1234".to_string());

        let result = indicator.render_for_title(None);
        assert_eq!(result, Some("⚡ [bash:1234] PTY".to_string()));
    }

    #[test]
    fn test_render_for_title_none_status() {
        let indicator = StatusIndicator::new(StatusType::None);
        let result = indicator.render_for_title(Some("Regular Title"));
        assert_eq!(result, Some("Regular Title".to_string()));
    }

    #[test]
    fn test_get_full_status() {
        let indicator = StatusIndicator::new(StatusType::PtyError)
            .with_process_info("failed_cmd".to_string())
            .with_custom_text("Connection Lost".to_string());

        let result = indicator.get_full_status();
        assert_eq!(result, "⚠️ [failed_cmd] Connection Lost");
    }

    #[test]
    fn test_from_muxbox_pty() {
        let mut muxbox = MuxBox::default();
        muxbox.id = "test_pty".to_string();
        muxbox.execution_mode = ExecutionMode::Pty;

        let indicator = StatusIndicator::from_muxbox(&muxbox, None);
        assert!(matches!(indicator.indicator_type, StatusType::PtyNormal));
        assert_eq!(indicator.get_indicator(), "⚡");
    }

    #[test]
    fn test_from_muxbox_non_pty() {
        let mut muxbox = MuxBox::default();
        muxbox.execution_mode = ExecutionMode::Thread;

        let indicator = StatusIndicator::from_muxbox(&muxbox, None);
        assert!(matches!(indicator.indicator_type, StatusType::None));
        assert!(!indicator.is_visible());
    }
}

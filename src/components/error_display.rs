use crate::model::common::{Bounds, ScreenBuffer};
use crate::draw_utils::print_with_color_and_background_at;

/// Error severity levels for display styling
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Critical error that stops execution
    Error,
    /// Warning that should be addressed
    Warning,
    /// Information for debugging
    Info,
    /// Hint for user guidance
    Hint,
}

/// Error display style configuration
#[derive(Debug, Clone)]
pub struct ErrorDisplayConfig {
    /// Color for error severity indicators
    pub error_color: String,
    /// Color for warning severity indicators
    pub warning_color: String,
    /// Color for info severity indicators
    pub info_color: String,
    /// Color for hint severity indicators
    pub hint_color: String,
    /// Color for line numbers
    pub line_number_color: String,
    /// Color for caret indicators (^^^)
    pub caret_color: String,
    /// Color for file path information
    pub file_path_color: String,
    /// Color for pipe characters and borders
    pub border_color: String,
    /// Background color for error display
    pub background_color: String,
    /// Whether to show context lines around error
    pub show_context: bool,
    /// Number of context lines to show before and after error
    pub context_lines: usize,
    /// Character to use for caret indicators
    pub caret_char: char,
    /// Character to use for line continuation
    pub continuation_char: char,
}

impl Default for ErrorDisplayConfig {
    fn default() -> Self {
        Self {
            error_color: "bright_red".to_string(),
            warning_color: "bright_yellow".to_string(),
            info_color: "bright_blue".to_string(),
            hint_color: "bright_green".to_string(),
            line_number_color: "bright_blue".to_string(),
            caret_color: "bright_red".to_string(),
            file_path_color: "bright_blue".to_string(),
            border_color: "bright_blue".to_string(),
            background_color: "black".to_string(),
            show_context: true,
            context_lines: 2,
            caret_char: '^',
            continuation_char: '.',
        }
    }
}

impl ErrorDisplayConfig {
    /// Create config optimized for terminal display
    pub fn terminal() -> Self {
        Self {
            show_context: false,
            context_lines: 1,
            ..Default::default()
        }
    }

    /// Create config optimized for detailed debugging
    pub fn detailed() -> Self {
        Self {
            show_context: true,
            context_lines: 3,
            ..Default::default()
        }
    }

    /// Create config with custom colors
    pub fn with_colors(error_color: String, warning_color: String, info_color: String) -> Self {
        Self {
            error_color,
            warning_color,
            info_color,
            ..Default::default()
        }
    }
}

/// Error information for display
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// Error message
    pub message: String,
    /// File path where error occurred
    pub file_path: String,
    /// Line number (1-based)
    pub line_number: usize,
    /// Column number (1-based)
    pub column_number: usize,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Optional help text
    pub help: Option<String>,
    /// Optional note text
    pub note: Option<String>,
}

/// Rust-style error message rendering component with line indicators and caret positioning
pub struct ErrorDisplay {
    /// Unique identifier for this error display instance
    pub id: String,
    /// Configuration for error display styling
    pub config: ErrorDisplayConfig,
}

impl ErrorDisplay {
    /// Create a new error display with specified ID and config
    pub fn new(id: String, config: ErrorDisplayConfig) -> Self {
        Self { id, config }
    }

    /// Create error display with default configuration
    pub fn with_defaults(id: String) -> Self {
        Self::new(id, ErrorDisplayConfig::default())
    }

    /// Create error display optimized for terminal
    pub fn with_terminal_config(id: String) -> Self {
        Self::new(id, ErrorDisplayConfig::terminal())
    }

    /// Create error display optimized for detailed debugging
    pub fn with_detailed_config(id: String) -> Self {
        Self::new(id, ErrorDisplayConfig::detailed())
    }

    /// Generate Rust-style error display text
    pub fn format_error(&self, error: &ErrorInfo, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();

        // Ensure we have valid line numbers (using 1-based indexing)
        if error.line_number == 0 || error.line_number > lines.len() {
            return format!("{}: {}", self.severity_text(&error.severity), error.message);
        }

        let error_line = lines[error.line_number - 1];
        let line_num_width = self.calculate_line_number_width(&error, &lines);

        let mut result = String::new();
        
        // Error header with severity
        result.push_str(&format!("{}: {}\n", self.severity_text(&error.severity), error.message));
        
        // File location
        result.push_str(&format!(" --> {}:{}:{}\n", error.file_path, error.line_number, error.column_number));
        
        // Context lines before error (if enabled)
        if self.config.show_context {
            let start_line = error.line_number.saturating_sub(self.config.context_lines + 1);
            for line_idx in start_line..(error.line_number - 1) {
                if line_idx < lines.len() {
                    result.push_str(&format!(
                        "{:width$} | {}\n",
                        line_idx + 1,
                        lines[line_idx],
                        width = line_num_width
                    ));
                }
            }
        }

        // Separator
        result.push_str(&format!("{}|\n", " ".repeat(line_num_width + 1)));
        
        // Error line
        result.push_str(&format!(
            "{:width$} | {}\n",
            error.line_number,
            error_line,
            width = line_num_width
        ));

        // Column indicator with caret
        if error.column_number > 0 && error.column_number <= error_line.len() + 1 {
            let spaces_before_pipe = " ".repeat(line_num_width);
            let spaces_before_caret = " ".repeat(error.column_number.saturating_sub(1));
            result.push_str(&format!(
                "{} | {}{}\n",
                spaces_before_pipe, spaces_before_caret, self.config.caret_char
            ));
        }

        // Context lines after error (if enabled)
        if self.config.show_context {
            let end_line = (error.line_number + self.config.context_lines).min(lines.len());
            for line_idx in error.line_number..end_line {
                result.push_str(&format!(
                    "{:width$} | {}\n",
                    line_idx + 1,
                    lines[line_idx],
                    width = line_num_width
                ));
            }
        }

        // Help text
        if let Some(help) = &error.help {
            result.push_str(&format!("\nhelp: {}\n", help));
        }

        // Note text
        if let Some(note) = &error.note {
            result.push_str(&format!("\nnote: {}\n", note));
        }

        result
    }

    /// Render error display within bounds
    pub fn render(
        &self,
        error: &ErrorInfo,
        content: &str,
        bounds: &Bounds,
        buffer: &mut ScreenBuffer,
    ) {
        let formatted_error = self.format_error(error, content);
        let error_lines: Vec<&str> = formatted_error.lines().collect();

        let viewable_height = bounds.height().saturating_sub(2);
        let start_y = bounds.top() + 1;

        // Render error lines within bounds
        for (line_idx, &line) in error_lines.iter().take(viewable_height).enumerate() {
            let y_position = start_y + line_idx;
            if y_position > bounds.bottom() - 1 {
                break;
            }

            // Determine color based on line content
            let text_color = self.get_line_color(line, &error.severity);

            print_with_color_and_background_at(
                y_position,
                bounds.left() + 1,
                &text_color,
                &self.config.background_color,
                line,
                buffer,
            );
        }
    }

    /// Render multiple errors in sequence
    pub fn render_multiple(
        &self,
        errors: &[ErrorInfo],
        content: &str,
        bounds: &Bounds,
        buffer: &mut ScreenBuffer,
    ) {
        let mut combined_output = String::new();
        
        for (idx, error) in errors.iter().enumerate() {
            combined_output.push_str(&self.format_error(error, content));
            
            // Add separator between errors
            if idx < errors.len() - 1 {
                combined_output.push_str(&format!("\n{}\n\n", "-".repeat(50)));
            }
        }

        let error_lines: Vec<&str> = combined_output.lines().collect();
        let viewable_height = bounds.height().saturating_sub(2);
        let start_y = bounds.top() + 1;

        // Render combined error lines within bounds
        for (line_idx, &line) in error_lines.iter().take(viewable_height).enumerate() {
            let y_position = start_y + line_idx;
            if y_position > bounds.bottom() - 1 {
                break;
            }

            // Determine color based on line content
            let text_color = self.get_line_color(line, &ErrorSeverity::Error);

            print_with_color_and_background_at(
                y_position,
                bounds.left() + 1,
                &text_color,
                &self.config.background_color,
                line,
                buffer,
            );
        }
    }

    /// Get severity text for display
    fn severity_text(&self, severity: &ErrorSeverity) -> &str {
        match severity {
            ErrorSeverity::Error => "error",
            ErrorSeverity::Warning => "warning",
            ErrorSeverity::Info => "info",
            ErrorSeverity::Hint => "hint",
        }
    }

    /// Calculate appropriate width for line numbers
    fn calculate_line_number_width(&self, error: &ErrorInfo, lines: &[&str]) -> usize {
        let max_line = if self.config.show_context {
            (error.line_number + self.config.context_lines).min(lines.len())
        } else {
            error.line_number
        };
        format!("{}", max_line).len().max(3)
    }

    /// Determine color for a specific line based on content
    fn get_line_color(&self, line: &str, severity: &ErrorSeverity) -> String {
        if line.starts_with("error:") {
            self.config.error_color.clone()
        } else if line.starts_with("warning:") {
            self.config.warning_color.clone()
        } else if line.starts_with("info:") {
            self.config.info_color.clone()
        } else if line.starts_with("hint:") {
            self.config.hint_color.clone()
        } else if line.starts_with("help:") {
            self.config.hint_color.clone()
        } else if line.starts_with("note:") {
            self.config.info_color.clone()
        } else if line.contains(" --> ") {
            self.config.file_path_color.clone()
        } else if line.contains(self.config.caret_char) {
            self.config.caret_color.clone()
        } else if line.contains(" | ") {
            if line.trim_start().starts_with("|") {
                self.config.border_color.clone()
            } else {
                // Line with line number
                self.config.line_number_color.clone()
            }
        } else {
            // Default text color based on severity
            match severity {
                ErrorSeverity::Error => self.config.error_color.clone(),
                ErrorSeverity::Warning => self.config.warning_color.clone(),
                ErrorSeverity::Info => self.config.info_color.clone(),
                ErrorSeverity::Hint => self.config.hint_color.clone(),
            }
        }
    }

    /// Create ErrorInfo from serde_yaml::Error
    pub fn from_yaml_error(
        yaml_error: &serde_yaml::Error,
        file_path: &str,
        message: String,
    ) -> Option<ErrorInfo> {
        if let Some(location) = yaml_error.location() {
            Some(ErrorInfo {
                message,
                file_path: file_path.to_string(),
                line_number: location.line(),
                column_number: location.column(),
                severity: ErrorSeverity::Error,
                help: Some("Check YAML syntax and structure".to_string()),
                note: None,
            })
        } else {
            None
        }
    }

    /// Create ErrorInfo for configuration validation
    pub fn validation_error(
        file_path: &str,
        line_number: usize,
        column_number: usize,
        message: String,
    ) -> ErrorInfo {
        ErrorInfo {
            message,
            file_path: file_path.to_string(),
            line_number,
            column_number,
            severity: ErrorSeverity::Error,
            help: Some("Verify configuration values and structure".to_string()),
            note: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::{Bounds, ScreenBuffer};

    fn create_test_buffer() -> ScreenBuffer {
        ScreenBuffer::new()
    }

    fn create_test_error() -> ErrorInfo {
        ErrorInfo {
            message: "expected `:`, found `!`".to_string(),
            file_path: "config.yaml".to_string(),
            line_number: 5,
            column_number: 12,
            severity: ErrorSeverity::Error,
            help: Some("Check for missing colons in YAML".to_string()),
            note: Some("YAML syntax error".to_string()),
        }
    }

    fn create_test_content() -> String {
        r#"app:
  title: "Test App"
  layouts:
    - name: main
      boxes: invalid!syntax
        - id: box1
          content: "Hello World""#.to_string()
    }

    #[test]
    fn test_error_display_creation() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        assert_eq!(display.id, "test");
        assert_eq!(display.config.caret_char, '^');
    }

    #[test]
    fn test_error_display_terminal_config() {
        let display = ErrorDisplay::with_terminal_config("test".to_string());
        assert!(!display.config.show_context);
        assert_eq!(display.config.context_lines, 1);
    }

    #[test]
    fn test_error_display_detailed_config() {
        let display = ErrorDisplay::with_detailed_config("test".to_string());
        assert!(display.config.show_context);
        assert_eq!(display.config.context_lines, 3);
    }

    #[test]
    fn test_error_severity_display() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        
        assert_eq!(display.severity_text(&ErrorSeverity::Error), "error");
        assert_eq!(display.severity_text(&ErrorSeverity::Warning), "warning");
        assert_eq!(display.severity_text(&ErrorSeverity::Info), "info");
        assert_eq!(display.severity_text(&ErrorSeverity::Hint), "hint");
    }

    #[test]
    fn test_format_error_basic() {
        let display = ErrorDisplay::with_terminal_config("test".to_string());
        let error = create_test_error();
        let content = create_test_content();

        let formatted = display.format_error(&error, &content);
        
        assert!(formatted.contains("error: expected `:`, found `!`"));
        assert!(formatted.contains(" --> config.yaml:5:12"));
        assert!(formatted.contains("boxes: invalid!syntax"));
        assert!(formatted.contains("help: Check for missing colons"));
        assert!(formatted.contains("note: YAML syntax error"));
    }

    #[test]
    fn test_format_error_with_context() {
        let display = ErrorDisplay::with_detailed_config("test".to_string());
        let error = create_test_error();
        let content = create_test_content();

        let formatted = display.format_error(&error, &content);
        
        assert!(formatted.contains("layouts:"));
        assert!(formatted.contains("- name: main"));
        assert!(formatted.contains("boxes: invalid!syntax"));
        assert!(formatted.contains("- id: box1"));
    }

    #[test]
    fn test_format_error_invalid_line() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        let mut error = create_test_error();
        error.line_number = 999; // Invalid line number
        let content = create_test_content();

        let formatted = display.format_error(&error, &content);
        
        assert_eq!(formatted, "error: expected `:`, found `!`");
    }

    #[test]
    fn test_line_number_width_calculation() {
        let display = ErrorDisplay::with_detailed_config("test".to_string());
        let error = create_test_error();
        let content = create_test_content();
        let lines: Vec<&str> = content.lines().collect();

        let width = display.calculate_line_number_width(&error, &lines);
        assert!(width >= 3); // Should be at least 3 characters wide
    }

    #[test]
    fn test_get_line_color() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        
        assert_eq!(display.get_line_color("error: something", &ErrorSeverity::Error), "bright_red");
        assert_eq!(display.get_line_color("warning: something", &ErrorSeverity::Warning), "bright_yellow");
        assert_eq!(display.get_line_color(" --> file.yaml:5:12", &ErrorSeverity::Error), "bright_blue");
        assert_eq!(display.get_line_color("  5 | some line", &ErrorSeverity::Error), "bright_blue");
        assert_eq!(display.get_line_color("    | ^", &ErrorSeverity::Error), "bright_red");
        assert_eq!(display.get_line_color("help: something", &ErrorSeverity::Error), "bright_green");
    }

    #[test]
    fn test_render_error() {
        let display = ErrorDisplay::with_terminal_config("test".to_string());
        let error = create_test_error();
        let content = create_test_content();
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();

        display.render(&error, &content, &bounds, &mut buffer);
        
        // Test just verifies no panic - detailed buffer inspection would require more setup
    }

    #[test]
    fn test_render_multiple_errors() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        let error1 = create_test_error();
        let mut error2 = create_test_error();
        error2.line_number = 7;
        error2.message = "unexpected token".to_string();
        
        let errors = vec![error1, error2];
        let content = create_test_content();
        let bounds = Bounds::new(10, 5, 80, 30);
        let mut buffer = create_test_buffer();

        display.render_multiple(&errors, &content, &bounds, &mut buffer);
        
        // Test just verifies no panic with multiple errors
    }

    #[test]
    fn test_from_yaml_error() {
        let yaml_content = "invalid: yaml: content:";
        let yaml_error: serde_yaml::Error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content)
            .unwrap_err();
        
        let error_info = ErrorDisplay::from_yaml_error(&yaml_error, "test.yaml", "YAML parsing failed".to_string());
        
        if let Some(info) = error_info {
            assert_eq!(info.file_path, "test.yaml");
            assert_eq!(info.message, "YAML parsing failed");
            assert!(matches!(info.severity, ErrorSeverity::Error));
        }
    }

    #[test]
    fn test_validation_error() {
        let error = ErrorDisplay::validation_error(
            "config.yaml",
            10,
            5,
            "Invalid configuration value".to_string(),
        );
        
        assert_eq!(error.file_path, "config.yaml");
        assert_eq!(error.line_number, 10);
        assert_eq!(error.column_number, 5);
        assert_eq!(error.message, "Invalid configuration value");
        assert!(matches!(error.severity, ErrorSeverity::Error));
    }

    #[test]
    fn test_custom_config_colors() {
        let config = ErrorDisplayConfig::with_colors(
            "red".to_string(),
            "yellow".to_string(),
            "blue".to_string(),
        );
        
        let display = ErrorDisplay::new("test".to_string(), config);
        assert_eq!(display.config.error_color, "red");
        assert_eq!(display.config.warning_color, "yellow");
        assert_eq!(display.config.info_color, "blue");
    }
}
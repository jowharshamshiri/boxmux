use crate::draw_utils::print_with_color_and_background_at;
use crate::model::common::{Bounds, ScreenBuffer};
use crate::components::ComponentDimensions;

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

/// Syntax highlighting token types for error context
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxToken {
    /// Language keywords (fn, let, impl, etc.)
    Keyword,
    /// String literals
    String,
    /// Numeric literals
    Number,
    /// Comments
    Comment,
    /// Function and method names
    Function,
    /// Type names
    Type,
    /// Variable names
    Variable,
    /// Operators (+, -, =, etc.)
    Operator,
    /// Punctuation and delimiters
    Punctuation,
    /// Regular text
    Text,
}

/// Syntax highlighting configuration
#[derive(Debug, Clone)]
pub struct SyntaxHighlightConfig {
    /// Enable syntax highlighting
    pub enabled: bool,
    /// Color for keywords
    pub keyword_color: String,
    /// Color for string literals
    pub string_color: String,
    /// Color for numeric literals
    pub number_color: String,
    /// Color for comments
    pub comment_color: String,
    /// Color for function names
    pub function_color: String,
    /// Color for type names
    pub type_color: String,
    /// Color for variable names
    pub variable_color: String,
    /// Color for operators
    pub operator_color: String,
    /// Color for punctuation
    pub punctuation_color: String,
    /// Language for syntax highlighting (rust, yaml, json, etc.)
    pub language: String,
}

impl Default for SyntaxHighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            keyword_color: "magenta".to_string(),
            string_color: "green".to_string(),
            number_color: "yellow".to_string(),
            comment_color: "bright_black".to_string(),
            function_color: "blue".to_string(),
            type_color: "cyan".to_string(),
            variable_color: "white".to_string(),
            operator_color: "red".to_string(),
            punctuation_color: "bright_black".to_string(),
            language: "rust".to_string(),
        }
    }
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
    /// Syntax highlighting configuration
    pub syntax_highlighting: SyntaxHighlightConfig,
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
            syntax_highlighting: SyntaxHighlightConfig::default(),
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
    /// Enhanced caret positioning configuration
    pub caret_positioning: Option<CaretPositioning>,
}

/// Multi-line error span for precise highlighting
#[derive(Debug, Clone)]
pub struct ErrorSpan {
    /// Start line number (1-indexed)
    pub start_line: usize,
    /// Start column number (1-indexed)
    pub start_column: usize,
    /// End line number (1-indexed, can be same as start_line for single-line errors)
    pub end_line: usize,
    /// End column number (1-indexed)
    pub end_column: usize,
    /// Message for this span
    pub message: String,
}

/// Enhanced caret positioning configuration
#[derive(Debug, Clone)]
pub struct CaretPositioning {
    /// Primary error span (required)
    pub primary_span: ErrorSpan,
    /// Secondary spans for additional context (optional)
    pub secondary_spans: Vec<ErrorSpan>,
    /// Whether to show multi-line carets
    pub show_multi_line_carets: bool,
    /// Whether to show line continuation indicators
    pub show_line_continuations: bool,
    /// Character for multi-line caret start
    pub multi_line_start_char: char,
    /// Character for multi-line caret middle
    pub multi_line_middle_char: char,
    /// Character for multi-line caret end
    pub multi_line_end_char: char,
}

impl Default for CaretPositioning {
    fn default() -> Self {
        Self {
            primary_span: ErrorSpan {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 1,
                message: String::new(),
            },
            secondary_spans: Vec::new(),
            show_multi_line_carets: true,
            show_line_continuations: true,
            multi_line_start_char: '┌',
            multi_line_middle_char: '│',
            multi_line_end_char: '└',
        }
    }
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

    /// Create error display with syntax highlighting for specific language
    pub fn with_syntax_highlighting(id: String, language: String) -> Self {
        let mut config = ErrorDisplayConfig::default();
        config.syntax_highlighting.language = language;
        Self::new(id, config)
    }

    /// Create error display with custom syntax highlighting config
    pub fn with_custom_syntax_config(id: String, syntax_config: SyntaxHighlightConfig) -> Self {
        let config = ErrorDisplayConfig {
            syntax_highlighting: syntax_config,
            ..Default::default()
        };
        Self::new(id, config)
    }

    /// Generate Rust-style error display text
    pub fn format_error(&self, error: &ErrorInfo, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();

        // Ensure we have valid line numbers (using 1-based indexing)
        if error.line_number == 0 || error.line_number > lines.len() {
            return format!("{}: {}", self.severity_text(&error.severity), error.message);
        }

        let error_line = lines[error.line_number - 1];
        let line_num_width = self.calculate_line_number_width(error, &lines);

        let mut result = String::new();

        // Error header with severity
        result.push_str(&format!(
            "{}: {}\n",
            self.severity_text(&error.severity),
            error.message
        ));

        // File location
        result.push_str(&format!(
            " --> {}:{}:{}\n",
            error.file_path, error.line_number, error.column_number
        ));

        // Context lines before error (if enabled)
        if self.config.show_context {
            let start_line = error
                .line_number
                .saturating_sub(self.config.context_lines + 1);
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

        let viewable_height = ComponentDimensions::new(*bounds).content_bounds().height();
        let start_y = ComponentDimensions::new(*bounds).content_bounds().top();

        // Render error lines within bounds
        for (line_idx, &line) in error_lines.iter().take(viewable_height).enumerate() {
            let y_position = start_y + line_idx;
            if y_position > ComponentDimensions::new(*bounds).content_bounds().bottom() {
                break;
            }

            // Determine color based on line content
            let text_color = self.get_line_color(line, &error.severity);

            print_with_color_and_background_at(
                y_position,
                ComponentDimensions::new(*bounds).content_bounds().left(),
                &Some(text_color),
                &Some(self.config.background_color.clone()),
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
        let viewable_height = ComponentDimensions::new(*bounds).content_bounds().height();
        let start_y = ComponentDimensions::new(*bounds).content_bounds().top();

        // Render combined error lines within bounds
        for (line_idx, &line) in error_lines.iter().take(viewable_height).enumerate() {
            let y_position = start_y + line_idx;
            if y_position > ComponentDimensions::new(*bounds).content_bounds().bottom() {
                break;
            }

            // Determine color based on line content
            let text_color = self.get_line_color(line, &ErrorSeverity::Error);

            print_with_color_and_background_at(
                y_position,
                ComponentDimensions::new(*bounds).content_bounds().left(),
                &Some(text_color),
                &Some(self.config.background_color.clone()),
                line,
                buffer,
            );
        }
    }

    /// Apply syntax highlighting to a line of code
    pub fn apply_syntax_highlighting(&self, line: &str) -> String {
        if !self.config.syntax_highlighting.enabled {
            return line.to_string();
        }

        let tokens = self.tokenize_line(line);
        let mut result = String::new();

        for (token_type, text) in tokens {
            let color = self.get_token_color(&token_type);
            if color != "white" && !text.trim().is_empty() {
                // Add ANSI color code for non-default colors
                result.push_str(&format!(
                    "\x1b[{}m{}\x1b[0m",
                    self.color_to_ansi(&color),
                    text
                ));
            } else {
                result.push_str(text);
            }
        }

        result
    }

    /// Tokenize a line of code for syntax highlighting
    fn tokenize_line<'a>(&self, line: &'a str) -> Vec<(SyntaxToken, &'a str)> {
        let mut tokens = Vec::new();
        let mut current_pos = 0;
        let chars: Vec<char> = line.chars().collect();

        while current_pos < chars.len() {
            let remaining = &line[current_pos..];

            // Skip whitespace
            if chars[current_pos].is_whitespace() {
                let whitespace_end = self.find_whitespace_end(&chars, current_pos);
                tokens.push((SyntaxToken::Text, &line[current_pos..whitespace_end]));
                current_pos = whitespace_end;
                continue;
            }

            // Comments
            if remaining.starts_with("//") || remaining.starts_with("#") {
                tokens.push((SyntaxToken::Comment, &line[current_pos..]));
                break;
            }

            // String literals
            if chars[current_pos] == '"' {
                let string_end = self.find_string_end(&chars, current_pos);
                tokens.push((SyntaxToken::String, &line[current_pos..string_end]));
                current_pos = string_end;
                continue;
            }

            // Numbers
            if chars[current_pos].is_ascii_digit() {
                let number_end = self.find_number_end(&chars, current_pos);
                tokens.push((SyntaxToken::Number, &line[current_pos..number_end]));
                current_pos = number_end;
                continue;
            }

            // Keywords and identifiers
            if chars[current_pos].is_alphabetic() || chars[current_pos] == '_' {
                let word_end = self.find_word_end(&chars, current_pos);
                let word = &line[current_pos..word_end];
                let token_type = self.classify_word(word);
                tokens.push((token_type, word));
                current_pos = word_end;
                continue;
            }

            // Operators and punctuation
            let operator_end = self.find_operator_end(&chars, current_pos);
            let operator = &line[current_pos..operator_end];
            let token_type = if self.is_operator(operator) {
                SyntaxToken::Operator
            } else {
                SyntaxToken::Punctuation
            };
            tokens.push((token_type, operator));
            current_pos = operator_end;
        }

        tokens
    }

    /// Get color for a specific token type
    fn get_token_color(&self, token_type: &SyntaxToken) -> String {
        match token_type {
            SyntaxToken::Keyword => self.config.syntax_highlighting.keyword_color.clone(),
            SyntaxToken::String => self.config.syntax_highlighting.string_color.clone(),
            SyntaxToken::Number => self.config.syntax_highlighting.number_color.clone(),
            SyntaxToken::Comment => self.config.syntax_highlighting.comment_color.clone(),
            SyntaxToken::Function => self.config.syntax_highlighting.function_color.clone(),
            SyntaxToken::Type => self.config.syntax_highlighting.type_color.clone(),
            SyntaxToken::Variable => self.config.syntax_highlighting.variable_color.clone(),
            SyntaxToken::Operator => self.config.syntax_highlighting.operator_color.clone(),
            SyntaxToken::Punctuation => self.config.syntax_highlighting.punctuation_color.clone(),
            SyntaxToken::Text => "white".to_string(),
        }
    }

    /// Convert color name to ANSI escape code
    fn color_to_ansi(&self, color: &str) -> &'static str {
        match color {
            "black" => "30",
            "red" => "31",
            "green" => "32",
            "yellow" => "33",
            "blue" => "34",
            "magenta" => "35",
            "cyan" => "36",
            "white" => "37",
            "bright_black" => "90",
            "bright_red" => "91",
            "bright_green" => "92",
            "bright_yellow" => "93",
            "bright_blue" => "94",
            "bright_magenta" => "95",
            "bright_cyan" => "96",
            "bright_white" => "97",
            _ => "37", // Default to white
        }
    }

    /// Find end of whitespace sequence
    fn find_whitespace_end(&self, chars: &[char], start: usize) -> usize {
        let mut pos = start;
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        pos
    }

    /// Find end of string literal
    fn find_string_end(&self, chars: &[char], start: usize) -> usize {
        let mut pos = start + 1; // Skip opening quote
        while pos < chars.len() {
            if chars[pos] == '"' && (pos == 0 || chars[pos - 1] != '\\') {
                return pos + 1;
            }
            pos += 1;
        }
        chars.len() // Unclosed string
    }

    /// Find end of number literal
    fn find_number_end(&self, chars: &[char], start: usize) -> usize {
        let mut pos = start;
        while pos < chars.len()
            && (chars[pos].is_ascii_digit() || chars[pos] == '.' || chars[pos] == '_')
        {
            pos += 1;
        }
        pos
    }

    /// Find end of word (identifier/keyword)
    fn find_word_end(&self, chars: &[char], start: usize) -> usize {
        let mut pos = start;
        while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
            pos += 1;
        }
        pos
    }

    /// Find end of operator sequence
    fn find_operator_end(&self, chars: &[char], start: usize) -> usize {
        let pos = start + 1;
        let first_char = chars[start];

        // Handle multi-character operators
        if pos < chars.len() {
            let second_char = chars[pos];
            match (first_char, second_char) {
                ('=', '=')
                | ('!', '=')
                | ('<', '=')
                | ('>', '=')
                | ('+', '=')
                | ('-', '=')
                | ('*', '=')
                | ('/', '=')
                | ('-', '>')
                | (':', ':')
                | ('.', '.') => pos + 1,
                _ => pos,
            }
        } else {
            pos
        }
    }

    /// Classify a word as keyword, type, function, or variable
    fn classify_word(&self, word: &str) -> SyntaxToken {
        match self.config.syntax_highlighting.language.as_str() {
            "rust" => self.classify_rust_word(word),
            "yaml" => self.classify_yaml_word(word),
            "json" => SyntaxToken::Variable, // JSON has no keywords
            _ => SyntaxToken::Variable,
        }
    }

    /// Classify Rust language keywords and identifiers
    fn classify_rust_word(&self, word: &str) -> SyntaxToken {
        match word {
            // Rust keywords
            "fn" | "let" | "mut" | "const" | "static" | "impl" | "struct" | "enum" | "trait"
            | "pub" | "use" | "mod" | "crate" | "super" | "self" | "Self" | "match" | "if"
            | "else" | "while" | "for" | "loop" | "break" | "continue" | "return" | "async"
            | "await" | "move" | "ref" | "as" | "in" | "where" | "unsafe" | "extern" => {
                SyntaxToken::Keyword
            }

            // Common types
            "String" | "str" | "Vec" | "Option" | "Result" | "Box" | "Rc" | "Arc" | "i8"
            | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
            | "usize" | "f32" | "f64" | "bool" | "char" => SyntaxToken::Type,

            _ => {
                // Function detection (simple heuristic)
                if word.chars().next().is_some_and(|c| c.is_lowercase())
                    && word.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    SyntaxToken::Function
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    SyntaxToken::Type
                } else {
                    SyntaxToken::Variable
                }
            }
        }
    }

    /// Classify YAML language keywords
    fn classify_yaml_word(&self, word: &str) -> SyntaxToken {
        match word {
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" => SyntaxToken::Keyword,
            _ => SyntaxToken::Variable,
        }
    }

    /// Check if text is an operator
    fn is_operator(&self, text: &str) -> bool {
        matches!(
            text,
            "+" | "-"
                | "*"
                | "/"
                | "%"
                | "="
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "&&"
                | "||"
                | "!"
                | "&"
                | "|"
                | "^"
                | "<<"
                | ">>"
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "^="
                | "&="
                | "|="
                | "<<="
                | ">>="
                | "->"
                | "::"
                | ".."
                | "..."
                | "?"
                | "@"
                | "#"
        )
    }

    /// Generate syntax-highlighted error display text
    pub fn format_error_with_highlighting(&self, error: &ErrorInfo, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();

        // Ensure we have valid line numbers (using 1-based indexing)
        if error.line_number == 0 || error.line_number > lines.len() {
            return format!("{}: {}", self.severity_text(&error.severity), error.message);
        }

        let error_line = lines[error.line_number - 1];
        let line_num_width = self.calculate_line_number_width(error, &lines);

        let mut result = String::new();

        // Error header with severity
        result.push_str(&format!(
            "{}: {}\n",
            self.severity_text(&error.severity),
            error.message
        ));

        // File location
        result.push_str(&format!(
            " --> {}:{}:{}\n",
            error.file_path, error.line_number, error.column_number
        ));

        // Context lines before error (if enabled)
        if self.config.show_context {
            let start_line = error
                .line_number
                .saturating_sub(self.config.context_lines + 1);
            for line_idx in start_line..(error.line_number - 1) {
                if line_idx < lines.len() {
                    let highlighted_line = self.apply_syntax_highlighting(lines[line_idx]);
                    result.push_str(&format!(
                        "{:width$} | {}\n",
                        line_idx + 1,
                        highlighted_line,
                        width = line_num_width
                    ));
                }
            }
        }

        // Separator
        result.push_str(&format!("{}|\n", " ".repeat(line_num_width + 1)));

        // Error line with syntax highlighting
        let highlighted_error_line = self.apply_syntax_highlighting(error_line);
        result.push_str(&format!(
            "{:width$} | {}\n",
            error.line_number,
            highlighted_error_line,
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
                let highlighted_line = self.apply_syntax_highlighting(lines[line_idx]);
                result.push_str(&format!(
                    "{:width$} | {}\n",
                    line_idx + 1,
                    highlighted_line,
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
        } else if line.starts_with("hint:") || line.starts_with("help:") {
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
        yaml_error.location().map(|location| ErrorInfo {
            message,
            file_path: file_path.to_string(),
            line_number: location.line(),
            column_number: location.column(),
            severity: ErrorSeverity::Error,
            help: Some("Check YAML syntax and structure".to_string()),
            note: None,
            caret_positioning: None,
        })
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
            caret_positioning: None,
        }
    }

    /// Render error with enhanced multi-line caret positioning
    pub fn render_with_enhanced_carets(
        &self,
        error: &ErrorInfo,
        content: &str,
        bounds: &Bounds,
        buffer: &mut ScreenBuffer,
    ) {
        if let Some(caret_positioning) = &error.caret_positioning {
            let formatted_error =
                self.format_error_with_multi_line_carets(error, content, caret_positioning);
            let error_lines: Vec<&str> = formatted_error.lines().collect();

            let viewable_height = ComponentDimensions::new(*bounds).content_bounds().height();
            let start_y = ComponentDimensions::new(*bounds).content_bounds().top();

            // Render error lines within bounds
            for (line_idx, &line) in error_lines.iter().take(viewable_height).enumerate() {
                let y_position = start_y + line_idx;
                if y_position > ComponentDimensions::new(*bounds).content_bounds().bottom() {
                    break;
                }

                // Enhanced color determination for multi-line carets
                let text_color =
                    self.get_enhanced_line_color(line, &error.severity, caret_positioning);

                print_with_color_and_background_at(
                    y_position,
                    ComponentDimensions::new(*bounds).content_bounds().left(),
                    &Some(text_color),
                    &Some(self.config.background_color.clone()),
                    line,
                    buffer,
                );
            }
        } else {
            // Fallback to standard rendering
            self.render(error, content, bounds, buffer);
        }
    }

    /// Format error message with enhanced multi-line caret positioning
    pub fn format_error_with_multi_line_carets(
        &self,
        error: &ErrorInfo,
        content: &str,
        caret_positioning: &CaretPositioning,
    ) -> String {
        let mut result = String::new();

        // Header with severity and message
        let _severity_color = self.get_severity_color(&error.severity);
        result.push_str(&format!(
            "{}: {}\n",
            match error.severity {
                ErrorSeverity::Error => "error",
                ErrorSeverity::Warning => "warning",
                ErrorSeverity::Info => "info",
                ErrorSeverity::Hint => "hint",
            },
            error.message
        ));

        // File location
        result.push_str(&format!(
            " --> {}:{}:{}\n",
            error.file_path,
            caret_positioning.primary_span.start_line,
            caret_positioning.primary_span.start_column
        ));

        let lines: Vec<&str> = content.lines().collect();
        let line_number_width = self.calculate_line_number_width(error, &lines);

        // Render multi-line error span
        self.render_multi_line_span(
            &mut result,
            &lines,
            &caret_positioning.primary_span,
            line_number_width,
            true, // is_primary
        );

        // Render secondary spans
        for secondary_span in &caret_positioning.secondary_spans {
            result.push_str(&format!("   {}\n", secondary_span.message));
            self.render_multi_line_span(
                &mut result,
                &lines,
                secondary_span,
                line_number_width,
                false, // is_primary
            );
        }

        // Add help and note
        if let Some(help) = &error.help {
            result.push_str(&format!("   = help: {}\n", help));
        }
        if let Some(note) = &error.note {
            result.push_str(&format!("   = note: {}\n", note));
        }

        result
    }

    /// Render a multi-line error span with proper caret indicators
    fn render_multi_line_span(
        &self,
        result: &mut String,
        lines: &[&str],
        span: &ErrorSpan,
        line_number_width: usize,
        _is_primary: bool,
    ) {
        let start_line = span.start_line.saturating_sub(1);
        let end_line = span.end_line.saturating_sub(1);

        // Show context lines before if enabled
        let context_start = if self.config.show_context {
            start_line.saturating_sub(self.config.context_lines)
        } else {
            start_line
        };

        let context_end = if self.config.show_context {
            (end_line + self.config.context_lines + 1).min(lines.len())
        } else {
            end_line + 1
        };

        for line_idx in context_start..context_end {
            if line_idx >= lines.len() {
                break;
            }

            let line_num = line_idx + 1;
            let is_error_line = line_idx >= start_line && line_idx <= end_line;
            let line_content = lines[line_idx];

            // Format line number with proper padding
            let line_num_str = if is_error_line {
                format!("{:width$}", line_num, width = line_number_width)
            } else {
                " ".repeat(line_number_width)
            };

            // Apply syntax highlighting
            let highlighted_content = self.apply_syntax_highlighting(line_content);

            if is_error_line {
                result.push_str(&format!("{} | {}\n", line_num_str, highlighted_content));

                // Add caret indicators for error lines
                if span.start_line == span.end_line {
                    // Single line error - traditional caret
                    let spaces_before_pipe = " ".repeat(line_number_width);
                    let spaces_before_caret = " ".repeat(span.start_column.saturating_sub(1));
                    let caret_length = if span.end_column > span.start_column {
                        span.end_column - span.start_column
                    } else {
                        1
                    };
                    let carets = self.config.caret_char.to_string().repeat(caret_length);
                    result.push_str(&format!(
                        "{} | {}{}\n",
                        spaces_before_pipe, spaces_before_caret, carets
                    ));
                } else {
                    // Multi-line error - enhanced carets
                    let spaces_before_pipe = " ".repeat(line_number_width);

                    if line_idx == start_line {
                        // Start line - show start character and line continuation
                        let spaces_before_caret = " ".repeat(span.start_column.saturating_sub(1));
                        let continuation_length = line_content
                            .len()
                            .saturating_sub(span.start_column.saturating_sub(1));
                        let continuation = if continuation_length > 0 {
                            format!("┌{}", "─".repeat(continuation_length.saturating_sub(1)))
                        } else {
                            "┌".to_string()
                        };
                        result.push_str(&format!(
                            "{} | {}{}\n",
                            spaces_before_pipe, spaces_before_caret, continuation
                        ));
                    } else if line_idx == end_line {
                        // End line - show end character and end caret
                        let end_caret_length = span.end_column.saturating_sub(1);
                        let end_continuation = if end_caret_length > 0 {
                            format!(
                                "{}└{}",
                                "─".repeat(end_caret_length.saturating_sub(1)),
                                self.config.caret_char
                            )
                        } else {
                            format!("└{}", self.config.caret_char)
                        };
                        result
                            .push_str(&format!("{} | {}\n", spaces_before_pipe, end_continuation));
                    } else {
                        // Middle line - show continuation character
                        result.push_str(&format!("{} | │\n", spaces_before_pipe));
                    }
                }
            } else {
                // Context line
                result.push_str(&format!("{} | {}\n", line_num_str, highlighted_content));
            }
        }
    }

    /// Get color for error severity level
    fn get_severity_color(&self, severity: &ErrorSeverity) -> String {
        match severity {
            ErrorSeverity::Error => self.config.error_color.clone(),
            ErrorSeverity::Warning => self.config.warning_color.clone(),
            ErrorSeverity::Info => self.config.info_color.clone(),
            ErrorSeverity::Hint => self.config.hint_color.clone(),
        }
    }

    /// Get enhanced line color for multi-line caret positioning
    fn get_enhanced_line_color(
        &self,
        line: &str,
        severity: &ErrorSeverity,
        caret_positioning: &CaretPositioning,
    ) -> String {
        // Check for multi-line caret indicators
        if line.contains(caret_positioning.multi_line_start_char)
            || line.contains(caret_positioning.multi_line_middle_char)
            || line.contains(caret_positioning.multi_line_end_char)
        {
            return self.config.caret_color.clone();
        }

        // Standard line color determination
        self.get_line_color(line, severity)
    }

    /// Create ErrorInfo with enhanced caret positioning for multi-line errors
    pub fn create_multi_line_error(
        message: String,
        file_path: String,
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
        severity: ErrorSeverity,
    ) -> ErrorInfo {
        let primary_span = ErrorSpan {
            start_line,
            start_column,
            end_line,
            end_column,
            message: message.clone(),
        };

        ErrorInfo {
            message,
            file_path,
            line_number: start_line,
            column_number: start_column,
            severity,
            help: None,
            note: None,
            caret_positioning: Some(CaretPositioning {
                primary_span,
                ..Default::default()
            }),
        }
    }

    /// Add secondary span to existing error for additional context
    pub fn add_secondary_span(
        &self,
        error: &mut ErrorInfo,
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
        message: String,
    ) {
        let secondary_span = ErrorSpan {
            start_line,
            start_column,
            end_line,
            end_column,
            message,
        };

        if let Some(caret_positioning) = &mut error.caret_positioning {
            caret_positioning.secondary_spans.push(secondary_span);
        } else {
            error.caret_positioning = Some(CaretPositioning {
                secondary_spans: vec![secondary_span],
                ..Default::default()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_highlight_config_creation() {
        let config = SyntaxHighlightConfig::default();
        assert!(config.enabled);
        assert_eq!(config.language, "rust");
        assert_eq!(config.keyword_color, "magenta");
        assert_eq!(config.string_color, "green");
    }

    #[test]
    fn test_error_display_with_syntax_highlighting() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        assert!(display.config.syntax_highlighting.enabled);
        assert_eq!(display.config.syntax_highlighting.language, "rust");
    }

    #[test]
    fn test_error_display_with_custom_syntax_config() {
        let mut syntax_config = SyntaxHighlightConfig::default();
        syntax_config.enabled = false;
        syntax_config.language = "yaml".to_string();

        let display = ErrorDisplay::with_custom_syntax_config("test".to_string(), syntax_config);
        assert!(!display.config.syntax_highlighting.enabled);
        assert_eq!(display.config.syntax_highlighting.language, "yaml");
    }

    #[test]
    fn test_tokenize_rust_line() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let tokens = display.tokenize_line("fn main() {");

        assert!(tokens.len() >= 4);
        // Find keyword token
        let keyword_token = tokens
            .iter()
            .find(|(token_type, text)| *token_type == SyntaxToken::Keyword && *text == "fn");
        assert!(keyword_token.is_some());
    }

    #[test]
    fn test_tokenize_string_literal() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let tokens = display.tokenize_line("let msg = \"hello\";");

        // Find string token
        let string_token = tokens
            .iter()
            .find(|(token_type, _)| *token_type == SyntaxToken::String);
        assert!(string_token.is_some());
    }

    #[test]
    fn test_tokenize_number_literal() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let tokens = display.tokenize_line("let x = 42;");

        // Find number token
        let number_token = tokens
            .iter()
            .find(|(token_type, text)| *token_type == SyntaxToken::Number && *text == "42");
        assert!(number_token.is_some());
    }

    #[test]
    fn test_tokenize_comment() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let tokens = display.tokenize_line("// This is a comment");

        // Should have comment token for the entire line
        let comment_token = tokens
            .iter()
            .find(|(token_type, _)| *token_type == SyntaxToken::Comment);
        assert!(comment_token.is_some());
    }

    #[test]
    fn test_classify_rust_keywords() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        assert_eq!(display.classify_rust_word("fn"), SyntaxToken::Keyword);
        assert_eq!(display.classify_rust_word("let"), SyntaxToken::Keyword);
        assert_eq!(display.classify_rust_word("impl"), SyntaxToken::Keyword);
        assert_eq!(display.classify_rust_word("struct"), SyntaxToken::Keyword);
    }

    #[test]
    fn test_classify_rust_types() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        assert_eq!(display.classify_rust_word("String"), SyntaxToken::Type);
        assert_eq!(display.classify_rust_word("Vec"), SyntaxToken::Type);
        assert_eq!(display.classify_rust_word("i32"), SyntaxToken::Type);
        assert_eq!(display.classify_rust_word("bool"), SyntaxToken::Type);
    }

    #[test]
    fn test_classify_yaml_keywords() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "yaml".to_string());

        assert_eq!(display.classify_yaml_word("true"), SyntaxToken::Keyword);
        assert_eq!(display.classify_yaml_word("false"), SyntaxToken::Keyword);
        assert_eq!(display.classify_yaml_word("null"), SyntaxToken::Keyword);
        assert_eq!(display.classify_yaml_word("yes"), SyntaxToken::Keyword);
    }

    #[test]
    fn test_is_operator() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        assert!(display.is_operator("="));
        assert!(display.is_operator("=="));
        assert!(display.is_operator("!="));
        assert!(display.is_operator("->"));
        assert!(display.is_operator("::"));
        assert!(!display.is_operator("abc"));
    }

    #[test]
    fn test_color_to_ansi() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        assert_eq!(display.color_to_ansi("red"), "31");
        assert_eq!(display.color_to_ansi("green"), "32");
        assert_eq!(display.color_to_ansi("bright_red"), "91");
        assert_eq!(display.color_to_ansi("unknown"), "37"); // Default to white
    }

    #[test]
    fn test_apply_syntax_highlighting_disabled() {
        let mut config = ErrorDisplayConfig::default();
        config.syntax_highlighting.enabled = false;
        let display = ErrorDisplay::new("test".to_string(), config);

        let line = "fn main() {}";
        let highlighted = display.apply_syntax_highlighting(line);
        assert_eq!(highlighted, line); // Should be unchanged when disabled
    }

    #[test]
    fn test_apply_syntax_highlighting_enabled() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        let line = "fn main() {}";
        let highlighted = display.apply_syntax_highlighting(line);
        assert!(highlighted.contains("\x1b[")); // Should contain ANSI codes
        assert!(highlighted.contains("fn")); // Should still contain the text
    }

    #[test]
    fn test_format_error_with_highlighting() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        let error = ErrorInfo {
            message: "expected `;`".to_string(),
            file_path: "test.rs".to_string(),
            line_number: 2,
            column_number: 15,
            severity: ErrorSeverity::Error,
            help: Some("add semicolon".to_string()),
            note: Some("syntax error".to_string()),
            caret_positioning: None,
        };

        let content = "fn main() {\n    let x = 42\n}";
        let formatted = display.format_error_with_highlighting(&error, content);

        assert!(formatted.contains("error: expected `;`"));
        assert!(formatted.contains(" --> test.rs:2:15"));
        assert!(formatted.contains("help: add semicolon"));
        assert!(formatted.contains("note: syntax error"));
        // Should contain syntax highlighting for the error line
        assert!(formatted.contains("let"));
    }

    #[test]
    fn test_find_string_end() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let chars: Vec<char> = "\"hello world\"".chars().collect();

        let end = display.find_string_end(&chars, 0);
        assert_eq!(end, 13); // Position after closing quote
    }

    #[test]
    fn test_find_string_end_unclosed() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let chars: Vec<char> = "\"hello world".chars().collect();

        let end = display.find_string_end(&chars, 0);
        assert_eq!(end, chars.len()); // Should return length for unclosed string
    }

    #[test]
    fn test_find_number_end() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let chars: Vec<char> = "123.45_u32".chars().collect();

        let end = display.find_number_end(&chars, 0);
        assert_eq!(end, 7); // Should include digits, dots, and underscores but not letters
    }

    #[test]
    fn test_find_word_end() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let chars: Vec<char> = "hello_world123 ".chars().collect();

        let end = display.find_word_end(&chars, 0);
        assert_eq!(end, 14); // Should include alphanumeric and underscores
    }

    #[test]
    fn test_find_operator_end_single_char() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let chars: Vec<char> = "+ 1".chars().collect();

        let end = display.find_operator_end(&chars, 0);
        assert_eq!(end, 1); // Single character operator
    }

    #[test]
    fn test_find_operator_end_multi_char() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let chars: Vec<char> = "== 1".chars().collect();

        let end = display.find_operator_end(&chars, 0);
        assert_eq!(end, 2); // Multi-character operator
    }

    #[test]
    fn test_get_token_color() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());

        assert_eq!(display.get_token_color(&SyntaxToken::Keyword), "magenta");
        assert_eq!(display.get_token_color(&SyntaxToken::String), "green");
        assert_eq!(display.get_token_color(&SyntaxToken::Number), "yellow");
        assert_eq!(
            display.get_token_color(&SyntaxToken::Comment),
            "bright_black"
        );
        assert_eq!(display.get_token_color(&SyntaxToken::Text), "white");
    }

    fn create_test_buffer() -> ScreenBuffer {
        ScreenBuffer::new()
    }

    #[test]
    fn test_multi_language_support() {
        // Test Rust highlighting
        let rust_display =
            ErrorDisplay::with_syntax_highlighting("rust_test".to_string(), "rust".to_string());
        assert_eq!(rust_display.config.syntax_highlighting.language, "rust");
        assert_eq!(rust_display.classify_rust_word("fn"), SyntaxToken::Keyword);

        // Test YAML highlighting
        let yaml_display =
            ErrorDisplay::with_syntax_highlighting("yaml_test".to_string(), "yaml".to_string());
        assert_eq!(yaml_display.config.syntax_highlighting.language, "yaml");
        assert_eq!(
            yaml_display.classify_yaml_word("true"),
            SyntaxToken::Keyword
        );
    }

    #[test]
    fn test_complex_tokenization() {
        let display =
            ErrorDisplay::with_syntax_highlighting("test".to_string(), "rust".to_string());
        let tokens = display
            .tokenize_line("let result: Result<String, Error> = Ok(\"success\".to_string());");

        // Verify we have various token types
        let has_keyword = tokens.iter().any(|(t, _)| *t == SyntaxToken::Keyword);
        let has_type = tokens.iter().any(|(t, _)| *t == SyntaxToken::Type);
        let has_string = tokens.iter().any(|(t, _)| *t == SyntaxToken::String);
        let has_operator = tokens.iter().any(|(t, _)| *t == SyntaxToken::Operator);
        let has_punctuation = tokens.iter().any(|(t, _)| *t == SyntaxToken::Punctuation);

        assert!(has_keyword);
        assert!(has_type);
        assert!(has_string);
        assert!(has_operator);
        assert!(has_punctuation);
    }

    #[test]
    fn test_error_span_creation() {
        let span = ErrorSpan {
            start_line: 5,
            start_column: 10,
            end_line: 7,
            end_column: 15,
            message: "Multi-line error span".to_string(),
        };

        assert_eq!(span.start_line, 5);
        assert_eq!(span.start_column, 10);
        assert_eq!(span.end_line, 7);
        assert_eq!(span.end_column, 15);
        assert_eq!(span.message, "Multi-line error span");
    }

    #[test]
    fn test_caret_positioning_default() {
        let caret_positioning = CaretPositioning::default();

        assert_eq!(caret_positioning.primary_span.start_line, 1);
        assert_eq!(caret_positioning.primary_span.start_column, 1);
        assert_eq!(caret_positioning.primary_span.end_line, 1);
        assert_eq!(caret_positioning.primary_span.end_column, 1);
        assert!(caret_positioning.secondary_spans.is_empty());
        assert!(caret_positioning.show_multi_line_carets);
        assert!(caret_positioning.show_line_continuations);
        assert_eq!(caret_positioning.multi_line_start_char, '┌');
        assert_eq!(caret_positioning.multi_line_middle_char, '│');
        assert_eq!(caret_positioning.multi_line_end_char, '└');
    }

    #[test]
    fn test_create_multi_line_error() {
        let error = ErrorDisplay::create_multi_line_error(
            "Multi-line syntax error".to_string(),
            "test.rs".to_string(),
            10,
            5,
            12,
            8,
            ErrorSeverity::Error,
        );

        assert_eq!(error.message, "Multi-line syntax error");
        assert_eq!(error.file_path, "test.rs");
        assert_eq!(error.line_number, 10);
        assert_eq!(error.column_number, 5);
        assert_eq!(error.severity, ErrorSeverity::Error);

        let caret_positioning = error.caret_positioning.unwrap();
        assert_eq!(caret_positioning.primary_span.start_line, 10);
        assert_eq!(caret_positioning.primary_span.start_column, 5);
        assert_eq!(caret_positioning.primary_span.end_line, 12);
        assert_eq!(caret_positioning.primary_span.end_column, 8);
    }

    #[test]
    fn test_add_secondary_span() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        let mut error = ErrorDisplay::create_multi_line_error(
            "Primary error".to_string(),
            "main.rs".to_string(),
            5,
            10,
            5,
            20,
            ErrorSeverity::Error,
        );

        display.add_secondary_span(&mut error, 8, 5, 8, 15, "Related issue here".to_string());

        let caret_positioning = error.caret_positioning.unwrap();
        assert_eq!(caret_positioning.secondary_spans.len(), 1);

        let secondary = &caret_positioning.secondary_spans[0];
        assert_eq!(secondary.start_line, 8);
        assert_eq!(secondary.start_column, 5);
        assert_eq!(secondary.end_line, 8);
        assert_eq!(secondary.end_column, 15);
        assert_eq!(secondary.message, "Related issue here");
    }

    #[test]
    fn test_enhanced_caret_formatting_single_line() {
        let mut config = ErrorDisplayConfig::default();
        config.syntax_highlighting.enabled = false; // Disable syntax highlighting for cleaner test assertions
        let display = ErrorDisplay::new("test".to_string(), config);
        let error = ErrorDisplay::create_multi_line_error(
            "Single line error".to_string(),
            "test.rs".to_string(),
            3,
            10,
            3,
            15,
            ErrorSeverity::Error,
        );

        let content = "line 1\nline 2\nthis is line 3 with error\nline 4";
        let caret_positioning = error.caret_positioning.as_ref().unwrap();
        let formatted =
            display.format_error_with_multi_line_carets(&error, &content, caret_positioning);

        assert!(formatted.contains("error: Single line error"));
        assert!(formatted.contains("--> test.rs:3:10"));
        assert!(formatted.contains("this is line 3 with error"));
        assert!(formatted.contains("^^^^^")); // 5 caret characters for positions 10-15
    }

    #[test]
    fn test_enhanced_caret_formatting_multi_line() {
        let mut config = ErrorDisplayConfig::default();
        config.syntax_highlighting.enabled = false; // Disable syntax highlighting for cleaner test assertions
        let display = ErrorDisplay::new("test".to_string(), config);
        let error = ErrorDisplay::create_multi_line_error(
            "Multi-line error".to_string(),
            "test.rs".to_string(),
            2,
            5,
            4,
            10,
            ErrorSeverity::Warning,
        );

        let content = "line 1\nstart error here\nmiddle line\nend error here\nline 5";
        let caret_positioning = error.caret_positioning.as_ref().unwrap();
        let formatted =
            display.format_error_with_multi_line_carets(&error, &content, caret_positioning);

        assert!(formatted.contains("warning: Multi-line error"));
        assert!(formatted.contains("--> test.rs:2:5"));
        assert!(formatted.contains("start error here"));
        assert!(formatted.contains("middle line"));
        assert!(formatted.contains("end error here"));
        assert!(formatted.contains("┌")); // Multi-line start
        assert!(formatted.contains("│")); // Multi-line middle
        assert!(formatted.contains("└")); // Multi-line end
    }

    #[test]
    fn test_enhanced_caret_with_secondary_spans() {
        let mut config = ErrorDisplayConfig::default();
        config.syntax_highlighting.enabled = false; // Disable syntax highlighting for cleaner test assertions
        let display = ErrorDisplay::new("test".to_string(), config);
        let mut error = ErrorDisplay::create_multi_line_error(
            "Primary error with context".to_string(),
            "lib.rs".to_string(),
            10,
            1,
            10,
            20,
            ErrorSeverity::Error,
        );

        display.add_secondary_span(&mut error, 15, 5, 15, 12, "Related definition".to_string());

        let content = (1..=20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let caret_positioning = error.caret_positioning.as_ref().unwrap();
        let formatted =
            display.format_error_with_multi_line_carets(&error, &content, caret_positioning);

        assert!(formatted.contains("Primary error with context"));
        assert!(formatted.contains("line 10"));
        assert!(formatted.contains("Related definition"));
        assert!(formatted.contains("line 15"));
    }

    #[test]
    fn test_enhanced_line_color_detection() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        let caret_positioning = CaretPositioning::default();

        // Test caret line detection
        let caret_line = format!("   | {}", caret_positioning.multi_line_start_char);
        let color =
            display.get_enhanced_line_color(&caret_line, &ErrorSeverity::Error, &caret_positioning);
        assert_eq!(color, "bright_red"); // Should use caret color

        // Test normal line with line number format (contains " | ")
        let normal_line = "   | regular code line";
        let color = display.get_enhanced_line_color(
            &normal_line,
            &ErrorSeverity::Error,
            &caret_positioning,
        );
        assert_eq!(color, "bright_blue"); // Should use line_number_color for lines with " | "
    }

    #[test]
    fn test_render_with_enhanced_carets() {
        let display = ErrorDisplay::with_defaults("test".to_string());
        let error = ErrorDisplay::create_multi_line_error(
            "Render test error".to_string(),
            "render.rs".to_string(),
            1,
            1,
            2,
            5,
            ErrorSeverity::Info,
        );

        let content = "first line\nsecond line";
        let bounds = Bounds::new(0, 0, 80, 10);
        let mut buffer = create_test_buffer();

        // Should not panic and should handle rendering gracefully
        display.render_with_enhanced_carets(&error, content, &bounds, &mut buffer);

        // Test fallback to standard rendering when no caret positioning
        let standard_error = ErrorInfo {
            message: "Standard error".to_string(),
            file_path: "std.rs".to_string(),
            line_number: 1,
            column_number: 1,
            severity: ErrorSeverity::Error,
            help: None,
            note: None,
            caret_positioning: None,
        };

        display.render_with_enhanced_carets(&standard_error, content, &bounds, &mut buffer);
    }

    #[test]
    fn test_multi_line_span_rendering() {
        let display = ErrorDisplay::with_detailed_config("detailed_test".to_string());
        let span = ErrorSpan {
            start_line: 2,
            start_column: 10,
            end_line: 4,
            end_column: 5,
            message: "Test span".to_string(),
        };

        let lines = vec![
            "line 1",
            "line 2 with error start",
            "line 3 middle",
            "line 4 error end",
            "line 5",
        ];
        let mut result = String::new();

        display.render_multi_line_span(&mut result, &lines, &span, 3, true);

        // Test content is present (may contain ANSI codes)
        assert!(
            result.contains("line")
                && result.contains("2")
                && result.contains("with")
                && result.contains("error")
                && result.contains("start")
        );
        assert!(result.contains("line") && result.contains("3") && result.contains("middle"));
        assert!(
            result.contains("line")
                && result.contains("4")
                && result.contains("error")
                && result.contains("end")
        );
        assert!(result.contains("┌")); // Start indicator
        assert!(result.contains("│")); // Middle indicator
        assert!(result.contains("└")); // End indicator
    }
}

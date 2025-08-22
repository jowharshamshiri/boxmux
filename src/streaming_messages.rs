use std::time::{SystemTime, Instant};
#[cfg(test)]
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamingOutput {
    pub panel_id: String,
    pub line_content: String,
    pub sequence: u64,
    pub timestamp: SystemTime,
    pub is_stderr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamingComplete {
    pub panel_id: String,
    pub task_id: Uuid,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub total_lines: u64,
    pub timestamp: SystemTime,
    pub command: Option<String>,
    pub stderr_output: Option<String>,
    pub error_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBatch {
    pub outputs: Vec<StreamingOutput>,
    pub batch_sequence: u64,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StreamingStatus {
    Starting,
    Running,
    RateLimited,
    Completed(bool), // success flag
    Failed(String),  // error message
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamingStatusUpdate {
    pub panel_id: String,
    pub task_id: Uuid,
    pub status: StreamingStatus,
    pub line_count: u64,
    pub timestamp: SystemTime,
}

impl StreamingOutput {
    pub fn new(
        panel_id: String, 
        line_content: String, 
        sequence: u64, 
        is_stderr: bool
    ) -> Self {
        Self {
            panel_id,
            line_content,
            sequence,
            timestamp: SystemTime::now(),
            is_stderr,
        }
    }
}

impl StreamingStatusUpdate {
    pub fn new(
        panel_id: String,
        task_id: Uuid,
        status: StreamingStatus,
        line_count: u64,
    ) -> Self {
        Self {
            panel_id,
            task_id,
            status,
            line_count,
            timestamp: SystemTime::now(),
        }
    }
}

impl StreamingComplete {
    pub fn new(
        panel_id: String,
        task_id: Uuid,
        exit_code: Option<i32>,
        total_lines: u64,
    ) -> Self {
        let success = exit_code.map_or(false, |code| code == 0);
        Self {
            panel_id,
            task_id,
            exit_code,
            success,
            total_lines,
            timestamp: SystemTime::now(),
            command: None,
            stderr_output: None,
            error_context: None,
        }
    }

    pub fn with_error_details(
        panel_id: String,
        task_id: Uuid,
        exit_code: Option<i32>,
        total_lines: u64,
        command: Option<String>,
        stderr_output: Option<String>,
        error_context: Option<String>,
    ) -> Self {
        let success = exit_code.map_or(false, |code| code == 0);
        Self {
            panel_id,
            task_id,
            exit_code,
            success,
            total_lines,
            timestamp: SystemTime::now(),
            command,
            stderr_output,
            error_context,
        }
    }

    pub fn format_error_message(&self) -> String {
        if self.success {
            return "Command completed successfully".to_string();
        }

        let mut error_parts = Vec::new();
        
        // Basic error with exit code
        match self.exit_code {
            Some(code) => error_parts.push(format!("Command failed with exit code: {}", code)),
            None => error_parts.push("Command failed (process error)".to_string()),
        }
        
        // Add command context if available
        if let Some(ref cmd) = self.command {
            error_parts.push(format!("Command: {}", cmd));
        }
        
        // Add error context if available
        if let Some(ref context) = self.error_context {
            error_parts.push(format!("Context: {}", context));
        }
        
        // Add stderr output if available
        if let Some(ref stderr) = self.stderr_output {
            if !stderr.trim().is_empty() {
                error_parts.push(format!("Error output: {}", stderr.trim()));
            }
        }
        
        error_parts.join("\n")
    }
}

impl OutputBatch {
    pub fn new(outputs: Vec<StreamingOutput>, batch_sequence: u64) -> Self {
        Self {
            outputs,
            batch_sequence,
            timestamp: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_output_creation() {
        let output = StreamingOutput::new(
            "panel1".to_string(),
            "test line".to_string(),
            1,
            false,
        );
        
        assert_eq!(output.panel_id, "panel1");
        assert_eq!(output.line_content, "test line");
        assert_eq!(output.sequence, 1);
        assert!(!output.is_stderr);
    }

    #[test]
    fn test_streaming_complete_creation() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::new(
            "panel1".to_string(),
            task_id,
            Some(0),
            10,
        );
        
        assert_eq!(complete.panel_id, "panel1");
        assert_eq!(complete.task_id, task_id);
        assert_eq!(complete.exit_code, Some(0));
        assert!(complete.success);
        assert_eq!(complete.total_lines, 10);
        assert!(complete.command.is_none());
        assert!(complete.stderr_output.is_none());
        assert!(complete.error_context.is_none());
    }

    #[test]
    fn test_streaming_complete_failure() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::new(
            "panel1".to_string(),
            task_id,
            Some(1),
            5,
        );
        
        assert!(!complete.success);
        assert_eq!(complete.exit_code, Some(1));
        assert!(complete.command.is_none());
        assert!(complete.stderr_output.is_none());
        assert!(complete.error_context.is_none());
    }

    #[test]
    fn test_output_batch_creation() {
        let outputs = vec![
            StreamingOutput::new("panel1".to_string(), "line1".to_string(), 1, false),
            StreamingOutput::new("panel1".to_string(), "line2".to_string(), 2, false),
        ];
        
        let batch = OutputBatch::new(outputs, 1);
        
        assert_eq!(batch.outputs.len(), 2);
        assert_eq!(batch.batch_sequence, 1);
        assert_eq!(batch.outputs[0].line_content, "line1");
        assert_eq!(batch.outputs[1].line_content, "line2");
    }

    #[test]
    fn test_stderr_output() {
        let output = StreamingOutput::new(
            "panel1".to_string(),
            "error message".to_string(),
            1,
            true,
        );
        
        assert!(output.is_stderr);
        assert_eq!(output.line_content, "error message");
    }

    #[test]
    fn test_streaming_complete_with_error_details() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::with_error_details(
            "panel1".to_string(),
            task_id,
            Some(2),
            5,
            Some("echo 'test' && false".to_string()),
            Some("command not found: false".to_string()),
            Some("Script execution failed".to_string()),
        );
        
        assert_eq!(complete.panel_id, "panel1");
        assert_eq!(complete.task_id, task_id);
        assert_eq!(complete.exit_code, Some(2));
        assert!(!complete.success);
        assert_eq!(complete.total_lines, 5);
        assert_eq!(complete.command, Some("echo 'test' && false".to_string()));
        assert_eq!(complete.stderr_output, Some("command not found: false".to_string()));
        assert_eq!(complete.error_context, Some("Script execution failed".to_string()));
    }

    #[test]
    fn test_streaming_complete_format_error_message_success() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::new(
            "panel1".to_string(),
            task_id,
            Some(0),
            10,
        );
        
        let error_msg = complete.format_error_message();
        assert_eq!(error_msg, "Command completed successfully");
    }

    #[test]
    fn test_streaming_complete_format_error_message_simple_failure() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::new(
            "panel1".to_string(),
            task_id,
            Some(1),
            5,
        );
        
        let error_msg = complete.format_error_message();
        assert_eq!(error_msg, "Command failed with exit code: 1");
    }

    #[test]
    fn test_streaming_complete_format_error_message_comprehensive() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::with_error_details(
            "panel1".to_string(),
            task_id,
            Some(2),
            5,
            Some("echo 'test' && false".to_string()),
            Some("command not found: false".to_string()),
            Some("Script execution failed".to_string()),
        );
        
        let error_msg = complete.format_error_message();
        let expected = "Command failed with exit code: 2\nCommand: echo 'test' && false\nContext: Script execution failed\nError output: command not found: false";
        assert_eq!(error_msg, expected);
    }

    #[test]
    fn test_streaming_complete_format_error_message_no_exit_code() {
        let task_id = Uuid::new_v4();
        let complete = StreamingComplete::with_error_details(
            "panel1".to_string(),
            task_id,
            None,
            0,
            Some("echo 'test'".to_string()),
            None,
            Some("Process spawn failed".to_string()),
        );
        
        let error_msg = complete.format_error_message();
        let expected = "Command failed (process error)\nCommand: echo 'test'\nContext: Process spawn failed";
        assert_eq!(error_msg, expected);
    }
}

/// Token bucket implementation for rate limiting streaming messages
#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    tokens: Arc<Mutex<u32>>,
    refill_rate: u32, // tokens per second
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: Arc::new(Mutex::new(capacity)),
            refill_rate,
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Try to consume tokens, returns true if successful
    pub fn try_consume(&self, tokens_needed: u32) -> bool {
        self.refill_tokens();
        
        let mut tokens = self.tokens.lock().unwrap();
        if *tokens >= tokens_needed {
            *tokens -= tokens_needed;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.lock().unwrap();
        let mut tokens = self.tokens.lock().unwrap();
        
        let elapsed = now.duration_since(*last_refill);
        let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate as f64) as u32;
        
        if tokens_to_add > 0 {
            *tokens = (*tokens + tokens_to_add).min(self.capacity);
            *last_refill = now;
        }
    }

    /// Check available tokens without consuming
    pub fn available_tokens(&self) -> u32 {
        self.refill_tokens();
        *self.tokens.lock().unwrap()
    }
}

/// Rate limiter for streaming messages with per-panel token buckets
#[derive(Debug)]
pub struct StreamingRateLimiter {
    panel_buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    default_capacity: u32,
    default_refill_rate: u32,
}

impl StreamingRateLimiter {
    /// Create new rate limiter with default settings
    pub fn new(default_capacity: u32, default_refill_rate: u32) -> Self {
        Self {
            panel_buckets: Arc::new(Mutex::new(HashMap::new())),
            default_capacity,
            default_refill_rate,
        }
    }

    /// Create with standard settings (100 messages capacity, 50 messages/sec refill)
    pub fn standard() -> Self {
        Self::new(100, 50)
    }

    /// Create with high-throughput settings (500 messages capacity, 200 messages/sec refill)
    pub fn high_throughput() -> Self {
        Self::new(500, 200)
    }

    /// Check if streaming output can be sent
    pub fn allow_streaming_output(&self, panel_id: &str) -> bool {
        self.get_or_create_bucket(panel_id).try_consume(1)
    }

    /// Check if streaming complete can be sent (uses fewer tokens)
    pub fn allow_streaming_complete(&self, panel_id: &str) -> bool {
        self.get_or_create_bucket(panel_id).try_consume(1)
    }

    /// Check if batch output can be sent (uses more tokens based on batch size)
    pub fn allow_batch_output(&self, panel_id: &str, batch_size: u32) -> bool {
        self.get_or_create_bucket(panel_id).try_consume(batch_size.max(1))
    }

    /// Get current token count for a panel
    pub fn get_available_tokens(&self, panel_id: &str) -> u32 {
        self.get_or_create_bucket(panel_id).available_tokens()
    }

    /// Get or create token bucket for panel
    fn get_or_create_bucket(&self, panel_id: &str) -> TokenBucket {
        let mut buckets = self.panel_buckets.lock().unwrap();
        buckets.entry(panel_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.default_capacity, self.default_refill_rate))
            .clone()
    }

    /// Reset rate limiter (for testing)
    pub fn reset(&self) {
        let mut buckets = self.panel_buckets.lock().unwrap();
        buckets.clear();
    }
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_token_bucket_basic_consumption() {
        let bucket = TokenBucket::new(10, 5);
        
        // Should be able to consume initial tokens
        assert!(bucket.try_consume(5));
        assert_eq!(bucket.available_tokens(), 5);
        
        // Should be able to consume remaining tokens
        assert!(bucket.try_consume(5));
        assert_eq!(bucket.available_tokens(), 0);
        
        // Should fail when no tokens left
        assert!(!bucket.try_consume(1));
    }

    #[test]
    fn test_token_bucket_refill() {
        let bucket = TokenBucket::new(10, 10); // 10 tokens/sec refill
        
        // Consume all tokens
        assert!(bucket.try_consume(10));
        assert_eq!(bucket.available_tokens(), 0);
        
        // Wait for refill (allow some tolerance for timing)
        thread::sleep(Duration::from_millis(200));
        
        // Should have some tokens refilled
        let available = bucket.available_tokens();
        assert!(available > 0 && available <= 10);
    }

    #[test]
    fn test_streaming_rate_limiter_per_panel() {
        let limiter = StreamingRateLimiter::new(5, 10);
        
        // Different panels should have independent limits
        assert!(limiter.allow_streaming_output("panel1"));
        assert!(limiter.allow_streaming_output("panel2"));
        
        // Exhaust panel1 tokens
        for _ in 0..4 {
            assert!(limiter.allow_streaming_output("panel1"));
        }
        assert!(!limiter.allow_streaming_output("panel1"));
        
        // Panel2 should still work
        assert!(limiter.allow_streaming_output("panel2"));
    }

    #[test]
    fn test_streaming_rate_limiter_batch_consumption() {
        let limiter = StreamingRateLimiter::new(10, 5);
        
        // Large batch should consume multiple tokens
        assert!(limiter.allow_batch_output("panel1", 5));
        assert_eq!(limiter.get_available_tokens("panel1"), 5);
        
        // Should not allow batch larger than remaining tokens
        assert!(!limiter.allow_batch_output("panel1", 10));
        
        // Should allow smaller batch
        assert!(limiter.allow_batch_output("panel1", 3));
    }

    #[test]
    fn test_rate_limiter_standard_settings() {
        let limiter = StreamingRateLimiter::standard();
        
        // Should start with full capacity
        assert_eq!(limiter.get_available_tokens("test_panel"), 100);
        
        // Should allow normal streaming operations
        assert!(limiter.allow_streaming_output("test_panel"));
        assert!(limiter.allow_streaming_complete("test_panel"));
    }
}
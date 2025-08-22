use std::time::SystemTime;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingOutput {
    pub panel_id: String,
    pub line_content: String,
    pub sequence: u64,
    pub timestamp: SystemTime,
    pub is_stderr: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingComplete {
    pub panel_id: String,
    pub task_id: Uuid,
    pub exit_code: Option<i32>,
    pub success: bool,
    pub total_lines: u64,
    pub timestamp: SystemTime,
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
        }
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
}
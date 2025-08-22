use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use log::{debug, error, warn};

#[derive(Debug, Clone)]
pub struct OutputLine {
    pub content: String,
    pub sequence: u64,
    pub timestamp: Instant,
    pub is_stderr: bool,
}

#[derive(Debug)]
pub struct StreamingExecutor {
    sender: Sender<OutputLine>,
    receiver: Receiver<OutputLine>,
    sequence_counter: u64,
    buffer_size: usize,
    rate_limit_interval: Duration,
}

impl StreamingExecutor {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            sequence_counter: 0,
            buffer_size: 10,  // Buffer up to 10 lines before flushing
            rate_limit_interval: Duration::from_millis(16), // ~60fps rate limiting
        }
    }

    pub fn with_config(buffer_size: usize, rate_limit_ms: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            sequence_counter: 0,
            buffer_size,
            rate_limit_interval: Duration::from_millis(rate_limit_ms),
        }
    }

    pub fn spawn_streaming(&mut self, command: &str, working_dir: Option<&str>) -> Result<(Child, Receiver<OutputLine>, String), Box<dyn std::error::Error>> {
        debug!("Starting streaming execution: {}", command);
        let command_copy = command.to_string();
        
        // Create a new channel for this specific streaming task
        let (sender, receiver) = mpsc::channel();
        
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(&["/C", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(&["-c", command]);
            c
        };

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn command '{}': {}", command, e))?;

        let stdout = child.stdout.take()
            .ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take()
            .ok_or("Failed to capture stderr")?;

        let sender_clone = sender.clone();
        let mut seq_counter = self.sequence_counter;
        let buffer_size = self.buffer_size;
        let rate_limit_interval = self.rate_limit_interval;
        
        // Spawn thread for stdout with buffering and rate limiting
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut line_buffer = Vec::new();
            let mut last_flush = Instant::now();
            
            for line in reader.lines() {
                match line {
                    Ok(content) => {
                        seq_counter += 1;
                        let output_line = OutputLine {
                            content,
                            sequence: seq_counter,
                            timestamp: Instant::now(),
                            is_stderr: false,
                        };
                        line_buffer.push(output_line);
                        
                        // Flush buffer if size limit reached or rate limit interval passed
                        let should_flush = line_buffer.len() >= buffer_size || 
                                         last_flush.elapsed() >= rate_limit_interval;
                        
                        if should_flush {
                            for buffered_line in line_buffer.drain(..) {
                                if sender_clone.send(buffered_line).is_err() {
                                    warn!("Failed to send stdout line - receiver dropped");
                                    return;
                                }
                            }
                            last_flush = Instant::now();
                        }
                    }
                    Err(e) => {
                        error!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            
            // Flush remaining buffer on exit
            for buffered_line in line_buffer.drain(..) {
                if sender_clone.send(buffered_line).is_err() {
                    break;
                }
            }
        });

        let sender_clone = self.sender.clone();
        let mut seq_counter = self.sequence_counter;
        let buffer_size = self.buffer_size;
        let rate_limit_interval = self.rate_limit_interval;
        
        // Spawn thread for stderr with buffering and rate limiting
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut line_buffer = Vec::new();
            let mut last_flush = Instant::now();
            
            for line in reader.lines() {
                match line {
                    Ok(content) => {
                        seq_counter += 1;
                        let output_line = OutputLine {
                            content,
                            sequence: seq_counter,
                            timestamp: Instant::now(),
                            is_stderr: true,
                        };
                        line_buffer.push(output_line);
                        
                        // Flush buffer if size limit reached or rate limit interval passed
                        let should_flush = line_buffer.len() >= buffer_size || 
                                         last_flush.elapsed() >= rate_limit_interval;
                        
                        if should_flush {
                            for buffered_line in line_buffer.drain(..) {
                                if sender_clone.send(buffered_line).is_err() {
                                    warn!("Failed to send stderr line - receiver dropped");
                                    return;
                                }
                            }
                            last_flush = Instant::now();
                        }
                    }
                    Err(e) => {
                        error!("Error reading stderr: {}", e);
                        break;
                    }
                }
            }
            
            // Flush remaining buffer on exit
            for buffered_line in line_buffer.drain(..) {
                if sender_clone.send(buffered_line).is_err() {
                    break;
                }
            }
        });

        Ok((child, receiver, command_copy))
    }

    pub fn read_line(&self, timeout: Option<Duration>) -> Option<OutputLine> {
        if let Some(duration) = timeout {
            match self.receiver.recv_timeout(duration) {
                Ok(line) => Some(line),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => None,
            }
        } else {
            self.receiver.recv().ok()
        }
    }

    pub fn try_read_line(&self) -> Option<OutputLine> {
        self.receiver.try_recv().ok()
    }

    pub fn get_exit_status(&self, child: &mut Child) -> Option<std::process::ExitStatus> {
        child.try_wait().ok().flatten()
    }
    
    pub fn get_receiver(&self) -> &Receiver<OutputLine> {
        &self.receiver
    }
}

impl Default for StreamingExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_streaming_executor_creation() {
        let executor = StreamingExecutor::new();
        assert_eq!(executor.sequence_counter, 0);
    }

    #[test]
    fn test_simple_command_streaming() {
        let mut executor = StreamingExecutor::new();
        let (mut child, receiver, _command) = executor.spawn_streaming("echo 'test line'", None).unwrap();
        
        let mut output_lines = Vec::new();
        let start = Instant::now();
        
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(line) = receiver.try_recv() {
                output_lines.push(line);
            }
            
            if let Some(status) = executor.get_exit_status(&mut child) {
                if status.success() {
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(10));
        }
        
        assert!(!output_lines.is_empty());
        assert!(output_lines.iter().any(|line| line.content.contains("test line")));
    }

    #[test]
    fn test_multi_line_output() {
        let mut executor = StreamingExecutor::new();
        let command = if cfg!(target_os = "windows") {
            "echo line1 && echo line2 && echo line3"
        } else {
            "printf 'line1\\nline2\\nline3\\n'"
        };
        
        let (mut child, receiver, _command) = executor.spawn_streaming(command, None).unwrap();
        
        let mut output_lines = Vec::new();
        let start = Instant::now();
        
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(line) = receiver.try_recv() {
                output_lines.push(line);
            }
            
            if let Some(status) = executor.get_exit_status(&mut child) {
                if status.success() {
                    // Wait longer for remaining output to arrive
                    thread::sleep(Duration::from_millis(100));
                    // Drain all remaining messages from receiver
                    while let Ok(line) = receiver.try_recv() {
                        output_lines.push(line);
                    }
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(10));
        }
        
        assert!(output_lines.len() >= 3);
        assert!(output_lines.iter().any(|line| line.content.contains("line1")));
        assert!(output_lines.iter().any(|line| line.content.contains("line2")));
        assert!(output_lines.iter().any(|line| line.content.contains("line3")));
    }

    #[test]
    fn test_sequence_numbering() {
        let mut executor = StreamingExecutor::new();
        let (mut child, receiver, _command) = executor.spawn_streaming("echo 'test'", None).unwrap();
        
        let mut sequences = Vec::new();
        let start = Instant::now();
        
        while start.elapsed() < Duration::from_secs(2) {
            if let Ok(line) = receiver.try_recv() {
                sequences.push(line.sequence);
            }
            
            if let Some(status) = executor.get_exit_status(&mut child) {
                if status.success() {
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(10));
        }
        
        // Sequences should be incremental
        for i in 1..sequences.len() {
            assert!(sequences[i] > sequences[i-1]);
        }
    }

    #[test]
    fn test_stderr_capture() {
        let mut executor = StreamingExecutor::new();
        let command = if cfg!(target_os = "windows") {
            "echo error message 1>&2"
        } else {
            "echo 'error message' >&2"
        };
        
        let (mut child, receiver, _command) = executor.spawn_streaming(command, None).unwrap();
        
        let mut stderr_lines = Vec::new();
        let start = Instant::now();
        
        while start.elapsed() < Duration::from_secs(2) {
            if let Some(line) = executor.try_read_line() {
                if line.is_stderr {
                    stderr_lines.push(line);
                }
            }
            
            if let Some(status) = executor.get_exit_status(&mut child) {
                if !status.success() || start.elapsed() > Duration::from_millis(500) {
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(10));
        }
        
        assert!(!stderr_lines.is_empty());
        assert!(stderr_lines.iter().any(|line| line.content.contains("error message")));
    }
}
use std::collections::VecDeque;

/// Circular buffer for storing PTY output lines with configurable size limit
/// Provides efficient memory-bounded storage with O(1) append operations
#[derive(Debug, Clone)]
pub struct CircularBuffer {
    buffer: VecDeque<String>,
    max_size: usize,
    total_lines_added: u64, // Track total lines ever added for diagnostics
}

impl CircularBuffer {
    /// Create a new circular buffer with the specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
            total_lines_added: 0,
        }
    }

    /// Add a line to the buffer, removing the oldest if at capacity
    pub fn push(&mut self, line: String) {
        if self.buffer.len() >= self.max_size && self.max_size > 0 {
            self.buffer.pop_front(); // Remove oldest line
        }

        self.buffer.push_back(line);
        self.total_lines_added += 1;
    }

    /// Get the current number of lines in the buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get all lines as a vector (for display purposes)
    pub fn get_all_lines(&self) -> Vec<String> {
        self.buffer.iter().cloned().collect()
    }

    /// Get the last N lines (most recent)
    pub fn get_last_lines(&self, n: usize) -> Vec<String> {
        let start_idx = if n >= self.buffer.len() {
            0
        } else {
            self.buffer.len() - n
        };
        self.buffer.iter().skip(start_idx).cloned().collect()
    }

    /// Get lines within a range (for scrolling)
    pub fn get_lines_range(&self, start: usize, count: usize) -> Vec<String> {
        if start >= self.buffer.len() {
            return Vec::new();
        }

        let end = (start + count).min(self.buffer.len());
        self.buffer.range(start..end).cloned().collect()
    }

    /// Get all lines as a single string (joined with newlines)
    pub fn get_content(&self) -> String {
        self.buffer.iter().cloned().collect::<Vec<_>>().join("\n")
    }

    /// Get the last N lines as a single string
    pub fn get_recent_content(&self, lines: usize) -> String {
        self.get_last_lines(lines).join("\n")
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        // Don't reset total_lines_added to preserve history
    }

    /// Get maximum buffer size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Set new maximum size (may truncate existing content)
    pub fn set_max_size(&mut self, new_max_size: usize) {
        self.max_size = new_max_size;

        // Truncate if current size exceeds new limit
        while self.buffer.len() > new_max_size && new_max_size > 0 {
            self.buffer.pop_front();
        }

        // Resize capacity for efficiency
        if new_max_size > 0 {
            self.buffer.reserve(new_max_size);
        }
    }

    /// Get diagnostic information about the buffer
    pub fn get_stats(&self) -> BufferStats {
        BufferStats {
            current_lines: self.buffer.len(),
            max_size: self.max_size,
            total_lines_added: self.total_lines_added,
            memory_usage_bytes: self.estimate_memory_usage(),
            is_at_capacity: self.buffer.len() >= self.max_size,
        }
    }

    /// Estimate memory usage in bytes (approximate)
    fn estimate_memory_usage(&self) -> usize {
        let string_overhead = std::mem::size_of::<String>();
        let content_size: usize = self.buffer.iter().map(|s| s.len()).sum();
        let deque_overhead =
            std::mem::size_of::<VecDeque<String>>() + (self.buffer.capacity() * string_overhead);

        content_size + deque_overhead
    }

    /// Search for lines containing the given text (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<(usize, String)> {
        let query_lower = query.to_lowercase();
        self.buffer
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                if line.to_lowercase().contains(&query_lower) {
                    Some((idx, line.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get lines in reverse order (most recent first)
    pub fn get_lines_reverse(&self) -> Vec<String> {
        self.buffer.iter().rev().cloned().collect()
    }
}

/// Statistics about the circular buffer
#[derive(Debug, Clone)]
pub struct BufferStats {
    pub current_lines: usize,
    pub max_size: usize,
    pub total_lines_added: u64,
    pub memory_usage_bytes: usize,
    pub is_at_capacity: bool,
}

impl Default for CircularBuffer {
    /// Create a buffer with default size of 1000 lines
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_buffer_basic_operations() {
        let mut buffer = CircularBuffer::new(3);

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        buffer.push("line1".to_string());
        buffer.push("line2".to_string());
        buffer.push("line3".to_string());

        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.get_all_lines(), vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_circular_buffer_overflow() {
        let mut buffer = CircularBuffer::new(2);

        buffer.push("line1".to_string());
        buffer.push("line2".to_string());
        buffer.push("line3".to_string()); // Should remove line1

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.get_all_lines(), vec!["line2", "line3"]);

        buffer.push("line4".to_string()); // Should remove line2
        assert_eq!(buffer.get_all_lines(), vec!["line3", "line4"]);
    }

    #[test]
    fn test_circular_buffer_get_last_lines() {
        let mut buffer = CircularBuffer::new(5);

        for i in 1..=5 {
            buffer.push(format!("line{}", i));
        }

        assert_eq!(buffer.get_last_lines(2), vec!["line4", "line5"]);
        assert_eq!(
            buffer.get_last_lines(10),
            vec!["line1", "line2", "line3", "line4", "line5"]
        );
    }

    #[test]
    fn test_circular_buffer_search() {
        let mut buffer = CircularBuffer::new(5);

        buffer.push("error: file not found".to_string());
        buffer.push("info: processing data".to_string());
        buffer.push("warning: deprecated function".to_string());
        buffer.push("error: connection failed".to_string());

        let results = buffer.search("error");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // First error at index 0
        assert_eq!(results[1].0, 3); // Second error at index 3
    }

    #[test]
    fn test_circular_buffer_resize() {
        let mut buffer = CircularBuffer::new(5);

        for i in 1..=5 {
            buffer.push(format!("line{}", i));
        }

        // Resize to smaller capacity
        buffer.set_max_size(3);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_all_lines(), vec!["line3", "line4", "line5"]);
    }

    #[test]
    fn test_circular_buffer_stats() {
        let mut buffer = CircularBuffer::new(2);

        buffer.push("test1".to_string());
        buffer.push("test2".to_string());
        buffer.push("test3".to_string()); // Overflow

        let stats = buffer.get_stats();
        assert_eq!(stats.current_lines, 2);
        assert_eq!(stats.max_size, 2);
        assert_eq!(stats.total_lines_added, 3);
        assert!(stats.is_at_capacity);
        assert!(stats.memory_usage_bytes > 0);
    }

    #[test]
    fn test_circular_buffer_content_methods() {
        let mut buffer = CircularBuffer::new(3);

        buffer.push("line1".to_string());
        buffer.push("line2".to_string());
        buffer.push("line3".to_string());

        assert_eq!(buffer.get_content(), "line1\nline2\nline3");
        assert_eq!(buffer.get_recent_content(2), "line2\nline3");
    }
}

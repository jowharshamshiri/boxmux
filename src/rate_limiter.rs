use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Token bucket rate limiter for controlling output frequency
#[derive(Debug)]
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    refill_rate: u32, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    pub fn consume_or_wait(&mut self, tokens: u32) -> Option<Duration> {
        self.refill();
        
        if self.tokens >= tokens {
            self.tokens -= tokens;
            None
        } else {
            // Calculate how long to wait for tokens
            let tokens_needed = tokens - self.tokens;
            let wait_time = Duration::from_secs_f64(tokens_needed as f64 / self.refill_rate as f64);
            Some(wait_time)
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let time_passed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = (time_passed * self.refill_rate as f64) as u32;
        
        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }
}

/// Rate limiter for streaming output with configurable policies
#[derive(Debug)]
pub struct StreamingRateLimiter {
    token_bucket: TokenBucket,
    output_queue: VecDeque<(Instant, String)>,
    max_queue_size: usize,
}

impl StreamingRateLimiter {
    pub fn new(max_lines_per_second: u32, max_queue_size: usize) -> Self {
        Self {
            token_bucket: TokenBucket::new(max_lines_per_second * 2, max_lines_per_second),
            output_queue: VecDeque::new(),
            max_queue_size,
        }
    }

    pub fn should_allow_output(&mut self) -> bool {
        self.token_bucket.try_consume(1)
    }

    pub fn queue_output(&mut self, output: String) -> bool {
        if self.output_queue.len() >= self.max_queue_size {
            // Drop oldest if queue is full
            self.output_queue.pop_front();
        }
        
        self.output_queue.push_back((Instant::now(), output));
        true
    }

    pub fn get_next_output(&mut self) -> Option<String> {
        if self.should_allow_output() {
            self.output_queue.pop_front().map(|(_, output)| output)
        } else {
            None
        }
    }

    pub fn queue_size(&self) -> usize {
        self.output_queue.len()
    }

    pub fn is_queue_full(&self) -> bool {
        self.output_queue.len() >= self.max_queue_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(10, 5);
        
        // Should have initial capacity
        assert!(bucket.try_consume(5));
        assert!(bucket.try_consume(5));
        assert!(!bucket.try_consume(1)); // Should fail, no tokens left
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(5, 10); // 10 tokens per second
        
        // Consume all tokens
        assert!(bucket.try_consume(5));
        assert!(!bucket.try_consume(1));
        
        // Wait and refill should happen
        thread::sleep(Duration::from_millis(600)); // 0.6 seconds -> ~6 tokens
        assert!(bucket.try_consume(5)); // Should work after refill
    }

    #[test]
    fn test_streaming_rate_limiter() {
        let mut limiter = StreamingRateLimiter::new(5, 10); // 5 lines/sec, queue size 10
        
        // Queue some outputs
        assert!(limiter.queue_output("line1".to_string()));
        assert!(limiter.queue_output("line2".to_string()));
        
        // Should be able to get output initially
        assert!(limiter.get_next_output().is_some());
        
        // Queue size should decrease
        assert_eq!(limiter.queue_size(), 1);
    }

    #[test]
    fn test_rate_limiter_queue_overflow() {
        let mut limiter = StreamingRateLimiter::new(1, 2); // Very small queue
        
        // Fill queue
        assert!(limiter.queue_output("line1".to_string()));
        assert!(limiter.queue_output("line2".to_string()));
        
        // Adding more should drop oldest
        assert!(limiter.queue_output("line3".to_string()));
        assert_eq!(limiter.queue_size(), 2);
        
        // First line should be dropped, second should still be there
        let output = limiter.get_next_output().unwrap();
        assert_eq!(output, "line2");
    }

    #[test]
    fn test_consume_or_wait() {
        let mut bucket = TokenBucket::new(5, 10);
        
        // Consume some tokens
        assert!(bucket.consume_or_wait(3).is_none()); // Should succeed immediately
        
        // Try to consume more than available
        let wait_time = bucket.consume_or_wait(5); // Only 2 tokens left, need 5
        assert!(wait_time.is_some());
        assert!(wait_time.unwrap() > Duration::from_millis(200)); // Should need to wait
    }
}
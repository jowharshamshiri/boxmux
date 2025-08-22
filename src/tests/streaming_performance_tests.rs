use super::*;
use crate::tests::test_utils::TestDataFactory;
use crate::streaming_executor::StreamingExecutor;
use crate::streaming_panel_manager::StreamingPanelManager;
use crate::rate_limiter::{TokenBucket, StreamingRateLimiter};
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::thread;

/// Test streaming executor performance with buffering and rate limiting
#[test]
fn test_streaming_executor_performance() {
    let start_time = Instant::now();
    
    // Create executor with optimized settings
    let mut executor = StreamingExecutor::with_config(50, 10); // 50 line buffer, 10ms rate limit
    
    // Execute command that generates lots of output
    let command = if cfg!(target_os = "windows") {
        "for /L %i in (1,1,1000) do echo Line %i"
    } else {
        "for i in {1..1000}; do echo \"Line $i\"; done"
    };
    
    let (mut child, receiver) = executor.spawn_streaming(command, None).unwrap();
    
    let mut lines_received = 0;
    let mut total_latency = Duration::default();
    
    // Collect output with performance measurement
    let collection_start = Instant::now();
    while collection_start.elapsed() < Duration::from_secs(10) {
        if let Ok(output_line) = receiver.try_recv() {
            lines_received += 1;
            let latency = output_line.timestamp.elapsed();
            total_latency += latency;
        }
        
        if let Some(status) = executor.get_exit_status(&mut child) {
            if status.success() {
                // Drain remaining output
                while let Ok(_) = receiver.try_recv() {
                    lines_received += 1;
                }
                break;
            }
        }
        
        thread::sleep(Duration::from_millis(1));
    }
    
    let total_time = start_time.elapsed();
    let avg_latency = if lines_received > 0 { 
        total_latency / lines_received as u32 
    } else { 
        Duration::default() 
    };
    
    println!("Streaming performance test results:");
    println!("  Total time: {:?}", total_time);
    println!("  Lines received: {}", lines_received);
    println!("  Average latency: {:?}", avg_latency);
    println!("  Throughput: {:.2} lines/sec", lines_received as f64 / total_time.as_secs_f64());
    
    // Performance assertions
    assert!(lines_received > 0, "Should receive some output");
    assert!(total_time < Duration::from_secs(15), "Should complete within 15 seconds");
    assert!(avg_latency < Duration::from_millis(100), "Average latency should be under 100ms");
}

/// Test rate limiter performance under high load
#[test]
fn test_rate_limiter_performance() {
    let mut rate_limiter = StreamingRateLimiter::new(100, 1000); // 100 lines/sec, 1000 queue size
    let start_time = Instant::now();
    
    // Generate high-frequency output
    let mut lines_processed = 0;
    let mut lines_queued = 0;
    
    for i in 0..2000 {
        let line = format!("Test line {}", i);
        
        if rate_limiter.should_allow_output() {
            lines_processed += 1;
        } else {
            rate_limiter.queue_output(line);
            lines_queued += 1;
        }
        
        // Occasionally try to get queued output
        if i % 10 == 0 {
            while let Some(_) = rate_limiter.get_next_output() {
                lines_processed += 1;
            }
        }
    }
    
    // Process remaining queue
    while let Some(_) = rate_limiter.get_next_output() {
        lines_processed += 1;
    }
    
    let total_time = start_time.elapsed();
    
    println!("Rate limiter performance test results:");
    println!("  Total time: {:?}", total_time);
    println!("  Lines processed: {}", lines_processed);
    println!("  Lines queued: {}", lines_queued);
    println!("  Final queue size: {}", rate_limiter.queue_size());
    println!("  Processing rate: {:.2} lines/sec", lines_processed as f64 / total_time.as_secs_f64());
    
    // Performance assertions
    assert!(lines_processed > 100, "Should process significant number of lines");
    assert!(total_time < Duration::from_secs(5), "Should complete within 5 seconds");
    assert!(lines_processed + rate_limiter.queue_size() >= 1000, "Should process or queue significant number of lines");
}

/// Test streaming panel manager performance with rate limiting
#[test]
fn test_streaming_panel_manager_performance() {
    let (sender, receiver) = mpsc::channel();
    let manager = StreamingPanelManager::with_rate_limit(sender, 50, 500); // 50 lines/sec, 500 queue size
    
    // Create test panel with script that generates output
    let mut panel = TestDataFactory::create_test_panel("perf_test_panel");
    panel.script = Some(vec!["for i in {1..500}; do echo \"Performance test line $i\"; done".to_string()]);
    
    let start_time = Instant::now();
    let task_id = manager.start_streaming(&panel).unwrap();
    
    let mut messages_received = 0;
    let collection_start = Instant::now();
    
    // Collect messages with timeout
    while collection_start.elapsed() < Duration::from_secs(15) {
        if let Ok(msg) = receiver.try_recv() {
            if let crate::thread_manager::Message::PanelOutputUpdate(panel_id, _, _) = msg {
                if panel_id == "perf_test_panel" {
                    messages_received += 1;
                }
            }
        }
        
        thread::sleep(Duration::from_millis(10));
        
        // Check if task is still active
        if manager.get_active_task_count() == 0 {
            break;
        }
    }
    
    let total_time = start_time.elapsed();
    
    // Get performance statistics
    if let Some((active_tasks, queue_size, queue_utilization)) = manager.get_performance_stats() {
        println!("Streaming panel manager performance test results:");
        println!("  Total time: {:?}", total_time);
        println!("  Messages received: {}", messages_received);
        println!("  Active tasks: {}", active_tasks);
        println!("  Queue size: {}", queue_size);
        println!("  Queue utilization: {:.2}%", queue_utilization);
        println!("  Message rate: {:.2} messages/sec", messages_received as f64 / total_time.as_secs_f64());
    }
    
    // Performance assertions
    assert!(messages_received > 0, "Should receive some messages");
    assert!(total_time < Duration::from_secs(20), "Should complete within 20 seconds");
    
    // Cleanup
    manager.stop_all_tasks();
}

/// Test token bucket rate limiting accuracy
#[test]
fn test_token_bucket_accuracy() {
    let mut bucket = TokenBucket::new(10, 10); // 10 capacity, 10 tokens per second
    let start_time = Instant::now();
    
    let mut successful_consumes = 0;
    let mut failed_consumes = 0;
    
    // Try to consume tokens rapidly
    for _ in 0..50 {
        if bucket.try_consume(1) {
            successful_consumes += 1;
        } else {
            failed_consumes += 1;
        }
        thread::sleep(Duration::from_millis(50)); // 20 per second attempt rate
    }
    
    let total_time = start_time.elapsed();
    let expected_tokens = (total_time.as_secs_f64() * 10.0) as u32 + 10; // Refill rate + initial capacity
    
    println!("Token bucket accuracy test results:");
    println!("  Total time: {:?}", total_time);
    println!("  Successful consumes: {}", successful_consumes);
    println!("  Failed consumes: {}", failed_consumes);
    println!("  Expected tokens: ~{}", expected_tokens);
    println!("  Actual rate: {:.2} tokens/sec", successful_consumes as f64 / total_time.as_secs_f64());
    
    // Accuracy assertions - allow some tolerance for timing variations
    assert!(successful_consumes >= expected_tokens.saturating_sub(5), 
           "Should consume close to expected number of tokens");
    assert!(successful_consumes <= expected_tokens + 5,
           "Should not exceed token limit significantly");
}

/// Benchmark streaming executor with different configurations
#[test]
fn benchmark_streaming_configurations() {
    let test_configs = vec![
        (1, 100, "Minimal buffering"),    // 1 line buffer, 100ms rate limit
        (10, 50, "Small buffering"),     // 10 line buffer, 50ms rate limit  
        (50, 16, "Optimal buffering"),   // 50 line buffer, 16ms rate limit (~60fps)
        (100, 8, "Heavy buffering"),     // 100 line buffer, 8ms rate limit
    ];
    
    for (buffer_size, rate_limit_ms, description) in test_configs {
        let start_time = Instant::now();
        
        let mut executor = StreamingExecutor::with_config(buffer_size, rate_limit_ms);
        let command = "for i in {1..200}; do echo \"Benchmark line $i\"; done";
        let (mut child, receiver) = executor.spawn_streaming(command, None).unwrap();
        
        let mut lines_received = 0;
        while start_time.elapsed() < Duration::from_secs(10) {
            if let Ok(_) = receiver.try_recv() {
                lines_received += 1;
            }
            
            if let Some(status) = executor.get_exit_status(&mut child) {
                if status.success() {
                    while let Ok(_) = receiver.try_recv() {
                        lines_received += 1;
                    }
                    break;
                }
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        let total_time = start_time.elapsed();
        let throughput = lines_received as f64 / total_time.as_secs_f64();
        
        println!("{}: {} lines in {:?} ({:.2} lines/sec)", 
                description, lines_received, total_time, throughput);
        
        assert!(lines_received > 0, "Configuration should process some lines: {}", description);
        assert!(total_time < Duration::from_secs(15), "Configuration should complete reasonably quickly: {}", description);
    }
}
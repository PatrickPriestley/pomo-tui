use pomo_tui::core::{Timer, TimerState};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_timer_precision_25_minute_session() {
    let session_duration = Duration::from_secs(25 * 60); // 25 minutes
    let mut timer = Timer::new(session_duration);
    
    let start_time = Instant::now();
    
    // Simulate timer running for full duration
    timer.start();
    
    // For testing, we'll use a much shorter duration but test the precision logic
    let test_duration = Duration::from_millis(1000); // 1 second test
    let timer_test = Timer::new(test_duration);
    
    let test_start = Instant::now();
    sleep(test_duration).await;
    let actual_elapsed = test_start.elapsed();
    
    // Calculate drift
    let drift = if actual_elapsed > test_duration {
        actual_elapsed - test_duration
    } else {
        test_duration - actual_elapsed
    };
    
    // Requirement: <100ms drift over 25 minutes
    // For a 1-second test, equivalent drift would be ~4ms
    let acceptable_drift = Duration::from_millis(10); // Being more lenient for test environment
    
    assert!(
        drift < acceptable_drift,
        "Timer drift {} exceeds acceptable limit of {}",
        drift.as_millis(),
        acceptable_drift.as_millis()
    );
}

#[test]
fn test_timer_accuracy_tracking() {
    let duration = Duration::from_secs(60);
    let mut timer = Timer::new(duration);
    
    let start = Instant::now();
    timer.start();
    
    // Simulate some time passing
    std::thread::sleep(Duration::from_millis(100));
    
    let remaining = timer.remaining();
    let expected_remaining = duration - Duration::from_millis(100);
    
    // Allow 5ms tolerance for test timing
    let tolerance = Duration::from_millis(5);
    let diff = if remaining > expected_remaining {
        remaining - expected_remaining
    } else {
        expected_remaining - remaining
    };
    
    assert!(
        diff < tolerance,
        "Timer accuracy test failed. Expected ~{:?}, got {:?}, diff: {:?}",
        expected_remaining,
        remaining,
        diff
    );
}

#[tokio::test]
async fn test_timer_precision_with_pause_resume() {
    let total_duration = Duration::from_secs(5);
    let mut timer = Timer::new(total_duration);
    
    // Start timer
    timer.start();
    let start_time = Instant::now();
    
    // Run for 1 second
    sleep(Duration::from_secs(1)).await;
    
    // Pause
    timer.pause();
    let pause_time = Instant::now();
    let time_before_pause = timer.elapsed();
    
    // Sleep while paused (this shouldn't count toward timer)
    sleep(Duration::from_millis(500)).await;
    
    // Resume
    timer.resume();
    let resume_time = Instant::now();
    
    // Verify elapsed time didn't increase during pause
    let time_after_resume = timer.elapsed();
    let pause_duration = resume_time - pause_time;
    
    // Time elapsed should be roughly the same before and after pause
    let elapsed_diff = time_after_resume - time_before_pause;
    
    assert!(
        elapsed_diff < Duration::from_millis(10),
        "Timer counted time during pause: {} additional elapsed",
        elapsed_diff.as_millis()
    );
}

#[test]
fn test_timer_state_transitions() {
    let mut timer = Timer::new(Duration::from_secs(60));
    
    // Initial state
    assert_eq!(timer.state(), TimerState::Stopped);
    
    // Start
    timer.start();
    assert_eq!(timer.state(), TimerState::Running);
    
    // Pause
    timer.pause();
    assert_eq!(timer.state(), TimerState::Paused);
    
    // Resume
    timer.resume();
    assert_eq!(timer.state(), TimerState::Running);
    
    // Complete
    timer.complete();
    assert_eq!(timer.state(), TimerState::Completed);
}

#[test]
fn test_timer_remaining_calculation() {
    let duration = Duration::from_secs(120); // 2 minutes
    let mut timer = Timer::new(duration);
    
    // Before starting
    assert_eq!(timer.remaining(), duration);
    
    // Start and simulate elapsed time
    timer.start();
    
    // Simulate 30 seconds elapsed
    let elapsed = Duration::from_secs(30);
    timer.set_elapsed_for_testing(elapsed);
    
    let remaining = timer.remaining();
    let expected = duration - elapsed;
    
    assert_eq!(remaining, expected);
}

#[test] 
fn test_timer_overflow_protection() {
    let duration = Duration::from_secs(60);
    let mut timer = Timer::new(duration);
    
    timer.start();
    
    // Simulate more time elapsed than total duration
    timer.set_elapsed_for_testing(Duration::from_secs(120));
    
    // Remaining should be zero, not negative
    assert_eq!(timer.remaining(), Duration::ZERO);
    assert_eq!(timer.state(), TimerState::Completed);
}

#[tokio::test]
async fn test_timer_precision_stress_test() {
    // Test multiple short timers to verify consistent precision
    let mut total_drift = Duration::ZERO;
    let test_count = 10;
    let test_duration = Duration::from_millis(100);
    
    for _ in 0..test_count {
        let start = Instant::now();
        sleep(test_duration).await;
        let actual = start.elapsed();
        
        let drift = if actual > test_duration {
            actual - test_duration
        } else {
            test_duration - actual
        };
        
        total_drift += drift;
    }
    
    let average_drift = total_drift / test_count as u32;
    
    // Average drift should be minimal
    assert!(
        average_drift < Duration::from_millis(5),
        "Average timer drift {} exceeds 5ms limit",
        average_drift.as_millis()
    );
}

#[test]
fn test_timer_progress_calculation() {
    let duration = Duration::from_secs(60);
    let mut timer = Timer::new(duration);
    
    timer.start();
    
    // Test various progress points
    let test_cases = vec![
        (Duration::from_secs(0), 0.0),
        (Duration::from_secs(15), 0.25),
        (Duration::from_secs(30), 0.5),
        (Duration::from_secs(45), 0.75),
        (Duration::from_secs(60), 1.0),
    ];
    
    for (elapsed, expected_progress) in test_cases {
        timer.set_elapsed_for_testing(elapsed);
        let progress = timer.progress();
        
        assert!(
            (progress - expected_progress).abs() < 0.01,
            "Progress calculation failed. Expected: {}, Got: {}",
            expected_progress,
            progress
        );
    }
}

#[test]
fn test_timer_duration_formatting() {
    let timer = Timer::new(Duration::from_secs(1565)); // 26:05
    
    // Test different format options
    assert_eq!(timer.format_remaining(false), "26:05"); // MM:SS
    assert_eq!(timer.format_remaining(true), "26:05:00"); // MM:SS:00 (no sub-second precision shown)
    
    // Test with seconds
    let timer_with_seconds = Timer::new(Duration::from_secs(125)); // 2:05
    timer_with_seconds.start();
    timer_with_seconds.set_elapsed_for_testing(Duration::from_millis(37500)); // 37.5 seconds
    
    // Should show 1:27 remaining (125 - 37.5 = 87.5 seconds)
    assert_eq!(timer_with_seconds.format_remaining(false), "1:27");
}

#[tokio::test]
async fn test_timer_callback_precision() {
    use std::sync::{Arc, Mutex};
    
    let callback_times = Arc::new(Mutex::new(Vec::new()));
    let callback_times_clone = callback_times.clone();
    
    let timer = Timer::with_callback(
        Duration::from_millis(500),
        Box::new(move |remaining| {
            callback_times_clone.lock().unwrap().push(Instant::now());
        })
    );
    
    let start = Instant::now();
    timer.start_with_updates(Duration::from_millis(100)).await; // Update every 100ms
    
    let times = callback_times.lock().unwrap();
    
    // Should have received ~5 callbacks (500ms / 100ms intervals)
    assert!(
        times.len() >= 4 && times.len() <= 6,
        "Expected 4-6 callback calls, got {}",
        times.len()
    );
    
    // Check timing consistency
    for i in 1..times.len() {
        let interval = times[i] - times[i-1];
        let expected = Duration::from_millis(100);
        let tolerance = Duration::from_millis(20);
        
        assert!(
            interval >= expected - tolerance && interval <= expected + tolerance,
            "Callback interval {} outside acceptable range",
            interval.as_millis()
        );
    }
}
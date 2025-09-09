/// Integration tests for break scheduling logic
/// These tests verify the automatic break scheduling system:
/// - Short break (5min) after each session
/// - Long break (15-30min) after 4 consecutive sessions  
/// - Break type determination based on session count
/// - Break scheduling persistence and state management
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum BreakType {
    Short,
    Long,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakSchedule {
    pub break_type: BreakType,
    pub duration: Duration,
    pub after_session_id: u64,
}

#[derive(Debug)]
pub struct BreakScheduler {
    consecutive_sessions: u32,
    sessions_before_long_break: u32,
    short_break_duration: Duration,
    long_break_duration: Duration,
}

impl Default for BreakScheduler {
    fn default() -> Self {
        Self {
            consecutive_sessions: 0,
            sessions_before_long_break: 4,
            short_break_duration: Duration::from_secs(5 * 60), // 5 minutes
            long_break_duration: Duration::from_secs(15 * 60), // 15 minutes
        }
    }
}

impl BreakScheduler {
    pub fn new(
        sessions_before_long_break: u32,
        short_duration: Duration,
        long_duration: Duration,
    ) -> Self {
        Self {
            consecutive_sessions: 0,
            sessions_before_long_break,
            short_break_duration: short_duration,
            long_break_duration: long_duration,
        }
    }

    pub fn schedule_break(&mut self, session_id: u64) -> BreakSchedule {
        self.consecutive_sessions += 1;

        if self.consecutive_sessions >= self.sessions_before_long_break {
            // Reset counter after long break
            self.consecutive_sessions = 0;
            BreakSchedule {
                break_type: BreakType::Long,
                duration: self.long_break_duration,
                after_session_id: session_id,
            }
        } else {
            BreakSchedule {
                break_type: BreakType::Short,
                duration: self.short_break_duration,
                after_session_id: session_id,
            }
        }
    }

    pub fn reset_streak(&mut self) {
        self.consecutive_sessions = 0;
    }

    pub fn current_streak(&self) -> u32 {
        self.consecutive_sessions
    }

    pub fn next_break_type(&self) -> BreakType {
        if self.consecutive_sessions + 1 >= self.sessions_before_long_break {
            BreakType::Long
        } else {
            BreakType::Short
        }
    }
}

#[test]
fn test_break_scheduler_creation() {
    let scheduler = BreakScheduler::default();
    assert_eq!(scheduler.consecutive_sessions, 0);
    assert_eq!(scheduler.sessions_before_long_break, 4);
    assert_eq!(scheduler.short_break_duration, Duration::from_secs(300));
    assert_eq!(scheduler.long_break_duration, Duration::from_secs(900));
}

#[test]
fn test_break_scheduler_custom_settings() {
    let scheduler = BreakScheduler::new(
        3,                         // Long break after 3 sessions
        Duration::from_secs(600),  // 10 min short breaks
        Duration::from_secs(1800), // 30 min long breaks
    );

    assert_eq!(scheduler.sessions_before_long_break, 3);
    assert_eq!(scheduler.short_break_duration, Duration::from_secs(600));
    assert_eq!(scheduler.long_break_duration, Duration::from_secs(1800));
}

#[test]
fn test_first_three_sessions_schedule_short_breaks() {
    let mut scheduler = BreakScheduler::default();

    // First session
    let break1 = scheduler.schedule_break(1);
    assert_eq!(break1.break_type, BreakType::Short);
    assert_eq!(break1.duration, Duration::from_secs(300));
    assert_eq!(break1.after_session_id, 1);

    // Second session
    let break2 = scheduler.schedule_break(2);
    assert_eq!(break2.break_type, BreakType::Short);
    assert_eq!(break2.after_session_id, 2);

    // Third session
    let break3 = scheduler.schedule_break(3);
    assert_eq!(break3.break_type, BreakType::Short);
    assert_eq!(break3.after_session_id, 3);
}

#[test]
fn test_fourth_session_schedules_long_break() {
    let mut scheduler = BreakScheduler::default();

    // Complete first three sessions
    scheduler.schedule_break(1);
    scheduler.schedule_break(2);
    scheduler.schedule_break(3);

    // Fourth session should trigger long break
    let break4 = scheduler.schedule_break(4);
    assert_eq!(break4.break_type, BreakType::Long);
    assert_eq!(break4.duration, Duration::from_secs(900));
    assert_eq!(break4.after_session_id, 4);
}

#[test]
fn test_cycle_repeats_after_long_break() {
    let mut scheduler = BreakScheduler::default();

    // Complete full cycle (4 sessions)
    scheduler.schedule_break(1);
    scheduler.schedule_break(2);
    scheduler.schedule_break(3);
    let long_break = scheduler.schedule_break(4);
    assert_eq!(long_break.break_type, BreakType::Long);

    // Next sessions should start new cycle with short breaks
    let break5 = scheduler.schedule_break(5);
    assert_eq!(break5.break_type, BreakType::Short);

    let break6 = scheduler.schedule_break(6);
    assert_eq!(break6.break_type, BreakType::Short);

    let break7 = scheduler.schedule_break(7);
    assert_eq!(break7.break_type, BreakType::Short);

    // Eighth session (4th in new cycle) should trigger long break again
    let break8 = scheduler.schedule_break(8);
    assert_eq!(break8.break_type, BreakType::Long);
}

#[test]
fn test_streak_tracking() {
    let mut scheduler = BreakScheduler::default();

    assert_eq!(scheduler.current_streak(), 0);

    scheduler.schedule_break(1);
    assert_eq!(scheduler.current_streak(), 1);

    scheduler.schedule_break(2);
    assert_eq!(scheduler.current_streak(), 2);

    scheduler.schedule_break(3);
    assert_eq!(scheduler.current_streak(), 3);

    // Long break resets streak
    scheduler.schedule_break(4);
    assert_eq!(scheduler.current_streak(), 0);
}

#[test]
fn test_streak_reset() {
    let mut scheduler = BreakScheduler::default();

    // Build up streak
    scheduler.schedule_break(1);
    scheduler.schedule_break(2);
    assert_eq!(scheduler.current_streak(), 2);

    // Manual reset
    scheduler.reset_streak();
    assert_eq!(scheduler.current_streak(), 0);

    // Next session should be back to beginning of cycle
    let break_after_reset = scheduler.schedule_break(3);
    assert_eq!(break_after_reset.break_type, BreakType::Short);
    assert_eq!(scheduler.current_streak(), 1);
}

#[test]
fn test_next_break_type_prediction() {
    let mut scheduler = BreakScheduler::default();

    // Predict next break types
    assert_eq!(scheduler.next_break_type(), BreakType::Short);

    scheduler.schedule_break(1);
    assert_eq!(scheduler.next_break_type(), BreakType::Short);

    scheduler.schedule_break(2);
    assert_eq!(scheduler.next_break_type(), BreakType::Short);

    scheduler.schedule_break(3);
    assert_eq!(scheduler.next_break_type(), BreakType::Long);

    scheduler.schedule_break(4); // Long break scheduled
    assert_eq!(scheduler.next_break_type(), BreakType::Short); // Cycle reset
}

#[test]
fn test_custom_break_cycle_length() {
    let mut scheduler = BreakScheduler::new(
        2, // Long break after only 2 sessions
        Duration::from_secs(300),
        Duration::from_secs(900),
    );

    // First session
    let break1 = scheduler.schedule_break(1);
    assert_eq!(break1.break_type, BreakType::Short);

    // Second session should trigger long break
    let break2 = scheduler.schedule_break(2);
    assert_eq!(break2.break_type, BreakType::Long);

    // Cycle should reset
    let break3 = scheduler.schedule_break(3);
    assert_eq!(break3.break_type, BreakType::Short);

    let break4 = scheduler.schedule_break(4);
    assert_eq!(break4.break_type, BreakType::Long);
}

#[test]
fn test_break_durations_are_correct() {
    let mut scheduler = BreakScheduler::default();

    let short_break = scheduler.schedule_break(1);
    assert_eq!(short_break.duration, Duration::from_secs(5 * 60)); // 5 minutes

    // Skip to long break
    scheduler.schedule_break(2);
    scheduler.schedule_break(3);
    let long_break = scheduler.schedule_break(4);
    assert_eq!(long_break.duration, Duration::from_secs(15 * 60)); // 15 minutes
}

#[test]
fn test_custom_break_durations() {
    let short_duration = Duration::from_secs(10 * 60); // 10 minutes
    let long_duration = Duration::from_secs(30 * 60); // 30 minutes

    let mut scheduler = BreakScheduler::new(4, short_duration, long_duration);

    let short_break = scheduler.schedule_break(1);
    assert_eq!(short_break.duration, short_duration);

    // Skip to long break
    scheduler.schedule_break(2);
    scheduler.schedule_break(3);
    let long_break = scheduler.schedule_break(4);
    assert_eq!(long_break.duration, long_duration);
}

#[test]
fn test_session_id_tracking() {
    let mut scheduler = BreakScheduler::default();

    let break1 = scheduler.schedule_break(101);
    assert_eq!(break1.after_session_id, 101);

    let break2 = scheduler.schedule_break(205);
    assert_eq!(break2.after_session_id, 205);

    // Session IDs can be non-sequential
    let break3 = scheduler.schedule_break(150);
    assert_eq!(break3.after_session_id, 150);
}

#[test]
fn test_multiple_schedulers_independent() {
    let mut scheduler1 = BreakScheduler::default();
    let mut scheduler2 = BreakScheduler::default();

    // Different scheduling patterns
    scheduler1.schedule_break(1);
    scheduler1.schedule_break(2);

    scheduler2.schedule_break(10);

    // Should have different streak counts
    assert_eq!(scheduler1.current_streak(), 2);
    assert_eq!(scheduler2.current_streak(), 1);

    // Different next break predictions
    assert_eq!(scheduler1.next_break_type(), BreakType::Short);
    assert_eq!(scheduler2.next_break_type(), BreakType::Short);

    // Complete cycle on scheduler1
    scheduler1.schedule_break(3);
    let long_break = scheduler1.schedule_break(4);
    assert_eq!(long_break.break_type, BreakType::Long);
    assert_eq!(scheduler1.current_streak(), 0);

    // scheduler2 should be unaffected
    assert_eq!(scheduler2.current_streak(), 1);
}

#[test]
fn test_edge_case_single_session_cycle() {
    let mut scheduler = BreakScheduler::new(
        1, // Long break after every session
        Duration::from_secs(300),
        Duration::from_secs(900),
    );

    // Every session should trigger long break
    let break1 = scheduler.schedule_break(1);
    assert_eq!(break1.break_type, BreakType::Long);
    assert_eq!(scheduler.current_streak(), 0);

    let break2 = scheduler.schedule_break(2);
    assert_eq!(break2.break_type, BreakType::Long);
    assert_eq!(scheduler.current_streak(), 0);
}

#[test]
fn test_large_cycle_length() {
    let mut scheduler = BreakScheduler::new(
        10, // Long break after 10 sessions
        Duration::from_secs(300),
        Duration::from_secs(900),
    );

    // Schedule 9 sessions - all should be short breaks
    for i in 1u64..=9 {
        let scheduled_break = scheduler.schedule_break(i);
        assert_eq!(scheduled_break.break_type, BreakType::Short);
        assert_eq!(scheduler.current_streak(), i as u32);
    }

    // 10th session should trigger long break
    let break10 = scheduler.schedule_break(10);
    assert_eq!(break10.break_type, BreakType::Long);
    assert_eq!(scheduler.current_streak(), 0);

    // Next session starts new cycle
    let break11 = scheduler.schedule_break(11);
    assert_eq!(break11.break_type, BreakType::Short);
    assert_eq!(scheduler.current_streak(), 1);
}

#[test]
fn test_break_scheduling_with_interruptions() {
    // Simulate real-world scenario where user might interrupt cycles
    let mut scheduler = BreakScheduler::default();

    // Normal start
    scheduler.schedule_break(1);
    scheduler.schedule_break(2);
    assert_eq!(scheduler.current_streak(), 2);

    // User takes extended break or interruption - reset streak
    scheduler.reset_streak();

    // Resume with fresh cycle
    let break_after_reset = scheduler.schedule_break(3);
    assert_eq!(break_after_reset.break_type, BreakType::Short);
    assert_eq!(scheduler.current_streak(), 1);

    // Continue normally
    scheduler.schedule_break(4);
    scheduler.schedule_break(5);
    let long_break = scheduler.schedule_break(6);
    assert_eq!(long_break.break_type, BreakType::Long);
}

#[test]
fn test_break_schedule_clone() {
    let schedule = BreakSchedule {
        break_type: BreakType::Short,
        duration: Duration::from_secs(300),
        after_session_id: 42,
    };

    let cloned = schedule.clone();
    assert_eq!(schedule.break_type, cloned.break_type);
    assert_eq!(schedule.duration, cloned.duration);
    assert_eq!(schedule.after_session_id, cloned.after_session_id);
}

#[test]
fn test_break_type_equality() {
    assert_eq!(BreakType::Short, BreakType::Short);
    assert_eq!(BreakType::Long, BreakType::Long);
    assert_ne!(BreakType::Short, BreakType::Long);
}

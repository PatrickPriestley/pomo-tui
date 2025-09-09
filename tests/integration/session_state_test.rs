/// Integration tests for session state machine
/// These tests verify state transitions work correctly:
/// - active → paused → active → completed
/// - active → abandoned  
/// - Invalid transitions are rejected
/// - State persistence across operations

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Active,
    Paused,
    Completed,
    Abandoned,
}

#[derive(Debug)]
pub enum SessionEvent {
    Start,
    Pause,
    Resume,
    Complete,
    Abandon,
}

#[derive(Debug)]
pub struct SessionStateMachine {
    current_state: SessionState,
    session_id: u64,
    start_time: std::time::Instant,
    pause_time: Option<std::time::Instant>,
    total_pause_duration: std::time::Duration,
}

impl SessionStateMachine {
    pub fn new(session_id: u64) -> Self {
        Self {
            current_state: SessionState::Active,
            session_id,
            start_time: std::time::Instant::now(),
            pause_time: None,
            total_pause_duration: std::time::Duration::ZERO,
        }
    }

    pub fn current_state(&self) -> &SessionState {
        &self.current_state
    }

    pub fn transition(&mut self, event: SessionEvent) -> Result<(), String> {
        let new_state = match (&self.current_state, event) {
            (SessionState::Active, SessionEvent::Pause) => {
                self.pause_time = Some(std::time::Instant::now());
                SessionState::Paused
            }
            (SessionState::Paused, SessionEvent::Resume) => {
                if let Some(pause_start) = self.pause_time {
                    self.total_pause_duration += pause_start.elapsed();
                }
                self.pause_time = None;
                SessionState::Active
            }
            (SessionState::Active, SessionEvent::Complete) => SessionState::Completed,
            (SessionState::Active, SessionEvent::Abandon) => SessionState::Abandoned,
            (SessionState::Paused, SessionEvent::Complete) => {
                if let Some(pause_start) = self.pause_time {
                    self.total_pause_duration += pause_start.elapsed();
                }
                self.pause_time = None;
                SessionState::Completed
            }
            (SessionState::Paused, SessionEvent::Abandon) => {
                if let Some(pause_start) = self.pause_time {
                    self.total_pause_duration += pause_start.elapsed();
                }
                self.pause_time = None;
                SessionState::Abandoned
            }
            // Invalid transitions
            (SessionState::Completed, _) => {
                return Err("Cannot transition from completed state".to_string())
            }
            (SessionState::Abandoned, _) => {
                return Err("Cannot transition from abandoned state".to_string())
            }
            (SessionState::Active, SessionEvent::Resume) => {
                return Err("Cannot resume active session".to_string())
            }
            (SessionState::Paused, SessionEvent::Pause) => {
                return Err("Cannot pause already paused session".to_string())
            }
            (_, SessionEvent::Start) => return Err("Session already started".to_string()),
        };

        self.current_state = new_state;
        Ok(())
    }

    pub fn active_duration(&self) -> std::time::Duration {
        let total_elapsed = self.start_time.elapsed();
        let current_pause = if let Some(pause_start) = self.pause_time {
            pause_start.elapsed()
        } else {
            std::time::Duration::ZERO
        };

        total_elapsed - self.total_pause_duration - current_pause
    }
}

#[test]
fn test_session_state_machine_creation() {
    // Test initial state
    let session = SessionStateMachine::new(1);
    assert_eq!(*session.current_state(), SessionState::Active);
    assert_eq!(session.session_id, 1);
}

#[test]
fn test_valid_state_transitions() {
    let mut session = SessionStateMachine::new(1);

    // Test: Active → Paused
    assert_eq!(*session.current_state(), SessionState::Active);
    assert!(session.transition(SessionEvent::Pause).is_ok());
    assert_eq!(*session.current_state(), SessionState::Paused);

    // Test: Paused → Active
    assert!(session.transition(SessionEvent::Resume).is_ok());
    assert_eq!(*session.current_state(), SessionState::Active);

    // Test: Active → Completed
    assert!(session.transition(SessionEvent::Complete).is_ok());
    assert_eq!(*session.current_state(), SessionState::Completed);
}

#[test]
fn test_abandon_from_active() {
    let mut session = SessionStateMachine::new(1);

    // Test: Active → Abandoned
    assert_eq!(*session.current_state(), SessionState::Active);
    assert!(session.transition(SessionEvent::Abandon).is_ok());
    assert_eq!(*session.current_state(), SessionState::Abandoned);
}

#[test]
fn test_abandon_from_paused() {
    let mut session = SessionStateMachine::new(1);

    // Pause first
    assert!(session.transition(SessionEvent::Pause).is_ok());
    assert_eq!(*session.current_state(), SessionState::Paused);

    // Test: Paused → Abandoned
    assert!(session.transition(SessionEvent::Abandon).is_ok());
    assert_eq!(*session.current_state(), SessionState::Abandoned);
}

#[test]
fn test_complete_from_paused() {
    let mut session = SessionStateMachine::new(1);

    // Pause first
    assert!(session.transition(SessionEvent::Pause).is_ok());
    assert_eq!(*session.current_state(), SessionState::Paused);

    // Test: Paused → Completed
    assert!(session.transition(SessionEvent::Complete).is_ok());
    assert_eq!(*session.current_state(), SessionState::Completed);
}

#[test]
fn test_invalid_transitions() {
    let mut session = SessionStateMachine::new(1);

    // Cannot resume active session
    assert!(session.transition(SessionEvent::Resume).is_err());

    // Pause then test invalid transitions
    assert!(session.transition(SessionEvent::Pause).is_ok());

    // Cannot pause already paused session
    assert!(session.transition(SessionEvent::Pause).is_err());

    // Complete the session
    assert!(session.transition(SessionEvent::Complete).is_ok());

    // Cannot transition from completed state
    assert!(session.transition(SessionEvent::Pause).is_err());
    assert!(session.transition(SessionEvent::Resume).is_err());
    assert!(session.transition(SessionEvent::Complete).is_err());
    assert!(session.transition(SessionEvent::Abandon).is_err());
}

#[test]
fn test_invalid_transitions_from_abandoned() {
    let mut session = SessionStateMachine::new(1);

    // Abandon the session
    assert!(session.transition(SessionEvent::Abandon).is_ok());

    // Cannot transition from abandoned state
    assert!(session.transition(SessionEvent::Pause).is_err());
    assert!(session.transition(SessionEvent::Resume).is_err());
    assert!(session.transition(SessionEvent::Complete).is_err());
    assert!(session.transition(SessionEvent::Abandon).is_err());
}

#[test]
fn test_session_timing_active_duration() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut session = SessionStateMachine::new(1);

    // Run for short time
    sleep(Duration::from_millis(100));
    let duration1 = session.active_duration();

    // Should be approximately 100ms
    assert!(duration1 >= Duration::from_millis(90));
    assert!(duration1 <= Duration::from_millis(150));
}

#[test]
fn test_session_timing_with_pause() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut session = SessionStateMachine::new(1);

    // Run actively for 100ms
    sleep(Duration::from_millis(100));

    // Pause
    assert!(session.transition(SessionEvent::Pause).is_ok());

    // Sleep while paused (should not count as active time)
    sleep(Duration::from_millis(200));

    // Resume
    assert!(session.transition(SessionEvent::Resume).is_ok());

    // Run actively for another 100ms
    sleep(Duration::from_millis(100));

    let active_duration = session.active_duration();

    // Should be approximately 200ms (paused time excluded)
    assert!(active_duration >= Duration::from_millis(180));
    assert!(active_duration <= Duration::from_millis(250));
}

#[test]
fn test_multiple_pause_resume_cycles() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut session = SessionStateMachine::new(1);

    // First active period
    sleep(Duration::from_millis(50));

    // First pause/resume cycle
    assert!(session.transition(SessionEvent::Pause).is_ok());
    sleep(Duration::from_millis(100)); // Paused time
    assert!(session.transition(SessionEvent::Resume).is_ok());

    // Second active period
    sleep(Duration::from_millis(50));

    // Second pause/resume cycle
    assert!(session.transition(SessionEvent::Pause).is_ok());
    sleep(Duration::from_millis(100)); // Paused time
    assert!(session.transition(SessionEvent::Resume).is_ok());

    // Final active period
    sleep(Duration::from_millis(50));

    let active_duration = session.active_duration();

    // Should be approximately 150ms (3 x 50ms active periods)
    assert!(active_duration >= Duration::from_millis(130));
    assert!(active_duration <= Duration::from_millis(200));
}

#[test]
fn test_session_state_machine_clone() {
    let session1 = SessionStateMachine::new(1);
    let session2 = SessionStateMachine::new(2);

    assert_eq!(session1.session_id, 1);
    assert_eq!(session2.session_id, 2);
    assert_eq!(*session1.current_state(), *session2.current_state());
}

#[test]
fn test_complex_state_transition_sequence() {
    let mut session = SessionStateMachine::new(1);

    // Complex sequence: Active → Paused → Active → Paused → Active → Completed
    let transitions = vec![
        SessionEvent::Pause,
        SessionEvent::Resume,
        SessionEvent::Pause,
        SessionEvent::Resume,
        SessionEvent::Complete,
    ];

    let expected_states = vec![
        SessionState::Paused,
        SessionState::Active,
        SessionState::Paused,
        SessionState::Active,
        SessionState::Completed,
    ];

    for (event, expected_state) in transitions.into_iter().zip(expected_states.into_iter()) {
        assert!(session.transition(event).is_ok());
        assert_eq!(*session.current_state(), expected_state);
    }
}

#[test]
fn test_session_state_error_messages() {
    let mut session = SessionStateMachine::new(1);

    // Test specific error messages
    let result = session.transition(SessionEvent::Resume);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot resume active session");

    // Pause and test double pause
    assert!(session.transition(SessionEvent::Pause).is_ok());
    let result = session.transition(SessionEvent::Pause);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot pause already paused session");

    // Complete and test transitions from completed state
    assert!(session.transition(SessionEvent::Complete).is_ok());
    let result = session.transition(SessionEvent::Pause);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Cannot transition from completed state"
    );
}

#[test]
fn test_concurrent_sessions() {
    // Test that multiple sessions can exist independently
    let mut session1 = SessionStateMachine::new(1);
    let mut session2 = SessionStateMachine::new(2);

    // Different state transitions for each session
    assert!(session1.transition(SessionEvent::Pause).is_ok());
    assert!(session2.transition(SessionEvent::Complete).is_ok());

    assert_eq!(*session1.current_state(), SessionState::Paused);
    assert_eq!(*session2.current_state(), SessionState::Completed);

    // session1 can still transition
    assert!(session1.transition(SessionEvent::Resume).is_ok());
    assert_eq!(*session1.current_state(), SessionState::Active);

    // session2 cannot transition (completed)
    assert!(session2.transition(SessionEvent::Pause).is_err());
}

#[test]
fn test_session_timing_precision() {
    use std::thread::sleep;
    use std::time::Duration;

    let mut session = SessionStateMachine::new(1);

    // Very short active period
    sleep(Duration::from_millis(10));
    let duration1 = session.active_duration();

    // Pause for longer period
    assert!(session.transition(SessionEvent::Pause).is_ok());
    sleep(Duration::from_millis(100));

    // Resume for another short period
    assert!(session.transition(SessionEvent::Resume).is_ok());
    sleep(Duration::from_millis(10));

    let final_duration = session.active_duration();

    // Should be approximately 20ms total active time
    assert!(final_duration >= Duration::from_millis(15));
    assert!(final_duration <= Duration::from_millis(40));

    // Final duration should be greater than first measurement
    assert!(final_duration > duration1);
}

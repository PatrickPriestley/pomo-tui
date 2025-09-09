use pomo_tui::core::{SessionStateMachine, SessionState, SessionEvent};
use pomo_tui::database::models::Session;
use std::time::Duration;

#[test]
fn test_session_state_machine_valid_transitions() {
    let mut state_machine = SessionStateMachine::new();
    
    // Test complete valid flow: Idle -> Active -> Paused -> Active -> Completed
    assert_eq!(state_machine.current_state(), SessionState::Idle);
    
    // Start session
    assert!(state_machine.transition(SessionEvent::Start).is_ok());
    assert_eq!(state_machine.current_state(), SessionState::Active);
    
    // Pause session
    assert!(state_machine.transition(SessionEvent::Pause).is_ok());
    assert_eq!(state_machine.current_state(), SessionState::Paused);
    
    // Resume session
    assert!(state_machine.transition(SessionEvent::Resume).is_ok());
    assert_eq!(state_machine.current_state(), SessionState::Active);
    
    // Complete session
    assert!(state_machine.transition(SessionEvent::Complete).is_ok());
    assert_eq!(state_machine.current_state(), SessionState::Completed);
}

#[test]
fn test_session_state_machine_abandon_from_active() {
    let mut state_machine = SessionStateMachine::new();
    
    // Start session
    state_machine.transition(SessionEvent::Start).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Active);
    
    // Abandon session
    assert!(state_machine.transition(SessionEvent::Abandon).is_ok());
    assert_eq!(state_machine.current_state(), SessionState::Abandoned);
}

#[test]
fn test_session_state_machine_abandon_from_paused() {
    let mut state_machine = SessionStateMachine::new();
    
    // Start and pause session
    state_machine.transition(SessionEvent::Start).unwrap();
    state_machine.transition(SessionEvent::Pause).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Paused);
    
    // Abandon from paused state
    assert!(state_machine.transition(SessionEvent::Abandon).is_ok());
    assert_eq!(state_machine.current_state(), SessionState::Abandoned);
}

#[test]
fn test_session_state_machine_invalid_transitions() {
    let mut state_machine = SessionStateMachine::new();
    
    // Cannot pause before starting
    assert!(state_machine.transition(SessionEvent::Pause).is_err());
    assert_eq!(state_machine.current_state(), SessionState::Idle);
    
    // Cannot resume before starting
    assert!(state_machine.transition(SessionEvent::Resume).is_err());
    assert_eq!(state_machine.current_state(), SessionState::Idle);
    
    // Cannot complete before starting
    assert!(state_machine.transition(SessionEvent::Complete).is_err());
    assert_eq!(state_machine.current_state(), SessionState::Idle);
    
    // Cannot abandon before starting
    assert!(state_machine.transition(SessionEvent::Abandon).is_err());
    assert_eq!(state_machine.current_state(), SessionState::Idle);
}

#[test]
fn test_session_state_machine_resume_without_pause() {
    let mut state_machine = SessionStateMachine::new();
    
    state_machine.transition(SessionEvent::Start).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Active);
    
    // Cannot resume when already active
    assert!(state_machine.transition(SessionEvent::Resume).is_err());
    assert_eq!(state_machine.current_state(), SessionState::Active);
}

#[test]
fn test_session_state_machine_double_pause() {
    let mut state_machine = SessionStateMachine::new();
    
    state_machine.transition(SessionEvent::Start).unwrap();
    state_machine.transition(SessionEvent::Pause).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Paused);
    
    // Cannot pause when already paused
    assert!(state_machine.transition(SessionEvent::Pause).is_err());
    assert_eq!(state_machine.current_state(), SessionState::Paused);
}

#[test]
fn test_session_state_machine_terminal_states() {
    let mut state_machine = SessionStateMachine::new();
    
    // Test completed state is terminal
    state_machine.transition(SessionEvent::Start).unwrap();
    state_machine.transition(SessionEvent::Complete).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Completed);
    
    // No further transitions allowed from completed
    assert!(state_machine.transition(SessionEvent::Start).is_err());
    assert!(state_machine.transition(SessionEvent::Pause).is_err());
    assert!(state_machine.transition(SessionEvent::Resume).is_err());
    assert!(state_machine.transition(SessionEvent::Abandon).is_err());
    assert!(state_machine.transition(SessionEvent::Complete).is_err());
}

#[test]
fn test_session_state_machine_abandoned_is_terminal() {
    let mut state_machine = SessionStateMachine::new();
    
    // Test abandoned state is terminal
    state_machine.transition(SessionEvent::Start).unwrap();
    state_machine.transition(SessionEvent::Abandon).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Abandoned);
    
    // No further transitions allowed from abandoned
    assert!(state_machine.transition(SessionEvent::Start).is_err());
    assert!(state_machine.transition(SessionEvent::Pause).is_err());
    assert!(state_machine.transition(SessionEvent::Resume).is_err());
    assert!(state_machine.transition(SessionEvent::Complete).is_err());
    assert!(state_machine.transition(SessionEvent::Abandon).is_err());
}

#[test]
fn test_break_state_machine() {
    use pomo_tui::core::{BreakStateMachine, BreakState, BreakEvent};
    
    let mut break_machine = BreakStateMachine::new();
    
    // Initial state
    assert_eq!(break_machine.current_state(), BreakState::Idle);
    
    // Start break
    assert!(break_machine.transition(BreakEvent::Start).is_ok());
    assert_eq!(break_machine.current_state(), BreakState::Active);
    
    // Complete break
    assert!(break_machine.transition(BreakEvent::Complete).is_ok());
    assert_eq!(break_machine.current_state(), BreakState::Completed);
}

#[test]
fn test_break_state_machine_skip() {
    use pomo_tui::core::{BreakStateMachine, BreakState, BreakEvent};
    
    let mut break_machine = BreakStateMachine::new();
    
    // Skip break without starting
    assert!(break_machine.transition(BreakEvent::Skip).is_ok());
    assert_eq!(break_machine.current_state(), BreakState::Skipped);
    
    // Test skip from active
    let mut break_machine2 = BreakStateMachine::new();
    break_machine2.transition(BreakEvent::Start).unwrap();
    assert!(break_machine2.transition(BreakEvent::Skip).is_ok());
    assert_eq!(break_machine2.current_state(), BreakState::Skipped);
}

#[test]
fn test_task_state_machine() {
    use pomo_tui::core::{TaskStateMachine, TaskState, TaskEvent};
    
    let mut task_machine = TaskStateMachine::new();
    
    // Initial state
    assert_eq!(task_machine.current_state(), TaskState::Pending);
    
    // Start working on task
    assert!(task_machine.transition(TaskEvent::StartSession).is_ok());
    assert_eq!(task_machine.current_state(), TaskState::InProgress);
    
    // Complete task
    assert!(task_machine.transition(TaskEvent::Complete).is_ok());
    assert_eq!(task_machine.current_state(), TaskState::Completed);
}

#[test]
fn test_task_state_machine_archive() {
    use pomo_tui::core::{TaskStateMachine, TaskState, TaskEvent};
    
    let mut task_machine = TaskStateMachine::new();
    
    // Archive from pending
    assert!(task_machine.transition(TaskEvent::Archive).is_ok());
    assert_eq!(task_machine.current_state(), TaskState::Archived);
    
    // Test archive from in-progress
    let mut task_machine2 = TaskStateMachine::new();
    task_machine2.transition(TaskEvent::StartSession).unwrap();
    assert!(task_machine2.transition(TaskEvent::Archive).is_ok());
    assert_eq!(task_machine2.current_state(), TaskState::Archived);
}

#[test]
fn test_state_machine_event_history() {
    let mut state_machine = SessionStateMachine::new();
    
    // Perform several transitions
    state_machine.transition(SessionEvent::Start).unwrap();
    state_machine.transition(SessionEvent::Pause).unwrap();
    state_machine.transition(SessionEvent::Resume).unwrap();
    state_machine.transition(SessionEvent::Complete).unwrap();
    
    let history = state_machine.event_history();
    
    assert_eq!(history.len(), 4);
    assert_eq!(history[0].event, SessionEvent::Start);
    assert_eq!(history[1].event, SessionEvent::Pause);
    assert_eq!(history[2].event, SessionEvent::Resume);
    assert_eq!(history[3].event, SessionEvent::Complete);
    
    // Verify timestamps are in order
    for i in 1..history.len() {
        assert!(history[i].timestamp >= history[i-1].timestamp);
    }
}

#[test]
fn test_state_machine_time_tracking() {
    let mut state_machine = SessionStateMachine::new();
    
    let start_time = std::time::Instant::now();
    
    state_machine.transition(SessionEvent::Start).unwrap();
    std::thread::sleep(Duration::from_millis(10));
    
    state_machine.transition(SessionEvent::Pause).unwrap();
    let pause_time = std::time::Instant::now();
    std::thread::sleep(Duration::from_millis(5));
    
    state_machine.transition(SessionEvent::Resume).unwrap();
    std::thread::sleep(Duration::from_millis(10));
    
    state_machine.transition(SessionEvent::Complete).unwrap();
    let end_time = std::time::Instant::now();
    
    let total_active_time = state_machine.total_active_time();
    let total_paused_time = state_machine.total_paused_time();
    
    // Should have ~20ms active time and ~5ms paused time (with some tolerance)
    assert!(total_active_time >= Duration::from_millis(15));
    assert!(total_active_time <= Duration::from_millis(30));
    assert!(total_paused_time >= Duration::from_millis(3));
    assert!(total_paused_time <= Duration::from_millis(10));
}

#[test]
fn test_concurrent_state_machine_access() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let state_machine = Arc::new(Mutex::new(SessionStateMachine::new()));
    let mut handles = vec![];
    
    // Start session in one thread
    let sm_clone = state_machine.clone();
    handles.push(thread::spawn(move || {
        let mut sm = sm_clone.lock().unwrap();
        sm.transition(SessionEvent::Start).unwrap();
    }));
    
    // Wait for threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final state
    let sm = state_machine.lock().unwrap();
    assert_eq!(sm.current_state(), SessionState::Active);
}

#[test]
fn test_state_machine_validation_rules() {
    let mut state_machine = SessionStateMachine::new();
    
    // Test that validation rules are enforced
    state_machine.transition(SessionEvent::Start).unwrap();
    
    // Try invalid event with validation
    let result = state_machine.transition_with_validation(
        SessionEvent::Resume,
        |current_state, event| {
            // Custom validation: only allow resume from paused state
            matches!(current_state, SessionState::Paused)
        }
    );
    
    assert!(result.is_err());
    assert_eq!(state_machine.current_state(), SessionState::Active);
}

#[test]
fn test_state_machine_rollback() {
    let mut state_machine = SessionStateMachine::new();
    
    state_machine.transition(SessionEvent::Start).unwrap();
    state_machine.transition(SessionEvent::Pause).unwrap();
    
    // Save checkpoint
    let checkpoint = state_machine.create_checkpoint();
    
    // Make more transitions
    state_machine.transition(SessionEvent::Resume).unwrap();
    state_machine.transition(SessionEvent::Complete).unwrap();
    
    assert_eq!(state_machine.current_state(), SessionState::Completed);
    
    // Rollback to checkpoint
    state_machine.restore_from_checkpoint(checkpoint).unwrap();
    assert_eq!(state_machine.current_state(), SessionState::Paused);
}

#[test]
fn test_application_state_machine() {
    use pomo_tui::core::{AppStateMachine, AppState, AppEvent};
    
    let mut app_machine = AppStateMachine::new();
    
    // Initial state
    assert_eq!(app_machine.current_state(), AppState::Idle);
    
    // Load application
    assert!(app_machine.transition(AppEvent::Load).is_ok());
    assert_eq!(app_machine.current_state(), AppState::Running);
    
    // Start session
    assert!(app_machine.transition(AppEvent::StartSession).is_ok());
    assert_eq!(app_machine.current_state(), AppState::InSession);
    
    // Start break
    assert!(app_machine.transition(AppEvent::StartBreak).is_ok());
    assert_eq!(app_machine.current_state(), AppState::OnBreak);
    
    // End break
    assert!(app_machine.transition(AppEvent::EndBreak).is_ok());
    assert_eq!(app_machine.current_state(), AppState::Running);
    
    // Shutdown
    assert!(app_machine.transition(AppEvent::Shutdown).is_ok());
    assert_eq!(app_machine.current_state(), AppState::Shutdown);
}
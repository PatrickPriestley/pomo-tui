use crate::core::{BreathingExercise, BreathingPattern, Timer};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, time::Duration};
use tokio::time;

pub struct App {
    timer: Timer,
    breathing_exercise: Option<BreathingExercise>,
    mode: AppMode,
    should_quit: bool,
    session_count: u32,
    break_was_shortened: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Pomodoro,
    Break,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            timer: Timer::new(25 * 60), // 25 minute pomodoro
            breathing_exercise: None,
            mode: AppMode::Pomodoro,
            should_quit: false,
            session_count: 0,
            break_was_shortened: false,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Setup terminal
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let mut interval = time::interval(Duration::from_millis(100));

        loop {
            // Draw UI
            terminal.draw(|f| super::ui::draw(f, self))?;

            // Handle input
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }

            // Update timers
            self.update();

            // Check if should quit
            if self.should_quit {
                break;
            }

            interval.tick().await;
        }

        // Restore terminal
        disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char(' ') => self.toggle_timer(),
            KeyCode::Char('r') => self.reset_timer(),
            KeyCode::Char('s') => self.skip_to_break(),
            KeyCode::Char('b') => self.skip_break(),
            KeyCode::Char('h') => self.shorten_break(),
            KeyCode::Char('e') => self.extend_break(),
            KeyCode::Char('1') => self.set_breathing_pattern(BreathingPattern::Simple),
            KeyCode::Char('2') => self.set_breathing_pattern(BreathingPattern::Box),
            KeyCode::Char('3') => self.set_breathing_pattern(BreathingPattern::FourSevenEight),
            _ => {}
        }
    }

    fn toggle_timer(&mut self) {
        match self.timer.state() {
            crate::core::timer::TimerState::Idle => self.timer.start(),
            crate::core::timer::TimerState::Running => self.timer.pause(),
            crate::core::timer::TimerState::Paused => self.timer.resume(),
            crate::core::timer::TimerState::Completed => self.start_next_phase(),
        }
    }

    fn reset_timer(&mut self) {
        self.timer.reset();
        if self.mode == AppMode::Break {
            self.breathing_exercise = None;
        }
    }

    pub fn skip_to_break(&mut self) {
        if self.mode == AppMode::Pomodoro {
            // Increment session count when skipping pomodoro
            self.session_count += 1;
            self.start_break();
        }
    }

    fn skip_break(&mut self) {
        if self.mode == AppMode::Break {
            self.start_pomodoro();
        }
    }

    fn shorten_break(&mut self) {
        if self.mode == AppMode::Break {
            let current_duration = self.timer.duration().as_secs();
            let short_break_duration = 5 * 60; // 5 minutes

            // Only shorten if current duration is longer than short break
            if current_duration > short_break_duration {
                self.timer = Timer::new(short_break_duration);
                self.break_was_shortened = true;
                // Maintain breathing exercise if present
                if self.breathing_exercise.is_none() {
                    self.breathing_exercise =
                        Some(BreathingExercise::new(BreathingPattern::Simple));
                }
            }
        }
    }

    fn extend_break(&mut self) {
        if self.mode == AppMode::Break && self.break_was_shortened {
            // Check if we're in a break after 4th session (which should be long break)
            if (self.session_count % 4) == 0 {
                let long_break_duration = 15 * 60; // 15 minutes
                self.timer = Timer::new(long_break_duration);
                self.break_was_shortened = false;
                // Maintain breathing exercise if present
                if self.breathing_exercise.is_none() {
                    self.breathing_exercise =
                        Some(BreathingExercise::new(BreathingPattern::Simple));
                }
            }
        }
    }

    fn set_breathing_pattern(&mut self, pattern: BreathingPattern) {
        if self.mode == AppMode::Break {
            self.breathing_exercise = Some(BreathingExercise::new(pattern));
        }
    }

    fn update(&mut self) {
        // Check if timer expired
        if self.timer.is_expired() {
            self.timer.stop();
            // Increment session count when pomodoro completes
            if self.mode == AppMode::Pomodoro {
                self.session_count += 1;
            }
        }

        // Update breathing exercise if active and timer is running
        if self.timer.state() == crate::core::timer::TimerState::Running {
            if let Some(ref mut exercise) = self.breathing_exercise {
                exercise.update(Duration::from_millis(100));
            }
        }
    }

    fn start_next_phase(&mut self) {
        match self.mode {
            AppMode::Pomodoro => self.start_break(),
            AppMode::Break => self.start_pomodoro(),
        }
    }

    fn start_break(&mut self) {
        let break_duration = if self.session_count % 4 == 0 {
            15 * 60 // Long break after 4 sessions
        } else {
            5 * 60 // Short break
        };

        self.mode = AppMode::Break;
        self.timer = Timer::new(break_duration);
        self.break_was_shortened = false; // Reset shortened state for new break
        self.breathing_exercise = Some(BreathingExercise::new(BreathingPattern::Simple));
        // Don't auto-start - wait for user to press space
    }

    fn start_pomodoro(&mut self) {
        self.mode = AppMode::Pomodoro;
        self.timer = Timer::new(25 * 60);
        self.break_was_shortened = false; // Reset shortened state for new pomodoro
        self.breathing_exercise = None;
        // Don't auto-start - wait for user to press space
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn breathing_exercise(&self) -> Option<&BreathingExercise> {
        self.breathing_exercise.as_ref()
    }

    pub fn mode(&self) -> AppMode {
        self.mode
    }

    pub fn session_count(&self) -> u32 {
        self.session_count
    }

    pub fn break_was_shortened(&self) -> bool {
        self.break_was_shortened
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_skip_break_only_works_in_break_mode() {
        let mut app = App::new().unwrap();

        // Start in Pomodoro mode
        assert_eq!(app.mode(), AppMode::Pomodoro);

        // Skip break should do nothing in Pomodoro mode
        app.skip_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);

        // Switch to break mode
        app.skip_to_break();
        assert_eq!(app.mode(), AppMode::Break);

        // Now skip break should work
        app.skip_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);
    }

    #[test]
    fn test_skip_break_resets_timer_to_pomodoro() {
        let mut app = App::new().unwrap();

        // Start a break
        app.skip_to_break();
        assert_eq!(app.mode(), AppMode::Break);

        // Verify break timer duration (5 min for first break)
        assert_eq!(app.timer().duration().as_secs(), 5 * 60);

        // Skip the break
        app.skip_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);

        // Verify timer is reset to pomodoro duration
        assert_eq!(app.timer().duration().as_secs(), 25 * 60);
        assert_eq!(app.timer().state(), crate::core::timer::TimerState::Idle);
    }

    #[test]
    fn test_key_handler_skip_break() {
        let mut app = App::new().unwrap();

        // Start in break mode
        app.skip_to_break();
        assert_eq!(app.mode(), AppMode::Break);

        // Press 'b' key to skip break
        let key_event = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        app.handle_key(key_event);

        // Should be back in Pomodoro mode
        assert_eq!(app.mode(), AppMode::Pomodoro);
    }

    #[test]
    fn test_skip_break_maintains_session_count() {
        let mut app = App::new().unwrap();

        let initial_count = app.session_count();

        // Skip to break (increments session count)
        app.skip_to_break();
        assert_eq!(app.session_count(), initial_count + 1);

        // Skip the break (should not change session count)
        app.skip_break();
        assert_eq!(app.session_count(), initial_count + 1);
        assert_eq!(app.mode(), AppMode::Pomodoro);
    }

    #[test]
    fn test_shorten_break_reduces_timer_duration() {
        let mut app = App::new().unwrap();

        // Start a long break (after 4 sessions)
        app.session_count = 4;
        app.start_break();
        assert_eq!(app.mode(), AppMode::Break);
        assert_eq!(app.timer().duration().as_secs(), 15 * 60); // 15 min long break

        // Shorten the break
        app.shorten_break();

        // Should reduce to short break duration
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // 5 min short break
        assert_eq!(app.mode(), AppMode::Break); // Should still be in break mode
        assert_eq!(app.timer().state(), crate::core::timer::TimerState::Idle); // Timer should reset
    }

    #[test]
    fn test_shorten_break_only_works_in_break_mode() {
        let mut app = App::new().unwrap();

        // Start in Pomodoro mode
        assert_eq!(app.mode(), AppMode::Pomodoro);
        let original_duration = app.timer().duration();

        // Shorten break should do nothing in Pomodoro mode
        app.shorten_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);
        assert_eq!(app.timer().duration(), original_duration);
    }

    #[test]
    fn test_shorten_break_with_short_break_has_no_effect() {
        let mut app = App::new().unwrap();

        // Start a short break (first session)
        app.skip_to_break();
        assert_eq!(app.mode(), AppMode::Break);
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // Already 5 min short break

        // Shortening should have no effect since it's already a short break
        app.shorten_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // Still 5 min
        assert_eq!(app.mode(), AppMode::Break);
    }

    #[test]
    fn test_key_handler_shorten_break() {
        let mut app = App::new().unwrap();

        // Start in long break mode (after 4 sessions)
        app.session_count = 4;
        app.start_break();
        assert_eq!(app.mode(), AppMode::Break);
        assert_eq!(app.timer().duration().as_secs(), 15 * 60);

        // Press 'h' key to shorten break
        let key_event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        app.handle_key(key_event);

        // Should shorten to 5 minutes
        assert_eq!(app.timer().duration().as_secs(), 5 * 60);
        assert_eq!(app.mode(), AppMode::Break);
    }

    #[test]
    fn test_extended_break_automation_complete_workflow() {
        let mut app = App::new().unwrap();

        // Complete 3 pomodoro sessions (should be short breaks)
        for i in 1..=3 {
            app.skip_to_break();
            assert_eq!(app.session_count(), i);
            assert_eq!(app.mode(), AppMode::Break);
            assert_eq!(app.timer().duration().as_secs(), 5 * 60); // Short break
            app.skip_break(); // Skip to next pomodoro
        }

        // 4th session should trigger long break (15 min)
        app.skip_to_break();
        assert_eq!(app.session_count(), 4);
        assert_eq!(app.mode(), AppMode::Break);
        assert_eq!(app.timer().duration().as_secs(), 15 * 60); // Long break

        // Test shortening the long break
        app.shorten_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // Shortened to 5 min
        assert_eq!(app.mode(), AppMode::Break); // Still in break mode

        // Complete the break cycle
        app.skip_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);

        // Next break should be short again (cycle reset)
        app.skip_to_break();
        assert_eq!(app.session_count(), 5);
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // Back to short break
    }

    #[test]
    fn test_extend_break_restores_full_duration() {
        let mut app = App::new().unwrap();

        // Start a long break (after 4 sessions)
        app.session_count = 4;
        app.start_break();
        assert_eq!(app.timer().duration().as_secs(), 15 * 60); // 15 min long break
        assert!(!app.break_was_shortened());

        // Shorten the break
        app.shorten_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // 5 min short break
        assert!(app.break_was_shortened());

        // Extend the break back to full duration
        app.extend_break();
        assert_eq!(app.timer().duration().as_secs(), 15 * 60); // Back to 15 min
        assert!(!app.break_was_shortened()); // Should reset shortened flag
        assert_eq!(app.mode(), AppMode::Break); // Still in break mode
    }

    #[test]
    fn test_extend_break_only_works_on_shortened_breaks() {
        let mut app = App::new().unwrap();

        // Start a regular long break (not shortened)
        app.session_count = 4;
        app.start_break();
        assert_eq!(app.timer().duration().as_secs(), 15 * 60);
        assert!(!app.break_was_shortened());

        // Try to extend - should have no effect
        app.extend_break();
        assert_eq!(app.timer().duration().as_secs(), 15 * 60); // No change
        assert!(!app.break_was_shortened());
    }

    #[test]
    fn test_extend_break_only_works_in_break_mode() {
        let mut app = App::new().unwrap();

        // Start in Pomodoro mode
        assert_eq!(app.mode(), AppMode::Pomodoro);
        let original_duration = app.timer().duration();

        // Extend should do nothing in Pomodoro mode
        app.extend_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);
        assert_eq!(app.timer().duration(), original_duration);
        assert!(!app.break_was_shortened());
    }

    #[test]
    fn test_extend_break_only_works_for_long_breaks() {
        let mut app = App::new().unwrap();

        // Start a short break (first session)
        app.skip_to_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // 5 min short break

        // Manually set shortened flag (simulate a broken state)
        app.break_was_shortened = true;

        // Try to extend - should have no effect since it's not after 4th session
        app.extend_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // No change
        assert!(app.break_was_shortened()); // Flag should remain
    }

    #[test]
    fn test_key_handler_extend_break() {
        let mut app = App::new().unwrap();

        // Start a long break and shorten it
        app.session_count = 4;
        app.start_break();
        app.shorten_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60);
        assert!(app.break_was_shortened());

        // Press 'e' key to extend break
        let key_event = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        app.handle_key(key_event);

        // Should extend back to 15 minutes
        assert_eq!(app.timer().duration().as_secs(), 15 * 60);
        assert!(!app.break_was_shortened());
        assert_eq!(app.mode(), AppMode::Break);
    }

    #[test]
    fn test_shortened_state_resets_on_new_cycles() {
        let mut app = App::new().unwrap();

        // Start a long break and shorten it
        app.session_count = 4;
        app.start_break();
        app.shorten_break();
        assert!(app.break_was_shortened());

        // Skip break to start new pomodoro
        app.skip_break();
        assert!(!app.break_was_shortened()); // Should reset when starting new pomodoro

        // Start new break
        app.skip_to_break();
        assert!(!app.break_was_shortened()); // Should remain reset for new break
    }

    #[test]
    fn test_complete_shorten_extend_workflow() {
        let mut app = App::new().unwrap();

        // Build up to 4th session for long break
        for i in 1..=3 {
            app.skip_to_break();
            assert_eq!(app.session_count(), i);
            assert!(!app.break_was_shortened()); // Should be false for short breaks
            app.skip_break();
        }

        // 4th session: Long break (15 min)
        app.skip_to_break();
        assert_eq!(app.session_count(), 4);
        assert_eq!(app.timer().duration().as_secs(), 15 * 60);
        assert!(!app.break_was_shortened()); // Initially not shortened

        // Shorten to 5 minutes
        app.shorten_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60);
        assert!(app.break_was_shortened()); // Now marked as shortened

        // Extend back to 15 minutes
        app.extend_break();
        assert_eq!(app.timer().duration().as_secs(), 15 * 60);
        assert!(!app.break_was_shortened()); // No longer marked as shortened

        // Shorten again
        app.shorten_break();
        assert_eq!(app.timer().duration().as_secs(), 5 * 60);
        assert!(app.break_was_shortened());

        // Extend again
        app.extend_break();
        assert_eq!(app.timer().duration().as_secs(), 15 * 60);
        assert!(!app.break_was_shortened());

        // Complete the break
        app.skip_break();
        assert_eq!(app.mode(), AppMode::Pomodoro);
        assert!(!app.break_was_shortened()); // Should reset when leaving break

        // Next cycle should start fresh
        app.skip_to_break();
        assert_eq!(app.session_count(), 5);
        assert_eq!(app.timer().duration().as_secs(), 5 * 60); // Back to short break
        assert!(!app.break_was_shortened());
    }
}

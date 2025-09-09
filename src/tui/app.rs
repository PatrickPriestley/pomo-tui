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
        self.breathing_exercise = Some(BreathingExercise::new(BreathingPattern::Simple));
        // Don't auto-start - wait for user to press space
    }

    fn start_pomodoro(&mut self) {
        self.mode = AppMode::Pomodoro;
        self.timer = Timer::new(25 * 60);
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
}

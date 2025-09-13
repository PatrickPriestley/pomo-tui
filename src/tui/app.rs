use crate::audio::{AudioManager, SoundType};
use crate::core::{BreathingExercise, BreathingPattern, Timer};
use crate::integrations::{DndState, MacOSDndController};
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
    dnd_controller: Option<MacOSDndController>,
    dnd_auto_enabled: bool,
    dnd_state: DndState,
    status_message: Option<String>,
    audio_manager: AudioManager,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Pomodoro,
    Break,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut dnd_controller = if MacOSDndController::is_supported() {
            Some(MacOSDndController::new())
        } else {
            None
        };

        let mut dnd_state = DndState::Unknown;
        let mut status_message = None;

        if let Some(ref mut controller) = dnd_controller {
            // Get initial state
            dnd_state = controller.get_state().unwrap_or(DndState::Unknown);

            // Check if Focus mode is properly configured (handle errors gracefully)
            match controller.check_shortcuts_exist() {
                Ok((enable_exists, disable_exists)) => {
                    if !enable_exists || !disable_exists {
                        // Focus mode not configured - show warning in status only
                        status_message =
                            Some("⚠️ Focus mode not configured - press 'F' for help".to_string());
                    }
                }
                Err(_) => {
                    // If shortcuts check fails (permissions, device not configured, etc.)
                    // just show a warning and continue - don't crash the app
                    status_message = Some("⚠️ Focus mode unavailable - press 'F' for help".to_string());
                }
            }
        }

        // Initialize audio manager
        let audio_manager = AudioManager::default();

        Ok(Self {
            timer: Timer::new(25 * 60), // 25 minute pomodoro
            breathing_exercise: None,
            mode: AppMode::Pomodoro,
            should_quit: false,
            session_count: 0,
            break_was_shortened: false,
            dnd_controller,
            dnd_auto_enabled: true, // Default to auto-enable DND
            dnd_state,
            status_message,
            audio_manager,
        })
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Setup terminal
        enable_raw_mode().map_err(|e| {
            // Provide helpful error message for terminal setup failures
            Box::<dyn Error>::from(format!(
                "Failed to initialize terminal: {}\n\n\
                This typically means pomo-tui is not running in a proper terminal.\n\
                Please run pomo-tui directly from your terminal (Terminal.app, iTerm2, etc.)\n\
                rather than through an IDE or other application.",
                e
            ))
        })?;
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
            KeyCode::Char('q') => {
                // Restore DND state before quitting
                self.restore_dnd_state();
                self.should_quit = true;
            }
            KeyCode::Esc => {
                // Clear status message or quit if no message
                if self.status_message.is_some() {
                    self.status_message = None;
                } else {
                    self.restore_dnd_state();
                    self.should_quit = true;
                }
            }
            KeyCode::Char(' ') => self.toggle_timer(),
            KeyCode::Char('r') => self.reset_timer(),
            KeyCode::Char('s') => self.skip_to_break(),
            KeyCode::Char('b') => self.skip_break(),
            KeyCode::Char('h') => self.shorten_break(),
            KeyCode::Char('e') => self.extend_break(),
            KeyCode::Char('d') => {
                match self.toggle_dnd() {
                    Ok(_) => {
                        self.status_message =
                            Some("✅ Focus mode toggled successfully".to_string());
                    }
                    Err(err) => {
                        // Show clean error message without stderr output to avoid overlapping text
                        self.status_message = Some(format!("❌ {}", err));
                        // Update state from system to ensure consistency
                        self.update_dnd_state();
                    }
                }
            }
            KeyCode::Char('a') => self.toggle_dnd_auto_enable(),
            KeyCode::Char('c') => {
                // Clear status message
                self.status_message = None;
            }
            KeyCode::Char('f') => {
                // Show Focus mode setup help
                if let Some(ref controller) = self.dnd_controller {
                    match controller.check_shortcuts_exist() {
                        Ok((enable_exists, disable_exists)) => {
                            if enable_exists && disable_exists {
                                self.status_message = Some(
                                    "✅ Focus mode shortcuts configured correctly!".to_string(),
                                );
                            } else {
                                // Show full setup instructions in status message
                                self.status_message = Some(format!(
                                    "📋 Focus Mode Setup Instructions:\n\n{}",
                                    controller.get_setup_instructions()
                                ));
                            }
                        }
                        Err(_) => {
                            self.status_message =
                                Some("❌ Unable to check Focus mode shortcuts".to_string());
                        }
                    }
                } else {
                    self.status_message =
                        Some("❌ Focus mode not supported on this platform".to_string());
                }
            }
            KeyCode::Char('1') => self.set_breathing_pattern(BreathingPattern::Simple),
            KeyCode::Char('2') => self.set_breathing_pattern(BreathingPattern::Box),
            KeyCode::Char('3') => self.set_breathing_pattern(BreathingPattern::FourSevenEight),
            KeyCode::Char('m') => self.toggle_audio_mute(),
            KeyCode::Char('+') | KeyCode::Char('=') => self.increase_volume(),
            KeyCode::Char('-') => self.decrease_volume(),
            KeyCode::Char('t') => self.play_test_sound(),
            _ => {}
        }
    }

    fn toggle_timer(&mut self) {
        match self.timer.state() {
            crate::core::timer::TimerState::Idle => {
                self.timer.start();
                // Enable DND when starting a Pomodoro session
                if self.mode == AppMode::Pomodoro {
                    self.auto_enable_dnd();
                }
            }
            crate::core::timer::TimerState::Running => {
                self.timer.pause();
                // Disable Focus mode when pausing to allow interruptions
                if self.mode == AppMode::Pomodoro {
                    self.auto_disable_dnd();
                }
            }
            crate::core::timer::TimerState::Paused => {
                self.timer.resume();
                // Re-enable Focus mode when resuming Pomodoro
                if self.mode == AppMode::Pomodoro {
                    self.auto_enable_dnd();
                }
            }
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
            // DND is automatically disabled in start_break()
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

    fn toggle_audio_mute(&mut self) {
        let is_muted = self.audio_manager.toggle_mute();
        if is_muted {
            self.status_message = Some("🔇 Audio muted".to_string());
        } else {
            self.status_message = Some("🔊 Audio unmuted".to_string());
        }
    }

    fn increase_volume(&mut self) {
        let current_volume = self.audio_manager.volume();
        let new_volume = (current_volume + 0.1).min(1.0);
        self.audio_manager.set_volume(new_volume);
        let percentage = (new_volume * 100.0).round() as u8;
        self.status_message = Some(format!("🔊 Volume: {}%", percentage));
    }

    fn decrease_volume(&mut self) {
        let current_volume = self.audio_manager.volume();
        let new_volume = (current_volume - 0.1).max(0.0);
        self.audio_manager.set_volume(new_volume);
        let percentage = (new_volume * 100.0).round() as u8;
        self.status_message = Some(format!("🔉 Volume: {}%", percentage));
    }

    fn play_test_sound(&mut self) {
        match self.audio_manager.play_test_sound() {
            Ok(()) => {
                let volume = (self.audio_manager.volume() * 100.0).round() as u8;
                let mute_status = if self.audio_manager.is_muted() { " (muted)" } else { "" };
                self.status_message = Some(format!("🎵 Test sound played - Volume: {}{}", volume, mute_status));
            }
            Err(_) => {
                self.status_message = Some("❌ Audio not available".to_string());
            }
        }
    }

    fn update(&mut self) {
        // Check if timer expired
        if self.timer.is_expired() {
            self.timer.stop();
            
            // Play appropriate notification sound
            match self.mode {
                AppMode::Pomodoro => {
                    // Session completed - time for a break
                    let _ = self.audio_manager.play_notification(SoundType::SessionComplete);
                    self.session_count += 1;
                }
                AppMode::Break => {
                    // Break completed - time to work
                    let _ = self.audio_manager.play_notification(SoundType::BreakComplete);
                }
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
        let is_long_break = self.session_count % 4 == 0;
        let break_duration = if is_long_break {
            15 * 60 // Long break after 4 sessions
        } else {
            5 * 60 // Short break
        };

        self.mode = AppMode::Break;
        self.timer = Timer::new(break_duration);
        self.break_was_shortened = false; // Reset shortened state for new break
        self.breathing_exercise = Some(BreathingExercise::new(BreathingPattern::Simple));
        
        // Play special sound for long break
        if is_long_break {
            let _ = self.audio_manager.play_notification(SoundType::LongBreakStart);
        }
        
        // Disable DND when starting a break
        self.auto_disable_dnd();
        // Don't auto-start - wait for user to press space
    }

    fn start_pomodoro(&mut self) {
        self.mode = AppMode::Pomodoro;
        self.timer = Timer::new(25 * 60);
        self.break_was_shortened = false; // Reset shortened state for new pomodoro
        self.breathing_exercise = None;
        // DND will be enabled when timer starts (in toggle_timer)
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

    pub fn dnd_state(&self) -> DndState {
        self.dnd_state
    }

    pub fn dnd_auto_enabled(&self) -> bool {
        self.dnd_auto_enabled
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    pub fn is_dnd_supported(&self) -> bool {
        self.dnd_controller.is_some()
    }

    pub fn audio_is_muted(&self) -> bool {
        self.audio_manager.is_muted()
    }

    pub fn audio_is_available(&self) -> bool {
        self.audio_manager.is_available()
    }

    pub fn audio_volume(&self) -> f32 {
        self.audio_manager.volume()
    }

    /// Toggle DND auto-enable setting
    pub fn toggle_dnd_auto_enable(&mut self) {
        self.dnd_auto_enabled = !self.dnd_auto_enabled;
    }

    /// Manually toggle DND state
    pub fn toggle_dnd(&mut self) -> Result<DndState, String> {
        if let Some(ref mut controller) = self.dnd_controller {
            match controller.toggle() {
                Ok(new_state) => {
                    self.dnd_state = new_state;
                    // Force a fresh read from the system to ensure UI reflects actual state
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    self.update_dnd_state();
                    Ok(self.dnd_state)
                }
                Err(e) => {
                    let error_msg = e.to_string();

                    // Refresh shortcuts cache to detect if they were removed
                    if controller.refresh_shortcuts_cache().is_err() {
                        // If refresh fails, shortcuts were likely removed
                    }

                    if error_msg.contains("not configured") || error_msg.contains("not found") {
                        // Clear any previous status and show clean error
                        Err(
                            "Focus mode shortcuts not configured - press 'F' for setup help"
                                .to_string(),
                        )
                    } else {
                        Err(format!(
                            "Focus mode error: {}",
                            error_msg.lines().next().unwrap_or(&error_msg)
                        ))
                    }
                }
            }
        } else {
            Err("Focus mode not supported on this platform".to_string())
        }
    }

    /// Check if Focus mode shortcuts are properly configured
    pub fn check_focus_setup(&mut self) -> Result<String, String> {
        if let Some(ref mut controller) = self.dnd_controller {
            match controller.check_shortcuts_exist() {
                Ok((enable_exists, disable_exists)) => {
                    if enable_exists && disable_exists {
                        Ok("Focus mode shortcuts are configured correctly!".to_string())
                    } else {
                        let missing = if !enable_exists && !disable_exists {
                            "both enable and disable shortcuts"
                        } else if !enable_exists {
                            "enable shortcut"
                        } else {
                            "disable shortcut"
                        };
                        Err(format!(
                            "Missing {} for Focus mode.\n\n{}",
                            missing,
                            controller.get_setup_instructions()
                        ))
                    }
                }
                Err(e) => Err(format!("Could not check shortcuts: {}", e)),
            }
        } else {
            Err("Focus mode not supported on this platform".to_string())
        }
    }

    /// Update current DND state from system
    fn update_dnd_state(&mut self) {
        if let Some(ref mut controller) = self.dnd_controller {
            if let Ok(state) = controller.get_state() {
                self.dnd_state = state;
            }
        }
    }

    /// Enable DND if auto-enable is on
    fn auto_enable_dnd(&mut self) {
        if self.dnd_auto_enabled && self.dnd_controller.is_some() {
            if let Some(ref mut controller) = self.dnd_controller {
                let _ = controller.enable();
                self.update_dnd_state();
            }
        }
    }

    /// Disable DND if auto-enable is on
    fn auto_disable_dnd(&mut self) {
        if self.dnd_auto_enabled && self.dnd_controller.is_some() {
            if let Some(ref mut controller) = self.dnd_controller {
                let _ = controller.disable();
                self.update_dnd_state();
            }
        }
    }

    /// Restore original DND state when app exits
    pub fn restore_dnd_state(&mut self) {
        if let Some(ref mut controller) = self.dnd_controller {
            let _ = controller.restore_original_state();
            self.update_dnd_state();
        }
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

    #[test]
    fn test_dnd_initialization() {
        let app = App::new().unwrap();

        // DND should be initialized properly
        if app.is_dnd_supported() {
            // On macOS, DND controller should be present
            assert!(matches!(
                app.dnd_state(),
                DndState::Enabled | DndState::Disabled | DndState::Unknown
            ));
            assert!(app.dnd_auto_enabled()); // Default to auto-enabled
        } else {
            // On non-macOS, DND should not be supported
            assert_eq!(app.dnd_state(), DndState::Unknown);
        }
    }

    #[test]
    fn test_dnd_auto_enable_toggle() {
        let mut app = App::new().unwrap();

        let initial_state = app.dnd_auto_enabled();
        app.toggle_dnd_auto_enable();
        assert_eq!(app.dnd_auto_enabled(), !initial_state);

        app.toggle_dnd_auto_enable();
        assert_eq!(app.dnd_auto_enabled(), initial_state);
    }

    #[test]
    fn test_dnd_manual_toggle() {
        let mut app = App::new().unwrap();

        if app.is_dnd_supported() {
            // Test manual DND toggle
            let initial_state = app.dnd_state();
            let result = app.toggle_dnd();

            // Should either succeed or fail gracefully
            match result {
                Ok(new_state) => {
                    // If successful, state should have changed (unless it was Unknown)
                    if initial_state != DndState::Unknown {
                        assert_ne!(new_state, initial_state);
                    }
                }
                Err(_) => {
                    // Failed toggle is acceptable (might be permission issues)
                }
            }
        } else {
            // On non-macOS, should return error
            let result = app.toggle_dnd();
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("not supported"));
        }
    }

    #[test]
    fn test_dnd_keyboard_controls() {
        let mut app = App::new().unwrap();

        let initial_auto_state = app.dnd_auto_enabled();

        // Test 'a' key for auto-enable toggle
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        app.handle_key(key_event);
        assert_eq!(app.dnd_auto_enabled(), !initial_auto_state);

        // Test 'd' key for manual DND toggle (should not panic)
        let key_event = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
        app.handle_key(key_event);
        // No assertion here as result depends on platform and permissions
    }

    #[test]
    fn test_dnd_quit_behavior() {
        let mut app = App::new().unwrap();

        // Test that quitting restores DND state
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        app.handle_key(key_event);

        // Should be marked for quit
        assert!(app.should_quit);
        // restore_dnd_state() should have been called (no panic means success)
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_dnd_macos_integration() {
        let app = App::new().unwrap();

        // On macOS, DND should be supported
        assert!(app.is_dnd_supported());

        // DND state should be readable (might be any valid state)
        assert!(matches!(
            app.dnd_state(),
            DndState::Enabled | DndState::Disabled | DndState::Unknown
        ));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_dnd_non_macos_behavior() {
        let app = App::new().unwrap();

        // On non-macOS, DND should not be supported
        assert!(!app.is_dnd_supported());
        assert_eq!(app.dnd_state(), DndState::Unknown);
    }
}

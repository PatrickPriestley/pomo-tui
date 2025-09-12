use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::app::{App, Message, Tab, AppMode};
use crate::{Result, PomoError};

/// Handle keyboard events and convert them to application messages
pub async fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    // Handle global shortcuts first (work in any mode)
    if let Some(message) = handle_global_shortcuts(key, &app.state.current_mode, &app.state.current_tab) {
        app.send_message(message)?;
        return Ok(());
    }
    
    // Handle mode-specific shortcuts
    match app.state.current_mode {
        AppMode::Normal => handle_normal_mode(app, key).await?,
        AppMode::TaskInput => handle_task_input_mode(app, key).await?,
        AppMode::SessionActive => handle_session_active_mode(app, key).await?,
        AppMode::Break => handle_break_mode(app, key).await?,
        AppMode::Settings => handle_settings_mode(app, key).await?,
    }
    
    Ok(())
}

/// Handle global keyboard shortcuts that work in any mode
fn handle_global_shortcuts(key: KeyEvent, current_mode: &AppMode, current_tab: &Tab) -> Option<Message> {
    match key.code {
        // Quit application
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::Quit)
        }
        KeyCode::Esc if matches!(current_mode, AppMode::Normal) => {
            Some(Message::Quit)
        }
        
        // Help toggle
        KeyCode::F1 | KeyCode::Char('?') => {
            // Toggle help display (would be handled in app state)
            None
        }
        
        // Tab navigation (works in normal mode)
        KeyCode::Tab if matches!(current_mode, AppMode::Normal) => {
            Some(Message::NextTab)
        }
        KeyCode::BackTab if matches!(current_mode, AppMode::Normal) => {
            Some(Message::PreviousTab)
        }
        
        // Number keys for quick tab switching
        KeyCode::Char('1') if matches!(current_mode, AppMode::Normal) => {
            Some(Message::SwitchToTab(Tab::Tasks))
        }
        KeyCode::Char('2') if matches!(current_mode, AppMode::Normal) => {
            Some(Message::SwitchToTab(Tab::Timer))
        }
        KeyCode::Char('3') if matches!(current_mode, AppMode::Normal) => {
            Some(Message::SwitchToTab(Tab::Statistics))
        }
        KeyCode::Char('4') if matches!(current_mode, AppMode::Normal) => {
            Some(Message::SwitchToTab(Tab::Settings))
        }
        KeyCode::Char('5') if matches!(current_mode, AppMode::Normal) => {
            Some(Message::SwitchToTab(Tab::Integrations))
        }
        
        // Emergency session control
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::PauseSession)
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::ResumeSession)
        }
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::AbandonSession)
        }
        
        _ => None,
    }
}

/// Handle keyboard input in normal mode
async fn handle_normal_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    let message = match app.state.current_tab {
        Tab::Tasks => handle_tasks_tab(key, &app.state.selected_task_index),
        Tab::Timer => handle_timer_tab(key, app.is_session_active()),
        Tab::Statistics => handle_statistics_tab(key),
        Tab::Settings => handle_settings_tab(key),
        Tab::Integrations => handle_integrations_tab(key),
    };
    
    if let Some(msg) = message {
        app.send_message(msg)?;
    }
    
    Ok(())
}

/// Handle keyboard input in task input mode
async fn handle_task_input_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    let message = match key.code {
        KeyCode::Char(c) => Some(Message::InputChar(c)),
        KeyCode::Backspace => Some(Message::InputBackspace),
        KeyCode::Enter => Some(Message::InputEnter),
        KeyCode::Esc => Some(Message::InputEscape),
        _ => None,
    };
    
    if let Some(msg) = message {
        app.send_message(msg)?;
    }
    
    Ok(())
}

/// Handle keyboard input during active session
async fn handle_session_active_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    let message = match key.code {
        // Session control
        KeyCode::Char(' ') => Some(Message::PauseSession),
        KeyCode::Enter => Some(Message::CompleteSession),
        KeyCode::Esc => Some(Message::AbandonSession),
        
        // Audio control
        KeyCode::Char('m') => Some(Message::ToggleAudio),
        KeyCode::Char('a') => {
            use crate::audio::player::SoundType;
            Some(Message::PlayAmbientSound(SoundType::BrownNoise))
        }
        KeyCode::Char('s') => Some(Message::StopAmbientSound),
        
        // Volume control
        KeyCode::Char('+') | KeyCode::Char('=') => {
            // Increase volume (would need current volume state)
            None
        }
        KeyCode::Char('-') => {
            // Decrease volume
            None
        }
        
        // Integration controls
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::SyncGitHub)
        }
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::UpdateSlackStatus)
        }
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::ToggleWebsiteBlocking)
        }
        
        _ => None,
    };
    
    if let Some(msg) = message {
        app.send_message(msg)?;
    }
    
    Ok(())
}

/// Handle keyboard input during break mode
async fn handle_break_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    let message = match key.code {
        KeyCode::Char(' ') | KeyCode::Enter => Some(Message::CompleteBreak),
        KeyCode::Char('s') => Some(Message::SkipBreak),
        KeyCode::Esc => Some(Message::CompleteBreak),
        _ => None,
    };
    
    if let Some(msg) = message {
        app.send_message(msg)?;
    }
    
    Ok(())
}

/// Handle keyboard input in settings mode
async fn handle_settings_mode(app: &mut App, key: KeyEvent) -> Result<()> {
    // Settings mode would have its own navigation
    let message = match key.code {
        KeyCode::Esc => Some(Message::InputEscape),
        _ => None,
    };
    
    if let Some(msg) = message {
        app.send_message(msg)?;
    }
    
    Ok(())
}

/// Handle keyboard input in tasks tab
fn handle_tasks_tab(key: KeyEvent, selected_task: &Option<usize>) -> Option<Message> {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(current) = selected_task {
                if *current > 0 {
                    Some(Message::SelectTask(current - 1))
                } else {
                    None
                }
            } else {
                Some(Message::SelectTask(0))
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(current) = selected_task {
                Some(Message::SelectTask(current + 1))
            } else {
                Some(Message::SelectTask(0))
            }
        }
        
        // Task operations
        KeyCode::Char('n') => Some(Message::NewTask),
        KeyCode::Enter => {
            if let Some(index) = selected_task {
                // Start session with selected task
                Some(Message::StartSession(Some(*index as i64)))
            } else {
                Some(Message::StartSession(None))
            }
        }
        KeyCode::Char(' ') => {
            if let Some(index) = selected_task {
                Some(Message::ToggleTaskStatus(*index as i64))
            } else {
                None
            }
        }
        KeyCode::Char('e') => {
            if let Some(index) = selected_task {
                Some(Message::EditTask(*index as i64))
            } else {
                None
            }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if let Some(index) = selected_task {
                Some(Message::DeleteTask(*index as i64))
            } else {
                None
            }
        }
        
        // Refresh
        KeyCode::Char('r') => Some(Message::RefreshData),
        
        _ => None,
    }
}

/// Handle keyboard input in timer tab
fn handle_timer_tab(key: KeyEvent, is_session_active: bool) -> Option<Message> {
    match key.code {
        // Start/control session
        KeyCode::Enter => {
            if is_session_active {
                Some(Message::CompleteSession)
            } else {
                Some(Message::StartSession(None))
            }
        }
        KeyCode::Char(' ') => {
            if is_session_active {
                Some(Message::PauseSession)
            } else {
                None
            }
        }
        KeyCode::Esc => {
            if is_session_active {
                Some(Message::AbandonSession)
            } else {
                None
            }
        }
        
        // Quick session starts
        KeyCode::Char('p') => {
            // Start pomodoro (25 min)
            Some(Message::StartTimer(std::time::Duration::from_secs(25 * 60)))
        }
        KeyCode::Char('s') => {
            // Start short break (5 min)
            Some(Message::StartBreak("short".to_string()))
        }
        KeyCode::Char('l') => {
            // Start long break (15 min)
            Some(Message::StartBreak("long".to_string()))
        }
        
        // Audio controls
        KeyCode::Char('a') => {
            use crate::audio::player::SoundType;
            Some(Message::PlayAmbientSound(SoundType::BrownNoise))
        }
        KeyCode::Char('q') => {
            Some(Message::StopAmbientSound)
        }
        
        _ => None,
    }
}

/// Handle keyboard input in statistics tab
fn handle_statistics_tab(key: KeyEvent) -> Option<Message> {
    match key.code {
        // View switching
        KeyCode::Char('o') => {
            // Switch to overview (would need to track current view)
            None
        }
        KeyCode::Char('c') => {
            // Switch to charts
            None
        }
        KeyCode::Char('t') => {
            // Switch to trends
            None
        }
        
        // Time period switching
        KeyCode::Left => {
            // Previous time period
            None
        }
        KeyCode::Right => {
            // Next time period
            None
        }
        
        // Refresh
        KeyCode::Char('r') => Some(Message::RefreshData),
        
        _ => None,
    }
}

/// Handle keyboard input in settings tab
fn handle_settings_tab(key: KeyEvent) -> Option<Message> {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => None,
        KeyCode::Down | KeyCode::Char('j') => None,
        
        // Edit setting
        KeyCode::Enter => None,
        
        // Reset to defaults
        KeyCode::Char('R') => None,
        
        _ => None,
    }
}

/// Handle keyboard input in integrations tab
fn handle_integrations_tab(key: KeyEvent) -> Option<Message> {
    match key.code {
        // GitHub operations
        KeyCode::Char('g') => Some(Message::SyncGitHub),
        
        // Slack operations
        KeyCode::Char('s') => Some(Message::UpdateSlackStatus),
        
        // Website blocking
        KeyCode::Char('b') => Some(Message::ToggleWebsiteBlocking),
        
        // Git operations
        KeyCode::Char('c') => {
            // Create git branch (would need selected task)
            None
        }
        
        // Test connections
        KeyCode::Char('t') => {
            // Test integrations
            None
        }
        
        _ => None,
    }
}

/// Keyboard shortcut help text
pub fn get_help_text(current_tab: &Tab, current_mode: &AppMode) -> Vec<(&'static str, &'static str)> {
    let mut help = Vec::new();
    
    // Global shortcuts
    help.push(("Ctrl+Q", "Quit"));
    help.push(("F1/?", "Help"));
    help.push(("1-5", "Switch tabs"));
    help.push(("Tab", "Next tab"));
    
    // Tab-specific shortcuts
    match current_tab {
        Tab::Tasks => {
            help.push(("↑↓/jk", "Navigate"));
            help.push(("n", "New task"));
            help.push(("Enter", "Start session"));
            help.push(("Space", "Toggle status"));
            help.push(("e", "Edit"));
            help.push(("d/Del", "Delete"));
            help.push(("r", "Refresh"));
        }
        Tab::Timer => {
            help.push(("Enter", "Start/Complete"));
            help.push(("Space", "Pause/Resume"));
            help.push(("Esc", "Abandon"));
            help.push(("p", "Pomodoro (25min)"));
            help.push(("s", "Short break (5min)"));
            help.push(("l", "Long break (15min)"));
            help.push(("a", "Ambient sound"));
            help.push(("q", "Quiet"));
        }
        Tab::Statistics => {
            help.push(("o", "Overview"));
            help.push(("c", "Charts"));
            help.push(("t", "Trends"));
            help.push(("←→", "Time period"));
            help.push(("r", "Refresh"));
        }
        Tab::Settings => {
            help.push(("↑↓/jk", "Navigate"));
            help.push(("Enter", "Edit"));
            help.push(("R", "Reset defaults"));
        }
        Tab::Integrations => {
            help.push(("g", "Sync GitHub"));
            help.push(("s", "Update Slack"));
            help.push(("b", "Toggle blocking"));
            help.push(("c", "Git branch"));
            help.push(("t", "Test connections"));
        }
    }
    
    // Mode-specific shortcuts
    match current_mode {
        AppMode::SessionActive => {
            help.push(("Ctrl+P", "Emergency pause"));
            help.push(("Ctrl+X", "Emergency abandon"));
            help.push(("Ctrl+G", "Sync GitHub"));
            help.push(("Ctrl+L", "Update Slack"));
            help.push(("Ctrl+B", "Toggle blocking"));
        }
        AppMode::TaskInput => {
            help.push(("Enter", "Save"));
            help.push(("Esc", "Cancel"));
        }
        AppMode::Break => {
            help.push(("Space/Enter", "End break"));
            help.push(("s", "Skip break"));
        }
        _ => {}
    }
    
    help
}

/// Check if a key combination is valid for the current context
pub fn is_valid_key_combination(key: KeyEvent, current_mode: &AppMode) -> bool {
    match current_mode {
        AppMode::TaskInput => {
            // In input mode, most keys are for text input
            matches!(key.code, 
                KeyCode::Char(_) | 
                KeyCode::Backspace | 
                KeyCode::Enter | 
                KeyCode::Esc |
                KeyCode::Left |
                KeyCode::Right
            )
        }
        _ => true, // Most keys are valid in other modes
    }
}

/// Get contextual status line text based on current state
pub fn get_status_line_text(current_tab: &Tab, current_mode: &AppMode, has_active_session: bool) -> String {
    match (current_mode, current_tab) {
        (AppMode::SessionActive, _) => {
            "🍅 Session Active | Space:Pause Enter:Complete Esc:Abandon".to_string()
        }
        (AppMode::Break, _) => {
            "☕ Break Time | Space:End s:Skip".to_string()
        }
        (AppMode::TaskInput, _) => {
            "✏️ Task Input | Enter:Save Esc:Cancel".to_string()
        }
        (AppMode::Normal, Tab::Tasks) => {
            "📋 Tasks | n:New Enter:Start Space:Toggle ↑↓:Navigate".to_string()
        }
        (AppMode::Normal, Tab::Timer) => {
            if has_active_session {
                "⏰ Timer | Space:Pause Enter:Complete".to_string()
            } else {
                "⏰ Timer | Enter:Start p:Pomodoro s:Short l:Long".to_string()
            }
        }
        (AppMode::Normal, Tab::Statistics) => {
            "📊 Statistics | ←→:Period o:Overview c:Charts t:Trends".to_string()
        }
        (AppMode::Normal, Tab::Settings) => {
            "⚙️ Settings | ↑↓:Navigate Enter:Edit R:Reset".to_string()
        }
        (AppMode::Normal, Tab::Integrations) => {
            "🔗 Integrations | g:GitHub s:Slack b:Blocking t:Test".to_string()
        }
        _ => "Pomo-TUI | F1:Help Ctrl+Q:Quit".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_global_shortcuts() {
        let key_quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        let result = handle_global_shortcuts(key_quit, &AppMode::Normal, &Tab::Tasks);
        assert!(matches!(result, Some(Message::Quit)));

        let key_tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
        let result = handle_global_shortcuts(key_tab, &AppMode::Normal, &Tab::Tasks);
        assert!(matches!(result, Some(Message::NextTab)));

        let key_1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        let result = handle_global_shortcuts(key_1, &AppMode::Normal, &Tab::Timer);
        assert!(matches!(result, Some(Message::SwitchToTab(Tab::Tasks))));
    }

    #[test]
    fn test_tasks_tab_shortcuts() {
        let key_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        let result = handle_tasks_tab(key_n, &None);
        assert!(matches!(result, Some(Message::NewTask)));

        let key_up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let result = handle_tasks_tab(key_up, &Some(5));
        assert!(matches!(result, Some(Message::SelectTask(4))));

        let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = handle_tasks_tab(key_enter, &Some(0));
        assert!(matches!(result, Some(Message::StartSession(Some(0)))));
    }

    #[test]
    fn test_timer_tab_shortcuts() {
        let key_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
        let result = handle_timer_tab(key_p, false);
        assert!(matches!(result, Some(Message::StartTimer(_))));

        let key_space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        let result = handle_timer_tab(key_space, true);
        assert!(matches!(result, Some(Message::PauseSession)));
    }

    #[test]
    fn test_help_text_generation() {
        let help = get_help_text(&Tab::Tasks, &AppMode::Normal);
        assert!(!help.is_empty());
        
        // Check that basic shortcuts are included
        assert!(help.iter().any(|(key, _)| key.contains("Quit")));
        assert!(help.iter().any(|(_, desc)| desc.contains("Navigate")));
    }

    #[test]
    fn test_status_line_text() {
        let status = get_status_line_text(&Tab::Tasks, &AppMode::Normal, false);
        assert!(status.contains("Tasks"));
        
        let status = get_status_line_text(&Tab::Timer, &AppMode::SessionActive, true);
        assert!(status.contains("Session Active"));
        
        let status = get_status_line_text(&Tab::Settings, &AppMode::TaskInput, false);
        assert!(status.contains("Task Input"));
    }

    #[test]
    fn test_valid_key_combinations() {
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(is_valid_key_combination(char_key, &AppMode::TaskInput));
        assert!(is_valid_key_combination(char_key, &AppMode::Normal));

        let escape_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(is_valid_key_combination(escape_key, &AppMode::TaskInput));
        assert!(is_valid_key_combination(escape_key, &AppMode::Normal));
    }
}
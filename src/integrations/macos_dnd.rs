use std::io;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DndState {
    Enabled,
    Disabled,
    Unknown,
}

#[derive(Debug, thiserror::Error)]
pub enum DndError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Platform not supported")]
    UnsupportedPlatform,
    #[error("Accessibility permissions required")]
    PermissionsRequired,
}

#[derive(Debug, Clone)]
pub enum DndMethod {
    Shortcuts, // Modern Focus mode via shortcuts command
    Defaults,  // Legacy DND via defaults
    KeyboardShortcut,
    AppleScript,
}

#[derive(Debug)]
pub struct MacOSDndController {
    preferred_method: DndMethod,
    original_state: Option<DndState>,
    enable_shortcut_name: String,
    disable_shortcut_name: String,
    last_known_state: Option<DndState>,
    shortcuts_available: Option<bool>,
}

impl Default for MacOSDndController {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOSDndController {
    pub fn new() -> Self {
        // Check for environment variable overrides
        let enable_shortcut = std::env::var("FOCUS_ENABLE_SHORTCUT")
            .unwrap_or_else(|_| "Set Focus".to_string());
        let disable_shortcut = std::env::var("FOCUS_DISABLE_SHORTCUT")
            .unwrap_or_else(|_| "Turn Off Focus".to_string());
            
        Self::with_shortcuts(&enable_shortcut, &disable_shortcut)
    }
    
    pub fn with_shortcuts(enable_shortcut: &str, disable_shortcut: &str) -> Self {
        // Detect best method based on macOS version and available tools
        let preferred_method = Self::detect_best_method();
        
        Self {
            preferred_method,
            original_state: None,
            enable_shortcut_name: enable_shortcut.to_string(),
            disable_shortcut_name: disable_shortcut.to_string(),
            last_known_state: None,
            shortcuts_available: None,
        }
    }
    
    /// Detect the best available method for controlling Focus/DND
    fn detect_best_method() -> DndMethod {
        // Check if shortcuts command is available (macOS Monterey+)
        if Command::new("/usr/bin/shortcuts")
            .arg("list")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        {
            DndMethod::Shortcuts
        } else {
            // Fall back to defaults method for older macOS
            DndMethod::Defaults
        }
    }
    
    /// Check if running on macOS
    pub fn is_supported() -> bool {
        cfg!(target_os = "macos")
    }

    /// Get current Do Not Disturb state
    pub fn get_state(&mut self) -> Result<DndState, DndError> {
        if !Self::is_supported() {
            return Err(DndError::UnsupportedPlatform);
        }

        // Check if shortcuts are available (cache the result)
        if self.shortcuts_available.is_none() {
            self.shortcuts_available = Some(
                self.check_shortcuts_exist()
                    .map(|(enable, disable)| enable && disable)
                    .unwrap_or(false)
            );
        }

        // If we have shortcuts and they're working, return last known state or default
        if self.shortcuts_available == Some(true) {
            // For shortcuts method, we can't reliably read state, so use memory
            return Ok(self.last_known_state.unwrap_or(DndState::Disabled));
        }

        // Try to read state from system preferences (fallback)
        match self.try_get_state_with_method(&DndMethod::Defaults) {
            Ok(state) => {
                self.last_known_state = Some(state);
                Ok(state)
            },
            Err(_) => {
                // If all else fails, use last known or unknown
                Ok(self.last_known_state.unwrap_or(DndState::Unknown))
            }
        }
    }

    /// Set Do Not Disturb state
    pub fn set_state(&mut self, state: DndState) -> Result<(), DndError> {
        if !Self::is_supported() {
            return Err(DndError::UnsupportedPlatform);
        }

        // Store original state if not already stored
        if self.original_state.is_none() {
            // Use a temporary value to avoid borrowing issues
            let current_state = if let Some(ref last) = self.last_known_state {
                *last
            } else {
                DndState::Unknown
            };
            self.original_state = Some(current_state);
        }

        // If preferred method is Shortcuts, validate they exist and update cache
        if matches!(self.preferred_method, DndMethod::Shortcuts) {
            // First refresh the cache to ensure we have current state
            if let Err(_) = self.refresh_shortcuts_cache() {
                // If cache refresh fails, shortcuts are likely not available
                return Err(DndError::CommandFailed(format!(
                    "Focus mode shortcuts not available.\n\n{}",
                    self.get_setup_instructions()
                )));
            }
            
            // Check cache state
            if self.shortcuts_available == Some(false) {
                return Err(DndError::CommandFailed(format!(
                    "Focus mode shortcuts not configured.\n\n{}",
                    self.get_setup_instructions()
                )));
            }
            
            // Double-check by querying shortcuts directly
            if let Ok((enable_exists, disable_exists)) = self.check_shortcuts_exist() {
                if !enable_exists || !disable_exists {
                    // Update cache to reflect current state
                    self.shortcuts_available = Some(false);
                    return Err(DndError::CommandFailed(format!(
                        "Focus mode shortcuts not configured.\n\n{}",
                        self.get_setup_instructions()
                    )));
                }
            }
        }

        // If preferred method is KeyboardShortcut, validate accessibility permissions
        if matches!(self.preferred_method, DndMethod::KeyboardShortcut) {
            if !self.check_accessibility_permissions() {
                return Err(DndError::PermissionsRequired);
            }
        }

        // Try preferred method first  
        let preferred_method = self.preferred_method.clone();
        let preferred_result = self.try_set_state_with_method(&preferred_method, state);
        if preferred_result.is_ok() {
            // Update our memory of the state when successful
            self.last_known_state = Some(state);
            return Ok(());
        }

        // Don't try fallback methods for Focus mode - they don't actually work
        // Return the error from the preferred method with setup instructions
        preferred_result
    }

    /// Enable Do Not Disturb
    pub fn enable(&mut self) -> Result<(), DndError> {
        self.set_state(DndState::Enabled)
    }

    /// Disable Do Not Disturb
    pub fn disable(&mut self) -> Result<(), DndError> {
        self.set_state(DndState::Disabled)
    }

    /// Toggle Do Not Disturb state
    pub fn toggle(&mut self) -> Result<DndState, DndError> {
        // Get current state (this updates last_known_state)
        let current_state = self.get_state()?;
        let new_state = match current_state {
            DndState::Enabled => DndState::Disabled,
            DndState::Disabled => DndState::Enabled,
            DndState::Unknown => DndState::Enabled, // Default to enabling if unknown
        };
        self.set_state(new_state)?;
        Ok(new_state)
    }

    /// Restore original Do Not Disturb state
    pub fn restore_original_state(&mut self) -> Result<(), DndError> {
        if let Some(original_state) = self.original_state {
            self.set_state(original_state)?;
            self.original_state = None;
        }
        Ok(())
    }

    /// Get the original state that was stored
    pub fn get_original_state(&self) -> Option<DndState> {
        self.original_state
    }

    fn try_get_state_with_method(&self, method: &DndMethod) -> Result<DndState, DndError> {
        match method {
            DndMethod::Shortcuts => self.get_state_shortcuts(),
            DndMethod::Defaults => self.get_state_defaults(),
            DndMethod::KeyboardShortcut => Ok(DndState::Unknown), // Can't read state with shortcuts
            DndMethod::AppleScript => self.get_state_applescript(),
        }
    }

    fn try_set_state_with_method(
        &mut self,
        method: &DndMethod,
        state: DndState,
    ) -> Result<(), DndError> {
        match method {
            DndMethod::Shortcuts => self.set_state_shortcuts(state),
            DndMethod::Defaults => self.set_state_defaults(state),
            DndMethod::KeyboardShortcut => self.set_state_keyboard_shortcut(),
            DndMethod::AppleScript => self.set_state_applescript(state),
        }
    }

    fn get_state_defaults(&self) -> Result<DndState, DndError> {
        let output = Command::new("defaults")
            .args([
                "-currentHost",
                "read",
                "com.apple.notificationcenterui",
                "doNotDisturb",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(DndState::Disabled); // Default to disabled if key doesn't exist
        }

        let output_bytes = String::from_utf8_lossy(&output.stdout);
        let output_str = output_bytes.trim();
        match output_str {
            "1" => Ok(DndState::Enabled),
            "0" => Ok(DndState::Disabled),
            _ => Ok(DndState::Unknown),
        }
    }

    fn set_state_defaults(&self, state: DndState) -> Result<(), DndError> {
        // Note: This method only changes preference files and doesn't actually
        // enable modern Focus mode. It's kept for backward compatibility only.
        let value = match state {
            DndState::Enabled => "TRUE",
            DndState::Disabled => "FALSE",
            DndState::Unknown => return Ok(()), // Don't set unknown state
        };

        let output = Command::new("defaults")
            .args([
                "-currentHost",
                "write",
                "com.apple.notificationcenterui",
                "doNotDisturb",
                "-bool",
                value,
            ])
            .output()?;

        if output.status.success() {
            // Even if the command succeeds, warn that this doesn't control modern Focus mode
            Err(DndError::CommandFailed(
                "Legacy DND preference changed but Focus mode requires shortcuts.\n\
                Please configure Focus mode shortcuts for proper functionality.".to_string()
            ))
        } else {
            // Sanitize stderr to prevent terminal escape sequences from interfering with TUI
            let stderr = String::from_utf8_lossy(&output.stderr)
                .trim()
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .collect::<String>();
            Err(DndError::CommandFailed(format!(
                "defaults command failed: {}",
                stderr
            )))
        }
    }

    fn set_state_keyboard_shortcut(&self) -> Result<(), DndError> {
        // The standard macOS keyboard shortcut for Focus/DND toggle is Cmd+Shift+Option+Control+D
        let script = r#"tell application "System Events" to keystroke "d" using {command down, shift down, option down, control down}"#;

        let output = Command::new("osascript").args(["-e", script]).output()?;

        if output.status.success() {
            Ok(())
        } else {
            // If accessibility permission is denied, provide a helpful error
            let stderr = String::from_utf8_lossy(&output.stderr)
                .trim()
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .collect::<String>();
            if stderr.contains("assistive access") {
                Err(DndError::PermissionsRequired)
            } else {
                Err(DndError::CommandFailed(format!(
                    "Focus mode keyboard shortcut failed: {}",
                    stderr
                )))
            }
        }
    }

    fn get_state_applescript(&self) -> Result<DndState, DndError> {
        // This is a placeholder for AppleScript state reading
        // Implementation would depend on macOS version and available APIs
        Ok(DndState::Unknown)
    }

    fn get_state_shortcuts(&self) -> Result<DndState, DndError> {
        // For shortcuts method, we can't reliably read the current state
        // This will be handled by memory tracking in get_state()
        Ok(DndState::Unknown)
    }

    fn set_state_shortcuts(&mut self, state: DndState) -> Result<(), DndError> {
        let shortcut_name = match state {
            DndState::Enabled => &self.enable_shortcut_name,
            DndState::Disabled => &self.disable_shortcut_name,
            DndState::Unknown => return Ok(()),
        };

        let output = Command::new("/usr/bin/shortcuts")
            .args(["run", shortcut_name])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            // Sanitize stderr to prevent terminal escape sequences from interfering with TUI
            let stderr = String::from_utf8_lossy(&output.stderr)
                .trim()
                .chars()
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .collect::<String>();
            if stderr.contains("not found") || stderr.contains("No shortcut") {
                // Shortcuts were removed - clear the cache
                self.shortcuts_available = Some(false);
                Err(DndError::CommandFailed(format!(
                    "Shortcut '{}' not found. Please create it in the Shortcuts app first.", 
                    shortcut_name
                )))
            } else {
                Err(DndError::CommandFailed(format!(
                    "Shortcut '{}' failed: {}",
                    shortcut_name, stderr
                )))
            }
        }
    }

    fn set_state_applescript(&self, _state: DndState) -> Result<(), DndError> {
        // This is a placeholder for AppleScript DND control
        // Implementation would depend on macOS version and available APIs
        Err(DndError::CommandFailed(
            "AppleScript method not implemented".to_string(),
        ))
    }

    /// Check if the system has accessibility permissions for AppleScript
    pub fn check_accessibility_permissions(&self) -> bool {
        // Try to execute a simple AppleScript to check permissions
        let script = r#"tell application "System Events" to get name"#;
        if let Ok(output) = Command::new("osascript").args(["-e", script]).output() {
            output.status.success()
        } else {
            false
        }
    }
    
    /// Check if the configured shortcuts exist
    pub fn check_shortcuts_exist(&self) -> Result<(bool, bool), DndError> {
        let output = Command::new("/usr/bin/shortcuts")
            .args(["list"])
            .output()?;
            
        if !output.status.success() {
            return Err(DndError::CommandFailed(
                "Failed to list shortcuts".to_string()
            ));
        }
        
        let shortcuts_list = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = shortcuts_list.lines().collect();
        let enable_exists = lines.iter().any(|line| line.trim() == self.enable_shortcut_name);
        let disable_exists = lines.iter().any(|line| line.trim() == self.disable_shortcut_name);
        
        Ok((enable_exists, disable_exists))
    }
    
    /// Force refresh the shortcuts availability cache
    pub fn refresh_shortcuts_cache(&mut self) -> Result<(), DndError> {
        match self.check_shortcuts_exist() {
            Ok((enable_exists, disable_exists)) => {
                self.shortcuts_available = Some(enable_exists && disable_exists);
                Ok(())
            }
            Err(e) => {
                // If we can't check, assume not available
                self.shortcuts_available = Some(false);
                Err(e)
            }
        }
    }
    
    /// Get setup instructions for creating required shortcuts
    pub fn get_setup_instructions(&self) -> String {
        format!(
            r#"To enable Focus mode control, create these shortcuts:

1. Open the Shortcuts app

2. Create shortcut: "{}"
   • Add action: "Set Focus" 
   • Choose Focus mode (Work/Do Not Disturb/etc.)
   • Set duration: "Until I turn it off"

3. Create shortcut: "{}"
   • Add action: "Turn Off Focus"

Once created, the app can control Focus automatically.

Note: You can name shortcuts anything, but they must contain
the exact actions "Set Focus" and "Turn Off Focus".

Alternative: Customize names with environment variables:
  FOCUS_ENABLE_SHORTCUT="Your Enable Name"
  FOCUS_DISABLE_SHORTCUT="Your Disable Name""#,
            self.enable_shortcut_name,
            self.disable_shortcut_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = MacOSDndController::new();
        assert_eq!(controller.original_state, None);
        // On modern macOS, preferred method should be Shortcuts if available
        assert!(matches!(controller.preferred_method, DndMethod::Shortcuts | DndMethod::Defaults));
    }

    #[test]
    fn test_is_supported() {
        // This will return true only on macOS
        let supported = MacOSDndController::is_supported();
        if cfg!(target_os = "macos") {
            assert!(supported);
        } else {
            assert!(!supported);
        }
    }

    #[test]
    fn test_unsupported_platform_error() {
        if !cfg!(target_os = "macos") {
            let mut controller = MacOSDndController::new();
            let result = controller.get_state();
            assert!(matches!(result, Err(DndError::UnsupportedPlatform)));
        }
    }

    #[test]
    fn test_state_transitions() {
        // Test state logic without actually executing commands
        let mut controller = MacOSDndController::new();

        // Test that original state tracking works conceptually
        assert_eq!(controller.get_original_state(), None);

        // Simulate storing original state
        controller.original_state = Some(DndState::Disabled);
        assert_eq!(controller.get_original_state(), Some(DndState::Disabled));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_state_on_macos() {
        let mut controller = MacOSDndController::new();
        // This test will only run on macOS
        // Don't assert specific state as it depends on current system state
        let _result = controller.get_state();
        // Just ensure it doesn't panic and returns a valid result type
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_accessibility_permissions() {
        let controller = MacOSDndController::new();
        // This test will only run on macOS
        let _has_permissions = controller.check_accessibility_permissions();
        // Don't assert specific value as it depends on system configuration
    }
}

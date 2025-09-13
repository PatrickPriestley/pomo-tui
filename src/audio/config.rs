//! Audio configuration and persistence

use super::types::{AudioError, NotificationStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Audio configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
    /// Whether audio is muted
    pub muted: bool,
    /// Style of notifications to play
    pub notification_style: NotificationStyle,
    /// Whether to play sound when starting a session
    pub play_session_start: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            volume: 0.7, // 70% volume by default
            muted: false,
            notification_style: NotificationStyle::Musical,
            play_session_start: false,
        }
    }
}

impl AudioConfig {
    /// Load configuration from disk, or return default if not found
    pub fn load() -> Result<Self, AudioError> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| AudioError::ConfigError(format!("Failed to read config: {}", e)))?;
            
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| AudioError::ConfigError(format!("Failed to parse config: {}", e)))?;
            
        Ok(config)
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), AudioError> {
        let config_path = Self::config_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AudioError::ConfigError(format!("Failed to create config dir: {}", e)))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| AudioError::ConfigError(format!("Failed to serialize config: {}", e)))?;
            
        fs::write(&config_path, content)
            .map_err(|e| AudioError::ConfigError(format!("Failed to write config: {}", e)))?;
            
        Ok(())
    }

    /// Get the path to the configuration file
    fn config_path() -> Result<PathBuf, AudioError> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| AudioError::ConfigError("No config directory found".to_string()))?;
        
        path.push("pomo-tui");
        path.push("audio.json");
        
        Ok(path)
    }

    /// Adjust volume by a delta amount
    pub fn adjust_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
    }

    /// Toggle mute state and return new state
    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.muted
    }

    /// Set notification style
    pub fn set_notification_style(&mut self, style: NotificationStyle) {
        self.notification_style = style;
    }

    /// Get volume as percentage (0-100)
    pub fn volume_percentage(&self) -> u8 {
        (self.volume * 100.0).round() as u8
    }
}
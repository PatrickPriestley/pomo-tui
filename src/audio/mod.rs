//! Audio notification system for pomo-tui
//!
//! Provides audio feedback for session transitions and timer events.
//! Uses rodio for cross-platform audio playback with generated tones.

mod config;
mod player;
mod sounds;
mod types;

pub use config::AudioConfig;
pub use player::AudioPlayer;
pub use sounds::ToneGenerator;
pub use types::{AudioError, NotificationStyle, SoundType};

use std::sync::{Arc, Mutex};

/// Main audio manager that coordinates all audio functionality
pub struct AudioManager {
    player: Arc<Mutex<Option<AudioPlayer>>>,
    config: AudioConfig,
}

impl AudioManager {
    /// Create a new audio manager with default configuration
    pub fn new() -> Result<Self, AudioError> {
        let config = AudioConfig::load().unwrap_or_default();
        let player = match AudioPlayer::new() {
            Ok(player) => Arc::new(Mutex::new(Some(player))),
            Err(_) => {
                // Audio initialization failed - continue without audio
                Arc::new(Mutex::new(None))
            }
        };

        Ok(Self { player, config })
    }

    /// Play a notification sound
    pub fn play_notification(&self, sound_type: SoundType) -> Result<(), AudioError> {
        if self.config.muted {
            return Ok(());
        }

        let mut player_guard = self
            .player
            .lock()
            .map_err(|_| AudioError::PlaybackFailed("Mutex lock failed".to_string()))?;
        if let Some(ref mut player) = *player_guard {
            player.play_sound(sound_type, self.config.volume)?;
        }
        Ok(())
    }

    /// Toggle mute state
    pub fn toggle_mute(&mut self) -> bool {
        self.config.muted = !self.config.muted;
        self.save_config();
        self.config.muted
    }

    /// Set volume level (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.config.volume = volume.clamp(0.0, 1.0);
        self.save_config();
    }

    /// Get current volume level
    pub fn volume(&self) -> f32 {
        self.config.volume
    }

    /// Check if audio is muted
    pub fn is_muted(&self) -> bool {
        self.config.muted
    }

    /// Check if audio system is available
    pub fn is_available(&self) -> bool {
        if let Ok(player_guard) = self.player.lock() {
            player_guard.is_some()
        } else {
            false
        }
    }

    /// Play a test sound for configuration
    pub fn play_test_sound(&self) -> Result<(), AudioError> {
        self.play_notification(SoundType::SessionComplete)
    }

    /// Save current configuration to disk
    fn save_config(&self) {
        let _ = self.config.save(); // Ignore errors in saving config
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            player: Arc::new(Mutex::new(None)),
            config: AudioConfig::default(),
        })
    }
}

//! Audio type definitions and enums

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Types of audio notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    /// Work session completed (25min → break)
    SessionComplete,
    /// Break completed (5min/15min → work)
    BreakComplete,
    /// Long break started (after 4 sessions)
    LongBreakStart,
    /// Work session started (optional)
    SessionStart,
    /// Test/preview sound
    Test,
}

/// Audio notification styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStyle {
    /// Simple single tones
    Simple,
    /// Musical chimes and chords
    Musical,
    /// Soft, ADHD-friendly tones
    Gentle,
}

impl Default for NotificationStyle {
    fn default() -> Self {
        NotificationStyle::Musical
    }
}

/// Audio-related errors
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Audio device not available")]
    DeviceUnavailable,
    
    #[error("Audio initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Playback failed: {0}")]
    PlaybackFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Audio frequency constants for tone generation
pub mod frequencies {
    /// Musical notes in Hz
    pub const C4: f32 = 261.63;
    pub const D4: f32 = 293.66;
    pub const E4: f32 = 329.63;
    pub const F4: f32 = 349.23;
    pub const G4: f32 = 392.00;
    pub const A4: f32 = 440.00;
    pub const B4: f32 = 493.88;
    pub const C5: f32 = 523.25;
    
    /// Common tone frequencies (reserved for future use)
    #[allow(dead_code)]
    pub const NOTIFICATION: f32 = A4;
    #[allow(dead_code)]
    pub const SUCCESS: f32 = C5;
    #[allow(dead_code)]
    pub const ATTENTION: f32 = G4;
}
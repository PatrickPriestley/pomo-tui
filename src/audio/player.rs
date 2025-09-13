//! Audio player implementation using rodio

use super::config::AudioConfig;
use super::sounds::ToneGenerator;
use super::types::{AudioError, NotificationStyle, SoundType};
use rodio::{OutputStream, OutputStreamHandle, Sink};

/// Audio player that manages audio output and playback
pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    current_sink: Option<Sink>,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new() -> Result<Self, AudioError> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::InitializationFailed(format!("Failed to create audio output: {}", e)))?;

        Ok(Self {
            _stream,
            stream_handle,
            current_sink: None,
        })
    }

    /// Play a sound with the specified volume
    pub fn play_sound(&mut self, sound_type: SoundType, volume: f32) -> Result<(), AudioError> {
        // Stop any currently playing sound
        self.stop_current_sound();

        // Create a new sink for this sound
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackFailed(format!("Failed to create sink: {}", e)))?;

        // Set volume
        sink.set_volume(volume);

        // Generate the appropriate sound based on type
        let notification_style = NotificationStyle::Musical; // Default for now, will be configurable
        let source = ToneGenerator::notification_sound(sound_type, notification_style, 44100);

        // Add the source to the sink and play
        sink.append(source);
        sink.detach(); // Let the sink play independently

        Ok(())
    }

    /// Play a sound with configuration from AudioConfig
    pub fn play_with_config(
        &mut self,
        sound_type: SoundType,
        config: &AudioConfig,
    ) -> Result<(), AudioError> {
        if config.muted {
            return Ok(());
        }

        // Stop any currently playing sound
        self.stop_current_sound();

        // Create a new sink for this sound
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackFailed(format!("Failed to create sink: {}", e)))?;

        // Set volume
        sink.set_volume(config.volume);

        // Generate the appropriate sound based on type and style
        let source = ToneGenerator::notification_sound(sound_type, config.notification_style, 44100);

        // Add the source to the sink and play
        sink.append(source);
        
        // Store the sink to allow stopping if needed
        self.current_sink = Some(sink);

        Ok(())
    }

    /// Stop any currently playing sound
    pub fn stop_current_sound(&mut self) {
        if let Some(sink) = self.current_sink.take() {
            sink.stop();
        }
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        if let Some(ref sink) = self.current_sink {
            !sink.empty()
        } else {
            false
        }
    }

    /// Test if the audio system is working by playing a brief test tone
    pub fn test_audio(&mut self) -> Result<(), AudioError> {
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackFailed(format!("Audio test failed: {}", e)))?;

        // Generate a brief test tone
        let source = ToneGenerator::notification_sound(SoundType::Test, NotificationStyle::Simple, 44100);
        
        sink.set_volume(0.5); // Medium volume for test
        sink.append(source);
        sink.detach();

        Ok(())
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.stop_current_sound();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_creation() {
        // This test might fail in CI environments without audio
        match AudioPlayer::new() {
            Ok(_) => println!("Audio player created successfully"),
            Err(e) => println!("Audio player creation failed (expected in CI): {}", e),
        }
    }

    #[test]
    fn test_sound_generation() {
        if let Ok(mut player) = AudioPlayer::new() {
            // Test playing different sound types
            let _ = player.play_sound(SoundType::Test, 0.5);
            
            // Give some time for the sound to play
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
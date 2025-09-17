//! Sound generation and tone synthesis

use super::types::{frequencies, NotificationStyle, SoundType};
use rodio::source::Source;
use std::f32::consts::PI;
use std::time::Duration;

/// Generates audio tones and waveforms
pub struct ToneGenerator;

impl ToneGenerator {
    /// Generate a sine wave tone
    pub fn sine_wave(frequency: f32, duration: Duration, sample_rate: u32) -> SineWave {
        SineWave::new(frequency, duration, sample_rate)
    }

    /// Generate a notification sound based on type and style
    pub fn notification_sound(
        sound_type: SoundType,
        style: NotificationStyle,
        sample_rate: u32,
    ) -> Box<dyn Source<Item = f32> + Send> {
        match (sound_type, style) {
            // Session Complete sounds
            (SoundType::SessionComplete, NotificationStyle::Simple) => Box::new(Self::sine_wave(
                frequencies::A4,
                Duration::from_millis(500),
                sample_rate,
            )),
            (SoundType::SessionComplete, NotificationStyle::Musical) => {
                Box::new(Self::create_chord(
                    &[frequencies::C4, frequencies::E4],
                    Duration::from_millis(800),
                    sample_rate,
                ))
            }
            (SoundType::SessionComplete, NotificationStyle::Gentle) => Box::new(Self::sine_wave(
                frequencies::G4,
                Duration::from_millis(600),
                sample_rate,
            )),

            // Break Complete sounds
            (SoundType::BreakComplete, NotificationStyle::Simple) => Box::new(Self::sine_wave(
                frequencies::C5,
                Duration::from_millis(400),
                sample_rate,
            )),
            (SoundType::BreakComplete, NotificationStyle::Musical) => {
                Box::new(Self::create_sequence(
                    &[frequencies::C4, frequencies::G4],
                    Duration::from_millis(300),
                    sample_rate,
                ))
            }
            (SoundType::BreakComplete, NotificationStyle::Gentle) => Box::new(Self::sine_wave(
                frequencies::F4,
                Duration::from_millis(400),
                sample_rate,
            )),

            // Long Break Start sounds
            (SoundType::LongBreakStart, NotificationStyle::Simple) => Box::new(Self::sine_wave(
                frequencies::E4,
                Duration::from_millis(700),
                sample_rate,
            )),
            (SoundType::LongBreakStart, NotificationStyle::Musical) => {
                Box::new(Self::create_chord(
                    &[frequencies::C4, frequencies::E4, frequencies::G4],
                    Duration::from_millis(1000),
                    sample_rate,
                ))
            }
            (SoundType::LongBreakStart, NotificationStyle::Gentle) => Box::new(Self::sine_wave(
                frequencies::D4,
                Duration::from_millis(800),
                sample_rate,
            )),

            // Session Start sounds
            (SoundType::SessionStart, NotificationStyle::Simple) => Box::new(Self::sine_wave(
                frequencies::B4,
                Duration::from_millis(200),
                sample_rate,
            )),
            (SoundType::SessionStart, NotificationStyle::Musical) => Box::new(Self::sine_wave(
                frequencies::C5,
                Duration::from_millis(250),
                sample_rate,
            )),
            (SoundType::SessionStart, NotificationStyle::Gentle) => Box::new(Self::sine_wave(
                frequencies::A4,
                Duration::from_millis(300),
                sample_rate,
            )),

            // Break End sounds (gentle and soothing)
            (SoundType::BreakEnd, NotificationStyle::Simple) => Box::new(Self::sine_wave(
                frequencies::E4,
                Duration::from_millis(600),
                sample_rate,
            )),
            (SoundType::BreakEnd, NotificationStyle::Musical) => {
                Box::new(Self::create_chord(
                    &[frequencies::C4, frequencies::E4],
                    Duration::from_millis(800),
                    sample_rate,
                ))
            }
            (SoundType::BreakEnd, NotificationStyle::Gentle) => Box::new(Self::sine_wave(
                frequencies::F4,
                Duration::from_millis(700),
                sample_rate,
            )),

            // Test sound
            (SoundType::Test, _) => Box::new(Self::sine_wave(
                frequencies::A4,
                Duration::from_millis(500),
                sample_rate,
            )),
        }
    }

    /// Create a chord by mixing multiple frequencies
    fn create_chord(
        frequencies: &[f32],
        duration: Duration,
        sample_rate: u32,
    ) -> impl Source<Item = f32> + Send {
        let waves: Vec<SineWave> = frequencies
            .iter()
            .map(|&freq| SineWave::new(freq, duration, sample_rate))
            .collect();

        ChordSource::new(waves)
    }

    /// Create a sequence of tones
    fn create_sequence(
        frequencies: &[f32],
        note_duration: Duration,
        sample_rate: u32,
    ) -> impl Source<Item = f32> + Send {
        let waves: Vec<SineWave> = frequencies
            .iter()
            .map(|&freq| SineWave::new(freq, note_duration, sample_rate))
            .collect();

        SequenceSource::new(waves)
    }
}

/// A sine wave audio source
pub struct SineWave {
    frequency: f32,
    sample_rate: u32,
    current_sample: usize,
    total_samples: usize,
}

impl SineWave {
    pub fn new(frequency: f32, duration: Duration, sample_rate: u32) -> Self {
        let total_samples = (duration.as_secs_f32() * sample_rate as f32) as usize;
        Self {
            frequency,
            sample_rate,
            current_sample: 0,
            total_samples,
        }
    }
}

impl Iterator for SineWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.total_samples {
            return None;
        }

        let t = self.current_sample as f32 / self.sample_rate as f32;
        let sample = (t * self.frequency * 2.0 * PI).sin() * 0.3; // 30% amplitude
        self.current_sample += 1;

        Some(sample)
    }
}

impl Source for SineWave {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.total_samples - self.current_sample)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.total_samples as f32 / self.sample_rate as f32,
        ))
    }
}

/// Source that plays multiple sine waves as a chord
struct ChordSource {
    waves: Vec<SineWave>,
    finished: bool,
}

impl ChordSource {
    fn new(waves: Vec<SineWave>) -> Self {
        Self {
            waves,
            finished: false,
        }
    }
}

impl Iterator for ChordSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let mut sample = 0.0;
        let mut active_waves = 0;

        for wave in &mut self.waves {
            if let Some(wave_sample) = wave.next() {
                sample += wave_sample;
                active_waves += 1;
            }
        }

        if active_waves == 0 {
            self.finished = true;
            return None;
        }

        // Average the samples to prevent clipping
        Some(sample / active_waves as f32)
    }
}

impl Source for ChordSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.waves
            .iter()
            .filter_map(|w| w.current_frame_len())
            .max()
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        self.waves.iter().filter_map(|w| w.total_duration()).max()
    }
}

/// Source that plays sine waves in sequence
struct SequenceSource {
    waves: Vec<SineWave>,
    current_wave: usize,
}

impl SequenceSource {
    fn new(waves: Vec<SineWave>) -> Self {
        Self {
            waves,
            current_wave: 0,
        }
    }
}

impl Iterator for SequenceSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_wave < self.waves.len() {
            if let Some(sample) = self.waves[self.current_wave].next() {
                return Some(sample);
            } else {
                self.current_wave += 1;
            }
        }
        None
    }
}

impl Source for SequenceSource {
    fn current_frame_len(&self) -> Option<usize> {
        if self.current_wave < self.waves.len() {
            self.waves[self.current_wave].current_frame_len()
        } else {
            None
        }
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<Duration> {
        self.waves
            .iter()
            .filter_map(|w| w.total_duration())
            .reduce(|acc, duration| acc + duration)
    }
}

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreathingPattern {
    Box,            // 4-4-4-4 breathing
    FourSevenEight, // 4-7-8 breathing
    Simple,         // 4-4 breathing
}

#[derive(Debug, Clone)]
pub struct BreathingExercise {
    pattern: BreathingPattern,
    current_phase: BreathPhase,
    phase_elapsed: Duration,
    total_elapsed: Duration,
    cycle_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreathPhase {
    Inhale,
    Hold,
    Exhale,
    Rest,
}

impl BreathingExercise {
    pub fn new(pattern: BreathingPattern) -> Self {
        Self {
            pattern,
            current_phase: BreathPhase::Inhale,
            phase_elapsed: Duration::ZERO,
            total_elapsed: Duration::ZERO,
            cycle_count: 0,
        }
    }

    pub fn update(&mut self, delta: Duration) {
        self.phase_elapsed += delta;
        self.total_elapsed += delta;

        let phase_duration = self.get_phase_duration();

        if self.phase_elapsed >= phase_duration {
            self.advance_phase();
        }
    }

    fn get_phase_duration(&self) -> Duration {
        match self.pattern {
            BreathingPattern::Box => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(4),
                BreathPhase::Hold => Duration::from_secs(4),
                BreathPhase::Exhale => Duration::from_secs(4),
                BreathPhase::Rest => Duration::from_secs(4),
            },
            BreathingPattern::FourSevenEight => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(4),
                BreathPhase::Hold => Duration::from_secs(7),
                BreathPhase::Exhale => Duration::from_secs(8),
                BreathPhase::Rest => Duration::ZERO,
            },
            BreathingPattern::Simple => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(4),
                BreathPhase::Hold => Duration::ZERO,
                BreathPhase::Exhale => Duration::from_secs(4),
                BreathPhase::Rest => Duration::ZERO,
            },
        }
    }

    fn advance_phase(&mut self) {
        self.phase_elapsed = Duration::ZERO;

        self.current_phase = match self.pattern {
            BreathingPattern::Box => match self.current_phase {
                BreathPhase::Inhale => BreathPhase::Hold,
                BreathPhase::Hold => BreathPhase::Exhale,
                BreathPhase::Exhale => BreathPhase::Rest,
                BreathPhase::Rest => {
                    self.cycle_count += 1;
                    BreathPhase::Inhale
                }
            },
            BreathingPattern::FourSevenEight => match self.current_phase {
                BreathPhase::Inhale => BreathPhase::Hold,
                BreathPhase::Hold => BreathPhase::Exhale,
                BreathPhase::Exhale | BreathPhase::Rest => {
                    self.cycle_count += 1;
                    BreathPhase::Inhale
                }
            },
            BreathingPattern::Simple => match self.current_phase {
                BreathPhase::Inhale => BreathPhase::Exhale,
                BreathPhase::Exhale | BreathPhase::Hold | BreathPhase::Rest => {
                    self.cycle_count += 1;
                    BreathPhase::Inhale
                }
            },
        };
    }

    pub fn get_phase_progress(&self) -> f64 {
        let duration = self.get_phase_duration();
        if duration.as_secs() == 0 {
            return 1.0;
        }
        self.phase_elapsed.as_secs_f64() / duration.as_secs_f64()
    }

    pub fn get_instruction(&self) -> &str {
        match self.current_phase {
            BreathPhase::Inhale => "Breathe In",
            BreathPhase::Hold => "Hold",
            BreathPhase::Exhale => "Breathe Out",
            BreathPhase::Rest => "Rest",
        }
    }

    pub fn get_pattern(&self) -> BreathingPattern {
        self.pattern
    }

    pub fn get_pattern_name(&self) -> &str {
        match self.pattern {
            BreathingPattern::Box => "Box Breathing (4-4-4-4)",
            BreathingPattern::FourSevenEight => "4-7-8 Breathing",
            BreathingPattern::Simple => "Simple Breathing (4-4)",
        }
    }

    pub fn get_cycle_count(&self) -> u32 {
        self.cycle_count
    }

    pub fn get_remaining_in_phase(&self) -> Duration {
        let duration = self.get_phase_duration();
        duration.saturating_sub(self.phase_elapsed)
    }

    pub fn reset(&mut self) {
        self.current_phase = BreathPhase::Inhale;
        self.phase_elapsed = Duration::ZERO;
        self.total_elapsed = Duration::ZERO;
        self.cycle_count = 0;
    }
}

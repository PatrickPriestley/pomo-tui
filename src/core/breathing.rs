use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreathingPattern {
    ExtendedExhale, // 3-0-6-0 breathing (ADHD-friendly)
    Coherent,       // 5-0-5-0 breathing (gentle continuous)
    ShortBox,       // 3-3-3-3 breathing (less intense)
    Simple,         // 4-0-4-0 breathing
}

#[derive(Debug, Clone)]
pub struct BreathingExercise {
    pattern: BreathingPattern,
    current_phase: BreathPhase,
    phase_elapsed: Duration,
    total_elapsed: Duration,
    cycle_count: u32,
    post_exhale_transition: bool, // Track if we're in post-exhale transition
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreathPhase {
    Inhale,
    Hold,
    Exhale,
    Rest,
    Transition, // Brief pause between inhale and exhale
}

impl BreathingExercise {
    pub fn new(pattern: BreathingPattern) -> Self {
        Self {
            pattern,
            current_phase: BreathPhase::Inhale,
            phase_elapsed: Duration::ZERO,
            total_elapsed: Duration::ZERO,
            cycle_count: 0,
            post_exhale_transition: false,
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
            BreathingPattern::ExtendedExhale => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(3),
                BreathPhase::Hold => Duration::ZERO,
                BreathPhase::Exhale => Duration::from_secs(6),
                BreathPhase::Rest => Duration::ZERO,
                BreathPhase::Transition => Duration::from_millis(1200), // 1.2 second pause
            },
            BreathingPattern::Coherent => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(5),
                BreathPhase::Hold => Duration::ZERO,
                BreathPhase::Exhale => Duration::from_secs(5),
                BreathPhase::Rest => Duration::ZERO,
                BreathPhase::Transition => Duration::from_millis(1000), // 1.0 second pause
            },
            BreathingPattern::ShortBox => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(3),
                BreathPhase::Hold => Duration::from_secs(3),
                BreathPhase::Exhale => Duration::from_secs(3),
                BreathPhase::Rest => Duration::from_secs(3),
                BreathPhase::Transition => Duration::ZERO, // Box breathing has holds instead
            },
            BreathingPattern::Simple => match self.current_phase {
                BreathPhase::Inhale => Duration::from_secs(4),
                BreathPhase::Hold => Duration::ZERO,
                BreathPhase::Exhale => Duration::from_secs(4),
                BreathPhase::Rest => Duration::ZERO,
                BreathPhase::Transition => Duration::from_millis(1100), // 1.1 second pause
            },
        }
    }

    fn advance_phase(&mut self) {
        self.phase_elapsed = Duration::ZERO;

        self.current_phase = match self.pattern {
            BreathingPattern::ExtendedExhale => match self.current_phase {
                BreathPhase::Inhale => {
                    self.post_exhale_transition = false;
                    BreathPhase::Transition
                }
                BreathPhase::Transition => {
                    if self.post_exhale_transition {
                        self.cycle_count += 1;
                        self.post_exhale_transition = false;
                        BreathPhase::Inhale
                    } else {
                        BreathPhase::Exhale
                    }
                }
                BreathPhase::Exhale => {
                    self.post_exhale_transition = true;
                    BreathPhase::Transition
                }
                BreathPhase::Hold | BreathPhase::Rest => {
                    self.cycle_count += 1;
                    BreathPhase::Inhale
                }
            },
            BreathingPattern::Coherent => match self.current_phase {
                BreathPhase::Inhale => {
                    self.post_exhale_transition = false;
                    BreathPhase::Transition
                }
                BreathPhase::Transition => {
                    if self.post_exhale_transition {
                        self.cycle_count += 1;
                        self.post_exhale_transition = false;
                        BreathPhase::Inhale
                    } else {
                        BreathPhase::Exhale
                    }
                }
                BreathPhase::Exhale => {
                    self.post_exhale_transition = true;
                    BreathPhase::Transition
                }
                BreathPhase::Hold | BreathPhase::Rest => {
                    self.cycle_count += 1;
                    BreathPhase::Inhale
                }
            },
            BreathingPattern::ShortBox => match self.current_phase {
                BreathPhase::Inhale => BreathPhase::Hold,
                BreathPhase::Hold => BreathPhase::Exhale,
                BreathPhase::Exhale => BreathPhase::Rest,
                BreathPhase::Rest => {
                    self.cycle_count += 1;
                    self.post_exhale_transition = false; // Reset for consistency
                    BreathPhase::Inhale
                }
                BreathPhase::Transition => BreathPhase::Exhale, // Shouldn't happen, but handle gracefully
            },
            BreathingPattern::Simple => match self.current_phase {
                BreathPhase::Inhale => {
                    self.post_exhale_transition = false;
                    BreathPhase::Transition
                }
                BreathPhase::Transition => {
                    if self.post_exhale_transition {
                        self.cycle_count += 1;
                        self.post_exhale_transition = false;
                        BreathPhase::Inhale
                    } else {
                        BreathPhase::Exhale
                    }
                }
                BreathPhase::Exhale => {
                    self.post_exhale_transition = true;
                    BreathPhase::Transition
                }
                BreathPhase::Hold | BreathPhase::Rest => {
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
            BreathPhase::Transition => "...",
        }
    }

    pub fn get_pattern(&self) -> BreathingPattern {
        self.pattern
    }

    pub fn get_pattern_name(&self) -> &str {
        match self.pattern {
            BreathingPattern::ExtendedExhale => "Extended Exhale (3-6)",
            BreathingPattern::Coherent => "Coherent Breathing (5-5)",
            BreathingPattern::ShortBox => "Short Box (3-3-3-3)",
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
        self.post_exhale_transition = false;
    }

    pub fn get_current_phase(&self) -> BreathPhase {
        self.current_phase
    }

    pub fn get_total_elapsed(&self) -> Duration {
        self.total_elapsed
    }

    pub fn is_post_exhale_transition(&self) -> bool {
        self.post_exhale_transition
    }
}

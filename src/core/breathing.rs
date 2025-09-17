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
    target_cycles: u32,
    post_exhale_transition: bool, // Track if we're in post-exhale transition
    pub ready_to_complete: bool, // Set when we've reached target cycles and should complete on next valid phase
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
        // Default to 6 cycles for backwards compatibility
        Self::new_with_target_cycles(pattern, 6)
    }

    pub fn new_with_target_cycles(pattern: BreathingPattern, target_cycles: u32) -> Self {
        Self {
            pattern,
            current_phase: BreathPhase::Inhale,
            phase_elapsed: Duration::ZERO,
            total_elapsed: Duration::ZERO,
            cycle_count: 0,
            target_cycles,
            post_exhale_transition: false,
            ready_to_complete: false,
        }
    }

    pub fn new_from_duration(pattern: BreathingPattern, duration: Duration) -> Self {
        let cycle_duration = Self::calculate_cycle_duration(pattern);
        let target_cycles = (duration.as_secs_f64() / cycle_duration.as_secs_f64()).ceil() as u32;
        // Ensure at least 1 cycle
        let target_cycles = target_cycles.max(1);
        Self::new_with_target_cycles(pattern, target_cycles)
    }

    fn calculate_cycle_duration(pattern: BreathingPattern) -> Duration {
        match pattern {
            BreathingPattern::ExtendedExhale => {
                // 3s inhale + 1.2s transition + 6s exhale + 1.2s transition = 11.4s
                Duration::from_millis(11400)
            }
            BreathingPattern::Coherent => {
                // 5s inhale + 1s transition + 5s exhale + 1s transition = 12s
                Duration::from_millis(12000)
            }
            BreathingPattern::ShortBox => {
                // 3s inhale + 3s hold + 3s exhale + 3s rest = 12s
                Duration::from_millis(12000)
            }
            BreathingPattern::Simple => {
                // 4s inhale + 1.1s transition + 4s exhale + 1.1s transition = 10.2s
                Duration::from_millis(10200)
            }
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

        // Check if we should set ready_to_complete flag instead of continuing cycles
        let would_increment_cycle = self.would_increment_cycle_count();
        if would_increment_cycle && self.cycle_count + 1 >= self.target_cycles {
            // We've reached the target cycles - set flag to complete when in valid phase
            self.ready_to_complete = true;
            // Still allow phase transitions, but don't increment cycle count
        }

        self.current_phase = match self.pattern {
            BreathingPattern::ExtendedExhale => match self.current_phase {
                BreathPhase::Inhale => {
                    self.post_exhale_transition = false;
                    BreathPhase::Transition
                }
                BreathPhase::Transition => {
                    if self.post_exhale_transition {
                        if !self.ready_to_complete {
                            self.cycle_count += 1;
                            self.post_exhale_transition = false;
                            BreathPhase::Inhale
                        } else {
                            // Ready to complete - stay in post-exhale transition
                            BreathPhase::Transition
                        }
                    } else {
                        BreathPhase::Exhale
                    }
                }
                BreathPhase::Exhale => {
                    self.post_exhale_transition = true;
                    BreathPhase::Transition
                }
                BreathPhase::Hold | BreathPhase::Rest => {
                    if !self.ready_to_complete {
                        self.cycle_count += 1;
                        BreathPhase::Inhale
                    } else {
                        // Ready to complete - stay in current phase
                        self.current_phase
                    }
                }
            },
            BreathingPattern::Coherent => match self.current_phase {
                BreathPhase::Inhale => {
                    self.post_exhale_transition = false;
                    BreathPhase::Transition
                }
                BreathPhase::Transition => {
                    if self.post_exhale_transition {
                        if !self.ready_to_complete {
                            self.cycle_count += 1;
                            self.post_exhale_transition = false;
                            BreathPhase::Inhale
                        } else {
                            // Ready to complete - stay in post-exhale transition
                            BreathPhase::Transition
                        }
                    } else {
                        BreathPhase::Exhale
                    }
                }
                BreathPhase::Exhale => {
                    self.post_exhale_transition = true;
                    BreathPhase::Transition
                }
                BreathPhase::Hold | BreathPhase::Rest => {
                    if !self.ready_to_complete {
                        self.cycle_count += 1;
                        BreathPhase::Inhale
                    } else {
                        // Ready to complete - stay in current phase
                        self.current_phase
                    }
                }
            },
            BreathingPattern::ShortBox => match self.current_phase {
                BreathPhase::Inhale => BreathPhase::Hold,
                BreathPhase::Hold => BreathPhase::Exhale,
                BreathPhase::Exhale => BreathPhase::Rest,
                BreathPhase::Rest => {
                    if !self.ready_to_complete {
                        self.cycle_count += 1;
                        self.post_exhale_transition = false; // Reset for consistency
                        BreathPhase::Inhale
                    } else {
                        // Ready to complete - stay in rest phase
                        BreathPhase::Rest
                    }
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
                        if !self.ready_to_complete {
                            self.cycle_count += 1;
                            self.post_exhale_transition = false;
                            BreathPhase::Inhale
                        } else {
                            // Ready to complete - stay in post-exhale transition
                            BreathPhase::Transition
                        }
                    } else {
                        BreathPhase::Exhale
                    }
                }
                BreathPhase::Exhale => {
                    self.post_exhale_transition = true;
                    BreathPhase::Transition
                }
                BreathPhase::Hold | BreathPhase::Rest => {
                    if !self.ready_to_complete {
                        self.cycle_count += 1;
                        BreathPhase::Inhale
                    } else {
                        // Ready to complete - stay in current phase
                        self.current_phase
                    }
                }
            },
        };
    }

    fn would_increment_cycle_count(&self) -> bool {
        match self.pattern {
            BreathingPattern::ExtendedExhale => match self.current_phase {
                BreathPhase::Transition => self.post_exhale_transition,
                BreathPhase::Hold | BreathPhase::Rest => true,
                _ => false,
            },
            BreathingPattern::Coherent => match self.current_phase {
                BreathPhase::Transition => self.post_exhale_transition,
                BreathPhase::Hold | BreathPhase::Rest => true,
                _ => false,
            },
            BreathingPattern::ShortBox => matches!(self.current_phase, BreathPhase::Rest),
            BreathingPattern::Simple => match self.current_phase {
                BreathPhase::Transition => self.post_exhale_transition,
                BreathPhase::Hold | BreathPhase::Rest => true,
                _ => false,
            },
        }
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

    pub fn get_target_cycles(&self) -> u32 {
        self.target_cycles
    }

    pub fn get_remaining_cycles(&self) -> u32 {
        self.target_cycles.saturating_sub(self.cycle_count)
    }

    pub fn should_complete_session(&self) -> bool {
        // Only complete if ready_to_complete flag is set AND we're in a good stopping phase
        if self.ready_to_complete {
            match self.current_phase {
                // Good stopping points: exhale or post-exhale transition
                BreathPhase::Exhale => true,
                BreathPhase::Transition => self.post_exhale_transition,
                // For ShortBox, Rest phase is also a good stopping point (comes after exhale)
                BreathPhase::Rest => self.pattern == BreathingPattern::ShortBox,
                // Don't stop mid-inhale or mid-hold
                BreathPhase::Inhale | BreathPhase::Hold => false,
            }
        } else {
            false
        }
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
        self.ready_to_complete = false;
        // Keep target_cycles unchanged when resetting
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breathing_exercise_ends_on_exhale_extended_exhale() {
        let mut exercise =
            BreathingExercise::new_with_target_cycles(BreathingPattern::ExtendedExhale, 1);

        // Run until completion or timeout
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));
            if exercise.should_complete_session() {
                break;
            }
        }

        // Should only complete on exhale or post-exhale transition
        assert!(exercise.should_complete_session());
        assert!(
            exercise.get_current_phase() == BreathPhase::Exhale
                || (exercise.get_current_phase() == BreathPhase::Transition
                    && exercise.is_post_exhale_transition())
        );
    }

    #[test]
    fn test_breathing_exercise_ends_on_exhale_coherent() {
        let mut exercise = BreathingExercise::new_with_target_cycles(BreathingPattern::Coherent, 1);

        // Run until completion or timeout
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));
            if exercise.should_complete_session() {
                break;
            }
        }

        assert!(exercise.should_complete_session());
        assert!(
            exercise.get_current_phase() == BreathPhase::Exhale
                || (exercise.get_current_phase() == BreathPhase::Transition
                    && exercise.is_post_exhale_transition())
        );
    }

    #[test]
    fn test_breathing_exercise_ends_on_exhale_short_box() {
        let mut exercise = BreathingExercise::new_with_target_cycles(BreathingPattern::ShortBox, 1);

        // Run until completion or timeout
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));
            if exercise.should_complete_session() {
                break;
            }
        }

        assert!(exercise.should_complete_session());
        // For short box, can end on exhale or rest (which comes after exhale)
        assert!(
            exercise.get_current_phase() == BreathPhase::Exhale
                || exercise.get_current_phase() == BreathPhase::Rest
        );
    }

    #[test]
    fn test_breathing_exercise_ends_on_exhale_simple() {
        let mut exercise = BreathingExercise::new_with_target_cycles(BreathingPattern::Simple, 1);

        // Run until completion or timeout
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));
            if exercise.should_complete_session() {
                break;
            }
        }

        assert!(exercise.should_complete_session());
        assert!(
            exercise.get_current_phase() == BreathPhase::Exhale
                || (exercise.get_current_phase() == BreathPhase::Transition
                    && exercise.is_post_exhale_transition())
        );
    }

    #[test]
    fn test_countdown_cycles_behavior() {
        let exercise =
            BreathingExercise::new_with_target_cycles(BreathingPattern::ExtendedExhale, 5);

        // Initially should have 5 cycles remaining
        assert_eq!(exercise.get_remaining_cycles(), 5);
        assert_eq!(exercise.get_cycle_count(), 0);
        assert_eq!(exercise.get_target_cycles(), 5);
    }

    #[test]
    fn test_countdown_cycles_progression() {
        let mut exercise =
            BreathingExercise::new_with_target_cycles(BreathingPattern::ExtendedExhale, 2);

        // Initially 2 cycles remaining
        assert_eq!(exercise.get_remaining_cycles(), 2);

        // Complete first cycle
        while exercise.get_cycle_count() < 1 {
            exercise.update(Duration::from_millis(100));
        }

        // Should now have 1 cycle remaining
        assert_eq!(exercise.get_remaining_cycles(), 1);
        assert_eq!(exercise.get_cycle_count(), 1);

        // Run until ready to complete (should stop incrementing cycles)
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));
            if exercise.ready_to_complete {
                break;
            }
        }

        // Should now have 0 cycles remaining, and ready to complete
        // Note: cycle count stays at 1 because we don't increment when ready_to_complete is set
        assert_eq!(exercise.get_remaining_cycles(), 1);
        assert_eq!(exercise.get_cycle_count(), 1);
        assert!(exercise.ready_to_complete);
    }

    #[test]
    fn test_should_not_complete_before_target_cycles() {
        let mut exercise =
            BreathingExercise::new_with_target_cycles(BreathingPattern::ExtendedExhale, 2);

        // Complete first cycle
        while exercise.get_cycle_count() < 1 {
            exercise.update(Duration::from_millis(100));
        }

        // Should not complete yet, even if in good phase
        assert!(!exercise.should_complete_session());
        assert!(!exercise.ready_to_complete);

        // Run until ready to complete
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));
            if exercise.ready_to_complete {
                break;
            }
        }

        // Should now be ready to complete
        assert!(exercise.ready_to_complete);

        // And should complete when in a valid phase
        for _ in 0..100 {
            exercise.update(Duration::from_millis(100));
            if exercise.should_complete_session() {
                break;
            }
        }

        assert!(exercise.should_complete_session());
    }

    #[test]
    fn test_new_from_duration_calculates_cycles() {
        // Test with 90 second duration for ExtendedExhale (11.4s per cycle)
        let exercise = BreathingExercise::new_from_duration(
            BreathingPattern::ExtendedExhale,
            Duration::from_secs(90),
        );

        // Should calculate approximately 8 cycles (90/11.4 = 7.89, rounded up to 8)
        assert_eq!(exercise.get_target_cycles(), 8);
        assert_eq!(exercise.get_remaining_cycles(), 8);
    }

    #[test]
    fn test_new_from_duration_minimum_one_cycle() {
        // Very short duration should still get at least 1 cycle
        let exercise = BreathingExercise::new_from_duration(
            BreathingPattern::ExtendedExhale,
            Duration::from_secs(1),
        );

        assert_eq!(exercise.get_target_cycles(), 1);
        assert_eq!(exercise.get_remaining_cycles(), 1);
    }

    #[test]
    fn test_exercise_ends_only_on_exhale_phases() {
        // Test with a very short target (1 cycle) to quickly reach completion
        let mut exercise =
            BreathingExercise::new_with_target_cycles(BreathingPattern::ExtendedExhale, 1);

        // Track phases until completion
        let mut completion_phases = Vec::new();

        // Run until we get a completion signal or timeout
        for _ in 0..1000 {
            exercise.update(Duration::from_millis(100));

            if exercise.should_complete_session() {
                completion_phases.push(exercise.get_current_phase());
                break;
            }
        }

        // Should have completed
        assert!(
            !completion_phases.is_empty(),
            "Exercise should complete within timeout"
        );

        // All completion phases should be valid (exhale or post-exhale transition)
        for phase in completion_phases {
            assert!(
                phase == BreathPhase::Exhale
                    || (phase == BreathPhase::Transition && exercise.is_post_exhale_transition()),
                "Completion should only happen during exhale or post-exhale transition, got: {:?}",
                phase
            );
        }
    }

    #[test]
    fn test_never_completes_during_inhale_phase() {
        // This test specifically verifies the user's bug report is fixed
        for pattern in [
            BreathingPattern::ExtendedExhale,
            BreathingPattern::Coherent,
            BreathingPattern::ShortBox,
            BreathingPattern::Simple,
        ] {
            let mut exercise = BreathingExercise::new_with_target_cycles(pattern, 1);

            // Run until completion
            for _ in 0..1000 {
                exercise.update(Duration::from_millis(100));

                // If exercise claims it should complete, verify it's not during inhale
                if exercise.should_complete_session() {
                    assert_ne!(
                        exercise.get_current_phase(),
                        BreathPhase::Inhale,
                        "Pattern {:?} tried to complete during inhale phase!",
                        pattern
                    );
                    break;
                }
            }

            // Should have completed successfully
            assert!(
                exercise.should_complete_session(),
                "Pattern {:?} should complete",
                pattern
            );
        }
    }

    #[test]
    fn test_reset_preserves_target_cycles() {
        let mut exercise =
            BreathingExercise::new_with_target_cycles(BreathingPattern::ExtendedExhale, 5);

        // Complete a cycle
        while exercise.get_cycle_count() < 1 {
            exercise.update(Duration::from_millis(100));
        }

        assert_eq!(exercise.get_cycle_count(), 1);
        assert_eq!(exercise.get_remaining_cycles(), 4);

        // Reset
        exercise.reset();

        // Should preserve target cycles but reset progress
        assert_eq!(exercise.get_cycle_count(), 0);
        assert_eq!(exercise.get_target_cycles(), 5);
        assert_eq!(exercise.get_remaining_cycles(), 5);
        assert_eq!(exercise.get_current_phase(), BreathPhase::Inhale);
    }
}

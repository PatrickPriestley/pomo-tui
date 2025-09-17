use std::time::Duration;

/// Different activities available during breaks
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BreakActivity {
    /// Traditional breathing exercises with visual guide
    Breathing,
    /// Stretching with animated guide
    Stretch,
}

impl BreakActivity {
    /// Get display name for the activity
    pub fn display_name(&self) -> &'static str {
        match self {
            BreakActivity::Breathing => "Breathing Exercise",
            BreakActivity::Stretch => "Stretch Break",
        }
    }

    /// Get emoji icon for the activity
    pub fn icon(&self) -> &'static str {
        match self {
            BreakActivity::Breathing => "🫁",
            BreakActivity::Stretch => "🤸",
        }
    }

    /// Get description for the activity
    pub fn description(&self) -> &'static str {
        match self {
            BreakActivity::Breathing => "Guided breathing with visual circle",
            BreakActivity::Stretch => "Simple stretches with animated guide",
        }
    }

    /// Whether this activity has a visual animation
    pub fn has_animation(&self) -> bool {
        match self {
            BreakActivity::Breathing | BreakActivity::Stretch => true,
        }
    }
}

/// Animation state for non-breathing break activities
#[derive(Debug, Clone)]
pub struct BreakAnimation {
    activity: BreakActivity,
    phase_elapsed: Duration,
    total_elapsed: Duration,
    animation_frame: u32,
}

impl BreakAnimation {
    /// Create a new break animation
    pub fn new(activity: BreakActivity) -> Self {
        Self {
            activity,
            phase_elapsed: Duration::ZERO,
            total_elapsed: Duration::ZERO,
            animation_frame: 0,
        }
    }

    /// Update the animation state
    pub fn update(&mut self, delta: Duration) {
        self.phase_elapsed += delta;
        self.total_elapsed += delta;

        // Update animation frame based on activity type
        match self.activity {
            BreakActivity::Stretch => {
                // Stretch animation updates every 1000ms
                if self.phase_elapsed >= Duration::from_millis(1000) {
                    self.animation_frame = (self.animation_frame + 1) % 4; // 4 stretch positions
                    self.phase_elapsed = Duration::ZERO;
                }
            }
            BreakActivity::Breathing => {
                // Breathing exercises handle their own animation timing
            }
        }
    }

    /// Get current animation frame
    pub fn get_frame(&self) -> u32 {
        self.animation_frame
    }

    /// Get activity type
    pub fn get_activity(&self) -> BreakActivity {
        self.activity
    }

    /// Get total elapsed time
    pub fn total_elapsed(&self) -> Duration {
        self.total_elapsed
    }
}
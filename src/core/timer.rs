use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Timer {
    start: Option<Instant>,
    duration: Duration,
    paused_at: Option<Instant>,
    total_pause_duration: Duration,
    state: TimerState,
}

impl Timer {
    pub fn new(seconds: u64) -> Self {
        Timer {
            start: None,
            duration: Duration::from_secs(seconds),
            paused_at: None,
            total_pause_duration: Duration::ZERO,
            state: TimerState::Idle,
        }
    }

    pub fn start(&mut self) {
        if self.state == TimerState::Idle {
            self.start = Some(Instant::now());
            self.state = TimerState::Running;
        }
    }

    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.paused_at = Some(Instant::now());
            self.state = TimerState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == TimerState::Paused {
            if let Some(paused_at) = self.paused_at {
                self.total_pause_duration += paused_at.elapsed();
                self.paused_at = None;
                self.state = TimerState::Running;
            }
        }
    }

    pub fn stop(&mut self) {
        self.state = TimerState::Completed;
    }

    pub fn reset(&mut self) {
        self.start = None;
        self.paused_at = None;
        self.total_pause_duration = Duration::ZERO;
        self.state = TimerState::Idle;
    }

    pub fn elapsed(&self) -> Duration {
        match self.start {
            Some(start) => {
                let base_elapsed = start.elapsed();
                let current_pause = if self.state == TimerState::Paused {
                    self.paused_at
                        .map(|p| p.elapsed())
                        .unwrap_or(Duration::ZERO)
                } else {
                    Duration::ZERO
                };
                base_elapsed - self.total_pause_duration - current_pause
            }
            None => Duration::ZERO,
        }
    }

    pub fn remaining(&self) -> Duration {
        let elapsed = self.elapsed();
        if elapsed >= self.duration {
            Duration::ZERO
        } else {
            self.duration - elapsed
        }
    }

    pub fn is_expired(&self) -> bool {
        self.state == TimerState::Running && self.elapsed() >= self.duration
    }

    pub fn progress(&self) -> f64 {
        if self.duration.as_secs() == 0 {
            return 1.0;
        }
        let elapsed = self.elapsed().as_secs_f64();
        let total = self.duration.as_secs_f64();
        (elapsed / total).min(1.0)
    }

    pub fn state(&self) -> TimerState {
        if self.is_expired() {
            TimerState::Completed
        } else {
            self.state
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }
}

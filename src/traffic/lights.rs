//! Traffic light state management and timing logic

use std::time::{Duration, Instant};
use crate::traffic::{Direction, DEFAULT_GREEN_DURATION, DEFAULT_YELLOW_DURATION, DEFAULT_RED_DURATION};

/// Traffic light states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightState {
    Red,
    Yellow,
    Green,
}

impl LightState {
    /// Get the color code for terminal display
    pub fn color_code(&self) -> &'static str {
        match self {
            LightState::Red => "\x1b[31m●\x1b[0m",      // Red circle
            LightState::Yellow => "\x1b[33m●\x1b[0m",   // Yellow circle
            LightState::Green => "\x1b[32m●\x1b[0m",    // Green circle
        }
    }

    /// Check if vehicles can proceed through this light
    pub fn can_proceed(&self) -> bool {
        matches!(self, LightState::Green)
    }
}

/// Traffic light controller for a single direction
#[derive(Debug, Clone)]
pub struct TrafficLight {
    pub direction: Direction,
    pub state: LightState,
    pub last_change: Instant,
    pub green_duration: Duration,
    pub yellow_duration: Duration,
    pub red_duration: Duration,
    pub emergency_override: bool,
}

impl TrafficLight {
    /// Create a new traffic light
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            state: LightState::Red,
            last_change: Instant::now(),
            green_duration: DEFAULT_GREEN_DURATION,
            yellow_duration: DEFAULT_YELLOW_DURATION,
            red_duration: DEFAULT_RED_DURATION,
            emergency_override: false,
        }
    }

    /// Update the traffic light state based on timing
    pub fn update(&mut self) {
        if self.emergency_override {
            self.state = LightState::Red;
            return;
        }

        let elapsed = self.last_change.elapsed();
        let should_change = match self.state {
            LightState::Green => elapsed >= self.green_duration,
            LightState::Yellow => elapsed >= self.yellow_duration,
            LightState::Red => elapsed >= self.red_duration,
        };

        if should_change {
            self.advance_state();
            self.last_change = Instant::now();
        }
    }

    /// Check if the light should change state
    pub fn should_change(&self) -> bool {
        if self.emergency_override {
            return false;
        }
        
        let elapsed = self.last_change.elapsed();
        match self.state {
            LightState::Green => elapsed >= self.green_duration,
            LightState::Yellow => elapsed >= self.yellow_duration,
            LightState::Red => elapsed >= self.red_duration,
        }
    }

    /// Advance to the next light state
    fn advance_state(&mut self) {
        self.state = match self.state {
            LightState::Green => LightState::Yellow,
            LightState::Yellow => LightState::Red,
            LightState::Red => LightState::Green,
        };
    }

    /// Set emergency override (all lights red)
    pub fn set_emergency_override(&mut self, active: bool) {
        self.emergency_override = active;
        if active {
            self.state = LightState::Red;
            self.last_change = Instant::now();
        }
    }

    /// Get time remaining in current state
    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.last_change.elapsed();
        let total_duration = match self.state {
            LightState::Green => self.green_duration,
            LightState::Yellow => self.yellow_duration,
            LightState::Red => self.red_duration,
        };
        
        if elapsed >= total_duration {
            Duration::from_secs(0)
        } else {
            total_duration - elapsed
        }
    }

    /// Check if this light conflicts with another direction
    pub fn conflicts_with(&self, other_direction: Direction) -> bool {
        let (perp1, perp2) = self.direction.perpendicular();
        other_direction == perp1 || other_direction == perp2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_state_colors() {
        assert_eq!(LightState::Red.color_code(), "\x1b[31m●\x1b[0m");
        assert_eq!(LightState::Yellow.color_code(), "\x1b[33m●\x1b[0m");
        assert_eq!(LightState::Green.color_code(), "\x1b[32m●\x1b[0m");
    }

    #[test]
    fn test_can_proceed() {
        assert!(!LightState::Red.can_proceed());
        assert!(!LightState::Yellow.can_proceed());
        assert!(LightState::Green.can_proceed());
    }

    #[test]
    fn test_traffic_light_creation() {
        let light = TrafficLight::new(Direction::North);
        assert_eq!(light.direction, Direction::North);
        assert_eq!(light.state, LightState::Red);
        assert!(!light.emergency_override);
    }

    #[test]
    fn test_should_change() {
        let mut light = TrafficLight::new(Direction::North);
        light.state = LightState::Green;
        light.last_change = Instant::now() - Duration::from_secs(10);
        
        // Should change since 10 seconds > DEFAULT_GREEN_DURATION (8 seconds)
        assert!(light.should_change());
        
        // Reset timer
        light.last_change = Instant::now();
        assert!(!light.should_change());
    }

    #[test]
    fn test_emergency_override() {
        let mut light = TrafficLight::new(Direction::North);
        light.state = LightState::Green;
        
        light.set_emergency_override(true);
        assert_eq!(light.state, LightState::Red);
        assert!(!light.should_change()); // Should not change during emergency
    }
}
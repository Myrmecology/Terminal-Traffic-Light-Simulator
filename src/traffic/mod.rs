//! Traffic management module
//! 
//! This module handles all traffic-related functionality including
//! traffic lights, intersections, and vehicle management.

pub mod lights;
pub mod intersection;
pub mod vehicles;

pub use lights::*;
pub use intersection::*;
pub use vehicles::*;

use std::time::Duration;

/// Traffic system configuration constants
pub const DEFAULT_GREEN_DURATION: Duration = Duration::from_secs(8);
pub const DEFAULT_YELLOW_DURATION: Duration = Duration::from_secs(2);
pub const DEFAULT_RED_DURATION: Duration = Duration::from_secs(10);

/// Directions for traffic flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Get the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Get perpendicular directions
    pub fn perpendicular(&self) -> (Direction, Direction) {
        match self {
            Direction::North | Direction::South => (Direction::East, Direction::West),
            Direction::East | Direction::West => (Direction::North, Direction::South),
        }
    }

    /// Convert to movement deltas (dx, dy)
    pub fn to_delta(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

/// Vehicle spawner for generating new vehicles
#[derive(Debug)]
pub struct VehicleSpawner {
    spawn_rate: f32,  // vehicles per second
    time_since_last_spawn: f32,
    vehicle_id_counter: u32,
}

impl VehicleSpawner {
    /// Create a new vehicle spawner
    pub fn new(spawn_rate: f32) -> Self {
        Self {
            spawn_rate,
            time_since_last_spawn: 0.0,
            vehicle_id_counter: 0,
        }
    }

    /// Update the spawner
    pub fn update(&mut self, delta_time: f32) {
        self.time_since_last_spawn += delta_time;
    }

    /// Try to spawn a vehicle
    pub fn try_spawn(&mut self, position: Position, direction: Direction) -> Option<Vehicle> {
        if self.spawn_rate <= 0.0 {
            return None;
        }

        let spawn_interval = 1.0 / self.spawn_rate;
        
        if self.time_since_last_spawn >= spawn_interval {
            self.time_since_last_spawn = 0.0;
            self.vehicle_id_counter += 1;
            
            // Randomly choose vehicle type (mostly cars, occasional trucks, rare emergency)
            let rand_val = rand::random::<f32>();
            let vehicle_type = if rand_val < 0.02 {
                VehicleType::Emergency  // 2% chance
            } else if rand_val < 0.2 {
                VehicleType::Truck      // 18% chance
            } else {
                VehicleType::Car        // 80% chance
            };
            
            Some(Vehicle::new(
                self.vehicle_id_counter,
                vehicle_type,
                position,
                direction,
            ))
        } else {
            None
        }
    }

    /// Get current spawn rate
    pub fn spawn_rate(&self) -> f32 {
        self.spawn_rate
    }

    /// Set spawn rate
    pub fn set_spawn_rate(&mut self, rate: f32) {
        self.spawn_rate = rate.max(0.0);
    }

    /// Reset the spawner
    pub fn reset(&mut self) {
        self.time_since_last_spawn = 0.0;
        self.vehicle_id_counter = 0;
    }
}

/// Traffic density levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrafficDensity {
    Light,
    Moderate,
    Heavy,
    RushHour,
}

impl TrafficDensity {
    /// Convert to spawn rate multiplier
    pub fn to_spawn_multiplier(&self) -> f32 {
        match self {
            TrafficDensity::Light => 0.5,
            TrafficDensity::Moderate => 1.0,
            TrafficDensity::Heavy => 1.5,
            TrafficDensity::RushHour => 2.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
    }

    #[test]
    fn test_direction_perpendicular() {
        let (e, w) = Direction::North.perpendicular();
        assert_eq!(e, Direction::East);
        assert_eq!(w, Direction::West);
    }

    #[test]
    fn test_vehicle_spawner() {
        let mut spawner = VehicleSpawner::new(1.0); // 1 vehicle per second
        spawner.update(0.5);
        assert!(spawner.try_spawn(Position::new(0, 0), Direction::North).is_none());
        
        spawner.update(0.5);
        assert!(spawner.try_spawn(Position::new(0, 0), Direction::North).is_some());
    }

    #[test]
    fn test_traffic_density() {
        assert_eq!(TrafficDensity::Light.to_spawn_multiplier(), 0.5);
        assert_eq!(TrafficDensity::RushHour.to_spawn_multiplier(), 2.5);
    }
}
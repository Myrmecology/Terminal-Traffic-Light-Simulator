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
}

/// Traffic priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Normal = 0,
    Emergency = 1,
}
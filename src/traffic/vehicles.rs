//! Vehicle simulation and movement logic

use std::time::Instant;
use crate::traffic::{Direction, Priority};
use rand::Rng;

/// Types of vehicles in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Truck,
    Emergency,
}

impl VehicleType {
    /// Get the ASCII representation of the vehicle
    pub fn sprite(&self, direction: Direction) -> &'static str {
        match (self, direction) {
            (VehicleType::Car, Direction::North) => "^",
            (VehicleType::Car, Direction::South) => "v",
            (VehicleType::Car, Direction::East) => ">",
            (VehicleType::Car, Direction::West) => "<",
            (VehicleType::Truck, Direction::North) => "▲",
            (VehicleType::Truck, Direction::South) => "▼",
            (VehicleType::Truck, Direction::East) => "►",
            (VehicleType::Truck, Direction::West) => "◄",
            (VehicleType::Emergency, Direction::North) => "🚑",
            (VehicleType::Emergency, Direction::South) => "🚑",
            (VehicleType::Emergency, Direction::East) => "🚑",
            (VehicleType::Emergency, Direction::West) => "🚑",
        }
    }

    /// Get the color code for the vehicle
    pub fn color_code(&self) -> &'static str {
        match self {
            VehicleType::Car => "\x1b[36m",      // Cyan
            VehicleType::Truck => "\x1b[35m",    // Magenta
            VehicleType::Emergency => "\x1b[91m", // Bright red
        }
    }

    /// Get the priority level
    pub fn priority(&self) -> Priority {
        match self {
            VehicleType::Emergency => Priority::Emergency,
            _ => Priority::Normal,
        }
    }

    /// Get movement speed (cells per second)
    pub fn speed(&self) -> f32 {
        match self {
            VehicleType::Car => 2.0,
            VehicleType::Truck => 1.5,
            VehicleType::Emergency => 3.0,
        }
    }
}

/// Position on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Move one step in the given direction
    pub fn step(&self, direction: Direction) -> Position {
        match direction {
            Direction::North => Position::new(self.x, self.y - 1),
            Direction::South => Position::new(self.x, self.y + 1),
            Direction::East => Position::new(self.x + 1, self.y),
            Direction::West => Position::new(self.x - 1, self.y),
        }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Vehicle state for behavior management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleState {
    Moving,
    Waiting,
    AtIntersection,
    Exited,
}

/// A vehicle in the traffic simulation
#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: u32,
    pub vehicle_type: VehicleType,
    pub position: Position,
    pub direction: Direction,
    pub state: VehicleState,
    pub last_move: Instant,
    pub target_position: Option<Position>,
    pub waited_time: f32,
}

impl Vehicle {
    /// Create a new vehicle
    pub fn new(id: u32, vehicle_type: VehicleType, position: Position, direction: Direction) -> Self {
        Self {
            id,
            vehicle_type,
            position,
            direction,
            state: VehicleState::Moving,
            last_move: Instant::now(),
            target_position: None,
            waited_time: 0.0,
        }
    }

    /// Create a random vehicle
    pub fn random(id: u32, spawn_position: Position, direction: Direction) -> Self {
        let mut rng = rand::thread_rng();
        let vehicle_type = match rng.gen_range(0..100) {
            0..=2 => VehicleType::Emergency,   // 3% emergency vehicles
            3..=20 => VehicleType::Truck,      // 18% trucks
            _ => VehicleType::Car,             // 79% cars
        };

        Self::new(id, vehicle_type, spawn_position, direction)
    }

    /// Update vehicle position and state
    pub fn update(&mut self, delta_time: f32) {
        self.waited_time += delta_time;

        if self.state == VehicleState::Exited {
            return;
        }

        let move_interval = 1.0 / self.vehicle_type.speed();
        if self.last_move.elapsed().as_secs_f32() >= move_interval {
            if self.state == VehicleState::Moving {
                self.position = self.position.step(self.direction);
                self.last_move = Instant::now();
            }
        }
    }

    /// Check if vehicle can move to the next position
    pub fn can_move(&self, next_position: Position, other_vehicles: &[Vehicle]) -> bool {
        // Check for collision with other vehicles
        for other in other_vehicles {
            if other.id != self.id && other.position == next_position {
                return false;
            }
        }
        true
    }

    /// Set vehicle state
    pub fn set_state(&mut self, state: VehicleState) {
        self.state = state;
        if state == VehicleState::Moving {
            self.last_move = Instant::now();
        }
    }

    /// Get colored sprite for display
    pub fn display_sprite(&self) -> String {
        format!("{}{}\x1b[0m", 
                self.vehicle_type.color_code(), 
                self.vehicle_type.sprite(self.direction))
    }

    /// Check if vehicle is emergency type
    pub fn is_emergency(&self) -> bool {
        self.vehicle_type == VehicleType::Emergency
    }

    /// Get the next position this vehicle wants to move to
    pub fn next_position(&self) -> Position {
        self.position.step(self.direction)
    }

    /// Check if vehicle has been waiting too long
    pub fn is_stuck(&self) -> bool {
        self.waited_time > 30.0 && self.state == VehicleState::Waiting
    }
}

/// Vehicle spawner for generating traffic
#[derive(Debug)]
pub struct VehicleSpawner {
    next_id: u32,
    spawn_rate: f32, // vehicles per second
    last_spawn: Instant,
}

impl VehicleSpawner {
    /// Create a new spawner
    pub fn new(spawn_rate: f32) -> Self {
        Self {
            next_id: 0,
            spawn_rate,
            last_spawn: Instant::now(),
        }
    }

    /// Try to spawn a new vehicle
    pub fn try_spawn(&mut self, spawn_position: Position, direction: Direction) -> Option<Vehicle> {
        let spawn_interval = 1.0 / self.spawn_rate;
        if self.last_spawn.elapsed().as_secs_f32() >= spawn_interval {
            let vehicle = Vehicle::random(self.next_id, spawn_position, direction);
            self.next_id += 1;
            self.last_spawn = Instant::now();
            Some(vehicle)
        } else {
            None
        }
    }

    /// Set spawn rate (vehicles per second)
    pub fn set_spawn_rate(&mut self, rate: f32) {
        self.spawn_rate = rate.max(0.1); // Minimum rate
    }

    /// Get current spawn rate
    pub fn spawn_rate(&self) -> f32 {
        self.spawn_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_step() {
        let pos = Position::new(5, 5);
        assert_eq!(pos.step(Direction::North), Position::new(5, 4));
        assert_eq!(pos.step(Direction::South), Position::new(5, 6));
        assert_eq!(pos.step(Direction::East), Position::new(6, 5));
        assert_eq!(pos.step(Direction::West), Position::new(4, 5));
    }

    #[test]
    fn test_vehicle_creation() {
        let vehicle = Vehicle::new(1, VehicleType::Car, Position::new(0, 0), Direction::North);
        assert_eq!(vehicle.id, 1);
        assert_eq!(vehicle.vehicle_type, VehicleType::Car);
        assert_eq!(vehicle.direction, Direction::North);
        assert_eq!(vehicle.state, VehicleState::Moving);
    }

    #[test]
    fn test_vehicle_type_priority() {
        assert_eq!(VehicleType::Car.priority(), Priority::Normal);
        assert_eq!(VehicleType::Truck.priority(), Priority::Normal);
        assert_eq!(VehicleType::Emergency.priority(), Priority::Emergency);
    }

    #[test]
    fn test_spawner() {
        let mut spawner = VehicleSpawner::new(1.0);
        assert_eq!(spawner.spawn_rate(), 1.0);
        spawner.set_spawn_rate(2.0);
        assert_eq!(spawner.spawn_rate(), 2.0);
    }
}
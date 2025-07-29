//! Simulation engine and event management
//! 
//! This module handles the core simulation logic, event processing,
//! and coordination between traffic, weather, and statistics systems.

pub mod events;
pub mod weather;
pub mod statistics;

pub use events::*;
pub use weather::*;
pub use statistics::*;

use std::time::{Duration, Instant};
use crate::traffic::{Intersection, Vehicle, VehicleSpawner, Direction, Position};

/// Core simulation engine that orchestrates all simulation systems
#[derive(Debug)]
pub struct SimulationEngine {
    pub intersections: Vec<Intersection>,
    pub vehicles: Vec<Vehicle>,
    pub spawners: Vec<VehicleSpawner>,
    pub event_manager: EventManager,
    pub weather_system: WeatherSystem,
    pub statistics: SimulationStats,
    pub simulation_time: Duration,
    pub last_update: Instant,
    pub running: bool,
    pub time_scale: f32,
    pub spawn_points: Vec<SpawnPoint>,
}

/// Vehicle spawn point configuration
#[derive(Debug, Clone)]
pub struct SpawnPoint {
    pub position: Position,
    pub direction: Direction,
    pub active: bool,
    pub spawn_rate: f32,
}

impl SpawnPoint {
    pub fn new(position: Position, direction: Direction, spawn_rate: f32) -> Self {
        Self {
            position,
            direction,
            active: true,
            spawn_rate,
        }
    }
}

/// Simulation configuration parameters
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub target_fps: u32,
    pub max_vehicles: usize,
    pub base_spawn_rate: f32,
    pub intersection_positions: Vec<Position>,
    pub spawn_positions: Vec<(Position, Direction)>,
    pub enable_weather: bool,
    pub enable_emergency_vehicles: bool,
    pub time_scale: f32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            target_fps: 30,
            max_vehicles: 100,
            base_spawn_rate: 0.5, // vehicles per second
            intersection_positions: vec![
                Position::new(20, 15),
                Position::new(60, 15),
                Position::new(20, 25),
                Position::new(60, 25),
            ],
            spawn_positions: vec![
                (Position::new(5, 15), Direction::East),
                (Position::new(80, 15), Direction::West),
                (Position::new(20, 5), Direction::South),
                (Position::new(20, 35), Direction::North),
                (Position::new(5, 25), Direction::East),
                (Position::new(80, 25), Direction::West),
                (Position::new(60, 5), Direction::South),
                (Position::new(60, 35), Direction::North),
            ],
            enable_weather: true,
            enable_emergency_vehicles: true,
            time_scale: 1.0,
        }
    }
}

impl SimulationEngine {
    /// Create a new simulation engine with configuration
    pub fn new(config: SimulationConfig) -> Self {
        let mut engine = Self {
            intersections: Vec::new(),
            vehicles: Vec::new(),
            spawners: Vec::new(),
            event_manager: EventManager::new(),
            weather_system: WeatherSystem::new(),
            statistics: SimulationStats::new(),
            simulation_time: Duration::from_secs(0),
            last_update: Instant::now(),
            running: false,
            time_scale: config.time_scale,
            spawn_points: Vec::new(),
        };

        engine.initialize_from_config(config);
        engine
    }

    /// Initialize simulation from configuration
    fn initialize_from_config(&mut self, config: SimulationConfig) {
        // Create intersections
        for (id, position) in config.intersection_positions.iter().enumerate() {
            let intersection = Intersection::new(id as u32, *position);
            self.intersections.push(intersection);
        }

        // Create spawn points and spawners
        for (position, direction) in config.spawn_positions {
            let spawn_point = SpawnPoint::new(position, direction, config.base_spawn_rate);
            self.spawn_points.push(spawn_point);
            
            let spawner = VehicleSpawner::new(config.base_spawn_rate);
            self.spawners.push(spawner);
        }

        // Configure weather system
        if config.enable_weather {
            self.weather_system.set_enabled(true);
        }

        // Configure emergency vehicles
        if config.enable_emergency_vehicles {
            self.event_manager.set_emergency_enabled(true);
        }

        // Initialize statistics
        self.statistics.max_vehicles = config.max_vehicles;
    }

    /// Start the simulation
    pub fn start(&mut self) {
        self.running = true;
        self.last_update = Instant::now();
        self.simulation_time = Duration::from_secs(0);
    }

    /// Stop the simulation
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Update simulation by one frame
    pub fn update(&mut self) -> Result<(), SimulationError> {
        if !self.running {
            return Ok(());
        }

        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32() * self.time_scale;
        self.last_update = now;
        self.simulation_time += Duration::from_secs_f32(delta_time);

        // Update all subsystems
        self.update_vehicle_spawning(delta_time)?;
        self.update_vehicles(delta_time)?;
        self.update_intersections()?;
        self.update_events(delta_time)?;
        self.update_weather(delta_time)?;
        self.update_statistics(delta_time)?;
        self.cleanup_vehicles()?;

        Ok(())
    }

    /// Update vehicle spawning
    fn update_vehicle_spawning(&mut self, delta_time: f32) -> Result<(), SimulationError> {
        if self.vehicles.len() >= self.statistics.max_vehicles {
            return Ok(());
        }

        for (i, spawn_point) in self.spawn_points.iter().enumerate() {
            if !spawn_point.active {
                continue;
            }

            if let Some(spawner) = self.spawners.get_mut(i) {
                if let Some(vehicle) = spawner.try_spawn(spawn_point.position, spawn_point.direction) {
                    // Check if spawn position is clear
                    let position_clear = !self.vehicles.iter().any(|v| 
                        v.position.distance_to(&spawn_point.position) < 2.0
                    );

                    if position_clear {
                        self.vehicles.push(vehicle);
                        self.statistics.total_vehicles_spawned += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Update all vehicles
    fn update_vehicles(&mut self, delta_time: f32) -> Result<(), SimulationError> {
        // Clone vehicles for collision checking
        let vehicles_clone = self.vehicles.clone();

        for vehicle in &mut self.vehicles {
            vehicle.update(delta_time);

            // Check for out of bounds
            if self.is_vehicle_out_of_bounds(vehicle) {
                vehicle.set_state(crate::traffic::VehicleState::Exited);
                self.statistics.total_vehicles_processed += 1;
            }
        }

        Ok(())
    }

    /// Update all intersections
    fn update_intersections(&mut self) -> Result<(), SimulationError> {
        for intersection in &mut self.intersections {
            intersection.update(&mut self.vehicles);
        }

        Ok(())
    }

    /// Update event system
    fn update_events(&mut self, delta_time: f32) -> Result<(), SimulationError> {
        let events = self.event_manager.update(delta_time, &self.vehicles, &mut self.intersections);
        
        for event in events {
            self.handle_simulation_event(event)?;
        }

        Ok(())
    }

    /// Update weather system
    fn update_weather(&mut self, delta_time: f32) -> Result<(), SimulationError> {
        self.weather_system.update(delta_time);

        // Apply weather effects to spawners
        let weather_multiplier = self.weather_system.get_traffic_multiplier();
        for spawner in &mut self.spawners {
            let base_rate = spawner.spawn_rate() / weather_multiplier; // Remove previous effect
            spawner.set_spawn_rate(base_rate * weather_multiplier);
        }

        Ok(())
    }

    /// Update statistics
    fn update_statistics(&mut self, delta_time: f32) -> Result<(), SimulationError> {
        self.statistics.update(delta_time, &self.vehicles, &self.intersections);
        Ok(())
    }

    /// Clean up exited vehicles
    fn cleanup_vehicles(&mut self) -> Result<(), SimulationError> {
        self.vehicles.retain(|v| v.state != crate::traffic::VehicleState::Exited);
        Ok(())
    }

    /// Handle simulation events
    fn handle_simulation_event(&mut self, event: SimulationEvent) -> Result<(), SimulationError> {
        match event {
            SimulationEvent::EmergencyVehicleSpawned { position, direction } => {
                let emergency_vehicle = Vehicle::new(
                    self.get_next_vehicle_id(),
                    crate::traffic::VehicleType::Emergency,
                    position,
                    direction,
                );
                self.vehicles.push(emergency_vehicle);
                self.statistics.emergency_vehicles_spawned += 1;
            }
            SimulationEvent::WeatherChanged { new_weather } => {
                self.weather_system.set_current_weather(new_weather);
            }
            SimulationEvent::TrafficIncident { intersection_id, duration } => {
                if let Some(intersection) = self.intersections.iter_mut()
                    .find(|i| i.id == intersection_id) {
                    // Temporarily reduce efficiency
                    intersection.efficiency_score *= 0.5;
                }
            }
            SimulationEvent::RushHourStarted => {
                for spawner in &mut self.spawners {
                    spawner.set_spawn_rate(spawner.spawn_rate() * 2.0);
                }
            }
            SimulationEvent::RushHourEnded => {
                for spawner in &mut self.spawners {
                    spawner.set_spawn_rate(spawner.spawn_rate() * 0.5);
                }
            }
        }

        Ok(())
    }

    /// Check if vehicle is out of simulation bounds
    fn is_vehicle_out_of_bounds(&self, vehicle: &Vehicle) -> bool {
        vehicle.position.x < -10 || vehicle.position.x > 100 ||
        vehicle.position.y < -10 || vehicle.position.y > 50
    }

    /// Get next available vehicle ID
    fn get_next_vehicle_id(&self) -> u32 {
        self.vehicles.iter().map(|v| v.id).max().unwrap_or(0) + 1
    }

    /// Trigger emergency vehicle
    pub fn trigger_emergency_vehicle(&mut self, spawn_index: usize) -> Result<(), SimulationError> {
        if let Some(spawn_point) = self.spawn_points.get(spawn_index) {
            let event = SimulationEvent::EmergencyVehicleSpawned {
                position: spawn_point.position,
                direction: spawn_point.direction,
            };
            self.handle_simulation_event(event)
        } else {
            Err(SimulationError::InvalidSpawnPoint)
        }
    }

    /// Set weather
    pub fn set_weather(&mut self, weather: WeatherType) {
        self.weather_system.set_current_weather(weather);
    }

    /// Increase traffic density
    pub fn increase_traffic_density(&mut self, multiplier: f32) {
        for spawner in &mut self.spawners {
            spawner.set_spawn_rate(spawner.spawn_rate() * multiplier);
        }
    }

    /// Decrease traffic density
    pub fn decrease_traffic_density(&mut self, multiplier: f32) {
        for spawner in &mut self.spawners {
            spawner.set_spawn_rate(spawner.spawn_rate() * multiplier);
        }
    }

    /// Get current simulation statistics
    pub fn get_statistics(&self) -> &SimulationStats {
        &self.statistics
    }

    /// Get current weather
    pub fn get_current_weather(&self) -> WeatherType {
        self.weather_system.get_current_weather()
    }

    /// Get active vehicles count
    pub fn get_active_vehicle_count(&self) -> usize {
        self.vehicles.len()
    }

    /// Get intersection by ID
    pub fn get_intersection(&self, id: u32) -> Option<&Intersection> {
        self.intersections.iter().find(|i| i.id == id)
    }

    /// Check if any emergency vehicles are active
    pub fn has_emergency_vehicles(&self) -> bool {
        self.vehicles.iter().any(|v| v.is_emergency())
    }

    /// Get simulation uptime
    pub fn get_uptime(&self) -> Duration {
        self.simulation_time
    }

    /// Set time scale for simulation speed
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.1).min(5.0); // Clamp between 0.1x and 5x
    }

    /// Get current time scale
    pub fn get_time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Reset simulation to initial state
    pub fn reset(&mut self) {
        self.vehicles.clear();
        self.simulation_time = Duration::from_secs(0);
        self.statistics = SimulationStats::new();
        
        // Reset intersections
        for intersection in &mut self.intersections {
            intersection.traffic_count = 0;
            intersection.efficiency_score = 100.0;
        }

        // Reset spawners
        for spawner in &mut self.spawners {
            spawner.set_spawn_rate(0.5); // Reset to default
        }

        self.last_update = Instant::now();
    }
}

/// Simulation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationError {
    InvalidSpawnPoint,
    MaxVehiclesReached,
    IntersectionNotFound,
    SystemNotRunning,
    ConfigurationError(String),
}

impl std::fmt::Display for SimulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationError::InvalidSpawnPoint => write!(f, "Invalid spawn point specified"),
            SimulationError::MaxVehiclesReached => write!(f, "Maximum vehicle limit reached"),
            SimulationError::IntersectionNotFound => write!(f, "Intersection not found"),
            SimulationError::SystemNotRunning => write!(f, "Simulation system is not running"),
            SimulationError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for SimulationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_engine_creation() {
        let config = SimulationConfig::default();
        let engine = SimulationEngine::new(config);
        assert!(!engine.running);
        assert!(!engine.intersections.is_empty());
        assert!(!engine.spawn_points.is_empty());
    }

    #[test]
    fn test_spawn_point() {
        let spawn_point = SpawnPoint::new(Position::new(10, 10), Direction::North, 1.0);
        assert_eq!(spawn_point.position, Position::new(10, 10));
        assert_eq!(spawn_point.direction, Direction::North);
        assert!(spawn_point.active);
    }

    #[test]
    fn test_simulation_start_stop() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new(config);
        
        assert!(!engine.running);
        engine.start();
        assert!(engine.running);
        engine.stop();
        assert!(!engine.running);
    }

    #[test]
    fn test_time_scale() {
        let config = SimulationConfig::default();
        let mut engine = SimulationEngine::new(config);
        
        engine.set_time_scale(2.0);
        assert_eq!(engine.get_time_scale(), 2.0);
        
        engine.set_time_scale(10.0); // Should be clamped to 5.0
        assert_eq!(engine.get_time_scale(), 5.0);
        
        engine.set_time_scale(0.05); // Should be clamped to 0.1
        assert_eq!(engine.get_time_scale(), 0.1);
    }

    #[test]
    fn test_vehicle_bounds_checking() {
        let config = SimulationConfig::default();
        let engine = SimulationEngine::new(config);
        
        let vehicle = Vehicle::new(1, crate::traffic::VehicleType::Car, Position::new(-20, 10), Direction::East);
        assert!(engine.is_vehicle_out_of_bounds(&vehicle));
        
        let vehicle = Vehicle::new(2, crate::traffic::VehicleType::Car, Position::new(50, 25), Direction::East);
        assert!(!engine.is_vehicle_out_of_bounds(&vehicle));
    }
}
//! Intersection management and traffic flow control

use std::collections::HashMap;
use std::time::Instant;
use crate::traffic::{Direction, TrafficLight, Vehicle, VehicleState, Position};

/// Intersection controller managing traffic lights and vehicle flow
#[derive(Debug)]
pub struct Intersection {
    pub id: u32,
    pub position: Position,
    pub lights: HashMap<Direction, TrafficLight>,
    pub waiting_vehicles: HashMap<Direction, Vec<u32>>, // Vehicle IDs waiting at intersection
    pub emergency_active: bool,
    pub emergency_start: Option<Instant>,
    pub emergency_duration: std::time::Duration,
    pub traffic_count: u64,
    pub efficiency_score: f32,
}

impl Intersection {
    /// Create a new intersection
    pub fn new(id: u32, position: Position) -> Self {
        let mut lights = HashMap::new();
        let mut waiting_vehicles = HashMap::new();

        // Initialize lights for all directions
        for direction in [Direction::North, Direction::South, Direction::East, Direction::West] {
            let mut light = TrafficLight::new(direction);
            // Start with North-South green, East-West red
            match direction {
                Direction::North | Direction::South => {
                    light.state = crate::traffic::lights::LightState::Green;
                    light.last_change = Instant::now();
                }
                Direction::East | Direction::West => {
                    light.state = crate::traffic::lights::LightState::Red;
                    light.last_change = Instant::now();
                }
            }
            lights.insert(direction, light);
            waiting_vehicles.insert(direction, Vec::new());
        }

        Self {
            id,
            position,
            lights,
            waiting_vehicles,
            emergency_active: false,
            emergency_start: None,
            emergency_duration: std::time::Duration::from_secs(15),
            traffic_count: 0,
            efficiency_score: 100.0,
        }
    }

    /// Update intersection state and traffic lights
    pub fn update(&mut self, vehicles: &mut [Vehicle]) {
        self.handle_emergency_vehicles(vehicles);
        self.update_traffic_lights();
        self.manage_vehicle_flow(vehicles);
        self.update_waiting_vehicles(vehicles);
        self.calculate_efficiency();
    }

    /// Handle emergency vehicles
    fn handle_emergency_vehicles(&mut self, vehicles: &[Vehicle]) {
        let emergency_near = vehicles.iter().any(|v| {
            v.is_emergency() && 
            v.position.distance_to(&self.position) < 5.0 &&
            v.state != VehicleState::Exited
        });

        if emergency_near && !self.emergency_active {
            self.activate_emergency_mode();
        } else if !emergency_near && self.emergency_active {
            if let Some(start_time) = self.emergency_start {
                if start_time.elapsed() >= self.emergency_duration {
                    self.deactivate_emergency_mode();
                }
            }
        }
    }

    /// Activate emergency vehicle mode (all lights red except emergency path)
    fn activate_emergency_mode(&mut self) {
        self.emergency_active = true;
        self.emergency_start = Some(Instant::now());
        
        for light in self.lights.values_mut() {
            light.set_emergency_override(true);
        }
    }

    /// Deactivate emergency vehicle mode
    fn deactivate_emergency_mode(&mut self) {
        self.emergency_active = false;
        self.emergency_start = None;
        
        for light in self.lights.values_mut() {
            light.set_emergency_override(false);
        }
    }

    /// Update all traffic lights
    fn update_traffic_lights(&mut self) {
        if self.emergency_active {
            return; // Lights are overridden
        }

        // Update each light's state
        let mut lights_to_update = Vec::new();
        for (direction, light) in &self.lights {
            if light.should_change() {
                lights_to_update.push(*direction);
            }
        }

        // Apply updates
        for direction in lights_to_update {
            if let Some(light) = self.lights.get_mut(&direction) {
                light.update();
            }
        }

        // Ensure opposing directions have same state
        self.synchronize_opposing_lights();
    }

    /// Synchronize opposing traffic lights
    fn synchronize_opposing_lights(&mut self) {
        // North-South synchronization
        let north_state = self.lights.get(&Direction::North).map(|l| (l.state, l.last_change));
        if let Some((state, last_change)) = north_state {
            if let Some(south) = self.lights.get_mut(&Direction::South) {
                south.state = state;
                south.last_change = last_change;
            }
        }

        // East-West synchronization
        let east_state = self.lights.get(&Direction::East).map(|l| (l.state, l.last_change));
        if let Some((state, last_change)) = east_state {
            if let Some(west) = self.lights.get_mut(&Direction::West) {
                west.state = state;
                west.last_change = last_change;
            }
        }
    }

    /// Manage vehicle flow through intersection
    fn manage_vehicle_flow(&mut self, vehicles: &mut [Vehicle]) {
        for vehicle in vehicles.iter_mut() {
            if vehicle.state == VehicleState::Exited {
                continue;
            }

            let distance_to_intersection = vehicle.position.distance_to(&self.position);
            
            // Vehicle approaching intersection
            if distance_to_intersection <= 2.0 && distance_to_intersection > 0.5 {
                if self.can_vehicle_proceed(vehicle) {
                    vehicle.set_state(VehicleState::Moving);
                } else {
                    vehicle.set_state(VehicleState::Waiting);
                    self.add_waiting_vehicle(vehicle.direction, vehicle.id);
                }
            }
            // Vehicle at intersection
            else if distance_to_intersection <= 0.5 {
                vehicle.set_state(VehicleState::AtIntersection);
                self.traffic_count += 1;
            }
            // Vehicle past intersection
            else if distance_to_intersection > 2.0 && vehicle.state == VehicleState::AtIntersection {
                vehicle.set_state(VehicleState::Moving);
                self.remove_waiting_vehicle(vehicle.direction, vehicle.id);
            }
        }
    }

    /// Check if vehicle can proceed through intersection
    fn can_vehicle_proceed(&self, vehicle: &Vehicle) -> bool {
        if vehicle.is_emergency() {
            return true; // Emergency vehicles always have priority
        }

        if let Some(light) = self.lights.get(&vehicle.direction) {
            light.state.can_proceed()
        } else {
            false
        }
    }

    /// Add vehicle to waiting queue
    fn add_waiting_vehicle(&mut self, direction: Direction, vehicle_id: u32) {
        if let Some(waiting_list) = self.waiting_vehicles.get_mut(&direction) {
            if !waiting_list.contains(&vehicle_id) {
                waiting_list.push(vehicle_id);
            }
        }
    }

    /// Remove vehicle from waiting queue
    fn remove_waiting_vehicle(&mut self, direction: Direction, vehicle_id: u32) {
        if let Some(waiting_list) = self.waiting_vehicles.get_mut(&direction) {
            waiting_list.retain(|&id| id != vehicle_id);
        }
    }

    /// Update waiting vehicles state
    fn update_waiting_vehicles(&mut self, vehicles: &mut [Vehicle]) {
        for (direction, waiting_ids) in &self.waiting_vehicles {
            if let Some(light) = self.lights.get(direction) {
                if light.state.can_proceed() {
                    // Allow waiting vehicles to proceed
                    for &vehicle_id in waiting_ids {
                        if let Some(vehicle) = vehicles.iter_mut().find(|v| v.id == vehicle_id) {
                            if vehicle.state == VehicleState::Waiting {
                                vehicle.set_state(VehicleState::Moving);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Calculate intersection efficiency score
    fn calculate_efficiency(&mut self) {
        let total_waiting = self.waiting_vehicles.values()
            .map(|v| v.len())
            .sum::<usize>() as f32;

        // Efficiency based on waiting vehicles (fewer waiting = higher efficiency)
        let waiting_penalty = total_waiting * 2.0;
        let base_score = 100.0;
        
        self.efficiency_score = (base_score - waiting_penalty).max(0.0);
        
        // Bonus for emergency vehicle handling
        if self.emergency_active {
            self.efficiency_score += 10.0;
        }
    }

    /// Get traffic light state for a direction
    pub fn get_light_state(&self, direction: Direction) -> Option<crate::traffic::lights::LightState> {
        self.lights.get(&direction).map(|light| light.state)
    }

    /// Get number of waiting vehicles in a direction
    pub fn get_waiting_count(&self, direction: Direction) -> usize {
        self.waiting_vehicles.get(&direction)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get total vehicles processed
    pub fn get_traffic_count(&self) -> u64 {
        self.traffic_count
    }

    /// Get efficiency score (0-100+)
    pub fn get_efficiency_score(&self) -> f32 {
        self.efficiency_score
    }

    /// Check if intersection is in emergency mode
    pub fn is_emergency_active(&self) -> bool {
        self.emergency_active
    }

    /// Get intersection status for display
    pub fn get_status_summary(&self) -> IntersectionStatus {
        let total_waiting = self.waiting_vehicles.values()
            .map(|v| v.len())
            .sum::<usize>();

        IntersectionStatus {
            id: self.id,
            emergency_active: self.emergency_active,
            total_waiting,
            traffic_count: self.traffic_count,
            efficiency_score: self.efficiency_score,
            north_south_light: self.get_light_state(Direction::North).unwrap_or(crate::traffic::lights::LightState::Red),
            east_west_light: self.get_light_state(Direction::East).unwrap_or(crate::traffic::lights::LightState::Red),
        }
    }

    /// Force light change for testing/demo purposes
    pub fn force_light_change(&mut self, direction: Direction) {
        if let Some(light) = self.lights.get_mut(&direction) {
            light.last_change = Instant::now() - light.green_duration;
        }
    }
}

/// Intersection status for UI display
#[derive(Debug, Clone)]
pub struct IntersectionStatus {
    pub id: u32,
    pub emergency_active: bool,
    pub total_waiting: usize,
    pub traffic_count: u64,
    pub efficiency_score: f32,
    pub north_south_light: crate::traffic::lights::LightState,
    pub east_west_light: crate::traffic::lights::LightState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection_creation() {
        let intersection = Intersection::new(1, Position::new(10, 10));
        assert_eq!(intersection.id, 1);
        assert_eq!(intersection.position, Position::new(10, 10));
        assert!(!intersection.emergency_active);
        assert_eq!(intersection.lights.len(), 4);
    }

    #[test]
    fn test_emergency_mode() {
        let mut intersection = Intersection::new(1, Position::new(10, 10));
        intersection.activate_emergency_mode();
        assert!(intersection.emergency_active);
        assert!(intersection.emergency_start.is_some());
    }

    #[test]
    fn test_waiting_vehicles() {
        let mut intersection = Intersection::new(1, Position::new(10, 10));
        intersection.add_waiting_vehicle(Direction::North, 123);
        assert_eq!(intersection.get_waiting_count(Direction::North), 1);
        
        intersection.remove_waiting_vehicle(Direction::North, 123);
        assert_eq!(intersection.get_waiting_count(Direction::North), 0);
    }

    #[test]
    fn test_efficiency_calculation() {
        let mut intersection = Intersection::new(1, Position::new(10, 10));
        intersection.calculate_efficiency();
        assert_eq!(intersection.get_efficiency_score(), 100.0);
        
        intersection.add_waiting_vehicle(Direction::North, 1);
        intersection.calculate_efficiency();
        assert!(intersection.get_efficiency_score() < 100.0);
    }
}
//! Event system for managing simulation events and special scenarios
//! 
//! This module handles emergency vehicles, traffic incidents, rush hour,
//! and other dynamic events that affect the traffic simulation.

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use rand::Rng;

use crate::traffic::{Vehicle, Intersection, Direction, Position, VehicleType};
use crate::simulation::WeatherType;

/// Event manager for coordinating simulation events
#[derive(Debug)]
pub struct EventManager {
    pub active_events: Vec<ActiveEvent>,
    pub event_queue: VecDeque<ScheduledEvent>,
    pub emergency_enabled: bool,
    pub rush_hour_enabled: bool,
    pub incidents_enabled: bool,
    pub last_emergency_spawn: Instant,
    pub emergency_cooldown: Duration,
    pub rush_hour_schedule: RushHourSchedule,
    pub incident_probability: f32,
    pub random_events_enabled: bool,
}

/// Active event with duration tracking
#[derive(Debug, Clone)]
pub struct ActiveEvent {
    pub event: SimulationEvent,
    pub started_at: Instant,
    pub duration: Option<Duration>,
    pub persistent: bool,
}

/// Scheduled event for future execution
#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    pub event: SimulationEvent,
    pub execute_at: Instant,
    pub priority: EventPriority,
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Rush hour configuration
#[derive(Debug, Clone)]
pub struct RushHourSchedule {
    pub morning_start: Duration,
    pub morning_end: Duration,
    pub evening_start: Duration,
    pub evening_end: Duration,
    pub traffic_multiplier: f32,
    pub currently_active: bool,
}

impl Default for RushHourSchedule {
    fn default() -> Self {
        Self {
            morning_start: Duration::from_secs(7 * 3600), // 7:00 AM
            morning_end: Duration::from_secs(9 * 3600),   // 9:00 AM
            evening_start: Duration::from_secs(17 * 3600), // 5:00 PM
            evening_end: Duration::from_secs(19 * 3600),   // 7:00 PM
            traffic_multiplier: 2.5,
            currently_active: false,
        }
    }
}

/// Simulation events that can occur
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationEvent {
    /// Emergency vehicle spawned at position with direction
    EmergencyVehicleSpawned {
        position: Position,
        direction: Direction,
    },
    /// Weather conditions changed
    WeatherChanged {
        new_weather: WeatherType,
    },
    /// Traffic incident at intersection
    TrafficIncident {
        intersection_id: u32,
        duration: Duration,
    },
    /// Rush hour period started
    RushHourStarted,
    /// Rush hour period ended
    RushHourEnded,
    /// Traffic light malfunction
    TrafficLightMalfunction {
        intersection_id: u32,
        duration: Duration,
    },
    /// Road construction affecting traffic
    RoadConstruction {
        start_position: Position,
        end_position: Position,
        duration: Duration,
    },
    /// Special event (parade, festival, etc.)
    SpecialEvent {
        center_position: Position,
        radius: f32,
        duration: Duration,
        traffic_impact: f32,
    },
    /// System maintenance mode
    MaintenanceMode {
        duration: Duration,
    },
}

impl EventManager {
    /// Create new event manager
    pub fn new() -> Self {
        Self {
            active_events: Vec::new(),
            event_queue: VecDeque::new(),
            emergency_enabled: true,
            rush_hour_enabled: true,
            incidents_enabled: true,
            last_emergency_spawn: Instant::now() - Duration::from_secs(60),
            emergency_cooldown: Duration::from_secs(30),
            rush_hour_schedule: RushHourSchedule::default(),
            incident_probability: 0.001, // 0.1% chance per update
            random_events_enabled: true,
        }
    }

    /// Update event system and return triggered events
    pub fn update(&mut self, delta_time: f32, vehicles: &[Vehicle], intersections: &mut [Intersection]) -> Vec<SimulationEvent> {
        let mut triggered_events = Vec::new();

        // Process scheduled events
        self.process_scheduled_events(&mut triggered_events);

        // Update active events
        self.update_active_events();

        // Check for automatic event triggers
        self.check_emergency_vehicle_triggers(vehicles, &mut triggered_events);
        self.check_rush_hour_triggers(&mut triggered_events);
        self.check_random_event_triggers(&mut triggered_events);

        // Apply active event effects to intersections
        self.apply_event_effects(intersections);

        triggered_events
    }

    /// Process events scheduled for execution
    fn process_scheduled_events(&mut self, triggered_events: &mut Vec<SimulationEvent>) {
        let now = Instant::now();
        
        while let Some(scheduled) = self.event_queue.front() {
            if scheduled.execute_at <= now {
                let scheduled = self.event_queue.pop_front().unwrap();
                triggered_events.push(scheduled.event.clone());
                
                // Add to active events if it has duration
                let duration = self.get_event_duration(&scheduled.event);
                let active_event = ActiveEvent {
                    event: scheduled.event,
                    started_at: now,
                    duration,
                    persistent: duration.is_none(),
                };
                self.active_events.push(active_event);
            } else {
                break;
            }
        }
    }

    /// Update active events and remove expired ones
    fn update_active_events(&mut self) {
        let now = Instant::now();
        
        self.active_events.retain(|event| {
            if let Some(duration) = event.duration {
                event.started_at.elapsed() < duration
            } else {
                event.persistent
            }
        });
    }

    /// Check for emergency vehicle triggers
    fn check_emergency_vehicle_triggers(&mut self, vehicles: &[Vehicle], triggered_events: &mut Vec<SimulationEvent>) {
        if !self.emergency_enabled {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_emergency_spawn) < self.emergency_cooldown {
            return;
        }

        // Random emergency vehicle spawn
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < 0.0005 { // 0.05% chance per update
            let spawn_positions = [
                (Position::new(5, 15), Direction::East),
                (Position::new(80, 15), Direction::West),
                (Position::new(20, 5), Direction::South),
                (Position::new(60, 35), Direction::North),
            ];

            if let Some((position, direction)) = spawn_positions.get(rng.gen_range(0..spawn_positions.len())) {
                triggered_events.push(SimulationEvent::EmergencyVehicleSpawned {
                    position: *position,
                    direction: *direction,
                });
                self.last_emergency_spawn = now;
            }
        }
    }

    /// Check for rush hour triggers
    fn check_rush_hour_triggers(&mut self, triggered_events: &mut Vec<SimulationEvent>) {
        if !self.rush_hour_enabled {
            return;
        }

        // Simulate time of day (for demo purposes, use elapsed seconds)
        let simulation_time = self.last_emergency_spawn.elapsed().as_secs() % (24 * 3600);
        let current_time = Duration::from_secs(simulation_time);

        let should_be_rush_hour = 
            (current_time >= self.rush_hour_schedule.morning_start && current_time <= self.rush_hour_schedule.morning_end) ||
            (current_time >= self.rush_hour_schedule.evening_start && current_time <= self.rush_hour_schedule.evening_end);

        if should_be_rush_hour && !self.rush_hour_schedule.currently_active {
            triggered_events.push(SimulationEvent::RushHourStarted);
            self.rush_hour_schedule.currently_active = true;
        } else if !should_be_rush_hour && self.rush_hour_schedule.currently_active {
            triggered_events.push(SimulationEvent::RushHourEnded);
            self.rush_hour_schedule.currently_active = false;
        }
    }

    /// Check for random event triggers
    fn check_random_event_triggers(&mut self, triggered_events: &mut Vec<SimulationEvent>) {
        if !self.random_events_enabled || !self.incidents_enabled {
            return;
        }

        let mut rng = rand::thread_rng();

        // Traffic incidents
        if rng.gen::<f32>() < self.incident_probability {
            let intersection_id = rng.gen_range(0..4);
            let duration = Duration::from_secs(rng.gen_range(30..120));
            
            triggered_events.push(SimulationEvent::TrafficIncident {
                intersection_id,
                duration,
            });
        }

        // Traffic light malfunctions (very rare)
        if rng.gen::<f32>() < 0.0001 {
            let intersection_id = rng.gen_range(0..4);
            let duration = Duration::from_secs(rng.gen_range(15..60));
            
            triggered_events.push(SimulationEvent::TrafficLightMalfunction {
                intersection_id,
                duration,
            });
        }
    }

    /// Apply active event effects to intersections
    fn apply_event_effects(&self, intersections: &mut [Intersection]) {
        for active_event in &self.active_events {
            match &active_event.event {
                SimulationEvent::TrafficIncident { intersection_id, .. } => {
                    if let Some(intersection) = intersections.iter_mut().find(|i| i.id == *intersection_id) {
                        intersection.efficiency_score = intersection.efficiency_score.min(50.0);
                    }
                }
                SimulationEvent::TrafficLightMalfunction { intersection_id, .. } => {
                    if let Some(intersection) = intersections.iter_mut().find(|i| i.id == *intersection_id) {
                        // Force all lights to red during malfunction
                        for light in intersection.lights.values_mut() {
                            light.set_emergency_override(true);
                        }
                    }
                }
                _ => {} // Other events handled elsewhere
            }
        }
    }

    /// Get duration for an event type
    fn get_event_duration(&self, event: &SimulationEvent) -> Option<Duration> {
        match event {
            SimulationEvent::TrafficIncident { duration, .. } => Some(*duration),
            SimulationEvent::TrafficLightMalfunction { duration, .. } => Some(*duration),
            SimulationEvent::RoadConstruction { duration, .. } => Some(*duration),
            SimulationEvent::SpecialEvent { duration, .. } => Some(*duration),
            SimulationEvent::MaintenanceMode { duration } => Some(*duration),
            _ => None, // Instantaneous events
        }
    }

    /// Schedule an event for future execution
    pub fn schedule_event(&mut self, event: SimulationEvent, delay: Duration, priority: EventPriority) {
        let scheduled_event = ScheduledEvent {
            event,
            execute_at: Instant::now() + delay,
            priority,
        };

        // Insert based on priority and timing
        let mut inserted = false;
        for (i, existing) in self.event_queue.iter().enumerate() {
            if scheduled_event.priority > existing.priority || 
               (scheduled_event.priority == existing.priority && scheduled_event.execute_at < existing.execute_at) {
                self.event_queue.insert(i, scheduled_event);
                inserted = true;
                break;
            }
        }

        if !inserted {
            self.event_queue.push_back(scheduled_event);
        }
    }

    /// Trigger emergency vehicle immediately
    pub fn trigger_emergency_vehicle(&mut self, position: Position, direction: Direction) {
        let event = SimulationEvent::EmergencyVehicleSpawned { position, direction };
        self.schedule_event(event, Duration::from_secs(0), EventPriority::Critical);
        self.last_emergency_spawn = Instant::now();
    }

    /// Trigger weather change
    pub fn trigger_weather_change(&mut self, weather: WeatherType) {
        let event = SimulationEvent::WeatherChanged { new_weather: weather };
        self.schedule_event(event, Duration::from_secs(0), EventPriority::Normal);
    }

    /// Trigger traffic incident
    pub fn trigger_traffic_incident(&mut self, intersection_id: u32, duration: Duration) {
        let event = SimulationEvent::TrafficIncident { intersection_id, duration };
        self.schedule_event(event, Duration::from_secs(0), EventPriority::High);
    }

    /// Enable/disable emergency vehicles
    pub fn set_emergency_enabled(&mut self, enabled: bool) {
        self.emergency_enabled = enabled;
    }

    /// Enable/disable rush hour
    pub fn set_rush_hour_enabled(&mut self, enabled: bool) {
        self.rush_hour_enabled = enabled;
    }

    /// Enable/disable random incidents
    pub fn set_incidents_enabled(&mut self, enabled: bool) {
        self.incidents_enabled = enabled;
    }

    /// Set incident probability
    pub fn set_incident_probability(&mut self, probability: f32) {
        self.incident_probability = probability.clamp(0.0, 1.0);
    }

    /// Get active event count
    pub fn get_active_event_count(&self) -> usize {
        self.active_events.len()
    }

    /// Get scheduled event count
    pub fn get_scheduled_event_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Check if rush hour is currently active
    pub fn is_rush_hour_active(&self) -> bool {
        self.rush_hour_schedule.currently_active
    }

    /// Get rush hour traffic multiplier
    pub fn get_rush_hour_multiplier(&self) -> f32 {
        if self.rush_hour_schedule.currently_active {
            self.rush_hour_schedule.traffic_multiplier
        } else {
            1.0
        }
    }

    /// Clear all events
    pub fn clear_all_events(&mut self) {
        self.active_events.clear();
        self.event_queue.clear();
        self.rush_hour_schedule.currently_active = false;
    }

    /// Get current active events for display
    pub fn get_active_events(&self) -> Vec<String> {
        self.active_events.iter().map(|ae| {
            match &ae.event {
                SimulationEvent::EmergencyVehicleSpawned { .. } => "Emergency Vehicle Active".to_string(),
                SimulationEvent::WeatherChanged { new_weather } => format!("Weather: {:?}", new_weather),
                SimulationEvent::TrafficIncident { intersection_id, .. } => format!("Incident at Intersection {}", intersection_id),
                SimulationEvent::RushHourStarted => "Rush Hour Active".to_string(),
                SimulationEvent::TrafficLightMalfunction { intersection_id, .. } => format!("Light Malfunction at {}", intersection_id),
                SimulationEvent::RoadConstruction { .. } => "Road Construction".to_string(),
                SimulationEvent::SpecialEvent { .. } => "Special Event".to_string(),
                SimulationEvent::MaintenanceMode { .. } => "Maintenance Mode".to_string(),
                _ => "Unknown Event".to_string(),
            }
        }).collect()
    }
}

/// Event statistics for monitoring
#[derive(Debug, Clone)]
pub struct EventStatistics {
    pub total_emergency_vehicles: u32,
    pub total_incidents: u32,
    pub total_malfunctions: u32,
    pub rush_hour_cycles: u32,
    pub weather_changes: u32,
}

impl EventStatistics {
    pub fn new() -> Self {
        Self {
            total_emergency_vehicles: 0,
            total_incidents: 0,
            total_malfunctions: 0,
            rush_hour_cycles: 0,
            weather_changes: 0,
        }
    }

    pub fn record_event(&mut self, event: &SimulationEvent) {
        match event {
            SimulationEvent::EmergencyVehicleSpawned { .. } => self.total_emergency_vehicles += 1,
            SimulationEvent::TrafficIncident { .. } => self.total_incidents += 1,
            SimulationEvent::TrafficLightMalfunction { .. } => self.total_malfunctions += 1,
            SimulationEvent::RushHourStarted => self.rush_hour_cycles += 1,
            SimulationEvent::WeatherChanged { .. } => self.weather_changes += 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_manager_creation() {
        let manager = EventManager::new();
        assert!(manager.emergency_enabled);
        assert!(manager.rush_hour_enabled);
        assert!(manager.incidents_enabled);
    }

    #[test]
    fn test_event_scheduling() {
        let mut manager = EventManager::new();
        let event = SimulationEvent::EmergencyVehicleSpawned {
            position: Position::new(10, 10),
            direction: Direction::North,
        };

        manager.schedule_event(event, Duration::from_secs(5), EventPriority::High);
        assert_eq!(manager.get_scheduled_event_count(), 1);
    }

    #[test]
    fn test_rush_hour_schedule() {
        let schedule = RushHourSchedule::default();
        assert!(!schedule.currently_active);
        assert!(schedule.traffic_multiplier > 1.0);
    }

    #[test]
    fn test_event_priority() {
        assert!(EventPriority::Critical > EventPriority::High);
        assert!(EventPriority::High > EventPriority::Normal);
        assert!(EventPriority::Normal > EventPriority::Low);
    }

    #[test]
    fn test_event_statistics() {
        let mut stats = EventStatistics::new();
        let event = SimulationEvent::EmergencyVehicleSpawned {
            position: Position::new(0, 0),
            direction: Direction::North,
        };

        stats.record_event(&event);
        assert_eq!(stats.total_emergency_vehicles, 1);
    }

    #[test]
    fn test_active_event() {
        let event = SimulationEvent::RushHourStarted;
        let active_event = ActiveEvent {
            event: event.clone(),
            started_at: Instant::now(),
            duration: Some(Duration::from_secs(10)),
            persistent: false,
        };

        assert_eq!(active_event.event, event);
        assert!(active_event.duration.is_some());
    }
}
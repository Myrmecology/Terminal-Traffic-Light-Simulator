//! Statistics tracking and performance monitoring
//! 
//! This module provides comprehensive statistics collection and analysis
//! for the traffic simulation, including performance metrics, vehicle
//! tracking, and efficiency calculations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::traffic::{Vehicle, Intersection, VehicleType, VehicleState, Direction};

/// Comprehensive simulation statistics
#[derive(Debug, Clone)]
pub struct SimulationStats {
    // Vehicle statistics
    pub total_vehicles_spawned: u64,
    pub total_vehicles_processed: u64,
    pub active_vehicles: u32,
    pub vehicle_counts: HashMap<VehicleType, u32>,
    pub vehicles_by_state: HashMap<VehicleState, u32>,
    pub emergency_vehicles_spawned: u32,

    // Performance metrics
    pub average_wait_time: f32,
    pub peak_wait_time: f32,
    pub total_wait_time: f32,
    pub overall_efficiency: f32,
    pub throughput_per_minute: f32,

    // Intersection statistics
    pub intersection_stats: HashMap<u32, IntersectionStats>,
    pub traffic_light_changes: u64,
    pub emergency_overrides: u32,

    // Time-based metrics
    pub simulation_start_time: Instant,
    pub total_simulation_time: Duration,
    pub last_update_time: Instant,
    pub update_frequency: f32,

    // Efficiency tracking
    pub efficiency_history: Vec<EfficiencySnapshot>,
    pub max_efficiency_samples: usize,
    pub current_fps: f64,
    pub target_fps: f64,

    // Traffic flow analysis
    pub flow_analysis: TrafficFlowAnalysis,
    
    // Configuration
    pub max_vehicles: usize,
    pub statistics_enabled: bool,
}

/// Individual intersection statistics
#[derive(Debug, Clone)]
pub struct IntersectionStats {
    pub id: u32,
    pub vehicles_processed: u64,
    pub average_efficiency: f32,
    pub total_wait_time: f32,
    pub emergency_activations: u32,
    pub light_cycle_count: u64,
    pub last_emergency_time: Option<Instant>,
    pub peak_queue_length: usize,
    pub current_queue_lengths: HashMap<Direction, usize>,
}

/// Efficiency snapshot for trend analysis
#[derive(Debug, Clone)]
pub struct EfficiencySnapshot {
    pub timestamp: Instant,
    pub overall_efficiency: f32,
    pub vehicle_count: u32,
    pub average_wait_time: f32,
    pub throughput: f32,
}

/// Traffic flow analysis data
#[derive(Debug, Clone)]
pub struct TrafficFlowAnalysis {
    pub flow_rates: HashMap<Direction, f32>, // vehicles per minute
    pub congestion_points: Vec<CongestionPoint>,
    pub optimal_light_timings: HashMap<u32, LightTiming>,
    pub bottleneck_intersections: Vec<u32>,
    pub peak_flow_times: Vec<Duration>,
}

/// Congestion point identification
#[derive(Debug, Clone)]
pub struct CongestionPoint {
    pub intersection_id: u32,
    pub direction: Direction,
    pub severity: CongestionSeverity,
    pub average_delay: f32,
    pub detected_at: Instant,
}

/// Congestion severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CongestionSeverity {
    Light,
    Moderate,
    Heavy,
    Severe,
}

/// Optimal light timing recommendations
#[derive(Debug, Clone)]
pub struct LightTiming {
    pub intersection_id: u32,
    pub recommended_green_duration: Duration,
    pub recommended_yellow_duration: Duration,
    pub recommended_red_duration: Duration,
    pub confidence: f32, // 0.0 to 1.0
}

impl SimulationStats {
    /// Create new statistics tracker
    pub fn new() -> Self {
        Self {
            total_vehicles_spawned: 0,
            total_vehicles_processed: 0,
            active_vehicles: 0,
            vehicle_counts: HashMap::new(),
            vehicles_by_state: HashMap::new(),
            emergency_vehicles_spawned: 0,
            
            average_wait_time: 0.0,
            peak_wait_time: 0.0,
            total_wait_time: 0.0,
            overall_efficiency: 100.0,
            throughput_per_minute: 0.0,
            
            intersection_stats: HashMap::new(),
            traffic_light_changes: 0,
            emergency_overrides: 0,
            
            simulation_start_time: Instant::now(),
            total_simulation_time: Duration::from_secs(0),
            last_update_time: Instant::now(),
            update_frequency: 30.0,
            
            efficiency_history: Vec::new(),
            max_efficiency_samples: 300, // 10 minutes at 30fps
            current_fps: 30.0,
            target_fps: 30.0,
            
            flow_analysis: TrafficFlowAnalysis {
                flow_rates: HashMap::new(),
                congestion_points: Vec::new(),
                optimal_light_timings: HashMap::new(),
                bottleneck_intersections: Vec::new(),
                peak_flow_times: Vec::new(),
            },
            
            max_vehicles: 100,
            statistics_enabled: true,
        }
    }

    /// Update statistics with current simulation state
    pub fn update(&mut self, delta_time: f32, vehicles: &[Vehicle], intersections: &[Intersection]) {
        if !self.statistics_enabled {
            return;
        }

        let now = Instant::now();
        self.total_simulation_time += Duration::from_secs_f32(delta_time);
        
        // Update basic vehicle statistics
        self.update_vehicle_statistics(vehicles);
        
        // Update intersection statistics
        self.update_intersection_statistics(intersections);
        
        // Update performance metrics
        self.update_performance_metrics(vehicles);
        
        // Update efficiency tracking
        self.update_efficiency_tracking(now);
        
        // Update traffic flow analysis
        self.update_traffic_flow_analysis(vehicles, intersections);
        
        // Update FPS tracking
        self.update_fps_tracking(delta_time);
        
        self.last_update_time = now;
    }

    /// Update vehicle-related statistics
    fn update_vehicle_statistics(&mut self, vehicles: &[Vehicle]) {
        self.active_vehicles = vehicles.len() as u32;
        
        // Clear and recalculate vehicle counts
        self.vehicle_counts.clear();
        self.vehicles_by_state.clear();
        
        for vehicle in vehicles {
            // Count by type
            *self.vehicle_counts.entry(vehicle.vehicle_type).or_insert(0) += 1;
            
            // Count by state
            *self.vehicles_by_state.entry(vehicle.state).or_insert(0) += 1;
        }
        
        // Calculate wait times
        let waiting_vehicles: Vec<_> = vehicles.iter()
            .filter(|v| v.state == VehicleState::Waiting)
            .collect();
        
        if !waiting_vehicles.is_empty() {
            let total_wait: f32 = waiting_vehicles.iter()
                .map(|v| v.waited_time)
                .sum();
            
            self.average_wait_time = total_wait / waiting_vehicles.len() as f32;
            self.peak_wait_time = waiting_vehicles.iter()
                .map(|v| v.waited_time)
                .fold(0.0, f32::max);
            self.total_wait_time += total_wait;
        }
    }

    /// Update intersection-specific statistics
    fn update_intersection_statistics(&mut self, intersections: &[Intersection]) {
        for intersection in intersections {
            let stats = self.intersection_stats
                .entry(intersection.id)
                .or_insert_with(|| IntersectionStats {
                    id: intersection.id,
                    vehicles_processed: 0,
                    average_efficiency: 100.0,
                    total_wait_time: 0.0,
                    emergency_activations: 0,
                    light_cycle_count: 0,
                    last_emergency_time: None,
                    peak_queue_length: 0,
                    current_queue_lengths: HashMap::new(),
                });

            // Update efficiency
            stats.average_efficiency = intersection.get_efficiency_score();
            stats.vehicles_processed = intersection.get_traffic_count();
            
            // Track emergency activations
            if intersection.is_emergency_active() {
                if stats.last_emergency_time.is_none() || 
                   stats.last_emergency_time.unwrap().elapsed() > Duration::from_secs(60) {
                    stats.emergency_activations += 1;
                    stats.last_emergency_time = Some(Instant::now());
                }
            }
            
            // Update queue lengths
            for direction in [Direction::North, Direction::South, Direction::East, Direction::West] {
                let queue_length = intersection.get_waiting_count(direction);
                stats.current_queue_lengths.insert(direction, queue_length);
                
                if queue_length > stats.peak_queue_length {
                    stats.peak_queue_length = queue_length;
                }
            }
        }
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self, vehicles: &[Vehicle]) {
        // Calculate overall efficiency
        let efficiency_sum: f32 = self.intersection_stats.values()
            .map(|stats| stats.average_efficiency)
            .sum();
        
        if !self.intersection_stats.is_empty() {
            self.overall_efficiency = efficiency_sum / self.intersection_stats.len() as f32;
        }
        
        // Calculate throughput (vehicles processed per minute)
        let elapsed_minutes = self.total_simulation_time.as_secs_f32() / 60.0;
        if elapsed_minutes > 0.0 {
            self.throughput_per_minute = self.total_vehicles_processed as f32 / elapsed_minutes;
        }
        
        // Apply penalties for congestion
        let congestion_penalty = self.calculate_congestion_penalty(vehicles);
        self.overall_efficiency = (self.overall_efficiency * (1.0 - congestion_penalty)).max(0.0);
    }

    /// Calculate congestion penalty based on stuck vehicles
    fn calculate_congestion_penalty(&self, vehicles: &[Vehicle]) -> f32 {
        let stuck_vehicles = vehicles.iter()
            .filter(|v| v.is_stuck())
            .count();
        
        if vehicles.is_empty() {
            0.0
        } else {
            (stuck_vehicles as f32 / vehicles.len() as f32) * 0.3 // Max 30% penalty
        }
    }

    /// Update efficiency tracking with historical data
    fn update_efficiency_tracking(&mut self, now: Instant) {
        let snapshot = EfficiencySnapshot {
            timestamp: now,
            overall_efficiency: self.overall_efficiency,
            vehicle_count: self.active_vehicles,
            average_wait_time: self.average_wait_time,
            throughput: self.throughput_per_minute,
        };
        
        self.efficiency_history.push(snapshot);
        
        // Maintain maximum sample size
        if self.efficiency_history.len() > self.max_efficiency_samples {
            self.efficiency_history.remove(0);
        }
    }

    /// Update traffic flow analysis
    fn update_traffic_flow_analysis(&mut self, vehicles: &[Vehicle], intersections: &[Intersection]) {
        // Calculate flow rates by direction
        for direction in [Direction::North, Direction::South, Direction::East, Direction::West] {
            let vehicles_in_direction = vehicles.iter()
                .filter(|v| v.direction == direction && v.state == VehicleState::Moving)
                .count();
            
            let elapsed_minutes = self.total_simulation_time.as_secs_f32() / 60.0;
            if elapsed_minutes > 0.0 {
                let flow_rate = vehicles_in_direction as f32 / elapsed_minutes;
                self.flow_analysis.flow_rates.insert(direction, flow_rate);
            }
        }
        
        // Identify congestion points
        self.identify_congestion_points(intersections);
        
        // Calculate optimal light timings
        self.calculate_optimal_light_timings(intersections);
        
        // Identify bottleneck intersections
        self.identify_bottleneck_intersections(intersections);
    }

    /// Identify congestion points in the simulation
    fn identify_congestion_points(&mut self, intersections: &[Intersection]) {
        self.flow_analysis.congestion_points.clear();
        
        for intersection in intersections {
            if let Some(stats) = self.intersection_stats.get(&intersection.id) {
                for (direction, &queue_length) in &stats.current_queue_lengths {
                    let severity = match queue_length {
                        0..=2 => continue, // No congestion
                        3..=5 => CongestionSeverity::Light,
                        6..=10 => CongestionSeverity::Moderate,
                        11..=20 => CongestionSeverity::Heavy,
                        _ => CongestionSeverity::Severe,
                    };
                    
                    let congestion_point = CongestionPoint {
                        intersection_id: intersection.id,
                        direction: *direction,
                        severity,
                        average_delay: stats.total_wait_time / stats.vehicles_processed.max(1) as f32,
                        detected_at: Instant::now(),
                    };
                    
                    self.flow_analysis.congestion_points.push(congestion_point);
                }
            }
        }
    }

    /// Calculate optimal light timings based on traffic patterns
    fn calculate_optimal_light_timings(&mut self, intersections: &[Intersection]) {
        for intersection in intersections {
            if let Some(stats) = self.intersection_stats.get(&intersection.id) {
                // Simple optimization based on queue lengths
                let total_waiting: usize = stats.current_queue_lengths.values().sum();
                
                if total_waiting > 0 {
                    let north_south_waiting = stats.current_queue_lengths.get(&Direction::North).unwrap_or(&0) +
                                            stats.current_queue_lengths.get(&Direction::South).unwrap_or(&0);
                    let east_west_waiting = stats.current_queue_lengths.get(&Direction::East).unwrap_or(&0) +
                                          stats.current_queue_lengths.get(&Direction::West).unwrap_or(&0);
                    
                    // Adjust green time based on relative demand
                    let ns_ratio = north_south_waiting as f32 / total_waiting as f32;
                    let ew_ratio = east_west_waiting as f32 / total_waiting as f32;
                    
                    let base_green = Duration::from_secs(8);
                    let ns_green = Duration::from_secs((base_green.as_secs_f32() * (1.0 + ns_ratio)).round() as u64);
                    let ew_green = Duration::from_secs((base_green.as_secs_f32() * (1.0 + ew_ratio)).round() as u64);
                    
                    let optimal_timing = LightTiming {
                        intersection_id: intersection.id,
                        recommended_green_duration: ns_green.max(ew_green),
                        recommended_yellow_duration: Duration::from_secs(2),
                        recommended_red_duration: Duration::from_secs(10),
                        confidence: (stats.vehicles_processed as f32 / 100.0).min(1.0),
                    };
                    
                    self.flow_analysis.optimal_light_timings.insert(intersection.id, optimal_timing);
                }
            }
        }
    }

    /// Identify bottleneck intersections
    fn identify_bottleneck_intersections(&mut self, intersections: &[Intersection]) {
        let mut intersection_scores: Vec<(u32, f32)> = intersections.iter()
            .map(|intersection| {
                let efficiency = intersection.get_efficiency_score();
                let waiting_count: usize = [Direction::North, Direction::South, Direction::East, Direction::West]
                    .iter()
                    .map(|&dir| intersection.get_waiting_count(dir))
                    .sum();
                
                // Lower score = more likely to be bottleneck
                let bottleneck_score = efficiency - (waiting_count as f32 * 5.0);
                (intersection.id, bottleneck_score)
            })
            .collect();
        
        // Sort by bottleneck score (ascending)
        intersection_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Take worst performing intersections as bottlenecks
        self.flow_analysis.bottleneck_intersections = intersection_scores
            .iter()
            .take(2) // Top 2 bottlenecks
            .filter(|(_, score)| *score < 70.0) // Only if significantly poor performance
            .map(|(id, _)| *id)
            .collect();
    }

    /// Update FPS tracking
    fn update_fps_tracking(&mut self, delta_time: f32) {
        if delta_time > 0.0 {
            let current_fps = 1.0 / delta_time as f64;
            // Smooth FPS calculation
            self.current_fps = self.current_fps * 0.9 + current_fps * 0.1;
        }
    }

    /// Get efficiency trend over time
    pub fn get_efficiency_trend(&self, duration: Duration) -> Vec<f32> {
        let cutoff_time = Instant::now() - duration;
        self.efficiency_history
            .iter()
            .filter(|snapshot| snapshot.timestamp >= cutoff_time)
            .map(|snapshot| snapshot.overall_efficiency)
            .collect()
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_vehicles: self.total_vehicles_spawned,
            active_vehicles: self.active_vehicles,
            efficiency_score: self.overall_efficiency,
            average_wait_time: self.average_wait_time,
            peak_wait_time: self.peak_wait_time,
            throughput: self.throughput_per_minute,
            fps: self.current_fps,
            uptime: self.total_simulation_time,
            congestion_points: self.flow_analysis.congestion_points.len(),
            bottleneck_count: self.flow_analysis.bottleneck_intersections.len(),
        }
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Enable/disable statistics collection
    pub fn set_enabled(&mut self, enabled: bool) {
        self.statistics_enabled = enabled;
    }

    /// Get vehicle type breakdown as percentages
    pub fn get_vehicle_type_percentages(&self) -> HashMap<VehicleType, f32> {
        let total = self.vehicle_counts.values().sum::<u32>() as f32;
        if total == 0.0 {
            return HashMap::new();
        }
        
        self.vehicle_counts.iter()
            .map(|(&vehicle_type, &count)| (vehicle_type, (count as f32 / total) * 100.0))
            .collect()
    }

    /// Get intersection efficiency rankings
    pub fn get_intersection_rankings(&self) -> Vec<(u32, f32)> {
        let mut rankings: Vec<_> = self.intersection_stats.iter()
            .map(|(&id, stats)| (id, stats.average_efficiency))
            .collect();
        
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings
    }
}

/// Performance summary for UI display
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_vehicles: u64,
    pub active_vehicles: u32,
    pub efficiency_score: f32,
    pub average_wait_time: f32,
    pub peak_wait_time: f32,
    pub throughput: f32,
    pub fps: f64,
    pub uptime: Duration,
    pub congestion_points: usize,
    pub bottleneck_count: usize,
}

impl PerformanceSummary {
    /// Get overall system health score (0-100)
    pub fn health_score(&self) -> f32 {
        let efficiency_factor = self.efficiency_score / 100.0;
        let fps_factor = (self.fps / 30.0).min(1.0) as f32;
        let congestion_factor = 1.0 - (self.congestion_points as f32 / 10.0).min(1.0);
        
        ((efficiency_factor + fps_factor + congestion_factor) / 3.0 * 100.0).max(0.0).min(100.0)
    }

    /// Get formatted uptime string
    pub fn uptime_string(&self) -> String {
        let total_seconds = self.uptime.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traffic::{Position, Direction};

    #[test]
    fn test_simulation_stats_creation() {
        let stats = SimulationStats::new();
        assert_eq!(stats.total_vehicles_spawned, 0);
        assert_eq!(stats.overall_efficiency, 100.0);
        assert!(stats.statistics_enabled);
    }

    #[test]
    fn test_efficiency_snapshot() {
        let snapshot = EfficiencySnapshot {
            timestamp: Instant::now(),
            overall_efficiency: 85.0,
            vehicle_count: 25,
            average_wait_time: 3.5,
            throughput: 12.5,
        };
        
        assert_eq!(snapshot.overall_efficiency, 85.0);
        assert_eq!(snapshot.vehicle_count, 25);
    }

    #[test]
    fn test_congestion_severity_ordering() {
        assert!(CongestionSeverity::Severe > CongestionSeverity::Heavy);
        assert!(CongestionSeverity::Heavy > CongestionSeverity::Moderate);
        assert!(CongestionSeverity::Moderate > CongestionSeverity::Light);
    }

    #[test]
    fn test_performance_summary_health_score() {
        let summary = PerformanceSummary {
            total_vehicles: 100,
            active_vehicles: 20,
            efficiency_score: 90.0,
            average_wait_time: 2.0,
            peak_wait_time: 5.0,
            throughput: 15.0,
            fps: 30.0,
            uptime: Duration::from_secs(300),
            congestion_points: 1,
            bottleneck_count: 0,
        };
        
        let health = summary.health_score();
        assert!(health > 80.0);
        assert!(health <= 100.0);
    }

    #[test]
    fn test_uptime_formatting() {
        let summary = PerformanceSummary {
            total_vehicles: 0,
            active_vehicles: 0,
            efficiency_score: 100.0,
            average_wait_time: 0.0,
            peak_wait_time: 0.0,
            throughput: 0.0,
            fps: 30.0,
            uptime: Duration::from_secs(3725), // 1h 2m 5s
            congestion_points: 0,
            bottleneck_count: 0,
        };
        
        assert_eq!(summary.uptime_string(), "1h 2m 5s");
    }

    #[test]
    fn test_intersection_stats() {
        let stats = IntersectionStats {
            id: 1,
            vehicles_processed: 150,
            average_efficiency: 85.5,
            total_wait_time: 120.0,
            emergency_activations: 3,
            light_cycle_count: 45,
            last_emergency_time: None,
            peak_queue_length: 8,
            current_queue_lengths: HashMap::new(),
        };
        
        assert_eq!(stats.id, 1);
        assert_eq!(stats.vehicles_processed, 150);
        assert_eq!(stats.peak_queue_length, 8);
    }
}
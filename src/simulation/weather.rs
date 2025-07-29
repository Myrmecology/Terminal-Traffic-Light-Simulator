//! Weather simulation system
//! 
//! This module handles dynamic weather conditions that affect traffic flow,
//! visibility, and vehicle behavior in the simulation.

use std::time::{Duration, Instant};
use rand::Rng;

/// Weather types available in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherType {
    Clear,
    LightRain,
    HeavyRain,
    Snow,
    Fog,
    Storm,
}

impl WeatherType {
    /// Get traffic flow multiplier for this weather type
    pub fn traffic_multiplier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::LightRain => 0.9,
            WeatherType::HeavyRain => 0.7,
            WeatherType::Snow => 0.6,
            WeatherType::Fog => 0.8,
            WeatherType::Storm => 0.5,
        }
    }

    /// Get visibility range multiplier
    pub fn visibility_multiplier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::LightRain => 0.9,
            WeatherType::HeavyRain => 0.7,
            WeatherType::Snow => 0.6,
            WeatherType::Fog => 0.4,
            WeatherType::Storm => 0.3,
        }
    }

    /// Get vehicle speed multiplier
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::LightRain => 0.95,
            WeatherType::HeavyRain => 0.8,
            WeatherType::Snow => 0.7,
            WeatherType::Fog => 0.85,
            WeatherType::Storm => 0.6,
        }
    }

    /// Get emergency vehicle spawn probability multiplier
    pub fn emergency_spawn_multiplier(&self) -> f32 {
        match self {
            WeatherType::Clear => 1.0,
            WeatherType::LightRain => 1.2,
            WeatherType::HeavyRain => 1.5,
            WeatherType::Snow => 1.8,
            WeatherType::Fog => 1.3,
            WeatherType::Storm => 2.0,
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::LightRain => "Light Rain",
            WeatherType::HeavyRain => "Heavy Rain",
            WeatherType::Snow => "Snow",
            WeatherType::Fog => "Fog",
            WeatherType::Storm => "Storm",
        }
    }

    /// Get all weather types for cycling
    pub fn all_types() -> &'static [WeatherType] {
        &[
            WeatherType::Clear,
            WeatherType::LightRain,
            WeatherType::HeavyRain,
            WeatherType::Snow,
            WeatherType::Fog,
            WeatherType::Storm,
        ]
    }

    /// Get random weather type
    pub fn random() -> WeatherType {
        let mut rng = rand::thread_rng();
        let types = Self::all_types();
        types[rng.gen_range(0..types.len())]
    }

    /// Get seasonal probability for this weather type
    pub fn seasonal_probability(&self, season: Season) -> f32 {
        match (self, season) {
            (WeatherType::Clear, _) => 0.4,
            (WeatherType::LightRain, Season::Spring) => 0.3,
            (WeatherType::LightRain, Season::Summer) => 0.2,
            (WeatherType::LightRain, Season::Fall) => 0.25,
            (WeatherType::LightRain, Season::Winter) => 0.15,
            (WeatherType::HeavyRain, Season::Spring) => 0.15,
            (WeatherType::HeavyRain, Season::Summer) => 0.1,
            (WeatherType::HeavyRain, Season::Fall) => 0.2,
            (WeatherType::HeavyRain, Season::Winter) => 0.05,
            (WeatherType::Snow, Season::Winter) => 0.3,
            (WeatherType::Snow, _) => 0.02,
            (WeatherType::Fog, Season::Fall) => 0.2,
            (WeatherType::Fog, Season::Winter) => 0.15,
            (WeatherType::Fog, _) => 0.08,
            (WeatherType::Storm, Season::Summer) => 0.1,
            (WeatherType::Storm, Season::Spring) => 0.08,
            (WeatherType::Storm, _) => 0.03,
        }
    }
}

/// Seasonal variations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

/// Weather transition patterns
#[derive(Debug, Clone)]
pub struct WeatherTransition {
    pub from: WeatherType,
    pub to: WeatherType,
    pub probability: f32,
    pub min_duration: Duration,
}

/// Weather system managing dynamic conditions
#[derive(Debug)]
pub struct WeatherSystem {
    pub current_weather: WeatherType,
    pub previous_weather: WeatherType,
    pub weather_start_time: Instant,
    pub weather_duration: Duration,
    pub min_weather_duration: Duration,
    pub max_weather_duration: Duration,
    pub enabled: bool,
    pub auto_change_enabled: bool,
    pub current_season: Season,
    pub intensity: f32, // 0.0 to 1.0
    pub transition_active: bool,
    pub transition_progress: f32,
    pub transition_duration: Duration,
    pub weather_transitions: Vec<WeatherTransition>,
    pub last_change_time: Instant,
}

impl WeatherSystem {
    /// Create new weather system
    pub fn new() -> Self {
        let mut system = Self {
            current_weather: WeatherType::Clear,
            previous_weather: WeatherType::Clear,
            weather_start_time: Instant::now(),
            weather_duration: Duration::from_secs(300), // 5 minutes default
            min_weather_duration: Duration::from_secs(60),
            max_weather_duration: Duration::from_secs(600),
            enabled: true,
            auto_change_enabled: true,
            current_season: Season::Spring,
            intensity: 1.0,
            transition_active: false,
            transition_progress: 0.0,
            transition_duration: Duration::from_secs(10),
            weather_transitions: Vec::new(),
            last_change_time: Instant::now(),
        };

        system.initialize_transitions();
        system
    }

    /// Initialize weather transition probabilities
    fn initialize_transitions(&mut self) {
        self.weather_transitions = vec![
            // From Clear
            WeatherTransition {
                from: WeatherType::Clear,
                to: WeatherType::LightRain,
                probability: 0.3,
                min_duration: Duration::from_secs(120),
            },
            WeatherTransition {
                from: WeatherType::Clear,
                to: WeatherType::Fog,
                probability: 0.15,
                min_duration: Duration::from_secs(180),
            },
            WeatherTransition {
                from: WeatherType::Clear,
                to: WeatherType::Snow,
                probability: 0.1,
                min_duration: Duration::from_secs(240),
            },

            // From Light Rain
            WeatherTransition {
                from: WeatherType::LightRain,
                to: WeatherType::Clear,
                probability: 0.4,
                min_duration: Duration::from_secs(90),
            },
            WeatherTransition {
                from: WeatherType::LightRain,
                to: WeatherType::HeavyRain,
                probability: 0.3,
                min_duration: Duration::from_secs(120),
            },
            WeatherTransition {
                from: WeatherType::LightRain,
                to: WeatherType::Storm,
                probability: 0.1,
                min_duration: Duration::from_secs(60),
            },

            // From Heavy Rain
            WeatherTransition {
                from: WeatherType::HeavyRain,
                to: WeatherType::LightRain,
                probability: 0.5,
                min_duration: Duration::from_secs(90),
            },
            WeatherTransition {
                from: WeatherType::HeavyRain,
                to: WeatherType::Clear,
                probability: 0.2,
                min_duration: Duration::from_secs(120),
            },
            WeatherTransition {
                from: WeatherType::HeavyRain,
                to: WeatherType::Storm,
                probability: 0.15,
                min_duration: Duration::from_secs(60),
            },

            // From Snow
            WeatherTransition {
                from: WeatherType::Snow,
                to: WeatherType::Clear,
                probability: 0.4,
                min_duration: Duration::from_secs(180),
            },
            WeatherTransition {
                from: WeatherType::Snow,
                to: WeatherType::Fog,
                probability: 0.2,
                min_duration: Duration::from_secs(120),
            },

            // From Fog
            WeatherTransition {
                from: WeatherType::Fog,
                to: WeatherType::Clear,
                probability: 0.6,
                min_duration: Duration::from_secs(120),
            },
            WeatherTransition {
                from: WeatherType::Fog,
                to: WeatherType::LightRain,
                probability: 0.2,
                min_duration: Duration::from_secs(90),
            },

            // From Storm
            WeatherTransition {
                from: WeatherType::Storm,
                to: WeatherType::HeavyRain,
                probability: 0.4,
                min_duration: Duration::from_secs(60),
            },
            WeatherTransition {
                from: WeatherType::Storm,
                to: WeatherType::LightRain,
                probability: 0.3,
                min_duration: Duration::from_secs(90),
            },
            WeatherTransition {
                from: WeatherType::Storm,
                to: WeatherType::Clear,
                probability: 0.2,
                min_duration: Duration::from_secs(180),
            },
        ];
    }

    /// Update weather system
    pub fn update(&mut self, delta_time: f32) {
        if !self.enabled {
            return;
        }

        self.update_transitions(delta_time);

        if self.auto_change_enabled {
            self.check_weather_change();
        }

        self.update_intensity();
    }

    /// Update weather transitions
    fn update_transitions(&mut self, delta_time: f32) {
        if self.transition_active {
            self.transition_progress += delta_time / self.transition_duration.as_secs_f32();
            
            if self.transition_progress >= 1.0 {
                self.transition_active = false;
                self.transition_progress = 0.0;
                self.previous_weather = self.current_weather;
            }
        }
    }

    /// Check if weather should change
    fn check_weather_change(&mut self) {
        let elapsed = self.weather_start_time.elapsed();
        
        if elapsed < self.min_weather_duration {
            return;
        }

        let mut rng = rand::thread_rng();
        
        // Increase probability of change based on how long current weather has lasted
        let change_probability = if elapsed > self.weather_duration {
            0.1 // 10% chance per update after duration expires
        } else {
            0.001 // 0.1% chance per update before duration
        };

        if rng.gen::<f32>() < change_probability {
            self.trigger_weather_change();
        }
    }

    /// Trigger a weather change
    fn trigger_weather_change(&mut self) {
        let possible_transitions: Vec<_> = self.weather_transitions
            .iter()
            .filter(|t| t.from == self.current_weather)
            .collect();

        if possible_transitions.is_empty() {
            return;
        }

        let mut rng = rand::thread_rng();
        let total_probability: f32 = possible_transitions.iter().map(|t| t.probability).sum();
        let mut roll = rng.gen::<f32>() * total_probability;

        for transition in possible_transitions {
            roll -= transition.probability;
            if roll <= 0.0 {
                self.change_weather(transition.to);
                break;
            }
        }
    }

    /// Change to new weather type
    fn change_weather(&mut self, new_weather: WeatherType) {
        if new_weather == self.current_weather {
            return;
        }

        self.previous_weather = self.current_weather;
        self.current_weather = new_weather;
        self.weather_start_time = Instant::now();
        self.last_change_time = Instant::now();
        
        // Set random duration for new weather
        let mut rng = rand::thread_rng();
        let duration_range = self.max_weather_duration.as_secs() - self.min_weather_duration.as_secs();
        let random_duration = self.min_weather_duration.as_secs() + rng.gen_range(0..=duration_range);
        self.weather_duration = Duration::from_secs(random_duration);

        // Start transition
        self.transition_active = true;
        self.transition_progress = 0.0;
    }

    /// Update weather intensity based on type and duration
    fn update_intensity(&mut self) {
        let elapsed = self.weather_start_time.elapsed().as_secs_f32();
        
        self.intensity = match self.current_weather {
            WeatherType::Clear => 1.0,
            WeatherType::LightRain => 0.6 + 0.4 * (elapsed / 60.0).min(1.0),
            WeatherType::HeavyRain => 0.8 + 0.2 * (elapsed / 30.0).min(1.0),
            WeatherType::Snow => 0.5 + 0.5 * (elapsed / 120.0).min(1.0),
            WeatherType::Fog => 0.3 + 0.7 * (elapsed / 90.0).min(1.0),
            WeatherType::Storm => 0.9 + 0.1 * (elapsed / 15.0).min(1.0),
        };
    }

    /// Set current weather manually
    pub fn set_current_weather(&mut self, weather: WeatherType) {
        if weather != self.current_weather {
            self.change_weather(weather);
        }
    }

    /// Get current weather
    pub fn get_current_weather(&self) -> WeatherType {
        self.current_weather
    }

    /// Get traffic flow multiplier considering current weather and intensity
    pub fn get_traffic_multiplier(&self) -> f32 {
        let base_multiplier = self.current_weather.traffic_multiplier();
        let intensity_factor = 0.5 + 0.5 * self.intensity;
        base_multiplier * intensity_factor
    }

    /// Get visibility multiplier
    pub fn get_visibility_multiplier(&self) -> f32 {
        let base_multiplier = self.current_weather.visibility_multiplier();
        let intensity_factor = 0.3 + 0.7 * self.intensity;
        base_multiplier * intensity_factor
    }

    /// Get speed multiplier for vehicles
    pub fn get_speed_multiplier(&self) -> f32 {
        let base_multiplier = self.current_weather.speed_multiplier();
        let intensity_factor = 0.6 + 0.4 * self.intensity;
        base_multiplier * intensity_factor
    }

    /// Get emergency spawn multiplier
    pub fn get_emergency_spawn_multiplier(&self) -> f32 {
        self.current_weather.emergency_spawn_multiplier() * self.intensity
    }

    /// Enable/disable weather system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.current_weather = WeatherType::Clear;
            self.intensity = 1.0;
            self.transition_active = false;
        }
    }

    /// Enable/disable automatic weather changes
    pub fn set_auto_change_enabled(&mut self, enabled: bool) {
        self.auto_change_enabled = enabled;
    }

    /// Set current season
    pub fn set_season(&mut self, season: Season) {
        self.current_season = season;
    }

    /// Get weather info for display
    pub fn get_weather_info(&self) -> WeatherInfo {
        WeatherInfo {
            current_weather: self.current_weather,
            intensity: self.intensity,
            duration_elapsed: self.weather_start_time.elapsed(),
            transitioning: self.transition_active,
            transition_progress: self.transition_progress,
        }
    }

    /// Cycle to next weather type (for manual control)
    pub fn cycle_weather(&mut self) {
        let current_index = WeatherType::all_types()
            .iter()
            .position(|&w| w == self.current_weather)
            .unwrap_or(0);
        
        let next_index = (current_index + 1) % WeatherType::all_types().len();
        let next_weather = WeatherType::all_types()[next_index];
        
        self.set_current_weather(next_weather);
    }

    /// Get time until next automatic weather change
    pub fn time_until_next_change(&self) -> Option<Duration> {
        if !self.auto_change_enabled {
            return None;
        }

        let elapsed = self.weather_start_time.elapsed();
        if elapsed >= self.weather_duration {
            Some(Duration::from_secs(0))
        } else {
            Some(self.weather_duration - elapsed)
        }
    }

    /// Check if weather affects traffic significantly
    pub fn has_significant_traffic_impact(&self) -> bool {
        self.get_traffic_multiplier() < 0.8
    }
}

/// Weather information for UI display
#[derive(Debug, Clone)]
pub struct WeatherInfo {
    pub current_weather: WeatherType,
    pub intensity: f32,
    pub duration_elapsed: Duration,
    pub transitioning: bool,
    pub transition_progress: f32,
}

impl WeatherInfo {
    /// Get display string for current weather
    pub fn display_string(&self) -> String {
        let intensity_desc = match self.intensity {
            i if i < 0.3 => "Light",
            i if i < 0.7 => "Moderate",
            _ => "Heavy",
        };

        match self.current_weather {
            WeatherType::Clear => "Clear".to_string(),
            WeatherType::LightRain => format!("{} Rain", intensity_desc),
            WeatherType::HeavyRain => "Heavy Rain".to_string(),
            WeatherType::Snow => format!("{} Snow", intensity_desc),
            WeatherType::Fog => format!("{} Fog", intensity_desc),
            WeatherType::Storm => "Storm".to_string(),
        }
    }

    /// Get weather status for UI
    pub fn status_string(&self) -> String {
        if self.transitioning {
            format!("Changing... ({:.0}%)", self.transition_progress * 100.0)
        } else {
            format!("Stable ({})", format_duration(self.duration_elapsed))
        }
    }
}

/// Format duration for display
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    
    if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_type_multipliers() {
        assert_eq!(WeatherType::Clear.traffic_multiplier(), 1.0);
        assert!(WeatherType::Storm.traffic_multiplier() < 1.0);
        assert!(WeatherType::Snow.speed_multiplier() < WeatherType::Clear.speed_multiplier());
    }

    #[test]
    fn test_weather_system_creation() {
        let system = WeatherSystem::new();
        assert_eq!(system.current_weather, WeatherType::Clear);
        assert!(system.enabled);
        assert!(system.auto_change_enabled);
    }

    #[test]
    fn test_weather_change() {
        let mut system = WeatherSystem::new();
        system.set_current_weather(WeatherType::Rain);
        assert_eq!(system.current_weather, WeatherType::Rain);
    }

    #[test]
    fn test_weather_transitions() {
        let system = WeatherSystem::new();
        assert!(!system.weather_transitions.is_empty());
        
        let clear_transitions: Vec<_> = system.weather_transitions
            .iter()
            .filter(|t| t.from == WeatherType::Clear)
            .collect();
        assert!(!clear_transitions.is_empty());
    }

    #[test]
    fn test_seasonal_probabilities() {
        assert!(WeatherType::Snow.seasonal_probability(Season::Winter) > 
                WeatherType::Snow.seasonal_probability(Season::Summer));
        assert!(WeatherType::Storm.seasonal_probability(Season::Summer) >
                WeatherType::Storm.seasonal_probability(Season::Winter));
    }

    #[test]
    fn test_weather_info_display() {
        let info = WeatherInfo {
            current_weather: WeatherType::LightRain,
            intensity: 0.8,
            duration_elapsed: Duration::from_secs(150),
            transitioning: false,
            transition_progress: 0.0,
        };

        let display = info.display_string();
        assert!(display.contains("Rain"));
    }

    #[test]
    fn test_weather_cycling() {
        let mut system = WeatherSystem::new();
        let initial_weather = system.current_weather;
        system.cycle_weather();
        assert_ne!(system.current_weather, initial_weather);
    }
}
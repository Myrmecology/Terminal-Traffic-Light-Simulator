//! Configuration management module
//! 
//! This module handles all configuration settings for the traffic simulation,
//! including default values, file loading, and runtime configuration changes.

pub mod settings;

pub use settings::*;

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};

/// Main configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub simulation: SimulationConfig,
    pub rendering: RenderingConfig,
    pub traffic: TrafficConfig,
    pub weather: WeatherConfig,
    pub ui: UiConfig,
    pub performance: PerformanceConfig,
    pub debug: DebugConfig,
}

/// Simulation-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub target_fps: u32,
    pub time_scale: f32,
    pub max_vehicles: usize,
    pub auto_start: bool,
    pub enable_statistics: bool,
    pub enable_events: bool,
    pub enable_weather: bool,
    pub enable_emergency_vehicles: bool,
    pub random_seed: Option<u64>,
}

/// Rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    pub screen_width: usize,
    pub screen_height: usize,
    pub enable_colors: bool,
    pub enable_animations: bool,
    pub enable_weather_effects: bool,
    pub frame_buffer_size: usize,
    pub vsync_enabled: bool,
    pub ascii_style: AsciiStyle,
}

/// ASCII art style options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsciiStyle {
    Simple,
    Enhanced,
    Unicode,
    Retro,
}

/// Traffic system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficConfig {
    pub base_spawn_rate: f32,
    pub emergency_vehicle_probability: f32,
    pub truck_probability: f32,
    pub default_green_duration: u64, // seconds
    pub default_yellow_duration: u64,
    pub default_red_duration: u64,
    pub intersection_positions: Vec<(i32, i32)>,
    pub spawn_positions: Vec<SpawnPositionConfig>,
    pub enable_adaptive_lights: bool,
    pub enable_rush_hour: bool,
}

/// Spawn position configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPositionConfig {
    pub x: i32,
    pub y: i32,
    pub direction: DirectionConfig,
    pub spawn_rate: f32,
    pub enabled: bool,
}

/// Direction configuration (serializable version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DirectionConfig {
    North,
    South,
    East,
    West,
}

impl From<DirectionConfig> for crate::traffic::Direction {
    fn from(config: DirectionConfig) -> Self {
        match config {
            DirectionConfig::North => crate::traffic::Direction::North,
            DirectionConfig::South => crate::traffic::Direction::South,
            DirectionConfig::East => crate::traffic::Direction::East,
            DirectionConfig::West => crate::traffic::Direction::West,
        }
    }
}

impl From<crate::traffic::Direction> for DirectionConfig {
    fn from(direction: crate::traffic::Direction) -> Self {
        match direction {
            crate::traffic::Direction::North => DirectionConfig::North,
            crate::traffic::Direction::South => DirectionConfig::South,
            crate::traffic::Direction::East => DirectionConfig::East,
            crate::traffic::Direction::West => DirectionConfig::West,
        }
    }
}

/// Weather system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub enabled: bool,
    pub auto_change: bool,
    pub initial_weather: WeatherTypeConfig,
    pub change_probability: f32,
    pub min_duration: u64, // seconds
    pub max_duration: u64,
    pub seasonal_effects: bool,
    pub weather_intensity: f32,
}

/// Weather type configuration (serializable version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherTypeConfig {
    Clear,
    LightRain,
    HeavyRain,
    Snow,
    Fog,
    Storm,
}

impl From<WeatherTypeConfig> for crate::simulation::WeatherType {
    fn from(config: WeatherTypeConfig) -> Self {
        match config {
            WeatherTypeConfig::Clear => crate::simulation::WeatherType::Clear,
            WeatherTypeConfig::LightRain => crate::simulation::WeatherType::LightRain,
            WeatherTypeConfig::HeavyRain => crate::simulation::WeatherType::HeavyRain,
            WeatherTypeConfig::Snow => crate::simulation::WeatherType::Snow,
            WeatherTypeConfig::Fog => crate::simulation::WeatherType::Fog,
            WeatherTypeConfig::Storm => crate::simulation::WeatherType::Storm,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_status_panel: bool,
    pub show_statistics_panel: bool,
    pub show_controls_panel: bool,
    pub enable_alerts: bool,
    pub alert_duration: u64, // seconds
    pub update_frequency: u32, // Hz
    pub color_scheme: ColorSchemeConfig,
}

/// Color scheme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSchemeConfig {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub error: String,
    pub warning: String,
    pub success: String,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_optimizations: bool,
    pub max_fps: u32,
    pub enable_dirty_rendering: bool,
    pub buffer_size: usize,
    pub garbage_collection_frequency: u32,
    pub statistics_history_size: usize,
    pub enable_profiling: bool,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    pub enable_debug_mode: bool,
    pub show_fps: bool,
    pub show_vehicle_ids: bool,
    pub show_intersection_stats: bool,
    pub enable_logging: bool,
    pub log_level: LogLevel,
    pub log_file: Option<String>,
    pub enable_hot_reload: bool,
}

/// Logging level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            simulation: SimulationConfig {
                target_fps: 30,
                time_scale: 1.0,
                max_vehicles: 100,
                auto_start: true,
                enable_statistics: true,
                enable_events: true,
                enable_weather: true,
                enable_emergency_vehicles: true,
                random_seed: None,
            },
            rendering: RenderingConfig {
                screen_width: 120,
                screen_height: 40,
                enable_colors: true,
                enable_animations: true,
                enable_weather_effects: true,
                frame_buffer_size: 4800, // width * height
                vsync_enabled: true,
                ascii_style: AsciiStyle::Enhanced,
            },
            traffic: TrafficConfig {
                base_spawn_rate: 0.5,
                emergency_vehicle_probability: 0.03,
                truck_probability: 0.18,
                default_green_duration: 8,
                default_yellow_duration: 2,
                default_red_duration: 10,
                intersection_positions: vec![
                    (20, 15),
                    (60, 15),
                    (20, 25),
                    (60, 25),
                ],
                spawn_positions: vec![
                    SpawnPositionConfig { x: 5, y: 15, direction: DirectionConfig::East, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 80, y: 15, direction: DirectionConfig::West, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 20, y: 5, direction: DirectionConfig::South, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 20, y: 35, direction: DirectionConfig::North, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 5, y: 25, direction: DirectionConfig::East, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 80, y: 25, direction: DirectionConfig::West, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 60, y: 5, direction: DirectionConfig::South, spawn_rate: 0.5, enabled: true },
                    SpawnPositionConfig { x: 60, y: 35, direction: DirectionConfig::North, spawn_rate: 0.5, enabled: true },
                ],
                enable_adaptive_lights: true,
                enable_rush_hour: true,
            },
            weather: WeatherConfig {
                enabled: true,
                auto_change: true,
                initial_weather: WeatherTypeConfig::Clear,
                change_probability: 0.001,
                min_duration: 60,
                max_duration: 600,
                seasonal_effects: true,
                weather_intensity: 1.0,
            },
            ui: UiConfig {
                show_status_panel: true,
                show_statistics_panel: true,
                show_controls_panel: true,
                enable_alerts: true,
                alert_duration: 5,
                update_frequency: 30,
                color_scheme: ColorSchemeConfig {
                    primary: "#00ff00".to_string(),
                    secondary: "#ffff00".to_string(),
                    accent: "#ff0000".to_string(),
                    background: "#000000".to_string(),
                    text: "#ffffff".to_string(),
                    error: "#ff0000".to_string(),
                    warning: "#ffff00".to_string(),
                    success: "#00ff00".to_string(),
                },
            },
            performance: PerformanceConfig {
                enable_optimizations: true,
                max_fps: 60,
                enable_dirty_rendering: true,
                buffer_size: 8192,
                garbage_collection_frequency: 60,
                statistics_history_size: 300,
                enable_profiling: false,
            },
            debug: DebugConfig {
                enable_debug_mode: false,
                show_fps: true,
                show_vehicle_ids: false,
                show_intersection_stats: false,
                enable_logging: false,
                log_level: LogLevel::Info,
                log_file: None,
                enable_hot_reload: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileError(e.to_string()))?;
        
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        fs::write(path, content)
            .map_err(|e| ConfigError::FileError(e.to_string()))?;
        
        Ok(())
    }

    /// Load configuration from file or create default
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load_from_file(&path).unwrap_or_else(|_| {
            let config = Self::default();
            // Try to save default config for next time
            let _ = config.save_to_file(&path);
            config
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate simulation config
        if self.simulation.target_fps == 0 || self.simulation.target_fps > 120 {
            return Err(ConfigError::ValidationError("target_fps must be between 1 and 120".to_string()));
        }

        if self.simulation.time_scale <= 0.0 || self.simulation.time_scale > 10.0 {
            return Err(ConfigError::ValidationError("time_scale must be between 0.1 and 10.0".to_string()));
        }

        if self.simulation.max_vehicles == 0 || self.simulation.max_vehicles > 1000 {
            return Err(ConfigError::ValidationError("max_vehicles must be between 1 and 1000".to_string()));
        }

        // Validate rendering config
        if self.rendering.screen_width < 80 || self.rendering.screen_height < 24 {
            return Err(ConfigError::ValidationError("screen size must be at least 80x24".to_string()));
        }

        // Validate traffic config
        if self.traffic.base_spawn_rate < 0.0 || self.traffic.base_spawn_rate > 10.0 {
            return Err(ConfigError::ValidationError("base_spawn_rate must be between 0.0 and 10.0".to_string()));
        }

        if self.traffic.emergency_vehicle_probability < 0.0 || self.traffic.emergency_vehicle_probability > 1.0 {
            return Err(ConfigError::ValidationError("emergency_vehicle_probability must be between 0.0 and 1.0".to_string()));
        }

        // Validate weather config
        if self.weather.change_probability < 0.0 || self.weather.change_probability > 1.0 {
            return Err(ConfigError::ValidationError("weather change_probability must be between 0.0 and 1.0".to_string()));
        }

        if self.weather.min_duration >= self.weather.max_duration {
            return Err(ConfigError::ValidationError("min_duration must be less than max_duration".to_string()));
        }

        // Validate performance config
        if self.performance.max_fps > 120 {
            return Err(ConfigError::ValidationError("max_fps should not exceed 120".to_string()));
        }

        Ok(())
    }

    /// Merge with another config (other takes precedence)
    pub fn merge(&mut self, other: &Config) {
        // This is a simple implementation - in practice you might want more sophisticated merging
        *self = other.clone();
    }

    /// Get a subset of config for a specific system
    pub fn get_simulation_config(&self) -> &SimulationConfig {
        &self.simulation
    }

    pub fn get_rendering_config(&self) -> &RenderingConfig {
        &self.rendering
    }

    pub fn get_traffic_config(&self) -> &TrafficConfig {
        &self.traffic
    }

    pub fn get_weather_config(&self) -> &WeatherConfig {
        &self.weather
    }

    pub fn get_ui_config(&self) -> &UiConfig {
        &self.ui
    }

    pub fn get_performance_config(&self) -> &PerformanceConfig {
        &self.performance
    }

    pub fn get_debug_config(&self) -> &DebugConfig {
        &self.debug
    }

    /// Apply runtime configuration changes
    pub fn apply_runtime_changes(&mut self, changes: RuntimeConfigChanges) {
        if let Some(fps) = changes.target_fps {
            self.simulation.target_fps = fps;
        }
        if let Some(scale) = changes.time_scale {
            self.simulation.time_scale = scale;
        }
        if let Some(max_vehicles) = changes.max_vehicles {
            self.simulation.max_vehicles = max_vehicles;
        }
        if let Some(spawn_rate) = changes.spawn_rate {
            self.traffic.base_spawn_rate = spawn_rate;
        }
        if let Some(weather_enabled) = changes.weather_enabled {
            self.weather.enabled = weather_enabled;
        }
    }
}

/// Runtime configuration changes
#[derive(Debug, Default)]
pub struct RuntimeConfigChanges {
    pub target_fps: Option<u32>,
    pub time_scale: Option<f32>,
    pub max_vehicles: Option<usize>,
    pub spawn_rate: Option<f32>,
    pub weather_enabled: Option<bool>,
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    FileError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileError(msg) => write!(f, "File error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration manager for handling runtime config updates
#[derive(Debug)]
pub struct ConfigManager {
    pub config: Config,
    pub config_file_path: Option<String>,
    pub auto_save: bool,
    pub last_modified: Option<std::time::SystemTime>,
}

impl ConfigManager {
    /// Create a new config manager
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            config_file_path: None,
            auto_save: false,
            last_modified: None,
        }
    }

    /// Create config manager with file path
    pub fn with_file<P: AsRef<Path>>(path: P) -> Self {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let config = Config::load_or_default(&path);
        
        Self {
            config,
            config_file_path: Some(path_str),
            auto_save: true,
            last_modified: None,
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, changes: RuntimeConfigChanges) -> Result<(), ConfigError> {
        self.config.apply_runtime_changes(changes);
        self.config.validate()?;
        
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(ref path) = self.config_file_path {
            self.config.save_to_file(path)
        } else {
            Err(ConfigError::FileError("No config file path set".to_string()))
        }
    }

    /// Reload configuration from file
    pub fn reload(&mut self) -> Result<(), ConfigError> {
        if let Some(ref path) = self.config_file_path {
            self.config = Config::load_from_file(path)?;
            Ok(())
        } else {
            Err(ConfigError::FileError("No config file path set".to_string()))
        }
    }

    /// Check if config file has been modified externally
    pub fn check_for_external_changes(&mut self) -> Result<bool, ConfigError> {
        if let Some(ref path) = self.config_file_path {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if let Some(last_modified) = self.last_modified {
                        if modified > last_modified {
                            self.last_modified = Some(modified);
                            return Ok(true);
                        }
                    } else {
                        self.last_modified = Some(modified);
                    }
                }
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.simulation.target_fps, 30);
        assert_eq!(config.simulation.max_vehicles, 100);
        assert!(config.simulation.enable_statistics);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        
        config.simulation.target_fps = 0;
        assert!(config.validate().is_err());
        
        config.simulation.target_fps = 30;
        config.simulation.time_scale = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_file_operations() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = Config::default();
        
        // Test saving
        assert!(config.save_to_file(temp_file.path()).is_ok());
        
        // Test loading
        let loaded_config = Config::load_from_file(temp_file.path()).unwrap();
        assert_eq!(loaded_config.simulation.target_fps, config.simulation.target_fps);
    }

    #[test]
    fn test_runtime_config_changes() {
        let mut config = Config::default();
        let changes = RuntimeConfigChanges {
            target_fps: Some(60),
            time_scale: Some(2.0),
            max_vehicles: Some(200),
            spawn_rate: Some(1.0),
            weather_enabled: Some(false),
        };
        
        config.apply_runtime_changes(changes);
        assert_eq!(config.simulation.target_fps, 60);
        assert_eq!(config.simulation.time_scale, 2.0);
        assert_eq!(config.simulation.max_vehicles, 200);
        assert_eq!(config.traffic.base_spawn_rate, 1.0);
        assert!(!config.weather.enabled);
    }

    #[test]
    fn test_config_manager() {
        let manager = ConfigManager::new();
        assert_eq!(manager.config.simulation.target_fps, 30);
        assert!(!manager.auto_save);
        assert!(manager.config_file_path.is_none());
    }

    #[test]
    fn test_direction_conversion() {
        let config_dir = DirectionConfig::North;
        let traffic_dir: crate::traffic::Direction = config_dir.into();
        assert_eq!(traffic_dir, crate::traffic::Direction::North);
        
        let converted_back: DirectionConfig = traffic_dir.into();
        assert!(matches!(converted_back, DirectionConfig::North));
    }
}
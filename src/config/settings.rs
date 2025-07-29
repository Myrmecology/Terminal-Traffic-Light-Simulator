//! Settings management and configuration utilities
//! 
//! This module provides additional utilities for managing settings,
//! including preset configurations, environment variable overrides,
//! and command-line argument parsing.

use clap::{Arg, Command, ArgMatches};
use std::env;
use std::collections::HashMap;

use crate::config::{Config, ConfigError, RuntimeConfigChanges};

/// Application settings manager
#[derive(Debug)]
pub struct Settings {
    pub config: Config,
    pub command_line_overrides: HashMap<String, String>,
    pub environment_overrides: HashMap<String, String>,
    pub preset_name: Option<String>,
}

/// Predefined configuration presets
#[derive(Debug, Clone)]
pub enum ConfigPreset {
    Demo,
    Performance,
    Debug,
    LowEnd,
    HighEnd,
    Educational,
}

impl Settings {
    /// Create new settings with default configuration
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            command_line_overrides: HashMap::new(),
            environment_overrides: HashMap::new(),
            preset_name: None,
        }
    }

    /// Create settings from command line arguments
    pub fn from_args() -> Result<Self, ConfigError> {
        let matches = Self::build_cli().try_get_matches()
            .map_err(|e| ConfigError::ParseError(format!("Command line parsing error: {}", e)))?;
        
        let mut settings = Self::new();
        settings.apply_command_line_args(&matches)?;
        settings.apply_environment_variables();
        
        Ok(settings)
    }

    /// Build CLI argument parser
    fn build_cli() -> Command {
        Command::new("Terminal Traffic Light Simulator")
            .version("0.1.0")
            .author("Your Name")
            .about("A fast, secure terminal-based traffic light simulator built in Rust")
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("Sets a custom config file")
                    .num_args(1)
            )
            .arg(
                Arg::new("preset")
                    .short('p')
                    .long("preset")
                    .value_name("PRESET")
                    .help("Use a configuration preset")
                    .value_parser(["demo", "performance", "debug", "lowend", "highend", "educational"])
                    .num_args(1)
            )
            .arg(
                Arg::new("fps")
                    .long("fps")
                    .value_name("FPS")
                    .help("Target frames per second")
                    .value_parser(clap::value_parser!(u32))
                    .num_args(1)
            )
            .arg(
                Arg::new("vehicles")
                    .long("max-vehicles")
                    .value_name("COUNT")
                    .help("Maximum number of vehicles")
                    .value_parser(clap::value_parser!(usize))
                    .num_args(1)
            )
            .arg(
                Arg::new("spawn-rate")
                    .long("spawn-rate")
                    .value_name("RATE")
                    .help("Vehicle spawn rate (vehicles per second)")
                    .value_parser(clap::value_parser!(f32))
                    .num_args(1)
            )
            .arg(
                Arg::new("time-scale")
                    .long("time-scale")
                    .value_name("SCALE")
                    .help("Simulation time scale multiplier")
                    .value_parser(clap::value_parser!(f32))
                    .num_args(1)
            )
            .arg(
                Arg::new("no-weather")
                    .long("no-weather")
                    .help("Disable weather effects")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("no-emergency")
                    .long("no-emergency")
                    .help("Disable emergency vehicles")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("no-statistics")
                    .long("no-statistics")
                    .help("Disable statistics collection")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("debug")
                    .short('d')
                    .long("debug")
                    .help("Enable debug mode")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("fullscreen")
                    .short('f')
                    .long("fullscreen")
                    .help("Use full terminal size")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("seed")
                    .long("seed")
                    .value_name("SEED")
                    .help("Random seed for reproducible simulations")
                    .value_parser(clap::value_parser!(u64))
                    .num_args(1)
            )
            .arg(
                Arg::new("width")
                    .long("width")
                    .value_name("WIDTH")
                    .help("Screen width in characters")
                    .value_parser(clap::value_parser!(usize))
                    .num_args(1)
            )
            .arg(
                Arg::new("height")
                    .long("height")
                    .value_name("HEIGHT")
                    .help("Screen height in characters")
                    .value_parser(clap::value_parser!(usize))
                    .num_args(1)
            )
    }

    /// Apply command line arguments to configuration
    fn apply_command_line_args(&mut self, matches: &ArgMatches) -> Result<(), ConfigError> {
        // Load custom config file if specified
        if let Some(config_file) = matches.get_one::<String>("config") {
            self.config = Config::load_from_file(config_file)?;
        }

        // Apply preset if specified
        if let Some(preset_name) = matches.get_one::<String>("preset") {
            let preset = match preset_name.as_str() {
                "demo" => ConfigPreset::Demo,
                "performance" => ConfigPreset::Performance,
                "debug" => ConfigPreset::Debug,
                "lowend" => ConfigPreset::LowEnd,
                "highend" => ConfigPreset::HighEnd,
                "educational" => ConfigPreset::Educational,
                _ => return Err(ConfigError::ValidationError("Invalid preset name".to_string())),
            };
            self.apply_preset(preset);
            self.preset_name = Some(preset_name.clone());
        }

        // Apply individual settings
        if let Some(&fps) = matches.get_one::<u32>("fps") {
            self.config.simulation.target_fps = fps;
        }

        if let Some(&max_vehicles) = matches.get_one::<usize>("vehicles") {
            self.config.simulation.max_vehicles = max_vehicles;
        }

        if let Some(&spawn_rate) = matches.get_one::<f32>("spawn-rate") {
            self.config.traffic.base_spawn_rate = spawn_rate;
        }

        if let Some(&time_scale) = matches.get_one::<f32>("time-scale") {
            self.config.simulation.time_scale = time_scale;
        }

        if let Some(&seed) = matches.get_one::<u64>("seed") {
            self.config.simulation.random_seed = Some(seed);
        }

        if let Some(&width) = matches.get_one::<usize>("width") {
            self.config.rendering.screen_width = width;
        }

        if let Some(&height) = matches.get_one::<usize>("height") {
            self.config.rendering.screen_height = height;
        }

        // Apply boolean flags
        if matches.get_flag("no-weather") {
            self.config.weather.enabled = false;
        }

        if matches.get_flag("no-emergency") {
            self.config.simulation.enable_emergency_vehicles = false;
        }

        if matches.get_flag("no-statistics") {
            self.config.simulation.enable_statistics = false;
        }

        if matches.get_flag("debug") {
            self.config.debug.enable_debug_mode = true;
            self.config.debug.show_fps = true;
            self.config.debug.show_vehicle_ids = true;
            self.config.debug.show_intersection_stats = true;
        }

        if matches.get_flag("fullscreen") {
            // Will be handled by terminal detection
            self.config.rendering.screen_width = 0; // Special value for auto-detect
            self.config.rendering.screen_height = 0;
        }

        self.config.validate()?;
        Ok(())
    }

    /// Apply environment variable overrides
    fn apply_environment_variables(&mut self) {
        let env_vars = [
            ("TRAFFIC_SIM_FPS", "simulation.target_fps"),
            ("TRAFFIC_SIM_MAX_VEHICLES", "simulation.max_vehicles"),
            ("TRAFFIC_SIM_SPAWN_RATE", "traffic.base_spawn_rate"),
            ("TRAFFIC_SIM_TIME_SCALE", "simulation.time_scale"),
            ("TRAFFIC_SIM_ENABLE_WEATHER", "weather.enabled"),
            ("TRAFFIC_SIM_ENABLE_DEBUG", "debug.enable_debug_mode"),
            ("TRAFFIC_SIM_SCREEN_WIDTH", "rendering.screen_width"),
            ("TRAFFIC_SIM_SCREEN_HEIGHT", "rendering.screen_height"),
        ];

        for (env_var, config_path) in env_vars.iter() {
            if let Ok(value) = env::var(env_var) {
                self.environment_overrides.insert(config_path.to_string(), value.clone());
                self.apply_environment_override(config_path, &value);
            }
        }
    }

    /// Apply a single environment variable override
    fn apply_environment_override(&mut self, config_path: &str, value: &str) {
        match config_path {
            "simulation.target_fps" => {
                if let Ok(fps) = value.parse::<u32>() {
                    self.config.simulation.target_fps = fps;
                }
            }
            "simulation.max_vehicles" => {
                if let Ok(max_vehicles) = value.parse::<usize>() {
                    self.config.simulation.max_vehicles = max_vehicles;
                }
            }
            "traffic.base_spawn_rate" => {
                if let Ok(spawn_rate) = value.parse::<f32>() {
                    self.config.traffic.base_spawn_rate = spawn_rate;
                }
            }
            "simulation.time_scale" => {
                if let Ok(time_scale) = value.parse::<f32>() {
                    self.config.simulation.time_scale = time_scale;
                }
            }
            "weather.enabled" => {
                if let Ok(enabled) = value.parse::<bool>() {
                    self.config.weather.enabled = enabled;
                }
            }
            "debug.enable_debug_mode" => {
                if let Ok(enabled) = value.parse::<bool>() {
                    self.config.debug.enable_debug_mode = enabled;
                }
            }
            "rendering.screen_width" => {
                if let Ok(width) = value.parse::<usize>() {
                    self.config.rendering.screen_width = width;
                }
            }
            "rendering.screen_height" => {
                if let Ok(height) = value.parse::<usize>() {
                    self.config.rendering.screen_height = height;
                }
            }
            _ => {} // Unknown config path
        }
    }

    /// Apply a configuration preset
    pub fn apply_preset(&mut self, preset: ConfigPreset) {
        match preset {
            ConfigPreset::Demo => {
                self.config.simulation.target_fps = 30;
                self.config.simulation.max_vehicles = 50;
                self.config.traffic.base_spawn_rate = 0.8;
                self.config.simulation.time_scale = 1.5;
                self.config.weather.enabled = true;
                self.config.weather.auto_change = true;
                self.config.simulation.enable_emergency_vehicles = true;
                self.config.ui.show_status_panel = true;
                self.config.ui.show_statistics_panel = true;
                self.config.ui.show_controls_panel = true;
                self.config.debug.show_fps = true;
            }
            ConfigPreset::Performance => {
                self.config.simulation.target_fps = 60;
                self.config.simulation.max_vehicles = 200;
                self.config.traffic.base_spawn_rate = 1.5;
                self.config.simulation.time_scale = 1.0;
                self.config.weather.enabled = false;
                self.config.rendering.enable_animations = false;
                self.config.rendering.enable_weather_effects = false;
                self.config.performance.enable_optimizations = true;
                self.config.performance.enable_dirty_rendering = true;
                self.config.simulation.enable_statistics = false;
            }
            ConfigPreset::Debug => {
                self.config.simulation.target_fps = 15;
                self.config.simulation.max_vehicles = 20;
                self.config.traffic.base_spawn_rate = 0.2;
                self.config.simulation.time_scale = 0.5;
                self.config.debug.enable_debug_mode = true;
                self.config.debug.show_fps = true;
                self.config.debug.show_vehicle_ids = true;
                self.config.debug.show_intersection_stats = true;
                self.config.debug.enable_logging = true;
                self.config.debug.log_level = crate::config::LogLevel::Debug;
            }
            ConfigPreset::LowEnd => {
                self.config.simulation.target_fps = 20;
                self.config.simulation.max_vehicles = 30;
                self.config.traffic.base_spawn_rate = 0.3;
                self.config.rendering.enable_colors = true;
                self.config.rendering.enable_animations = false;
                self.config.rendering.enable_weather_effects = false;
                self.config.rendering.ascii_style = crate::config::AsciiStyle::Simple;
                self.config.weather.enabled = false;
                self.config.performance.enable_optimizations = true;
                self.config.ui.show_statistics_panel = false;
            }
            ConfigPreset::HighEnd => {
                self.config.simulation.target_fps = 60;
                self.config.simulation.max_vehicles = 300;
                self.config.traffic.base_spawn_rate = 2.0;
                self.config.rendering.enable_colors = true;
                self.config.rendering.enable_animations = true;
                self.config.rendering.enable_weather_effects = true;
                self.config.rendering.ascii_style = crate::config::AsciiStyle::Unicode;
                self.config.weather.enabled = true;
                self.config.weather.auto_change = true;
                self.config.performance.max_fps = 120;
                self.config.performance.statistics_history_size = 600;
            }
            ConfigPreset::Educational => {
                self.config.simulation.target_fps = 20;
                self.config.simulation.max_vehicles = 40;
                self.config.traffic.base_spawn_rate = 0.5;
                self.config.simulation.time_scale = 0.8;
                self.config.ui.show_status_panel = true;
                self.config.ui.show_statistics_panel = true;
                self.config.ui.show_controls_panel = true;
                self.config.debug.show_fps = true;
                self.config.simulation.enable_emergency_vehicles = true;
                self.config.weather.enabled = true;
                self.config.weather.auto_change = false; // Manual control for education
            }
        }
    }

    /// Get available presets
    pub fn get_available_presets() -> Vec<(&'static str, &'static str)> {
        vec![
            ("demo", "Demonstration mode with balanced settings"),
            ("performance", "High performance mode for stress testing"),
            ("debug", "Debug mode with detailed information"),
            ("lowend", "Optimized for low-end systems"),
            ("highend", "Full features for high-end systems"),
            ("educational", "Educational mode with manual controls"),
        ]
    }

    /// Validate current settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.config.validate()
    }

    /// Get runtime configuration changes
    pub fn get_runtime_changes(&self) -> RuntimeConfigChanges {
        RuntimeConfigChanges {
            target_fps: Some(self.config.simulation.target_fps),
            time_scale: Some(self.config.simulation.time_scale),
            max_vehicles: Some(self.config.simulation.max_vehicles),
            spawn_rate: Some(self.config.traffic.base_spawn_rate),
            weather_enabled: Some(self.config.weather.enabled),
        }
    }

    /// Print configuration summary
    pub fn print_summary(&self) {
        println!("=== Traffic Light Simulator Configuration ===");
        if let Some(ref preset) = self.preset_name {
            println!("Preset: {}", preset);
        }
        println!("Target FPS: {}", self.config.simulation.target_fps);
        println!("Max Vehicles: {}", self.config.simulation.max_vehicles);
        println!("Spawn Rate: {:.2} vehicles/sec", self.config.traffic.base_spawn_rate);
        println!("Time Scale: {:.1}x", self.config.simulation.time_scale);
        println!("Screen Size: {}x{}", self.config.rendering.screen_width, self.config.rendering.screen_height);
        println!("Weather: {}", if self.config.weather.enabled { "Enabled" } else { "Disabled" });
        println!("Emergency Vehicles: {}", if self.config.simulation.enable_emergency_vehicles { "Enabled" } else { "Disabled" });
        println!("Statistics: {}", if self.config.simulation.enable_statistics { "Enabled" } else { "Disabled" });
        println!("Debug Mode: {}", if self.config.debug.enable_debug_mode { "Enabled" } else { "Disabled" });
        
        if !self.command_line_overrides.is_empty() {
            println!("\nCommand Line Overrides:");
            for (key, value) in &self.command_line_overrides {
                println!("  {}: {}", key, value);
            }
        }
        
        if !self.environment_overrides.is_empty() {
            println!("\nEnvironment Variable Overrides:");
            for (key, value) in &self.environment_overrides {
                println!("  {}: {}", key, value);
            }
        }
        println!("============================================");
    }

    /// Check if terminal size should be auto-detected
    pub fn should_auto_detect_size(&self) -> bool {
        self.config.rendering.screen_width == 0 || self.config.rendering.screen_height == 0
    }

    /// Update screen size (for auto-detection)
    pub fn update_screen_size(&mut self, width: usize, height: usize) {
        if self.should_auto_detect_size() {
            self.config.rendering.screen_width = width;
            self.config.rendering.screen_height = height;
        }
    }

    /// Get help text for configuration options
    pub fn get_help_text() -> String {
        let mut help = String::new();
        help.push_str("Configuration Options:\n\n");
        help.push_str("Command Line Arguments:\n");
        help.push_str("  --config FILE         Load configuration from file\n");
        help.push_str("  --preset PRESET       Use configuration preset\n");
        help.push_str("  --fps FPS             Set target FPS\n");
        help.push_str("  --max-vehicles COUNT  Set maximum vehicles\n");
        help.push_str("  --spawn-rate RATE     Set spawn rate\n");
        help.push_str("  --time-scale SCALE    Set time scale\n");
        help.push_str("  --no-weather          Disable weather\n");
        help.push_str("  --no-emergency        Disable emergency vehicles\n");
        help.push_str("  --debug               Enable debug mode\n");
        help.push_str("  --fullscreen          Use full terminal size\n\n");
        
        help.push_str("Environment Variables:\n");
        help.push_str("  TRAFFIC_SIM_FPS              Target FPS\n");
        help.push_str("  TRAFFIC_SIM_MAX_VEHICLES     Maximum vehicles\n");
        help.push_str("  TRAFFIC_SIM_SPAWN_RATE       Spawn rate\n");
        help.push_str("  TRAFFIC_SIM_TIME_SCALE       Time scale\n");
        help.push_str("  TRAFFIC_SIM_ENABLE_WEATHER   Enable weather\n");
        help.push_str("  TRAFFIC_SIM_ENABLE_DEBUG     Enable debug mode\n\n");
        
        help.push_str("Available Presets:\n");
        for (name, description) in Self::get_available_presets() {
            help.push_str(&format!("  {:<12} {}\n", name, description));
        }
        
        help
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_creation() {
        let settings = Settings::new();
        assert_eq!(settings.config.simulation.target_fps, 30);
        assert!(settings.command_line_overrides.is_empty());
        assert!(settings.environment_overrides.is_empty());
    }

    #[test]
    fn test_preset_application() {
        let mut settings = Settings::new();
        settings.apply_preset(ConfigPreset::Performance);
        
        assert_eq!(settings.config.simulation.target_fps, 60);
        assert_eq!(settings.config.simulation.max_vehicles, 200);
        assert!(!settings.config.weather.enabled);
        assert!(settings.config.performance.enable_optimizations);
    }

    #[test]
    fn test_demo_preset() {
        let mut settings = Settings::new();
        settings.apply_preset(ConfigPreset::Demo);
        
        assert_eq!(settings.config.simulation.target_fps, 30);
        assert_eq!(settings.config.simulation.max_vehicles, 50);
        assert_eq!(settings.config.simulation.time_scale, 1.5);
        assert!(settings.config.weather.enabled);
        assert!(settings.config.simulation.enable_emergency_vehicles);
    }

    #[test]
    fn test_debug_preset() {
        let mut settings = Settings::new();
        settings.apply_preset(ConfigPreset::Debug);
        
        assert_eq!(settings.config.simulation.target_fps, 15);
        assert_eq!(settings.config.simulation.max_vehicles, 20);
        assert!(settings.config.debug.enable_debug_mode);
        assert!(settings.config.debug.show_fps);
        assert!(settings.config.debug.show_vehicle_ids);
    }

    #[test]
    fn test_environment_override() {
        let mut settings = Settings::new();
        settings.apply_environment_override("simulation.target_fps", "45");
        assert_eq!(settings.config.simulation.target_fps, 45);
        
        settings.apply_environment_override("weather.enabled", "false");
        assert!(!settings.config.weather.enabled);
    }

    #[test]
    fn test_auto_detect_size() {
        let mut settings = Settings::new();
        assert!(!settings.should_auto_detect_size());
        
        settings.config.rendering.screen_width = 0;
        settings.config.rendering.screen_height = 0;
        assert!(settings.should_auto_detect_size());
        
        settings.update_screen_size(100, 50);
        assert_eq!(settings.config.rendering.screen_width, 100);
        assert_eq!(settings.config.rendering.screen_height, 50);
    }

    #[test]
    fn test_available_presets() {
        let presets = Settings::get_available_presets();
        assert!(!presets.is_empty());
        assert!(presets.iter().any(|(name, _)| *name == "demo"));
        assert!(presets.iter().any(|(name, _)| *name == "performance"));
    }

    #[test]
    fn test_runtime_changes() {
        let settings = Settings::new();
        let changes = settings.get_runtime_changes();
        assert!(changes.target_fps.is_some());
        assert!(changes.time_scale.is_some());
        assert!(changes.max_vehicles.is_some());
    }
}
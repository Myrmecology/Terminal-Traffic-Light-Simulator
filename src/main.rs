//! Terminal Traffic Light Simulator
//! 
//! A fast, secure, and interactive traffic simulation system built entirely in Rust.
//! Features real-time traffic management, weather effects, emergency vehicles,
//! and comprehensive performance monitoring.

mod traffic;
mod rendering;
mod simulation;
mod config;

use std::time::{Duration, Instant};
use std::thread;

use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::terminal;

use crate::config::{Settings, ConfigError};
use crate::rendering::{Terminal, InputHandler, InputEvent, ScreenBuffer, UserInterface, FrameRate};
use crate::rendering::{CityRenderer, WeatherType as RenderWeatherType, TimeOfDay};
use crate::simulation::{SimulationEngine, SimulationError, WeatherType};
use crate::traffic::Position;

/// Main application structure
#[derive(Debug)]
struct TrafficSimulatorApp {
    terminal: Terminal,
    input_handler: InputHandler,
    screen_buffer: ScreenBuffer,
    user_interface: UserInterface,
    city_renderer: CityRenderer,
    simulation_engine: SimulationEngine,
    frame_rate: FrameRate,
    settings: Settings,
    running: bool,
    paused: bool,
    last_fps_update: Instant,
    frame_count: u64,
}

impl TrafficSimulatorApp {
    /// Create new application instance
    fn new() -> Result<Self, ApplicationError> {
        // Load settings from command line and environment
        let mut settings = Settings::from_args()
            .map_err(|e| ApplicationError::ConfigurationError(e))?;

        // Initialize terminal
        let mut terminal = Terminal::new()
            .map_err(|e| ApplicationError::TerminalError(format!("Failed to initialize terminal: {}", e)))?;

        // Auto-detect terminal size if needed
        if settings.should_auto_detect_size() {
            let (width, height) = terminal.size()
                .map_err(|e| ApplicationError::TerminalError(format!("Failed to get terminal size: {}", e)))?;
            settings.update_screen_size(width as usize, height as usize);
        }

        // Validate final configuration
        settings.validate()
            .map_err(|e| ApplicationError::ConfigurationError(e))?;

        // Print configuration summary if debug mode is enabled
        if settings.config.debug.enable_debug_mode {
            settings.print_summary();
        }

        // Initialize components
        let input_handler = InputHandler::new();
        let screen_buffer = ScreenBuffer::new(
            settings.config.rendering.screen_width,
            settings.config.rendering.screen_height,
        );
        let user_interface = UserInterface::new();
        let city_renderer = CityRenderer::new();

        // Create simulation configuration
        let sim_config = crate::simulation::SimulationConfig {
            target_fps: settings.config.simulation.target_fps,
            max_vehicles: settings.config.simulation.max_vehicles,
            base_spawn_rate: settings.config.traffic.base_spawn_rate,
            intersection_positions: settings.config.traffic.intersection_positions
                .iter()
                .map(|(x, y)| Position::new(*x, *y))
                .collect(),
            spawn_positions: settings.config.traffic.spawn_positions
                .iter()
                .map(|sp| (Position::new(sp.x, sp.y), sp.direction.into()))
                .collect(),
            enable_weather: settings.config.weather.enabled,
            enable_emergency_vehicles: settings.config.simulation.enable_emergency_vehicles,
            time_scale: settings.config.simulation.time_scale,
        };

        let simulation_engine = SimulationEngine::new(sim_config);
        let frame_rate = FrameRate::new(settings.config.simulation.target_fps);

        Ok(Self {
            terminal,
            input_handler,
            screen_buffer,
            user_interface,
            city_renderer,
            simulation_engine,
            frame_rate,
            settings,
            running: false,
            paused: false,
            last_fps_update: Instant::now(),
            frame_count: 0,
        })
    }

    /// Run the main application loop
    fn run(&mut self) -> Result<(), ApplicationError> {
        self.running = true;
        self.simulation_engine.start();

        // Add welcome message
        self.user_interface.add_alert(
            "Welcome to Traffic Light Simulator! Press 'H' for help.".to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(5),
        );

        println!("Starting Traffic Light Simulator...");
        println!("Press 'Q' to quit, 'H' for help.");

        while self.running {
            // Handle input events
            self.handle_input()?;

            // Update simulation if not paused
            if !self.paused {
                self.update_simulation()?;
            }

            // Render frame
            self.render_frame()?;

            // Handle frame rate limiting
            self.frame_rate.wait_for_next_frame();
            self.update_fps_counter();

            // Check for terminal resize
            self.handle_resize()?;
        }

        self.shutdown()?;
        Ok(())
    }

    /// Handle input events
    fn handle_input(&mut self) -> Result<(), ApplicationError> {
        if let Some(event) = self.input_handler.poll_input(Duration::from_millis(1))
            .map_err(|e| ApplicationError::InputError(format!("Input polling error: {}", e)))? {
            
            match event {
                InputEvent::Quit => {
                    self.running = false;
                }
                InputEvent::Char('q') | InputEvent::Char('Q') => {
                    self.running = false;
                }
                InputEvent::Char('h') | InputEvent::Char('H') => {
                    self.show_help();
                }
                InputEvent::Char(' ') => {
                    self.toggle_pause();
                }
                InputEvent::Char('e') | InputEvent::Char('E') => {
                    self.trigger_emergency_vehicle();
                }
                InputEvent::Char('r') | InputEvent::Char('R') => {
                    self.toggle_rain();
                }
                InputEvent::Char('s') | InputEvent::Char('S') => {
                    self.toggle_snow();
                }
                InputEvent::Char('f') | InputEvent::Char('F') => {
                    self.toggle_fog();
                }
                InputEvent::Char('c') | InputEvent::Char('C') => {
                    self.set_clear_weather();
                }
                InputEvent::Char('+') => {
                    self.increase_traffic();
                }
                InputEvent::Char('-') => {
                    self.decrease_traffic();
                }
                InputEvent::Char('1') => {
                    self.set_time_scale(0.5);
                }
                InputEvent::Char('2') => {
                    self.set_time_scale(1.0);
                }
                InputEvent::Char('3') => {
                    self.set_time_scale(2.0);
                }
                InputEvent::Char('d') | InputEvent::Char('D') => {
                    self.toggle_debug_mode();
                }
                InputEvent::Char('i') | InputEvent::Char('I') => {
                    self.show_statistics();
                }
                InputEvent::Escape => {
                    self.user_interface.alert_system.clear_current();
                }
                InputEvent::Resize(width, height) => {
                    self.handle_terminal_resize(width, height)?;
                }
                _ => {} // Ignore other inputs
            }
        }

        Ok(())
    }

    /// Update simulation state
    fn update_simulation(&mut self) -> Result<(), ApplicationError> {
        self.simulation_engine.update()
            .map_err(|e| ApplicationError::SimulationError(e))?;

        // Update UI with simulation statistics
        let stats = self.simulation_engine.get_statistics();
        self.user_interface.update(stats);

        // Update weather display
        let current_weather = self.simulation_engine.get_current_weather();
        self.user_interface.set_weather(weather_type_to_string(current_weather));

        // Update emergency status
        let has_emergency = self.simulation_engine.has_emergency_vehicles();
        self.user_interface.set_emergency_active(has_emergency);

        // Update city renderer weather
        self.city_renderer.set_weather(convert_weather_type(current_weather));
        self.city_renderer.update_weather_frame();

        Ok(())
    }

    /// Render a frame
    fn render_frame(&mut self) -> Result<(), ApplicationError> {
        // Clear screen buffer
        self.screen_buffer.clear();

        // Render city background
        self.city_renderer.render_city_background(&mut self.screen_buffer);

        // Get simulation area from UI
        let sim_area = self.user_interface.get_simulation_area();

        // Render intersections
        for intersection in &self.simulation_engine.intersections {
            let screen_x = sim_area.x + (intersection.position.x as usize).saturating_sub(sim_area.x);
            let screen_y = sim_area.y + (intersection.position.y as usize).saturating_sub(sim_area.y);

            if screen_x < sim_area.x + sim_area.width && screen_y < sim_area.y + sim_area.height {
                let ns_light = intersection.get_light_state(crate::traffic::Direction::North)
                    .unwrap_or(crate::traffic::lights::LightState::Red);
                let ew_light = intersection.get_light_state(crate::traffic::Direction::East)
                    .unwrap_or(crate::traffic::lights::LightState::Red);

                self.city_renderer.render_intersection(
                    &mut self.screen_buffer,
                    screen_x,
                    screen_y,
                    ns_light,
                    ew_light,
                );
            }
        }

        // Render vehicles
        for vehicle in &self.simulation_engine.vehicles {
            let screen_x = sim_area.x + (vehicle.position.x as usize).saturating_sub(sim_area.x);
            let screen_y = sim_area.y + (vehicle.position.y as usize).saturating_sub(sim_area.y);

            if screen_x < sim_area.x + sim_area.width && screen_y < sim_area.y + sim_area.height {
                self.city_renderer.render_vehicle(
                    &mut self.screen_buffer,
                    screen_x,
                    screen_y,
                    vehicle.vehicle_type,
                    vehicle.direction,
                );
            }
        }

        // Render roads connecting intersections
        self.render_roads();

        // Render weather effects
        self.city_renderer.render_weather(&mut self.screen_buffer);

        // Render UI
        self.user_interface.render(&mut self.screen_buffer);

        // Update FPS display
        if self.settings.config.debug.show_fps {
            let fps_text = format!("FPS: {:.1}", self.frame_rate.current_fps());
            self.screen_buffer.draw_text(
                self.screen_buffer.width - 15,
                0,
                &fps_text,
                crossterm::style::Color::Yellow,
            );
        }

        // Render to terminal
        self.terminal.render_buffer(&mut self.screen_buffer)
            .map_err(|e| ApplicationError::RenderError(format!("Render error: {}", e)))?;

        Ok(())
    }

    /// Render roads between intersections
    fn render_roads(&mut self) {
        let sim_area = self.user_interface.get_simulation_area();
        
        // Horizontal roads
        for y in [15, 25] {
            let screen_y = sim_area.y + y.saturating_sub(sim_area.y);
            if screen_y < sim_area.y + sim_area.height {
                self.city_renderer.render_road(
                    &mut self.screen_buffer,
                    sim_area.x + 8,
                    screen_y,
                    crate::traffic::Direction::East,
                    40,
                );
            }
        }

        // Vertical roads
        for x in [20, 60] {
            let screen_x = sim_area.x + x.saturating_sub(sim_area.x);
            if screen_x < sim_area.x + sim_area.width {
                self.city_renderer.render_road(
                    &mut self.screen_buffer,
                    screen_x,
                    sim_area.y + 8,
                    crate::traffic::Direction::North,
                    20,
                );
            }
        }
    }

    /// Handle terminal resize
    fn handle_resize(&mut self) -> Result<(), ApplicationError> {
        if let Some((width, height)) = self.terminal.check_resize()
            .map_err(|e| ApplicationError::TerminalError(format!("Resize check error: {}", e)))? {
            self.handle_terminal_resize(width, height)?;
        }
        Ok(())
    }

    /// Handle terminal resize event
    fn handle_terminal_resize(&mut self, width: u16, height: u16) -> Result<(), ApplicationError> {
        let new_width = width as usize;
        let new_height = height as usize;

        // Update screen buffer
        self.screen_buffer = ScreenBuffer::new(new_width, new_height);

        // Update UI layout
        self.user_interface.resize(new_width, new_height);

        // Clear screen
        self.terminal.clear()
            .map_err(|e| ApplicationError::TerminalError(format!("Clear error: {}", e)))?;

        Ok(())
    }

    /// Update FPS counter
    fn update_fps_counter(&mut self) {
        self.frame_count += 1;
        
        if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
            let fps = self.frame_count as f64 / self.last_fps_update.elapsed().as_secs_f64();
            self.user_interface.statistics_panel.current_fps = fps;
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
        }
    }

    /// Toggle pause state
    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        let message = if self.paused {
            "Simulation Paused - Press SPACE to resume"
        } else {
            "Simulation Resumed"
        };
        
        self.user_interface.add_alert(
            message.to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Show help message
    fn show_help(&mut self) {
        self.user_interface.add_alert(
            "Controls: E=Emergency, R=Rain, S=Snow, F=Fog, C=Clear, +/-=Traffic, SPACE=Pause, Q=Quit".to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(8),
        );
    }

    /// Trigger emergency vehicle
    fn trigger_emergency_vehicle(&mut self) {
        if let Err(e) = self.simulation_engine.trigger_emergency_vehicle(0) {
            self.user_interface.add_alert(
                format!("Failed to trigger emergency vehicle: {}", e),
                crate::rendering::AlertLevel::Warning,
                Duration::from_secs(3),
            );
        } else {
            self.user_interface.add_alert(
                "Emergency vehicle dispatched!".to_string(),
                crate::rendering::AlertLevel::Emergency,
                Duration::from_secs(3),
            );
        }
    }

    /// Toggle rain weather
    fn toggle_rain(&mut self) {
        let current_weather = self.simulation_engine.get_current_weather();
        let new_weather = match current_weather {
            WeatherType::LightRain | WeatherType::HeavyRain => WeatherType::Clear,
            _ => WeatherType::LightRain,
        };
        
        self.simulation_engine.set_weather(new_weather);
        self.user_interface.add_alert(
            format!("Weather changed to: {}", weather_type_to_string(new_weather)),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Toggle snow weather
    fn toggle_snow(&mut self) {
        let current_weather = self.simulation_engine.get_current_weather();
        let new_weather = if current_weather == WeatherType::Snow {
            WeatherType::Clear
        } else {
            WeatherType::Snow
        };
        
        self.simulation_engine.set_weather(new_weather);
        self.user_interface.add_alert(
            format!("Weather changed to: {}", weather_type_to_string(new_weather)),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Toggle fog weather
    fn toggle_fog(&mut self) {
        let current_weather = self.simulation_engine.get_current_weather();
        let new_weather = if current_weather == WeatherType::Fog {
            WeatherType::Clear
        } else {
            WeatherType::Fog
        };
        
        self.simulation_engine.set_weather(new_weather);
        self.user_interface.add_alert(
            format!("Weather changed to: {}", weather_type_to_string(new_weather)),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Set clear weather
    fn set_clear_weather(&mut self) {
        self.simulation_engine.set_weather(WeatherType::Clear);
        self.user_interface.add_alert(
            "Weather cleared".to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Increase traffic density
    fn increase_traffic(&mut self) {
        self.simulation_engine.increase_traffic_density(1.2);
        self.user_interface.add_alert(
            "Traffic density increased".to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Decrease traffic density
    fn decrease_traffic(&mut self) {
        self.simulation_engine.decrease_traffic_density(0.8);
        self.user_interface.add_alert(
            "Traffic density decreased".to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Set simulation time scale
    fn set_time_scale(&mut self, scale: f32) {
        self.simulation_engine.set_time_scale(scale);
        self.user_interface.add_alert(
            format!("Time scale set to {:.1}x", scale),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Toggle debug mode
    fn toggle_debug_mode(&mut self) {
        self.settings.config.debug.enable_debug_mode = !self.settings.config.debug.enable_debug_mode;
        let message = if self.settings.config.debug.enable_debug_mode {
            "Debug mode enabled"
        } else {
            "Debug mode disabled"
        };
        
        self.user_interface.add_alert(
            message.to_string(),
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(2),
        );
    }

    /// Show statistics
    fn show_statistics(&mut self) {
        let stats = self.simulation_engine.get_statistics();
        let message = format!(
            "Vehicles: {} active, {} total | Efficiency: {:.1}% | FPS: {:.1}",
            stats.active_vehicles,
            stats.total_vehicles_spawned,
            stats.overall_efficiency,
            self.frame_rate.current_fps()
        );
        
        self.user_interface.add_alert(
            message,
            crate::rendering::AlertLevel::Info,
            Duration::from_secs(5),
        );
    }

    /// Shutdown application gracefully
    fn shutdown(&mut self) -> Result<(), ApplicationError> {
        self.simulation_engine.stop();
        
        self.terminal.restore()
            .map_err(|e| ApplicationError::TerminalError(format!("Terminal restore error: {}", e)))?;
        
        println!("Traffic Light Simulator stopped. Thanks for using it!");
        Ok(())
    }
}

/// Convert simulation weather type to string
fn weather_type_to_string(weather: WeatherType) -> String {
    match weather {
        WeatherType::Clear => "Clear".to_string(),
        WeatherType::LightRain => "Light Rain".to_string(),
        WeatherType::HeavyRain => "Heavy Rain".to_string(),
        WeatherType::Snow => "Snow".to_string(),
        WeatherType::Fog => "Fog".to_string(),
        WeatherType::Storm => "Storm".to_string(),
    }
}

/// Convert simulation weather type to rendering weather type
fn convert_weather_type(weather: WeatherType) -> RenderWeatherType {
    match weather {
        WeatherType::Clear => RenderWeatherType::Clear,
        WeatherType::LightRain | WeatherType::HeavyRain | WeatherType::Storm => RenderWeatherType::Rain,
        WeatherType::Snow => RenderWeatherType::Snow,
        WeatherType::Fog => RenderWeatherType::Fog,
    }
}

/// Application error types
#[derive(Debug)]
enum ApplicationError {
    ConfigurationError(ConfigError),
    TerminalError(String),
    InputError(String),
    RenderError(String),
    SimulationError(SimulationError),
    SystemError(String),
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::ConfigurationError(e) => write!(f, "Configuration error: {}", e),
            ApplicationError::TerminalError(e) => write!(f, "Terminal error: {}", e),
            ApplicationError::InputError(e) => write!(f, "Input error: {}", e),
            ApplicationError::RenderError(e) => write!(f, "Render error: {}", e),
            ApplicationError::SimulationError(e) => write!(f, "Simulation error: {}", e),
            ApplicationError::SystemError(e) => write!(f, "System error: {}", e),
        }
    }
}

impl std::error::Error for ApplicationError {}

/// Main entry point
fn main() {
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down gracefully...");
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Run application
    match TrafficSimulatorApp::new() {
        Ok(mut app) => {
            if let Err(e) = app.run() {
                eprintln!("Application error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_type_conversion() {
        assert_eq!(weather_type_to_string(WeatherType::Clear), "Clear");
        assert_eq!(weather_type_to_string(WeatherType::LightRain), "Light Rain");
        assert_eq!(weather_type_to_string(WeatherType::Snow), "Snow");
    }

    #[test]
    fn test_render_weather_conversion() {
        assert_eq!(convert_weather_type(WeatherType::Clear), RenderWeatherType::Clear);
        assert_eq!(convert_weather_type(WeatherType::LightRain), RenderWeatherType::Rain);
        assert_eq!(convert_weather_type(WeatherType::Snow), RenderWeatherType::Snow);
        assert_eq!(convert_weather_type(WeatherType::Fog), RenderWeatherType::Fog);
    }

    #[test]
    fn test_application_error_display() {
        let error = ApplicationError::SystemError("Test error".to_string());
        assert_eq!(format!("{}", error), "System error: Test error");
    }
}

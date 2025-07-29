//! ASCII art and graphics for the traffic simulation
//! 
//! This module contains all ASCII art elements including city layouts,
//! vehicle sprites, intersection graphics, and weather effects.

use crossterm::style::Color;
use crate::rendering::{ScreenBuffer, ScreenCell};
use crate::traffic::{Direction, VehicleType};
use std::collections::HashMap;

/// ASCII art templates for different elements
#[derive(Debug, Clone)]
pub struct AsciiArt {
    pub intersection_template: Vec<String>,
    pub road_templates: HashMap<Direction, Vec<String>>,
    pub vehicle_sprites: HashMap<(VehicleType, Direction), String>,
    pub weather_effects: HashMap<WeatherType, Vec<String>>,
    pub ui_elements: HashMap<UiElement, Vec<String>>,
}

/// Weather effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeatherType {
    Clear,
    Rain,
    Snow,
    Fog,
}

/// UI element types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiElement {
    StatusPanel,
    TrafficLight,
    EmergencyAlert,
    StatisticsBoard,
    ControlsHelp,
}

impl AsciiArt {
    /// Create new ASCII art collection
    pub fn new() -> Self {
        let mut art = Self {
            intersection_template: Vec::new(),
            road_templates: HashMap::new(),
            vehicle_sprites: HashMap::new(),
            weather_effects: HashMap::new(),
            ui_elements: HashMap::new(),
        };

        art.initialize_intersection();
        art.initialize_roads();
        art.initialize_vehicles();
        art.initialize_weather();
        art.initialize_ui_elements();

        art
    }

    /// Initialize intersection graphics
    fn initialize_intersection(&mut self) {
        self.intersection_template = vec![
            "┌─────┬─────┐".to_string(),
            "│     │     │".to_string(),
            "│     │     │".to_string(),
            "├─────┼─────┤".to_string(),
            "│     │     │".to_string(),
            "│     │     │".to_string(),
            "└─────┴─────┘".to_string(),
        ];
    }

    /// Initialize road graphics
    fn initialize_roads(&mut self) {
        // Horizontal road
        self.road_templates.insert(Direction::East, vec![
            "═══════════".to_string(),
            "───────────".to_string(),
            "═══════════".to_string(),
        ]);

        // Vertical road
        self.road_templates.insert(Direction::North, vec![
            "║".to_string(),
            "│".to_string(),
            "║".to_string(),
        ]);

        // Copy for opposite directions
        self.road_templates.insert(Direction::West, 
            self.road_templates[&Direction::East].clone());
        self.road_templates.insert(Direction::South, 
            self.road_templates[&Direction::North].clone());
    }

    /// Initialize vehicle sprites
    fn initialize_vehicles(&mut self) {
        // Cars
        self.vehicle_sprites.insert((VehicleType::Car, Direction::North), "▲".to_string());
        self.vehicle_sprites.insert((VehicleType::Car, Direction::South), "▼".to_string());
        self.vehicle_sprites.insert((VehicleType::Car, Direction::East), "►".to_string());
        self.vehicle_sprites.insert((VehicleType::Car, Direction::West), "◄".to_string());

        // Trucks
        self.vehicle_sprites.insert((VehicleType::Truck, Direction::North), "█".to_string());
        self.vehicle_sprites.insert((VehicleType::Truck, Direction::South), "█".to_string());
        self.vehicle_sprites.insert((VehicleType::Truck, Direction::East), "█".to_string());
        self.vehicle_sprites.insert((VehicleType::Truck, Direction::West), "█".to_string());

        // Emergency vehicles
        self.vehicle_sprites.insert((VehicleType::Emergency, Direction::North), "♦".to_string());
        self.vehicle_sprites.insert((VehicleType::Emergency, Direction::South), "♦".to_string());
        self.vehicle_sprites.insert((VehicleType::Emergency, Direction::East), "♦".to_string());
        self.vehicle_sprites.insert((VehicleType::Emergency, Direction::West), "♦".to_string());
    }

    /// Initialize weather effects
    fn initialize_weather(&mut self) {
        self.weather_effects.insert(WeatherType::Clear, vec![]);

        self.weather_effects.insert(WeatherType::Rain, vec![
            "╱".to_string(),
            "╲".to_string(),
            "╱".to_string(),
            "╲".to_string(),
        ]);

        self.weather_effects.insert(WeatherType::Snow, vec![
            "❄".to_string(),
            "❅".to_string(),
            "❆".to_string(),
            "*".to_string(),
        ]);

        self.weather_effects.insert(WeatherType::Fog, vec![
            "░".to_string(),
            "▒".to_string(),
            "▓".to_string(),
        ]);
    }

    /// Initialize UI elements
    fn initialize_ui_elements(&mut self) {
        // Status panel
        self.ui_elements.insert(UiElement::StatusPanel, vec![
            "╔══════════════════════╗".to_string(),
            "║ TRAFFIC CONTROL SYS  ║".to_string(),
            "║ Status: OPERATIONAL  ║".to_string(),
            "╠══════════════════════╣".to_string(),
            "║ Vehicles: 000        ║".to_string(),
            "║ Efficiency: 100%     ║".to_string(),
            "║ Emergency: OFF       ║".to_string(),
            "╚══════════════════════╝".to_string(),
        ]);

        // Traffic light display
        self.ui_elements.insert(UiElement::TrafficLight, vec![
            "┌─┐".to_string(),
            "│●│".to_string(),
            "│●│".to_string(),
            "│●│".to_string(),
            "└─┘".to_string(),
        ]);

        // Emergency alert
        self.ui_elements.insert(UiElement::EmergencyAlert, vec![
            "⚠⚠⚠ EMERGENCY VEHICLE ⚠⚠⚠".to_string(),
            "    ALL LIGHTS OVERRIDE     ".to_string(),
            "     PLEASE STAND BY       ".to_string(),
        ]);

        // Statistics board
        self.ui_elements.insert(UiElement::StatisticsBoard, vec![
            "╔════════ STATISTICS ════════╗".to_string(),
            "║ Cars Processed: 0000       ║".to_string(),
            "║ Average Wait: 00.0s        ║".to_string(),
            "║ Efficiency Score: 100%     ║".to_string(),
            "║ Current FPS: 30.0          ║".to_string(),
            "╚════════════════════════════╝".to_string(),
        ]);

        // Controls help
        self.ui_elements.insert(UiElement::ControlsHelp, vec![
            "╔════════ CONTROLS ═══════════╗".to_string(),
            "║ E - Emergency Vehicle       ║".to_string(),
            "║ R - Toggle Rain             ║".to_string(),
            "║ S - Toggle Snow             ║".to_string(),
            "║ F - Toggle Fog              ║".to_string(),
            "║ + - Increase Traffic        ║".to_string(),
            "║ - - Decrease Traffic        ║".to_string(),
            "║ Q - Quit                    ║".to_string(),
            "╚═════════════════════════════╝".to_string(),
        ]);
    }

    /// Get vehicle sprite for direction and type
    pub fn get_vehicle_sprite(&self, vehicle_type: VehicleType, direction: Direction) -> &str {
        self.vehicle_sprites
            .get(&(vehicle_type, direction))
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    /// Get weather effect sprite
    pub fn get_weather_sprite(&self, weather: WeatherType, frame: usize) -> Option<&str> {
        self.weather_effects
            .get(&weather)?
            .get(frame % self.weather_effects[&weather].len())
            .map(|s| s.as_str())
    }

    /// Get UI element template
    pub fn get_ui_template(&self, element: UiElement) -> Option<&[String]> {
        self.ui_elements.get(&element).map(|v| v.as_slice())
    }
}

/// City layout renderer
#[derive(Debug)]
pub struct CityRenderer {
    pub ascii_art: AsciiArt,
    pub weather: WeatherType,
    pub weather_frame: usize,
    pub time_of_day: TimeOfDay,
}

/// Time of day for different visual themes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

impl CityRenderer {
    /// Create new city renderer
    pub fn new() -> Self {
        Self {
            ascii_art: AsciiArt::new(),
            weather: WeatherType::Clear,
            weather_frame: 0,
            time_of_day: TimeOfDay::Day,
        }
    }

    /// Render intersection at position
    pub fn render_intersection(&self, buffer: &mut ScreenBuffer, x: usize, y: usize, 
                              north_south_light: crate::traffic::lights::LightState,
                              east_west_light: crate::traffic::lights::LightState) {
        // Draw intersection base
        for (dy, line) in self.ascii_art.intersection_template.iter().enumerate() {
            for (dx, ch) in line.chars().enumerate() {
                if x + dx < buffer.width && y + dy < buffer.height {
                    let color = self.get_intersection_color(ch);
                    buffer.set_char(x + dx, y + dy, ScreenCell::with_colors(ch, color, Color::Black));
                }
            }
        }

        // Draw traffic lights
        self.render_traffic_light(buffer, x + 1, y + 1, north_south_light);
        self.render_traffic_light(buffer, x + 5, y + 1, east_west_light);
        self.render_traffic_light(buffer, x + 1, y + 5, north_south_light);
        self.render_traffic_light(buffer, x + 5, y + 5, east_west_light);
    }

    /// Render single traffic light
    fn render_traffic_light(&self, buffer: &mut ScreenBuffer, x: usize, y: usize, 
                           state: crate::traffic::lights::LightState) {
        let (color, symbol) = match state {
            crate::traffic::lights::LightState::Red => (Color::Red, '●'),
            crate::traffic::lights::LightState::Yellow => (Color::Yellow, '●'),
            crate::traffic::lights::LightState::Green => (Color::Green, '●'),
        };

        if x < buffer.width && y < buffer.height {
            buffer.set_char(x, y, ScreenCell::with_colors(symbol, color, Color::Black));
        }
    }

    /// Get color for intersection elements
    fn get_intersection_color(&self, ch: char) -> Color {
        match ch {
            '┌' | '┬' | '┐' | '├' | '┼' | '┤' | '└' | '┴' | '┘' | '│' | '─' => {
                match self.time_of_day {
                    TimeOfDay::Day => Color::White,
                    TimeOfDay::Night => Color::DarkGrey,
                    TimeOfDay::Dawn | TimeOfDay::Dusk => Color::Grey,
                }
            }
            _ => Color::DarkGrey,
        }
    }

    /// Render road segment
    pub fn render_road(&self, buffer: &mut ScreenBuffer, x: usize, y: usize, 
                       direction: Direction, length: usize) {
        let template = &self.ascii_art.road_templates[&direction];
        
        match direction {
            Direction::East | Direction::West => {
                // Horizontal road
                for dx in 0..length {
                    for (dy, line) in template.iter().enumerate() {
                        if let Some(ch) = line.chars().next() {
                            if x + dx < buffer.width && y + dy < buffer.height {
                                let color = self.get_road_color();
                                buffer.set_char(x + dx, y + dy, ScreenCell::with_colors(ch, color, Color::Black));
                            }
                        }
                    }
                }
            }
            Direction::North | Direction::South => {
                // Vertical road
                for dy in 0..length {
                    for (dx, line) in template.iter().enumerate() {
                        if let Some(ch) = line.chars().next() {
                            if x + dx < buffer.width && y + dy < buffer.height {
                                let color = self.get_road_color();
                                buffer.set_char(x + dx, y + dy, ScreenCell::with_colors(ch, color, Color::Black));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get road color based on time of day
    fn get_road_color(&self) -> Color {
        match self.time_of_day {
            TimeOfDay::Day => Color::DarkGrey,
            TimeOfDay::Night => Color::Black,
            TimeOfDay::Dawn | TimeOfDay::Dusk => Color::DarkGrey,
        }
    }

    /// Render weather effects
    pub fn render_weather(&self, buffer: &mut ScreenBuffer) {
        if self.weather == WeatherType::Clear {
            return;
        }

        let effects = &self.ascii_art.weather_effects[&self.weather];
        if effects.is_empty() {
            return;
        }

        let density = match self.weather {
            WeatherType::Rain => 15,
            WeatherType::Snow => 10,
            WeatherType::Fog => 25,
            WeatherType::Clear => 0,
        };

        for i in 0..density {
            let x = (i * 7) % buffer.width;
            let y = (i * 11 + self.weather_frame) % buffer.height;
            
            if let Some(sprite) = self.ascii_art.get_weather_sprite(self.weather, i) {
                if let Some(ch) = sprite.chars().next() {
                    let color = self.get_weather_color();
                    buffer.set_char(x, y, ScreenCell::with_colors(ch, color, Color::Black));
                }
            }
        }
    }

    /// Get weather effect color
    fn get_weather_color(&self) -> Color {
        match self.weather {
            WeatherType::Rain => Color::Blue,
            WeatherType::Snow => Color::White,
            WeatherType::Fog => Color::DarkGrey,
            WeatherType::Clear => Color::White,
        }
    }

    /// Update weather animation frame
    pub fn update_weather_frame(&mut self) {
        self.weather_frame = (self.weather_frame + 1) % 100;
    }

    /// Set weather type
    pub fn set_weather(&mut self, weather: WeatherType) {
        self.weather = weather;
        self.weather_frame = 0;
    }

    /// Set time of day
    pub fn set_time_of_day(&mut self, time: TimeOfDay) {
        self.time_of_day = time;
    }

    /// Render city background
    pub fn render_city_background(&self, buffer: &mut ScreenBuffer) {
        let bg_color = match self.time_of_day {
            TimeOfDay::Day => Color::Black,
            TimeOfDay::Night => Color::Black,
            TimeOfDay::Dawn => Color::Black,
            TimeOfDay::Dusk => Color::Black,
        };

        // Fill background
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                buffer.set_char(x, y, ScreenCell::with_colors(' ', Color::White, bg_color));
            }
        }

        // Add some city elements
        self.render_city_buildings(buffer);
    }

    /// Render decorative city buildings
    fn render_city_buildings(&self, buffer: &mut ScreenBuffer) {
        let buildings = [
            ("█████", 5, buffer.height - 8),
            ("███", 15, buffer.height - 6),
            ("███████", 25, buffer.height - 10),
            ("████", 40, buffer.height - 7),
        ];

        for (building, x, y) in buildings.iter() {
            for (dx, ch) in building.chars().enumerate() {
                for dy in 0..5 {
                    if x + dx < buffer.width && y + dy < buffer.height {
                        let color = match self.time_of_day {
                            TimeOfDay::Night => Color::DarkGrey,
                            _ => Color::Grey,
                        };
                        buffer.set_char(x + dx, y + dy, ScreenCell::with_colors(ch, color, Color::Black));
                    }
                }
            }
        }
    }

    /// Render vehicle sprite
    pub fn render_vehicle(&self, buffer: &mut ScreenBuffer, x: usize, y: usize,
                         vehicle_type: VehicleType, direction: Direction) {
        let sprite = self.ascii_art.get_vehicle_sprite(vehicle_type, direction);
        let color = match vehicle_type {
            VehicleType::Car => Color::Cyan,
            VehicleType::Truck => Color::Magenta,
            VehicleType::Emergency => Color::Red,
        };

        if let Some(ch) = sprite.chars().next() {
            if x < buffer.width && y < buffer.height {
                buffer.set_char(x, y, ScreenCell::with_colors(ch, color, Color::Black));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_art_creation() {
        let art = AsciiArt::new();
        assert!(!art.intersection_template.is_empty());
        assert!(!art.road_templates.is_empty());
        assert!(!art.vehicle_sprites.is_empty());
    }

    #[test]
    fn test_vehicle_sprites() {
        let art = AsciiArt::new();
        let sprite = art.get_vehicle_sprite(VehicleType::Car, Direction::North);
        assert_eq!(sprite, "▲");
    }

    #[test]
    fn test_weather_effects() {
        let art = AsciiArt::new();
        let rain_sprite = art.get_weather_sprite(WeatherType::Rain, 0);
        assert!(rain_sprite.is_some());
    }

    #[test]
    fn test_city_renderer() {
        let renderer = CityRenderer::new();
        assert_eq!(renderer.weather, WeatherType::Clear);
        assert_eq!(renderer.time_of_day, TimeOfDay::Day);
    }

    #[test]
    fn test_ui_elements() {
        let art = AsciiArt::new();
        let status_panel = art.get_ui_template(UiElement::StatusPanel);
        assert!(status_panel.is_some());
        assert!(!status_panel.unwrap().is_empty());
    }
}
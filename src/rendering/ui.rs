//! User interface components and layout management
//! 
//! This module handles the main UI layout, status displays, statistics,
//! and user interaction elements for the traffic simulation.

use crossterm::style::Color;
use std::time::{Duration, Instant};

use crate::rendering::{ScreenBuffer, ScreenCell, ColorScheme, SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::traffic::VehicleType;
use crate::simulation::statistics::SimulationStats;

/// Main UI layout manager
#[derive(Debug)]
pub struct UserInterface {
    pub layout: Layout,
    pub status_panel: StatusPanel,
    pub statistics_panel: StatisticsPanel,
    pub controls_panel: ControlsPanel,
    pub alert_system: AlertSystem,
    pub color_scheme: ColorScheme,
    pub last_update: Instant,
}

/// Screen layout configuration
#[derive(Debug, Clone)]
pub struct Layout {
    pub simulation_area: Rectangle,
    pub status_area: Rectangle,
    pub statistics_area: Rectangle,
    pub controls_area: Rectangle,
    pub alert_area: Rectangle,
}

/// Rectangle for UI positioning
#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rectangle {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains_point(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }

    pub fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}

impl Layout {
    /// Create default layout for the given screen size
    pub fn new(screen_width: usize, screen_height: usize) -> Self {
        let main_width = screen_width.saturating_sub(30);
        let panel_width = 28;
        
        Self {
            simulation_area: Rectangle::new(1, 1, main_width, screen_height.saturating_sub(2)),
            status_area: Rectangle::new(main_width + 2, 1, panel_width, 12),
            statistics_area: Rectangle::new(main_width + 2, 14, panel_width, 10),
            controls_area: Rectangle::new(main_width + 2, 25, panel_width, 10),
            alert_area: Rectangle::new(main_width / 4, 2, main_width / 2, 5),
        }
    }

    /// Adjust layout for screen resize
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        *self = Layout::new(new_width, new_height);
    }
}

/// Status panel showing current system state
#[derive(Debug)]
pub struct StatusPanel {
    pub emergency_active: bool,
    pub weather_type: String,
    pub time_of_day: String,
    pub system_status: SystemStatus,
    pub intersection_count: usize,
    pub total_vehicles: usize,
}

/// System status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemStatus {
    Operational,
    Warning,
    Emergency,
    Error,
}

impl SystemStatus {
    pub fn color(&self) -> Color {
        match self {
            SystemStatus::Operational => Color::Green,
            SystemStatus::Warning => Color::Yellow,
            SystemStatus::Emergency => Color::Red,
            SystemStatus::Error => Color::DarkRed,
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            SystemStatus::Operational => "OPERATIONAL",
            SystemStatus::Warning => "WARNING",
            SystemStatus::Emergency => "EMERGENCY",
            SystemStatus::Error => "ERROR",
        }
    }
}

impl StatusPanel {
    pub fn new() -> Self {
        Self {
            emergency_active: false,
            weather_type: "Clear".to_string(),
            time_of_day: "Day".to_string(),
            system_status: SystemStatus::Operational,
            intersection_count: 0,
            total_vehicles: 0,
        }
    }

    pub fn render(&self, buffer: &mut ScreenBuffer, area: Rectangle) {
        // Draw border
        self.draw_border(buffer, area, "STATUS", Color::Cyan);

        let content_area = Rectangle::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let mut y = content_area.y;

        // System status
        buffer.draw_text(content_area.x, y, "System:", Color::White);
        buffer.draw_text(content_area.x + 8, y, self.system_status.text(), self.system_status.color());
        y += 1;

        // Emergency status
        let emergency_text = if self.emergency_active { "ACTIVE" } else { "STANDBY" };
        let emergency_color = if self.emergency_active { Color::Red } else { Color::Green };
        buffer.draw_text(content_area.x, y, "Emergency:", Color::White);
        buffer.draw_text(content_area.x + 10, y, emergency_text, emergency_color);
        y += 1;

        // Weather
        buffer.draw_text(content_area.x, y, "Weather:", Color::White);
        buffer.draw_text(content_area.x + 9, y, &self.weather_type, Color::Cyan);
        y += 1;

        // Time of day
        buffer.draw_text(content_area.x, y, "Time:", Color::White);
        buffer.draw_text(content_area.x + 6, y, &self.time_of_day, Color::Yellow);
        y += 2;

        // Counts
        buffer.draw_text(content_area.x, y, &format!("Intersections: {}", self.intersection_count), Color::White);
        y += 1;
        buffer.draw_text(content_area.x, y, &format!("Vehicles: {}", self.total_vehicles), Color::White);
    }

    fn draw_border(&self, buffer: &mut ScreenBuffer, area: Rectangle, title: &str, color: Color) {
        // Top border with title
        buffer.set_char(area.x, area.y, ScreenCell::with_colors('╔', color, Color::Black));
        
        let title_start = area.x + 2;
        buffer.draw_text(title_start, area.y, title, color);
        
        for x in area.x + 1..area.x + area.width - 1 {
            let ch = if x >= title_start && x < title_start + title.len() {
                ' '
            } else {
                '═'
            };
            buffer.set_char(x, area.y, ScreenCell::with_colors(ch, color, Color::Black));
        }
        buffer.set_char(area.x + area.width - 1, area.y, ScreenCell::with_colors('╗', color, Color::Black));

        // Side borders
        for y in area.y + 1..area.y + area.height - 1 {
            buffer.set_char(area.x, y, ScreenCell::with_colors('║', color, Color::Black));
            buffer.set_char(area.x + area.width - 1, y, ScreenCell::with_colors('║', color, Color::Black));
        }

        // Bottom border
        buffer.set_char(area.x, area.y + area.height - 1, ScreenCell::with_colors('╚', color, Color::Black));
        for x in area.x + 1..area.x + area.width - 1 {
            buffer.set_char(x, area.y + area.height - 1, ScreenCell::with_colors('═', color, Color::Black));
        }
        buffer.set_char(area.x + area.width - 1, area.y + area.height - 1, ScreenCell::with_colors('╝', color, Color::Black));
    }
}

/// Statistics panel showing performance metrics
#[derive(Debug)]
pub struct StatisticsPanel {
    pub current_fps: f64,
    pub total_processed: u64,
    pub average_wait_time: f32,
    pub efficiency_score: f32,
    pub vehicle_breakdown: VehicleBreakdown,
}

#[derive(Debug, Clone)]
pub struct VehicleBreakdown {
    pub cars: u32,
    pub trucks: u32,
    pub emergency: u32,
}

impl StatisticsPanel {
    pub fn new() -> Self {
        Self {
            current_fps: 30.0,
            total_processed: 0,
            average_wait_time: 0.0,
            efficiency_score: 100.0,
            vehicle_breakdown: VehicleBreakdown {
                cars: 0,
                trucks: 0,
                emergency: 0,
            },
        }
    }

    pub fn update_from_stats(&mut self, stats: &SimulationStats) {
        self.total_processed = stats.total_vehicles_processed;
        self.average_wait_time = stats.average_wait_time;
        self.efficiency_score = stats.overall_efficiency;
        self.vehicle_breakdown = VehicleBreakdown {
            cars: stats.vehicle_counts.get(&VehicleType::Car).copied().unwrap_or(0),
            trucks: stats.vehicle_counts.get(&VehicleType::Truck).copied().unwrap_or(0),
            emergency: stats.vehicle_counts.get(&VehicleType::Emergency).copied().unwrap_or(0),
        };
    }

    pub fn render(&self, buffer: &mut ScreenBuffer, area: Rectangle) {
        self.draw_border(buffer, area, "STATISTICS", Color::Green);

        let content_area = Rectangle::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let mut y = content_area.y;

        // FPS
        buffer.draw_text(content_area.x, y, &format!("FPS: {:.1}", self.current_fps), Color::White);
        y += 1;

        // Total processed
        buffer.draw_text(content_area.x, y, &format!("Processed: {}", self.total_processed), Color::White);
        y += 1;

        // Average wait time
        buffer.draw_text(content_area.x, y, &format!("Avg Wait: {:.1}s", self.average_wait_time), Color::White);
        y += 1;

        // Efficiency score
        let efficiency_color = if self.efficiency_score >= 80.0 {
            Color::Green
        } else if self.efficiency_score >= 60.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        buffer.draw_text(content_area.x, y, "Efficiency:", Color::White);
        buffer.draw_text(content_area.x + 12, y, &format!("{:.0}%", self.efficiency_score), efficiency_color);
        y += 2;

        // Vehicle breakdown
        buffer.draw_text(content_area.x, y, "VEHICLE TYPES:", Color::Cyan);
        y += 1;
        buffer.draw_text(content_area.x, y, &format!("Cars: {}", self.vehicle_breakdown.cars), Color::White);
        y += 1;
        buffer.draw_text(content_area.x, y, &format!("Trucks: {}", self.vehicle_breakdown.trucks), Color::White);
        y += 1;
        buffer.draw_text(content_area.x, y, &format!("Emergency: {}", self.vehicle_breakdown.emergency), Color::Red);
    }

    fn draw_border(&self, buffer: &mut ScreenBuffer, area: Rectangle, title: &str, color: Color) {
        // Reuse border drawing logic from StatusPanel
        let status_panel = StatusPanel::new();
        status_panel.draw_border(buffer, area, title, color);
    }
}

/// Controls panel showing available commands
#[derive(Debug)]
pub struct ControlsPanel {
    pub visible: bool,
    pub help_mode: bool,
}

impl ControlsPanel {
    pub fn new() -> Self {
        Self {
            visible: true,
            help_mode: false,
        }
    }

    pub fn render(&self, buffer: &mut ScreenBuffer, area: Rectangle) {
        if !self.visible {
            return;
        }

        self.draw_border(buffer, area, "CONTROLS", Color::Yellow);

        let content_area = Rectangle::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
        let mut y = content_area.y;

        let controls = [
            ("E", "Emergency", Color::Red),
            ("R", "Rain", Color::Blue),
            ("S", "Snow", Color::White),
            ("F", "Fog", Color::DarkGrey),
            ("+", "More Traffic", Color::Green),
            ("-", "Less Traffic", Color::Yellow),
            ("Q", "Quit", Color::Red),
        ];

        for (key, description, color) in controls.iter() {
            if y < content_area.y + content_area.height - 1 {
                buffer.draw_text(content_area.x, y, key, *color);
                buffer.draw_text(content_area.x + 2, y, "-", Color::White);
                buffer.draw_text(content_area.x + 4, y, description, Color::White);
                y += 1;
            }
        }
    }

    fn draw_border(&self, buffer: &mut ScreenBuffer, area: Rectangle, title: &str, color: Color) {
        let status_panel = StatusPanel::new();
        status_panel.draw_border(buffer, area, title, color);
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
}

/// Alert system for notifications and warnings
#[derive(Debug)]
pub struct AlertSystem {
    pub current_alert: Option<Alert>,
    pub alert_queue: Vec<Alert>,
    pub blink_state: bool,
    pub last_blink: Instant,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub message: String,
    pub level: AlertLevel,
    pub duration: Duration,
    pub created_at: Instant,
    pub persistent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Emergency,
    Critical,
}

impl AlertLevel {
    pub fn color(&self) -> Color {
        match self {
            AlertLevel::Info => Color::Cyan,
            AlertLevel::Warning => Color::Yellow,
            AlertLevel::Emergency => Color::Red,
            AlertLevel::Critical => Color::DarkRed,
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            AlertLevel::Info => 1,
            AlertLevel::Warning => 2,
            AlertLevel::Emergency => 3,
            AlertLevel::Critical => 4,
        }
    }
}

impl AlertSystem {
    pub fn new() -> Self {
        Self {
            current_alert: None,
            alert_queue: Vec::new(),
            blink_state: false,
            last_blink: Instant::now(),
        }
    }

    pub fn add_alert(&mut self, message: String, level: AlertLevel, duration: Duration) {
        let alert = Alert {
            message,
            level,
            duration,
            created_at: Instant::now(),
            persistent: level == AlertLevel::Critical || level == AlertLevel::Emergency,
        };

        // Insert based on priority
        let mut inserted = false;
        for (i, existing) in self.alert_queue.iter().enumerate() {
            if alert.level.priority() > existing.level.priority() {
                self.alert_queue.insert(i, alert.clone());
                inserted = true;
                break;
            }
        }

        if !inserted {
            self.alert_queue.push(alert);
        }
    }

    pub fn update(&mut self) {
        // Update blink state
        if self.last_blink.elapsed() >= Duration::from_millis(500) {
            self.blink_state = !self.blink_state;
            self.last_blink = Instant::now();
        }

        // Check current alert expiration
        if let Some(ref alert) = self.current_alert {
            if !alert.persistent && alert.created_at.elapsed() >= alert.duration {
                self.current_alert = None;
            }
        }

        // Get next alert from queue
        if self.current_alert.is_none() && !self.alert_queue.is_empty() {
            self.current_alert = Some(self.alert_queue.remove(0));
        }
    }

    pub fn render(&self, buffer: &mut ScreenBuffer, area: Rectangle) {
        if let Some(ref alert) = self.current_alert {
            // Blink for high priority alerts
            let should_show = if alert.level == AlertLevel::Emergency || alert.level == AlertLevel::Critical {
                self.blink_state
            } else {
                true
            };

            if should_show {
                self.render_alert(buffer, area, alert);
            }
        }
    }

    fn render_alert(&self, buffer: &mut ScreenBuffer, area: Rectangle, alert: &Alert) {
        let bg_color = if alert.level == AlertLevel::Critical {
            Color::DarkRed
        } else {
            Color::Black
        };

        // Fill alert area with background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buffer.set_char(x, y, ScreenCell::with_colors(' ', Color::White, bg_color));
            }
        }

        // Draw border
        let border_color = alert.level.color();
        for x in area.x..area.x + area.width {
            buffer.set_char(x, area.y, ScreenCell::with_colors('═', border_color, bg_color));
            buffer.set_char(x, area.y + area.height - 1, ScreenCell::with_colors('═', border_color, bg_color));
        }
        for y in area.y..area.y + area.height {
            buffer.set_char(area.x, y, ScreenCell::with_colors('║', border_color, bg_color));
            buffer.set_char(area.x + area.width - 1, y, ScreenCell::with_colors('║', border_color, bg_color));
        }

        // Corner characters
        buffer.set_char(area.x, area.y, ScreenCell::with_colors('╔', border_color, bg_color));
        buffer.set_char(area.x + area.width - 1, area.y, ScreenCell::with_colors('╗', border_color, bg_color));
        buffer.set_char(area.x, area.y + area.height - 1, ScreenCell::with_colors('╚', border_color, bg_color));
        buffer.set_char(area.x + area.width - 1, area.y + area.height - 1, ScreenCell::with_colors('╝', border_color, bg_color));

        // Center the message
        let message_y = area.y + area.height / 2;
        let message_x = area.x + (area.width.saturating_sub(alert.message.len())) / 2;
        
        buffer.draw_text_with_bg(message_x, message_y, &alert.message, alert.level.color(), bg_color);
    }

    pub fn clear_current(&mut self) {
        self.current_alert = None;
    }

    pub fn clear_all(&mut self) {
        self.current_alert = None;
        self.alert_queue.clear();
    }
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            layout: Layout::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            status_panel: StatusPanel::new(),
            statistics_panel: StatisticsPanel::new(),
            controls_panel: ControlsPanel::new(),
            alert_system: AlertSystem::new(),
            color_scheme: ColorScheme::default(),
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self, stats: &SimulationStats) {
        self.statistics_panel.update_from_stats(stats);
        self.alert_system.update();
        self.last_update = Instant::now();
    }

    pub fn render(&self, buffer: &mut ScreenBuffer) {
        self.status_panel.render(buffer, self.layout.status_area);
        self.statistics_panel.render(buffer, self.layout.statistics_area);
        self.controls_panel.render(buffer, self.layout.controls_area);
        self.alert_system.render(buffer, self.layout.alert_area);
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.layout.resize(width, height);
    }

    pub fn add_alert(&mut self, message: String, level: AlertLevel, duration: Duration) {
        self.alert_system.add_alert(message, level, duration);
    }

    pub fn set_emergency_active(&mut self, active: bool) {
        self.status_panel.emergency_active = active;
        if active {
            self.add_alert(
                "⚠ EMERGENCY VEHICLE DETECTED ⚠".to_string(),
                AlertLevel::Emergency,
                Duration::from_secs(5),
            );
        }
    }

    pub fn set_weather(&mut self, weather: &str) {
        self.status_panel.weather_type = weather.to_string();
    }

    pub fn set_system_status(&mut self, status: SystemStatus) {
        self.status_panel.system_status = status;
    }

    pub fn get_simulation_area(&self) -> Rectangle {
        self.layout.simulation_area
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_creation() {
        let layout = Layout::new(120, 40);
        assert!(layout.simulation_area.width > 0);
        assert!(layout.simulation_area.height > 0);
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::new(10, 10, 20, 15);
        assert!(rect.contains_point(15, 15));
        assert!(!rect.contains_point(5, 5));
        assert_eq!(rect.center(), (20, 17));
    }

    #[test]
    fn test_alert_system() {
        let mut alert_system = AlertSystem::new();
        alert_system.add_alert(
            "Test alert".to_string(),
            AlertLevel::Warning,
            Duration::from_secs(3),
        );
        assert_eq!(alert_system.alert_queue.len(), 1);
    }

    #[test]
    fn test_system_status() {
        assert_eq!(SystemStatus::Operational.text(), "OPERATIONAL");
        assert_eq!(SystemStatus::Emergency.color(), Color::Red);
    }

    #[test]
    fn test_alert_priority() {
        assert!(AlertLevel::Critical.priority() > AlertLevel::Warning.priority());
        assert!(AlertLevel::Emergency.priority() > AlertLevel::Info.priority());
    }
}
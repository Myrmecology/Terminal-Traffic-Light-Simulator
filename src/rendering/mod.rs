//! Rendering module for terminal-based graphics and UI
//! 
//! This module handles all visual output including terminal control,
//! ASCII art, and user interface elements.

pub mod terminal;
pub mod ascii_art;
pub mod ui;

pub use terminal::*;
pub use ascii_art::*;
pub use ui::*;

use std::collections::HashMap;
use crossterm::style::Color;

/// Screen dimensions and layout constants
pub const SCREEN_WIDTH: usize = 120;
pub const SCREEN_HEIGHT: usize = 40;
pub const INTERSECTION_SIZE: usize = 5;
pub const ROAD_WIDTH: usize = 3;

/// Color scheme for different elements
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub background: Color,
    pub road: Color,
    pub intersection: Color,
    pub text: Color,
    pub highlight: Color,
    pub emergency: Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: Color::Black,
            road: Color::DarkGrey,
            intersection: Color::Grey,
            text: Color::White,
            highlight: Color::Yellow,
            emergency: Color::Red,
        }
    }
}

/// Screen buffer for efficient rendering
#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<ScreenCell>>,
    pub dirty_regions: Vec<Region>,
}

/// Individual screen cell with character and color
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenCell {
    pub character: char,
    pub foreground: Color,
    pub background: Color,
    pub bold: bool,
}

impl Default for ScreenCell {
    fn default() -> Self {
        Self {
            character: ' ',
            foreground: Color::White,
            background: Color::Black,
            bold: false,
        }
    }
}

impl ScreenCell {
    pub fn new(character: char) -> Self {
        Self {
            character,
            ..Default::default()
        }
    }

    pub fn with_colors(character: char, foreground: Color, background: Color) -> Self {
        Self {
            character,
            foreground,
            background,
            bold: false,
        }
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
}

/// Screen region for dirty area tracking
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Region {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains_point(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }

    pub fn overlaps(&self, other: &Region) -> bool {
        !(self.x + self.width <= other.x ||
          other.x + other.width <= self.x ||
          self.y + self.height <= other.y ||
          other.y + other.height <= self.y)
    }
}

impl ScreenBuffer {
    /// Create a new screen buffer
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![vec![ScreenCell::default(); width]; height];
        Self {
            width,
            height,
            cells,
            dirty_regions: Vec::new(),
        }
    }

    /// Clear the screen buffer
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = ScreenCell::default();
            }
        }
        self.mark_dirty(Region::new(0, 0, self.width, self.height));
    }

    /// Set a character at position
    pub fn set_char(&mut self, x: usize, y: usize, cell: ScreenCell) {
        if x < self.width && y < self.height {
            if self.cells[y][x] != cell {
                self.cells[y][x] = cell;
                self.mark_dirty(Region::new(x, y, 1, 1));
            }
        }
    }

    /// Get a character at position
    pub fn get_char(&self, x: usize, y: usize) -> Option<&ScreenCell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y][x])
        } else {
            None
        }
    }

    /// Draw text at position
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, color: Color) {
        for (i, ch) in text.chars().enumerate() {
            if x + i < self.width && y < self.height {
                self.set_char(x + i, y, ScreenCell::with_colors(ch, color, Color::Black));
            }
        }
    }

    /// Draw text with background
    pub fn draw_text_with_bg(&mut self, x: usize, y: usize, text: &str, fg: Color, bg: Color) {
        for (i, ch) in text.chars().enumerate() {
            if x + i < self.width && y < self.height {
                self.set_char(x + i, y, ScreenCell::with_colors(ch, fg, bg));
            }
        }
    }

    /// Fill rectangle with character
    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, cell: ScreenCell) {
        for dy in 0..height {
            for dx in 0..width {
                if x + dx < self.width && y + dy < self.height {
                    self.set_char(x + dx, y + dy, cell.clone());
                }
            }
        }
    }

    /// Draw horizontal line
    pub fn draw_hline(&mut self, x: usize, y: usize, length: usize, ch: char, color: Color) {
        for i in 0..length {
            if x + i < self.width && y < self.height {
                self.set_char(x + i, y, ScreenCell::with_colors(ch, color, Color::Black));
            }
        }
    }

    /// Draw vertical line
    pub fn draw_vline(&mut self, x: usize, y: usize, length: usize, ch: char, color: Color) {
        for i in 0..length {
            if x < self.width && y + i < self.height {
                self.set_char(x, y + i, ScreenCell::with_colors(ch, color, Color::Black));
            }
        }
    }

    /// Mark region as dirty for optimized rendering
    pub fn mark_dirty(&mut self, region: Region) {
        // Merge overlapping regions for efficiency
        let mut merged = false;
        for existing in &mut self.dirty_regions {
            if existing.overlaps(&region) {
                let new_x = existing.x.min(region.x);
                let new_y = existing.y.min(region.y);
                let new_width = (existing.x + existing.width).max(region.x + region.width) - new_x;
                let new_height = (existing.y + existing.height).max(region.y + region.height) - new_y;
                
                *existing = Region::new(new_x, new_y, new_width, new_height);
                merged = true;
                break;
            }
        }
        
        if !merged {
            self.dirty_regions.push(region);
        }
    }

    /// Get dirty regions and clear them
    pub fn take_dirty_regions(&mut self) -> Vec<Region> {
        std::mem::take(&mut self.dirty_regions)
    }

    /// Check if buffer has dirty regions
    pub fn has_dirty_regions(&self) -> bool {
        !self.dirty_regions.is_empty()
    }
}

/// Frame rate management
#[derive(Debug)]
pub struct FrameRate {
    target_fps: u32,
    frame_duration: std::time::Duration,
    last_frame: std::time::Instant,
    frame_count: u64,
    fps_counter: f64,
    last_fps_update: std::time::Instant,
}

impl FrameRate {
    pub fn new(target_fps: u32) -> Self {
        Self {
            target_fps,
            frame_duration: std::time::Duration::from_secs_f64(1.0 / target_fps as f64),
            last_frame: std::time::Instant::now(),
            frame_count: 0,
            fps_counter: 0.0,
            last_fps_update: std::time::Instant::now(),
        }
    }

    /// Wait for next frame
    pub fn wait_for_next_frame(&mut self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.frame_duration {
            std::thread::sleep(self.frame_duration - elapsed);
        }
        self.last_frame = std::time::Instant::now();
        self.frame_count += 1;

        // Update FPS counter every second
        if self.last_fps_update.elapsed().as_secs() >= 1 {
            self.fps_counter = self.frame_count as f64 / self.last_fps_update.elapsed().as_secs_f64();
            self.frame_count = 0;
            self.last_fps_update = std::time::Instant::now();
        }
    }

    /// Get current FPS
    pub fn current_fps(&self) -> f64 {
        self.fps_counter
    }

    /// Set target FPS
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
        self.frame_duration = std::time::Duration::from_secs_f64(1.0 / fps as f64);
    }
}

/// Animation frame for smooth transitions
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub progress: f32, // 0.0 to 1.0
    pub easing: EasingFunction,
}

/// Easing functions for animations
#[derive(Debug, Clone, Copy)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl EasingFunction {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - 2.0 * (1.0 - t) * (1.0 - t)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_buffer_creation() {
        let buffer = ScreenBuffer::new(80, 24);
        assert_eq!(buffer.width, 80);
        assert_eq!(buffer.height, 24);
        assert_eq!(buffer.cells.len(), 24);
        assert_eq!(buffer.cells[0].len(), 80);
    }

    #[test]
    fn test_screen_cell() {
        let cell = ScreenCell::new('A');
        assert_eq!(cell.character, 'A');
        assert_eq!(cell.foreground, Color::White);
        
        let colored_cell = ScreenCell::with_colors('B', Color::Red, Color::Blue);
        assert_eq!(colored_cell.character, 'B');
        assert_eq!(colored_cell.foreground, Color::Red);
        assert_eq!(colored_cell.background, Color::Blue);
    }

    #[test]
    fn test_region_overlap() {
        let region1 = Region::new(0, 0, 10, 10);
        let region2 = Region::new(5, 5, 10, 10);
        let region3 = Region::new(20, 20, 5, 5);
        
        assert!(region1.overlaps(&region2));
        assert!(!region1.overlaps(&region3));
    }

    #[test]
    fn test_frame_rate() {
        let frame_rate = FrameRate::new(30);
        assert_eq!(frame_rate.target_fps, 30);
        assert!(frame_rate.frame_duration.as_millis() > 0);
    }

    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);
        assert!(EasingFunction::EaseIn.apply(0.5) < 0.5);
        assert!(EasingFunction::EaseOut.apply(0.5) > 0.5);
    }
}
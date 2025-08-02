//! Terminal control and management
//! 
//! This module provides cross-platform terminal control using crossterm,
//! including raw mode, cursor management, and efficient screen updates.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, Write, Stdout, Result as IoResult};
use std::time::Duration;

use crate::rendering::{ScreenBuffer, Region};

/// Terminal controller for managing screen output and input
pub struct Terminal {
    stdout: Stdout,
    original_hook: Option<Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static>>,
    raw_mode_enabled: bool,
    alternate_screen: bool,
    cursor_hidden: bool,
    width: u16,
    height: u16,
}

impl Terminal {
    /// Initialize terminal with raw mode and alternate screen
    pub fn new() -> IoResult<Self> {
        let stdout = io::stdout();
        let (width, height) = size()?;

        let mut terminal = Self {
            stdout,
            original_hook: None,
            raw_mode_enabled: false,
            alternate_screen: false,
            cursor_hidden: false,
            width,
            height,
        };

        terminal.setup()?;
        Ok(terminal)
    }

    /// Setup terminal for application use
    fn setup(&mut self) -> IoResult<()> {
        // Setup panic hook for clean restoration
        self.setup_panic_hook();

        // Enable raw mode
        enable_raw_mode()?;
        self.raw_mode_enabled = true;

        // Enter alternate screen
        execute!(self.stdout, EnterAlternateScreen)?;
        self.alternate_screen = true;

        // Hide cursor
        execute!(self.stdout, Hide)?;
        self.cursor_hidden = true;

        // Clear screen
        execute!(self.stdout, Clear(ClearType::All))?;

        self.stdout.flush()?;
        Ok(())
    }

    /// Setup panic hook to restore terminal on panic
    fn setup_panic_hook(&mut self) {
        let original_hook = std::panic::take_hook();
        self.original_hook = Some(original_hook);

        std::panic::set_hook(Box::new(|panic_info| {
            // Attempt to restore terminal
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            
            // Call original panic hook
            eprintln!("Application panicked: {}", panic_info);
            std::process::exit(1);
        }));
    }

    /// Restore terminal to original state
    pub fn restore(&mut self) -> IoResult<()> {
        if self.cursor_hidden {
            execute!(self.stdout, Show)?;
            self.cursor_hidden = false;
        }

        if self.alternate_screen {
            execute!(self.stdout, LeaveAlternateScreen)?;
            self.alternate_screen = false;
        }

        if self.raw_mode_enabled {
            disable_raw_mode()?;
            self.raw_mode_enabled = false;
        }

        // Restore original panic hook
        if let Some(hook) = self.original_hook.take() {
            std::panic::set_hook(hook);
        }

        self.stdout.flush()?;
        Ok(())
    }

    /// Get terminal size
    pub fn size(&mut self) -> IoResult<(u16, u16)> {
        let (width, height) = size()?;
        self.width = width;
        self.height = height;
        Ok((width, height))
    }

    /// Clear the entire screen
    pub fn clear(&mut self) -> IoResult<()> {
        execute!(self.stdout, Clear(ClearType::All))?;
        Ok(())
    }

    /// Move cursor to position
    pub fn move_cursor(&mut self, x: u16, y: u16) -> IoResult<()> {
        execute!(self.stdout, MoveTo(x, y))?;
        Ok(())
    }

    /// Hide cursor
    pub fn hide_cursor(&mut self) -> IoResult<()> {
        if !self.cursor_hidden {
            execute!(self.stdout, Hide)?;
            self.cursor_hidden = true;
        }
        Ok(())
    }

    /// Show cursor
    pub fn show_cursor(&mut self) -> IoResult<()> {
        if self.cursor_hidden {
            execute!(self.stdout, Show)?;
            self.cursor_hidden = false;
        }
        Ok(())
    }

    /// Render screen buffer efficiently using dirty regions
    pub fn render_buffer(&mut self, buffer: &mut ScreenBuffer) -> IoResult<()> {
        let dirty_regions = buffer.take_dirty_regions();
        
        if dirty_regions.is_empty() {
            return Ok(());
        }

        for region in dirty_regions {
            self.render_region(buffer, &region)?;
        }

        self.stdout.flush()?;
        Ok(())
    }

    /// Render a specific region of the screen buffer
    fn render_region(&mut self, buffer: &ScreenBuffer, region: &Region) -> IoResult<()> {
        for y in region.y..region.y + region.height {
            if y >= buffer.height {
                break;
            }

            queue!(self.stdout, MoveTo(region.x as u16, y as u16))?;

            let mut current_fg = None;
            let mut current_bg = None;

            for x in region.x..region.x + region.width {
                if x >= buffer.width {
                    break;
                }

                if let Some(cell) = buffer.get_char(x, y) {
                    // Optimize color changes
                    if current_fg != Some(cell.foreground) {
                        queue!(self.stdout, SetForegroundColor(cell.foreground))?;
                        current_fg = Some(cell.foreground);
                    }

                    if current_bg != Some(cell.background) {
                        queue!(self.stdout, SetBackgroundColor(cell.background))?;
                        current_bg = Some(cell.background);
                    }

                    queue!(self.stdout, Print(cell.character))?;
                }
            }
        }

        queue!(self.stdout, ResetColor)?;
        Ok(())
    }

    /// Render entire screen buffer (fallback for full refresh)
    pub fn render_full_buffer(&mut self, buffer: &ScreenBuffer) -> IoResult<()> {
        execute!(self.stdout, Clear(ClearType::All))?;

        for y in 0..buffer.height {
            queue!(self.stdout, MoveTo(0, y as u16))?;
            
            let mut current_fg = None;
            let mut current_bg = None;

            for x in 0..buffer.width {
                if let Some(cell) = buffer.get_char(x, y) {
                    // Optimize color changes
                    if current_fg != Some(cell.foreground) {
                        queue!(self.stdout, SetForegroundColor(cell.foreground))?;
                        current_fg = Some(cell.foreground);
                    }

                    if current_bg != Some(cell.background) {
                        queue!(self.stdout, SetBackgroundColor(cell.background))?;
                        current_bg = Some(cell.background);
                    }

                    queue!(self.stdout, Print(cell.character))?;
                }
            }
        }

        queue!(self.stdout, ResetColor)?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Draw text at specific position with color
    pub fn draw_text(&mut self, x: u16, y: u16, text: &str, fg: Color, bg: Color) -> IoResult<()> {
        queue!(
            self.stdout,
            MoveTo(x, y),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(text),
            ResetColor
        )?;
        Ok(())
    }

    /// Draw text with only foreground color
    pub fn draw_text_fg(&mut self, x: u16, y: u16, text: &str, color: Color) -> IoResult<()> {
        queue!(
            self.stdout,
            MoveTo(x, y),
            SetForegroundColor(color),
            Print(text),
            ResetColor
        )?;
        Ok(())
    }

    /// Flush output buffer
    pub fn flush(&mut self) -> IoResult<()> {
        self.stdout.flush()?;
        Ok(())
    }

    /// Check if resize event occurred
    pub fn check_resize(&mut self) -> IoResult<Option<(u16, u16)>> {
        let (width, height) = size()?;
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            Ok(Some((width, height)))
        } else {
            Ok(None)
        }
    }
}

// Add Debug implementation for Terminal
impl std::fmt::Debug for Terminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Terminal")
            .field("raw_mode_enabled", &self.raw_mode_enabled)
            .field("alternate_screen", &self.alternate_screen)
            .field("cursor_hidden", &self.cursor_hidden)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Input handler for non-blocking keyboard input
#[derive(Debug)]
pub struct InputHandler {
    pub last_input: Option<KeyEvent>,
}

impl InputHandler {
    /// Create new input handler
    pub fn new() -> Self {
        Self { last_input: None }
    }

    /// Poll for input events (non-blocking)
    pub fn poll_input(&mut self, timeout: Duration) -> IoResult<Option<InputEvent>> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => {
                    self.last_input = Some(key_event);
                    Ok(Some(self.map_key_event(key_event)))
                }
                Event::Resize(width, height) => {
                    Ok(Some(InputEvent::Resize(width, height)))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Map crossterm key event to our input event
    fn map_key_event(&self, key_event: KeyEvent) -> InputEvent {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => InputEvent::Quit,
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } => InputEvent::Char(c),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => InputEvent::Enter,
            KeyEvent {
                code: KeyCode::Esc,
                ..
            } => InputEvent::Escape,
            KeyEvent {
                code: KeyCode::Up,
                ..
            } => InputEvent::ArrowUp,
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => InputEvent::ArrowDown,
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => InputEvent::ArrowLeft,
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => InputEvent::ArrowRight,
            KeyEvent {
                code: KeyCode::F(n),
                ..
            } => InputEvent::Function(n),
            _ => InputEvent::Unknown,
        }
    }
}

/// Input events for the application
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Char(char),
    Enter,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Function(u8),
    Resize(u16, u16),
    Quit,
    Unknown,
}

/// Terminal capabilities detection
#[derive(Debug)]
pub struct TerminalCapabilities {
    pub supports_color: bool,
    pub supports_unicode: bool,
    pub supports_mouse: bool,
    pub width: u16,
    pub height: u16,
}

impl TerminalCapabilities {
    /// Detect terminal capabilities
    pub fn detect() -> IoResult<Self> {
        let (width, height) = size()?;
        
        Ok(Self {
            supports_color: true, // crossterm handles color support detection
            supports_unicode: true, // assume modern terminal
            supports_mouse: false, // not implemented yet
            width,
            height,
        })
    }

    /// Check if terminal is large enough for the application
    pub fn is_adequate_size(&self) -> bool {
        self.width >= 80 && self.height >= 24
    }

    /// Get recommended buffer size
    pub fn recommended_buffer_size(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_event_mapping() {
        let handler = InputHandler::new();
        
        let char_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let mapped = handler.map_key_event(char_event);
        assert_eq!(mapped, InputEvent::Char('a'));
        
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let mapped = handler.map_key_event(ctrl_c);
        assert_eq!(mapped, InputEvent::Quit);
    }

    #[test]
    fn test_terminal_capabilities() {
        // This test might fail in CI environments without a terminal
        if let Ok(caps) = TerminalCapabilities::detect() {
            assert!(caps.width > 0);
            assert!(caps.height > 0);
            assert!(caps.supports_color);
        }
    }

    #[test]
    fn test_input_handler_creation() {
        let handler = InputHandler::new();
        assert!(handler.last_input.is_none());
    }
}
#[derive(Debug)]
pub struct Car {
    pub id: u32,
    pub position: usize,
    pub stopped: bool,
}

impl Car {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            position: 0,
            stopped: false,
        }
    }

    pub fn update(&mut self, can_move: bool) {
        if can_move {
            self.stopped = false;
            self.position += 1;
        } else {
            self.stopped = true;
        }
    }

    pub fn render(&self) -> String {
        if self.stopped {
            format!("🚗 (stopped) at {}", self.position)
        } else {
            format!("🚗 moving to {}", self.position)
        }
    }

    pub fn lane_symbol(&self) -> &str {
        if self.stopped {
            "🅿️"
        } else {
            "🚗"
        }
    }

    /// Optional logic for smart stopping
    pub fn should_stop_for_yellow(&self, lane_length: usize) -> bool {
        self.position >= lane_length - 2
    }
}


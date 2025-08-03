#[derive(Debug, Clone, Copy)]
pub enum TrafficLightState {
    Red,
    Yellow,
    Green,
}

impl TrafficLightState {
    pub fn next(self) -> Self {
        match self {
            TrafficLightState::Red => TrafficLightState::Green,
            TrafficLightState::Green => TrafficLightState::Yellow,
            TrafficLightState::Yellow => TrafficLightState::Red,
        }
    }
}


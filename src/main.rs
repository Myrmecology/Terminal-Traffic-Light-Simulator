mod systems;

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};
use systems::car::Car;
use systems::traffic_light::TrafficLightState;

use crossterm::{
    event::{poll, read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

fn main() {
    let mut current = TrafficLightState::Red;
    let mut cars: Vec<Car> = Vec::new();
    let mut tick: u32 = 0;
    let mut next_car_id = 1;
    let lane_length: usize = 20;

    enable_raw_mode().expect("Failed to enable raw mode");

    loop {
        let light_duration = match current {
            TrafficLightState::Red => 5,
            TrafficLightState::Green => 5,
            TrafficLightState::Yellow => 2,
        };

        for remaining in (1..=light_duration).rev() {
            if poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = read().unwrap() {
                    if key_event.code == KeyCode::Char('q') {
                        println!("\n\n👋 Quitting simulation...\n");
                        disable_raw_mode().unwrap();
                        return;
                    }
                }
            }

            print!("\x1B[2J\x1B[1;1H");
            stdout().flush().unwrap();

            println!();
            println!("===============================");
            println!("     TRAFFIC LIGHT SIMULATOR");
            println!("===============================");
            println!();

            println!("Tick: {}", tick);
            println!();

            let light_symbol = match current {
                TrafficLightState::Red => "🟥",
                TrafficLightState::Green => "🟩",
                TrafficLightState::Yellow => "🟨",
            };
            println!("Current light: {} ({}s left)", light_symbol, remaining);
            println!();

            println!("Vehicle count: {}", next_car_id - 1);
            println!();
            println!();

            if tick % 3 == 0 && remaining == light_duration {
                cars.push(Car::new(next_car_id));
                next_car_id += 1;
            }

            for car in cars.iter_mut() {
                let can_move = match current {
                    TrafficLightState::Green => true,
                    TrafficLightState::Red => false,
                    TrafficLightState::Yellow => {
                        !car.should_stop_for_yellow(lane_length)
                    }
                };
                car.update(can_move);
            }

            let mut lane_top = vec!["  "; lane_length];
            for car in &cars {
                if car.position < lane_length {
                    lane_top[car.position] = car.lane_symbol();
                }
            }

            let lane_bottom = vec!["🛣️"; lane_length];

            print!("{}   ", light_symbol);
            for symbol in &lane_top {
                print!("{}", symbol);
            }
            println!();

            print!("    ");
            for symbol in &lane_bottom {
                print!("{}", symbol);
            }
            println!();

            println!();
            println!();

            sleep(Duration::from_secs(1));
        }

        current = current.next();
        tick += 1;
    }
}












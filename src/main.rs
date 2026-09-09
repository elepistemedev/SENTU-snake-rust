use std::thread;
use std::time::Duration;

use macroquad::prelude::*;

use snake::sim::Simulation;
use snake::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "snake-ai".to_owned(),
        high_dpi: true,
        sample_count: 1,
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut sim = Simulation::new();
    let mut is_viz_enabled = true;
    let mut is_slow_mode = true;

    loop {
        let mut iterations = 0;

        // Force slow mode in VS
        let effective_slow_mode = is_slow_mode || sim.is_vs_mode();

        loop {
            sim.update(is_viz_enabled, effective_slow_mode);
            iterations += 1;

            if effective_slow_mode {
                break;
            }
            if iterations >= 50 {
                break;
            }
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            is_viz_enabled = !is_viz_enabled;
            if !is_viz_enabled {
                is_slow_mode = false;
            }
        }
        if is_key_released(KeyCode::Space) {
            is_slow_mode = !is_slow_mode;
        }
        if is_key_pressed(KeyCode::V) {
            if sim.toggle_vs_mode() {
                is_slow_mode = true; // Enable slow mode when entering VS
            }
        }

        if effective_slow_mode {
            thread::sleep(Duration::from_millis(SIM_SLEEP_MILLIS));
        }
        next_frame().await
    }
}

//! Unified snake shell — single binary entry.
//!
//! Replaces both former binaries (`snake` = GA driver, `snake-dqn` = DQN driver)
//! with one macroquad loop that owns the whole app lifecycle: a welcome menu plus
//! the five playable views (DQN train, DQN versus, GA train, GA versus, DQN vs
//! GA). All lifecycle and state-machine logic lives in [`snake::app::App`]; this
//! file only opens the window (fullscreen, as the GA binary was — AD-8) and pumps
//! frames until the user quits from the menu (`Esc`).

use snake::app::App;

fn window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "snake-ai".to_owned(),
        high_dpi: true,
        sample_count: 1,
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
    loop {
        if app.run_frame() {
            break;
        }
        macroquad::prelude::next_frame().await;
    }
}

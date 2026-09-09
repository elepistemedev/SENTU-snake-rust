use macroquad::prelude::*;
use snake::game_dqn::GameDQN;
use snake::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Snake DQN Training".to_owned(),
        high_dpi: true,
        sample_count: 1,
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = GameDQN::new();
    let mut episode = 0;
    let mut total_reward = 0.0;
    let mut best_score = 0;

    loop {
        clear_background(BLACK);

        let (reward, done) = game.step();
        total_reward += reward;

        if done {
            episode += 1;
            if game.score > best_score {
                best_score = game.score;
            }

            println!(
                "Episode: {} | Score: {} | Best: {} | Reward: {:.2} | Epsilon: {:.3}",
                episode,
                game.score,
                best_score,
                total_reward,
                game.agent.get_epsilon()
            );

            total_reward = 0.0;
            game.reset();
        }

        // Visualization
        draw_text(
            &format!("Episode: {}", episode),
            10.0,
            30.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Score: {}", game.score),
            10.0,
            60.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Best: {}", best_score),
            10.0,
            90.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Epsilon: {:.3}", game.agent.get_epsilon()),
            10.0,
            120.0,
            30.0,
            WHITE,
        );

        // Draw grid
        let tile_size = 20.0;
        let offset_x = 250.0;
        let offset_y = 50.0;

        // Draw food
        draw_rectangle(
            offset_x + game.food.x as f32 * tile_size,
            offset_y + game.food.y as f32 * tile_size,
            tile_size,
            tile_size,
            RED,
        );

        // Draw snake
        for (i, segment) in game.body.iter().enumerate() {
            let color = if i == 0 { GREEN } else { DARKGREEN };
            draw_rectangle(
                offset_x + segment.x as f32 * tile_size,
                offset_y + segment.y as f32 * tile_size,
                tile_size,
                tile_size,
                color,
            );
        }

        // Draw grid lines
        for i in 0..=GRID_W {
            draw_line(
                offset_x + i as f32 * tile_size,
                offset_y,
                offset_x + i as f32 * tile_size,
                offset_y + GRID_H as f32 * tile_size,
                1.0,
                GRAY,
            );
        }
        for i in 0..=GRID_H {
            draw_line(
                offset_x,
                offset_y + i as f32 * tile_size,
                offset_x + GRID_W as f32 * tile_size,
                offset_y + i as f32 * tile_size,
                1.0,
                GRAY,
            );
        }

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}

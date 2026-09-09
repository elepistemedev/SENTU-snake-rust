//! VS Mode visualization - Two snakes battle head-to-head

use macroquad::prelude::*;
use crate::*;

const PANEL_BG: Color = Color::new(0.05, 0.05, 0.05, 0.95);
const PANEL_BORDER: Color = Color::new(0.4, 0.4, 0.4, 1.0);
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const SNAKE1_COLOR: Color = Color::new(0.3, 0.9, 0.3, 1.0);
const SNAKE2_COLOR: Color = Color::new(0.9, 0.3, 0.3, 1.0);
const TITLE_SIZE: f32 = 28.0;
const TEXT_SIZE: f32 = 20.0;

pub struct VizVS;

impl VizVS {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&self, game1: &crate::game::Game, game2: &crate::game::Game, max_score_ever: usize) {
        clear_background(BLACK);
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Calculate grid size
        let grid_size = (screen_h - 100.0).min((screen_w - 500.0) / 2.0);
        let center_panel_w = 400.0;
        
        // Left grid (Snake 1)
        let left_x = 50.0;
        self.draw_grid(game1, left_x, 50.0, grid_size, SNAKE1_COLOR, "SNAKE #1");
        
        // Right grid (Snake 2)
        let right_x = screen_w - grid_size - 50.0;
        self.draw_grid(game2, right_x, 50.0, grid_size, SNAKE2_COLOR, "SNAKE #2");
        
        // Center panel
        let center_x = (screen_w - center_panel_w) / 2.0;
        self.draw_center_panel(game1, game2, center_x, 50.0, center_panel_w, screen_h - 100.0, max_score_ever);
    }

    fn draw_grid(&self, game: &crate::game::Game, x: f32, y: f32, size: f32, color: Color, title: &str) {
        let tile_size = size / GRID_W as f32;

        // Title
        let display_title = if title == "SNAKE #1" {
            "BEST EVER"
        } else {
            "2ND BEST"
        };
        draw_text(display_title, x, y - 15.0, TITLE_SIZE, color);

        // Border
        draw_rectangle(x - 5.0, y - 5.0, size + 10.0, size + 10.0, PANEL_BORDER);
        draw_rectangle(x, y, size, size, BLACK);

        // Grid lines
        for i in 0..=GRID_W {
            let line_x = x + i as f32 * tile_size;
            draw_line(line_x, y, line_x, y + size, 1.0, Color::new(0.15, 0.15, 0.15, 1.0));
        }
        for i in 0..=GRID_H {
            let line_y = y + i as f32 * tile_size;
            draw_line(x, line_y, x + size, line_y, 1.0, Color::new(0.15, 0.15, 0.15, 1.0));
        }

        // Food
        draw_rectangle(
            x + game.food.x as f32 * tile_size + 2.0,
            y + game.food.y as f32 * tile_size + 2.0,
            tile_size - 4.0,
            tile_size - 4.0,
            WHITE,
        );

        // Snake
        for (i, segment) in game.body.iter().enumerate() {
            let seg_color = if i == 0 {
                color
            } else {
                Color::new(color.r * 0.7, color.g * 0.7, color.b * 0.7, 1.0)
            };
            draw_rectangle(
                x + segment.x as f32 * tile_size + 1.0,
                y + segment.y as f32 * tile_size + 1.0,
                tile_size - 2.0,
                tile_size - 2.0,
                seg_color,
            );
        }

        // Status indicator
        if game.is_complete {
            draw_text("DEAD", x + size / 2.0 - 40.0, y + size + 30.0, 24.0, RED);
        }
    }

    fn draw_center_panel(&self, game1: &crate::game::Game, game2: &crate::game::Game, x: f32, y: f32, w: f32, h: f32, max_score_ever: usize) {
        draw_rectangle(x, y, w, h, PANEL_BG);
        draw_rectangle_lines(x, y, w, h, 4.0, PANEL_BORDER);

        let mut cy = y + 60.0;

        // VS Title
        draw_text("VS", x + w / 2.0 - 30.0, cy, 48.0, Color::new(1.0, 0.8, 0.0, 1.0));
        cy += 60.0;

        // Historical Max Score
        draw_text("RECORD TO BEAT", x + w / 2.0 - 100.0, cy, 18.0, Color::new(0.7, 0.7, 0.7, 1.0));
        cy += 30.0;
        draw_text(&format!("{}", max_score_ever), x + w / 2.0 - 20.0, cy, 36.0, Color::new(1.0, 0.8, 0.0, 1.0));
        cy += 60.0;

        // Scores
        let score1 = game1.score();
        let score2 = game2.score();
        
        draw_text("SCORE", x + w / 2.0 - 50.0, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 40.0;
        
        // Check for new records
        let is_record1 = score1 > max_score_ever;
        let is_record2 = score2 > max_score_ever;
        
        let color1 = if is_record1 { Color::new(1.0, 0.8, 0.0, 1.0) } else { SNAKE1_COLOR };
        let color2 = if is_record2 { Color::new(1.0, 0.8, 0.0, 1.0) } else { SNAKE2_COLOR };
        
        draw_text(&format!("{}", score1), x + 80.0, cy, 42.0, color1);
        if is_record1 {
            draw_text("NEW!", x + 60.0, cy + 30.0, 16.0, Color::new(1.0, 0.8, 0.0, 1.0));
        }
        
        draw_text(&format!("{}", score2), x + w - 120.0, cy, 42.0, color2);
        if is_record2 {
            draw_text("NEW!", x + w - 140.0, cy + 30.0, 16.0, Color::new(1.0, 0.8, 0.0, 1.0));
        }
        cy += 80.0;

        // Steps
        draw_text("STEPS", x + w / 2.0 - 50.0, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 35.0;
        draw_text(&format!("{}", game1.num_steps), x + 80.0, cy, TEXT_SIZE, SNAKE1_COLOR);
        draw_text(&format!("{}", game2.num_steps), x + w - 120.0, cy, TEXT_SIZE, SNAKE2_COLOR);
        cy += 60.0;

        // Fitness
        draw_text("FITNESS", x + w / 2.0 - 60.0, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 35.0;
        draw_text(&format!("{:.1}", game1.fitness()), x + 80.0, cy, TEXT_SIZE, SNAKE1_COLOR);
        draw_text(&format!("{:.1}", game2.fitness()), x + w - 120.0, cy, TEXT_SIZE, SNAKE2_COLOR);
        cy += 80.0;

        // Winner indicator
        if game1.is_complete && game2.is_complete {
            cy += 20.0;
            let winner = if score1 > score2 {
                "BEST EVER WINS!"
            } else if score2 > score1 {
                "2ND BEST WINS!"
            } else {
                "TIE!"
            };
            let winner_color = if score1 > score2 {
                SNAKE1_COLOR
            } else if score2 > score1 {
                SNAKE2_COLOR
            } else {
                YELLOW
            };
            draw_text(winner, x + w / 2.0 - 100.0, cy, 32.0, winner_color);
            cy += 60.0;
            draw_text("[V] Back to Training", x + w / 2.0 - 120.0, cy, 18.0, TEXT_COLOR);
        } else if game1.is_complete {
            draw_text("Best Ever eliminated!", x + 60.0, cy, 20.0, RED);
        } else if game2.is_complete {
            draw_text("2nd Best eliminated!", x + 60.0, cy, 20.0, RED);
        }

        // Controls
        cy = y + h - 40.0;
        draw_text("[SPACE] Slow  [ESC] Quit", x + w / 2.0 - 120.0, cy, 16.0, Color::new(0.6, 0.6, 0.6, 1.0));
    }
}

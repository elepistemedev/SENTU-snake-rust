//! VS Mode visualization - Two snakes battle head-to-head

use crate::*;
use macroquad::prelude::*;

const PANEL_BG: Color = Color::new(0.05, 0.05, 0.05, 0.95);
const PANEL_BORDER: Color = Color::new(0.4, 0.4, 0.4, 1.0);
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const SNAKE1_COLOR: Color = Color::new(0.3, 0.9, 0.3, 1.0);
const SNAKE2_COLOR: Color = Color::new(0.9, 0.3, 0.3, 1.0);
const GOLD: Color = Color::new(1.0, 0.8, 0.0, 1.0);
const RECORD_LABEL_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);
const CONTROLS_COLOR: Color = Color::new(0.6, 0.6, 0.6, 1.0);
const TITLE_SIZE: f32 = 28.0;
const TEXT_SIZE: f32 = 20.0;

/// Render configuration for a versus match.
///
/// Every label, title, and accent color the arena draws comes from here. The
/// [`VsFlavor::ga_default`] flavor reproduces the pre-flavor hardcoded GA
/// output byte-for-byte; DQN-internal and cross flavors relabel the players and
/// set `record: None`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VsFlavor {
    /// Title drawn above player 1's grid.
    pub player1_title: &'static str,
    /// Title drawn above player 2's grid.
    pub player2_title: &'static str,
    /// Player 1 accent color (head + labels).
    pub player1_color: Color,
    /// Player 2 accent color (head + labels).
    pub player2_color: Color,
    /// When `Some(record)` a "record to beat" section is shown and scores that
    /// beat it are tagged as new records; `None` renders no record semantics.
    pub record: Option<usize>,
    /// Section label shown above the record value.
    pub record_beat_label: &'static str,
    /// Tag drawn next to a score that beats the record.
    pub new_record_label: &'static str,
    /// Banner shown when player 1 wins.
    pub winner1_label: &'static str,
    /// Banner shown when player 2 wins.
    pub winner2_label: &'static str,
    /// Banner shown on a tied finish.
    pub tie_label: &'static str,
    /// Message shown when player 1 dies before player 2.
    pub eliminated1_label: &'static str,
    /// Message shown when player 2 dies before player 1.
    pub eliminated2_label: &'static str,
    /// Hint drawn after a finished match.
    pub back_label: &'static str,
    /// Hint drawn at the bottom of the center panel.
    pub controls_label: &'static str,
}

impl VsFlavor {
    /// The GA internal flavor: identical titles, colors, record semantics, and
    /// strings to the legacy hardcoded versus output.
    pub fn ga_default(record: usize) -> Self {
        Self {
            player1_title: "BEST EVER",
            player2_title: "2ND BEST",
            player1_color: SNAKE1_COLOR,
            player2_color: SNAKE2_COLOR,
            record: Some(record),
            record_beat_label: "RECORD TO BEAT",
            new_record_label: "NEW!",
            winner1_label: "BEST EVER WINS!",
            winner2_label: "2ND BEST WINS!",
            tie_label: "TIE!",
            eliminated1_label: "Best Ever eliminated!",
            eliminated2_label: "2nd Best eliminated!",
            back_label: "[V] Back to Training",
            controls_label: "[SPACE] Slow  [ESC] Quit",
        }
    }
}

pub struct VizVS;

impl Default for VizVS {
    fn default() -> Self {
        Self::new()
    }
}

impl VizVS {
    pub fn new() -> Self {
        Self
    }

    /// Legacy draw entry: routes through the GA-default flavor so the existing
    /// GA versus output is unchanged.
    pub fn draw(
        &self,
        game1: &crate::game::Game,
        game2: &crate::game::Game,
        max_score_ever: usize,
    ) {
        self.draw_flavored(game1, game2, &VsFlavor::ga_default(max_score_ever));
    }

    /// Draw a versus match for any [`VsFlavor`].
    pub fn draw_flavored(
        &self,
        game1: &crate::game::Game,
        game2: &crate::game::Game,
        flavor: &VsFlavor,
    ) {
        clear_background(BLACK);
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Calculate grid size
        let grid_size = (screen_h - 100.0).min((screen_w - 500.0) / 2.0);
        let center_panel_w = 400.0;

        // Left grid (Snake 1)
        let left_x = 50.0;
        self.draw_grid(
            game1,
            left_x,
            50.0,
            grid_size,
            flavor.player1_color,
            flavor.player1_title,
        );

        // Right grid (Snake 2)
        let right_x = screen_w - grid_size - 50.0;
        self.draw_grid(
            game2,
            right_x,
            50.0,
            grid_size,
            flavor.player2_color,
            flavor.player2_title,
        );

        // Center panel
        let center_x = (screen_w - center_panel_w) / 2.0;
        self.draw_center_panel(
            game1,
            game2,
            center_x,
            50.0,
            center_panel_w,
            screen_h - 100.0,
            flavor,
        );
    }

    fn draw_grid(
        &self,
        game: &crate::game::Game,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        title: &str,
    ) {
        let tile_size = size / GRID_W as f32;

        // Title
        draw_text(title, x, y - 15.0, TITLE_SIZE, color);

        // Border
        draw_rectangle(x - 5.0, y - 5.0, size + 10.0, size + 10.0, PANEL_BORDER);
        draw_rectangle(x, y, size, size, BLACK);

        // Grid lines
        for i in 0..=GRID_W {
            let line_x = x + i as f32 * tile_size;
            draw_line(
                line_x,
                y,
                line_x,
                y + size,
                1.0,
                Color::new(0.15, 0.15, 0.15, 1.0),
            );
        }
        for i in 0..=GRID_H {
            let line_y = y + i as f32 * tile_size;
            draw_line(
                x,
                line_y,
                x + size,
                line_y,
                1.0,
                Color::new(0.15, 0.15, 0.15, 1.0),
            );
        }

        // Food
        let theme_food = crate::theme::load_theme().colors().food;
        draw_rectangle(
            x + game.food.x as f32 * tile_size + 2.0,
            y + game.food.y as f32 * tile_size + 2.0,
            tile_size - 4.0,
            tile_size - 4.0,
            theme_food,
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

    fn draw_center_panel(
        &self,
        game1: &crate::game::Game,
        game2: &crate::game::Game,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        flavor: &VsFlavor,
    ) {
        draw_rectangle(x, y, w, h, PANEL_BG);
        draw_rectangle_lines(x, y, w, h, 4.0, PANEL_BORDER);

        let mut cy = y + 60.0;

        // VS Title
        draw_text("VS", x + w / 2.0 - 30.0, cy, 48.0, GOLD);
        cy += 60.0;

        // Historical record (only for record-bearing flavors, e.g. GA)
        if let Some(record) = flavor.record {
            draw_text(
                flavor.record_beat_label,
                x + w / 2.0 - 100.0,
                cy,
                18.0,
                RECORD_LABEL_COLOR,
            );
            cy += 30.0;
            draw_text(format!("{record}"), x + w / 2.0 - 20.0, cy, 36.0, GOLD);
            cy += 60.0;
        }

        // Scores
        let score1 = game1.score();
        let score2 = game2.score();

        draw_text("SCORE", x + w / 2.0 - 50.0, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 40.0;

        // Check for new records
        let is_record1 = flavor.record.is_some_and(|record| score1 > record);
        let is_record2 = flavor.record.is_some_and(|record| score2 > record);

        let color1 = if is_record1 {
            GOLD
        } else {
            flavor.player1_color
        };
        let color2 = if is_record2 {
            GOLD
        } else {
            flavor.player2_color
        };

        draw_text(format!("{score1}"), x + 80.0, cy, 42.0, color1);
        if is_record1 {
            draw_text(flavor.new_record_label, x + 60.0, cy + 30.0, 16.0, GOLD);
        }

        draw_text(format!("{score2}"), x + w - 120.0, cy, 42.0, color2);
        if is_record2 {
            draw_text(
                flavor.new_record_label,
                x + w - 140.0,
                cy + 30.0,
                16.0,
                GOLD,
            );
        }
        cy += 80.0;

        // Steps
        draw_text("STEPS", x + w / 2.0 - 50.0, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 35.0;
        draw_text(
            format!("{}", game1.num_steps),
            x + 80.0,
            cy,
            TEXT_SIZE,
            flavor.player1_color,
        );
        draw_text(
            format!("{}", game2.num_steps),
            x + w - 120.0,
            cy,
            TEXT_SIZE,
            flavor.player2_color,
        );
        cy += 60.0;

        // Fitness
        draw_text("FITNESS", x + w / 2.0 - 60.0, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 35.0;
        draw_text(
            format!("{:.1}", game1.fitness()),
            x + 80.0,
            cy,
            TEXT_SIZE,
            flavor.player1_color,
        );
        draw_text(
            format!("{:.1}", game2.fitness()),
            x + w - 120.0,
            cy,
            TEXT_SIZE,
            flavor.player2_color,
        );
        cy += 80.0;

        // Winner indicator
        if game1.is_complete && game2.is_complete {
            cy += 20.0;
            let winner = if score1 > score2 {
                flavor.winner1_label
            } else if score2 > score1 {
                flavor.winner2_label
            } else {
                flavor.tie_label
            };
            let winner_color = if score1 > score2 {
                flavor.player1_color
            } else if score2 > score1 {
                flavor.player2_color
            } else {
                YELLOW
            };
            draw_text(winner, x + w / 2.0 - 100.0, cy, 32.0, winner_color);
            cy += 60.0;
            draw_text(flavor.back_label, x + w / 2.0 - 120.0, cy, 18.0, TEXT_COLOR);
        } else if game1.is_complete {
            draw_text(flavor.eliminated1_label, x + 60.0, cy, 20.0, RED);
        } else if game2.is_complete {
            draw_text(flavor.eliminated2_label, x + 60.0, cy, 20.0, RED);
        }

        // Controls
        cy = y + h - 40.0;
        draw_text(
            flavor.controls_label,
            x + w / 2.0 - 120.0,
            cy,
            16.0,
            CONTROLS_COLOR,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The GA-default flavor must reproduce today's strings byte-for-byte
    /// so the pre-flavor versus output never regresses.
    #[test]
    fn ga_default_flavor_pins_legacy_strings() {
        let f = VsFlavor::ga_default(37);
        assert_eq!(f.player1_title, "BEST EVER");
        assert_eq!(f.player2_title, "2ND BEST");
        assert_eq!(f.record, Some(37));
        assert_eq!(f.record_beat_label, "RECORD TO BEAT");
        assert_eq!(f.new_record_label, "NEW!");
        assert_eq!(f.winner1_label, "BEST EVER WINS!");
        assert_eq!(f.winner2_label, "2ND BEST WINS!");
        assert_eq!(f.tie_label, "TIE!");
        assert_eq!(f.eliminated1_label, "Best Ever eliminated!");
        assert_eq!(f.eliminated2_label, "2nd Best eliminated!");
        assert_eq!(f.back_label, "[V] Back to Training");
        assert_eq!(f.controls_label, "[SPACE] Slow  [ESC] Quit");
    }

    #[test]
    fn ga_default_flavor_pins_legacy_colors() {
        let f = VsFlavor::ga_default(0);
        assert!((f.player1_color.r - 0.3).abs() < 1e-6 && (f.player1_color.g - 0.9).abs() < 1e-6);
        assert!((f.player2_color.r - 0.9).abs() < 1e-6 && (f.player2_color.b - 0.3).abs() < 1e-6);
    }

    #[test]
    fn record_none_flavor_carries_no_record_semantics() {
        let f = VsFlavor::ga_default(0);
        let none_flavor = VsFlavor { record: None, ..f };
        assert_eq!(none_flavor.record, None);
    }
}

//! VS Mode visualization - Two snakes battle head-to-head

use crate::*;
use crate::ui_kit::{
    draw_centered_text, draw_terminal_box, ACCENT_CYAN, ACCENT_GOLD, ACCENT_GREEN, ACCENT_RED,
    TEXT_MUTED,
};
use crate::versus::SeriesInfo;
use macroquad::prelude::*;

const PANEL_BORDER: Color = Color::new(0.4, 0.4, 0.4, 1.0);
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const SNAKE1_COLOR: Color = Color::new(0.3, 0.9, 0.3, 1.0);
const SNAKE2_COLOR: Color = Color::new(0.9, 0.3, 0.3, 1.0);
const TITLE_SIZE: f32 = 28.0;
const TEXT_SIZE: f32 = 20.0;

/// Pure representation of a stage's badge display state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageBadgeState {
    WonLeft { score1: usize, score2: usize },
    WonRight { score1: usize, score2: usize },
    Tie { score1: usize, score2: usize },
    InProgress,
    Pending,
}

/// Pure computation of a stage's state given the series scoreboard.
pub fn stage_badge_state(info: &SeriesInfo, stage_idx: usize) -> StageBadgeState {
    if let Some(res) = info.stages.get(stage_idx).and_then(|s| *s) {
        match res.winner {
            crate::versus::Winner::Left => StageBadgeState::WonLeft {
                score1: res.score1,
                score2: res.score2,
            },
            crate::versus::Winner::Right => StageBadgeState::WonRight {
                score1: res.score1,
                score2: res.score2,
            },
            crate::versus::Winner::Tie => StageBadgeState::Tie {
                score1: res.score1,
                score2: res.score2,
            },
        }
    } else if stage_idx == info.games_played && !info.is_over {
        StageBadgeState::InProgress
    } else {
        StageBadgeState::Pending
    }
}

/// Pure helper to format a single stage result into a readable summary string.
pub fn format_stage_summary(stage: &crate::versus::StageResult, flavor: &VsFlavor) -> String {
    let winner_name = match stage.winner {
        crate::versus::Winner::Left => flavor.player1_title,
        crate::versus::Winner::Right => flavor.player2_title,
        crate::versus::Winner::Tie => "Empate",
    };
    format!(
        "S{}: {} ({}-{})",
        stage.stage_number, winner_name, stage.score1, stage.score2
    )
}

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

        // Center panel - shifted from 50.0 to 75.0 to avoid Series HUD collisions
        let center_x = (screen_w - center_panel_w) / 2.0;
        self.draw_center_panel(
            game1,
            game2,
            center_x,
            75.0,
            center_panel_w,
            screen_h - 125.0,
            flavor,
            None,
        );
    }

    /// Draw a versus match with the series HUD overlay.
    ///
    /// Renders the grids, center panel with stage history, top series HUD,
    /// and if finished, overlays the celebratory victory popup modal.
    pub fn draw_series(
        &self,
        game1: &crate::game::Game,
        game2: &crate::game::Game,
        flavor: &VsFlavor,
        info: &SeriesInfo,
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

        // Center panel with SeriesInfo stage history
        let center_x = (screen_w - center_panel_w) / 2.0;
        self.draw_center_panel(
            game1,
            game2,
            center_x,
            75.0,
            center_panel_w,
            screen_h - 125.0,
            flavor,
            Some(info),
        );

        self.draw_series_hud(center_x, center_panel_w, flavor, info);

        // Celebratory victory popup modal when series concludes
        if info.is_over {
            self.draw_victory_popup(screen_w, screen_h, flavor, info);
        }
    }

    /// Draw the series scoreboard banner above the center panel.
    ///
    /// Layout (dedicated top margin y = 0.0..75.0, outside the center panel):
    /// - y = 18.0: GAME N / 5
    /// - y = 36.0: Win pips (Player 1 left, Player 2 right)
    /// - y = 56.0: Series Winner banner when finished
    fn draw_series_hud(
        &self,
        panel_x: f32,
        panel_w: f32,
        flavor: &VsFlavor,
        info: &SeriesInfo,
    ) {
        let center_x = panel_x + panel_w * 0.5;

        // "GAME N / 5" at y = 18.0
        let game_no = (info.games_played + 1).min(info.total_games);
        let label = format!("GAME {} / {}", game_no, info.total_games);
        draw_centered_text(&label, center_x, 18.0, 18.0, ACCENT_GOLD);

        // Win pips at y = 36.0: Player 1 (left) and Player 2 (right).
        // Best-of-5 indicator pips with player colors.
        let pip_r = 5.0;
        let pip_gap = 18.0;
        let total_pips = crate::versus::SERIES_GAMES;
        let pip_y = 36.0;

        // Player 1 pips (left side, expanding outward from center)
        for i in 0..total_pips {
            let cx = center_x - 24.0 - i as f32 * pip_gap;
            let filled = i < info.left_wins;
            let col = flavor.player1_color;
            if filled {
                // Glowing circular indicator pip with player color
                draw_circle(cx, pip_y, pip_r + 2.0, Color::new(col.r, col.g, col.b, 0.25));
                draw_circle(cx, pip_y, pip_r, col);
            } else {
                draw_circle_lines(cx, pip_y, pip_r, 1.5, Color::new(col.r, col.g, col.b, 0.35));
            }
        }

        // Player 2 pips (right side, expanding outward from center)
        for i in 0..total_pips {
            let cx = center_x + 24.0 + i as f32 * pip_gap;
            let filled = i < info.right_wins;
            let col = flavor.player2_color;
            if filled {
                // Glowing circular indicator pip with player color
                draw_circle(cx, pip_y, pip_r + 2.0, Color::new(col.r, col.g, col.b, 0.25));
                draw_circle(cx, pip_y, pip_r, col);
            } else {
                draw_circle_lines(cx, pip_y, pip_r, 1.5, Color::new(col.r, col.g, col.b, 0.35));
            }
        }

        // Series champion banner at y = 56.0 (only when the series is over).
        // Positioned outside the center panel (which starts at y = 75.0).
        if info.is_over {
            let (banner, color) = if info.left_wins > info.right_wins {
                (format!("🏆 {} WINS SERIES!", flavor.player1_title), flavor.player1_color)
            } else if info.right_wins > info.left_wins {
                (format!("🏆 {} WINS SERIES!", flavor.player2_title), flavor.player2_color)
            } else {
                ("SERIES TIE!".to_string(), ACCENT_GOLD)
            };
            draw_centered_text(&banner, center_x, 56.0, 20.0, color);
        }
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

        // Title centered over grid
        draw_centered_text(title, x + size * 0.5, y - 15.0, TITLE_SIZE, color);

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
        let theme = crate::theme::load_theme();
        crate::render_snake::draw_apple(
            x + game.food.x as f32 * tile_size,
            y + game.food.y as f32 * tile_size,
            tile_size,
            theme,
            theme.colors().food,
        );

        // Snake
        for (i, segment) in game.body.iter().enumerate() {
            let seg_x = x + segment.x as f32 * tile_size;
            let seg_y = y + segment.y as f32 * tile_size;
            let seg_color = if i == 0 {
                color
            } else {
                Color::new(color.r * 0.7, color.g * 0.7, color.b * 0.7, 1.0)
            };
            let bulge = game.swallow.bulge_at(i);
            let head_scale = if i == 0 {
                game.swallow.head_scale()
            } else {
                0.0
            };
            crate::render_snake::draw_connected_segment(
                &game.body,
                i,
                game.dir,
                seg_x,
                seg_y,
                tile_size,
                theme,
                seg_color,
                bulge,
                head_scale,
            );
        }

        // Status indicator centered under grid
        if game.is_complete {
            draw_centered_text("DEAD", x + size * 0.5, y + size + 30.0, 24.0, ACCENT_RED);
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
        series_info: Option<&SeriesInfo>,
    ) {
        draw_terminal_box(x, y, w, h, "", false);

        let center_cx = x + w * 0.5;
        let col1_cx = x + w * 0.25;
        let col2_cx = x + w * 0.75;

        let mut cy = y + 36.0;

        // VS Title
        draw_centered_text("VS", center_cx, cy, 38.0, ACCENT_GOLD);
        cy += 26.0;

        // Stage breakdown bar in center panel
        if let Some(info) = series_info {
            self.draw_stages_bar(x, cy, w, flavor, info);
            cy += 46.0;
        } else {
            cy += 12.0;
        }

        // Historical record (only for record-bearing flavors, e.g. GA)
        if let Some(record) = flavor.record {
            draw_centered_text(flavor.record_beat_label, center_cx, cy, 15.0, TEXT_MUTED);
            cy += 20.0;
            draw_centered_text(&format!("{record}"), center_cx, cy, 28.0, ACCENT_GOLD);
            cy += 38.0;
        }

        // Scores section
        let score1 = game1.score();
        let score2 = game2.score();

        draw_centered_text("SCORE", center_cx, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 34.0;

        // Check for new records
        let is_record1 = flavor.record.is_some_and(|record| score1 > record);
        let is_record2 = flavor.record.is_some_and(|record| score2 > record);

        let color1 = if is_record1 {
            ACCENT_GOLD
        } else {
            flavor.player1_color
        };
        let color2 = if is_record2 {
            ACCENT_GOLD
        } else {
            flavor.player2_color
        };

        draw_centered_text(&format!("{score1}"), col1_cx, cy, 38.0, color1);
        draw_centered_text(&format!("{score2}"), col2_cx, cy, 38.0, color2);

        if is_record1 {
            draw_centered_text(flavor.new_record_label, col1_cx, cy + 22.0, 14.0, ACCENT_GOLD);
        }
        if is_record2 {
            draw_centered_text(flavor.new_record_label, col2_cx, cy + 22.0, 14.0, ACCENT_GOLD);
        }
        cy += 46.0;

        // Steps section
        draw_centered_text("STEPS", center_cx, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 26.0;
        draw_centered_text(
            &format!("{}", game1.num_steps),
            col1_cx,
            cy,
            TEXT_SIZE,
            flavor.player1_color,
        );
        draw_centered_text(
            &format!("{}", game2.num_steps),
            col2_cx,
            cy,
            TEXT_SIZE,
            flavor.player2_color,
        );
        cy += 42.0;

        // Fitness section
        draw_centered_text("FITNESS", center_cx, cy, TEXT_SIZE, TEXT_COLOR);
        cy += 26.0;
        draw_centered_text(
            &format!("{:.1}", game1.fitness()),
            col1_cx,
            cy,
            TEXT_SIZE,
            flavor.player1_color,
        );
        draw_centered_text(
            &format!("{:.1}", game2.fitness()),
            col2_cx,
            cy,
            TEXT_SIZE,
            flavor.player2_color,
        );
        cy += 48.0;

        // Winner indicator
        if game1.is_complete && game2.is_complete {
            cy += 8.0;
            let (winner, winner_color) = if score1 > score2 {
                (flavor.winner1_label, flavor.player1_color)
            } else if score2 > score1 {
                (flavor.winner2_label, flavor.player2_color)
            } else {
                (flavor.tie_label, ACCENT_GOLD)
            };
            draw_centered_text(winner, center_cx, cy, 26.0, winner_color);
            cy += 32.0;
            draw_centered_text(flavor.back_label, center_cx, cy, 16.0, TEXT_COLOR);
        } else if game1.is_complete {
            draw_centered_text(flavor.eliminated1_label, center_cx, cy, 16.0, ACCENT_RED);
        } else if game2.is_complete {
            draw_centered_text(flavor.eliminated2_label, center_cx, cy, 16.0, ACCENT_RED);
        }

        // Controls
        let controls_y = y + h - 22.0;
        draw_centered_text(
            flavor.controls_label,
            center_cx,
            controls_y,
            14.0,
            TEXT_MUTED,
        );
    }

    /// Draw a 5-slot horizontal stage breakdown bar inside the center panel.
    fn draw_stages_bar(
        &self,
        panel_x: f32,
        y: f32,
        panel_w: f32,
        flavor: &VsFlavor,
        info: &SeriesInfo,
    ) {
        let total_stages = crate::versus::SERIES_GAMES;
        let slot_w = 68.0;
        let gap = 8.0;
        let total_w = total_stages as f32 * slot_w + (total_stages - 1) as f32 * gap;
        let start_x = panel_x + (panel_w - total_w) * 0.5;
        let slot_h = 36.0;

        for i in 0..total_stages {
            let sx = start_x + i as f32 * (slot_w + gap);
            let state = stage_badge_state(info, i);

            match state {
                StageBadgeState::WonLeft { score1, score2 } => {
                    let col = flavor.player1_color;
                    let bg = Color::new(col.r * 0.18, col.g * 0.18, col.b * 0.18, 0.95);
                    draw_rectangle(sx, y, slot_w, slot_h, bg);
                    draw_rectangle_lines(sx, y, slot_w, slot_h, 1.5, col);
                    draw_text(&format!("S{}", i + 1), sx + 6.0, y + 14.0, 11.0, TEXT_MUTED);
                    draw_text("GANA", sx + 28.0, y + 14.0, 11.0, col);
                    let score_str = format!("{score1}-{score2}");
                    draw_centered_text(&score_str, sx + slot_w * 0.5, y + 29.0, 12.0, WHITE);
                }
                StageBadgeState::WonRight { score1, score2 } => {
                    let col = flavor.player2_color;
                    let bg = Color::new(col.r * 0.18, col.g * 0.18, col.b * 0.18, 0.95);
                    draw_rectangle(sx, y, slot_w, slot_h, bg);
                    draw_rectangle_lines(sx, y, slot_w, slot_h, 1.5, col);
                    draw_text(&format!("S{}", i + 1), sx + 6.0, y + 14.0, 11.0, TEXT_MUTED);
                    draw_text("GANA", sx + 28.0, y + 14.0, 11.0, col);
                    let score_str = format!("{score1}-{score2}");
                    draw_centered_text(&score_str, sx + slot_w * 0.5, y + 29.0, 12.0, WHITE);
                }
                StageBadgeState::Tie { score1, score2 } => {
                    let col = ACCENT_GOLD;
                    let bg = Color::new(col.r * 0.15, col.g * 0.15, col.b * 0.15, 0.95);
                    draw_rectangle(sx, y, slot_w, slot_h, bg);
                    draw_rectangle_lines(sx, y, slot_w, slot_h, 1.5, col);
                    draw_text(&format!("S{}", i + 1), sx + 6.0, y + 14.0, 11.0, TEXT_MUTED);
                    draw_text("EMP", sx + 28.0, y + 14.0, 11.0, col);
                    let score_str = format!("{score1}-{score2}");
                    draw_centered_text(&score_str, sx + slot_w * 0.5, y + 29.0, 12.0, WHITE);
                }
                StageBadgeState::InProgress => {
                    let pulse = ((get_time() as f32 * 5.0).sin() * 0.5 + 0.5) * 0.3 + 0.7;
                    let cyan_pulse = Color::new(ACCENT_CYAN.r * pulse, ACCENT_CYAN.g * pulse, ACCENT_CYAN.b * pulse, 1.0);
                    draw_rectangle(sx, y, slot_w, slot_h, Color::new(0.0, 0.18, 0.22, 0.9));
                    draw_rectangle_lines(sx, y, slot_w, slot_h, 1.5, cyan_pulse);
                    draw_centered_text(&format!("S{}", i + 1), sx + slot_w * 0.5, y + 14.0, 11.0, ACCENT_CYAN);
                    draw_centered_text("EN JUEGO", sx + slot_w * 0.5, y + 29.0, 11.0, WHITE);
                }
                StageBadgeState::Pending => {
                    draw_rectangle(sx, y, slot_w, slot_h, Color::new(0.06, 0.07, 0.09, 0.6));
                    draw_rectangle_lines(sx, y, slot_w, slot_h, 1.0, PANEL_BORDER);
                    draw_centered_text(&format!("S{}", i + 1), sx + slot_w * 0.5, y + 14.0, 11.0, TEXT_MUTED);
                    draw_centered_text("- -", sx + slot_w * 0.5, y + 29.0, 11.0, Color::new(0.35, 0.40, 0.45, 1.0));
                }
            }
        }
    }

    /// Draw the celebratory victory popup modal when the series ends with a champion.
    fn draw_victory_popup(
        &self,
        screen_w: f32,
        screen_h: f32,
        flavor: &VsFlavor,
        info: &SeriesInfo,
    ) {
        // 1. Semi-transparent backdrop overlay
        draw_rectangle(0.0, 0.0, screen_w, screen_h, Color::new(0.02, 0.02, 0.05, 0.84));

        let modal_w = 560.0f32.min(screen_w - 40.0);
        let modal_h = 390.0f32.min(screen_h - 40.0);
        let modal_x = (screen_w - modal_w) * 0.5;
        let modal_y = (screen_h - modal_h) * 0.5;

        let (winner_title, winner_color) = if info.left_wins > info.right_wins {
            (flavor.player1_title, flavor.player1_color)
        } else if info.right_wins > info.left_wins {
            (flavor.player2_title, flavor.player2_color)
        } else {
            ("EMPATE", ACCENT_GOLD)
        };

        // Modal terminal box with glowing border in winner's color
        draw_terminal_box(modal_x, modal_y, modal_w, modal_h, "🏆 ¡CAMPEÓN DE LA SERIE! 🏆", true);
        draw_rectangle_lines(modal_x, modal_y, modal_w, modal_h, 2.5, winner_color);

        let cx = modal_x + modal_w * 0.5;
        let mut cy = modal_y + 32.0;

        let t = get_time() as f32;

        // 2. Animated Confetti / celebratory twinkling stars
        let confetti_colors = [winner_color, ACCENT_GOLD, ACCENT_CYAN, WHITE, ACCENT_GREEN];
        for i in 0..16 {
            let angle = i as f32 * 0.3927 + t * 0.8;
            let radius = 190.0 + (t * 2.0 + i as f32).sin() * 25.0;
            let px = cx + angle.cos() * radius;
            let py = (modal_y + 130.0) + angle.sin() * (radius * 0.55);
            if px > modal_x + 10.0 && px < modal_x + modal_w - 10.0 && py > modal_y + 20.0 && py < modal_y + modal_h - 20.0 {
                let conf_col = confetti_colors[i % confetti_colors.len()];
                let size = 3.0 + ((t * 4.0 + i as f32).sin().abs() * 3.0);
                draw_rectangle(px, py, size, size, conf_col);
            }
        }

        // 3. Celebrating Snake Graphic
        let snake_center_y = cy + 42.0;
        let snake_len = 7;
        let seg_size = 14.0;
        let bounce = (t * 4.0).sin() * 5.0;

        for i in (0..snake_len).rev() {
            let offset_x = (i as f32 - 3.0) * (seg_size * 1.2);
            let wave = (t * 5.0 - i as f32 * 0.6).sin() * 8.0;
            let seg_x = cx + offset_x;
            let seg_y = snake_center_y + wave + bounce;

            let col = if i == 0 {
                winner_color
            } else {
                Color::new(winner_color.r * 0.75, winner_color.g * 0.75, winner_color.b * 0.75, 1.0)
            };

            // Segment circle with glow
            draw_circle(seg_x, seg_y, seg_size * 0.5 + 2.0, Color::new(col.r, col.g, col.b, 0.3));
            draw_circle(seg_x, seg_y, seg_size * 0.5, col);

            // If head (i == 0): draw happy eyes and a golden crown!
            if i == 0 {
                // Happy eyes
                draw_circle(seg_x - 3.0, seg_y - 2.0, 3.0, WHITE);
                draw_circle(seg_x + 3.0, seg_y - 2.0, 3.0, WHITE);
                draw_circle(seg_x - 3.0, seg_y - 2.0, 1.5, BLACK);
                draw_circle(seg_x + 3.0, seg_y - 2.0, 1.5, BLACK);

                // Golden Crown 👑 over the head
                let crown_y = seg_y - 18.0;
                draw_triangle(
                    Vec2::new(seg_x - 10.0, crown_y),
                    Vec2::new(seg_x - 6.0, crown_y - 9.0),
                    Vec2::new(seg_x - 2.0, crown_y),
                    ACCENT_GOLD,
                );
                draw_triangle(
                    Vec2::new(seg_x - 4.0, crown_y),
                    Vec2::new(seg_x, crown_y - 12.0),
                    Vec2::new(seg_x + 4.0, crown_y),
                    ACCENT_GOLD,
                );
                draw_triangle(
                    Vec2::new(seg_x + 2.0, crown_y),
                    Vec2::new(seg_x + 6.0, crown_y - 9.0),
                    Vec2::new(seg_x + 10.0, crown_y),
                    ACCENT_GOLD,
                );
                draw_line(seg_x - 10.0, crown_y, seg_x + 10.0, crown_y, 2.5, ACCENT_GOLD);
            }
        }

        cy += 100.0;

        // 4. Winner Announcement Text
        let victory_str = if info.left_wins == info.right_wins {
            "¡SERIE EMPATADA!".to_string()
        } else {
            format!("¡{} ES EL CAMPEÓN!", winner_title)
        };
        draw_centered_text(&victory_str, cx, cy, 26.0, winner_color);
        cy += 30.0;

        // Series score comparison
        let score_line = format!(
            "{} [{}]  VS  [{}] {}",
            flavor.player1_title, info.left_wins, info.right_wins, flavor.player2_title
        );
        draw_centered_text(&score_line, cx, cy, 20.0, ACCENT_GOLD);
        cy += 34.0;

        // 5. Stage Breakdown Cards inside modal
        let stage_box_w = 480.0f32.min(modal_w - 40.0);
        let stage_box_x = cx - stage_box_w * 0.5;
        let stage_slot_w = (stage_box_w - 32.0) / 5.0;

        for i in 0..crate::versus::SERIES_GAMES {
            let sx = stage_box_x + i as f32 * (stage_slot_w + 8.0);
            if let Some(stage) = info.stages[i] {
                let (bg_c, border_c, tag) = match stage.winner {
                    crate::versus::Winner::Left => (
                        Color::new(flavor.player1_color.r * 0.2, flavor.player1_color.g * 0.2, flavor.player1_color.b * 0.2, 0.9),
                        flavor.player1_color,
                        "J1",
                    ),
                    crate::versus::Winner::Right => (
                        Color::new(flavor.player2_color.r * 0.2, flavor.player2_color.g * 0.2, flavor.player2_color.b * 0.2, 0.9),
                        flavor.player2_color,
                        "J2",
                    ),
                    crate::versus::Winner::Tie => (
                        Color::new(ACCENT_GOLD.r * 0.15, ACCENT_GOLD.g * 0.15, ACCENT_GOLD.b * 0.15, 0.9),
                        ACCENT_GOLD,
                        "EMP",
                    ),
                };
                draw_rectangle(sx, cy, stage_slot_w, 36.0, bg_c);
                draw_rectangle_lines(sx, cy, stage_slot_w, 36.0, 1.5, border_c);
                draw_text(&format!("S{}: {}", i + 1, tag), sx + 6.0, cy + 15.0, 11.0, border_c);
                let sc = format!("{}-{}", stage.score1, stage.score2);
                draw_centered_text(&sc, sx + stage_slot_w * 0.5, cy + 29.0, 12.0, WHITE);
            } else {
                draw_rectangle(sx, cy, stage_slot_w, 36.0, Color::new(0.06, 0.07, 0.09, 0.6));
                draw_rectangle_lines(sx, cy, stage_slot_w, 36.0, 1.0, PANEL_BORDER);
                draw_centered_text(&format!("S{}", i + 1), sx + stage_slot_w * 0.5, cy + 16.0, 11.0, TEXT_MUTED);
                draw_centered_text("- -", sx + stage_slot_w * 0.5, cy + 29.0, 11.0, Color::new(0.35, 0.40, 0.45, 1.0));
            }
        }
        cy += 50.0;

        // 6. Navigation Buttons / Instructions
        let footer_text = "[ENTER] o [ESC] Menú Principal    [R] Revancha";
        draw_centered_text(footer_text, cx, cy, 15.0, TEXT_MUTED);
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

    #[test]
    fn center_panel_columns_are_symmetrically_aligned() {
        let panel_x = 100.0;
        let panel_w = 400.0;
        let center = panel_x + panel_w * 0.5;
        let col1 = panel_x + panel_w * 0.25;
        let col2 = panel_x + panel_w * 0.75;
        // Col 1 and Col 2 must be equidistant from center
        assert_eq!(center - col1, col2 - center);
        // Col 1 and Col 2 must have identical padding from panel outer borders
        assert_eq!(col1 - panel_x, (panel_x + panel_w) - col2);
    }

    #[test]
    fn hud_geometry_series_banner_is_above_panel() {
        let panel_y = 75.0;
        let game_y = 18.0;
        let pip_y = 36.0;
        let banner_y = 56.0;

        assert!(game_y < pip_y, "Game label must sit above pips");
        assert!(pip_y < banner_y, "Pips must sit above series winner banner");
        assert!(
            banner_y < panel_y,
            "Series winner banner must sit completely outside and above the center panel (y = 75.0)"
        );
    }

    #[test]
    fn series_pips_do_not_collide_at_center() {
        let center_x = 200.0;
        let pip_gap = 18.0;
        let pip_r = 5.0;
        let total_pips = crate::versus::SERIES_GAMES;

        // Innermost pips (index 0)
        let p1_inner_cx = center_x - 24.0;
        let p2_inner_cx = center_x + 24.0;

        // The gap between innermost pip edges must be strictly positive
        let clearance = (p2_inner_cx - pip_r) - (p1_inner_cx + pip_r);
        assert!(clearance >= 30.0, "Pips must have clear center clearance without overlapping");

        // Outermost pips (index 4)
        let p1_outer_cx = center_x - 24.0 - (total_pips - 1) as f32 * pip_gap;
        let p2_outer_cx = center_x + 24.0 + (total_pips - 1) as f32 * pip_gap;
        assert!(p1_outer_cx < p1_inner_cx);
        assert!(p2_outer_cx > p2_inner_cx);
    }

    #[test]
    fn test_stage_badge_state_and_summary() {
        let mut info = SeriesInfo {
            games_played: 1,
            total_games: 5,
            left_wins: 1,
            right_wins: 0,
            ties: 0,
            is_over: false,
            stages: [None; 5],
        };
        info.stages[0] = Some(crate::versus::StageResult {
            stage_number: 1,
            winner: crate::versus::Winner::Left,
            score1: 12,
            score2: 4,
        });

        assert_eq!(
            stage_badge_state(&info, 0),
            StageBadgeState::WonLeft { score1: 12, score2: 4 }
        );
        assert_eq!(stage_badge_state(&info, 1), StageBadgeState::InProgress);
        assert_eq!(stage_badge_state(&info, 2), StageBadgeState::Pending);

        let flavor = VsFlavor::ga_default(0);
        let summary = format_stage_summary(&info.stages[0].unwrap(), &flavor);
        assert_eq!(summary, "S1: BEST EVER (12-4)");
    }
}

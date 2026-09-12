//! Advanced visualization dashboard for Snake AI training (GA)
//!
//! Mirrors the retro-terminal UI/UX grammar of `dqn_dash`:
//! - Left column: Live multi-snake arena (rank 0 active theme + ghost population) and bottom GA config panel
//! - Center column: Full-height 12x8x4 Neural Network diagram with active argmax glow and value readouts
//! - Right column: Responsive stacked stats panels, normalized progress bars, and allocation-free charts

use crate::configs::*;
use crate::theme::GameTheme;
use crate::ui_kit::{
    draw_badge, draw_progress_bar, draw_responsive_chart, draw_terminal_box, ACCENT_CYAN,
    ACCENT_GOLD, ACCENT_GREEN, COLOR_BG, PANEL_BORDER, TEXT_MUTED,
};
use macroquad::prelude::*;

const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const TEXT_SIZE: f32 = 18.0;

pub struct VizAdvanced {
    gen_times: Vec<f32>,
    gen_scores: Vec<usize>,
    gen_scores_f32: Vec<f32>,
    max_history_size: usize,
}

impl Default for VizAdvanced {
    fn default() -> Self {
        Self::new()
    }
}

impl VizAdvanced {
    pub fn new() -> Self {
        Self {
            gen_times: Vec::new(),
            gen_scores: Vec::new(),
            gen_scores_f32: Vec::new(),
            max_history_size: 50,
        }
    }


    pub fn update_generation(&mut self, time: f32, score: usize) {
        self.gen_times.push(time);
        self.gen_scores.push(score);
        self.gen_scores_f32.push(score as f32);

        if self.gen_times.len() > self.max_history_size {
            self.gen_times.remove(0);
        }
        if self.gen_scores.len() > self.max_history_size {
            self.gen_scores.remove(0);
        }
        if self.gen_scores_f32.len() > self.max_history_size {
            self.gen_scores_f32.remove(0);
        }
    }

    pub fn get_history(&self) -> (Vec<f32>, Vec<usize>) {
        (self.gen_times.clone(), self.gen_scores.clone())
    }

    pub fn set_history(&mut self, mut times: Vec<f32>, mut scores: Vec<usize>) {
        if times.len() > self.max_history_size {
            times.drain(0..(times.len() - self.max_history_size));
        }
        if scores.len() > self.max_history_size {
            scores.drain(0..(scores.len() - self.max_history_size));
        }
        self.gen_scores_f32 = scores.iter().map(|&s| s as f32).collect();
        self.gen_times = times;
        self.gen_scores = scores;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        games: &[&crate::game::Game],
        gen: usize,
        max_score: usize,
        gen_max: usize,
        sim_time: f32,
        current_score: usize,
        fitness: f32,
        steps: usize,
        theme: GameTheme,
    ) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Left column: Grid + Model Info
        self.draw_game_grid(games, screen_w, screen_h, theme);
        self.draw_model_info(screen_w, screen_h);

        // Center column: Neural Network (full height)
        if !games.is_empty() {
            self.draw_neural_network(games[0], screen_w, screen_h);
        }

        // Right column: Stats + Charts
        let (steps_without_food, hunger_limit) = games
            .first()
            .map(|g| (g.core.steps_without_food, g.core.hunger_limit()))
            .unwrap_or((0, 100));
        self.draw_stats_panels(
            gen,
            max_score,
            gen_max,
            sim_time,
            current_score,
            fitness,
            steps,
            steps_without_food,
            hunger_limit,
            screen_w,
            screen_h,
        );
    }

    fn draw_game_grid(
        &self,
        games: &[&crate::game::Game],
        _screen_w: f32,
        screen_h: f32,
        theme: GameTheme,
    ) {
        let grid_size = screen_h - 340.0;
        let x = 20.0;
        let y = 20.0;
        let tile_size = grid_size / GRID_W as f32;

        // Border and backdrop
        draw_rectangle(
            x - 8.0,
            y - 8.0,
            grid_size + 16.0,
            grid_size + 16.0,
            PANEL_BORDER,
        );
        draw_rectangle(x, y, grid_size, grid_size, COLOR_BG);

        // Grid lines
        for i in 0..=GRID_W {
            let line_x = x + i as f32 * tile_size;
            draw_line(
                line_x,
                y,
                line_x,
                y + grid_size,
                1.0,
                Color::new(0.12, 0.12, 0.15, 1.0),
            );
        }
        for i in 0..=GRID_H {
            let line_y = y + i as f32 * tile_size;
            draw_line(
                x,
                line_y,
                x + grid_size,
                line_y,
                1.0,
                Color::new(0.12, 0.12, 0.15, 1.0),
            );
        }

        if games.is_empty() {
            return;
        }

        let colors = theme.colors();

        // Food
        let best_game = games[0];
        crate::render_snake::draw_apple(
            x + best_game.food.x as f32 * tile_size,
            y + best_game.food.y as f32 * tile_size,
            tile_size,
            theme,
            colors.food,
            best_game.core.food_freshness(),
        );

        // Draw all snakes (reverse order so best is on top)
        for (rank, game) in games.iter().enumerate().rev() {
            for (i, segment) in game.body.iter().enumerate() {
                let seg_x = x + segment.x as f32 * tile_size;
                let seg_y = y + segment.y as f32 * tile_size;
                if rank == 0 {
                    // Best snake uses active theme with continuous body / directional eyes / swallow bulges
                    let color = if i == 0 { colors.head } else { colors.body };
                    crate::render_snake::draw_connected_segment(
                        &game.body,
                        i,
                        game.dir,
                        seg_x,
                        seg_y,
                        tile_size,
                        theme,
                        color,
                        game.swallow.bulge_at(i),
                        game.swallow.head_scale(),
                    );
                } else {
                    // Population ghosts: gray with decreasing opacity
                    let alpha = 0.35 - (rank as f32 * 0.028);
                    let segment_color = if i == 0 {
                        Color::new(0.65, 0.70, 0.75, alpha)
                    } else {
                        Color::new(0.45, 0.50, 0.55, alpha * 0.7)
                    };
                    draw_rectangle(
                        seg_x + 1.0,
                        seg_y + 1.0,
                        tile_size - 2.0,
                        tile_size - 2.0,
                        segment_color,
                    );
                }
            }
        }
    }

    fn draw_neural_network(&self, game: &crate::game::Game, _screen_w: f32, screen_h: f32) {
        let left_col_width = screen_h - 320.0;
        let panel_w = 550.0;
        let panel_h = screen_h - 40.0;
        let panel_x = left_col_width + 40.0;
        let panel_y = 20.0;

        draw_terminal_box(panel_x, panel_y, panel_w, panel_h, "NEURAL NETWORK", false);

        let outputs = game.get_net_output();
        if outputs.is_empty() {
            return;
        }

        let layer_spacing = 160.0;

        // Input layer (12 nodes)
        let input_x = panel_x + 80.0;
        let input_start_y = panel_y + 80.0;
        let input_spacing = (panel_h - 160.0) / (INP_LAYER_SIZE as f32 - 1.0);

        // Hidden layer (8 nodes)
        let hidden_x = input_x + layer_spacing;
        let hidden_start_y = panel_y + 150.0;
        let hidden_spacing = (panel_h - 300.0) / (HIDDEN_LAYER_SIZE as f32 - 1.0);

        // Output layer (4 nodes)
        let output_x = hidden_x + layer_spacing;
        let output_start_y = panel_y + 250.0;
        let output_spacing = (panel_h - 500.0) / (OUTPUT_LAYER_SIZE as f32 - 1.0);
        let output_labels = ["LEFT", "RIGHT", "BOTTOM", "TOP"];

        // Synapse connections: Input -> Hidden (subtle lines)
        for i in 0..INP_LAYER_SIZE {
            for j in 0..HIDDEN_LAYER_SIZE {
                draw_line(
                    input_x,
                    input_start_y + i as f32 * input_spacing,
                    hidden_x,
                    hidden_start_y + j as f32 * hidden_spacing,
                    1.0,
                    Color::new(0.20, 0.22, 0.28, 0.15),
                );
            }
        }

        // Synapse connections: Hidden -> Output
        for i in 0..HIDDEN_LAYER_SIZE {
            for j in 0..OUTPUT_LAYER_SIZE {
                draw_line(
                    hidden_x,
                    hidden_start_y + i as f32 * hidden_spacing,
                    output_x,
                    output_start_y + j as f32 * output_spacing,
                    1.0,
                    Color::new(0.20, 0.22, 0.28, 0.15),
                );
            }
        }

        // Input nodes
        for i in 0..INP_LAYER_SIZE {
            let y = input_start_y + i as f32 * input_spacing;
            draw_circle(input_x, y, 7.0, ACCENT_CYAN);
            draw_text(&format!("I{}", i), input_x - 35.0, y + 6.0, 16.0, TEXT_COLOR);
        }

        // Hidden nodes
        for i in 0..HIDDEN_LAYER_SIZE {
            let y = hidden_start_y + i as f32 * hidden_spacing;
            draw_circle(hidden_x, y, 8.0, Color::new(0.9, 0.6, 0.0, 1.0));
            draw_text(&format!("H{}", i), hidden_x - 35.0, y + 6.0, 16.0, TEXT_COLOR);
        }

        // Output nodes with argmax detection
        let final_output = outputs.last().unwrap();
        let mut argmax = 0;
        let mut max_val = f64::NEG_INFINITY;
        for (idx, &val) in final_output.iter().enumerate() {
            if val > max_val {
                max_val = val;
                argmax = idx;
            }
        }

        for (i, &value) in final_output.iter().enumerate() {
            let y = output_start_y + i as f32 * output_spacing;
            let intensity = if value.is_finite() {
                (value as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let color = Color::new(intensity, intensity * 0.3, intensity * 0.9, 1.0);

            // Active argmax halo and border
            if i == argmax {
                draw_circle(output_x, y, 14.0, Color::new(0.0, 0.90, 0.90, 0.22));
                draw_circle_lines(output_x, y, 13.0, 2.0, WHITE);
            }

            draw_circle(output_x, y, 10.0, color);

            let label_color = if i == argmax {
                ACCENT_CYAN
            } else {
                TEXT_COLOR
            };
            draw_text(
                &format!("{} {:.3}", output_labels[i], value),
                output_x + 22.0,
                y + 7.0,
                18.0,
                label_color,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_stats_panels(
        &self,
        gen: usize,
        max_score: usize,
        gen_max: usize,
        sim_time: f32,
        current_score: usize,
        fitness: f32,
        steps: usize,
        steps_without_food: usize,
        hunger_limit: usize,
        screen_w: f32,
        screen_h: f32,
    ) {
        let left_col_width = screen_h - 320.0;
        let nn_width = 550.0;
        let panel_x = left_col_width + nn_width + 60.0;
        let panel_w = (screen_w - panel_x - 20.0).max(0.0);
        if panel_w <= 10.0 {
            return;
        }

        let stats_h = 160.0;
        let run_h = 135.0;
        let bar_h = 55.0;
        let fixed_total = 20.0 + stats_h + 15.0 + run_h + 15.0 + bar_h + 10.0 + bar_h + 20.0;
        let remaining_h = (screen_h - fixed_total - 20.0).max(120.0);
        let chart_h = (remaining_h / 2.0) - 10.0;

        // 1. SIM STATS
        let mut y = 20.0;
        draw_terminal_box(panel_x, y, panel_w, stats_h, "SIM STATS", false);
        if max_score > 0 {
            draw_badge("GUARDADO", panel_x + panel_w - 95.0, y + 13.0, ACCENT_GREEN);
        }
        let rows = [
            ("Generación:", format!("{}", gen)),
            ("Récord Histórico:", format!("{}", max_score)),
            ("Máx Generación:", format!("{}", gen_max)),
            ("Tiempo Simulación:", format!("{:.2}s", sim_time)),
        ];
        let mut row_y = y + 50.0;
        for (lbl, val) in rows {
            draw_text(lbl, panel_x + 18.0, row_y, TEXT_SIZE, TEXT_MUTED);
            let val_dims = measure_text(&val, None, TEXT_SIZE as u16, 1.0);
            draw_text(
                &val,
                panel_x + panel_w - val_dims.width - 18.0,
                row_y,
                TEXT_SIZE,
                TEXT_COLOR,
            );
            row_y += 26.0;
        }

        // 2. VIZ STATS
        y += stats_h + 15.0;
        draw_terminal_box(panel_x, y, panel_w, run_h, "VIZ STATS", false);
        let run_rows = [
            ("Puntuación:", format!("{}", current_score)),
            ("Fitness:", format!("{:.1}", fitness)),
            ("Pasos:", format!("{}", steps)),
            ("Sin Comer:", format!("{}/{}", steps_without_food, hunger_limit)),
        ];
        let mut run_row_y = y + 42.0;
        for (lbl, val) in run_rows {
            draw_text(lbl, panel_x + 18.0, run_row_y, TEXT_SIZE, TEXT_MUTED);
            let val_dims = measure_text(&val, None, TEXT_SIZE as u16, 1.0);
            draw_text(
                &val,
                panel_x + panel_w - val_dims.width - 18.0,
                run_row_y,
                TEXT_SIZE,
                TEXT_COLOR,
            );
            run_row_y += 22.0;
        }

        // 3. VIZ SCORE Bar (normalized against session record or base target 20)
        y += run_h + 15.0;
        let target = (max_score.max(20)) as f32;
        let score_fraction = (current_score as f32 / target).clamp(0.0, 1.0);
        let score_label = format!("Score: {}", current_score);
        draw_progress_bar(
            panel_x,
            y,
            panel_w,
            bar_h,
            score_fraction,
            &score_label,
            ACCENT_CYAN,
        );

        // 4. MAX SCORE Bar
        y += bar_h + 10.0;
        let max_fraction = if max_score > 0 {
            (max_score as f32 / target).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let max_label = format!("Récord: {}", max_score);
        draw_progress_bar(
            panel_x,
            y,
            panel_w,
            bar_h,
            max_fraction,
            &max_label,
            ACCENT_GOLD,
        );

        // 5. GEN TIMES Chart
        y += bar_h + 20.0;
        let times_max = self
            .gen_times
            .iter()
            .fold(0.0f32, |acc, &v| acc.max(v));
        let times_label = if times_max > 0.0 {
            format!("(MAX: {:.0})", times_max)
        } else {
            String::new()
        };
        draw_responsive_chart(
            panel_x,
            y,
            panel_w,
            chart_h,
            "GEN TIMES",
            &times_label,
            &self.gen_times,
            self.max_history_size,
            SKYBLUE,
        );

        // 6. GEN SCORES Chart
        y += chart_h + 10.0;
        let scores_max = self
            .gen_scores_f32
            .iter()
            .fold(0.0f32, |acc, &v| acc.max(v));
        let scores_label = if scores_max > 0.0 {
            format!("(MAX: {:.0})", scores_max)
        } else {
            String::new()
        };
        draw_responsive_chart(
            panel_x,
            y,
            panel_w,
            chart_h,
            "GEN SCORES",
            &scores_label,
            &self.gen_scores_f32,
            self.max_history_size,
            ACCENT_GREEN,
        );
    }

    fn draw_model_info(&self, _screen_w: f32, screen_h: f32) {
        let panel_x = 20.0;
        let panel_y = screen_h - 300.0;
        let panel_w = screen_h - 320.0;
        let panel_h = 280.0;

        draw_terminal_box(panel_x, panel_y, panel_w, panel_h, "ALGORITMO GENÉTICO", false);

        let rows = [
            ("Población Agentes:", format!("{}", *NUM_GAMES_PER_STREAM)),
            ("Límite Hambre:", "100-800 (Dinámico)".to_string()),
            (
                "Tasa Mutación:",
                format!("{:.1}%", *BRAIN_MUTATION_RATE * 100.0),
            ),
            (
                "Arquitectura:",
                format!(
                    "{}x{}x{}",
                    INP_LAYER_SIZE, HIDDEN_LAYER_SIZE, OUTPUT_LAYER_SIZE
                ),
            ),
            ("Streams Simulación:", format!("{}", *NUM_STREAMS)),
        ];

        let mut y = panel_y + 48.0;
        for (lbl, val) in rows {
            draw_text(lbl, panel_x + 18.0, y, 16.0, TEXT_MUTED);
            let val_dims = measure_text(&val, None, 16, 1.0);
            draw_text(
                &val,
                panel_x + panel_w - val_dims.width - 18.0,
                y,
                16.0,
                TEXT_COLOR,
            );
            y += 24.0;
        }

        // Controls footer with retro badges
        let ctrl_y = panel_y + panel_h - 36.0;
        draw_text("Controles:", panel_x + 18.0, ctrl_y + 14.0, 15.0, TEXT_MUTED);

        let shortcuts = [
            ("TAB", "HUD"),
            ("SPACE", "Vel"),
            ("V", "VS"),
            ("ESC", "Menú"),
        ];
        let mut badge_x = panel_x + 95.0;
        for (key, act) in shortcuts {
            let badge_w = measure_text(key, None, 13, 1.0).width + 12.0;
            draw_badge(key, badge_x, ctrl_y, ACCENT_CYAN);
            badge_x += badge_w + 4.0;
            draw_text(act, badge_x, ctrl_y + 14.0, 14.0, TEXT_COLOR);
            badge_x += measure_text(act, None, 14, 1.0).width + 10.0;
        }
    }
}

// ---------------------------------------------------------------------------
// Free-standing rendering entry point (Fase 4)
// ---------------------------------------------------------------------------

/// Render the GA advanced training dashboard from a read-only [`SimSnapshot`].
///
/// Replaces `Simulation::draw_advanced` (which required macroquad inside the
/// domain module). The caller (`GaTrainView`) owns `viz` and passes it here.
pub fn draw_sim(
    snap: &crate::sim::SimSnapshot<'_>,
    viz: &VizAdvanced,
    theme: crate::theme::GameTheme,
) {
    clear_background(crate::ui_kit::COLOR_BG);
    if !snap.top_games.is_empty() {
        viz.draw(
            &snap.top_games,
            snap.gen_count,
            snap.best_ever,
            snap.gen_max,
            snap.elapsed_secs,
            snap.champ_score,
            snap.champ_fitness,
            snap.champ_steps,
            theme,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_round_trip_and_cap() {
        let mut viz = VizAdvanced::new();
        let times: Vec<f32> = (0..60).map(|i| i as f32).collect();
        let scores: Vec<usize> = (0..60).collect();
        viz.set_history(times, scores);

        let (out_times, out_scores) = viz.get_history();
        assert_eq!(out_times.len(), 50);
        assert_eq!(out_scores.len(), 50);
        assert_eq!(out_times.first(), Some(&10.0));
        assert_eq!(out_scores.first(), Some(&10));
        assert_eq!(out_times.last(), Some(&59.0));
        assert_eq!(out_scores.last(), Some(&59));
    }

    #[test]
    fn update_generation_pushes_and_caps() {
        let mut viz = VizAdvanced::new();
        for i in 0..55 {
            viz.update_generation(i as f32, i * 2);
        }
        let (times, scores) = viz.get_history();
        assert_eq!(times.len(), 50);
        assert_eq!(scores.len(), 50);
        assert_eq!(times[0], 5.0);
        assert_eq!(scores[0], 10);
    }
}

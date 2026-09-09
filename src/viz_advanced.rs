//! Advanced visualization dashboard for Snake AI training

use macroquad::prelude::*;
use crate::*;

const PANEL_BG: Color = Color::new(0.05, 0.05, 0.05, 0.95);
const PANEL_BORDER: Color = Color::new(0.4, 0.4, 0.4, 1.0);
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const ACCENT_COLOR: Color = Color::new(0.0, 0.9, 0.9, 1.0);
const TITLE_SIZE: f32 = 22.0;
const TEXT_SIZE: f32 = 18.0;

pub struct VizAdvanced {
    gen_times: Vec<f32>,
    gen_scores: Vec<usize>,
    max_history_size: usize,
}

impl VizAdvanced {
    pub fn new() -> Self {
        Self {
            gen_times: Vec::new(),
            gen_scores: Vec::new(),
            max_history_size: 50,
        }
    }

    pub fn update_generation(&mut self, time: f32, score: usize) {
        self.gen_times.push(time);
        self.gen_scores.push(score);
        
        if self.gen_times.len() > self.max_history_size {
            self.gen_times.remove(0);
        }
        if self.gen_scores.len() > self.max_history_size {
            self.gen_scores.remove(0);
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
        self.gen_times = times;
        self.gen_scores = scores;
    }

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
    ) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // Left column: Grid + Model Info
        self.draw_game_grid(games, screen_w, screen_h);
        self.draw_model_info(screen_w, screen_h);

        // Center column: Neural Network (full height)
        if !games.is_empty() {
            self.draw_neural_network(games[0], screen_w, screen_h);
        }

        // Right column: Stats + Charts
        self.draw_stats_panels(
            gen,
            max_score,
            gen_max,
            sim_time,
            current_score,
            fitness,
            steps,
            screen_w,
            screen_h,
        );
    }

    fn draw_game_grid(&self, games: &[&crate::game::Game], _screen_w: f32, screen_h: f32) {
        let grid_size = screen_h - 340.0; // Dynamic size based on screen height
        let x = 20.0;
        let y = 20.0;
        let tile_size = grid_size / GRID_W as f32;

        // Border
        draw_rectangle(x - 8.0, y - 8.0, grid_size + 16.0, grid_size + 16.0, PANEL_BORDER);
        draw_rectangle(x, y, grid_size, grid_size, BLACK);

        // Grid lines
        for i in 0..=GRID_W {
            let line_x = x + i as f32 * tile_size;
            draw_line(line_x, y, line_x, y + grid_size, 1.0, Color::new(0.15, 0.15, 0.15, 1.0));
        }
        for i in 0..=GRID_H {
            let line_y = y + i as f32 * tile_size;
            draw_line(x, line_y, x + grid_size, line_y, 1.0, Color::new(0.15, 0.15, 0.15, 1.0));
        }

        if games.is_empty() {
            return;
        }

        let theme = crate::theme::load_theme();
        let colors = theme.colors();

        // Food
        let best_game = games[0];
        crate::render_snake::draw_apple(
            x + best_game.food.x as f32 * tile_size,
            y + best_game.food.y as f32 * tile_size,
            tile_size,
            theme,
            colors.food,
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
                    // Others: gray ghosts with decreasing opacity
                    let alpha = 0.4 - (rank as f32 * 0.03);
                    let segment_color = if i == 0 {
                        Color::new(0.7, 0.7, 0.7, alpha) // Ghost head
                    } else {
                        Color::new(0.5, 0.5, 0.5, alpha * 0.7) // Ghost body
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

        draw_rectangle(panel_x, panel_y, panel_w, panel_h, PANEL_BG);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 3.0, PANEL_BORDER);
        draw_text("NEURAL NETWORK", panel_x + 20.0, panel_y + 28.0, TITLE_SIZE, ACCENT_COLOR);

        let outputs = game.get_net_output();
        if outputs.is_empty() {
            return;
        }

        // Vertical layout - layers from left to right
        let layer_spacing = 160.0;
        
        // Input layer (12 nodes) - leftmost
        let input_x = panel_x + 80.0;
        let input_start_y = panel_y + 80.0;
        let input_spacing = (panel_h - 160.0) / (INP_LAYER_SIZE as f32 - 1.0);

        // Hidden layer (8 nodes) - middle
        let hidden_x = input_x + layer_spacing;
        let hidden_start_y = panel_y + 150.0;
        let hidden_spacing = (panel_h - 300.0) / (HIDDEN_LAYER_SIZE as f32 - 1.0);
        
        // Output layer (4 nodes) - rightmost
        let output_x = hidden_x + layer_spacing;
        let output_start_y = panel_y + 250.0;
        let output_spacing = (panel_h - 500.0) / (OUTPUT_LAYER_SIZE as f32 - 1.0);
        let output_labels = ["LEFT", "RIGHT", "BOTTOM", "TOP"];

        // Draw ALL connections: Input -> Hidden
        for i in 0..INP_LAYER_SIZE {
            for j in 0..HIDDEN_LAYER_SIZE {
                draw_line(
                    input_x,
                    input_start_y + i as f32 * input_spacing,
                    hidden_x,
                    hidden_start_y + j as f32 * hidden_spacing,
                    1.5,
                    Color::new(0.3, 0.3, 0.3, 0.2),
                );
            }
        }

        // Draw ALL connections: Hidden -> Output
        for i in 0..HIDDEN_LAYER_SIZE {
            for j in 0..OUTPUT_LAYER_SIZE {
                draw_line(
                    hidden_x,
                    hidden_start_y + i as f32 * hidden_spacing,
                    output_x,
                    output_start_y + j as f32 * output_spacing,
                    1.5,
                    Color::new(0.3, 0.3, 0.3, 0.2),
                );
            }
        }

        // Draw input nodes
        for i in 0..INP_LAYER_SIZE {
            let y = input_start_y + i as f32 * input_spacing;
            draw_circle(input_x, y, 7.0, ACCENT_COLOR);
            draw_text(&format!("I{}", i), input_x - 35.0, y + 6.0, 18.0, TEXT_COLOR);
        }

        // Draw hidden nodes
        for i in 0..HIDDEN_LAYER_SIZE {
            let y = hidden_start_y + i as f32 * hidden_spacing;
            draw_circle(hidden_x, y, 8.0, Color::new(0.9, 0.6, 0.0, 1.0));
            draw_text(&format!("H{}", i), hidden_x - 35.0, y + 6.0, 18.0, TEXT_COLOR);
        }

        // Draw output nodes with activation
        let final_output = outputs.last().unwrap();
        for (i, &value) in final_output.iter().enumerate() {
            let y = output_start_y + i as f32 * output_spacing;
            let intensity = (value as f32).clamp(0.0, 1.0);
            let color = Color::new(intensity, intensity * 0.3, intensity * 0.9, 1.0);
            draw_circle(output_x, y, 10.0, color);
            draw_text(output_labels[i], output_x + 22.0, y + 7.0, 20.0, TEXT_COLOR);
        }
    }

    fn draw_stats_panels(
        &self,
        gen: usize,
        max_score: usize,
        gen_max: usize,
        sim_time: f32,
        current_score: usize,
        fitness: f32,
        steps: usize,
        screen_w: f32,
        screen_h: f32,
    ) {
        let left_col_width = screen_h - 320.0;
        let nn_width = 550.0;
        let panel_x = left_col_width + nn_width + 60.0;
        let panel_w = screen_w - panel_x - 20.0;
        let mut y = 20.0;

        // SIM STATS
        self.draw_panel(panel_x, y, panel_w, 150.0, "SIM STATS");
        y += 45.0;
        draw_text(&format!("Gen: {}", gen), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 30.0;
        draw_text(&format!("Max: {}", max_score), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 30.0;
        draw_text(&format!("Gen Max: {}", gen_max), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 30.0;
        draw_text(&format!("Sim Ts: {:.2} secs", sim_time), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);

        // VIZ STATS
        y += 45.0;
        self.draw_panel(panel_x, y, panel_w, 130.0, "VIZ STATS");
        y += 45.0;
        draw_text(&format!("Score: {}", current_score), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 30.0;
        draw_text(&format!("Fitness: {:.2}", fitness), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 30.0;
        draw_text(&format!("Steps: {}", steps), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);

        // VIZ SCORE bar
        y += 45.0;
        self.draw_panel(panel_x, y, panel_w, 100.0, "VIZ SCORE");
        y += 50.0;
        let score_pct = (current_score as f32 / 20.0).min(1.0);
        draw_rectangle(panel_x + 20.0, y, (panel_w - 40.0) * score_pct, 30.0, MAGENTA);
        draw_rectangle_lines(panel_x + 20.0, y, panel_w - 40.0, 30.0, 2.0, PANEL_BORDER);
        draw_text(&format!("{}%", (score_pct * 100.0) as i32), panel_x + panel_w / 2.0 - 20.0, y + 21.0, 18.0, WHITE);

        // MAX SCORE bar
        y += 70.0;
        self.draw_panel(panel_x, y, panel_w, 100.0, "MAX SCORE");
        y += 50.0;
        let max_pct = (max_score as f32 / 20.0).min(1.0);
        draw_rectangle(panel_x + 20.0, y, (panel_w - 40.0) * max_pct, 30.0, RED);
        draw_rectangle_lines(panel_x + 20.0, y, panel_w - 40.0, 30.0, 2.0, PANEL_BORDER);
        draw_text(&format!("{}%", (max_pct * 100.0) as i32), panel_x + panel_w / 2.0 - 20.0, y + 21.0, 18.0, WHITE);

        // GEN TIMES chart
        y += 80.0;
        let chart_h = (screen_h - y - 20.0) / 2.0 - 10.0;
        self.draw_chart(panel_x, y, panel_w, chart_h, "GEN TIMES", &self.gen_times, SKYBLUE);

        // GEN SCORES chart
        y += chart_h + 20.0;
        let scores_f32: Vec<f32> = self.gen_scores.iter().map(|&s| s as f32).collect();
        self.draw_chart(panel_x, y, panel_w, chart_h, "GEN SCORES", &scores_f32, GREEN);
    }

    fn draw_panel(&self, x: f32, y: f32, w: f32, h: f32, title: &str) {
        draw_rectangle(x, y, w, h, PANEL_BG);
        draw_rectangle_lines(x, y, w, h, 3.0, PANEL_BORDER);
        draw_text(title, x + 20.0, y + 30.0, TITLE_SIZE, ACCENT_COLOR);
    }

    fn draw_chart(&self, x: f32, y: f32, w: f32, h: f32, title: &str, data: &[f32], color: Color) {
        draw_rectangle(x, y, w, h, PANEL_BG);
        draw_rectangle_lines(x, y, w, h, 3.0, PANEL_BORDER);
        let max_val = if !data.is_empty() {
            data.iter().fold(0.0f32, |a, &b| a.max(b))
        } else {
            0.0
        };
        let chart_title = if max_val > 0.0 {
            format!("{} (MAX: {:.0})", title, max_val)
        } else {
            title.to_string()
        };
        draw_text(&chart_title, x + 20.0, y + 30.0, TITLE_SIZE, ACCENT_COLOR);

        let count_text = format!("{}/{} GENS", data.len(), self.max_history_size);
        draw_text(&count_text, x + w - 120.0, y + 28.0, 16.0, TEXT_COLOR);
        
        if data.is_empty() {
            return;
        }

        let chart_x = x + 20.0;
        let chart_y = y + 50.0;
        let chart_w = w - 40.0;
        let chart_h = h - 70.0;

        let scale = max_val.max(1.0);
        let step = chart_w / (self.max_history_size as f32).max(1.0);
        let bar_w = (step - 1.0).max(2.0);

        for (i, &val) in data.iter().enumerate() {
            let bar_h = (val / scale) * chart_h;
            let bar_x = chart_x + i as f32 * step;
            let bar_y = chart_y + chart_h - bar_h;
            draw_rectangle(bar_x, bar_y, bar_w, bar_h, color);
        }
    }

    fn draw_model_info(&self, _screen_w: f32, screen_h: f32) {
        let panel_x = 20.0;
        let panel_y = screen_h - 300.0;
        let panel_w = screen_h - 320.0;
        let panel_h = 280.0;

        self.draw_panel(panel_x, panel_y, panel_w, panel_h, "SNAKE AI");
        
        let mut y = panel_y + 55.0;
        draw_text(&format!("Agents: {}", NUM_GAMES_PER_STREAM), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 35.0;
        draw_text(&format!("Step Limit: {}", NUM_SIM_STEPS), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 35.0;
        draw_text(&format!("Mutation Rate: {}", BRAIN_MUTATION_RATE), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 35.0;
        draw_text(&format!("Architecture: {}x{}x{}", INP_LAYER_SIZE, HIDDEN_LAYER_SIZE, OUTPUT_LAYER_SIZE), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 35.0;
        draw_text(&format!("Streams: {}", NUM_STREAMS), panel_x + 20.0, y, TEXT_SIZE, TEXT_COLOR);
        y += 45.0;
        draw_text("Controls: [TAB] Viz  [SPACE] Slow  [V] VS  [ESC] Quit", panel_x + 20.0, y, 18.0, ACCENT_COLOR);
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

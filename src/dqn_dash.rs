//! DQN training dashboard — the advanced-dashboard mirror for DQN train.
//!
//! Mirrors `VizAdvanced`'s dashboard grammar (the user-verified GA reference:
//! left column = live snake grid over a bottom model-info panel, center =
//! full-height "NEURAL NETWORK" panel, right column = stacked stat panels,
//! score bars, and history bar charts), fed by live DQN data instead of the GA
//! population. `viz_advanced.rs` itself is under a zero-diff guard and is never
//! touched or shared with: every geometry/color constant is copied verbatim into
//! this module (design D-2), so the whole mirror is one self-contained diff a
//! reviewer can check line by line against the reference.
//!
//! Pure, macroquad-free pieces (design D-7 / §3):
//! - [`EpisodeHistory`] — the bounded per-episode ring (cap
//!   [`EPISODE_HISTORY_CAP`] = 50, mirroring `VizAdvanced`'s cap) feeding the
//!   "EPISODE TIMES"/"EPISODE SCORES" charts. `times` holds each completed
//!   episode's duration as its step count at completion (recorded pre-reset by
//!   `DqnTrainView::end_episode`), so chart 1 is deterministic.
//! - [`output_intensity`] — clamps the sigmoid q-network output (already in
//!   (0,1)) to the [0,1] domain the output-node color mapping consumes
//!   (`draw_neural_network` colors each of the LEFT/RIGHT/BOTTOM/TOP nodes).
//! - [`score_bar_fraction`] — the SCORE/BEST bars' fill fraction over the
//!   documented DQN episode bound `FULL_BAR = (NUM_SIM_STEPS * 2) as f32`
//!   (200.0 today; `score <= steps <= 200` in `GameDQN`), clamped to [0,1].
//!   The GA reference's hardcoded `/20` denominator is NOT copied.
//! - `ema_update`/`argmax_index`/`loss_ema`/`target_updates` viven en
//!   `crate::dqn` (testeadas); este módulo solo las consume.
//!
//! Drawing entry: [`draw`] borrows only `&GameDQN`, the view's two scalars
//! (`episode`, `best_score`) and `&EpisodeHistory` — no `DqnTrainView` crosses
//! into this module (design D-3/D-4).

use crate::configs::{GRID_H, GRID_W};
use crate::dqn::{
    argmax_index, BATCH_SIZE, DQN_HIDDEN_LAYER_SIZE, DQN_INP_LAYER_SIZE, DQN_OUTPUT_LAYER_SIZE,
    DQN_STEP_LIMIT, EPSILON_DECAY, EPSILON_END, EPSILON_START, GAMMA, LEARNING_RATE,
    REPLAY_BUFFER_SIZE,
};
use crate::game_dqn::GameDQN;
use crate::ui_kit::{
    draw_badge, draw_progress_bar, draw_responsive_chart, draw_terminal_box, ACCENT_CYAN,
    ACCENT_GOLD, ACCENT_GREEN, COLOR_BG, PANEL_BORDER, TEXT_MUTED,
};
use macroquad::prelude::*;

/// Max entries kept by [`EpisodeHistory`], mirroring `VizAdvanced`'s 50-entry
/// cap (`viz_advanced.rs`, read-only).
pub const EPISODE_HISTORY_CAP: usize = 50;

/// Bounded per-episode history ring feeding the dashboard's charts (design
/// D-7). Pure and macroquad-free.
///
/// Invariant: `times.len() == scores.len()` at all times — a `push` appends to
/// both vectors and eviction `remove(0)` drops the *oldest pair* from both.
///
/// `times` holds each completed episode's duration as its step count at
/// completion (recorded pre-reset in `DqnTrainView::end_episode`, per the spec
/// — not a wall-clock `Instant`), so chart 1 ("EPISODE TIMES") is deterministic
/// and unit-testable.
pub struct EpisodeHistory {
    /// Episode durations (step counts at completion), oldest first.
    pub times: Vec<f32>,
    /// Episode final scores, oldest first (index-aligned with `times`).
    pub scores: Vec<usize>,
    /// Pre-converted f32 episode final scores for allocation-free chart rendering.
    pub scores_f32: Vec<f32>,
    cap: usize,
}

impl EpisodeHistory {
    /// An empty ring capped at [`EPISODE_HISTORY_CAP`] entries.
    pub fn new() -> Self {
        Self {
            times: Vec::new(),
            scores: Vec::new(),
            scores_f32: Vec::new(),
            cap: EPISODE_HISTORY_CAP,
        }
    }

    /// Append one `(time, score)` pair; overflow evicts the oldest pair so the
    /// ring holds at most `cap` entries with the newest last.
    pub fn push(&mut self, time: f32, score: usize) {
        debug_assert!(
            self.times.len() == self.scores.len() && self.scores.len() == self.scores_f32.len(),
            "EpisodeHistory invariant broken: times/scores out of alignment"
        );
        self.times.push(time);
        self.scores.push(score);
        self.scores_f32.push(score as f32);
        while self.len() > self.cap {
            self.times.remove(0);
            self.scores.remove(0);
            self.scores_f32.remove(0);
        }
    }

    /// Empty both vectors (fresh-agent session reset).
    pub fn clear(&mut self) {
        self.times.clear();
        self.scores.clear();
        self.scores_f32.clear();
    }

    /// Number of recorded episodes (`scores.len() == times.len()`).
    pub fn len(&self) -> usize {
        self.scores.len()
    }
}

/// Pure color-driving clamp (design seam 5): sigmoid q-network outputs already
/// live in (0,1), but the clamp pins the domain the output-node color mapping
/// consumes — below 0 → 0.0, above 1 → 1.0, interior values unchanged.
/// Consumed by [`crate::dqn_dash`]'s `draw_neural_network` to color each of
/// the four LEFT/RIGHT/BOTTOM/TOP output nodes.
pub fn output_intensity(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

/// Pure SCORE/BEST bar fraction over the documented DQN episode bound (design
/// §3): `FULL_BAR = DQN_STEP_LIMIT as f32` = 500.0 — `GameDQN::step`
/// forces `done` at `steps >= DQN_STEP_LIMIT` and every food eaten consumes a
/// step, so `score <= steps <= 500`. Clamped to [0,1]; the GA reference's `/20`
/// denominator is never used on the DQN dashboard. Consumed by
/// [`crate::dqn_dash`]'s `draw_stats_panels` to size the "SCORE" and "BEST"
/// bars.
fn score_bar_fraction(score: usize) -> f32 {
    let full_bar = DQN_STEP_LIMIT as f32;
    (score as f32 / full_bar).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Retro-Terminal UI Helpers
// ---------------------------------------------------------------------------

/// Helper to draw a key-value metric row with right-aligned value for retro-terminal panels.
fn draw_stat_row(x: f32, y: f32, w: f32, label: &str, value: &str) {
    draw_text(label, x, y, 14.0, TEXT_MUTED);
    let dims = measure_text(value, None, 14, 1.0);
    draw_text(value, x + w - dims.width, y, 14.0, WHITE);
}

/// Draw a formatted shortcut key badge with its description text.
fn draw_key_badge(x: f32, y: f32, key: &str, desc: &str) -> f32 {
    let font_size = 13.0;
    let dims = measure_text(key, None, font_size as u16, 1.0);
    let pad_x = 7.0;
    let badge_w = dims.width + pad_x * 2.0;
    draw_badge(key, x, y, ACCENT_CYAN);
    let desc_x = x + badge_w + 6.0;
    draw_text(desc, desc_x, y + 14.5, font_size, TEXT_MUTED);
    let desc_dims = measure_text(desc, None, font_size as u16, 1.0);
    badge_w + 6.0 + desc_dims.width
}

/// Left column: the live `GameDQN` grid — reference `draw_game_grid` geometry
/// verbatim (`x=20, y=20, grid_size = screen_h − 340`, border −8/+16, black
/// field, 0.15 grid lines, WHITE food with a `tile − 4` inset). Only the one
/// live snake is drawn (head `(0.3,0.9,0.3)`, body `(0.2,0.7,0.2)`, the
/// reference's best-snake style) — DQN has no top-N population, so there is NO
/// ghost/rank loop (proposal success-criterion divergence).
fn draw_grid(game: &GameDQN, screen_h: f32, theme: crate::theme::GameTheme) {
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

    let colors = theme.colors();

    // Food (single live game)
    crate::render_snake::draw_apple(
        x + game.food.x as f32 * tile_size,
        y + game.food.y as f32 * tile_size,
        tile_size,
        theme,
        colors.food,
    );

    // The one live snake
    for (i, segment) in game.body.iter().enumerate() {
        let seg_x = x + segment.x as f32 * tile_size;
        let seg_y = y + segment.y as f32 * tile_size;
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
    }
}

/// Center column: full-height "NEURAL NETWORK" panel — retro-terminal layout
/// with subtle synapse lines, input and hidden node readouts, active output node
/// glow and border for the argmax action, and columnar label/value alignment.
fn draw_neural_network(game: &GameDQN, screen_h: f32) {
    let left_col_width = screen_h - 320.0;
    let panel_w = 550.0;
    let panel_h = screen_h - 40.0;
    let panel_x = left_col_width + 40.0;
    let panel_y = 20.0;

    draw_terminal_box(panel_x, panel_y, panel_w, panel_h, "NEURAL NETWORK", false);

    let outputs = game.agent.q_network.predict_linear_output(&game.observation());
    if outputs.is_empty() {
        return;
    }

    // Vertical layout - layers from left to right
    let layer_spacing = 145.0;

    // Input layer (9 nodes) - leftmost
    let input_x = panel_x + 60.0;
    let input_start_y = panel_y + 80.0;
    let input_spacing = (panel_h - 160.0) / (DQN_INP_LAYER_SIZE as f32 - 1.0);

    // Hidden layer (32 nodes) - middle
    let hidden_x = input_x + layer_spacing;
    let hidden_start_y = panel_y + 150.0;
    let hidden_spacing = (panel_h - 300.0) / (DQN_HIDDEN_LAYER_SIZE as f32 - 1.0);

    // Output layer (3 nodes) - rightmost
    let output_x = hidden_x + layer_spacing;
    let output_start_y = panel_y + 250.0;
    let output_spacing = (panel_h - 500.0) / (DQN_OUTPUT_LAYER_SIZE as f32 - 1.0);
    let output_labels = ["STRAIGHT", "TURN LEFT", "TURN RIGHT"];

    // Subtle synapse lines (inactive connections)
    let synapse_color = Color::new(0.2, 0.2, 0.25, 0.15);

    // Draw ALL connections: Input -> Hidden
    for i in 0..DQN_INP_LAYER_SIZE {
        let iy = input_start_y + i as f32 * input_spacing;
        for j in 0..DQN_HIDDEN_LAYER_SIZE {
            let hy = hidden_start_y + j as f32 * hidden_spacing;
            draw_line(input_x, iy, hidden_x, hy, 1.0, synapse_color);
        }
    }

    // Draw ALL connections: Hidden -> Output
    for i in 0..DQN_HIDDEN_LAYER_SIZE {
        let hy = hidden_start_y + i as f32 * hidden_spacing;
        for j in 0..DQN_OUTPUT_LAYER_SIZE {
            let oy = output_start_y + j as f32 * output_spacing;
            draw_line(hidden_x, hy, output_x, oy, 1.0, synapse_color);
        }
    }

    // Draw input nodes
    for i in 0..DQN_INP_LAYER_SIZE {
        let y = input_start_y + i as f32 * input_spacing;
        draw_circle(input_x, y, 7.0, ACCENT_CYAN);
        draw_text(&format!("I{}", i), input_x - 30.0, y + 5.0, 15.0, TEXT_MUTED);
    }

    // Draw hidden nodes
    for i in 0..DQN_HIDDEN_LAYER_SIZE {
        let y = hidden_start_y + i as f32 * hidden_spacing;
        draw_circle(hidden_x, y, 6.5, Color::new(0.9, 0.6, 0.0, 1.0));
        draw_text(&format!("H{}", i), hidden_x - 32.0, y + 5.0, 14.0, TEXT_MUTED);
    }

    // Draw output nodes with activation: continuous Q-value readout per action
    // plus active node glow, ring, and neon cyan action text on the argmax.
    let final_output = outputs.last().unwrap();
    let argmax = argmax_index(final_output);
    for (i, &value) in final_output.iter().enumerate() {
        let y = output_start_y + i as f32 * output_spacing;
        let intensity = output_intensity(value) as f32;
        let color = Color::new(intensity, intensity * 0.3, intensity * 0.9, 1.0);

        if i == argmax {
            // Active output node: neon cyan glow and border ring
            draw_circle(output_x, y, 16.0, Color::new(0.0, 0.90, 0.90, 0.22));
            draw_circle(output_x, y, 10.0, color);
            draw_circle_lines(output_x, y, 13.0, 2.0, ACCENT_CYAN);
        } else {
            draw_circle(output_x, y, 10.0, color);
            draw_circle_lines(output_x, y, 12.0, 1.0, Color::new(0.3, 0.35, 0.4, 0.6));
        }

        let label = output_labels[i];
        let val_str = format!("{:.3}", value);
        let (label_color, val_color) = if i == argmax {
            (ACCENT_CYAN, ACCENT_CYAN)
        } else {
            (TEXT_MUTED, Color::new(0.85, 0.85, 0.85, 1.0))
        };

        // Clean columnar alignment: action name followed by Q-value
        draw_text(label, output_x + 22.0, y + 6.0, 16.0, label_color);
        draw_text(&val_str, output_x + 130.0, y + 6.0, 16.0, val_color);
    }
}

/// Bottom-left model-info panel — retro-terminal panel displaying
/// hyperparameters and structured keyboard shortcuts with badge frames.
fn draw_model_info(_game: &GameDQN, screen_h: f32) {
    let panel_x = 20.0;
    let panel_y = screen_h - 300.0;
    let panel_w = screen_h - 320.0;
    let panel_h = 280.0;

    draw_terminal_box(panel_x, panel_y, panel_w, panel_h, "DQN TRAIN", false);

    let mut y = panel_y + 48.0;
    draw_stat_row(
        panel_x + 16.0,
        y,
        panel_w - 32.0,
        "Architecture:",
        &format!(
            "{}x{}x{}",
            DQN_INP_LAYER_SIZE, DQN_HIDDEN_LAYER_SIZE, DQN_OUTPUT_LAYER_SIZE
        ),
    );
    y += 24.0;
    draw_stat_row(
        panel_x + 16.0,
        y,
        panel_w - 32.0,
        "Batch / Gamma:",
        &format!("{} / {:.2}", BATCH_SIZE, GAMMA),
    );
    y += 24.0;
    draw_stat_row(
        panel_x + 16.0,
        y,
        panel_w - 32.0,
        "Learning Rate:",
        &format!("{}", LEARNING_RATE),
    );
    y += 24.0;
    draw_stat_row(
        panel_x + 16.0,
        y,
        panel_w - 32.0,
        "Epsilon Schedule:",
        &format!("{:.2} -> {:.2} (x{:.3})", EPSILON_START, EPSILON_END, EPSILON_DECAY),
    );
    y += 24.0;
    draw_stat_row(
        panel_x + 16.0,
        y,
        panel_w - 32.0,
        "Step Limit:",
        &format!("{}", DQN_STEP_LIMIT),
    );

    // Separator before controls
    draw_line(
        panel_x + 16.0,
        panel_y + 180.0,
        panel_x + panel_w - 16.0,
        panel_y + 180.0,
        1.0,
        PANEL_BORDER,
    );

    // Formatted keyboard shortcuts with badge frames
    draw_text("CONTROLS", panel_x + 16.0, panel_y + 204.0, 13.0, ACCENT_GOLD);

    let shortcuts = [
        ("TAB", "HUD"),
        ("R", "Nuevo Agente"),
        ("ESC", "Menú"),
    ];
    let mut cur_x = panel_x + 16.0;
    let mut cur_y = panel_y + 218.0;
    for (key, desc) in &shortcuts {
        let item_w = {
            let k_dims = measure_text(key, None, 13, 1.0);
            let d_dims = measure_text(desc, None, 13, 1.0);
            k_dims.width + 14.0 + 6.0 + d_dims.width
        };
        if cur_x + item_w > panel_x + panel_w - 16.0 && cur_x > panel_x + 20.0 {
            cur_x = panel_x + 16.0;
            cur_y += 26.0;
        }
        let w = draw_key_badge(cur_x, cur_y, key, desc);
        cur_x += w + 12.0;
    }
}

/// Right column: stacked stats panels, score progress bars, and the per-episode history
/// charts. Dynamically calculates available height so charts never clip or collapse on
/// smaller screens.
fn draw_stats_panels(
    game: &GameDQN,
    episode: usize,
    best_score: usize,
    history: &EpisodeHistory,
    screen_w: f32,
    screen_h: f32,
) {
    let left_col_width = screen_h - 320.0;
    let nn_width = 550.0;
    let panel_x = left_col_width + nn_width + 60.0;
    let panel_w = screen_w - panel_x - 20.0;

    if panel_w <= 10.0 {
        return;
    }

    let stats_h = 160.0;
    let run_h = 95.0;
    let bar_h = 55.0;
    let fixed_total = 20.0 + stats_h + 15.0 + run_h + 15.0 + bar_h + 10.0 + bar_h + 20.0;
    let remaining_h = (screen_h - fixed_total - 20.0).max(120.0);
    let chart_h = (remaining_h / 2.0) - 10.0;

    // TRAINING STATS
    let stats_y = 20.0;
    draw_terminal_box(panel_x, stats_y, panel_w, stats_h, "TRAINING STATS", false);
    if best_score > 0 {
        draw_badge("GUARDADO", panel_x + panel_w - 95.0, stats_y + 13.0, ACCENT_GREEN);
    }
    let mut y = stats_y + 45.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Episode:", &format!("{}", episode));
    y += 19.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Epsilon:", &format!("{:.3}", game.agent.get_epsilon()));
    y += 19.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Loss:", &format!("{:.5}", game.agent.loss_ema()));
    y += 19.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Buffer:", &format!("{}/{}", game.agent.replay_buffer.len(), REPLAY_BUFFER_SIZE));
    y += 19.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Target Updates:", &format!("{}", game.agent.target_updates()));
    y += 19.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Best:", &format!("{}", best_score));

    // RUN STATS
    let run_y = stats_y + stats_h + 15.0;
    draw_terminal_box(panel_x, run_y, panel_w, run_h, "RUN STATS", false);
    y = run_y + 48.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Score:", &format!("{}", game.score));
    y += 24.0;
    draw_stat_row(panel_x + 16.0, y, panel_w - 32.0, "Steps:", &format!("{}", game.steps));

    // SCORE progress bar
    let score_bar_y = run_y + run_h + 15.0;
    let score_pct = score_bar_fraction(game.score);
    let score_label = format!("SCORE: {}", game.score);
    draw_progress_bar(
        panel_x,
        score_bar_y,
        panel_w,
        bar_h,
        score_pct,
        &score_label,
        ACCENT_CYAN,
    );

    // BEST progress bar
    let best_bar_y = score_bar_y + bar_h + 10.0;
    let best_pct = score_bar_fraction(best_score);
    let best_label = format!("BEST: {}", best_score);
    draw_progress_bar(
        panel_x,
        best_bar_y,
        panel_w,
        bar_h,
        best_pct,
        &best_label,
        ACCENT_GOLD,
    );

    // Responsive history charts (zero heap allocations inside per-frame loop)
    let chart1_y = fixed_total;
    let max_time = history.times.iter().fold(0.0f32, |a, &b| a.max(b));
    let time_max_label = if history.times.is_empty() {
        String::new()
    } else {
        format!("MAX: {:.0}", max_time)
    };
    draw_responsive_chart(
        panel_x,
        chart1_y,
        panel_w,
        chart_h,
        "EPISODE TIMES",
        &time_max_label,
        &history.times,
        EPISODE_HISTORY_CAP,
        ACCENT_CYAN,
    );

    let chart2_y = chart1_y + chart_h + 20.0;
    let max_score = history.scores.iter().max().copied().unwrap_or(0);
    let score_max_label = if history.scores.is_empty() {
        String::new()
    } else {
        format!("MAX: {}", max_score)
    };
    draw_responsive_chart(
        panel_x,
        chart2_y,
        panel_w,
        chart_h,
        "EPISODE SCORES",
        &score_max_label,
        &history.scores_f32,
        EPISODE_HISTORY_CAP,
        ACCENT_GREEN,
    );
}

/// Dashboard entry point (design D-3/D-4): starts with `clear_background(COLOR_BG)`
/// and draws the four panels in order — left grid + model info, center network, right
/// stats/charts. Borrows only `&GameDQN`, the view's two scalars and the ring.
pub fn draw(game: &GameDQN, episode: usize, best_score: usize, history: &EpisodeHistory) {
    let theme = crate::theme::load_theme();
    draw_with_theme(game, episode, best_score, history, theme);
}

/// Variant of [`draw`] accepting an explicit [`GameTheme`].
pub fn draw_with_theme(
    game: &GameDQN,
    episode: usize,
    best_score: usize,
    history: &EpisodeHistory,
    theme: crate::theme::GameTheme,
) {
    let screen_w = screen_width();
    let screen_h = screen_height();
    clear_background(COLOR_BG);
    draw_grid(game, screen_h, theme);
    draw_model_info(game, screen_h);
    draw_neural_network(game, screen_h);
    draw_stats_panels(game, episode, best_score, history, screen_w, screen_h);
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- EpisodeHistory ring (design seam 3 / spec "ring caps at 50 and drops
    // the oldest"); assertions relocated verbatim from the view's staged tests. ---

    #[test]
    fn history_ring_caps_at_fifty_and_evicts_the_oldest_pair() {
        let mut history = EpisodeHistory::new();
        for i in 0..60 {
            history.push(i as f32, (i * 10) as usize);
        }
        assert_eq!(
            history.len(),
            EPISODE_HISTORY_CAP,
            "ring must hold exactly EPISODE_HISTORY_CAP entries"
        );
        assert_eq!(history.len(), 50, "60 pushes must cap the ring at 50");
        assert_eq!(history.times.first(), Some(&10.0), "oldest pair (time 0) evicted");
        assert_eq!(history.scores.first(), Some(&100), "oldest pair (score 0) evicted");
        assert_eq!(history.times.last(), Some(&59.0), "newest pair present");
        assert_eq!(history.scores.last(), Some(&590), "newest pair present");
    }

    #[test]
    fn history_times_and_scores_stay_index_aligned_across_evictions() {
        let mut history = EpisodeHistory::new();
        for i in 0..55 {
            history.push(i as f32, (i * 10) as usize);
        }
        assert_eq!(history.len(), 50);
        for (n, (time, score)) in history
            .times
            .iter()
            .zip(history.scores.iter())
            .enumerate()
        {
            assert_eq!(*time, (n + 5) as f32, "pair {n} time out of order");
            assert_eq!(*score, (n + 5) * 10, "pair {n} score out of order");
            assert_eq!(history.scores_f32[n], (n + 5) as f32 * 10.0, "scores_f32 {n} out of order");
        }
    }

    #[test]
    fn history_clear_empties_both_vectors_and_the_ring_stays_reusable() {
        let mut history = EpisodeHistory::new();
        history.push(1.0, 1);
        history.push(2.0, 4);
        assert_eq!(history.len(), 2);
        history.clear();
        assert_eq!(history.len(), 0, "clear empties the ring");
        assert!(
            history.times.is_empty() && history.scores.is_empty() && history.scores_f32.is_empty(),
            "clear empties all vectors"
        );
        history.push(3.0, 9);
        assert_eq!(history.times, vec![3.0], "cleared ring accepts new entries");
        assert_eq!(history.scores, vec![9]);
        assert_eq!(history.scores_f32, vec![9.0]);
    }

    // --- output_intensity clamp (design seam 5 / spec "output-node color
    // input") ----------------------------------------------------------------

    #[test]
    fn output_intensity_clamps_the_color_driving_domain_to_zero_one() {
        assert_eq!(output_intensity(-0.5), 0.0, "-0.5 clamps to 0.0");
        assert_eq!(output_intensity(0.5), 0.5, "interior values pass through");
        assert_eq!(output_intensity(1.5), 1.0, "1.5 clamps to 1.0");
    }

    // --- score-bar fraction over the documented DQN bound (design §3 / spec
    // "score-bar fractions are clamped against the documented episode bound") ---

    #[test]
    fn score_bar_fraction_clamps_against_the_documented_episode_bound() {
        let full_bar = DQN_STEP_LIMIT as f32;
        assert_eq!(full_bar, 500.0, "documented episode bound is 500 steps");
        assert_eq!(score_bar_fraction(0), 0.0);
        assert_eq!(
            score_bar_fraction(full_bar as usize),
            1.0,
            "a score equal to the bound fills the bar"
        );
        assert_eq!(
            score_bar_fraction(full_bar as usize + 50),
            1.0,
            "scores above the bound clamp to 1.0"
        );
    }
}

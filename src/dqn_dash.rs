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

use crate::configs::{
    GRID_H, GRID_W, HIDDEN_LAYER_SIZE, INP_LAYER_SIZE, NUM_SIM_STEPS, OUTPUT_LAYER_SIZE,
};
use crate::dqn::{
    BATCH_SIZE, EPSILON_DECAY, EPSILON_END, EPSILON_START, GAMMA, LEARNING_RATE,
    REPLAY_BUFFER_SIZE,
};
use crate::game_dqn::GameDQN;
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
    cap: usize,
}

impl EpisodeHistory {
    /// An empty ring capped at [`EPISODE_HISTORY_CAP`] entries.
    pub fn new() -> Self {
        Self {
            times: Vec::new(),
            scores: Vec::new(),
            cap: EPISODE_HISTORY_CAP,
        }
    }

    /// Append one `(time, score)` pair; overflow evicts the oldest pair so the
    /// ring holds at most `cap` entries with the newest last.
    pub fn push(&mut self, time: f32, score: usize) {
        debug_assert!(
            self.times.len() == self.scores.len(),
            "EpisodeHistory invariant broken: times/scores out of alignment"
        );
        self.times.push(time);
        self.scores.push(score);
        while self.len() > self.cap {
            self.times.remove(0);
            self.scores.remove(0);
        }
    }

    /// Empty both vectors (fresh-agent session reset).
    pub fn clear(&mut self) {
        self.times.clear();
        self.scores.clear();
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
/// §3): `FULL_BAR = (NUM_SIM_STEPS * 2) as f32` = 200.0 today — `GameDQN::step`
/// forces `done` at `steps >= NUM_SIM_STEPS * 2` and every food eaten consumes a
/// step, so `score <= steps <= 200`. Clamped to [0,1]; the GA reference's `/20`
/// denominator is never used on the DQN dashboard. Consumed by
/// [`crate::dqn_dash`]'s `draw_stats_panels` to size the "SCORE" and "BEST"
/// bars.
fn score_bar_fraction(score: usize) -> f32 {
    let full_bar = (NUM_SIM_STEPS * 2) as f32;
    (score as f32 / full_bar).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// VizAdvanced-grammar constants copied verbatim (design D-2). The reference
// (`viz_advanced.rs`) is under a zero-diff guard; nothing is exported from it
// or shared with it — the mirror is self-contained so the whole diff can be
// reviewed line by line against the reference.
// ---------------------------------------------------------------------------

const PANEL_BG: Color = Color::new(0.05, 0.05, 0.05, 0.95);
const PANEL_BORDER: Color = Color::new(0.4, 0.4, 0.4, 1.0);
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const ACCENT_COLOR: Color = Color::new(0.0, 0.9, 0.9, 1.0);
const TITLE_SIZE: f32 = 22.0;
const TEXT_SIZE: f32 = 18.0;

/// One bordered panel with its accent title — same body/geometry as the
/// reference `VizAdvanced::draw_panel` (title baseline at `y + 30`).
fn draw_panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, PANEL_BG);
    draw_rectangle_lines(x, y, w, h, 3.0, PANEL_BORDER);
    draw_text(title, x + 20.0, y + 30.0, TITLE_SIZE, ACCENT_COLOR);
}

/// Left column: the live `GameDQN` grid — reference `draw_game_grid` geometry
/// verbatim (`x=20, y=20, grid_size = screen_h − 340`, border −8/+16, black
/// field, 0.15 grid lines, WHITE food with a `tile − 4` inset). Only the one
/// live snake is drawn (head `(0.3,0.9,0.3)`, body `(0.2,0.7,0.2)`, the
/// reference's best-snake style) — DQN has no top-N population, so there is NO
/// ghost/rank loop (proposal success-criterion divergence).
fn draw_grid(game: &GameDQN, screen_h: f32) {
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

    // Food (single live game)
    draw_rectangle(
        x + game.food.x as f32 * tile_size + 2.0,
        y + game.food.y as f32 * tile_size + 2.0,
        tile_size - 4.0,
        tile_size - 4.0,
        WHITE,
    );

    // The one live snake
    for (i, segment) in game.body.iter().enumerate() {
        let segment_color = if i == 0 {
            Color::new(0.3, 0.9, 0.3, 1.0) // Head
        } else {
            Color::new(0.2, 0.7, 0.2, 1.0) // Body
        };
        draw_rectangle(
            x + segment.x as f32 * tile_size + 1.0,
            y + segment.y as f32 * tile_size + 1.0,
            tile_size - 2.0,
            tile_size - 2.0,
            segment_color,
        );
    }
}

/// Center column: full-height "NEURAL NETWORK" panel — reference
/// `draw_neural_network` geometry verbatim (`left_col_width = screen_h − 320`,
/// `panel_x = left_col_width + 40`, `panel_w = 550`, `panel_h = screen_h − 40`,
/// `y = 20`). Fed LIVE every frame: the current 12-input observation through
/// the q-network (`predict(&game.observation()).last()`) — same sigmoid output
/// domain as the reference, so the output-node color mapping transfers
/// unchanged (design D-4/D-5). Input nodes I0..I11 (accent, r=7), hidden nodes
/// H0..H7 (orange `(0.9,0.6,0.0)`, r=8), output nodes (r=10) colored by
/// [`output_intensity`] → `(v, v*0.3, v*0.9)` with labels
/// `["LEFT","RIGHT","BOTTOM","TOP"]` (order matches `GameDQN::step`'s
/// action 0..3 mapping).
fn draw_neural_network(game: &GameDQN, screen_h: f32) {
    let left_col_width = screen_h - 320.0;
    let panel_w = 550.0;
    let panel_h = screen_h - 40.0;
    let panel_x = left_col_width + 40.0;
    let panel_y = 20.0;

    draw_rectangle(panel_x, panel_y, panel_w, panel_h, PANEL_BG);
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 3.0, PANEL_BORDER);
    draw_text("NEURAL NETWORK", panel_x + 20.0, panel_y + 28.0, TITLE_SIZE, ACCENT_COLOR);

    let outputs = game.agent.q_network.predict(&game.observation());
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
        let intensity = output_intensity(value) as f32;
        let color = Color::new(intensity, intensity * 0.3, intensity * 0.9, 1.0);
        draw_circle(output_x, y, 10.0, color);
        draw_text(output_labels[i], output_x + 22.0, y + 7.0, 20.0, TEXT_COLOR);
    }
}

/// Bottom-left model-info panel — reference `draw_model_info` geometry verbatim
/// (`x=20, y=screen_h − 300, w=screen_h − 320, h=280`), title "DQN TRAIN".
/// Every row value comes from the pub seams (design D-5/D-6), never a literal:
/// architecture/batch/gamma/LR/epsilon-schedule/step-limit, then an
/// accent controls line at the panel bottom (replaces the shell hint in this
/// zone — design D-8). The replay-buffer row moved to the right column's
/// TRAINING STATS panel.
fn draw_model_info(_game: &GameDQN, screen_h: f32) {
    let panel_x = 20.0;
    let panel_y = screen_h - 300.0;
    let panel_w = screen_h - 320.0;
    let panel_h = 280.0;

    draw_panel(panel_x, panel_y, panel_w, panel_h, "DQN TRAIN");

    let mut y = panel_y + 55.0;
    draw_text(
        &format!(
            "Architecture: {}x{}x{}",
            INP_LAYER_SIZE, HIDDEN_LAYER_SIZE, OUTPUT_LAYER_SIZE
        ),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 26.0;
    draw_text(
        &format!("Batch: {} | Gamma: {:.2}", BATCH_SIZE, GAMMA),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 26.0;
    draw_text(
        &format!("LR: {}", LEARNING_RATE),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 26.0;
    draw_text(
        &format!(
            "Epsilon: {:.2} -> {:.2} (x{:.3})",
            EPSILON_START, EPSILON_END, EPSILON_DECAY
        ),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 26.0;
    draw_text(
        &format!("Step Limit: {}", NUM_SIM_STEPS * 2),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    // Accent controls line at the panel bottom (mirrors the reference's
    // controls row at `panel_y + 240`).
    y += 55.0;
    draw_text(
        "Controls: [TAB] HUD  [R] Fresh Agent  [ESC] Menu",
        panel_x + 20.0,
        y,
        18.0,
        ACCENT_COLOR,
    );
}

/// One bar chart panel — reference `VizAdvanced::draw_chart` body verbatim:
/// title at `+20/+30`, chart region `chart_x/y = +20/+50`, `chart_w = w−40`,
/// `chart_h = h−70`, `max_val = data.max().max(1.0)`, `step = chart_w / len`,
/// bar `w = step.max(4) − 1`. Self-normalizing against the series max with a
/// 1.0 floor, so an empty/all-zero series draws no bars and never divides by
/// zero (spec "empty history charts render safely").
fn draw_chart(x: f32, y: f32, w: f32, h: f32, title: &str, data: &[f32], color: Color) {
    draw_rectangle(x, y, w, h, PANEL_BG);
    draw_rectangle_lines(x, y, w, h, 3.0, PANEL_BORDER);
    draw_text(title, x + 20.0, y + 30.0, TITLE_SIZE, ACCENT_COLOR);

    if data.is_empty() {
        return;
    }

    let chart_x = x + 20.0;
    let chart_y = y + 50.0;
    let chart_w = w - 40.0;
    let chart_h = h - 70.0;

    let max_val = data.iter().fold(0.0f32, |a, &b| a.max(b)).max(1.0);
    let step = chart_w / data.len().max(1) as f32;

    for (i, &val) in data.iter().enumerate() {
        let bar_h = (val / max_val) * chart_h;
        let bar_x = chart_x + i as f32 * step;
        let bar_y = chart_y + chart_h - bar_h;
        draw_rectangle(bar_x, bar_y, step.max(4.0) - 1.0, bar_h, color);
    }
}

/// Right column: stacked stats panels, score bars, and the per-episode history
/// charts — reference `draw_stats_panels` geometry (`x = left_col_width + 550
/// + 60`, `w = screen_w − x − 20`). Panel slots re-stacked for the DQN
/// workflow: TRAINING STATS (Episode/Epsilon/Loss/Buffer/Target Updates/Best)
/// @ y=20 h=180, RUN STATS @ y=230 h=130, SCORE bar @ 380, BEST bar @ 500,
/// charts @ 630, with DQN rows/data inside. The SCORE and BEST
/// bars both use [`score_bar_fraction`] over the documented episode bound — the
/// GA `/20` denominator is never used (design §3).
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

    // TRAINING STATS (h=180, six compact rows @24px). Metrics mirror the DQN
    // workflow: episode, exploration rate, smoothed TD loss, replay-buffer
    // fill, target-network syncs, and the session best.
    let stats_y = 20.0;
    draw_panel(panel_x, stats_y, panel_w, 180.0, "TRAINING STATS");
    let mut y = stats_y + 55.0;
    draw_text(
        &format!("Episode: {}", episode),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 24.0;
    draw_text(
        &format!("Epsilon: {:.3}", game.agent.get_epsilon()),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 24.0;
    draw_text(
        &format!("Loss: {:.5}", game.agent.loss_ema()),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 24.0;
    draw_text(
        &format!(
            "Buffer: {}/{}",
            game.agent.replay_buffer.len(),
            REPLAY_BUFFER_SIZE
        ),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 24.0;
    draw_text(
        &format!("Target Updates: {}", game.agent.target_updates()),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 24.0;
    draw_text(
        &format!("Best: {}", best_score),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );

    // RUN STATS (h=130) — shifted down to y=230 to fit the taller TRAINING STATS.
    let run_y = 230.0;
    draw_panel(panel_x, run_y, panel_w, 130.0, "RUN STATS");
    y = run_y + 45.0;
    draw_text(
        &format!("Score: {}", game.score),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );
    y += 30.0;
    draw_text(
        &format!("Steps: {}", game.steps),
        panel_x + 20.0,
        y,
        TEXT_SIZE,
        TEXT_COLOR,
    );

    // SCORE bar (live game score) at y=380.
    let score_bar_y = 380.0;
    draw_panel(panel_x, score_bar_y, panel_w, 100.0, "SCORE");
    let score_pct = score_bar_fraction(game.score);
    let bar_y = score_bar_y + 50.0;
    draw_rectangle(panel_x + 20.0, bar_y, (panel_w - 40.0) * score_pct, 30.0, MAGENTA);
    draw_rectangle_lines(panel_x + 20.0, bar_y, panel_w - 40.0, 30.0, 2.0, PANEL_BORDER);
    draw_text(
        &format!("{:.0}%", score_pct * 100.0),
        panel_x + panel_w / 2.0 - 20.0,
        bar_y + 21.0,
        18.0,
        WHITE,
    );

    // BEST bar (session best) at y=500.
    let best_bar_y = 500.0;
    draw_panel(panel_x, best_bar_y, panel_w, 100.0, "BEST");
    let best_pct = score_bar_fraction(best_score);
    let bar_y = best_bar_y + 50.0;
    draw_rectangle(panel_x + 20.0, bar_y, (panel_w - 40.0) * best_pct, 30.0, RED);
    draw_rectangle_lines(panel_x + 20.0, bar_y, panel_w - 40.0, 30.0, 2.0, PANEL_BORDER);
    draw_text(
        &format!("{:.0}%", best_pct * 100.0),
        panel_x + panel_w / 2.0 - 20.0,
        bar_y + 21.0,
        18.0,
        WHITE,
    );

    // History charts at y=630.
    let chart_y = 630.0;
    let chart_h = (screen_h - chart_y - 20.0) / 2.0 - 10.0;
    draw_chart(
        panel_x,
        chart_y,
        panel_w,
        chart_h,
        "EPISODE TIMES",
        &history.times,
        SKYBLUE,
    );
    let scores_f32: Vec<f32> = history.scores.iter().map(|&s| s as f32).collect();
    draw_chart(
        panel_x,
        chart_y + chart_h + 20.0,
        panel_w,
        chart_h,
        "EPISODE SCORES",
        &scores_f32,
        GREEN,
    );
}

/// Dashboard entry point (design D-3/D-4): starts with `clear_background(BLACK)`
/// (mirroring `sim.rs::draw_advanced`) and calls the column helpers in the
/// reference order — left grid + model info, center network, right
/// stats/charts. Borrows only `&GameDQN`, the view's two scalars and the ring.
pub fn draw(game: &GameDQN, episode: usize, best_score: usize, history: &EpisodeHistory) {
    let screen_w = screen_width();
    let screen_h = screen_height();
    clear_background(BLACK);
    draw_grid(game, screen_h);
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
            history.times.is_empty() && history.scores.is_empty(),
            "clear empties both vectors"
        );
        history.push(3.0, 9);
        assert_eq!(history.times, vec![3.0], "cleared ring accepts new entries");
        assert_eq!(history.scores, vec![9]);
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
        // Full bar = (NUM_SIM_STEPS * 2) as f32 = 200.0 with the current config:
        // GameDQN forces done at steps >= NUM_SIM_STEPS * 2 and score grows only
        // when food is eaten (each food also consumes a step), so
        // score <= steps <= 200. Never the GA reference's /20 denominator.
        let full_bar = (NUM_SIM_STEPS * 2) as f32;
        assert_eq!(full_bar, 200.0, "documented episode bound is 200 steps");
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

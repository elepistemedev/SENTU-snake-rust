//! GA training view — [`GaTrainView`] drives a [`Simulation`] through the
//! behavior-preserving seam (slice 1) with the legacy GA pacing, while rendering
//! in the DQN-style layout family (AD-6).
//!
//! * Pacing: slow = one sim tick per frame plus the legacy `SIM_SLEEP_MILLIS`
//!   sleep; fast = batches of up to [`MAX_FAST_TICKS_PER_FRAME`] ticks per frame
//!   (the legacy driver's 50-tick cap). A running sim-internal VS sub-state
//!   forces slow mode, exactly as the driver forced slow while
//!   `sim.is_vs_mode()` was true.
//! * Rendering: default is a single centered grid (the current best snake, via
//!   `Simulation::snapshot`) plus a text HUD. `Tab` (via
//!   [`GaTrainView::toggle_advanced`]) switches to the pre-existing `VizAdvanced`
//!   dashboard, which is drawn through the small public
//!   [`Simulation::draw_advanced`] accessor — that dashboard is untouched.
//! * While the sim's internal VS sub-state runs, the view renders the two games
//!   side-by-side through `VizVS::draw_flavored` with the GA-default flavor
//!   (byte-identical to the legacy VS output); the sim keeps auto-returning to
//!   Training on its own.
//!
//! Input ownership: the shell (slice 5) dispatches keys to
//! [`GaTrainView::toggle_slow`], [`GaTrainView::toggle_advanced`], and
//! [`GaTrainView::trigger_vs`]; `Esc`-to-menu is shell-owned, so this view never
//! polls keys itself.

use std::thread;
use std::time::Duration;

use macroquad::prelude::*;

use crate::configs::{GRID_H, GRID_W, SIM_SLEEP_MILLIS};
use crate::sim::{SimMode, Simulation};
use crate::viz_vs::{VizVS, VsFlavor};

/// Fast-mode batching cap, matching the legacy GA driver: at most this many sim
/// ticks per frame.
pub const MAX_FAST_TICKS_PER_FRAME: usize = 50;

/// Number of sim ticks to run in one shell frame for a given pacing request and
/// VS-sub-state. Pure pacing seam: slow mode steps exactly one batch per frame;
/// an active internal VS match also forces a single step per frame (the legacy
/// driver recomputed `is_slow_mode || sim.is_vs_mode()` every frame); fast
/// training batches up to [`MAX_FAST_TICKS_PER_FRAME`].
pub fn frame_tick_budget(slow_requested: bool, vs_active: bool) -> usize {
    if slow_requested || vs_active {
        1
    } else {
        MAX_FAST_TICKS_PER_FRAME
    }
}

/// Which renderer draws the current frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaRenderTarget {
    /// DQN-style single grid + text HUD (the default).
    Hud,
    /// The pre-existing `VizAdvanced` dashboard (`Tab`-gated).
    Advanced,
    /// The side-by-side versus arena for the sim's internal VS sub-state.
    Versus,
}

/// Pure render-path decision: a running internal VS match always draws the
/// versus arena (VS beats the advanced dashboard and the DQN-style HUD alike);
/// the advanced dashboard shows only while training with `Tab` enabled;
/// otherwise the DQN-style HUD is drawn.
pub fn render_target(advanced_enabled: bool, mode: SimMode) -> GaRenderTarget {
    match mode {
        SimMode::VS => GaRenderTarget::Versus,
        SimMode::Training if advanced_enabled => GaRenderTarget::Advanced,
        SimMode::Training => GaRenderTarget::Hud,
    }
}

/// One GA training session, wrapping a live [`Simulation`].
///
/// The sim is resumed naturally: like today's `Simulation::new()`, leaving and
/// re-entering GA train simply reloads `sim_metadata.json`/`best_snake.json`, so
/// no pause/resume bookkeeping lives here (AD-3).
pub struct GaTrainView {
    sim: Simulation,
    /// User pacing request (`Space`): `true` = one batch per frame + sleep.
    slow: bool,
    /// Advanced `VizAdvanced` dashboard toggle (`Tab`). Default off = DQN-style.
    advanced: bool,
}

impl GaTrainView {
    /// Start a GA training session. Defaults to slow pacing (legacy default) and
    /// the DQN-style HUD (AD-6: `Tab` toggles the advanced dashboard).
    pub fn new() -> Self {
        Self {
            sim: Simulation::new(),
            slow: true,
            advanced: false,
        }
    }

    /// Advance the sim one frame using the current pacing: a single tick (plus
    /// the legacy sleep) when slow or inside the internal VS sub-state, or a
    /// batch of up to [`MAX_FAST_TICKS_PER_FRAME`] when fast. Training and VS
    /// ticks are selected per-iteration from the sim's live mode, so an auto-VS
    /// that begins mid-batch behaves exactly like the legacy driver.
    pub fn tick(&mut self) {
        let budget = frame_tick_budget(self.slow, self.sim.is_vs_mode());
        for _ in 0..budget {
            match self.sim.mode() {
                SimMode::Training => self.sim.tick_training(),
                SimMode::VS => self.sim.tick_vs(),
            }
        }
        if budget == 1 {
            thread::sleep(Duration::from_millis(SIM_SLEEP_MILLIS));
        }
    }

    /// User pacing request (slow = one batch per frame + sleep).
    pub fn is_slow(&self) -> bool {
        self.slow
    }

    /// Set the pacing request (shell maps `Space` key-release to a toggle via
    /// [`GaTrainView::toggle_slow`]).
    pub fn toggle_slow(&mut self) {
        self.slow = !self.slow;
    }

    /// Whether the advanced `VizAdvanced` dashboard is selected.
    pub fn advanced_enabled(&self) -> bool {
        self.advanced
    }

    /// Toggle the advanced dashboard (`Tab` key).
    pub fn toggle_advanced(&mut self) {
        self.advanced = !self.advanced;
    }

    /// Whether the sim is inside its internal VS sub-state right now.
    pub fn is_vs_active(&self) -> bool {
        self.sim.is_vs_mode()
    }

    /// User-triggered VS (`V` key): enters the sim's internal VS sub-state when a
    /// best-ever net exists. Preserves the legacy semantic that entering VS
    /// forces slow mode (returns `true` on entry so callers know a match began).
    pub fn trigger_vs(&mut self) -> bool {
        if self.sim.toggle_vs_mode() {
            self.slow = true;
            true
        } else {
            false
        }
    }

    /// Current sim mode.
    pub fn mode(&self) -> SimMode {
        self.sim.mode()
    }

    /// Draw the frame selected by [`render_target`].
    pub fn draw(&self) {
        match render_target(self.advanced, self.sim.mode()) {
            GaRenderTarget::Versus => self.draw_versus(),
            GaRenderTarget::Advanced => self.sim.draw_advanced(),
            GaRenderTarget::Hud => self.draw_dqn_style(),
        }
    }

    /// Internal VS sub-state: draw both games side-by-side with the GA-default
    /// flavor (record = best-ever score), byte-identical to the legacy output.
    fn draw_versus(&self) {
        if let Some((game1, game2)) = self.sim.vs_state() {
            let record = self.sim.snapshot().best_ever;
            VizVS::new().draw_flavored(game1, game2, &VsFlavor::ga_default(record));
        }
    }

    /// DQN-style default: a left-anchored text HUD (generation, generation max,
    /// best ever, elapsed seconds, current champion score/fitness/steps) plus the
    /// current best snake's grid, centered and sized from the screen dimensions
    /// (AD-8) like the DQN train view.
    fn draw_dqn_style(&self) {
        clear_background(BLACK);

        let snap = self.sim.snapshot();

        draw_text(
            &format!("Generation: {}", snap.gen_count),
            10.0,
            30.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Gen Max: {}", snap.gen_max),
            10.0,
            60.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Best Ever: {}", snap.best_ever),
            10.0,
            90.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Elapsed: {:.1}s", snap.elapsed_secs),
            10.0,
            120.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Score: {}", snap.champ_score),
            10.0,
            150.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Fitness: {:.2}", snap.champ_fitness),
            10.0,
            180.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("Steps: {}", snap.champ_steps),
            10.0,
            210.0,
            30.0,
            WHITE,
        );
        draw_text(
            "[SPACE] Slow/Fast  [TAB] Dashboard  [V] VS  [ESC] Menu",
            10.0,
            250.0,
            20.0,
            GRAY,
        );

        if let Some(best_game) = snap.best_game {
            let (tile_size, offset_x, offset_y) = self.grid_layout();

            // Food
            draw_rectangle(
                offset_x + best_game.food.x as f32 * tile_size,
                offset_y + best_game.food.y as f32 * tile_size,
                tile_size,
                tile_size,
                RED,
            );

            // Snake
            for (i, segment) in best_game.body.iter().enumerate() {
                let color = if i == 0 { GREEN } else { DARKGREEN };
                draw_rectangle(
                    offset_x + segment.x as f32 * tile_size,
                    offset_y + segment.y as f32 * tile_size,
                    tile_size,
                    tile_size,
                    color,
                );
            }

            // Grid lines
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
        }
    }

    /// Compute `(tile_size, offset_x, offset_y)` so the `GRID_W × GRID_H` grid
    /// fits and is centered in the space right of the left HUD column at any
    /// resolution (AD-8). The GA HUD carries more rows/longer labels than the
    /// DQN HUD, so its column is a little wider.
    fn grid_layout(&self) -> (f32, f32, f32) {
        const HUD_COL: f32 = 300.0;
        const MARGIN: f32 = 20.0;

        let screen_w = screen_width();
        let screen_h = screen_height();
        let avail_w = (screen_w - HUD_COL - MARGIN).max(100.0);
        let avail_h = (screen_h - MARGIN * 2.0).max(100.0);

        let tile_size = (avail_w / GRID_W as f32).min(avail_h / GRID_H as f32);
        let grid_w = tile_size * GRID_W as f32;
        let grid_h = tile_size * GRID_H as f32;
        let offset_x = HUD_COL + (avail_w - grid_w) * 0.5;
        let offset_y = MARGIN + (avail_h - grid_h) * 0.5;
        (tile_size, offset_x, offset_y)
    }
}

#[cfg(test)]
mod tests {
    use super::{frame_tick_budget, render_target, GaRenderTarget};
    use crate::sim::SimMode;

    // --- frame_tick_budget: pacing pure seam -----------------------------------

    #[test]
    fn slow_training_steps_exactly_one_batch_per_frame() {
        assert_eq!(frame_tick_budget(true, false), 1);
    }

    #[test]
    fn internal_vs_substate_forces_single_step_per_frame() {
        // A running sim-internal VS match always runs slowly, exactly like the
        // legacy driver forced slow mode while `sim.is_vs_mode()` was true.
        assert_eq!(frame_tick_budget(false, true), 1);
        assert_eq!(frame_tick_budget(true, true), 1);
    }

    #[test]
    fn fast_training_batches_up_to_fifty_ticks_per_frame() {
        assert_eq!(frame_tick_budget(false, false), 50);
    }

    // --- render_target: which renderer draws, given mode + advanced toggle -----

    #[test]
    fn vs_substate_always_routes_to_the_versus_renderer() {
        // Even when the user has the advanced dashboard enabled, a running
        // internal VS match must render the two games side-by-side.
        assert_eq!(render_target(true, SimMode::VS), GaRenderTarget::Versus);
        assert_eq!(render_target(false, SimMode::VS), GaRenderTarget::Versus);
    }

    #[test]
    fn advanced_dashboard_is_training_only_and_tab_gated() {
        assert_eq!(
            render_target(true, SimMode::Training),
            GaRenderTarget::Advanced
        );
        assert_eq!(render_target(false, SimMode::Training), GaRenderTarget::Hud);
    }
}

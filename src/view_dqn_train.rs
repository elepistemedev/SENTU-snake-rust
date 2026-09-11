//! DQN training view — [`DqnTrainView`].
//!
//! Owns a [`GameDQN`] and advances it exactly one `GameDQN::step()` per frame
//! (via [`DqnTrainView::tick`]), carrying over the episode/best-score
//! bookkeeping and champion snapshot logic from the former `snake-dqn` binary.
//!
//! The view is a passive, resumable resource (AD-3): the shell pauses DQN
//! training by *stopping* calling [`DqnTrainView::tick`] while the menu is
//! shown — the struct, agent, episode counter, session best, and champion stay
//! alive — and resumes by calling it again. `Esc` handling and the `R` hotkey
//! belong to the shell; this view exposes [`DqnTrainView::fresh_agent`] for the
//! latter.
//!
//! Drawing: a fresh view defaults to the advanced dashboard target
//! ([`DqnRenderTarget::Dashboard`] — the `VizAdvanced`-grammar mirror fed by
//! live DQN data, drawn by the `dqn_dash` module). `Tab` toggles to the legacy
//! compact DQN-style HUD ([`DqnRenderTarget::Hud`]): a left-anchored text
//! column plus a grid that is centered and sized from `screen_width()`/
//! `screen_height()` (no hardcoded 800×600 offsets). Toggling never perturbs
//! the in-flight episode or session bookkeeping.

use macroquad::prelude::*;

use crate::champion_store;
use crate::configs::{GRID_H, GRID_W};
use crate::dqn_dash::{self, EpisodeHistory};
use crate::game_dqn::GameDQN;
use crate::nn::Net;

/// Where the DQN champion snapshot is persisted (serde `Net`, same format as
/// `best_snake.json`). Missing or corrupt contents degrade to no champion.
pub const DQN_CHAMPION_FILE: &str = "dqn_champion.json";

/// Which renderer draws the current DQN-train frame (design D-3, mirror of the
/// GA render-target seam). Two states only: DQN has no sim-internal VS
/// sub-state, so unlike GA there is no versus arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DqnRenderTarget {
/// The advanced dashboard (default): `VizAdvanced`-grammar mirror fed by
/// live DQN data (`dqn_dash` module, slice B).
Dashboard,
/// The legacy compact DQN-style HUD (text column + grid).
Hud,
}

/// Pure render-path decision (macroquad-free, RED-first seam, design D-3).
pub fn dqn_render_target(dashboard_enabled: bool) -> DqnRenderTarget {
if dashboard_enabled {
DqnRenderTarget::Dashboard
} else {
DqnRenderTarget::Hud
}
}

/// Episode-end record decision (pure, macroquad-free).
///
/// Given the score an episode just ended with, the current session best, and
/// the q-network that produced that run, return `(new best, champion snapshot)`.
/// The snapshot is `Some(q_network.clone())` exactly when `score > best_score`
/// (a new session record); ties and lower scores keep the best and return
/// `None`, so an existing champion is never replaced by an equal or worse run.
pub fn on_episode_end(
    score: usize,
    best_score: usize,
    q_network: &Net,
) -> (usize, Option<Net>) {
    if score > best_score {
        (score, Some(q_network.clone()))
    } else {
        (best_score, None)
    }
}

/// One resumable DQN training session: a [`GameDQN`], its episode/best
/// bookkeeping, and the latest champion snapshot (loaded on construction,
/// replaced and re-persisted on every new session record).
pub struct DqnTrainView {
    game: GameDQN,
    episode: usize,
    best_score: usize,
    champion: Option<Net>,
    /// Whether the advanced dashboard is the active render target (design
    /// D-3). Default `true`: the dashboard is the DQN-train default; `Tab`
    /// toggles to the compact HUD.
    dashboard: bool,
    /// Per-episode history fed at every `end_episode` (design D-7); cleared by
    /// `fresh_agent`. Survives menu pause/resume as part of the session state.
    history: EpisodeHistory,
    /// Where the champion is loaded from/saved to. Defaults to
    /// [`DQN_CHAMPION_FILE`]; tests inject a private temp path.
    champion_path: &'static str,
}

impl DqnTrainView {
    /// Start a fresh training session: a brand-new agent/board and zeroed
    /// bookkeeping, with any previously persisted champion loaded from
    /// `dqn_champion.json` (missing or corrupt file → no champion, never a
    /// panic).
    pub fn new() -> Self {
        Self::at_path(DQN_CHAMPION_FILE)
    }

    /// Same as [`DqnTrainView::new`] but with a custom champion file path.
    /// Private: production always uses [`DQN_CHAMPION_FILE`]; tests use this to
    /// keep the real champion file untouched.
    fn at_path(champion_path: &'static str) -> Self {
        // Only a champion matching the current DQN architecture (9×32×3) is
        // accepted; a stale 12×8×4 file is discarded gracefully (no panic).
        let champion = champion_store::load(champion_path)
            .filter(|net| net.matches_arch(&crate::dqn::DQN_ARCH));
        let meta_path = if champion_path == DQN_CHAMPION_FILE {
            champion_store::DQN_METADATA_FILE.to_string()
        } else {
            format!("{champion_path}.meta.json")
        };
        let metadata = champion_store::load_metadata(&meta_path);
        let (best_score, episode) = if champion.is_some() {
            if let Some(m) = metadata {
                (m.best_score, m.episode)
            } else {
                (1, 0)
            }
        } else {
            (0, 0)
        };
        Self {
            game: GameDQN::new(),
            episode,
            best_score,
            champion,
            dashboard: true,
            history: EpisodeHistory::new(),
            champion_path,
        }
    }

    /// Advance training one frame: exactly one `GameDQN::step()`. When the
    /// episode ends, bookkeeping runs through [`on_episode_end`]; a record
    /// replaces the in-memory champion and persists it via
    /// `champion_store::save(…, champion)`, then the board resets.
    pub fn tick(&mut self) {
        let (_reward, done) = self.game.step();
        if done {
            self.end_episode();
        }
    }

    fn end_episode(&mut self) {
        // Record the completed episode's (step-count duration, final score)
        // BEFORE any bookkeeping/reset, per the spec: exactly one entry per
        // completed episode, duration = steps at completion (deterministic,
        // no wall-clock timer). Reconciles design D-7.
        self.history.push(self.game.steps as f32, self.game.score);
        self.episode += 1;
        let (new_best, snapshot) =
            on_episode_end(self.game.score, self.best_score, &self.game.agent.q_network);
        self.best_score = new_best;

        if let Some(net) = snapshot {
            self.champion = Some(net.clone());
            println!(
                "Episode {} ended | NEW DQN RECORD | Score: {} | Best: {} | Epsilon: {:.3}",
                self.episode,
                self.game.score,
                self.best_score,
                self.game.agent.get_epsilon()
            );
            if let Err(e) = champion_store::save(self.champion_path, &net) {
                eprintln!(
                    "warning: could not persist DQN champion to {}: {e}",
                    self.champion_path
                );
            }
            let meta = champion_store::DqnMetadata {
                best_score: self.best_score,
                episode: self.episode,
            };
            let meta_path = if self.champion_path == DQN_CHAMPION_FILE {
                champion_store::DQN_METADATA_FILE.to_string()
            } else {
                format!("{}.meta.json", self.champion_path)
            };
            if let Err(e) = champion_store::save_metadata(&meta_path, &meta) {
                eprintln!("warning: could not persist DQN metadata to {meta_path}: {e}");
            }
        }

        self.game.reset();
    }

    /// Explicitly synchronize current metadata to disk on session exit/pause.
    pub fn sync_save(&self) {
        if self.champion.is_some() {
            let meta = champion_store::DqnMetadata {
                best_score: self.best_score,
                episode: self.episode,
            };
            let meta_path = if self.champion_path == DQN_CHAMPION_FILE {
                champion_store::DQN_METADATA_FILE.to_string()
            } else {
                format!("{}.meta.json", self.champion_path)
            };
            let _ = champion_store::save_metadata(&meta_path, &meta);
        }
    }

    /// Start a fresh agent: a new [`GameDQN`] (random q-network, epsilon 1.0,
    /// empty replay buffer) and zeroed episode/best bookkeeping. The champion
    /// is *retained* — it is a recorded historical snapshot, not a
    /// session-best artifact, so restarting the learner does not discard it.
    /// The shell maps the `R` hotkey to this method.
    pub fn fresh_agent(&mut self) {
        self.game = GameDQN::new();
        self.episode = 0;
        self.best_score = 0;
        // Session-reset semantics: the recorded per-episode history belongs to
        // the old learner. The champion and the active render target survive
        // (design seam 4 / spec).
        self.history.clear();
    }

    /// Score of the live game.
    pub fn score(&self) -> usize {
        self.game.score
    }

    /// Completed episodes in this session.
    pub fn episode(&self) -> usize {
        self.episode
    }

    /// Session best score.
    pub fn best_score(&self) -> usize {
        self.best_score
    }

    /// Current exploration rate of the live agent.
    pub fn epsilon(&self) -> f64 {
        self.game.agent.get_epsilon()
    }

    /// The live current policy: the q-network being trained right now. Greedy
    /// when frozen in the arena (arena brains always argmax).
    pub fn live_net(&self) -> &Net {
        &self.game.agent.q_network
    }

    /// The champion snapshot if one exists (loaded on construction and replaced
    /// on every new record).
    pub fn champion(&self) -> Option<&Net> {
        self.champion.as_ref()
    }

    /// Whether the advanced dashboard is the active render target.
    pub fn dashboard_enabled(&self) -> bool {
        self.dashboard
    }

    /// Toggle the render target between the dashboard and the compact HUD
    /// (shell maps the `Tab` key to this method). Pure state flip: never
    /// perturbs the in-flight episode or session bookkeeping.
    pub fn toggle_dashboard(&mut self) {
        self.dashboard = !self.dashboard;
    }

    /// The per-episode history ring (fed at every `end_episode`, cleared by
    /// `fresh_agent`), for the dashboard charts and tests.
    pub fn history(&self) -> &EpisodeHistory {
        &self.history
    }

    /// Draw the current frame. The advanced dashboard is the default target
    /// (design D-3): [`dqn_render_target`] routes to the `dqn_dash` mirror (fed
    /// by the live game plus this view's `episode`/`best_score`/history
    /// bookkeeping); the `Tab`-toggled legacy compact HUD draws through
    /// [`DqnTrainView::draw_hud`]. Dispatching never perturbs the in-flight
    /// episode or session state.
    pub fn draw(&self, theme: crate::theme::GameTheme) {
        match dqn_render_target(self.dashboard) {
            DqnRenderTarget::Dashboard => dqn_dash::draw_with_theme(
                &self.game,
                self.episode,
                self.best_score,
                &self.history,
                theme,
            ),
            DqnRenderTarget::Hud => self.draw_hud(theme),
        }
    }

    /// Draw the legacy compact DQN-style HUD (Episode/Score/Best/Epsilon) plus
    /// the snake grid. The grid is centered and sized from the current screen
    /// dimensions — the old `main_dqn` used hardcoded `offset_x = 250`,
    /// `tile_size = 20` offsets that assumed an 800×600 window. Moved unchanged
    /// out of the former single `draw()` (design D-3).
    fn draw_hud(&self, theme: crate::theme::GameTheme) {
        clear_background(BLACK);

        draw_text(
            &format!("Episode: {}", self.episode),
            10.0,
            30.0,
            30.0,
            WHITE,
        );
        draw_text(&format!("Score: {}", self.game.score), 10.0, 60.0, 30.0, WHITE);
        draw_text(&format!("Best: {}", self.best_score), 10.0, 90.0, 30.0, WHITE);
        draw_text(
            &format!("Epsilon: {:.3}", self.game.agent.get_epsilon()),
            10.0,
            120.0,
            30.0,
            WHITE,
        );

        let (tile_size, offset_x, offset_y) = self.grid_layout();

        let colors = theme.colors();

        // Food
        crate::render_snake::draw_apple(
            offset_x + self.game.food.x as f32 * tile_size,
            offset_y + self.game.food.y as f32 * tile_size,
            tile_size,
            theme,
            colors.food,
        );

        // Snake
        for (i, segment) in self.game.body.iter().enumerate() {
            let seg_x = offset_x + segment.x as f32 * tile_size;
            let seg_y = offset_y + segment.y as f32 * tile_size;
            let color = if i == 0 { colors.head } else { colors.body };
            crate::render_snake::draw_connected_segment(
                &self.game.body,
                i,
                self.game.dir,
                seg_x,
                seg_y,
                tile_size,
                theme,
                color,
                self.game.swallow.bulge_at(i),
                self.game.swallow.head_scale(),
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

    /// Compute `(tile_size, offset_x, offset_y)` so the `GRID_W × GRID_H` grid
    /// fits and is centered in the space right of the left HUD column, at any
    /// resolution (AD-8). At 800×600 this reproduces the old `main_dqn` family
    /// (left HUD at x≈10, grid starting at x≈250).
    fn grid_layout(&self) -> (f32, f32, f32) {
        // Left column reserved for the HUD text, as in the old main_dqn layout.
        const HUD_COL: f32 = 250.0;
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
    use super::*;

    /// Unique per-test champion file so parallel tests never race on disk.
    const TEST_BOOKKEEPING_FILE: &str = "dqn_champion_bookkeeping_test.json";
    const TEST_ROUNDTRIP_FILE: &str = "dqn_champion_roundtrip_test.json";
    const TEST_OLD_ARCH_FILE: &str = "dqn_champion_old_arch_test.json";

    fn cleanup_test_file(path: &str) {
        std::fs::remove_file(path).ok();
        std::fs::remove_file(format!("{path}.meta.json")).ok();
    }

    /// `Net` has no `PartialEq`. Exact-weight comparison (valid for in-memory
    /// clones, which never pass through serialization).
    fn nets_eq(a: &Net, b: &Net) -> bool {
        if a.layers.len() != b.layers.len() {
            return false;
        }
        for (la, lb) in a.layers.iter().zip(b.layers.iter()) {
            if la.nodes.len() != lb.nodes.len() {
                return false;
            }
            for (na, nb) in la.nodes.iter().zip(lb.nodes.iter()) {
                if na.len() != nb.len() {
                    return false;
                }
                for (wa, wb) in na.iter().zip(nb.iter()) {
                    if wa != wb {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Tolerance comparison for nets that went through serde JSON: f64 decimal
    /// round trips are not bit-exact at the last ulp (see slice-1 evidence).
    #[allow(dead_code)]
    fn nets_approx_eq(a: &Net, b: &Net) -> bool {
        if a.layers.len() != b.layers.len() {
            return false;
        }
        for (la, lb) in a.layers.iter().zip(b.layers.iter()) {
            if la.nodes.len() != lb.nodes.len() {
                return false;
            }
            for (na, nb) in la.nodes.iter().zip(lb.nodes.iter()) {
                if na.len() != nb.len() {
                    return false;
                }
                for (wa, wb) in na.iter().zip(nb.iter()) {
                    if (wa - wb).abs() > 1e-9 {
                        return false;
                    }
                }
            }
        }
        true
    }

    // --- on_episode_end: pure record decision (RED-first seam) -----------------

    #[test]
    fn on_episode_end_snapshots_q_network_exactly_when_score_beats_best() {
        let q_network = Net::new();
        let (new_best, snapshot) = on_episode_end(7, 3, &q_network);
        assert_eq!(new_best, 7, "record score must become the new best");
        let snap = snapshot.expect("a record episode must snapshot the q_network");
        assert!(
            nets_eq(&snap, &q_network),
            "snapshot must be an exact copy of the q_network"
        );
    }

    #[test]
    fn on_episode_end_returns_no_snapshot_on_tie() {
        let q_network = Net::new();
        let (new_best, snapshot) = on_episode_end(5, 5, &q_network);
        assert_eq!(new_best, 5, "best unchanged on a tie");
        assert!(snapshot.is_none(), "a tie must not replace the champion");
    }

    #[test]
    fn on_episode_end_returns_no_snapshot_on_lower_score() {
        let q_network = Net::new();
        let (new_best, snapshot) = on_episode_end(2, 9, &q_network);
        assert_eq!(new_best, 9, "best unchanged on a lower score");
        assert!(
            snapshot.is_none(),
            "a lower score must not replace the champion"
        );
    }

    // --- DqnTrainView bookkeeping (behavior pins, macroquad-free) ---------------

    /// A session bound to a private test champion file, so random records
    /// during tick tests never touch the real `dqn_champion.json`.
    fn test_view() -> DqnTrainView {
        DqnTrainView::at_path(TEST_BOOKKEEPING_FILE)
    }

    #[test]
    fn bounded_ticks_advance_an_episode_and_reset_the_board() {
        // GameDQN's step limit is NUM_SIM_STEPS * 2 = 200 steps, and every
        // episode must end within it (walls/self-collision end it earlier), so
        // 250 ticks deterministically complete at least one episode.
        let mut view = test_view();
        for _ in 0..250 {
            view.tick();
        }
        assert!(
            view.episode() >= 1,
            "after 250 ticks (>= the 200-step limit) at least one episode must end"
        );
        assert!(
            !view.game.is_complete,
            "an episode end must reset the board in the same tick"
        );
        assert!(
            view.best_score() <= 60,
            "sanctity: a session best can never exceed the food eaten in a run"
        );
        // Episode bookkeeping advanced regardless of whether a record happened.
        let _ = view.episode();
        cleanup_test_file(TEST_BOOKKEEPING_FILE);
    }

    #[test]
    fn fresh_agent_resets_epsilon_and_bookkeeping_but_keeps_champion() {
        let mut view = test_view();
        view.episode = 4;
        view.best_score = 9;
        // Train past the replay batch size so epsilon decays below 1.0.
        for _ in 0..60 {
            view.tick();
        }
        assert!(
            view.epsilon() < 0.99,
            "epsilon must decay below 1.0 after 60 training steps"
        );

            let recorded = Net::new();
            view.champion = Some(recorded.clone());
            // Slice A: pre-fill the per-episode history and flip the render target
            // to the compact HUD; a fresh agent clears history but preserves both
            // the champion and the display choice (design seam 4 / spec).
            view.history.push(1.0, 1);
            view.history.push(2.0, 4);
            view.history.push(3.0, 9);
            assert!(view.history().len() >= 3, "history pre-filled before fresh_agent");
            view.toggle_dashboard();
            assert!(
                !view.dashboard_enabled(),
                "pre-toggled to the compact HUD before fresh_agent"
            );
            view.fresh_agent();

            assert_eq!(view.episode(), 0, "fresh agent zeroes the episode counter");
            assert_eq!(view.history().len(), 0, "fresh agent empties the per-episode history");
            assert!(
                !view.dashboard_enabled(),
                "fresh agent must not change the active render target"
            );
        assert_eq!(view.best_score(), 0, "fresh agent zeroes the session best");
        assert!(
            (view.epsilon() - 1.0).abs() < 1e-9,
            "fresh agent restarts epsilon at 1.0"
        );
        let kept = view.champion().expect("champion must survive a fresh agent");
        assert!(
            nets_eq(kept, &recorded),
            "a recorded champion is not a session artifact and must be retained"
        );
        cleanup_test_file(TEST_BOOKKEEPING_FILE);
    }

    #[test]
    fn construction_loads_dqn_arch_champion_and_rejects_old_arch() {
        // Round-trip with the new DQN architecture (9x32x3): a record persists
        // a champion, and construction must load it back (serde JSON is not
        // bit-exact at the last ulp, so compare within 1e-9).
        let dqn_net = Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        champion_store::save(TEST_ROUNDTRIP_FILE, &dqn_net).expect("save must succeed");
        let view = DqnTrainView::at_path(TEST_ROUNDTRIP_FILE);
        let loaded = view
            .champion()
            .expect("construction must load the persisted champion");
        assert!(
            loaded.matches_arch(&crate::dqn::DQN_ARCH),
            "loaded champion must match the current DQN architecture"
        );
        std::fs::remove_file(TEST_ROUNDTRIP_FILE).ok();
    }

    #[test]
    fn old_arch_champion_is_gracefully_discarded() {
        let old_net = Net::new(); // 12x8x4, pre-relative-action DQN arch
        champion_store::save(TEST_OLD_ARCH_FILE, &old_net).expect("save must succeed");
        let view = DqnTrainView::at_path(TEST_OLD_ARCH_FILE);
        assert!(
            view.champion().is_none(),
            "old-arch champion must be discarded on load"
        );
        std::fs::remove_file(TEST_OLD_ARCH_FILE).ok();
    }

    // --- Slice A: DQN render-target seam (design D-3, spec "dashboard is the
    // default target") --------------------------------------------------------

    #[test]
    fn dqn_render_target_maps_the_dashboard_flag_to_the_render_target() {
        assert_eq!(dqn_render_target(true), DqnRenderTarget::Dashboard);
        assert_eq!(dqn_render_target(false), DqnRenderTarget::Hud);
    }

    #[test]
    fn fresh_view_defaults_to_dashboard_and_toggle_round_trips_without_perturbing_training() {
        let mut view = test_view();
        view.episode = 7;
        view.best_score = 11;
        let champion = Net::new();
        view.champion = Some(champion.clone());

        assert!(view.dashboard_enabled(), "a fresh view must open on the dashboard");
        assert_eq!(
            dqn_render_target(view.dashboard_enabled()),
            DqnRenderTarget::Dashboard
        );

        // Ticks between toggles (3 < the 12-step minimum wall path and the
        // 4-step self-collision loop) never flip the target nor reset the
        // in-flight session bookkeeping or champion.
        for _ in 0..3 {
            view.tick();
        }
        assert!(view.dashboard_enabled(), "ticking must not change the target");
        assert_eq!(view.episode(), 7, "ticking must not reset the episode counter");
        assert_eq!(view.best_score(), 11, "ticking must not reset the session best");
        assert!(
            nets_eq(view.champion().expect("champion set above"), &champion),
            "ticking must not replace the champion"
        );

        view.toggle_dashboard();
        assert!(!view.dashboard_enabled(), "one toggle leaves the dashboard");
        assert_eq!(
            dqn_render_target(view.dashboard_enabled()),
            DqnRenderTarget::Hud
        );
        for _ in 0..3 {
            view.tick();
        }
        assert!(!view.dashboard_enabled(), "ticking must not flip the target back");
        assert_eq!(view.episode(), 7, "ticking must not reset the episode counter");

        view.toggle_dashboard();
        assert!(view.dashboard_enabled(), "a second toggle returns to the dashboard");
        assert_eq!(
            dqn_render_target(view.dashboard_enabled()),
            DqnRenderTarget::Dashboard
        );
        assert_eq!(view.episode(), 7, "toggling never resets the session");
        assert_eq!(view.best_score(), 11, "toggling never resets the session best");
        cleanup_test_file(TEST_BOOKKEEPING_FILE);
    }

    // --- Slice A: episode-end / fresh-agent history bookkeeping (spec) ---------

    #[test]
    fn end_episode_records_one_history_entry_from_pre_reset_steps_and_score() {
        let mut view = test_view();
        view.episode = 3;
        view.best_score = 10; // above the seeded score: no record, no champion save
        view.game.score = 5;
        view.game.steps = 71;
        assert_eq!(
            view.history().len(),
            0,
            "no entries before the first episode end"
        );

        view.end_episode();

        assert_eq!(
            view.history().len(),
            1,
            "exactly one entry per completed episode"
        );
        assert_eq!(
            view.history().scores,
            vec![5],
            "entry holds the episode's final score"
        );
        assert_eq!(
            view.history().times,
            vec![71.0],
            "entry duration = step count at completion (pre-reset, no wall clock)"
        );
        assert_eq!(view.episode(), 4, "episode bookkeeping advanced");
        assert_eq!(view.game.score, 0, "board reset after recording");
        assert_eq!(view.game.steps, 0, "board reset after recording");
        assert!(
            view.champion().is_none(),
            "below-best episode never touches the champion"
        );
        cleanup_test_file(TEST_BOOKKEEPING_FILE);
    }

    #[test]
    fn existing_champion_and_metadata_persists_across_sessions_and_ignores_lower_scores() {
        const TEST_PERSIST_FILE: &str = "dqn_champion_persist_test.json";
        let meta_file = format!("{TEST_PERSIST_FILE}.meta.json");

        // Clean any leftover
        std::fs::remove_file(TEST_PERSIST_FILE).ok();
        std::fs::remove_file(&meta_file).ok();

        // Save initial champion with score 20, ep 50
        let original_champion = Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        champion_store::save(TEST_PERSIST_FILE, &original_champion).unwrap();
        let meta = champion_store::DqnMetadata {
            best_score: 20,
            episode: 50,
        };
        champion_store::save_metadata(&meta_file, &meta).unwrap();

        // Open new session at that path
        let mut view = DqnTrainView::at_path(TEST_PERSIST_FILE);
        assert_eq!(view.best_score(), 20, "session must initialize best_score from metadata");
        assert_eq!(view.episode(), 50, "session must initialize episode from metadata");
        assert!(view.champion().is_some());

        // Simulate an episode scoring 5 (below 20)
        view.game.score = 5;
        view.game.steps = 30;
        view.end_episode();

        // Best score must NOT be reduced, and champion must NOT be replaced
        assert_eq!(view.best_score(), 20);
        let loaded_net = champion_store::load(TEST_PERSIST_FILE).unwrap();
        assert!(nets_approx_eq(&loaded_net, &original_champion), "champion must not be replaced by lower score");

        // Simulate an episode scoring 25 (beats 20)
        view.game.score = 25;
        view.game.steps = 100;
        view.end_episode();

        assert_eq!(view.best_score(), 25);
        let loaded_meta = champion_store::load_metadata(&meta_file).unwrap();
        assert_eq!(loaded_meta.best_score, 25);
        assert_eq!(loaded_meta.episode, 52);

        // Cleanup
        std::fs::remove_file(TEST_PERSIST_FILE).ok();
        std::fs::remove_file(&meta_file).ok();
    }
}

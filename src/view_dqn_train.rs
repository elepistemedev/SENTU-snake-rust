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
//! latter. Drawing follows the DQN-style layout family of the old `main_dqn`:
//! a left-anchored HUD column plus a grid that is centered and sized from
//! `screen_width()`/`screen_height()` (no hardcoded 800×600 offsets).

use macroquad::prelude::*;

use crate::champion_store;
use crate::configs::{GRID_H, GRID_W};
use crate::game_dqn::GameDQN;
use crate::nn::Net;

/// Where the DQN champion snapshot is persisted (serde `Net`, same format as
/// `best_snake.json`). Missing or corrupt contents degrade to no champion.
pub const DQN_CHAMPION_FILE: &str = "dqn_champion.json";

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
        let champion = champion_store::load(champion_path);
        Self {
            game: GameDQN::new(),
            episode: 0,
            best_score: 0,
            champion,
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
        }

        self.game.reset();
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

    /// Draw the DQN-style HUD (Episode/Score/Best/Epsilon) plus the snake grid.
    /// The grid is centered and sized from the current screen dimensions — the
    /// old `main_dqn` used hardcoded `offset_x = 250`, `tile_size = 20` offsets
    /// that assumed an 800×600 window.
    pub fn draw(&self) {
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

        // Food
        draw_rectangle(
            offset_x + self.game.food.x as f32 * tile_size,
            offset_y + self.game.food.y as f32 * tile_size,
            tile_size,
            tile_size,
            RED,
        );

        // Snake
        for (i, segment) in self.game.body.iter().enumerate() {
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
        std::fs::remove_file(TEST_BOOKKEEPING_FILE).ok();
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
        view.fresh_agent();

        assert_eq!(view.episode(), 0, "fresh agent zeroes the episode counter");
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
        std::fs::remove_file(TEST_BOOKKEEPING_FILE).ok();
    }

    #[test]
    fn construction_loads_champion_persisted_by_a_record() {
        // Round-trip the real wiring on a private file: a record persists a
        // champion, and construction must load it back. serde JSON is not
        // bit-exact at the last ulp, so compare within 1e-9.
        let net = Net::new();
        champion_store::save(TEST_ROUNDTRIP_FILE, &net).expect("save must succeed");
        let view = DqnTrainView::at_path(TEST_ROUNDTRIP_FILE);
        let loaded = view
            .champion()
            .expect("construction must load the persisted champion");
        assert!(
            nets_approx_eq(loaded, &net),
            "loaded champion must match the saved one (within 1e-9)"
        );
        std::fs::remove_file(TEST_ROUNDTRIP_FILE).ok();
    }
}

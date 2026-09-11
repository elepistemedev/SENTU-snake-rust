//! Shared versus arena — two `Net` brains fight head-to-head.
//!
//! The pure core ([`Winner`], [`resolve_winner`], [`run_headless_match`],
//! [`VersusMatch`], [`BestOfSeries`]) is completely macroquad-free and
//! presentation-agnostic, stepping [`Game`]s in lockstep. Rendering is handled
//! separately by the presentation layer ([`crate::viz_vs`]).
//!
//! [`BestOfSeries`] orchestrates a 5-game series with best-of-3 semantics:
//! it holds the active match, tracks wins per side, and advances automatically
//! between games after a short frame-based pause (~2 s at 60 fps).

use crate::game::Game;
use crate::nn::Net;

/// Who won a finished versus match. `Left` is player 1 (first game), `Right`
/// is player 2 (second game); equal scores resolve to `Tie`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Winner {
    Left,
    Right,
    Tie,
}

/// Resolve a match result from two final scores. Pure score comparison:
/// higher score wins, equal scores tie.
pub fn resolve_winner(score1: usize, score2: usize) -> Winner {
    if score1 > score2 {
        Winner::Left
    } else if score2 > score1 {
        Winner::Right
    } else {
        Winner::Tie
    }
}

/// Run a headless versus match to completion, macroquad-free.
///
/// Both games are built with [`Game::with_brain`] and stepped in lockstep —
/// one [`Game::update`] each per tick — until both are complete or
/// `max_ticks` is exhausted. `Game::update` no-ops on a completed game, so a
/// player that dies early simply stops moving while the other keeps playing.
///
/// Returns `Some(winner)` when both games finished inside the budget;
/// `None` when the budget ran out first (pathological stalemate guard). In
/// practice the existing no-food step limits end a stuck game and a
/// random-walk snake cannot evade the arena walls forever, so even random
/// brains finish well within a generous budget.
pub fn run_headless_match(net_left: &Net, net_right: &Net, max_ticks: usize) -> Option<Winner> {
    let mut game1 = Game::with_brain(net_left);
    let mut game2 = Game::with_brain(net_right);

    for _ in 0..max_ticks {
        if game1.is_complete && game2.is_complete {
            break;
        }
        game1.update();
        game2.update();
    }

    if game1.is_complete && game2.is_complete {
        Some(resolve_winner(game1.score(), game2.score()))
    } else {
        None
    }
}

/// A versus match: two [`Game`]s with fixed brains stepped in lockstep.
pub struct VersusMatch {
    game1: Game,
    game2: Game,
}

impl VersusMatch {
    /// Build a fresh match pitting `net_left` (player 1) against `net_right`
    /// (player 2). Every construction is a fresh arena state.
    pub fn new(net_left: Net, net_right: Net) -> Self {
        Self {
            game1: Game::with_brain(&net_left),
            game2: Game::with_brain(&net_right),
        }
    }

    /// DQN-vs-DQN: both players use relative brains (9-input, 3-output).
    pub fn new_relative(net_left: Net, net_right: Net) -> Self {
        Self {
            game1: Game::with_relative_brain(&net_left),
            game2: Game::with_relative_brain(&net_right),
        }
    }

    /// Cross GA-vs-DQN: GA left (absolute brain), DQN right (relative brain).
    pub fn new_cross(ga_net: Net, dqn_net: Net) -> Self {
        Self {
            game1: Game::with_brain(&ga_net),
            game2: Game::with_relative_brain(&dqn_net),
        }
    }

    /// Access player 1's game.
    pub fn game1(&self) -> &Game {
        &self.game1
    }

    /// Access player 2's game.
    pub fn game2(&self) -> &Game {
        &self.game2
    }

    /// Advance the match one tick: each player moves at most once. The step is
    /// a no-op once the match is finished (`Game::update` also no-ops on a
    /// completed game, so a dead player stops while the survivor keeps going).
    pub fn tick(&mut self) {
        if self.is_finished() {
            return;
        }
        self.game1.update();
        self.game2.update();
    }

    /// True once both games have ended; only then is a [`Winner`] final.
    pub fn is_finished(&self) -> bool {
        self.game1.is_complete && self.game2.is_complete
    }

    /// The resolved winner, or `None` while the match is still running.
    pub fn winner(&self) -> Option<Winner> {
        if self.is_finished() {
            Some(resolve_winner(self.game1.score(), self.game2.score()))
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// BestOfSeries
// ---------------------------------------------------------------------------

/// Number of frames to pause between games (~2 s at 60 fps).
pub const SERIES_PAUSE_FRAMES: u32 = 120;
/// Total games in a series.
pub const SERIES_GAMES: usize = 5;
/// Wins needed to claim the series (best-of-3 within 5).
pub const SERIES_WINS_NEEDED: usize = 3;

/// Result of an individual stage/game within a series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageResult {
    /// 1-based stage index (1..=5).
    pub stage_number: usize,
    /// Who won this specific stage.
    pub winner: Winner,
    /// Player 1 (left) final score.
    pub score1: usize,
    /// Player 2 (right) final score.
    pub score2: usize,
}

/// A snapshot of the series scoreboard, suitable for the HUD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeriesInfo {
    /// Games completed so far (0..=5).
    pub games_played: usize,
    /// Total games in the series.
    pub total_games: usize,
    /// Left-side wins accumulated.
    pub left_wins: usize,
    /// Right-side wins accumulated.
    pub right_wins: usize,
    /// Tie games accumulated.
    pub ties: usize,
    /// True once the series has a final champion (or all 5 games played).
    pub is_over: bool,
    /// Results for each stage of the series.
    pub stages: [Option<StageResult>; SERIES_GAMES],
}

/// Orchestrates a 5-game series (best-of-3) with automatic transitions.
///
/// `B` is a callable `FnMut() -> VersusMatch` that constructs a fresh match
/// for every game. After each game finishes a short `pause_frames` countdown
/// runs before the next match is built. The series ends early the moment one
/// side accumulates [`SERIES_WINS_NEEDED`] wins.
pub struct BestOfSeries<B: FnMut() -> VersusMatch> {
    builder: B,
    current: VersusMatch,
    left_wins: usize,
    right_wins: usize,
    ties: usize,
    games_played: usize,
    /// Frames remaining in the between-game pause; `0` = not pausing.
    pause_remaining: u32,
    stages: [Option<StageResult>; SERIES_GAMES],
}

impl<B: FnMut() -> VersusMatch> BestOfSeries<B> {
    /// Start a fresh series using `builder` to construct each game.
    /// The first match is built immediately.
    pub fn new(mut builder: B) -> Self {
        let current = builder();
        Self {
            builder,
            current,
            left_wins: 0,
            right_wins: 0,
            ties: 0,
            games_played: 0,
            pause_remaining: 0,
            stages: [None; SERIES_GAMES],
        }
    }

    /// Reset the series to the beginning, constructing a fresh first match.
    pub fn restart(&mut self) {
        self.current = (self.builder)();
        self.left_wins = 0;
        self.right_wins = 0;
        self.ties = 0;
        self.games_played = 0;
        self.pause_remaining = 0;
        self.stages = [None; SERIES_GAMES];
    }

    /// Advance one frame:
    /// * If the current game just finished, record the result and start the
    ///   between-game pause.
    /// * While pausing, count down and, when the counter reaches zero, build
    ///   the next match (unless the series is already over).
    /// * Otherwise step the current match normally.
    pub fn tick(&mut self) {
        if self.is_series_over() {
            // Nothing to do once the series has ended.
            return;
        }

        if self.pause_remaining > 0 {
            self.pause_remaining -= 1;
            if self.pause_remaining == 0 && !self.is_series_over() {
                // Pause elapsed — start the next game.
                self.current = (self.builder)();
            }
            return;
        }

        // Normal play: step the current game.
        if !self.current.is_finished() {
            self.current.tick();
        }

        // Record result the moment the game ends.
        if self.current.is_finished() && self.games_played < SERIES_GAMES {
            let s1 = self.current.game1.score();
            let s2 = self.current.game2.score();
            let w = self.current.winner().unwrap_or(Winner::Tie);
            match w {
                Winner::Left => self.left_wins += 1,
                Winner::Right => self.right_wins += 1,
                Winner::Tie => self.ties += 1,
            }
            self.stages[self.games_played] = Some(StageResult {
                stage_number: self.games_played + 1,
                winner: w,
                score1: s1,
                score2: s2,
            });
            self.games_played += 1;
            // Begin pause only if there are more games to play.
            if !self.is_series_over() {
                self.pause_remaining = SERIES_PAUSE_FRAMES;
            }
        }
    }

    /// True when the series cannot produce any more games:
    /// either a side reached [`SERIES_WINS_NEEDED`] wins or all 5 were played.
    pub fn is_series_over(&self) -> bool {
        self.left_wins >= SERIES_WINS_NEEDED
            || self.right_wins >= SERIES_WINS_NEEDED
            || self.games_played >= SERIES_GAMES
    }

    /// The series champion: `Some(Winner::Left/Right)` when one side has
    /// [`SERIES_WINS_NEEDED`] wins; `Some(Winner::Tie)` when all 5 games are
    /// played with equal wins; `None` while the series is still running.
    pub fn series_winner(&self) -> Option<Winner> {
        if self.left_wins >= SERIES_WINS_NEEDED {
            return Some(Winner::Left);
        }
        if self.right_wins >= SERIES_WINS_NEEDED {
            return Some(Winner::Right);
        }
        if self.games_played >= SERIES_GAMES {
            return Some(if self.left_wins > self.right_wins {
                Winner::Left
            } else if self.right_wins > self.left_wins {
                Winner::Right
            } else {
                Winner::Tie
            });
        }
        None
    }

    /// The current (or last-played) match. Always valid.
    pub fn current_match(&self) -> &VersusMatch {
        &self.current
    }

    /// True while the between-game countdown is running.
    pub fn is_pausing(&self) -> bool {
        self.pause_remaining > 0
    }

    /// A scoreboard snapshot for the renderer HUD.
    pub fn series_info(&self) -> SeriesInfo {
        SeriesInfo {
            games_played: self.games_played,
            total_games: SERIES_GAMES,
            left_wins: self.left_wins,
            right_wins: self.right_wins,
            ties: self.ties,
            is_over: self.is_series_over(),
            stages: self.stages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- resolve_winner: pure score comparison -------------------------------

    #[test]
    fn resolve_winner_is_left_when_first_score_is_higher() {
        assert_eq!(resolve_winner(5, 3), Winner::Left);
        assert_eq!(resolve_winner(1, 0), Winner::Left);
        assert_eq!(resolve_winner(37, 1), Winner::Left);
    }

    #[test]
    fn resolve_winner_is_right_when_second_score_is_higher() {
        assert_eq!(resolve_winner(3, 5), Winner::Right);
        assert_eq!(resolve_winner(0, 1), Winner::Right);
        assert_eq!(resolve_winner(1, 37), Winner::Right);
    }

    #[test]
    fn resolve_winner_is_tie_on_equal_scores() {
        assert_eq!(resolve_winner(0, 0), Winner::Tie);
        assert_eq!(resolve_winner(7, 7), Winner::Tie);
        assert_eq!(resolve_winner(100, 100), Winner::Tie);
    }

    // --- Headless arena: macroquad-free lockstep termination -----------------

    #[test]
    fn headless_match_with_random_brains_terminates_and_yields_winner() {
        // No macroquad window: this only drives Game::update in lockstep. Each
        // tick both games step once. Random nets must finish well inside the
        // budget because the no-food step limits (Game::handle_step_limit) end
        // a stuck game and a random-walk snake cannot evade the walls forever.
        const MAX_TICKS: usize = 10_000;
        const TRIALS: usize = 20;

        for _ in 0..TRIALS {
            let net_left = Net::new();
            let net_right = Net::new();
            let winner = run_headless_match(&net_left, &net_right, MAX_TICKS);
            assert!(
                winner.is_some(),
                "headless match with random nets did not finish within {MAX_TICKS} ticks"
            );
        }
    }

    #[test]
    fn headless_match_respects_zero_tick_budget() {
        // With no ticks at all both games are still alive, so no winner.
        let net_left = Net::new();
        let net_right = Net::new();
        assert_eq!(run_headless_match(&net_left, &net_right, 0), None);
    }

    // --- VersusMatch state machine (renderer core, no drawing) ---------------

    #[test]
    fn versus_match_steps_both_games_until_finished_then_yields_winner() {
        let mut m = VersusMatch::new(Net::new(), Net::new());
        assert!(!m.is_finished());
        assert_eq!(m.winner(), None);

        // A random-brain match always completes within a generous tick budget;
        // `winner` must then be Some and stable across calls.
        let mut ticks = 0;
        while !m.is_finished() && ticks < 10_000 {
            m.tick();
            ticks += 1;
        }
        assert!(m.is_finished(), "match did not finish within 10_000 ticks");
        let winner = m.winner();
        assert!(winner.is_some());
        assert_eq!(m.winner(), winner, "winner must be stable after finish");
    }

    #[test]
    fn versus_match_finish_is_sticky_across_extra_ticks() {
        // Once finished, further ticks must not resurrect or re-roll anything:
        // the finished state and resolved winner stay put.
        let mut m = VersusMatch::new(Net::new(), Net::new());
        let mut ticks = 0;
        while !m.is_finished() && ticks < 10_000 {
            m.tick();
            ticks += 1;
        }
        assert!(m.is_finished());

        let winner_before = m.winner();
        for _ in 0..50 {
            m.tick();
        }
        assert!(m.is_finished());
        assert_eq!(m.winner(), winner_before);
    }

    #[test]
    fn headless_relative_match_terminates_and_yields_winner() {
        const MAX_TICKS: usize = 10_000;
        let dqn_net = Net::new_with_sizes(&[9, 32, 3]);
        let mut m = VersusMatch::new_relative(dqn_net.clone(), dqn_net.clone());
        let mut ticks = 0;
        while !m.is_finished() && ticks < MAX_TICKS {
            m.tick();
            ticks += 1;
        }
        assert!(m.is_finished(), "relative-brain match must finish within budget");
        assert!(m.winner().is_some());
    }

    #[test]
    fn headless_cross_match_ga_vs_dqn_terminates() {
        const MAX_TICKS: usize = 10_000;
        let ga_net = Net::new();
        let dqn_net = Net::new_with_sizes(&[9, 32, 3]);
        let mut m = VersusMatch::new_cross(ga_net, dqn_net);
        let mut ticks = 0;
        while !m.is_finished() && ticks < MAX_TICKS {
            m.tick();
            ticks += 1;
        }
        assert!(m.is_finished(), "cross GA-vs-DQN match must finish within budget");
        assert!(m.winner().is_some());
    }

    #[test]
    fn series_records_stage_results_for_each_completed_stage() {
        let builder = || VersusMatch::new(Net::new(), Net::new());
        let mut series = BestOfSeries::new(builder);
        assert_eq!(series.series_info().stages, [None; SERIES_GAMES]);

        // Finish stage 1 manually by setting completed
        series.current.game1.is_complete = true;
        series.current.game2.is_complete = true;
        // tick once to process the completion
        series.tick();

        let info = series.series_info();
        assert_eq!(info.games_played, 1);
        let s1 = info.stages[0].expect("stage 1 must be recorded");
        assert_eq!(s1.stage_number, 1);
        assert_eq!(s1.winner, Winner::Tie);
        assert_eq!(s1.score1, series.current.game1.score());
        assert_eq!(s1.score2, series.current.game2.score());

        // Restarting must clear all stages and scores
        series.restart();
        assert_eq!(series.series_info().stages, [None; SERIES_GAMES]);
        assert_eq!(series.series_info().games_played, 0);
        assert_eq!(series.series_info().left_wins, 0);
        assert_eq!(series.series_info().right_wins, 0);
    }
}

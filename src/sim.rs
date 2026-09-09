//! Simulación
//! Responsable de actualizar la población y la visualización
//! Gestiona las generaciones

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::game::Game;
use crate::nn::Net;
use crate::pop::Population;
use crate::viz_advanced::VizAdvanced;
use crate::viz_vs::VizVS;

#[derive(Serialize, Deserialize)]
struct SimMetadata {
    gen_count: usize,
    max_score_ever: usize,
    second_max_score_ever: usize,
    best_net: Option<Net>,
    second_best_net: Option<Net>,
    #[serde(default)]
    gen_times: Vec<f32>,
    #[serde(default)]
    gen_scores: Vec<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimMode {
    Training,
    VS,
}

/// Read-only metrics snapshot of a running `Simulation`, used to feed the
/// DQN-style GA training HUD without owning any drawing.
pub struct SimSnapshot<'a> {
    pub gen_count: usize,
    pub gen_max: usize,
    pub best_ever: usize,
    pub elapsed_secs: f32,
    pub champ_score: usize,
    pub champ_fitness: f32,
    pub champ_steps: usize,
    pub best_game: Option<&'a Game>,
}

pub struct Simulation {
    gen_count: usize,
    pop: Population,
    viz: VizAdvanced,
    viz_vs: VizVS,
    max_score_ever: usize,
    second_max_score_ever: usize,
    mode: SimMode,
    vs_game1: Option<Game>,
    vs_game2: Option<Game>,
    best_net_ever: Option<Net>,
    second_best_net_ever: Option<Net>,
}

/// Champions persisted for the standalone GA-versus arena.
///
/// Read from `sim_metadata.json` (best/second-best nets + all-time record)
/// with a `best_snake.json` fallback for the best net when the metadata file
/// is missing or lacks it. This is a passive snapshot of what a GA run left
/// on disk — loading it never mutates simulation state.
#[derive(Default)]
pub struct GaChampions {
    /// Best-ever net, when any persisted GA champion exists.
    pub best: Option<Net>,
    /// Second-best-ever net (only meaningful alongside `best`).
    pub second_best: Option<Net>,
    /// All-time best score, used as the GA versus "record to beat".
    pub record: usize,
}

/// Load the persisted GA champions for a standalone versus arena: best + second
/// best nets from `sim_metadata.json`, falling back to `best_snake.json` for the
/// best net when the metadata lacks it. Missing or corrupt files degrade to no
/// champions — never a panic.
pub fn load_ga_champions() -> GaChampions {
    let metadata = Simulation::load_metadata();
    let best = metadata.best_net.or_else(Population::load_best_net);
    GaChampions {
        best,
        second_best: metadata.second_best_net,
        record: metadata.max_score_ever,
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    pub fn new() -> Self {
        let metadata = Self::load_metadata();
        let mut viz = VizAdvanced::new();
        viz.set_history(metadata.gen_times, metadata.gen_scores);

        Self {
            gen_count: metadata.gen_count,
            pop: Population::new(),
            viz,
            viz_vs: VizVS::new(),
            max_score_ever: metadata.max_score_ever,
            second_max_score_ever: metadata.second_max_score_ever,
            mode: SimMode::Training,
            vs_game1: None,
            vs_game2: None,
            best_net_ever: metadata.best_net,
            second_best_net_ever: metadata.second_best_net,
        }
    }

    fn load_metadata() -> SimMetadata {
        if Path::new("sim_metadata.json").exists() {
            if let Ok(json) = fs::read_to_string("sim_metadata.json") {
                if let Ok(metadata) = serde_json::from_str(&json) {
                    return metadata;
                }
            }
        }
        SimMetadata {
            gen_count: 0,
            max_score_ever: 0,
            second_max_score_ever: 0,
            best_net: None,
            second_best_net: None,
            gen_times: Vec::new(),
            gen_scores: Vec::new(),
        }
    }

    fn save_metadata(&self) {
        let (gen_times, gen_scores) = self.viz.get_history();
        let metadata = SimMetadata {
            gen_count: self.gen_count,
            max_score_ever: self.max_score_ever,
            second_max_score_ever: self.second_max_score_ever,
            best_net: self.best_net_ever.clone(),
            second_best_net: self.second_best_net_ever.clone(),
            gen_times,
            gen_scores,
        };
        if let Ok(json) = serde_json::to_string_pretty(&metadata) {
            fs::write("sim_metadata.json", json).ok();
        }
    }

    pub fn update(&mut self, is_viz_enabled: bool, _is_slow_mode: bool) {
        match self.mode {
            SimMode::Training => {
                self.tick_training();

                if is_viz_enabled {
                    self.draw_advanced();
                }
            }
            SimMode::VS => {
                let both_complete = self.step_vs_games();

                if is_viz_enabled {
                    if let (Some(g1), Some(g2)) = (&self.vs_game1, &self.vs_game2) {
                        self.viz_vs.draw(g1, g2, self.max_score_ever);
                    }
                }

                // Auto return to training when both are dead
                if both_complete {
                    self.mode = SimMode::Training;
                    self.vs_game1 = None;
                    self.vs_game2 = None;
                }
            }
        }
    }

    /// Advance training by one population batch tick: steps the live games
    /// and, when a generation completes, closes it out, starts the next
    /// generation, and honors the every-100-generations auto-VS cadence.
    /// This is the logic half of the legacy `update()` training arm.
    pub fn tick_training(&mut self) {
        let games_alive = self.pop.update();
        if games_alive == 0 {
            self.end_current_genration();
            self.start_new_generation();

            // Auto VS every 100 generations
            if self.gen_count.is_multiple_of(100) && self.gen_count > 0 {
                self.toggle_vs_mode();
            }
        }
    }

    /// Advance the VS arena by one tick (each player moves once). When both
    /// games are complete, the sim returns to Training automatically. This
    /// is the logic half of the legacy `update()` VS arm.
    pub fn tick_vs(&mut self) {
        let both_complete = self.step_vs_games();
        if both_complete {
            self.mode = SimMode::Training;
            self.vs_game1 = None;
            self.vs_game2 = None;
        }
    }

    /// Steps both VS games once and reports whether both finished.
    fn step_vs_games(&mut self) -> bool {
        if let (Some(g1), Some(g2)) = (&mut self.vs_game1, &mut self.vs_game2) {
            g1.update();
            g2.update();
            g1.is_complete && g2.is_complete
        } else {
            false
        }
    }

    /// Current sim mode.
    pub fn mode(&self) -> SimMode {
        self.mode
    }

    /// References to both VS arena games when a match is running, or `None`
    /// while the sim is in Training mode.
    pub fn vs_state(&self) -> Option<(&Game, &Game)> {
        match (&self.vs_game1, &self.vs_game2) {
            (Some(g1), Some(g2)) => Some((g1, g2)),
            _ => None,
        }
    }

    /// Read-only HUD metrics derived from the live population and the
    /// all-time records, without mutating or drawing anything.
    pub fn snapshot(&self) -> SimSnapshot<'_> {
        let stats = self.pop.get_gen_summary();
        let best_game = self.pop.get_top_games(1).first().copied();
        SimSnapshot {
            gen_count: self.gen_count,
            gen_max: stats.max_score,
            best_ever: self.max_score_ever,
            elapsed_secs: stats.time_elapsed_secs,
            champ_score: best_game.map_or(0, Game::score),
            champ_fitness: best_game.map_or(0.0, Game::fitness),
            champ_steps: best_game.map_or(0, |g| g.num_steps),
            best_game,
        }
    }

    pub fn toggle_vs_mode(&mut self) -> bool {
        match self.mode {
            SimMode::Training => {
                if let Some(ref net1) = self.best_net_ever {
                    let net2 = if let Some(ref n) = self.second_best_net_ever {
                        n.clone()
                    } else {
                        // Fallback to current best if no second best
                        let top_games = self.pop.get_top_games(1);
                        if !top_games.is_empty() {
                            top_games[0].get_net().clone()
                        } else {
                            net1.clone()
                        }
                    };

                    self.vs_game1 = Some(Game::with_brain(net1));
                    self.vs_game2 = Some(Game::with_brain(&net2));
                    self.mode = SimMode::VS;
                    return true; // Signal to enable slow mode
                }
            }
            SimMode::VS => {
                self.mode = SimMode::Training;
                self.vs_game1 = None;
                self.vs_game2 = None;
            }
        }
        false
    }

    pub fn is_vs_mode(&self) -> bool {
        matches!(self.mode, SimMode::VS)
    }

    pub fn start_new_generation(&mut self) {
        self.gen_count += 1;
        self.pop.reset();
    }

    pub fn end_current_genration(&mut self) {
        let stats = self.pop.get_gen_summary();
        let current_score = stats.max_score;

        if current_score > self.max_score_ever {
            // New best - shift rankings
            self.second_best_net_ever = self.best_net_ever.clone();
            self.second_max_score_ever = self.max_score_ever;
            self.best_net_ever = stats.best_net.clone();
            self.max_score_ever = current_score;
        } else if current_score > self.second_max_score_ever && current_score > 0 {
            // New second best
            self.second_best_net_ever = stats.best_net.clone();
            self.second_max_score_ever = current_score;
        }

        self.viz
            .update_generation(stats.max_steps as f32, stats.max_score);

        // Save every 10 generations
        if self.gen_count.is_multiple_of(10) {
            self.pop.save_best_net();
            self.save_metadata();
        }
    }

    pub fn draw_advanced(&self) {
        clear_background(BLACK);

        let top_games = self.pop.get_top_games(10);
        if !top_games.is_empty() {
            let stats = self.pop.get_gen_summary();
            let best_game = top_games[0];

            self.viz.draw(
                &top_games,
                self.gen_count,
                self.max_score_ever,
                stats.max_score,
                stats.time_elapsed_secs,
                best_game.score(),
                best_game.fitness(),
                best_game.num_steps,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_metadata_deserializes_without_history_fields() {
        let legacy_json = r#"{
            "gen_count": 50,
            "max_score_ever": 15,
            "second_max_score_ever": 12,
            "best_net": null,
            "second_best_net": null
        }"#;

        let meta: SimMetadata = serde_json::from_str(legacy_json).expect("must parse legacy json");
        assert_eq!(meta.gen_count, 50);
        assert_eq!(meta.max_score_ever, 15);
        assert!(meta.gen_times.is_empty(), "default must be empty vec");
        assert!(meta.gen_scores.is_empty(), "default must be empty vec");
    }

    #[test]
    fn sim_metadata_round_trip_with_history() {
        let meta = SimMetadata {
            gen_count: 100,
            max_score_ever: 42,
            second_max_score_ever: 30,
            best_net: None,
            second_best_net: None,
            gen_times: vec![10.0, 20.0, 30.0],
            gen_scores: vec![1, 5, 12],
        };

        let json = serde_json::to_string(&meta).expect("serialize must succeed");
        let loaded: SimMetadata = serde_json::from_str(&json).expect("deserialize must succeed");

        assert_eq!(loaded.gen_count, 100);
        assert_eq!(loaded.max_score_ever, 42);
        assert_eq!(loaded.gen_times, vec![10.0, 20.0, 30.0]);
        assert_eq!(loaded.gen_scores, vec![1, 5, 12]);
    }

    #[test]
    fn population_get_gen_summary_includes_max_steps() {
        let pop = Population::new();
        let summary = pop.get_gen_summary();
        assert!(summary.max_score >= 1, "initial snake length is 1");
        assert_eq!(summary.max_steps, 0, "fresh population has not stepped");
    }
}

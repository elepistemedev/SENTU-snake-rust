//! Población
//! Gestiona múltiples streams (islas) de agentes neuro-evolutivos
//! También es responsable del Rejuvenecimiento de Islas

use std::fs;
use std::path::Path;
use std::time::Instant;

use rand::Rng;

use crate::stream::Stream;
use crate::*;

use self::nn::Net;

pub struct Population {
    gen_start_ts: Instant,
    streams: Vec<Stream>,
}

pub struct GenerationSummary {
    pub time_elapsed_secs: f32,
    pub max_score: usize,
    pub max_steps: usize,
    pub best_net: Option<Net>,
}

impl Population {
    pub fn new() -> Self {
        let saved_net = Self::load_best_net();
        Self::with_champion(saved_net)
    }

    pub fn with_champion(champion: Option<Net>) -> Self {
        let mut streams = Vec::new();

        for _ in 0..*NUM_STREAMS {
            let stream = if let Some(ref net) = champion {
                Stream::with_champion(net)
            } else {
                Stream::new()
            };
            streams.push(stream);
        }

        Self {
            gen_start_ts: Instant::now(),
            streams,
        }
    }

    pub fn update(&mut self) -> usize {
        let mut games_alive = *NUM_GAMES_PER_STREAM * *NUM_STREAMS;

        for stream in self.streams.iter_mut() {
            games_alive -= stream.update();
        }

        games_alive
    }

    pub fn reset(&mut self) {
        self.reset_with_champion(None);
    }

    pub fn reset_with_champion(&mut self, champion: Option<&Net>) {
        self.gen_start_ts = Instant::now();
        let mut nets: Vec<Net> = Vec::new();

        for stream in self.streams.iter_mut() {
            nets.push(stream.reset_with_champion(champion));
        }

        // Rejuvenecimiento de Islas
        let mut rng = rand::thread_rng();
        for stream in self.streams.iter_mut() {
            if !stream.is_local_maximum() {
                continue;
            }

            // Inyecta el mejor modelo de otro stream aleatorio
            stream.inject(&nets[rng.gen_range(0..nets.len())]);
        }
    }

    pub fn get_gen_summary(&self) -> GenerationSummary {
        let mut max_score = 0;
        let mut max_steps = 0;
        let mut best_net = None;

        for stream in self.streams.iter() {
            let (stream_score, stream_steps, stream_net) = stream.get_stream_summary();
            if stream_score > max_score {
                max_score = stream_score;
                best_net = stream_net;
            }
            if stream_steps > max_steps {
                max_steps = stream_steps;
            }
        }

        GenerationSummary {
            max_score,
            max_steps,
            time_elapsed_secs: self.gen_start_ts.elapsed().as_secs_f32(),
            best_net,
        }
    }

    pub fn get_best_game(&self) -> Option<&crate::game::Game> {
        let mut best_game: Option<&crate::game::Game> = None;

        for stream in self.streams.iter() {
            if let Some(game) = stream.get_best_game() {
                match best_game {
                    None => best_game = Some(game),
                    Some(current) => {
                        if crate::stream::is_better_game(game, current) {
                            best_game = Some(game);
                        }
                    }
                }
            }
        }

        best_game
    }

    pub fn get_top_games(&self, count: usize) -> Vec<&crate::game::Game> {
        let mut all_games = Vec::new();

        for stream in self.streams.iter() {
            all_games.extend(stream.get_all_games());
        }

        all_games.sort_by(|a, b| {
            let a_alive = !a.is_complete;
            let b_alive = !b.is_complete;
            b_alive.cmp(&a_alive)
                .then_with(|| b.score().cmp(&a.score()))
                .then_with(|| b.fitness().partial_cmp(&a.fitness()).unwrap_or(std::cmp::Ordering::Equal))
        });
        all_games.truncate(count);
        all_games
    }

    pub fn save_net(net: &Net) {
        if let Ok(json) = serde_json::to_string_pretty(net) {
            fs::write("best_snake.json", json).ok();
        }
    }

    pub fn save_best_net(&self) {
        if let Some(best_game) = self.get_best_game() {
            Self::save_net(best_game.get_net());
        }
    }

    /// Load the best net saved by a previous run from `best_snake.json`
    /// with fallback to `sim_metadata.json` (missing or corrupt → `None`, never a panic).
    /// Public so the standalone GA-versus and cross-match views can build a champion
    /// arena without a running `Simulation`.
    pub fn load_best_net() -> Option<Net> {
        if Path::new("best_snake.json").exists() {
            if let Ok(json) = fs::read_to_string("best_snake.json") {
                if let Ok(net) = serde_json::from_str(&json) {
                    return Some(net);
                }
            }
        }
        if Path::new("sim_metadata.json").exists() {
            if let Ok(json) = fs::read_to_string("sim_metadata.json") {
                #[derive(serde::Deserialize)]
                struct MetaFallback {
                    best_net: Option<Net>,
                }
                if let Ok(meta) = serde_json::from_str::<MetaFallback>(&json) {
                    if meta.best_net.is_some() {
                        return meta.best_net;
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn population_with_champion_initializes_streams_with_champion() {
        let champion = Net::new();
        let pop = Population::with_champion(Some(champion.clone()));
        let best_game = pop.get_best_game().expect("must have best game");
        let best_net = best_game.get_net();
        assert_eq!(best_net.n_inputs(), champion.n_inputs());
        assert_eq!(best_net.layers.len(), champion.layers.len());
    }

    #[test]
    fn population_get_top_games_prioritizes_alive_snakes() {
        let pop = Population::new();
        let top = pop.get_top_games(5);
        assert!(!top.is_empty());
        // Fresh population: all snakes alive
        for g in top {
            assert!(!g.is_complete);
        }
    }
}


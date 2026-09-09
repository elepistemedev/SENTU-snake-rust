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
    pub best_net: Option<Net>,
}

impl Population {
    pub fn new() -> Self {
        let mut streams = Vec::new();

        // Try to load saved model
        let saved_net = Self::load_best_net();

        for _ in 0..NUM_STREAMS {
            let mut stream = Stream::new();
            if let Some(ref net) = saved_net {
                stream.inject_net(net.clone());
            }
            streams.push(stream);
        }

        Self {
            streams,
            gen_start_ts: Instant::now(),
        }
    }

    pub fn update(&mut self) -> usize {
        let mut games_alive = NUM_GAMES_PER_STREAM * NUM_STREAMS;

        for stream in self.streams.iter_mut() {
            games_alive -= stream.update();
        }

        games_alive
    }

    pub fn reset(&mut self) {
        self.gen_start_ts = Instant::now();
        let mut nets = Vec::new();

        // Reinicio de streams
        for stream in self.streams.iter_mut() {
            let best_net = stream.reset();
            nets.push(best_net);
        }

        // No hay streams para cruzar
        if self.streams.len() <= 1 {
            return;
        }

        // Cruce de streams
        let mut rng = rand::thread_rng();
        for stream in self.streams.iter_mut() {
            if !stream.is_local_maximum() {
                continue;
            }

            stream.inject(&nets[rng.gen_range(0..nets.len())]);
        }
    }

    pub fn get_gen_summary(&self) -> GenerationSummary {
        let mut max_score = 0;
        let mut best_net = None;

        for stream in self.streams.iter() {
            let (stream_score, stream_net) = stream.get_stream_summary();
            if stream_score > max_score {
                max_score = stream_score;
                best_net = stream_net;
            }
        }

        GenerationSummary {
            max_score,
            time_elapsed_secs: self.gen_start_ts.elapsed().as_secs_f32(),
            best_net,
        }
    }

    pub fn get_best_game(&self) -> Option<&crate::game::Game> {
        let mut best_game: Option<&crate::game::Game> = None;
        let mut best_score = 0;

        for stream in self.streams.iter() {
            if let Some(game) = stream.get_best_game() {
                let score = game.score();
                if score > best_score {
                    best_score = score;
                    best_game = Some(game);
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

        all_games.sort_by(|a, b| b.score().cmp(&a.score()));
        all_games.truncate(count);
        all_games
    }

    pub fn save_best_net(&self) {
        if let Some(best_game) = self.get_best_game() {
            let net = best_game.get_net();
            let json = serde_json::to_string_pretty(&net).unwrap();
            fs::write("best_snake.json", json).ok();
        }
    }

    /// Load the best net saved by a previous run from `best_snake.json`
    /// (missing or corrupt → `None`, never a panic). Public so the standalone
    /// GA-versus and cross-match views can build a champion arena without a
    /// running `Simulation`.
    pub fn load_best_net() -> Option<Net> {
        if Path::new("best_snake.json").exists() {
            let json = fs::read_to_string("best_snake.json").ok()?;
            serde_json::from_str(&json).ok()
        } else {
            None
        }
    }
}

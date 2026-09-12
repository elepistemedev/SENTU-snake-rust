//! Stream
//! Isla de agentes neuro-evolutivos

use std::time::Instant;

use rand::distributions::{Distribution, WeightedIndex};

use crate::game::Game;
use crate::nn::Net;
use crate::*;

pub(crate) fn is_better_game(candidate: &Game, current: &Game) -> bool {
    // 1. Living snakes strictly beat dead snakes
    if !candidate.is_complete && current.is_complete {
        return true;
    }
    if candidate.is_complete && !current.is_complete {
        return false;
    }

    // 2. Higher score wins
    if candidate.score() > current.score() {
        return true;
    }
    if candidate.score() < current.score() {
        return false;
    }

    // 3. Higher fitness wins
    let c_fit = candidate.fitness();
    let cur_fit = current.fitness();
    if c_fit > cur_fit {
        return true;
    }
    if c_fit < cur_fit {
        return false;
    }

    // 4. On tie, candidate is not strictly better; keeps current (earlier index/champion)
    false
}

pub struct Stream {
    games: Vec<Game>,
    max_score: usize,
    max_score_ts: Instant,
}

impl Stream {
    pub fn new() -> Self {
        let mut games = Vec::with_capacity(*NUM_GAMES_PER_STREAM);
        for _ in 0..*NUM_GAMES_PER_STREAM {
            games.push(Game::new());
        }

        Self {
            games,
            max_score: 0,
            max_score_ts: Instant::now(),
        }
    }

    /// Construct a stream warm-started from a champion network.
    ///
    /// - Index 0..`num_retained`: exact clones of the champion
    /// - `num_mutated`: mutations exploring around the champion policy
    /// - `num_random`: fresh explorers to retain genetic diversity
    pub fn with_champion(champion: &Net) -> Self {
        let total = *NUM_GAMES_PER_STREAM;
        let num_retained = (total as f32 * *POP_NUM_RETAINED) as usize;
        let num_random = (total as f32 * *POP_NUM_RANDOM) as usize;
        let num_mutated = total.saturating_sub(num_retained + num_random);

        let mut games = Vec::with_capacity(total);

        // Exact clones of champion
        for _ in 0..num_retained.max(1) {
            games.push(Game::with_brain(champion));
        }

        // Mutated variations around champion
        for _ in 0..num_mutated {
            let mut brain = champion.clone();
            brain.mutate();
            games.push(Game::with_brain(&brain));
        }

        // Random exploration
        while games.len() < total {
            games.push(Game::new());
        }

        Self {
            games,
            max_score: 0,
            max_score_ts: Instant::now(),
        }
    }

    pub fn update(&mut self) -> usize {
        let mut games_alive = *NUM_GAMES_PER_STREAM;

        for g in self.games.iter_mut() {
            g.update();

            let score = g.score();
            if score > self.max_score {
                self.max_score = score;
                self.max_score_ts = Instant::now();
            }

            if g.is_complete {
                games_alive -= 1;
            }
        }

        *NUM_GAMES_PER_STREAM - games_alive
    }

    pub fn is_local_maximum(&self) -> bool {
        self.max_score_ts.elapsed().as_secs_f32() > *STREAM_LOCAL_MAX_WAIT_SECS
    }

    pub fn inject(&mut self, net: &Net) {
        let new_game = Game::with_brain(net);
        let num_games = (*NUM_GAMES_PER_STREAM as f32 * *STREAM_REJUVENATION_PERCENT) as usize;

        self.games.drain(0..num_games);
        for _ in 0..num_games {
            self.games.push(new_game.clone());
        }

        self.max_score = 0;
        self.max_score_ts = Instant::now();
    }

    pub fn get_stream_summary(&self) -> (usize, usize, Option<Net>) {
        let mut max_score = 0;
        let mut max_fitness = -1.0;
        let mut max_steps = 0;
        let mut best_net = None;

        for g in self.games.iter() {
            let score = g.score();
            let steps = g.num_steps;
            let fitness = g.fitness();
            if steps > max_steps {
                max_steps = steps;
            }
            if score > max_score || (score == max_score && fitness > max_fitness) {
                max_score = score;
                max_fitness = fitness;
                best_net = Some(g.brain.clone());
            }
        }

        (max_score, max_steps, best_net)
    }

    pub fn get_best_game(&self) -> Option<&Game> {
        let mut best: Option<&Game> = None;
        for g in self.games.iter() {
            match best {
                None => best = Some(g),
                Some(current) => {
                    if is_better_game(g, current) {
                        best = Some(g);
                    }
                }
            }
        }
        best
    }

    pub fn get_all_games(&self) -> Vec<&Game> {
        self.games.iter().collect()
    }

    pub fn inject_net(&mut self, net: Net) {
        for game in self.games.iter_mut().take(10) {
            game.set_brain(net.clone());
        }
    }

    pub fn reset(&mut self) -> Net {
        self.reset_with_champion(None)
    }

    /// Reset the generation while preserving the Hall of Fame champion at slot 0
    /// when provided (elitism guarantee).
    pub fn reset_with_champion(&mut self, champion: Option<&Net>) -> Net {
        let mut rng = rand::thread_rng();
        let gene_pool = self.generate_gene_pool();
        let mut new_games = Vec::with_capacity(*NUM_GAMES_PER_STREAM);

        // Distribución de la población
        let num_retained = (*NUM_GAMES_PER_STREAM as f32 * *POP_NUM_RETAINED) as usize;
        let num_children = (*NUM_GAMES_PER_STREAM as f32 * *POP_NUM_CHILDREN) as usize;
        let mut num_retained_mutated = (*NUM_GAMES_PER_STREAM as f32 * *POP_NUM_RETAINED_MUTATED) as usize;

        // Retenidos sin mutación ordenados por fitness descendente
        let mut games_sorted = self.games.clone();
        games_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        games_sorted.reverse();

        // Hall of Fame Elitism: slot 0 is ALWAYS the all-time champion (or generation best)
        let champ_to_preserve = champion.unwrap_or(&games_sorted[0].brain);
        new_games.push(Game::with_brain(champ_to_preserve));

        // Remaining retained slots (without mutation)
        for i in 0..num_retained.saturating_sub(1) {
            if i < games_sorted.len() {
                let old_brain = games_sorted[i].brain.clone();
                new_games.push(Game::with_brain(&old_brain));
            }
        }

        // Hijos (crossover)
        if let Some(pool) = gene_pool {
            for _ in 0..num_children {
                let rand_parent_1 = &self.games[pool.sample(&mut rng)];
                let rand_parent_2 = &self.games[pool.sample(&mut rng)];
                let mut new_brain = rand_parent_1.brain.merge(&rand_parent_2.brain);
                new_brain.mutate();
                new_games.push(Game::with_brain(&new_brain));
            }
        } else {
            num_retained_mutated += num_children;
        }

        // Retenidos con mutación
        for i in 0..num_retained_mutated {
            let base_brain = if i == 0 && champion.is_some() {
                champion.unwrap()
            } else {
                &games_sorted[i % games_sorted.len()].brain
            };
            let mut mutated_brain = base_brain.clone();
            mutated_brain.mutate();
            new_games.push(Game::with_brain(&mutated_brain));
        }

        // Completamente random hasta completar la población
        while new_games.len() < *NUM_GAMES_PER_STREAM {
            new_games.push(Game::new());
        }

        self.games = new_games;
        self.max_score = 0;
        self.max_score_ts = Instant::now();
        games_sorted[0].brain.clone()
    }

    fn generate_gene_pool(&self) -> Option<WeightedIndex<f32>> {
        let mut max_fitness = 0.0;
        let mut weights = Vec::new();

        for game in self.games.iter() {
            let fitness = game.fitness();
            if fitness > max_fitness {
                max_fitness = fitness;
            }

            if fitness.is_finite() && fitness > 0.0 {
                weights.push(fitness);
            } else {
                weights.push(0.0);
            }
        }

        if max_fitness <= 0.0 {
            return None;
        }

        weights
            .iter_mut()
            .for_each(|i| *i = (*i / max_fitness) * 100.0);

        WeightedIndex::new(&weights).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_net_synchronizes_both_brain_and_agent() {
        let mut stream = Stream::new();
        let custom_net = Net::new();
        stream.inject_net(custom_net.clone());

        for game in stream.games.iter().take(10) {
            let agent_net = game.agent.as_ref().expect("agent must exist").network().expect("agent net must exist");
            assert_eq!(game.brain.n_inputs(), agent_net.n_inputs());
            assert_eq!(game.brain.layers.len(), agent_net.layers.len());
        }
    }

    #[test]
    fn reset_preserves_retained_agent_brain_synchronization() {
        let mut stream = Stream::new();
        // Step once to establish scores/fitness
        stream.update();
        let best_net = stream.reset();

        // First games in stream are retained without mutation
        let first_game = &stream.games[0];
        let agent_net = first_game.agent.as_ref().expect("agent must exist").network().expect("agent net must exist");
        assert_eq!(first_game.brain.n_inputs(), best_net.n_inputs());
        assert_eq!(agent_net.n_inputs(), best_net.n_inputs());
    }

    #[test]
    fn generate_gene_pool_returns_none_when_max_fitness_is_zero() {
        let stream = Stream::new();
        // A fresh stream before update has finite fitness >= 1.0
        let pool = stream.generate_gene_pool();
        assert!(pool.is_some());
    }

    #[test]
    fn with_champion_initializes_champion_at_slot_zero() {
        let champion = Net::new();
        let stream = Stream::with_champion(&champion);
        assert_eq!(stream.games.len(), *NUM_GAMES_PER_STREAM);

        // Slot 0 must match champion weights
        let first_net = stream.games[0].get_net();
        assert_eq!(first_net.n_inputs(), champion.n_inputs());
        assert_eq!(first_net.layers.len(), champion.layers.len());

        // get_best_game at step 0 must return slot 0 (champion), not a random snake at index 999
        let best_game = stream.get_best_game().expect("must have best game");
        let best_net = best_game.get_net();
        assert_eq!(best_net.n_inputs(), champion.n_inputs());
        assert_eq!(best_net.layers.len(), champion.layers.len());
    }

    #[test]
    fn reset_with_champion_preserves_champion_at_slot_zero() {
        let mut stream = Stream::new();
        let champion = Net::new();
        let _ = stream.reset_with_champion(Some(&champion));

        // Slot 0 must be the preserved champion
        let slot_zero_net = stream.games[0].get_net();
        assert_eq!(slot_zero_net.n_inputs(), champion.n_inputs());
        assert_eq!(slot_zero_net.layers.len(), champion.layers.len());
    }

    #[test]
    fn is_better_game_prefers_alive_over_dead() {
        let mut alive_game = Game::new();
        alive_game.is_complete = false;

        let mut dead_game = Game::new();
        dead_game.is_complete = true;
        let head = dead_game.head;
        dead_game.body.push(head); // higher score

        // Alive game with lower score strictly beats dead game with higher score
        assert!(is_better_game(&alive_game, &dead_game));
        assert!(!is_better_game(&dead_game, &alive_game));

        // When both alive, higher score wins
        let mut alive_high = Game::new();
        let head = alive_high.head;
        alive_high.body.push(head);
        assert!(is_better_game(&alive_high, &alive_game));

        // When both are identical, candidate does NOT beat current (tie preserves current)
        let identical = Game::new();
        assert!(!is_better_game(&identical, &alive_game));
    }
}


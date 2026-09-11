//! Lógica del juego de la serpiente
//! Acciones de la serpiente desde una red neuronal

use crate::agent::{Agent, DqnPolicyAgent, GaAgent};
use crate::nn::Net;
use crate::snake_core::SnakeCore;
use crate::*;

#[derive(Clone)]
pub struct Game {
    /// Shared board physics (head, body, food, dir, swallow, step counters).
    pub core: SnakeCore,
    pub brain: Net,
    pub agent: Option<Box<dyn Agent>>,
    pub is_complete: bool,
    /// Cumulative score from eaten food based on freshness at the time of eating.
    pub food_freshness_score: f32,
}

// ---------------------------------------------------------------------------
// Deref – lets callers write `game.head`, `game.body`, `game.is_wall(pt)`, etc.
// ---------------------------------------------------------------------------

impl std::ops::Deref for Game {
    type Target = SnakeCore;
    #[inline]
    fn deref(&self) -> &SnakeCore {
        &self.core
    }
}

impl std::ops::DerefMut for Game {
    #[inline]
    fn deref_mut(&mut self) -> &mut SnakeCore {
        &mut self.core
    }
}

impl Game {
    pub fn new() -> Self {
        let brain = Net::new();
        Self {
            core: SnakeCore::new(),
            brain: brain.clone(),
            agent: Some(Box::new(GaAgent::new(brain))),
            is_complete: false,
            food_freshness_score: 0.0,
        }
    }

    pub fn update(&mut self) {
        if self.is_complete {
            return;
        }

        self.core.num_steps += 1;
        self.core.dir = self.get_brain_output();
        self.handle_food_collision();
        self.update_snake_positions();
        self.handle_step_limit();
        if self.core.is_wall(self.core.head) || self.core.is_snake_body(self.core.head) {
            self.is_complete = true;
        }
    }

    pub fn get_net_output(&self) -> Vec<Vec<f64>> {
        if let Some(agent) = &self.agent {
            if let Some(out) = agent.network_output(self) {
                return out;
            }
        }
        let vision = self.get_snake_vision();
        self.brain.predict(&vision)
    }

    pub fn get_net(&self) -> &Net {
        &self.brain
    }

    fn get_brain_output(&self) -> FourDirs {
        if let Some(agent) = &self.agent {
            agent.decide_direction(self)
        } else {
            let ga = GaAgent::new(self.brain.clone());
            ga.decide_direction(self)
        }
    }

    fn get_snake_vision(&self) -> Vec<f64> {
        // self.get_11_vision()
        // self.get_custom_vision()
        // self.get_eight_dir_vision()
        self.core.get_four_dir_vision()
    }

    /// Delegated to SnakeCore – kept as a public method for backward compat.
    pub fn get_four_dir_vision(&self) -> Vec<f64> {
        self.core.get_four_dir_vision()
    }

    pub fn fitness(&self) -> f32 {
        let apples = self.core.body.len().saturating_sub(1);
        let total_steps = self.core.num_steps as f32;

        if apples == 0 {
            // Gen 0: Premia supervivencia y navegación sin chocar
            return (1.0 + total_steps).max(1.0);
        }

        // Puntuación acumulada de manzanas según su frescura al comerlas
        // Multiplicador progresivo según longitud corporal (crecimiento cuadrático/polinomial)
        let food_factor = 1.0 + (apples as f32 * 0.1);
        let food_fitness = self.food_freshness_score * food_factor;

        // Supervivencia cautelosa: suma pasos pero descuenta la mitad de los pasos
        // ociosos sin comer al momento de morir (no premia la inanición vacía).
        let survival = total_steps - (self.core.steps_without_food as f32 * 0.5);

        (food_fitness + survival).max(1.0)
    }

    pub fn score(&self) -> usize {
        self.core.body.len()
    }

    /// Build a game with any arbitrary agent implementing [`Agent`].
    pub fn with_agent(agent: Box<dyn Agent>) -> Self {
        let mut new_game = Self::new();
        if let Some(net) = agent.network() {
            new_game.brain = net.clone();
        }
        new_game.agent = Some(agent);
        new_game
    }

    pub fn with_brain(new_brain: &Net) -> Self {
        Self::with_agent(Box::new(GaAgent::new(new_brain.clone())))
    }

    /// Build a game whose brain is a DQN relative-action net (9-input,
    /// 3-output): vision and actions are interpreted in the heading-relative
    /// frame. Used by the versus/cross arena for DQN players.
    pub fn with_relative_brain(new_brain: &Net) -> Self {
        Self::with_agent(Box::new(DqnPolicyAgent::new(new_brain.clone())))
    }

    // -----------------------------------------------------------------------
    // Private helpers (GA movement model — different from GameDQN body-insert)
    // -----------------------------------------------------------------------

    fn handle_food_collision(&mut self) {
        if self.core.head != self.core.food {
            self.core.steps_without_food += 1;
            self.core.swallow.advance(self.core.body.len());
            return;
        }

        // Calculate freshness of food before resetting steps_without_food
        let freshness = self.core.food_freshness();
        let apple_value = 400.0 + 600.0 * freshness;
        self.food_freshness_score += apple_value;

        self.core.swallow.push_eating();
        self.core.body.push(Point::new(self.core.head.x, self.core.head.y));
        self.core.respawn_food();
    }

    fn handle_step_limit(&mut self) {
        let limit = self.core.hunger_limit();
        if self.core.steps_without_food >= limit {
            self.core.respawn_food();
        }
    }

    /// GA-style movement: update head then shift body forward using prev-pos chain.
    fn update_snake_positions(&mut self) {
        self.core.head.x += self.core.dir.value().0;
        self.core.head.y += self.core.dir.value().1;

        let mut prev_pos = self.core.head.clone();
        for p in self.core.body.iter_mut() {
            let new_pos = *p;
            *p = prev_pos;
            prev_pos = new_pos;
        }
    }

    /// Compute the 12-element absolute observation matching GameDQN::get_state
    /// exactly: (wall_dist, food_projection, body_dist) across [Left, Right, Bottom, Top].
    pub fn get_relative_state(&self) -> Vec<f64> {
        self.core.get_relative_state()
    }

    pub fn render(&self) {
        for x in 0..=GRID_W {
            for y in 0..=GRID_H {
                let pt = (x, y).into();
                if self.core.is_wall(pt) {
                    print!("□");
                    continue;
                }
                if self.core.is_snake_body(pt) {
                    print!("■");
                    continue;
                }
                if self.core.head == pt {
                    print!("■");
                }
                if self.core.food == pt {
                    print!("●");
                }
                print!(".");
            }
            println!();
        }
        println!();
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.fitness() == other.fitness()
    }
}

impl PartialOrd for Game {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.fitness().partial_cmp(&other.fitness())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_dqn::GameDQN;

    #[test]
    fn relative_brain_observation_matches_game_dqn_observation() {
        let dummy_net = Net::new();
        let mut game = Game::with_relative_brain(&dummy_net);
        let mut dqn = GameDQN::new();

        // Configure identical positions
        let head = Point::new(10, 10);
        let body = vec![head, Point::new(10, 11), Point::new(10, 12)];
        let food = Point::new(15, 14); // diagonal food
        let dir = FourDirs::Top;

        game.head = head;
        game.body = body.clone();
        game.food = food;
        game.dir = dir;

        dqn.head = head;
        dqn.body = body;
        dqn.food = food;
        dqn.dir = dir;

        let game_abs = game.get_relative_state();
        let game_rel = rotate_vision_to_relative(&game_abs, game.dir);
        let dqn_obs = dqn.observation();

        assert_eq!(game_rel.len(), 9);
        assert_eq!(dqn_obs.len(), 9);

        for (i, (g, d)) in game_rel.iter().zip(dqn_obs.iter()).enumerate() {
            assert!(
                (g - d).abs() < 1e-6,
                "Mismatch at feature index {i}: Game relative={g}, GameDQN obs={d}"
            );
        }
    }

    #[test]
    fn relative_brain_body_distance_zero_when_no_body_present() {
        let dummy_net = Net::new();
        let mut game = Game::with_relative_brain(&dummy_net);
        let mut dqn = GameDQN::new();

        // 1-segment snake in middle of grid
        let head = Point::new(15, 15);
        let body = vec![head];
        let food = Point::new(20, 15);
        let dir = FourDirs::Right;

        game.head = head;
        game.body = body.clone();
        game.food = food;
        game.dir = dir;

        dqn.head = head;
        dqn.body = body;
        dqn.food = food;
        dqn.dir = dir;

        let game_obs = rotate_vision_to_relative(&game.get_relative_state(), game.dir);
        let dqn_obs = dqn.observation();

        // Body distance features are indices 2, 5, 8
        assert_eq!(game_obs[2], 0.0, "Body dist forward must be 0.0");
        assert_eq!(game_obs[5], 0.0, "Body dist left must be 0.0");
        assert_eq!(game_obs[8], 0.0, "Body dist right must be 0.0");
        assert_eq!(game_obs, dqn_obs);
    }

    #[test]
    fn handle_step_limit_respawns_food_and_resets_steps_without_killing_snake() {
        let dummy_net = Net::new();
        let mut game = Game::with_relative_brain(&dummy_net);

        // Body len = 1 (start): hunger limit is 100
        assert_eq!(game.core.hunger_limit(), 100);
        let initial_food = game.core.food;
        game.core.steps_without_food = 99;
        game.handle_step_limit();
        assert!(!game.is_complete);
        assert_eq!(game.core.steps_without_food, 99);
        assert_eq!(game.core.food, initial_food);

        game.core.steps_without_food = 100;
        game.handle_step_limit();
        // Snake must NOT die
        assert!(!game.is_complete);
        // Steps without food must reset to 0
        assert_eq!(game.core.steps_without_food, 0);

        // Reset and test higher size (snake size = 25 -> limit is 300)
        game.core.body = vec![Point::new(0, 0); 25];
        assert_eq!(game.core.hunger_limit(), 300);

        game.core.steps_without_food = 299;
        game.handle_step_limit();
        assert!(!game.is_complete);
        assert_eq!(game.core.steps_without_food, 299);

        game.core.steps_without_food = 300;
        game.handle_step_limit();
        assert!(!game.is_complete);
        assert_eq!(game.core.steps_without_food, 0);
    }

    #[test]
    fn eating_food_accumulates_freshness_score() {
        let mut game = Game::new();
        assert_eq!(game.food_freshness_score, 0.0);

        game.core.head = game.core.food;
        game.core.steps_without_food = 0;
        game.handle_food_collision();

        assert!((game.food_freshness_score - 1000.0).abs() < 1e-4);

        game.core.head = game.core.food;
        game.core.steps_without_food = 50;
        game.handle_food_collision();

        assert!((game.food_freshness_score - 1700.0).abs() < 1e-4);
    }

    #[test]
    fn fitness_rewards_fresh_food_more_than_rotting_food() {
        let mut fresh_game = Game::new();
        fresh_game.core.head = fresh_game.core.food;
        fresh_game.core.steps_without_food = 5;
        fresh_game.core.num_steps = 20;
        fresh_game.handle_food_collision();

        let mut rotting_game = Game::new();
        rotting_game.core.head = rotting_game.core.food;
        rotting_game.core.steps_without_food = 95;
        rotting_game.core.num_steps = 110;
        rotting_game.handle_food_collision();

        assert!(fresh_game.fitness() > rotting_game.fitness());
    }

    #[test]
    fn fitness_gen0_rewards_survival_steps() {
        let mut game_early = Game::new();
        game_early.core.num_steps = 5;

        let mut game_late = Game::new();
        game_late.core.num_steps = 60;

        assert!(game_late.fitness() > game_early.fitness());
    }

    #[test]
    fn fitness_eating_rotting_food_beats_starvation() {
        let mut game_starved = Game::new();
        game_starved.core.num_steps = 100;
        game_starved.core.steps_without_food = 100;

        let mut game_eaten = Game::new();
        game_eaten.core.head = game_eaten.core.food;
        game_eaten.core.steps_without_food = 95;
        game_eaten.core.num_steps = 100;
        game_eaten.handle_food_collision();

        assert!(game_eaten.fitness() > game_starved.fitness() * 5.0);
    }

    #[test]
    fn fitness_does_not_overflow_f32_on_large_scores() {
        let mut game = Game::new();
        game.core.body = vec![Point::new(0, 0); 100];
        game.core.num_steps = 10_000;
        game.food_freshness_score = 100.0 * 800.0;
        let fit = game.fitness();
        assert!(fit.is_finite());
        assert!(fit > 0.0);
    }
}


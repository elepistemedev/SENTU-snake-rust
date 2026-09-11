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
        let score = self.core.body.len() as f32;
        if score <= 1.0 {
            return 1.0;
        }

        if score < 5.0 {
            return (self.core.num_steps as f32 * 0.1) * (2.0 as f32).powf(score) * score;
        }

        let mut fitness = 1.0;
        fitness *= (2.0 as f32).powf(score) * score;
        fitness *= self.core.num_steps as f32;

        // TODO f32 shouldn't work as it can't hold such a big value
        // This is broken
        fitness
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

        self.core.swallow.push_eating();
        self.core.body.push(Point::new(self.core.head.x, self.core.head.y));
        self.core.food = self.core.get_random_empty_pos();
        self.core.steps_without_food = 0;
    }

    fn handle_step_limit(&mut self) {
        let limit = match self.score() {
            score if score > 10 => NUM_SIM_STEPS * 2,
            score if score > 20 => NUM_SIM_STEPS * 3,
            score if score > 30 => NUM_SIM_STEPS * 5,
            score if score > 80 => NUM_SIM_STEPS * 8,
            _ => NUM_SIM_STEPS,
        };

        if self.core.steps_without_food >= limit {
            self.is_complete = true;
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
}

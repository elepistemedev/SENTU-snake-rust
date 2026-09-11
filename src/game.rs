//! Lógica del juego de la serpiente
//! Acciones de la serpiente desde una red neuronal

use crate::agent::{Agent, DqnPolicyAgent, GaAgent};
use crate::nn::Net;
use crate::*;

#[derive(Clone)]
pub struct Game {
    pub head: Point,
    pub body: Vec<Point>,
    pub food: Point,
    pub dir: FourDirs,
    pub brain: Net,
    pub agent: Option<Box<dyn Agent>>,

    pub is_complete: bool,
    pub swallow: crate::render_snake::SwallowTracker,
    no_food_steps: usize,
    pub num_steps: usize,
}

impl Game {
    pub fn new() -> Self {
        let mut body = Vec::new();
        let head = Point::new(GRID_W / 2, GRID_H / 2);
        body.push(head.clone());
        let brain = Net::new();

        Self {
            body,
            head,
            food: Point::rand(),
            dir: FourDirs::get_rand_dir(),
            brain: brain.clone(),
            agent: Some(Box::new(GaAgent::new(brain))),
            is_complete: false,
            swallow: crate::render_snake::SwallowTracker::new(),
            no_food_steps: 0,
            num_steps: 0,
        }
    }

    pub fn update(&mut self) {
        if self.is_complete {
            return;
        }

        self.num_steps += 1;
        self.dir = self.get_brain_output();
        self.handle_food_collision();
        self.update_snake_positions();
        self.handle_step_limit();
        if self.is_wall(self.head) || self.is_snake_body(self.head) {
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
        self.get_four_dir_vision()
    }

    pub fn get_four_dir_vision(&self) -> Vec<f64> {
        let mut vision = Vec::new();
        let dirs = FourDirs::get_all_dirs();

        for d in dirs {
            let (wall, food, body) = self.look_in_dir(self.head, d);
            vision.push(wall as f64);
            vision.push(if food { 1.0 } else { 0.0 });
            vision.push(body as f64);
        }

        vision
    }

    pub fn fitness(&self) -> f32 {
        let score = self.body.len() as f32;
        if score <= 1.0 {
            return 1.0;
        }

        if score < 5.0 {
            return (self.num_steps as f32 * 0.1) * (2.0 as f32).powf(score) * score;
        }

        let mut fitness = 1.0;
        fitness *= (2.0 as f32).powf(score) * score;
        fitness *= self.num_steps as f32;

        // TODO f32 shouldn't work as it can't hold such a big value
        // This is broken
        fitness
    }

    pub fn score(&self) -> usize {
        self.body.len()
    }

    pub fn is_wall(&self, pt: Point) -> bool {
        pt.x >= GRID_W || pt.x <= 0 || pt.y >= GRID_H || pt.y <= 0
    }

    pub fn is_snake_body(&self, pt: Point) -> bool {
        for p in self.body.iter().skip(1) {
            if pt == *p {
                return true;
            }
        }

        false
    }

    fn update_snake_positions(&mut self) {
        self.head.x += self.dir.value().0;
        self.head.y += self.dir.value().1;

        let mut prev_pos = self.head.clone();
        for p in self.body.iter_mut() {
            let new_pos = *p;
            *p = prev_pos;
            prev_pos = new_pos;
        }
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

    fn handle_food_collision(&mut self) {
        if self.head != self.food {
            self.no_food_steps += 1;
            self.swallow.advance(self.body.len());
            return;
        }

        self.swallow.push_eating();
        self.body.push(Point::new(self.head.x, self.head.y));
        self.food = self.get_random_empty_pos();
        self.no_food_steps = 0;
    }

    fn handle_step_limit(&mut self) {
        let limit = match self.score() {
            score if score > 10 => NUM_SIM_STEPS * 2,
            score if score > 20 => NUM_SIM_STEPS * 3,
            score if score > 30 => NUM_SIM_STEPS * 5,
            score if score > 80 => NUM_SIM_STEPS * 8,
            _ => NUM_SIM_STEPS,
        };

        if self.no_food_steps >= limit {
            self.is_complete = true;
        }
    }

    fn get_random_empty_pos(&self) -> Point {
        let mut pt = Point::rand();

        let mut num_tries = 0;
        while num_tries < 5 {
            num_tries += 1;
            pt = Point::rand();

            if !self.body.contains(&pt) {
                break;
            }
        }

        pt
    }

    /// Compute the 12-element absolute observation matching GameDQN::get_state
    /// exactly: (wall_dist, food_projection, body_dist) across [Left, Right, Bottom, Top].
    pub fn get_relative_state(&self) -> Vec<f64> {
        let mut state = Vec::with_capacity(12);
        let dirs = FourDirs::get_all_dirs();

        let dx = (self.food.x - self.head.x) as f64;
        let dy = (self.food.y - self.head.y) as f64;
        let food_dist = (dx * dx + dy * dy).sqrt();
        let (unit_fx, unit_fy) = if food_dist > 0.0 {
            (dx / food_dist, dy / food_dist)
        } else {
            (0.0, 0.0)
        };

        for d in dirs {
            let (wall, food_on_ray, body) = self.look_in_dir_dqn(self.head, d);
            state.push(wall);
            let proj = unit_fx * d.0 as f64 + unit_fy * d.1 as f64;
            let food_val = if food_on_ray {
                1.0
            } else {
                proj.max(0.0)
            };
            state.push(food_val);
            state.push(body);
        }

        state
    }

    fn look_in_dir_dqn(&self, from: Point, dir: (i32, i32)) -> (f64, bool, f64) {
        let mut distance = 1.0;
        let mut food_found = false;
        let mut body_distance = f64::INFINITY;

        let mut current = Point::new(from.x + dir.0, from.y + dir.1);

        while !self.is_wall(current) {
            if current == self.food {
                food_found = true;
            }
            if self.is_snake_body(current) && body_distance == f64::INFINITY {
                body_distance = distance;
            }

            current.x += dir.0;
            current.y += dir.1;
            distance += 1.0;
        }

        let wall_dist = 1.0 / distance;
        let body_dist = if body_distance == f64::INFINITY {
            0.0
        } else {
            1.0 / body_distance
        };

        (wall_dist, food_found, body_dist)
    }

    fn look_in_dir(&self, st: Point, dir: (i32, i32)) -> (f32, bool, f32) {
        let mut food = false;
        // let mut body = false;
        let mut temp_pt: Point = st;
        let mut dist = 0;

        loop {
            if self.is_wall(temp_pt) {
                break;
            }

            if self.food == temp_pt {
                food = true;
            }

            if self.is_snake_body(temp_pt) {
                // body = true;
                break;
            }

            temp_pt = Point::new(temp_pt.x + dir.0, temp_pt.y + dir.1);

            dist += 1;
            if dist > 1000 {
                break;
            }
        }

        (1.0 / dist as f32, food, 1.0 / dist as f32)
    }

    pub fn render(&self) {
        for x in 0..=GRID_W {
            for y in 0..=GRID_H {
                let pt = (x, y).into();
                if self.is_wall(pt) {
                    print!("□");
                    continue;
                }
                if self.is_snake_body(pt) {
                    print!("■");
                    continue;
                }
                if self.head == pt {
                    print!("■");
                }
                if self.food == pt {
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


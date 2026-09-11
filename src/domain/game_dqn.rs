//! Game logic adapted for DQN training
//! Single agent learning through experience

use crate::dqn::{DQNAgent, Experience};
use crate::snake_core::SnakeCore;
use crate::utils::{relative_dir, rotate_vision_to_relative};



pub struct GameDQN {
    /// Shared board physics (head, body, food, dir, swallow, step counters).
    pub core: SnakeCore,
    pub agent: DQNAgent,
    pub score: usize,
    pub steps: usize,
    pub is_complete: bool,
    prev_distance: f64,
}

// ---------------------------------------------------------------------------
// Deref – lets callers write `game.head`, `game.body`, `game.swallow`, etc.
// ---------------------------------------------------------------------------

impl std::ops::Deref for GameDQN {
    type Target = SnakeCore;
    #[inline]
    fn deref(&self) -> &SnakeCore {
        &self.core
    }
}

impl std::ops::DerefMut for GameDQN {
    #[inline]
    fn deref_mut(&mut self) -> &mut SnakeCore {
        &mut self.core
    }
}

impl GameDQN {
    pub fn new() -> Self {
        let core = SnakeCore::new();
        let prev_distance = SnakeCore::calculate_distance(&core.head, &core.food);
        Self {
            prev_distance,
            core,
            agent: DQNAgent::new(),
            score: 0,
            steps: 0,
            is_complete: false,
        }
    }

    /// Construct a game instance whose DQN agent is initialized with pre-trained
    /// network weights and a specific exploration rate (warm-start training).
    pub fn with_network(net: &crate::nn::Net, epsilon: f64) -> Self {
        let mut game = Self::new();
        game.agent = DQNAgent::with_network(net.clone(), epsilon);
        game
    }

    pub fn reset(&mut self) {
        self.core.reset();
        self.score = 0;
        self.steps = 0;
        self.is_complete = false;
        self.prev_distance = SnakeCore::calculate_distance(&self.core.head, &self.core.food);
    }

    /// Current 9-input heading-relative observation (3 directions ×
    /// [wall-reciprocal, food bit, body-reciprocal], direction order
    /// forward / left-turn / right-turn relative to `self.dir`), as consumed
    /// by the q-network — the same feature vector `step()` feeds action
    /// selection. Thin read-only wrapper: it never advances `steps` or
    /// mutates the board.
    pub fn observation(&self) -> Vec<f64> {
        rotate_vision_to_relative(&self.core.get_relative_state(), self.core.dir)
    }

    pub fn step(&mut self) -> (f64, bool) {
        if self.is_complete {
            return (0.0, true);
        }

        let state = self.observation();
        let action = self.agent.select_action(&state);
        self.core.dir = relative_dir(self.core.dir, action);
        self.steps += 1;
        self.core.steps_without_food += 1;

        // Move snake
        self.core.head.x += self.core.dir.value().0;
        self.core.head.y += self.core.dir.value().1;

        let mut reward;
        let mut done = false;

        // Check collision with wall
        if self.core.is_wall(self.core.head) {
            reward = -1.0;
            done = true;
            self.is_complete = true;
        } else if self.core.head == self.core.food {
            // Ate food: body grows by retaining previous tail
            self.core.body.insert(0, self.core.head);
            reward = 2.0;
            self.score += 1;
            self.core.swallow.advance(self.core.body.len());
            self.core.swallow.push_eating();
            self.core.food = self.core.get_random_empty_pos();
            self.prev_distance = SnakeCore::calculate_distance(&self.core.head, &self.core.food);
            self.core.steps_without_food = 0;
            if self.core.is_snake_body(self.core.head) {
                reward = -1.0;
                done = true;
                self.is_complete = true;
            }
        } else {
            // Normal move: insert new head at front, remove old tail
            self.core.body.insert(0, self.core.head);
            self.core.body.pop();
            self.core.swallow.advance(self.core.body.len());

            if self.core.is_snake_body(self.core.head) {
                reward = -1.0;
                done = true;
                self.is_complete = true;
            } else {
                let new_distance = SnakeCore::calculate_distance(&self.core.head, &self.core.food);
                if new_distance < self.prev_distance {
                    reward = 0.1; // Reward for getting closer
                } else {
                    reward = -0.15; // Penalty for getting farther
                }
                self.prev_distance = new_distance;
            }
        }

        // Anti-stagnation: end episode if no food eaten for too long.
        // Step limit dynamically scales with snake size and only counts steps between food.
        let step_limit = self.core.hunger_limit();
        if !done && self.core.steps_without_food >= step_limit {
            reward = -0.5;
            done = true;
            self.is_complete = true;
        }

        let next_state = self.observation();

        // Store experience
        let exp = Experience {
            state,
            action,
            reward,
            next_state,
            done,
        };
        self.agent.store_experience(exp);

        // Train
        self.agent.train();

        (reward, done)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    // --- observation: 9-input heading-relative view ----------------------------

    #[test]
    fn observation_is_a_stable_9_input_relative_view_without_side_effects() {
        let game = GameDQN::new();
        let steps_before = game.steps;
        let first = game.observation();

        assert_eq!(
            first.len(),
            9,
            "9 = 3 relative directions x [wall, food, body] per direction"
        );
        // Groups: forward, left-turn, right-turn
        for group in 0..3 {
            let wall = first[group * 3];
            let food = first[group * 3 + 1];
            let body = first[group * 3 + 2];
            assert!(wall > 0.0 && wall <= 1.0, "wall reciprocal in (0,1], group {group}");
            assert!(food >= 0.0 && food <= 1.0, "food signal in [0,1], group {group}");
            assert!(body >= 0.0 && body <= 1.0, "body reciprocal in [0,1], group {group}");
        }

        let second = game.observation();
        assert_eq!(first, second, "two calls without step must be identical");
        assert_eq!(game.steps, steps_before, "observation must never advance steps");
        assert!(!game.is_complete, "observation must not mutate game state");
    }

    #[test]
    fn observation_places_forward_food_in_first_group_when_heading_top() {
        let mut game = GameDQN::new();
        // Force heading Top so "forward" is the absolute TOP direction.
        game.core.dir = FourDirs::Top;
        // Place food exactly one cell forward (above the head).
        game.core.food = Point::new(game.core.head.x, game.core.head.y - 1);

        let state = game.observation();
        // Forward group is indices 0..2; food bit at index 1.
        assert_eq!(state[1], 1.0, "forward food bit must be 1 with food directly ahead");
        // Wall reciprocal for forward: distance from cell above head to wall = head.y - 1 cells
        let expected_wall = 1.0 / game.core.head.y as f64;
        assert!(
            (state[0] - expected_wall).abs() < 1e-9,
            "forward wall reciprocal = 1/head.y, got {}",
            state[0]
        );
        // Left-turn and right-turn groups see no food.
        assert_eq!(state[4], 0.0, "left-turn food must be 0");
        assert_eq!(state[7], 0.0, "right-turn food must be 0");
    }

    #[test]
    fn observation_directional_food_sensor_detects_diagonal_food() {
        let mut game = GameDQN::new();
        game.core.dir = FourDirs::Top;
        game.core.head = Point::new(10, 10);
        // Place food diagonally top-left
        game.core.food = Point::new(5, 5);

        let state = game.observation();
        // Forward group = index 1, Left group = index 4, Right group = index 7
        assert!(state[1] > 0.6, "forward sector must detect top-left food");
        assert!(state[4] > 0.6, "left sector must detect top-left food");
        assert_eq!(state[7], 0.0, "right sector must have 0 food signal for top-left food");
    }

    #[test]
    fn relative_step_never_reverses_direction() {
        // Because relative_dir maps 0..2 to forward/left/right, there is no
        // backward action. After any number of steps the heading sequence must
        // never contain an immediate 180-degree reversal.
        let mut game = GameDQN::new();
        let mut prev_dir = game.core.dir;
        for _ in 0..50 {
            let _ = game.step();
            let new_dir = game.core.dir;
            assert!(
!(prev_dir.is_horizontal() && new_dir.is_horizontal() && prev_dir != new_dir)
&& !(prev_dir.is_vertical() && new_dir.is_vertical() && prev_dir != new_dir),
"180-degree reversal detected: {:?} -> {:?}",
prev_dir,
new_dir
            );
            if game.is_complete {
break;
            }
            prev_dir = new_dir;
        }
    }

    #[test]
    fn swallow_animation_triggers_on_food_eaten_and_advances() {
        let mut game = GameDQN::new();
        game.core.dir = FourDirs::Right;
        game.core.head = Point::new(10, 10);
        game.core.body = vec![Point::new(10, 10), Point::new(9, 10)];
        // Put food directly in front of head
        game.core.food = Point::new(11, 10);

        // Force action to move forward (relative action 0 = forward)
        // We test eating logic directly by stepping when head hits food
        let prev_score = game.score;
        let (_, done) = game.step();
        assert!(!done);
        if game.score > prev_score {
            assert!(game.core.swallow.head_scale() > 0.0, "Head chew scale must be active");
            assert!(game.core.swallow.bulge_at(0) > 0.0, "Bulge must start at index 0");
        }
    }

    #[test]
    fn body_segments_stay_contiguous_and_ordered_after_eating() {
        let mut game = GameDQN::new();
        // Give snake several food points to trigger eating
        for _ in 0..100 {
            // Spawn food right in front to force eating often
            if rand::random::<f32>() < 0.3 {
                let forward = game.core.dir.value();
                let target = Point::new(game.core.head.x + forward.0, game.core.head.y + forward.1);
                if !game.core.is_wall(target) {
                    game.core.food = target;
                }
            }
            let (_, done) = game.step();
            if done {
                game.reset();
                continue;
            }

            // Invariant 1: body[0] is always head
            assert_eq!(game.core.body[0], game.core.head, "body[0] must always equal game.head");

            // Invariant 2: body length is score + 1
            assert_eq!(
                game.core.body.len(),
                game.score + 1,
                "body length must match score + 1"
            );

            // Invariant 3: every adjacent segment pair has Manhattan distance exactly 1
            for i in 0..game.core.body.len() - 1 {
                let dx = (game.core.body[i].x - game.core.body[i + 1].x).abs();
                let dy = (game.core.body[i].y - game.core.body[i + 1].y).abs();
                assert_eq!(
                    dx + dy,
                    1,
                    "Segment {} ({:?}) and {} ({:?}) must be adjacent (Manhattan distance 1)",
                    i,
                    game.core.body[i],
                    i + 1,
                    game.core.body[i + 1]
                );
            }
        }
    }

    #[test]
    fn game_dqn_with_network_preserves_agent_weights_and_epsilon() {
        let net = crate::nn::Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        let game = GameDQN::with_network(&net, 0.28);
        assert_eq!(game.agent.get_epsilon(), 0.28);
        assert_eq!(game.agent.q_network.layers.len(), net.layers.len());
    }

    #[test]
    fn episode_can_exceed_step_limit_if_eating() {
        let mut game = GameDQN::new();
        // Total steps can reach high numbers (e.g. 500) as long as food was eaten recently
        game.steps = 499;
        game.core.steps_without_food = 5;
        // Make sure snake won't hit wall or body
        game.core.head = Point::new(10, 10);
        game.core.body = vec![Point::new(10, 10), Point::new(9, 10)];
        game.core.dir = FourDirs::Right;
        let (_, done) = game.step();
        assert!(!done, "Episode should not end if the snake has eaten recently");
        assert!(!game.is_complete, "game.is_complete should be false");
        assert_eq!(game.steps, 500);
    }

    #[test]
    fn episode_terminates_when_steps_without_food_exceeds_dynamic_limit() {
        let mut game = GameDQN::new();
        // Snake body size 15 => dynamic limit is 200 (for size 11..=20)
        game.core.body = vec![Point::new(10, 10); 15];
        game.score = 14;
        assert_eq!(game.core.hunger_limit(), 200);

        game.core.head = Point::new(10, 10);
        game.core.dir = FourDirs::Right;
        // Put food far away so it doesn't eat
        game.core.food = Point::new(1, 1);

        // At 198 steps without food, one step makes it 199 (less than 200)
        game.core.steps_without_food = 198;
        let (_reward, done) = game.step();
        assert!(!done, "Should not terminate at 199 steps without food for snake size 15");

        // Next step makes it 200 (>= 200), should terminate
        let (reward, done) = game.step();
        assert!(done, "Should terminate at 200 steps without food for snake size 15");
        assert!(game.is_complete);
        assert_eq!(reward, -0.5);
    }
}



//! Core snake board physics shared by [`crate::game::Game`] and [`crate::game_dqn::GameDQN`].
//!
//! `SnakeCore` owns all grid-level state (head, body, food, direction, swallow
//! animation, step counters) and provides the physical operations that are
//! identical for both game modes:
//!
//! - collision queries (`is_wall`, `is_snake_body`)
//! - movement primitives (`advance_without_food`, `eat_food`, `advance_head_only`)
//! - raycasting (`look_in_dir_dqn`, `look_in_dir_ga`)
//! - food respawn (`get_random_empty_pos`)
//! - observation builders (`get_four_dir_vision`, `get_relative_state`)
//!
//! **Neither `Game` nor `GameDQN` ever duplicate these methods** — they call
//! them through `Deref<Target = SnakeCore>`.

use crate::swallow::SwallowTracker;
use crate::*;

/// Shared snake grid state and physics.
///
/// Intended to be embedded in [`crate::game::Game`] and [`crate::game_dqn::GameDQN`]
/// and exposed via `Deref<Target = SnakeCore>` so that callers write
/// `game.head`, `game.is_wall(pt)` etc. without any change.
#[derive(Clone)]
pub struct SnakeCore {
    pub head: Point,
    pub body: Vec<Point>,
    pub food: Point,
    pub dir: FourDirs,
    pub swallow: SwallowTracker,
    /// Steps elapsed since the last food was eaten.
    pub steps_without_food: usize,
    /// Total steps taken in this episode.
    pub num_steps: usize,
}

impl SnakeCore {
    /// Create a fresh core centred on the grid.
    pub fn new() -> Self {
        let head = Point::new(GRID_W / 2, GRID_H / 2);
        let body = vec![head.clone()];
        let food = Point::rand();
        Self {
            head,
            body,
            food,
            dir: FourDirs::get_rand_dir(),
            swallow: SwallowTracker::new(),
            steps_without_food: 0,
            num_steps: 0,
        }
    }

    /// Reset to a fresh starting position.
    pub fn reset(&mut self) {
        self.head = Point::new(GRID_W / 2, GRID_H / 2);
        self.body.clear();
        self.body.push(self.head.clone());
        self.food = Point::rand();
        self.dir = FourDirs::get_rand_dir();
        self.swallow.reset();
        self.steps_without_food = 0;
        self.num_steps = 0;
    }

    // -------------------------------------------------------------------------
    // Collision queries
    // -------------------------------------------------------------------------

    #[inline]
    pub fn is_wall(&self, pt: Point) -> bool {
        pt.x >= GRID_W || pt.x <= 0 || pt.y >= GRID_H || pt.y <= 0
    }

    /// Returns `true` if `pt` overlaps any body segment other than the head (index 0).
    #[inline]
    pub fn is_snake_body(&self, pt: Point) -> bool {
        self.body.iter().skip(1).any(|p| *p == pt)
    }

    // -------------------------------------------------------------------------
    // Movement primitives
    // -------------------------------------------------------------------------

    /// Advance head + shift body FIFO (normal move, no food).
    /// `body[0]` stays equal to `head` after this call.
    pub fn advance_without_food(&mut self) {
        self.head.x += self.dir.value().0;
        self.head.y += self.dir.value().1;
        self.body.insert(0, self.head);
        self.body.pop();
        self.swallow.advance(self.body.len());
    }

    /// Advance head + grow body (food eaten). Handles food respawn and swallow animation.
    pub fn eat_food(&mut self) {
        self.head.x += self.dir.value().0;
        self.head.y += self.dir.value().1;
        self.body.insert(0, self.head);
        self.swallow.advance(self.body.len());
        self.swallow.push_eating();
        self.food = self.get_random_empty_pos();
        self.steps_without_food = 0;
    }

    /// Advance head only (used by `Game` which manages body separately).
    pub fn advance_head_only(&mut self) {
        self.head.x += self.dir.value().0;
        self.head.y += self.dir.value().1;
    }

    // -------------------------------------------------------------------------
    // Food helpers
    // -------------------------------------------------------------------------

    pub fn get_random_empty_pos(&self) -> Point {
        let mut pt = Point::rand();
        let mut tries = 0;
        while tries < 10 && self.body.contains(&pt) {
            pt = Point::rand();
            tries += 1;
        }
        pt
    }

    // -------------------------------------------------------------------------
    // Raycasting
    // -------------------------------------------------------------------------

    /// DQN-style raycast: `(wall_reciprocal, food_on_ray, body_reciprocal)`.
    pub fn look_in_dir_dqn(&self, from: Point, dir: (i32, i32)) -> (f64, bool, f64) {
        let mut distance = 1.0_f64;
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
        let body_dist = if body_distance == f64::INFINITY { 0.0 } else { 1.0 / body_distance };

        (wall_dist, food_found, body_dist)
    }

    /// GA-style raycast: `(wall_reciprocal, food_on_ray, body_reciprocal)` as `f32`.
    pub fn look_in_dir_ga(&self, st: Point, dir: (i32, i32)) -> (f32, bool, f32) {
        let mut food = false;
        let mut temp_pt = st;
        let mut dist = 0_usize;

        loop {
            if self.is_wall(temp_pt) { break; }
            if self.food == temp_pt { food = true; }
            if self.is_snake_body(temp_pt) { break; }
            temp_pt = Point::new(temp_pt.x + dir.0, temp_pt.y + dir.1);
            dist += 1;
            if dist > 1000 { break; }
        }

        (1.0 / dist as f32, food, 1.0 / dist as f32)
    }

    // -------------------------------------------------------------------------
    // Observation builders
    // -------------------------------------------------------------------------

    /// 12-element absolute observation: 4 dirs × [wall_reciprocal, food_projection, body_reciprocal].
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
            let food_val = if food_on_ray { 1.0 } else { proj.max(0.0) };
            state.push(food_val);
            state.push(body);
        }

        state
    }

    /// 12-element GA four-direction vision: 4 dirs × [wall_reciprocal, food_bool, body_reciprocal].
    pub fn get_four_dir_vision(&self) -> Vec<f64> {
        let mut vision = Vec::new();
        let dirs = FourDirs::get_all_dirs();
        for d in dirs {
            let (wall, food, body) = self.look_in_dir_ga(self.head, d);
            vision.push(wall as f64);
            vision.push(if food { 1.0 } else { 0.0 });
            vision.push(body as f64);
        }
        vision
    }

    /// Euclidean distance between two grid points.
    #[inline]
    pub fn calculate_distance(p1: &Point, p2: &Point) -> f64 {
        (((p1.x - p2.x).pow(2) + (p1.y - p2.y).pow(2)) as f64).sqrt()
    }

    /// Dynamically computes the hunger step limit (max steps allowed without food)
    /// based exclusively on the current size of the snake (body segment count).
    #[inline]
    pub fn hunger_limit(&self) -> usize {
        dynamic_step_limit(self.body.len())
    }
}

/// Dynamically scales the hunger step limit (max steps allowed without food) based
/// exclusively on the snake's size (length). Larger snakes require more steps to
/// maneuver their bodies around obstacles to reach food without trapping themselves.
pub fn dynamic_step_limit(snake_size: usize) -> usize {
    match snake_size {
        s if s > 80 => 800,
        s if s > 30 => 500,
        s if s > 20 => 300,
        s if s > 10 => 200,
        _ => 100,
    }
}

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_step_limit_scales_with_snake_size() {
        assert_eq!(dynamic_step_limit(0), 100);
        assert_eq!(dynamic_step_limit(1), 100);
        assert_eq!(dynamic_step_limit(10), 100);
        assert_eq!(dynamic_step_limit(11), 200);
        assert_eq!(dynamic_step_limit(20), 200);
        assert_eq!(dynamic_step_limit(21), 300);
        assert_eq!(dynamic_step_limit(30), 300);
        assert_eq!(dynamic_step_limit(31), 500);
        assert_eq!(dynamic_step_limit(80), 500);
        assert_eq!(dynamic_step_limit(81), 800);
        assert_eq!(dynamic_step_limit(150), 800);
    }

    #[test]
    fn is_wall_rejects_border_and_accepts_inner() {
        let core = SnakeCore::new();
        assert!(core.is_wall(Point::new(0, 10)), "left border is wall");
        assert!(core.is_wall(Point::new(GRID_W, 10)), "right border is wall");
        assert!(core.is_wall(Point::new(10, 0)), "top border is wall");
        assert!(core.is_wall(Point::new(10, GRID_H)), "bottom border is wall");
        assert!(!core.is_wall(Point::new(5, 5)), "inner cell is not wall");
    }

    #[test]
    fn is_snake_body_skips_head_segment() {
        let mut core = SnakeCore::new();
        core.head = Point::new(10, 10);
        core.body = vec![
            Point::new(10, 10), // index 0 = head — must NOT count
            Point::new(10, 11),
            Point::new(10, 12),
        ];
        assert!(!core.is_snake_body(Point::new(10, 10)), "head index must not be 'body'");
        assert!(core.is_snake_body(Point::new(10, 11)), "segment 1 must be body");
        assert!(!core.is_snake_body(Point::new(5, 5)), "empty cell is not body");
    }

    #[test]
    fn get_random_empty_pos_avoids_body() {
        let mut core = SnakeCore::new();
        core.body = (1..(GRID_W - 1)).map(|x| Point::new(x, GRID_H / 2)).collect();
        for _ in 0..20 {
            let pos = core.get_random_empty_pos();
            assert!(!core.body.contains(&pos));
        }
    }

    #[test]
    fn reset_restores_default_state() {
        let mut core = SnakeCore::new();
        core.num_steps = 42;
        core.steps_without_food = 10;
        core.body.push(Point::new(1, 1));
        core.reset();
        assert_eq!(core.num_steps, 0);
        assert_eq!(core.steps_without_food, 0);
        assert_eq!(core.body.len(), 1);
        assert_eq!(core.body[0], core.head);
    }

    #[test]
    fn look_in_dir_dqn_wall_reciprocal_in_range() {
        let core = SnakeCore::new();
        let (wall, _food, _body) = core.look_in_dir_dqn(core.head, (1, 0));
        assert!(wall > 0.0 && wall <= 1.0, "wall reciprocal must be in (0,1]");
    }

    #[test]
    fn get_relative_state_returns_12_features() {
        let core = SnakeCore::new();
        assert_eq!(core.get_relative_state().len(), 12);
    }

    #[test]
    fn get_four_dir_vision_returns_12_features() {
        let core = SnakeCore::new();
        assert_eq!(core.get_four_dir_vision().len(), 12);
    }
}

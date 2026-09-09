//! Game logic adapted for DQN training
//! Single agent learning through experience

use crate::dqn::{DQNAgent, Experience};
use crate::*;

pub struct GameDQN {
    pub head: Point,
    pub body: Vec<Point>,
    pub food: Point,
    pub dir: FourDirs,
    pub agent: DQNAgent,
    pub score: usize,
    pub steps: usize,
    pub is_complete: bool,
    prev_distance: f64,
}

impl GameDQN {
    pub fn new() -> Self {
        let head = Point::new(GRID_W / 2, GRID_H / 2);
        let mut body = Vec::new();
        body.push(head.clone());
        let food = Point::rand();
        
        Self {
            body,
            head,
            food,
            dir: FourDirs::get_rand_dir(),
            agent: DQNAgent::new(),
            score: 0,
            steps: 0,
            is_complete: false,
            prev_distance: Self::calculate_distance(&head, &food),
        }
    }

    pub fn reset(&mut self) {
        self.head = Point::new(GRID_W / 2, GRID_H / 2);
        self.body.clear();
        self.body.push(self.head.clone());
        self.food = Point::rand();
        self.dir = FourDirs::get_rand_dir();
        self.score = 0;
        self.steps = 0;
        self.is_complete = false;
        self.prev_distance = Self::calculate_distance(&self.head, &self.food);
    }

    pub fn step(&mut self) -> (f64, bool) {
        if self.is_complete {
            return (0.0, true);
        }

        let state = self.get_state();
        let action = self.agent.select_action(&state);
        
        // Map action to direction
        let new_dir = match action {
            0 => FourDirs::Left,
            1 => FourDirs::Right,
            2 => FourDirs::Bottom,
            _ => FourDirs::Top,
        };
        
        // Prevent 180-degree turns
        if !(self.dir.is_horizontal() && new_dir.is_horizontal() && self.dir != new_dir) &&
           !(self.dir.is_vertical() && new_dir.is_vertical() && self.dir != new_dir) {
            self.dir = new_dir;
        }
        
        self.steps += 1;
        
        // Move snake
        self.head.x += self.dir.value().0;
        self.head.y += self.dir.value().1;
        
        let mut reward = -0.01; // Small penalty for each step
        let mut done = false;
        
        // Check collision
        if self.is_wall(self.head) || self.is_snake_body(self.head) {
            reward = -1.0;
            done = true;
            self.is_complete = true;
        } else if self.head == self.food {
            // Ate food
            reward = 1.0;
            self.score += 1;
            self.body.push(self.head.clone());
            self.food = self.get_random_empty_pos();
        } else {
            // Normal move
            let new_distance = Self::calculate_distance(&self.head, &self.food);
            if new_distance < self.prev_distance {
                reward = 0.1; // Reward for getting closer
            }
            self.prev_distance = new_distance;
            
            // Update body
            let mut prev_pos = self.head.clone();
            for p in self.body.iter_mut() {
                let temp = *p;
                *p = prev_pos;
                prev_pos = temp;
            }
        }
        
        // Check step limit
        if self.steps >= NUM_SIM_STEPS * 2 {
            done = true;
            self.is_complete = true;
        }
        
        let next_state = self.get_state();
        
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

    fn get_state(&self) -> Vec<f64> {
        let mut state = Vec::new();
        let dirs = FourDirs::get_all_dirs();
        
        for d in dirs {
            let (wall, food, body) = self.look_in_dir(self.head, d);
            state.push(wall as f64);
            state.push(if food { 1.0 } else { 0.0 });
            state.push(body as f64);
        }
        
        state
    }

    fn look_in_dir(&self, from: Point, dir: (i32, i32)) -> (f64, bool, f64) {
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

    fn calculate_distance(p1: &Point, p2: &Point) -> f64 {
        (((p1.x - p2.x).pow(2) + (p1.y - p2.y).pow(2)) as f64).sqrt()
    }

    fn is_wall(&self, pt: Point) -> bool {
        pt.x >= GRID_W || pt.x <= 0 || pt.y >= GRID_H || pt.y <= 0
    }

    fn is_snake_body(&self, pt: Point) -> bool {
        self.body.iter().skip(1).any(|p| *p == pt)
    }

    fn get_random_empty_pos(&self) -> Point {
        let mut pt = Point::rand();
        let mut tries = 0;
        
        while tries < 10 && self.body.contains(&pt) {
            pt = Point::rand();
            tries += 1;
        }
        
        pt
    }
}

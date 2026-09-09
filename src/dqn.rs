//! Deep Q-Network implementation for Snake
//! Includes Q-network, experience replay, and training logic

use rand::Rng;
use crate::nn::Net;
use crate::*;

// DQN hyperparameters are `pub` (design D-6): the dashboard's model-info
// panel displays the real constants the agent is constructed with. Values are
// pinned by `hyperparameter_constants_keep_their_pre_change_values` below.
pub const REPLAY_BUFFER_SIZE: usize = 10000;
pub const BATCH_SIZE: usize = 32;
pub const GAMMA: f64 = 0.99;
pub const LEARNING_RATE: f64 = 0.001;
pub const EPSILON_START: f64 = 1.0;
pub const EPSILON_END: f64 = 0.01;
pub const EPSILON_DECAY: f64 = 0.995;

#[derive(Clone)]
pub struct Experience {
    pub state: Vec<f64>,
    pub action: usize,
    pub reward: f64,
    pub next_state: Vec<f64>,
    pub done: bool,
}

pub struct ReplayBuffer {
    buffer: Vec<Experience>,
    capacity: usize,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, exp: Experience) {
        if self.buffer.len() >= self.capacity {
            self.buffer.remove(0);
        }
        self.buffer.push(exp);
    }

    pub fn sample(&self, batch_size: usize) -> Vec<Experience> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::new();
        
        for _ in 0..batch_size.min(self.buffer.len()) {
            let idx = rng.gen_range(0..self.buffer.len());
            samples.push(self.buffer[idx].clone());
        }
        
        samples
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

pub struct DQNAgent {
    pub q_network: Net,
    pub target_network: Net,
    pub replay_buffer: ReplayBuffer,
    pub epsilon: f64,
    steps: usize,
}

impl DQNAgent {
    pub fn new() -> Self {
        let q_network = Net::new();
        let target_network = q_network.clone();
        
        Self {
            q_network,
            target_network,
            replay_buffer: ReplayBuffer::new(REPLAY_BUFFER_SIZE),
            epsilon: EPSILON_START,
            steps: 0,
        }
    }

    pub fn select_action(&mut self, state: &Vec<f64>) -> usize {
        let mut rng = rand::thread_rng();
        
        // Epsilon-greedy
        if rng.gen::<f64>() < self.epsilon {
            rng.gen_range(0..OUTPUT_LAYER_SIZE)
        } else {
            let q_values = self.q_network.predict(state).pop().unwrap();
            q_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap()
        }
    }

    pub fn store_experience(&mut self, exp: Experience) {
        self.replay_buffer.push(exp);
    }

    pub fn train(&mut self) {
        if self.replay_buffer.len() < BATCH_SIZE {
            return;
        }

        let batch = self.replay_buffer.sample(BATCH_SIZE);
        
        for exp in batch {
            let current_q = self.q_network.predict(&exp.state).pop().unwrap();
            let next_q = self.target_network.predict(&exp.next_state).pop().unwrap();
            
            let max_next_q = next_q.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let target_q = if exp.done {
                exp.reward
            } else {
                exp.reward + GAMMA * max_next_q
            };
            
            // Gradient descent approximation via weight adjustment
            let error = target_q - current_q[exp.action];
            self.update_weights(&exp.state, exp.action, error);
        }

        self.steps += 1;
        
        // Decay epsilon
        self.epsilon = (self.epsilon * EPSILON_DECAY).max(EPSILON_END);
        
        // Update target network every 100 steps
        if self.steps % 100 == 0 {
            self.target_network = self.q_network.clone();
        }
    }

    fn update_weights(&mut self, _state: &Vec<f64>, action: usize, error: f64) {
        // Simplified weight update (approximation of backprop)

        // Update output layer weights for the selected action
        if let Some(layer) = self.q_network.layers.last_mut() {
            if action < layer.nodes.len() {
                for weight in layer.nodes[action].iter_mut() {
                    *weight += LEARNING_RATE * error;
                }
            }
        }
    }

    pub fn get_epsilon(&self) -> f64 {
        self.epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Approval guards for the visibility-only seam (design D-6 / spec
    // "visibility-only seams leave DQN behavior pinned"): the dashboard reads
    // the very same constants the agent is constructed with, so their values
    // are pinned here. No logic change anywhere in this module.
    #[test]
    fn hyperparameter_constants_keep_their_pre_change_values() {
        assert_eq!(REPLAY_BUFFER_SIZE, 10000);
        assert_eq!(BATCH_SIZE, 32);
        assert_eq!(GAMMA, 0.99);
        assert_eq!(LEARNING_RATE, 0.001);
        assert_eq!(EPSILON_START, 1.0);
        assert_eq!(EPSILON_END, 0.01);
        assert_eq!(EPSILON_DECAY, 0.995);
    }
}

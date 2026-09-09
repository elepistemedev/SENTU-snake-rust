//! Deep Q-Network implementation for Snake
//! Includes Q-network, experience replay, and training logic

use rand::Rng;
use crate::nn::Net;

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
pub const DQN_INP_LAYER_SIZE: usize = 9;
pub const DQN_HIDDEN_LAYER_SIZE: usize = 32;
pub const DQN_OUTPUT_LAYER_SIZE: usize = 3;
pub const DQN_ARCH: [usize; 3] = [DQN_INP_LAYER_SIZE, DQN_HIDDEN_LAYER_SIZE, DQN_OUTPUT_LAYER_SIZE];
pub const TARGET_UPDATE_INTERVAL: usize = 100;
pub const LOSS_EMA_ALPHA: f64 = 0.05;

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

/// Exponential moving average update (pure): `alpha * sample + (1 - alpha) *
/// prev`. Precondition: `alpha` in [0, 1] (not validated — internal constant).
pub fn ema_update(prev: f64, sample: f64, alpha: f64) -> f64 {
    alpha * sample + (1.0 - alpha) * prev
}

/// Index of the maximum value (pure). On ties the FIRST maximum index wins —
/// the dashboard highlights this action, so the choice must be deterministic.
/// Callers pass a non-empty slice (an empty slice returns 0).
pub fn argmax_index(values: &[f64]) -> usize {
    let mut best = 0usize;
    for (i, &v) in values.iter().enumerate().skip(1) {
        if v > values[best] {
            best = i;
        }
    }
    best
}

pub struct DQNAgent {
    pub q_network: Net,
    pub target_network: Net,
    pub replay_buffer: ReplayBuffer,
    pub epsilon: f64,
    steps: usize,
    /// EMA (LOSS_EMA_ALPHA) of the mean squared TD error per train() batch.
    /// 0.0 until the first train() call with a full batch. Read by the
    /// dashboard's TRAINING STATS panel.
    loss_ema: f64,
    /// How many times the target network has been synced from the q-network
    /// (once per TARGET_UPDATE_INTERVAL train steps).
    target_updates: usize,
}

impl DQNAgent {
    pub fn new() -> Self {
        let q_network = Net::new_with_sizes(&DQN_ARCH);
        let target_network = q_network.clone();
        
        Self {
            q_network,
            target_network,
            replay_buffer: ReplayBuffer::new(REPLAY_BUFFER_SIZE),
            epsilon: EPSILON_START,
            steps: 0,
            loss_ema: 0.0,
            target_updates: 0,
        }
    }

    pub fn select_action(&mut self, state: &Vec<f64>) -> usize {
        let mut rng = rand::thread_rng();
        
        // Epsilon-greedy
        if rng.gen::<f64>() < self.epsilon {
            rng.gen_range(0..DQN_OUTPUT_LAYER_SIZE)
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
        
        let batch_len = batch.len();
        let mut squared_error_sum = 0.0f64;
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
            squared_error_sum += error * error;
            self.update_weights(&exp.state, exp.action, error);
        }

        // Training-metrics instrumentation (display only): mean squared TD
        // error of this batch, smoothed for the dashboard. Never feeds back
        // into the weight update above.
        let batch_loss = squared_error_sum / batch_len as f64;
        self.loss_ema = ema_update(self.loss_ema, batch_loss, LOSS_EMA_ALPHA);

        self.steps += 1;
        
        // Decay epsilon
        self.epsilon = (self.epsilon * EPSILON_DECAY).max(EPSILON_END);
        
        // Update target network every TARGET_UPDATE_INTERVAL steps
        if self.steps % TARGET_UPDATE_INTERVAL == 0 {
            self.target_network = self.q_network.clone();
            self.target_updates += 1;
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

    /// EMA of the per-batch mean squared TD error (0.0 before the first
    /// train() call with a full batch). Dashboard read-only seam.
    pub fn loss_ema(&self) -> f64 {
        self.loss_ema
    }

    /// Number of target-network syncs performed so far. Dashboard read-only seam.
    pub fn target_updates(&self) -> usize {
        self.target_updates
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
        assert_eq!(TARGET_UPDATE_INTERVAL, 100);
        assert_eq!(LOSS_EMA_ALPHA, 0.05);
        assert_eq!(DQN_INP_LAYER_SIZE, 9);
        assert_eq!(DQN_HIDDEN_LAYER_SIZE, 32);
        assert_eq!(DQN_OUTPUT_LAYER_SIZE, 3);
    }

    // --- Slice dqn-training-metrics: consts puros y helpers -------------------

    #[test]
    fn training_metrics_constants_keep_their_pinned_values() {
        assert_eq!(TARGET_UPDATE_INTERVAL, 100);
        assert_eq!(LOSS_EMA_ALPHA, 0.05);
    }

    #[test]
    fn ema_update_interpolates_between_previous_and_sample() {
        assert_eq!(ema_update(2.0, 4.0, 0.0), 2.0, "alpha 0 keeps the previous value");
        assert_eq!(ema_update(2.0, 4.0, 1.0), 4.0, "alpha 1 takes the sample");
        assert!(
            (ema_update(2.0, 4.0, 0.5) - 3.0).abs() < 1e-12,
            "alpha 0.5 is the midpoint"
        );
    }

    #[test]
    fn argmax_index_returns_the_first_index_of_the_maximum() {
        assert_eq!(argmax_index(&[0.1, 0.9, 0.4, 0.2]), 1);
        assert_eq!(argmax_index(&[0.9, 0.1, 0.4, 0.2]), 0, "leading max wins");
        assert_eq!(
            argmax_index(&[0.5, 0.9, 0.9, 0.1]),
            1,
            "on a tie the FIRST maximum index wins (documented dashboard semantics)"
        );
    }

    // --- Slice dqn-training-metrics: agent instrumentation ---------------------

    /// Experiencia sintética válida: estado del tamaño de entrada de la red
    /// (`DQN_INP_LAYER_SIZE`), recompensa positiva para que el error TD sea no nulo.
    fn synth_experience() -> Experience {
        Experience {
            state: vec![0.0; DQN_INP_LAYER_SIZE],
            action: 0,
            reward: 1.0,
            next_state: vec![0.0; DQN_INP_LAYER_SIZE],
            done: false,
        }
    }

    #[test]
    fn loss_ema_starts_at_zero_and_updates_after_first_train() {
        let mut agent = DQNAgent::new();
        assert_eq!(agent.loss_ema(), 0.0, "loss EMA starts at 0.0 before any train() call");
        assert_eq!(agent.target_updates(), 0);

        for _ in 0..BATCH_SIZE {
            agent.store_experience(synth_experience());
        }
        agent.train();

        assert!(agent.loss_ema() > 0.0, "one train() call with a full batch must move the EMA");
        let snapshot = agent.loss_ema();
        assert_eq!(agent.loss_ema(), snapshot, "the getter is a pure read");
        assert_eq!(agent.target_updates(), 0, "one train step must not sync the target network");
    }

    #[test]
    fn target_updates_increments_every_interval_train_steps() {
        let mut agent = DQNAgent::new();
        for _ in 0..(BATCH_SIZE * 4) {
            agent.store_experience(synth_experience());
        }

        for _ in 0..TARGET_UPDATE_INTERVAL {
            agent.train();
        }
        assert_eq!(agent.target_updates(), 1, "exactly one sync after TARGET_UPDATE_INTERVAL train steps");

        agent.train();
        assert_eq!(agent.target_updates(), 1, "no sync at INTERVAL + 1 steps");

        for _ in 0..(TARGET_UPDATE_INTERVAL - 1) {
            agent.train();
        }
        assert_eq!(agent.target_updates(), 2, "second sync at 2 * INTERVAL steps");
    }

    #[test]
    fn dqn_arch_constants_are_pinned() {
        assert_eq!(DQN_INP_LAYER_SIZE, 9);
        assert_eq!(DQN_HIDDEN_LAYER_SIZE, 32);
        assert_eq!(DQN_OUTPUT_LAYER_SIZE, 3);
        assert_eq!(DQN_ARCH, [9, 32, 3]);
    }

    #[test]
    fn dqn_agent_net_matches_arch_and_predicts_three_outputs() {
        let agent = DQNAgent::new();
        assert_eq!(agent.q_network.n_inputs(), 9);
        assert!(agent.q_network.matches_arch(&[9, 32, 3]));
        let out = agent.q_network.predict(&vec![0.0; 9]);
        let final_out = out.last().unwrap();
        assert_eq!(final_out.len(), 3, "DQN agent must output 3 Q-values");
    }

    #[test]
    fn select_action_never_exceeds_relative_action_space() {
        let mut agent = DQNAgent::new();
        let state = vec![0.0; 9];
        for _ in 0..30 {
            let action = agent.select_action(&state);
            assert!(action < 3, "action must be inside 0..DQN_OUTPUT_LAYER_SIZE");
        }
    }
}

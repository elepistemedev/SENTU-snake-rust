//! Unified Agent trait and concrete implementations for GA and DQN.

use crate::game::Game;
use crate::nn::Net;
use crate::utils::{relative_dir, rotate_vision_to_relative, FourDirs};

/// Universal decision-making agent for Snake.
pub trait Agent: Send + Sync {
    /// Human-readable name or model family (e.g. "Algoritmo Genético", "DQN").
    fn name(&self) -> &str;

    /// Choose a direction given the current game state.
    fn decide_direction(&self, game: &Game) -> FourDirs;

    /// Clone this agent into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Agent>;

    /// Optional access to the underlying neural network, if any.
    fn network(&self) -> Option<&Net> {
        None
    }

    /// Get layer-by-layer network activations for visualization/dashboards, if applicable.
    fn network_output(&self, _game: &Game) -> Option<Vec<Vec<f64>>> {
        None
    }
}

impl Clone for Box<dyn Agent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Agent driven by an absolute 12-input Genetic Algorithm neural network.
#[derive(Clone)]
pub struct GaAgent {
    pub net: Net,
}

impl GaAgent {
    pub fn new(net: Net) -> Self {
        Self { net }
    }
}

impl Agent for GaAgent {
    fn name(&self) -> &str {
        "Algoritmo Genético"
    }

    fn decide_direction(&self, game: &Game) -> FourDirs {
        let vision = game.get_four_dir_vision();
        let nn_out = self.net.predict(&vision).pop().unwrap();
        let max_index = nn_out
            .iter()
            .enumerate()
            .max_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap();
        let mut dir = match max_index {
            0 => FourDirs::Left,
            1 => FourDirs::Right,
            2 => FourDirs::Bottom,
            _ => FourDirs::Top,
        };

        // Prevent 180-degree instant reversal into own neck
        if game.dir.is_horizontal() && dir.is_horizontal() && game.dir != dir {
            dir = game.dir;
        }
        if game.dir.is_vertical() && dir.is_vertical() && game.dir != dir {
            dir = game.dir;
        }

        dir
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn network(&self) -> Option<&Net> {
        Some(&self.net)
    }

    fn network_output(&self, game: &Game) -> Option<Vec<Vec<f64>>> {
        let vision = game.get_four_dir_vision();
        Some(self.net.predict(&vision))
    }
}

/// Agent driven by a 9-input heading-relative DQN neural network policy.
#[derive(Clone)]
pub struct DqnPolicyAgent {
    pub net: Net,
}

impl DqnPolicyAgent {
    pub fn new(net: Net) -> Self {
        Self { net }
    }
}

impl Agent for DqnPolicyAgent {
    fn name(&self) -> &str {
        "DQN"
    }

    fn decide_direction(&self, game: &Game) -> FourDirs {
        let abs = game.get_relative_state();
        let vision = rotate_vision_to_relative(&abs, game.dir);
        let nn_out = self.net.predict(&vision).pop().unwrap();
        let max_index = nn_out
            .iter()
            .enumerate()
            .max_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap();
        relative_dir(game.dir, max_index)
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn network(&self) -> Option<&Net> {
        Some(&self.net)
    }

    fn network_output(&self, game: &Game) -> Option<Vec<Vec<f64>>> {
        let abs = game.get_relative_state();
        let vision = rotate_vision_to_relative(&abs, game.dir);
        Some(self.net.predict(&vision))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ga_agent_name_and_network() {
        let net = Net::new();
        let agent = GaAgent::new(net.clone());
        assert_eq!(agent.name(), "Algoritmo Genético");
        assert!(agent.network().is_some());
    }

    #[test]
    fn test_dqn_policy_agent_name_and_network() {
        let net = Net::new_with_sizes(&[9, 32, 3]);
        let agent = DqnPolicyAgent::new(net.clone());
        assert_eq!(agent.name(), "DQN");
        assert!(agent.network().is_some());
    }

    #[test]
    fn test_agent_clone_box() {
        let agent: Box<dyn Agent> = Box::new(GaAgent::new(Net::new()));
        let cloned = agent.clone();
        assert_eq!(cloned.name(), "Algoritmo Genético");
    }

    #[test]
    fn test_agent_decide_direction_returns_valid_direction() {
        let game = Game::new();
        let ga_agent = GaAgent::new(Net::new());
        let dir = ga_agent.decide_direction(&game);
        assert!([FourDirs::Left, FourDirs::Right, FourDirs::Top, FourDirs::Bottom].contains(&dir));

        let dqn_agent = DqnPolicyAgent::new(Net::new_with_sizes(&[9, 32, 3]));
        let dqn_dir = dqn_agent.decide_direction(&game);
        assert!([FourDirs::Left, FourDirs::Right, FourDirs::Top, FourDirs::Bottom].contains(&dqn_dir));
    }
}

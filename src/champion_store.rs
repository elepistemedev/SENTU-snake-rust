//! DQN champion persistence.
//!
//! Pure serde encode/decode of a `Net` plus thin filesystem save/load helpers.
//! Missing or corrupt files degrade to `None` — this module never panics on
//! bad input. The on-disk format matches `best_snake.json` (serde `Net`).

use std::fs;

use serde::{Deserialize, Serialize};

use crate::nn::Net;

pub const DQN_METADATA_FILE: &str = "dqn_metadata.json";

fn default_epsilon() -> f64 {
    crate::dqn::EPSILON_WARM_START
}

/// Bookkeeping metadata associated with the persisted DQN champion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DqnMetadata {
    pub best_score: usize,
    pub episode: usize,
    #[serde(default = "default_epsilon")]
    pub epsilon: f64,
}

/// Serialize a `Net` to pretty JSON (same format as `best_snake.json`).
pub fn encode(net: &Net) -> String {
    // `Net` is plain data; serialization cannot fail in practice.
    serde_json::to_string_pretty(net).unwrap_or_default()
}

/// Deserialize a `Net` from JSON, mapping failures to a readable error.
pub fn decode(s: &str) -> Result<Net, String> {
    serde_json::from_str(s).map_err(|e| format!("invalid champion JSON: {e}"))
}

/// Write `net` to `path`. Returns an io error if writing fails or encoding
/// produced no usable output.
pub fn save(path: &str, net: &Net) -> std::io::Result<()> {
    let json = encode(net);
    if json.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "champion serialization produced no output",
        ));
    }
    fs::write(path, json)
}

/// Load a `Net` from `path`. Missing files and corrupt contents both degrade
/// to `None`; this never panics.
pub fn load(path: &str) -> Option<Net> {
    let json = fs::read_to_string(path).ok()?;
    decode(&json).ok()
}

/// Save DQN champion metadata alongside the network.
pub fn save_metadata(path: &str, meta: &DqnMetadata) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(meta).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    fs::write(path, json)
}

/// Load DQN champion metadata if present.
pub fn load_metadata(path: &str) -> Option<DqnMetadata> {
    let json = fs::read_to_string(path).ok()?;
    serde_json::from_str(&json).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nn::Net;

    /// `Net` has no PartialEq; JSON float round trips are not bit-exact at the
    /// last ulp, so compare architecture and weights within a tight tolerance.
    fn nets_approx_eq(a: &Net, b: &Net) -> bool {
        if a.layers.len() != b.layers.len() {
            return false;
        }
        for (la, lb) in a.layers.iter().zip(b.layers.iter()) {
            if la.nodes.len() != lb.nodes.len() {
                return false;
            }
            for (na, nb) in la.nodes.iter().zip(lb.nodes.iter()) {
                if na.len() != nb.len() {
                    return false;
                }
                for (wa, wb) in na.iter().zip(nb.iter()) {
                    if (wa - wb).abs() > 1e-9 {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[test]
    fn encode_decode_round_trip_preserves_net() {
        let net = Net::new();
        let encoded = encode(&net);
        let decoded = decode(&encoded).expect("valid encoding must decode");
        assert!(
            nets_approx_eq(&net, &decoded),
            "decoded net must match original"
        );
    }

    #[test]
    fn decode_corrupt_input_errors() {
        assert!(decode("not json at all").is_err());
        assert!(decode("").is_err());
        assert!(decode("{\"layers\": 42}").is_err());
    }

    #[test]
    fn load_missing_file_returns_none_without_panicking() {
        let path = "definitely-missing-dqn-champion-4f2c.json";
        assert!(load(path).is_none());
    }

    #[test]
    fn save_then_load_round_trips_through_disk() {
        let net = Net::new();
        let path = std::env::temp_dir().join("snake_dqn_champion_store_test.json");
        let path_str = path.to_str().expect("temp path is utf-8");
        save(path_str, &net).expect("save to temp path must succeed");
        let loaded = load(path_str).expect("saved file must load");
        assert!(
            nets_approx_eq(&net, &loaded),
            "loaded net must match saved net"
        );
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn decode_corrupt_saved_file_degrades_to_none() {
        let path = std::env::temp_dir().join("snake_dqn_champion_store_corrupt.json");
        let path_str = path.to_str().expect("temp path is utf-8");
        std::fs::write(path_str, "{ this is not valid json").ok();
        assert!(load(path_str).is_none());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn metadata_round_trip_and_missing() {
        let path = std::env::temp_dir().join("snake_dqn_metadata_test.json");
        let path_str = path.to_str().expect("temp path is utf-8");

        assert!(load_metadata("definitely_missing_dqn_metadata.json").is_none());

        let meta = DqnMetadata {
            best_score: 25,
            episode: 150,
            epsilon: 0.22,
        };
        save_metadata(path_str, &meta).expect("save metadata must succeed");
        let loaded = load_metadata(path_str).expect("load metadata must succeed");
        assert_eq!(meta, loaded);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn metadata_deserializes_with_default_epsilon_when_missing() {
        let json = r#"{"best_score": 15, "episode": 42}"#;
        let meta: DqnMetadata = serde_json::from_str(json).expect("should deserialize legacy json");
        assert_eq!(meta.best_score, 15);
        assert_eq!(meta.episode, 42);
        assert_eq!(meta.epsilon, crate::dqn::EPSILON_WARM_START);
    }
}


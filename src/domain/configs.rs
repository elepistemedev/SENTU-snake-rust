use std::sync::LazyLock;
use crate::env_config::{env_f32, env_u64, env_usize};

// Game
pub const GRID_W: i32 = 25;
pub const GRID_H: i32 = 25;

// Sim
pub static NUM_GAMES_PER_STREAM: LazyLock<usize> = LazyLock::new(|| env_usize("NUM_GAMES_PER_STREAM", 1000));
pub static NUM_STREAMS: LazyLock<usize> = LazyLock::new(|| env_usize("NUM_STREAMS", 1));
pub static NUM_SIM_STEPS: LazyLock<usize> = LazyLock::new(|| env_usize("NUM_SIM_STEPS", 500));
pub static STREAM_REJUVENATION_PERCENT: LazyLock<f32> = LazyLock::new(|| env_f32("STREAM_REJUVENATION_PERCENT", 0.1));
pub static STREAM_LOCAL_MAX_WAIT_SECS: LazyLock<f32> = LazyLock::new(|| env_f32("STREAM_LOCAL_MAX_WAIT_SECS", 90.0));
pub static SIM_SLEEP_MILLIS: LazyLock<u64> = LazyLock::new(|| env_u64("SIM_SLEEP_MILLIS", 50));

// Pop
pub static POP_NUM_RETAINED: LazyLock<f32> = LazyLock::new(|| env_f32("POP_NUM_RETAINED", 0.01));
pub static POP_NUM_CHILDREN: LazyLock<f32> = LazyLock::new(|| env_f32("POP_NUM_CHILDREN", 0.5));
pub static POP_NUM_RANDOM: LazyLock<f32> = LazyLock::new(|| env_f32("POP_NUM_RANDOM", 0.2));
pub static POP_NUM_RETAINED_MUTATED: LazyLock<f32> = LazyLock::new(|| env_f32("POP_NUM_RETAINED_MUTATED", 0.29));

// Viz
pub const VIZ_GRID_W: i32 = 5;
pub const VIZ_GRID_H: i32 = 4;
pub const VIZ_DARK_THEME: bool = true;

// NN
pub static BRAIN_MUTATION_RATE: LazyLock<f32> = LazyLock::new(|| env_f32("BRAIN_MUTATION_RATE", 0.1));
pub static BRAIN_MUTATION_VARIATION: LazyLock<f32> = LazyLock::new(|| env_f32("BRAIN_MUTATION_VARIATION", 0.1));
pub const INP_LAYER_SIZE: usize = 12;
pub const HIDDEN_LAYER_SIZE: usize = 8;
pub const OUTPUT_LAYER_SIZE: usize = 4;

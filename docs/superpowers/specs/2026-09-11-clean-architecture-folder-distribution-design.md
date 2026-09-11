# Clean Architecture Folder Distribution Design

## Goal

Restructure the flat `src/` directory into explicit architectural layers (`domain/`, `persistence/`, `presentation/`, `presentation/views/`) reflecting Clean / Hexagonal Architecture, preserving 100% of test behavior, git file history (`git mv`), and backwards compatibility through module re-exports.

---

## Directory Structure

```text
src/
├── main.rs
├── lib.rs
│
├── domain/                         # Layer 1: Core Domain (100% Macroquad-free / Headless)
│   ├── mod.rs
│   ├── snake_core.rs               # Board physics, collisions, raycasting
│   ├── swallow.rs                  # Digestion state machine & bulge tracking
│   ├── game.rs                     # Snake game in Genetic Algorithm context
│   ├── game_dqn.rs                 # Gym-style RL Snake environment
│   ├── agent.rs                    # Agent trait, GaAgent, DqnPolicyAgent
│   ├── nn.rs                       # Neural network model & genetic operators
│   ├── dqn.rs                      # Deep Q-Learning agent, replay buffer
│   ├── pop.rs                      # Island population genetic evolution
│   ├── stream.rs                   # Single island evolution stream
│   ├── versus.rs                   # Headless 1v1 match & series engine
│   ├── sim.rs                      # Headless GA simulation orchestrator
│   ├── configs.rs                  # System constants and hyperparameters
│   └── utils.rs                    # Point, FourDirs, spatial/heading frames
│
├── persistence/                    # Layer 2: Application & Persistence
│   ├── mod.rs
│   ├── champion_store.rs           # JSON persistence for DQN champion & metadata
│   └── theme.rs                    # Color palette definitions & theme persistence
│
└── presentation/                   # Layer 3: Presentation & UI (Macroquad)
    ├── mod.rs
    ├── app.rs                      # Main app shell, mode state machine, menu
    ├── ui_kit.rs                   # Terminal design tokens, widgets, colors
    ├── render_snake.rs             # Snake, apple, eye offset rendering
    ├── dqn_dash.rs                 # DQN live training dashboard
    ├── viz.rs                      # Legacy multi-snake grid visualization
    ├── viz_advanced.rs             # Advanced GA dashboard renderer
    ├── viz_vs.rs                   # Versus match & series visualizer
    └── views/                      # Interactive screen views
        ├── mod.rs
        ├── view_ga_train.rs        # GA training view controller
        ├── view_ga_versus.rs       # GA versus view controller
        ├── view_dqn_train.rs       # DQN training view controller
        ├── view_dqn_versus.rs      # DQN versus view controller
        └── view_cross_match.rs     # GA vs DQN cross-match view controller
```

---

## Module Hierarchy & Backwards Compatibility

### `src/lib.rs`
```rust
pub mod domain;
pub mod persistence;
pub mod presentation;

// Backwards compatibility re-exports:
pub use domain::*;
pub use persistence::*;
pub use presentation::*;
```

### `src/domain/mod.rs`
Declares and exports:
- `pub mod agent;`
- `pub mod configs;`
- `pub mod dqn;`
- `pub mod game;`
- `pub mod game_dqn;`
- `pub mod nn;`
- `pub mod pop;`
- `pub mod sim;`
- `pub mod snake_core;`
- `pub mod stream;`
- `pub mod swallow;`
- `pub mod utils;`
- `pub mod versus;`
- And convenient re-exports: `pub use agent::*; pub use configs::*; pub use snake_core::SnakeCore; pub use swallow::SwallowTracker; pub use utils::*;`

### `src/persistence/mod.rs`
Declares and exports:
- `pub mod champion_store;`
- `pub mod theme;`
- And `pub use champion_store::*; pub use theme::*;`

### `src/presentation/mod.rs`
Declares and exports:
- `pub mod app;`
- `pub mod dqn_dash;`
- `pub mod render_snake;`
- `pub mod ui_kit;`
- `pub mod views;`
- `pub mod viz;`
- `pub mod viz_advanced;`
- `pub mod viz_vs;`
- And `pub use app::*; pub use views::*;`

### `src/presentation/views/mod.rs`
Declares and exports:
- `pub mod view_cross_match;`
- `pub mod view_dqn_train;`
- `pub mod view_dqn_versus;`
- `pub mod view_ga_train;`
- `pub mod view_ga_versus;`
- And re-exports all views.

---

## Migration Strategy (Git & Preservation)

1. Use `git mv` for every file move so git history and blame are preserved.
2. Create `mod.rs` files for each directory.
3. Update internal `use` statements where necessary.
4. Run `cargo check` and `cargo test` to verify all 144 unit tests pass without errors.
5. Update graphify knowledge graph.

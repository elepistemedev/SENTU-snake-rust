# Clean Architecture Folder Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reorganize the flat `src/` codebase into explicit Clean Architecture layer directories (`src/domain/`, `src/persistence/`, `src/presentation/`, `src/presentation/views/`) using `git mv` to preserve git history and blame, maintaining 100% backward compatibility via module re-exports in `src/lib.rs`.

**Architecture:** Clean / Hexagonal Architecture with three explicit layers:
1. `domain/` (100% Macroquad-free, headless core physics, models, RL/GA algorithms, math).
2. `persistence/` (JSON persistence for champions and themes).
3. `presentation/` (Macroquad UI, shell, widgets, visualizers) with sub-package `views/` for interactive screens.

**Tech Stack:** Rust, Cargo, Macroquad, Serde

**Spec:** `docs/superpowers/specs/2026-09-11-clean-architecture-folder-distribution-design.md`

## Global Constraints

- Use `git mv` for every file move so git blame and history remain intact.
- All 144 unit tests must continue to pass without failures.
- Zero compiler warnings (`cargo check`).
- Maintain 100% backward compatibility via `pub use` in `src/lib.rs`.

---

### Task 1: Create `src/domain/` and Migrate Domain Modules

**Files:**
- Move:
  - `src/agent.rs` -> `src/domain/agent.rs`
  - `src/configs.rs` -> `src/domain/configs.rs`
  - `src/dqn.rs` -> `src/domain/dqn.rs`
  - `src/game.rs` -> `src/domain/game.rs`
  - `src/game_dqn.rs` -> `src/domain/game_dqn.rs`
  - `src/nn.rs` -> `src/domain/nn.rs`
  - `src/pop.rs` -> `src/domain/pop.rs`
  - `src/sim.rs` -> `src/domain/sim.rs`
  - `src/snake_core.rs` -> `src/domain/snake_core.rs`
  - `src/stream.rs` -> `src/domain/stream.rs`
  - `src/swallow.rs` -> `src/domain/swallow.rs`
  - `src/utils.rs` -> `src/domain/utils.rs`
  - `src/versus.rs` -> `src/domain/versus.rs`
- Create: `src/domain/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: None (domain is the innermost layer).
- Produces: `crate::domain::{agent, configs, dqn, game, game_dqn, nn, pop, sim, snake_core, stream, swallow, utils, versus}` and re-exports.

- [ ] **Step 1: Create `src/domain/` directory and move files using `git mv`**
- [ ] **Step 2: Create `src/domain/mod.rs` declaring and re-exporting all domain submodules**
- [ ] **Step 3: Update `src/lib.rs` to declare `pub mod domain;` and re-export domain symbols**
- [ ] **Step 4: Run `cargo check` and fix any path references**
- [ ] **Step 5: Run `cargo test` to verify domain tests pass**
- [ ] **Step 6: Commit changes**

---

### Task 2: Create `src/persistence/` and Migrate Persistence Modules

**Files:**
- Move:
  - `src/champion_store.rs` -> `src/persistence/champion_store.rs`
  - `src/theme.rs` -> `src/persistence/theme.rs`
- Create: `src/persistence/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `crate::domain::nn::Net`, `crate::domain::dqn`
- Produces: `crate::persistence::{champion_store, theme}` and re-exports.

- [ ] **Step 1: Create `src/persistence/` directory and move files using `git mv`**
- [ ] **Step 2: Create `src/persistence/mod.rs` declaring and re-exporting all persistence submodules**
- [ ] **Step 3: Update `src/lib.rs` to declare `pub mod persistence;` and re-export persistence symbols**
- [ ] **Step 4: Run `cargo check` and verify compilation**
- [ ] **Step 5: Run `cargo test` to verify persistence tests pass**
- [ ] **Step 6: Commit changes**

---

### Task 3: Create `src/presentation/views/` and `src/presentation/` and Migrate UI Modules

**Files:**
- Move to `src/presentation/views/`:
  - `src/view_cross_match.rs` -> `src/presentation/views/view_cross_match.rs`
  - `src/view_dqn_train.rs` -> `src/presentation/views/view_dqn_train.rs`
  - `src/view_dqn_versus.rs` -> `src/presentation/views/view_dqn_versus.rs`
  - `src/view_ga_train.rs` -> `src/presentation/views/view_ga_train.rs`
  - `src/view_ga_versus.rs` -> `src/presentation/views/view_ga_versus.rs`
- Create: `src/presentation/views/mod.rs`
- Move to `src/presentation/`:
  - `src/app.rs` -> `src/presentation/app.rs`
  - `src/dqn_dash.rs` -> `src/presentation/dqn_dash.rs`
  - `src/render_snake.rs` -> `src/presentation/render_snake.rs`
  - `src/ui_kit.rs` -> `src/presentation/ui_kit.rs`
  - `src/viz.rs` -> `src/presentation/viz.rs`
  - `src/viz_advanced.rs` -> `src/presentation/viz_advanced.rs`
  - `src/viz_vs.rs` -> `src/presentation/viz_vs.rs`
- Create: `src/presentation/mod.rs`
- Modify: `src/lib.rs` and `src/main.rs`

**Interfaces:**
- Consumes: `crate::domain::*`, `crate::persistence::*`
- Produces: `crate::presentation::*`, `crate::presentation::views::*`

- [ ] **Step 1: Create directories `src/presentation/views/` and move view files using `git mv`**
- [ ] **Step 2: Create `src/presentation/views/mod.rs` declaring and re-exporting view submodules**
- [ ] **Step 3: Move remaining UI files into `src/presentation/` using `git mv`**
- [ ] **Step 4: Create `src/presentation/mod.rs` declaring submodules and re-exporting views**
- [ ] **Step 5: Update `src/lib.rs` and `src/main.rs` for the presentation module**
- [ ] **Step 6: Run `cargo check` and resolve any path adjustments**
- [ ] **Step 7: Run `cargo test` to verify all 144 tests pass**
- [ ] **Step 8: Commit changes**

---

### Task 4: Final Verification and Graphify Update

**Files:**
- `graphify-out/`

- [ ] **Step 1: Run full test suite `cargo test` and verify 144 passed with 0 failures**
- [ ] **Step 2: Run `cargo check` and verify 0 compiler warnings**
- [ ] **Step 3: Run graphify code update `graphify . --update --code-only`**
- [ ] **Step 4: Update walkthrough.md with final folder layout**
- [ ] **Step 5: Commit documentation and graphify output**

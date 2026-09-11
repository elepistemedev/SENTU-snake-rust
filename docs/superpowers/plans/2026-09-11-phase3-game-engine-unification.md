# Phase 3: Game Engine Unification (SnakeCore) — Implementation Plan

> **Completed:** 2026-09-11 | Commit: `e1e7ae8` on `feature/clean-architecture`

## Goal

Eliminate the code duplication between `src/game.rs` (`Game`) and
`src/game_dqn.rs` (`GameDQN`) by extracting all shared snake board physics
into a new `src/snake_core.rs` (`SnakeCore`). Both types embed `SnakeCore` and
expose it via `Deref<Target=SnakeCore>` so every existing caller remains
unchanged.

## Architecture

```
           ┌─────────────┐
           │  SnakeCore  │   (src/snake_core.rs)
           │  head body  │
           │  food dir   │
           │  swallow    │
           │  step ctrs  │
           └──────┬──────┘
        ┌─────────┴──────────┐
        │  Deref<SnakeCore>  │   both embed + expose via Deref/DerefMut
        ▼                    ▼
  ┌───────────┐       ┌──────────────┐
  │   Game    │       │   GameDQN    │
  │  brain    │       │  agent       │
  │  agent    │       │  score steps │
  │is_complete│       │ prev_distance│
  └───────────┘       └──────────────┘
```

## Extracted to SnakeCore

| Method | Previously duplicated in |
|---|---|
| `is_wall` | `game.rs` + `game_dqn.rs` |
| `is_snake_body` | `game.rs` + `game_dqn.rs` |
| `get_random_empty_pos` | `game.rs` (tries<5) + `game_dqn.rs` (tries<10 — better) |
| `look_in_dir_dqn` | `game.rs::look_in_dir_dqn` = `game_dqn.rs::look_in_dir` |
| `look_in_dir_ga` | `game.rs::look_in_dir` (GA f32 raycasting) |
| `calculate_distance` | `game_dqn.rs` (static fn) |
| `get_relative_state` | `game.rs` |
| `get_four_dir_vision` | `game.rs` |
| `advance_without_food` | movement in `game_dqn.rs::step` |
| `eat_food` | movement in `game_dqn.rs::step` |
| `advance_head_only` | `game.rs::update_snake_positions` preamble |
| `reset` | `game_dqn.rs::reset` board state |

## Files Changed

| Action | File |
|---|---|
| **NEW** | `src/snake_core.rs` |
| **MODIFY** | `src/lib.rs` — `pub mod snake_core; pub use snake_core::SnakeCore` |
| **MODIFY** | `src/game.rs` — embed `core: SnakeCore`, add `Deref<SnakeCore>` |
| **MODIFY** | `src/game_dqn.rs` — embed `core: SnakeCore`, add `Deref<SnakeCore>` |

## Verification

- `cargo test`: **144 passed, 0 failed** (137 pre-existing + 7 new SnakeCore tests)
- `cargo check`: **0 warnings**
- graphify: 811 nodes, 1641 edges (updated `--code-only`)

## Constraints Preserved

- All callers (`dqn_dash.rs`, `view_dqn_train.rs`, versus, etc.) write
  `game.head`, `game.body`, `game.is_wall(pt)` — unchanged.
- `GameDQN` body[0]==head invariant: preserved.
- No 180° direction reversals: preserved.
- `Game::fitness()`, `score()`: unchanged logic.
- All reward constants in `GameDQN::step` (±1.0, 2.0, 0.1, -0.15, -0.5): unchanged.

# Phase 5: Domain Layer Purification — Implementation Plan

> **Completed:** 2026-09-11 | Commit: `135b1c3` on `feature/clean-architecture`

## Goal

Purify all domain modules from any coupling to UI/graphics (`macroquad`), establishing a 100% headless, testable core domain according to Clean Architecture / Hexagonal Architecture principles.

## Changes Completed

1. **`src/configs.rs`**: Removed orphan `use macroquad::prelude::*;`.
2. **`src/swallow.rs`**: Extracted `SwallowTracker` state machine out of presentation (`render_snake.rs`) into a standalone domain module (`src/swallow.rs`). `render_snake.rs` re-exports it for backward compatibility. `snake_core.rs` now imports directly from domain (`swallow`) instead of presentation.
3. **`src/utils.rs` & `src/ui_kit.rs`**: Moved `Color` helpers (`color_with_a`, `are_colors_equal`) from `utils.rs` into `ui_kit.rs`. Removed `use macroquad::color::Color;` from `utils.rs`.
4. **`src/viz.rs`**: Updated to import color helpers from `crate::ui_kit`.

## Resulting Clean Architecture Layers

```
LAYER 1: DOMAIN CORE (100% Macroquad-free / Headless):
├── snake_core.rs      (física del tablero, colisiones, raycast)
├── swallow.rs         (física de deglución y animación de bultos)
├── game.rs            (juego en contexto GA)
├── game_dqn.rs        (juego en contexto DQN)
├── agent.rs           (trait polimórfico Agent, GaAgent, DqnPolicyAgent)
├── nn.rs              (red neuronal, capas, predicción, mutación)
├── dqn.rs             (Q-learning, replay buffer, target network)
├── pop.rs             (población, islas genéticas)
├── stream.rs          (isla genética individual)
├── versus.rs          (partidas 1v1, series al mejor de N)
├── sim.rs             (orquestador de simulación GA)
├── configs.rs         (constantes de configuración)
└── utils.rs           (coordenadas Point, FourDirs, frames relativos)

LAYER 2: APPLICATION & PERSISTENCE:
├── champion_store.rs  (persistencia serde JSON de campeón DQN)
└── theme.rs           (definición de paletas y persistencia JSON)

LAYER 3: PRESENTATION (Macroquad UI):
├── ui_kit.rs          (tokens de diseño, widgets, color helpers)
├── render_snake.rs    (dibujo de serpiente, ojos direccionales, manzana)
├── viz.rs / viz_advanced.rs / viz_vs.rs / dqn_dash.rs
├── view_*.rs          (controladores de vista)
└── app.rs / main.rs   (bucle principal y shell)
```

## Verification

- `cargo check`: 0 warnings, 0 errors.
- `cargo test`: 144 passed, 0 failed.
- Domain macroquad audit: `grep "macroquad" <all domain modules>` yields 0 import statements.
- Graphify: 830 nodes, 1648 edges.

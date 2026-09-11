# Graph Report - SENTU-snake-rust-feature-dqn  (2026-09-11)

## Corpus Check
- 64 files · ~79,749 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 993 nodes · 1905 edges · 52 communities (37 shown, 15 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 48 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b9415b6b`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Simulation
- Game
- SnakeCore
- App
- view_ga_versus.rs
- ui_kit.rs
- GameTheme
- view_dqn_train.rs
- versus.rs
- VsFlavor
- dqn.rs
- FourDirs
- Net
- view_cross_match.rs
- Viz
- view_dqn_versus.rs
- Deep Q-Network (DQN) Implementation
- Apply Progress — unified-snake-shell (Slices 1–5 of 5)
- Training dashboard UI Apply Progress Slice A
- game_dqn.rs
- env_config.rs
- Archive Report — unified-snake-shell
- Exploration: unified-snake-shell
- Unified Snake Shell — Proposal
- Module Hierarchy & Backwards Compatibility
- Unified Snake Shell — Design
- 3. Especificación Detallada por Módulo
- Phase 3: Game Engine Unification (SnakeCore) — Implementation Plan
- Global Constraints
- Global Constraints
- Global Constraints
- Global Constraints
- Snake Rust — app unificada (crate `snake`)
- Phase 5: Domain Layer Purification — Implementation Plan
- Canonical no regression of GA evolution behavior
- Unified snake shell verify report
- Serena Rust Project Configuration
- rules/graphify.md
- workflows/graphify.md
- DQN champion persistence requirement
- DQN internal versus view requirement
- DQN vs GA cross-match view requirement
- GA internal versus view requirement
- Unified single-binary app shell requirement
- Welcome menu with keyboard navigation requirement
- D-5 Public GameDQN observation accessor
- Training dashboard geometry mapping from viz_advanced
- Unified snake shell workload forecast
- Requirement: DQN bar and chart normalization denominators
- Requirement: Shell Tab routing and hint overlay for DQN dashboard
- snake

## God Nodes (most connected - your core abstractions)
1. `Net` - 76 edges
2. `Game` - 61 edges
3. `Apply Progress — unified-snake-shell (Slices 1–5 of 5)` - 42 edges
4. `App` - 33 edges
5. `SnakeCore` - 28 edges
6. `Simulation` - 27 edges
7. `GameTheme` - 27 edges
8. `DqnTrainView` - 27 edges
9. `Viz` - 23 edges
10. `GameDQN` - 21 edges

## Surprising Connections (you probably didn't know these)
- `draw_neural_network()` --calls--> `argmax_index()`  [INFERRED]
  src/presentation/dqn_dash.rs → src/domain/dqn.rs
- `relative_brain_observation_matches_game_dqn_observation()` --calls--> `rotate_vision_to_relative()`  [INFERRED]
  src/domain/game.rs → src/domain/utils.rs
- `relative_brain_body_distance_zero_when_no_body_present()` --calls--> `rotate_vision_to_relative()`  [INFERRED]
  src/domain/game.rs → src/domain/utils.rs
- `nets_eq()` --references--> `Net`  [EXTRACTED]
  src/presentation/views/view_cross_match.rs → src/domain/nn.rs
- `nets_approx_eq()` --references--> `Net`  [EXTRACTED]
  src/presentation/views/view_dqn_train.rs → src/domain/nn.rs

## Import Cycles
- 2-file cycle: `src/domain/agent.rs -> src/domain/game.rs -> src/domain/agent.rs`

## Hyperedges (group relationships)
- **Unified Snake Shell Lifecycle** — openspec_changes_archive_2026_09_09_unified_snake_shell_specs_app_spec_unified_single_binary_app_shell, openspec_changes_archive_2026_09_09_unified_snake_shell_tasks_implementation_phases, openspec_changes_archive_2026_09_09_unified_snake_shell_verify_report_verification_results, openspec_changes_archive_2026_09_09_unified_snake_shell_sync_report_sync_status [EXTRACTED 1.00]

## Communities (52 total, 15 thin omitted)

### Community 0 - "Simulation"
Cohesion: 0.05
Nodes (20): GenUpdate, load_ga_champions(), load_gen_history(), population_get_gen_summary_includes_max_steps(), Default, Option, Self, Vec (+12 more)

### Community 1 - "Game"
Cohesion: 0.11
Nodes (18): Ordering, PartialEq, PartialOrd, eating_food_accumulates_freshness_score(), fitness_does_not_overflow_f32_on_large_scores(), fitness_eating_rotting_food_beats_starvation(), fitness_gen0_rewards_survival_steps(), fitness_rewards_fresh_food_more_than_rotting_food() (+10 more)

### Community 2 - "SnakeCore"
Cohesion: 0.10
Nodes (15): Into, dynamic_step_limit(), food_freshness_scales_from_one_to_zero(), get_four_dir_vision_returns_12_features(), get_random_empty_pos_avoids_body(), get_relative_state_returns_12_features(), is_snake_body_skips_head_segment(), is_wall_rejects_border_and_accepts_inner() (+7 more)

### Community 3 - "App"
Cohesion: 0.06
Nodes (14): Conf, window_conf(), Action, App, app_menu_champion_helpers_dont_panic(), AppMode, match_view_slow_toggle(), MatchInner (+6 more)

### Community 4 - "view_ga_versus.rs"
Cohesion: 0.14
Nodes (18): GaSeries, GaChampions, best_and_second_best_produce_a_match_with_the_exact_nets(), champs(), GaVersusInner, GaVersusPlayers, GaVersusView, missing_best_net_means_no_match_even_with_second_best_present() (+10 more)

### Community 5 - "ui_kit.rs"
Cohesion: 0.06
Nodes (44): Cow, Error, GameDQN, Deref, DerefMut, draw(), draw_grid(), draw_key_badge() (+36 more)

### Community 6 - "GameTheme"
Cohesion: 0.10
Nodes (29): GameTheme, load_theme(), load_theme_from_path(), Color, Option, P, Result, save_theme() (+21 more)

### Community 7 - "view_dqn_train.rs"
Cohesion: 0.09
Nodes (23): bounded_ticks_advance_an_episode_and_reset_the_board(), cleanup_test_file(), construction_loads_dqn_arch_champion_and_rejects_old_arch(), dqn_frame_tick_budget(), dqn_render_target(), DqnRenderTarget, DqnTrainView, end_episode_records_one_history_entry_from_pre_reset_steps_and_score() (+15 more)

### Community 8 - "versus.rs"
Cohesion: 0.12
Nodes (19): BestOfSeries, BestOfSeries<B>, headless_cross_match_ga_vs_dqn_terminates(), headless_match_respects_zero_tick_budget(), headless_match_with_random_brains_terminates_and_yields_winner(), headless_relative_match_terminates_and_yields_winner(), resolve_winner(), B (+11 more)

### Community 9 - "VsFlavor"
Cohesion: 0.14
Nodes (17): SeriesInfo, draw_centered_text(), format_stage_summary(), ga_default_flavor_pins_legacy_colors(), ga_default_flavor_pins_legacy_strings(), record_none_flavor_carries_no_record_semantics(), B, Color (+9 more)

### Community 10 - "dqn.rs"
Cohesion: 0.12
Nodes (14): argmax_index(), dqn_agent_net_matches_arch_and_predicts_three_outputs(), dqn_agent_q_values_can_predict_negative_values(), dqn_agent_with_network_preserves_weights_and_epsilon(), DQNAgent, ema_update(), Experience, loss_ema_starts_at_zero_and_updates_after_first_train() (+6 more)

### Community 11 - "FourDirs"
Cohesion: 0.07
Nodes (25): Clone, Send, Agent, Box<dyn Agent>, DqnPolicyAgent, GaAgent, Box, Option (+17 more)

### Community 12 - "Net"
Cohesion: 0.05
Nodes (33): Layer, matches_arch_accepts_exact_and_rejects_others(), Net, new_delegates_to_new_with_sizes_with_ga_constants(), new_with_sizes_builds_predictable_net(), Self, Vec, GenerationSummary (+25 more)

### Community 13 - "view_cross_match.rs"
Cohesion: 0.13
Nodes (20): CrossSeries, both_champions_missing_reports_both_sides(), both_champions_present_produce_a_ready_match(), cross_flavor(), cross_match_rejects_dqn_champion_with_outdated_architecture(), cross_missing_message(), CrossInner, CrossMatchPlayers (+12 more)

### Community 14 - "Viz"
Cohesion: 0.13
Nodes (12): grid_to_world(), map_to_unit_interval(), are_colors_equal(), color_with_a(), Colors, Color, Default, Instant (+4 more)

### Community 15 - "view_dqn_versus.rs"
Cohesion: 0.13
Nodes (18): DqnSeries, dqn_flavor(), DqnVersusInner, DqnVersusPlayers, DqnVersusView, nets_eq(), plan_dqn_versus(), plan_falls_back_to_fresh_greedy_agent_when_live_policy_is_absent() (+10 more)

### Community 16 - "Deep Q-Network (DQN) Implementation"
Cohesion: 0.08
Nodes (23): 1. Q-Network, 2. Target Network, 3. Experience Replay Buffer, 4. Epsilon-Greedy, Algoritmo Genético (rama `main`), 📁 Archivos generados, 🧠 Componentes DQN, 🎮 Controles (+15 more)

### Community 17 - "Apply Progress — unified-snake-shell (Slices 1–5 of 5)"
Cohesion: 0.05
Nodes (42): Apply Progress — unified-snake-shell (Slices 1–5 of 5), Completed tasks (persisted checkboxes updated), Completed tasks (persisted checkboxes updated), Completed tasks (persisted checkboxes updated), Completed tasks (persisted checkboxes updated), Completed tasks (persisted checkboxes updated), Deviations from design, Deviations from design (+34 more)

### Community 18 - "Training dashboard UI Apply Progress Slice A"
Cohesion: 0.10
Nodes (21): DQN train view requirement, GA train view with DQN-style layout requirement, Training dashboard UI Apply Progress Slice A, D-1 Module placement dqn_dash.rs, D-2 Local drawing helper duplication, D-3 DQN render target dispatcher, D-7 Per-episode history ring buffer, D-8 GA train default flip to advanced dashboard (+13 more)

### Community 19 - "game_dqn.rs"
Cohesion: 0.24
Nodes (11): body_segments_stay_contiguous_and_ordered_after_eating(), episode_can_exceed_step_limit_if_eating(), episode_terminates_when_steps_without_food_exceeds_dynamic_limit(), game_dqn_with_network_preserves_agent_weights_and_epsilon(), observation_directional_food_sensor_detects_diagonal_food(), observation_is_a_stable_9_input_relative_view_without_side_effects(), observation_places_forward_food_in_first_group_when_heading_top(), relative_step_never_reverses_direction() (+3 more)

### Community 20 - "env_config.rs"
Cohesion: 0.12
Nodes (17): env_f32(), env_f64(), env_u64(), env_usize(), init(), load_from_path(), load_from_path_reads_temporary_file_properly(), parse_and_set_env() (+9 more)

### Community 21 - "Archive Report — unified-snake-shell"
Cohesion: 0.15
Nodes (12): Archive Report — unified-snake-shell, Archived path, Artifacts read (all present), Delta requirement names (ADDED-only, 9), Destructive merge approvals, Domains synced (canonical already reflects the delta — no re-merge performed), Final-state facts, Memory observation IDs (+4 more)

### Community 22 - "Exploration: unified-snake-shell"
Cohesion: 0.15
Nodes (12): Affected Areas, Approaches, Binaries and loops, Current State, Exploration: unified-snake-shell, Game/agent mechanics parity (key finding), Product decisions (user-confirmed, 2026-09-09), Recommendation (+4 more)

### Community 23 - "Unified Snake Shell — Proposal"
Cohesion: 0.17
Nodes (11): Affected Areas, Approach (summary), Decisions & Assumptions (from the proposal question round — user-confirmed), Goals (in scope — MVP), Non-Goals (explicit, deferred), Problem Statement, Risks & Mitigations, Rollback Plan (+3 more)

### Community 24 - "Module Hierarchy & Backwards Compatibility"
Cohesion: 0.18
Nodes (10): Clean Architecture Folder Distribution Design, Directory Structure, Goal, Migration Strategy (Git & Preservation), Module Hierarchy & Backwards Compatibility, `src/domain/mod.rs`, `src/lib.rs`, `src/persistence/mod.rs` (+2 more)

### Community 25 - "Unified Snake Shell — Design"
Cohesion: 0.18
Nodes (10): Architecture decisions (with rationale), DQN champion snapshot & persistence, Module layout (new/changed files), Per-mode tick policy (one loop), Pure-logic seams for strict TDD (RED → GREEN), Simulation seam (behavior-preserving), State & transition model, Unified Snake Shell — Design (+2 more)

### Community 26 - "3. Especificación Detallada por Módulo"
Cohesion: 0.20
Nodes (9): 1. Visión General y Principios de Diseño, 2. Arquitectura de Componentes, 3.1 Primitivas Compartidas (`src/ui_kit.rs`), 3.2 Menú Principal y Temas (`src/app.rs`), 3.3 Dashboards de Entrenamiento (`src/dqn_dash.rs` y `src/viz_advanced.rs`), 3.4 Modos Versus y Partidas Cruzadas (`src/viz_vs.rs` y vistas asociadas), 3. Especificación Detallada por Módulo, 4. Plan de Verificación (+1 more)

### Community 27 - "Phase 3: Game Engine Unification (SnakeCore) — Implementation Plan"
Cohesion: 0.25
Nodes (7): Architecture, Constraints Preserved, Extracted to SnakeCore, Files Changed, Goal, Phase 3: Game Engine Unification (SnakeCore) — Implementation Plan, Verification

### Community 28 - "Global Constraints"
Cohesion: 0.25
Nodes (7): Global Constraints, Task 1: Shared UI Kit (`src/ui_kit.rs`), Task 2: Main Menu & Theme Configuration Overhaul (`src/app.rs`), Task 3: Training Dashboards Overhaul (`src/dqn_dash.rs`), Task 4: Versus & Cross-Match Arenas Overhaul (`src/viz_vs.rs` & `src/view_*_versus.rs`), Task 5: Verification and Final Polish, UI/UX Terminal Polish & Cohesive Cyber-Retro Implementation Plan

### Community 29 - "Global Constraints"
Cohesion: 0.29
Nodes (6): Clean Architecture Folder Distribution Implementation Plan, Global Constraints, Task 1: Create `src/domain/` and Migrate Domain Modules, Task 2: Create `src/persistence/` and Migrate Persistence Modules, Task 3: Create `src/presentation/views/` and `src/presentation/` and Migrate UI Modules, Task 4: Final Verification and Graphify Update

### Community 30 - "Global Constraints"
Cohesion: 0.29
Nodes (6): Global Constraints, Phase 1: Invertir Dependencias y Romper el Ciclo versus.rs <-> viz_vs.rs Implementation Plan, Task 1: Desacoplar `src/versus.rs` de la Capa de Visualización, Task 2: Implementar Helpers de Renderizado en `src/viz_vs.rs`, Task 3: Actualizar las Vistas (`view_ga_versus.rs`, `view_dqn_versus.rs`, `view_cross_match.rs`), Task 4: Verificación Integral y Actualización del Grafo

### Community 31 - "Global Constraints"
Cohesion: 0.29
Nodes (6): Global Constraints, Phase 2: Abstracción del Trait Unificado Agent Implementation Plan, Task 1: Crear el Módulo `src/agent.rs` con Trait `Agent` e Implementaciones, Task 2: Refactorizar `Game` para Usar `Option<Box<dyn Agent>>`, Task 3: Conectar `VersusMatch` con el Trait Polimórfico, Task 4: Verificación Integral y Actualización del Grafo de Graphify

### Community 32 - "Snake Rust — app unificada (crate `snake`)"
Cohesion: 0.29
Nodes (6): Champions, Ejecutar, Estructura, IA-juego-snake-rust, Snake Rust — app unificada (crate `snake`), Teclas por vista

### Community 33 - "Phase 5: Domain Layer Purification — Implementation Plan"
Cohesion: 0.33
Nodes (5): Changes Completed, Goal, Phase 5: Domain Layer Purification — Implementation Plan, Resulting Clean Architecture Layers, Verification

### Community 34 - "Canonical no regression of GA evolution behavior"
Cohesion: 0.67
Nodes (3): No regression of GA evolution behavior requirement, Modified Requirement: No regression of GA evolution behavior, Canonical no regression of GA evolution behavior

### Community 35 - "Unified snake shell verify report"
Cohesion: 0.67
Nodes (3): Unified snake shell sync report, Unified snake shell implementation tasks, Unified snake shell verify report

## Knowledge Gaps
- **179 isolated node(s):** `snake`, `graphify`, `Workflow: graphify`, `Algoritmo Genético (rama `main`)`, `DQN (rama `feature/dqn`)` (+174 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Net` connect `Net` to `Simulation`, `Game`, `view_ga_versus.rs`, `view_dqn_train.rs`, `versus.rs`, `dqn.rs`, `FourDirs`, `view_cross_match.rs`, `Viz`, `view_dqn_versus.rs`, `game_dqn.rs`?**
  _High betweenness centrality (0.225) - this node is a cross-community bridge._
- **Why does `Game` connect `Game` to `Simulation`, `SnakeCore`, `ui_kit.rs`, `versus.rs`, `VsFlavor`, `FourDirs`, `Net`, `Viz`?**
  _High betweenness centrality (0.134) - this node is a cross-community bridge._
- **Why does `SnakeCore` connect `SnakeCore` to `Game`, `ui_kit.rs`, `FourDirs`, `game_dqn.rs`, `env_config.rs`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **What connects `snake`, `graphify`, `Workflow: graphify` to the rest of the system?**
  _179 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Simulation` be split into smaller, more focused modules?**
  _Cohesion score 0.05096153846153846 - nodes in this community are weakly interconnected._
- **Should `Game` be split into smaller, more focused modules?**
  _Cohesion score 0.11379800853485064 - nodes in this community are weakly interconnected._
- **Should `SnakeCore` be split into smaller, more focused modules?**
  _Cohesion score 0.1036036036036036 - nodes in this community are weakly interconnected._
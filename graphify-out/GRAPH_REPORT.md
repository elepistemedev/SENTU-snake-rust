# Graph Report - SENTU-snake-rust-feature-dqn  (2026-09-11)

## Corpus Check
- 52 files · ~73,621 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 724 nodes · 1530 edges · 43 communities (24 shown, 19 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 49 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- GA Simulation & Champions
- App State & Navigation
- Neural Network Architecture
- Game Engine Core
- DQN Dashboard Renderer
- UI Kit & Primitives
- DQN Training View
- Versus Match Engine
- Theme Configuration & Palettes
- Snake Visuals & Rendering
- GA Training View & Config
- DQN Agent & Policy
- Population Evolution
- Color & Visualization Helpers
- Cross-Match Tournament View
- Champion Storage & Metadata
- Training Dashboard Architecture
- Deep Q-Learning Theory
- Unified Shell Specifications
- Dashboard UI Migration
- Graphify Knowledge System
- Terminal UI/UX Design
- GA Regression Guardrails
- Shell Lifecycle Verification
- Project Setup & Serena
- UI Kit Specifications
- Responsive Dashboard Specs
- Versus Arena Specs
- Observation Parity Architecture
- DQN Persistence Specs
- DQN Versus View Specs
- Cross-Match View Specs
- GA Versus View Specs
- Unified Shell App Specs
- Welcome Menu Specs
- DQN Network Diagram Seams
- Dashboard Visual Parity
- Main Menu Specifications
- DQN Reward Modeling
- Shell Workload Forecast
- Normalization Denominators
- Tab Routing & Hint Overlays
- Snake Project Root

## God Nodes (most connected - your core abstractions)
1. `Net` - 69 edges
2. `Game` - 54 edges
3. `App` - 33 edges
4. `Simulation` - 29 edges
5. `GameTheme` - 27 edges
6. `GameDQN` - 26 edges
7. `DqnTrainView` - 25 edges
8. `Viz` - 23 edges
9. `Point` - 20 edges
10. `FourDirs` - 20 edges

## Surprising Connections (you probably didn't know these)
- `Persisted Snake Champions` --semantically_similar_to--> `DQN Champion Snapshot`  [INFERRED] [semantically similar]
  README.md → DQN_README.md
- `Terminal Polish Design Philosophy` --semantically_similar_to--> `AD-5 VsFlavor Customization`  [INFERRED] [semantically similar]
  docs/superpowers/specs/2026-09-11-ui-ux-terminal-polish-design.md → openspec/changes/archive/2026-09-09-unified-snake-shell/design.md
- `Unified Snake Shell Proposal` --conceptually_related_to--> `Unified Snake Rust App`  [INFERRED]
  openspec/changes/archive/2026-09-09-unified-snake-shell/proposal.md → README.md
- `nets_approx_eq()` --references--> `Net`  [EXTRACTED]
  src/champion_store.rs → src/nn.rs
- `draw_neural_network()` --calls--> `argmax_index()`  [INFERRED]
  src/dqn_dash.rs → src/dqn.rs

## Import Cycles
- 2-file cycle: `src/versus.rs -> src/viz_vs.rs -> src/versus.rs`

## Hyperedges (group relationships)
- **DQN Reinforcement Learning Pipeline** — dqn_readme_q_network, dqn_readme_target_network, dqn_readme_experience_replay, dqn_readme_epsilon_greedy, dqn_readme_reward_system [EXTRACTED 1.00]
- **Cyber-Retro Terminal Polish Architecture** — docs_superpowers_specs_2026_09_11_ui_ux_terminal_polish_design_design_philosophy, docs_superpowers_plans_2026_09_11_ui_ux_terminal_polish_shared_ui_kit_task, docs_superpowers_plans_2026_09_11_ui_ux_terminal_polish_training_dashboards_task, docs_superpowers_plans_2026_09_11_ui_ux_terminal_polish_versus_arenas_task [EXTRACTED 1.00]
- **Unified Snake Shell State Machine & Arena Architecture** — openspec_changes_archive_2026_09_09_unified_snake_shell_proposal_unified_shell_proposal, openspec_changes_archive_2026_09_09_unified_snake_shell_design_ad1_single_binary_state_machine, openspec_changes_archive_2026_09_09_unified_snake_shell_design_ad2_shared_versus_arena, openspec_changes_archive_2026_09_09_unified_snake_shell_apply_progress_slice_execution [EXTRACTED 1.00]
- **Unified Snake Shell Lifecycle** — openspec_changes_archive_2026_09_09_unified_snake_shell_specs_app_spec_unified_single_binary_app_shell, openspec_changes_archive_2026_09_09_unified_snake_shell_tasks_implementation_phases, openspec_changes_archive_2026_09_09_unified_snake_shell_verify_report_verification_results, openspec_changes_archive_2026_09_09_unified_snake_shell_sync_report_sync_status [EXTRACTED 1.00]
- **Training Dashboard UI Lifecycle** — openspec_changes_training_dashboard_ui_proposal_scope_and_goals, openspec_changes_training_dashboard_ui_design_d1_module_placement, openspec_changes_training_dashboard_ui_specs_app_spec_req_dqn_dashboard_visual_grammar_parity, openspec_changes_training_dashboard_ui_tasks_phase_slice_2_drawing, openspec_changes_training_dashboard_ui_apply_progress_slice_b, openspec_changes_training_dashboard_ui_verify_report_verification_summary [EXTRACTED 1.00]

## Communities (43 total, 19 thin omitted)

### Community 0 - "GA Simulation & Champions"
Cohesion: 0.06
Nodes (27): GaSeries, GaChampions, load_ga_champions(), population_get_gen_summary_includes_max_steps(), Default, Option, Self, Vec (+19 more)

### Community 1 - "App State & Navigation"
Cohesion: 0.07
Nodes (12): Conf, Action, App, app_menu_champion_helpers_dont_panic(), AppMode, MatchView, next_mode(), Default (+4 more)

### Community 2 - "Neural Network Architecture"
Cohesion: 0.08
Nodes (25): DqnSeries, Layer, matches_arch_accepts_exact_and_rejects_others(), Net, new_delegates_to_new_with_sizes_with_ga_constants(), new_with_sizes_builds_predictable_net(), Self, Vec (+17 more)

### Community 3 - "Game Engine Core"
Cohesion: 0.10
Nodes (20): Into, Ordering, PartialEq, PartialOrd, Game, relative_brain_body_distance_zero_when_no_body_present(), relative_brain_observation_matches_game_dqn_observation(), Option (+12 more)

### Community 4 - "DQN Dashboard Renderer"
Cohesion: 0.10
Nodes (28): argmax_index(), draw(), draw_grid(), draw_key_badge(), draw_model_info(), draw_neural_network(), draw_stat_row(), draw_stats_panels() (+20 more)

### Community 5 - "UI Kit & Primitives"
Cohesion: 0.10
Nodes (27): Cow, calculate_chart_slot_width(), clamp_fraction(), draw_centered_text(), draw_missing_champion_notice(), draw_progress_bar(), draw_responsive_chart(), draw_terminal_box() (+19 more)

### Community 6 - "DQN Training View"
Cohesion: 0.10
Nodes (21): bounded_ticks_advance_an_episode_and_reset_the_board(), cleanup_test_file(), construction_loads_dqn_arch_champion_and_rejects_old_arch(), dqn_render_target(), DqnRenderTarget, DqnTrainView, end_episode_records_one_history_entry_from_pre_reset_steps_and_score(), existing_champion_and_metadata_persists_across_sessions_and_ignores_lower_scores() (+13 more)

### Community 7 - "Versus Match Engine"
Cohesion: 0.12
Nodes (18): B, BestOfSeries, BestOfSeries<B>, headless_cross_match_ga_vs_dqn_terminates(), headless_match_respects_zero_tick_budget(), headless_match_with_random_brains_terminates_and_yields_winner(), headless_relative_match_terminates_and_yields_winner(), inert_flavor() (+10 more)

### Community 8 - "Theme Configuration & Palettes"
Cohesion: 0.10
Nodes (19): P, GameTheme, load_theme(), load_theme_from_path(), Color, Option, Result, save_theme() (+11 more)

### Community 9 - "Snake Visuals & Rendering"
Cohesion: 0.10
Nodes (21): advance_shifts_bulge_along_body_and_evicts_at_max_len(), classify_segment(), compute_eye_offsets(), directional_eyes_offset_towards_heading(), draw_apple(), draw_connected_segment(), draw_snake_body(), draw_snake_head() (+13 more)

### Community 10 - "GA Training View & Config"
Cohesion: 0.08
Nodes (9): SimMode, frame_tick_budget(), fresh_ga_train_view_defaults_to_the_advanced_dashboard(), ga_tab_toggles_off_the_advanced_default_and_back_without_perturbing_the_sim(), GaRenderTarget, GaTrainView, render_target(), Option (+1 more)

### Community 11 - "DQN Agent & Policy"
Cohesion: 0.13
Nodes (13): dqn_agent_net_matches_arch_and_predicts_three_outputs(), dqn_agent_q_values_can_predict_negative_values(), dqn_agent_with_network_preserves_weights_and_epsilon(), DQNAgent, ema_update(), Experience, loss_ema_starts_at_zero_and_updates_after_first_train(), ReplayBuffer (+5 more)

### Community 12 - "Population Evolution"
Cohesion: 0.10
Nodes (12): GenerationSummary, Population, Instant, Option, Self, Vec, Instant, Option (+4 more)

### Community 13 - "Color & Visualization Helpers"
Cohesion: 0.12
Nodes (13): are_colors_equal(), color_with_a(), grid_to_world(), map_to_unit_interval(), Color, Colors, Color, Default (+5 more)

### Community 14 - "Cross-Match Tournament View"
Cohesion: 0.13
Nodes (20): CrossSeries, both_champions_missing_reports_both_sides(), both_champions_present_produce_a_ready_match(), cross_flavor(), cross_match_rejects_dqn_champion_with_outdated_architecture(), cross_missing_message(), CrossInner, CrossMatchPlayers (+12 more)

### Community 15 - "Champion Storage & Metadata"
Cohesion: 0.17
Nodes (14): decode(), DqnMetadata, encode(), encode_decode_round_trip_preserves_net(), load(), load_metadata(), metadata_round_trip_and_missing(), nets_approx_eq() (+6 more)

### Community 16 - "Training Dashboard Architecture"
Cohesion: 0.16
Nodes (14): DQN train view requirement, GA train view with DQN-style layout requirement, Training dashboard UI Apply Progress Slice A, D-3 DQN render target dispatcher, D-7 Per-episode history ring buffer, D-8 GA train default flip to advanced dashboard, Training dashboard UI problem statement, Requirement: DQN per-episode history bookkeeping (+6 more)

### Community 17 - "Deep Q-Learning Theory"
Cohesion: 0.25
Nodes (8): Bellman Optimality Equation, DQN Champion Snapshot, Deep Q-Network Implementation, DQN Epsilon-Greedy Exploration Policy, DQN Experience Replay Buffer, DQN Q-Network, DQN Target Network, Persisted Snake Champions

### Community 18 - "Unified Shell Specifications"
Cohesion: 0.25
Nodes (8): Unified Snake Shell 5-Slice Execution, Unified Snake Shell Archive Record, AD-1 Single Binary & AppMode State Machine, AD-3 Paused Training Lifecycle, Shell Architecture Options Exploration, Unified Snake Shell Proposal, Snake AI Project Overview, Unified Snake Rust App

### Community 19 - "Dashboard UI Migration"
Cohesion: 0.25
Nodes (8): Training dashboard UI Apply Progress Slice B, D-1 Module placement dqn_dash.rs, D-2 Local drawing helper duplication, Training dashboard UI goals and scope, Phase D: Slice 2 mirrored dashboard drawing, Training dashboard UI review workload forecast, Training dashboard UI verification report summary, OpenSpec configuration and test runners

### Community 20 - "Graphify Knowledge System"
Cohesion: 0.67
Nodes (3): Graphify Knowledge Graph, Graphify Knowledge Graph Rule, Graphify Workflow

### Community 21 - "Terminal UI/UX Design"
Cohesion: 0.67
Nodes (3): UI/UX Terminal Polish Implementation Plan, Terminal Polish Design Philosophy, AD-5 VsFlavor Customization

### Community 22 - "GA Regression Guardrails"
Cohesion: 0.67
Nodes (3): No regression of GA evolution behavior requirement, Modified Requirement: No regression of GA evolution behavior, Canonical no regression of GA evolution behavior

### Community 23 - "Shell Lifecycle Verification"
Cohesion: 0.67
Nodes (3): Unified snake shell sync report, Unified snake shell implementation tasks, Unified snake shell verify report

## Knowledge Gaps
- **46 isolated node(s):** `snake`, `Graphify Knowledge Graph`, `Graphify Workflow`, `Serena Rust Project Configuration`, `IA-juego-snake-rust-main Project Entity` (+41 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Net` connect `Neural Network Architecture` to `GA Simulation & Champions`, `Game Engine Core`, `DQN Dashboard Renderer`, `DQN Training View`, `Versus Match Engine`, `GA Training View & Config`, `DQN Agent & Policy`, `Population Evolution`, `Color & Visualization Helpers`, `Cross-Match Tournament View`, `Champion Storage & Metadata`?**
  _High betweenness centrality (0.321) - this node is a cross-community bridge._
- **Why does `Game` connect `Game Engine Core` to `GA Simulation & Champions`, `Neural Network Architecture`, `UI Kit & Primitives`, `Versus Match Engine`, `Theme Configuration & Palettes`, `Snake Visuals & Rendering`, `Population Evolution`, `Color & Visualization Helpers`?**
  _High betweenness centrality (0.179) - this node is a cross-community bridge._
- **Why does `App` connect `App State & Navigation` to `Theme Configuration & Palettes`, `GA Training View & Config`, `DQN Training View`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **What connects `snake`, `Graphify Knowledge Graph`, `Graphify Workflow` to the rest of the system?**
  _46 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `GA Simulation & Champions` be split into smaller, more focused modules?**
  _Cohesion score 0.0647307924984876 - nodes in this community are weakly interconnected._
- **Should `App State & Navigation` be split into smaller, more focused modules?**
  _Cohesion score 0.06641604010025062 - nodes in this community are weakly interconnected._
- **Should `Neural Network Architecture` be split into smaller, more focused modules?**
  _Cohesion score 0.07918552036199095 - nodes in this community are weakly interconnected._
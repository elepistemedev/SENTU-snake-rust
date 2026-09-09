# Apply Progress — training-dashboard-ui

Change `training-dashboard-ui` · artifact: apply-progress · mode: STRICT TDD (`cargo test`) · slice: **SLICE A of 2** (pure/state seams) — drawing module `dqn_dash.rs` deferred to Slice B per the parent delivery path (chained slices).

## Structured status consumed

- `applyState: ready`, artifact store `openspec` (authoritative), change root `openspec/changes/training-dashboard-ui/`.
- Implementation tasks 0/27 at start; parent resolved the delivery gate for this invocation by launching Slice A only (chained slice A → B). Delivery/PR-boundary gate stays parent-owned (task F2, `chain strategy: pending`).
- `actionContext`: mode `repo-local`, allowed edit roots = repo root, warnings none.
- Guard list (zero-diff): `viz_advanced.rs`, `sim.rs` behavior, GA evolution files, `nn.rs`, `viz_vs.rs`, versus/cross views, `main.rs`, `Cargo.toml`, `lib.rs`, `README.md`.

## Preflight baseline (task A1 — DONE)

- `git status`: only untracked `openspec/changes/training-dashboard-ui/` (this change's artifacts); no local diffs anywhere.
- `git diff --stat` on the design §4 guard list: **empty** (guard files clean).
- `cargo build`: clean. `cargo test`: **54/54 passed** (baseline).
- Baseline recorded in this apply note for verify.

## Completed this slice (persisted checkbox updates in `tasks.md`: `- [ ]` → `- [x]`)

Checked off, task ids per `tasks.md`:

1. A1 — Preflight baseline (green tree + guard zero-diff).
2. B3 — RED render-target decision: `dqn_render_target(true) == DqnRenderTarget::Dashboard`, `(false) == Hud`.
3. B4 — RED dashboard default + `Tab` round-trip on the view (tick-interleaved, no bookkeeping perturbation).
4. B5 — RED one history entry per episode end, deterministic (`end_episode` hub, pre-reset `steps`/`score`, no wall clock).
5. B6 — RED `fresh_agent` clears history, keeps champion, preserves target (extended the existing `fresh_agent_resets_epsilon_and_bookkeeping_but_keeps_champion`).
6. B9 — RED `GameDQN::observation` accessor (12 inputs, order/bounds, side-effect-free).
7. B10 — RED GA train advanced-dashboard default (`advanced_enabled() == true`, `render_target(true, Training) == Advanced`).
8. C13 — GREEN DQN render-target seam + view fields (`DqnRenderTarget{Dashboard,Hud}`, `dqn_render_target`, `dashboard: bool = true` in `new()`/`at_path()`, `history: EpisodeHistory`, `toggle_dashboard`, `dashboard_enabled`, `history()`), module doc header updated. `tick`/`on_episode_end`/`grid_layout` bodies untouched; `draw()` body untouched except a marked Slice-B placeholder comment.
9. C14 — GREEN episode bookkeeping hooks: `end_episode` pushes `(game.steps as f32, game.score)` before `episode += 1`/record logic; champion snapshot/persistence byte-identical. `fresh_agent` calls `history.clear()` (champion retained, target untouched).
10. C16 — GREEN observation seam: private `get_state` untouched; thin `pub fn observation(&self) -> Vec<f64>` wrapper added.
11. C17 — GREEN hyperparameter consts `pub` in `src/dqn.rs` (7 consts, values unchanged) + value-pin tests.
12. C18 — GREEN GA default flip: `GaTrainView::new()` sets `advanced: true`; module/enum/field/`new()` docs updated. Pacing/versus-routing/evolution untouched.
13. C19 — GREEN app shell wiring: `Tab` forwarded to `toggle_dashboard()` in `handle_dqn_train_input` (mirrors the GA arm); shell hint at `(10, screen_height()-12)` drawn only when `!dashboard_enabled()`; module header forwarded-keys bullet updated. `Esc`/`R` semantics unchanged; no new `app.rs` tests (design §5).

Persisted artifact re-read after edits: 13 rows visibly `- [x]`, matching the list above; 14 implementation rows + 3 parent rows remain unchecked (Slice B / lifecycle).

## Files changed (Slice A)

- `src/view_dqn_train.rs` — seams, ring (staged), view fields, hooks, tests. ~320 lines net.
- `src/game_dqn.rs` — `observation()` wrapper + 2 tests (+~40).
- `src/dqn.rs` — 7 consts `pub` + comment + pin tests.
- `src/view_ga_train.rs` — default flip + doc updates + 2 tests.
- `src/app.rs` — `Tab` forward, hint gating, doc bullet.
- `openspec/changes/training-dashboard-ui/tasks.md`, `apply-progress.md` (this file).
- NOT touched: guard list, `lib.rs` (no module registration this slice), `README.md`.

## Test commands run (RED → GREEN evidence)

| Step | Command | Result |
|---|---|---|
| Baseline | `cargo test` | 54 passed |
| RED (view seams/ring) | `cargo test --lib` | compile fail — 36 errors (E0425 `dqn_render_target`/`EPISODE_HISTORY_CAP`; E0433 `DqnRenderTarget`/`EpisodeHistory`; E0599 `dashboard_enabled`/`history`/`toggle_dashboard`; E0609 no field `history`) |
| GREEN (view) | `cargo test --lib` | 60 passed |
| RED (observation) | `cargo test --lib` | compile fail — E0599 no method `observation` (3 sites) |
| GREEN (observation) | `cargo test --lib observation` | 2 passed |
| RED (GA flip) | `cargo test --lib` | 62 passed, **2 failed** (`fresh_ga_train_view_defaults_to_the_advanced_dashboard`, `ga_tab_toggles_off_the_advanced_default_...`) — `advanced_enabled()` false |
| GREEN (GA flip) | `cargo test --lib` | 64 passed |
| Pin (dqn consts) | `cargo test --lib hyperparameter` | 1 passed |
| Full gate | `cargo build` + `cargo test` | clean build, **65/65 passed**, 0 warnings (one `unused mut` fixed) |
| Guard recheck | `git diff --stat` guard list | empty |

## TDD Cycle Evidence

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|---|---|---|---|---|---|---|---|
| B3 render-target decision | `src/view_dqn_train.rs` | Unit | ✅ 54/54 | ✅ compile-fail | ✅ 60/60 | ✅ 2 cases | ➖ None needed |
| B4 default + toggle round-trip | `src/view_dqn_train.rs` | Unit | ✅ 54/54 | ✅ compile-fail | ✅ 60/60 | ✅ toggle+tick interleave | ➖ None needed |
| B5 end-episode record | `src/view_dqn_train.rs` | Unit | ✅ 54/54 | ✅ compile-fail | ✅ 60/60 | ➖ Single scenario | ➖ None needed |
| B6 fresh_agent clears history | `src/view_dqn_train.rs` | Unit | ✅ 54/54 | ✅ compile-fail | ✅ 60/60 | ✅ extension of existing pin | ➖ None needed |
| B9 observation accessor | `src/game_dqn.rs` | Unit | ✅ 54/54 | ✅ compile-fail | ✅ 62/64 run | ✅ 2 setups (fresh + seeded food) | ✅ `unused mut` removed |
| B10 GA advanced default | `src/view_ga_train.rs` | Unit | ✅ 54/54 | ✅ assertion fail (2) | ✅ 64/64 | ✅ 2 tests (default + Tab cycle) | ➖ None needed |
| C17 dqn consts pub | `src/dqn.rs` | Unit | ✅ 54/54 | N/A (visibility-only; approval guards; cross-module consumer lands Slice B) | ✅ 65/65 | ➖ Single pin test | ➖ None needed |

### Test Summary
- **Total tests written this slice**: 11 (+10 new tests, +1 extended existing test). 54 → 65.
- **Total tests passing**: 65/65.
- **Layers used**: Unit (11). No integration/E2E (no harness; repo convention — drawing untested, macroquad-free seams only).
- **Approval tests (refactoring)**: 1 — `dqn.rs::hyperparameter_constants_keep_their_pre_change_values` (behavior pin for the visibility-only change). No behavior refactors performed (draw bodies untouched).
- **Pure functions created**: `dqn_render_target`; `EpisodeHistory::{new,push,clear,len}`; `GameDQN::observation` wrapper.

## Deviations / notes (design vs. this slice)

1. **`EpisodeHistory` staged in `view_dqn_train.rs`** (parent Slice-A instruction) instead of the design D-7 location `dqn_dash.rs` (tasks B2/C12). The drawing module is deliberately NOT created this slice. Slice B must relocate the ring struct + impl + its 3 tests into `dqn_dash.rs` (design D-1/D-7) and re-point the view's `use`; until then tasks B2/C12 stay unchecked. `lib.rs` untouched this slice.
2. **`draw()` keeps rendering the compact HUD in both targets** (visible DQN output unchanged, per parent): a `// SLICE B placeholder` comment marks where the design D-3 dispatcher + private `draw_hud` refactor land. Consequently the interim tree has `dashboard_enabled() == true` (default) while draw() ignores it and the shell hint is gated off — coherent only after Slice B; noted for the reviewer.
3. **Naming aligned to design D-3 / tasks artifact**, not the loose Slice-A brief: `DqnRenderTarget::{Dashboard, Hud}` + `dqn_render_target(dashboard_enabled: bool)` + `toggle_dashboard()`/`dashboard_enabled()` (brief said `Compact`/`toggle_render_target`). `EpisodeHistory::push(time: f32, score: usize)` per design D-7/tasks (brief said `push(score, steps)`); chart 1 duration = step count at completion (spec wins over design D-7's wall-clock `Instant` — reconciliation note in tasks.md honored; no `Instant` field exists).
4. **`dqn.rs` visibility RED deferred**: the pin test is in-file (cannot observe `pub` from a child module); the first cross-module consumer is the Slice-B `dqn_dash.rs` model-info panel, where any missed `pub` is a compile error.
5. No wall-clock timers anywhere; `end_episode` test seeds `steps = 71` → asserts `times == [71.0]`.
6. GA flip tests construct `GaTrainView::new()` (≈1000 lightweight games in `Population`); verified `Simulation::new`/`Population::new` only load — read-only w.r.t. `sim_metadata.json`/`best_snake.json`, no writes, ~ms runtime.

## Remaining implementation tasks (unchecked, exact lines — Slice B + closure)

- `- [ ] RED — EpisodeHistory ring (design seam 3, ...)` — relocate staged ring tests to `dqn_dash.rs`.
- `- [ ] RED — output_intensity clamp (...)`
- `- [ ] RED — score-bar fraction against the documented DQN bound (...)`
- `- [ ] Register the new module: add pub mod dqn_dash; to src/lib.rs (...)`
- `- [ ] GREEN — EpisodeHistory + EPISODE_HISTORY_CAP in src/dqn_dash.rs (D-1/D-7)`
- `- [ ] GREEN — pure color/bar helpers in src/dqn_dash.rs (...)`
- `- [ ] GREEN — mirror skeleton and geometry copy (D-1/D-2)`
- `- [ ] GREEN — left column: live grid + draw_grid (D-4)`
- `- [ ] GREEN — center "NEURAL NETWORK" panel + draw_neural_network (D-4)`
- `- [ ] GREEN — model-info panel + draw_model_info (D-4)`
- `- [ ] GREEN — right column stats, bars, and history charts (D-4/D-7)`
- `- [ ] GREEN — entry point + draw routing (D-3/D-4)` — turn the `draw()` placeholder into the dispatcher + `draw_hud`.
- `- [ ] Full green gate (...)`
- `- [ ] Zero-diff guard recheck (...)`
- Parent-owned (deferred, 3): bounded post-apply review (diff `dqn_dash.rs` vs `viz_advanced.rs` + RED-first commit history), delivery gate (ask-on-risk, `chain strategy: pending`), user visual smoke.

## Workload / PR boundary

Slice A delta as staged ≈ 511 insertions / 38 deletions across 5 src files (includes the ~150 ring/`EpisodeHistory` test lines and ~85 seam lines that Slice B relocates into `dqn_dash.rs`; after relocation the net Slice-A footprint matches design §7 slice-1 (~150 lines) plus tests). Slice B brings the isolated `dqn_dash.rs` drawing (~250). Full change forecast stands at design §7 (~415–490 final lines vs the 400 budget; risk High — parent-owned delivery gate already open, NOT resolved here).

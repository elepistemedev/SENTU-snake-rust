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

## Slice B completed (this invocation — persisted checkbox updates in `tasks.md`)

Checked off, task ids per `tasks.md` (all 14 remaining implementation rows now `- [x]`; parent rows untouched):

1. B2 — RED `EpisodeHistory` ring in `dqn_dash.rs` (tests only, module NOT yet registered).
2. B7 — RED `output_intensity` clamp (`−0.5→0.0`, `0.5→0.5`, `1.5→1.0`).
3. B8 — RED score-bar fraction over the documented DQN bound (`0→0.0`, bound→1.0, bound+50→1.0`).
4. C12 — Register `pub mod dqn_dash;` in `src/lib.rs` (alphabetical: after `dqn`, before `game`).
5. C12-ring — GREEN `EpisodeHistory` + `EPISODE_HISTORY_CAP: usize = 50` in `src/dqn_dash.rs` (D-1/D-7; relocated from the view per design — the view now re-imports `use crate::dqn_dash::{self, EpisodeHistory};`).
6. C12-helpers — GREEN pure helpers `output_intensity` (pub) + `score_bar_fraction` (private, `FULL_BAR = (NUM_SIM_STEPS * 2) as f32`); module doc names both and their callers.
7. D20 — GREEN mirror skeleton: consts (`PANEL_BG/PANEL_BORDER/TEXT_COLOR/ACCENT_COLOR/TITLE_SIZE/TEXT_SIZE`) + `draw_panel`, copied verbatim from `viz_advanced.rs` lines 6–11 / `draw_panel`.
8. D21 — GREEN left column `draw_grid` (reference `draw_game_grid` math; single live snake, no ghost/rank loop).
9. D22 — GREEN center "NEURAL NETWORK" `draw_neural_network` (fed live via `predict(&game.observation())`; `output_intensity` colors).
10. D23 — GREEN model-info `draw_model_info` ("DQN TRAIN"; all rows from the pub seams; accent controls line at the reference's `panel_y + 240` slot).
11. D24 — GREEN right column `draw_stats_panels` + `draw_chart` (bars over `score_bar_fraction`, reference panel slots, self-normalizing charts).
12. D25 — GREEN entry `pub fn draw(game, episode, best_score, history)` + view dispatcher refactor (`draw()` → `match dqn_render_target`, compact body moved unchanged to private `draw_hud`).
13. E26 — Full green gate: `cargo build` clean (0 warnings), `cargo test` 67/67 (see TDD table below).
14. E27 — Zero-diff guard recheck + spec-normalization greps (see below).

## Slice B — files changed

- `src/dqn_dash.rs` — NEW (module doc, `EpisodeHistory` + `EPISODE_HISTORY_CAP`, `output_intensity`, `score_bar_fraction`, mirror consts/`draw_panel`, `draw_grid`, `draw_neural_network`, `draw_model_info`, `draw_chart`, `draw_stats_panels`, pub `draw`, `#[cfg(test)]` pure tests: 3 ring + clamp + fraction).
- `src/view_dqn_train.rs` — ring removed (relocated), `use crate::dqn_dash::{self, EpisodeHistory};` added, `draw()` → D-3 dispatcher + private `draw_hud` (compact body byte-identical), 3 ring tests removed (moved to `dqn_dash.rs`). Net −129/+25.
- `src/lib.rs` — `pub mod dqn_dash;` (+1).
- `openspec/.../tasks.md` + `apply-progress.md` (this file).
- NOT touched: guard list, `README.md`, `app.rs`/`view_ga_train.rs`/`game_dqn.rs`/`dqn.rs` (all Slice-A-committed, zero diff this slice).

## Slice B — TDD Cycle Evidence (strict TDD, `cargo test`)

| Task | Test File | Layer | Safety Net | RED | GREEN | TRIANGULATE | REFACTOR |
|---|---|---|---|---|---|---|---|
| B2 ring relocation (RED) | `src/dqn_dash.rs` | Unit | ✅ 65/65 | ✅ compile-fail on registration (10 errors: E0425 `output_intensity`/`score_bar_fraction`, E0433 `EpisodeHistory`, E0425 `EPISODE_HISTORY_CAP` — dqn_dash registered, seams absent) | ✅ 70/70 (dqn_dash ring tests pass) | ✅ 3 scenarios (cap+evict, index-alignment, clear) | ✅ ring removed from view; view re-imports `dqn_dash::EpisodeHistory` → 67/67 |
| B7 output_intensity clamp | `src/dqn_dash.rs` | Unit | ✅ 65/65 | ✅ compile-fail (fn absent) | ✅ | ✅ 3 boundaries (−0.5/0.5/1.5) | ➖ None |
| B8 score-bar fraction | `src/dqn_dash.rs` | Unit | ✅ 65/65 | ✅ compile-fail (fn absent) | ✅ | ✅ 0/bound/bound+50 + config pin `200.0` | ➖ None |
| D20–D25 drawing + dispatcher | `src/dqn_dash.rs`, `src/view_dqn_train.rs` | — (drawing untested by repo convention) | ✅ 67/67 | n/a (no macroquad harness) | verified by `cargo build` clean + geometry self-review vs `viz_advanced.rs` (below) + pending user visual smoke | n/a | `draw()` → dispatcher + `draw_hud` (body moved unchanged; nothing deleted) |

Test totals: Slice A 65 → Slice B **67** (net +2: +5 new dqn_dash tests, −3 ring tests relocated from the view module). Pure fns final: `dqn_render_target`, `EpisodeHistory::{new,push,clear,len}`, `output_intensity`, `score_bar_fraction`, `GameDQN::observation`.

## Slice B — line-by-line geometry self-review (dqn_dash vs the read-only viz_advanced.rs)

Reference file `src/viz_advanced.rs` (verified by `git diff` = empty):

| Reference element | Reference line | Mirror (dqn_dash) | Verdict |
|---|---|---|---|
| Colors/consts | 6–11 (`PANEL_BG` 0.05/0.05/0.05/0.95; `PANEL_BORDER` 0.4/0.4/0.4; `TEXT_COLOR` 0.9; `ACCENT_COLOR` 0.0/0.9/0.9; `TITLE_SIZE` 22.0; `TEXT_SIZE` 18.0) | identical consts (private) | ✅ verbatim |
| Panel body | `draw_panel` — bg rect, 3.0 border rect-lines, accent title at `x+20/y+30` TITLE_SIZE | identical body | ✅ verbatim |
| Grid | `draw_game_grid` — `x=20,y=20`, `grid_size=screen_h−340`, `tile=grid_size/GRID_W`, border `x−8..+16`, BLACK field, 0.15 grid lines, food `+2 inset tile−4` WHITE, snake `+1 inset tile−2` | identical math; food/snake of the live `GameDQN`; rank-0 colors head (0.3,0.9,0.3)/body (0.2,0.7,0.2) | ✅ (ghost/rank loop intentionally absent — DQN has no top-N) |
| NN panel | `draw_neural_network` — `left_col_width=screen_h−320`, `panel_x=left_col_width+40`, `panel_w=550`, `panel_h=screen_h−40`, `y=20`; title `+20/+28`; input `x+80`, `start_y +80`, spacing `(panel_h−160)/(INP−1)` r7 accent, `I{i}` at `x−35/y+6/18`; hidden `+160`, `start_y +150`, spacing `(panel_h−300)/(HID−1)` r8 (0.9,0.6,0.0), `H{i}`; output `+160`, `start_y +250`, spacing `(panel_h−500)/(OUT−1)` r10, labels `["LEFT","RIGHT","BOTTOM","TOP"]` at `x+22/y+7/20`; all-connection lines 1.5 (0.3,0.3,0.3,0.2) | identical formulas/labels; node counts from `configs`; feed = `q_network.predict(&game.observation())` `.last()`; output color `output_intensity(value)` → `(i, i*0.3, i*0.9)` | ✅ (data source diverges by design D-4/D-5) |
| Model info | `draw_model_info` — `x=20, y=screen_h−300, w=screen_h−320, h=280`; controls accent row baseline at `panel_y+240` | same panel geometry; 6 content rows from pub seams at `panel_y+55`, pitch 26; controls row baseline `panel_y+240` (same slot as reference); title "DQN TRAIN" | ✅ content rows/title differ (DQN data), controls geometry same |
| Right column | `draw_stats_panels` — `panel_x=left_col_width+550+60`, `panel_w=screen_w−panel_x−20`; slots SIM@20/h150, VIZ@200/h130, bar1@350/h100 (bar h30 at +50, `w−40` track, label `panel_x+panel_w/2−20` at `y+21`/18/WHITE), bar2@470/h100, charts@600 with `chart_h=(screen_h−y−20)/2−10` | panel x/w/slots identical; rows = EPISODE STATS (Episode/Best/Epsilon), RUN STATS (Score/Steps); bars "SCORE" (MAGENTA) & "BEST" (RED) via `score_bar_fraction` (`{:.0}%`); charts "EPISODE TIMES"(SKYBLUE, `history.times`) & "EPISODE SCORES"(GREEN, f32-cast) with identical `draw_chart` internals | ✅ row counts differ (fewer DQN rows), everything else grammar-identical |
| Chart internals | `draw_chart` — `chart_x/y=+20/+50`, `chart_w=w−40`, `chart_h=h−70`, `max_val=max(series).max(1.0)`, `step=chart_w/len`, bar `w=step.max(4)−1`; empty→early return | identical body | ✅ verbatim (empty/all-zero safe — spec) |
| Entry order | `VizAdvanced::draw` — grid, model-info, NN, stats | `draw` starts `clear_background(BLACK)` (mirroring `sim.rs::draw_advanced`), then grid, model-info, NN, stats | ✅ |

## Slice B — deviations / notes (design vs this slice)

1. **Strict-TDD run order note**: RED tests were observable only after `pub mod dqn_dash;` registration (tasks B2/B7/B8 write the file; C12 registration triggers the compile-fail RED; GREEN fills the seams). Ring relocation is folded into the GREEN ring task — the 3 ring tests now live in `dqn_dash.rs` with identical assertions; the view's staged copy (struct + impl + const + 3 tests) was deleted and re-imported. View diff is net −104 lines.
2. **Right-column slots preserved exactly**: because DQN's two stat panels hold fewer rows than GA's, the reference's *incremental* y rhythm would have shifted every panel below them; instead the mirror keeps the reference's absolute panel slots (RUN@200, SCORE bar@350, BEST@470, charts@600) so the whole right column lands on the same pixels as GA's dashboard — the strongest grammar parity. Recorded for the verify-phase review.
3. **Percent label** uses `{:.0}%` (rounded) per the tasks text where the reference truncates via `(pct*100.0) as i32`; geometry/position identical.
4. **Bar denominators**: `score_bar_fraction` computes `FULL_BAR = (NUM_SIM_STEPS * 2) as f32` (200.0) from the config seam — no `20.0`/`200.0` literal in any drawing code path (the only `200.0` in the file is the `#[cfg(test)]` pin asserting the derived bound). GA's `/20` never appears.
5. **`draw_hud`** holds the former compact `draw()` body unchanged (moved verbatim, still reachable via `Tab`); nothing was deleted from the compact path. `dashboard_enabled()`-gated shell hint (Slice A) now only ever shows with the HUD, matching the dashboard controls line.
6. Module doc links: `draw_hud` is linked as [`DqnTrainView::draw_hud`] from the `draw` doc; `output_intensity`/`score_bar_fraction` are linked from the module doc and their fn docs name their callers.

## Remaining tasks (persisted artifact — exact unchecked lines)

All implementation rows are `- [x]`. Three parent-owned lifecycle rows remain unchecked (deferred — this phase never runs them):

- `- [ ] Start or reuse a bounded post-apply review: diff src/dqn_dash.rs drawing against the read-only src/viz_advanced.rs geometry/color consts line-by-line, and confirm RED-first evidence exists for every seam in section B (failed-test-then-passed commit history per work-unit-commits).` <!-- sdd-owner: parent -->
- `- [ ] Delivery gate (ask-on-risk, NOT auto-resolved): ... record the user's choice before applying/merging.` <!-- sdd-owner: parent -->
- `- [ ] User visual smoke (acceptance, verify phase): run the app — ...` <!-- sdd-owner: parent -->

## Workload / PR boundary

Slice B delta (unstaged, this invocation): `src/dqn_dash.rs` new ≈ 640 lines (incl. doc/tests), `src/view_dqn_train.rs` −104 net, `src/lib.rs` +1. Design §7 slice-2 forecast was ~250 for `dqn_dash.rs` drawing; the as-written module is larger (full doc + 5 relocated+new pure tests + verbatim geometry) — the parent-owned 400-line/PR-boundary delivery gate (task F2, `ask-on-risk`, `chain strategy: pending`) is where the final PR split/exception decision still rests. Full change (slices A+B) stands at the design §7 forecast; nothing here invents a chain or an exception.


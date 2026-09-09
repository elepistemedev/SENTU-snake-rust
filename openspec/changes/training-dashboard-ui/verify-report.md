# Verify Report — training-dashboard-ui

Change `training-dashboard-ui` · artifact: verify-report · branch `feature/dqn-vs-genetico` · commits `fa2b654` (Slice A) + `a28a2fc` (Slice B) on `9d57f6a`.

**Status: PASS (logic-level) — with open parent gates.** All implementation tasks complete (27/27, zero unchecked implementation rows), build clean with 0 warnings, `cargo test` 67/67 green, zero-diff guard verified empty, strict-TDD evidence coherent. The user-facing visual acceptance (dashboard grammar vs `ui/ui-version4.png`, live Q-colored outputs) **cannot be verified headless** and remains the parent-owned smoke gate (tasks F2/F3). Archive is **not** ready: the parent-owned delivery-gate and visual-smoke rows are still open, and `sync`/`archive` are blocked in status until then.

## Status / structured context consumed

- Native status (`sdd-status`): change `training-dashboard-ui`, state `ready` for verify; `applyState: all_done`; implementation tasks 27/27, `unchecked: []`; deferred parent actions 3, all unchecked (F1 post-apply review, F2 delivery gate `ask-on-risk` `chain strategy: pending`, F3 user visual smoke).
- `actionContext`: mode `repo-local`, workspace/allowed-edit-root = repo root, no warnings. Verify ran read-only (no code edits; builds only).
- Config: `openspec/config.yaml` — `strict_tdd: true`, verify command `cargo test`. Global support file `.pi/agent/gentle-ai/support/strict-tdd-verify.md` applied (no project-local override).

## Commands run (exact)

| Command | Result |
|---|---|
| `git status` | clean working tree on `feature/dqn-vs-genetico` |
| `git diff --stat 9d57f6a HEAD -- src/viz_advanced.rs src/sim.rs src/game.rs src/pop.rs src/stream.rs src/nn.rs src/viz_vs.rs src/view_dqn_versus.rs src/view_ga_versus.rs src/view_cross_match.rs src/main.rs Cargo.toml README.md` | **empty** (zero-diff guard confirmed, both slices cumulatively) |
| `cargo clean && cargo build --all-targets` | clean, **0 warnings** |
| `cargo test` | **67 passed, 0 failed** (`src/lib.rs` 67; main/doc 0) — matches Slice-B record 54 → 65 → 67 |
| Per-commit stats | `fa2b654`: seams+flip+docs+tests (view_dqn_train/app/game_dqn/dqn/view_ga_train + artifacts); `a28a2fc`: `dqn_dash.rs` new 638 + `lib.rs` +1 + view relocation (−153) |

## Requirement coverage (spec `specs/app/spec.md`)

| Requirement | Verdict | Evidence |
|---|---|---|
| DQN train default render = dashboard; `Tab` toggles dash↔HUD; shell forwards Tab; Esc/resume via pure table unchanged | ✅ | `dashboard: true` in `new()`/`at_path()`; `draw()` dispatches `dqn_render_target(self.dashboard)` → `dqn_dash::draw` / private `draw_hud` (legacy body preserved); `app.rs handle_dqn_train_input` forwards `Tab` → `toggle_dashboard()`; `Esc` still `next_mode(…Esc)` → `ToMenu`, transition table diffed unchanged; resume semantics intact (tests `fresh_view_defaults…`, render-target maps) |
| DQN live data seams: observation accessor + live `predict`, hyperparams pub | ✅ | `GameDQN::observation()` thin wrapper over private `get_state` (unchanged internals); `dqn_dash` NN fed `q_network.predict(&game.observation()).last()` every frame; 7 `dqn.rs` consts `pub` with values pinned by test. Diff shows visibility-only + wrapper + tests |
| History ring: cap 50, one entry/episode pre-reset, step-count duration, clear on `fresh_agent`, charts fed, survives pause/resume | ✅ | `EpisodeHistory` in `dqn_dash.rs` (pub, macroquad-free), `end_episode` pushes `(game.steps as f32, game.score)` before `episode += 1`/reset (spec wins over design D-7 wall-clock — reconciliation note honored, no `Instant` field); `fresh_agent` clears ring only (champion + target kept); 3 ring tests + record test |
| Bar/chart denominators: `(NUM_SIM_STEPS*2)` bound, clamped, no GA `/20` | ✅ | `score_bar_fraction` derives `FULL_BAR` from `configs::NUM_SIM_STEPS`; no `20.0`/`/20` denominator in any drawing path (grep-verified — matches only geometry offsets, docs, and the `#[cfg(test)]` pin `200.0`); charts self-normalize `max(series).max(1.0)` with empty early-return |
| GA train defaults to advanced `VizAdvanced`; `render_target` vs-precedence unchanged; Tab toggles | ✅ | `GaTrainView::new()` `advanced: true` (+doc updates only); `render_target` body untouched; VS still routes to `Versus` (tests); persistence cadence untouched (`sim.rs`/pop/stream guard-clean) |
| Zero-diff guard | ✅ | Guard list empty across both commits (`viz_advanced.rs`, `sim.rs`, `game.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `viz_vs.rs`, both versus views, `view_cross_match.rs`, `main.rs`, `Cargo.toml`, `README.md`). `lib.rs` is NOT on the guard list and gained the intended `pub mod dqn_dash;`. `view_ga_train.rs` is not a guard file — its flip is the intended Slice-A change |
| Visual grammar parity + smoke | ⚠️ logic ✅ / visual pending | Line-by-line geometry/color comparison of `dqn_dash.rs` vs `viz_advanced.rs` matches (consts, panel slots, grid math, NN layer positions/labels, right-column absolute slots equal the reference's incremental result on the same pixels, chart internals, controls baseline at `panel_y+240`). Pixel-level acceptance is a **user smoke** (below) |

## Task completion

Implementation rows: **27/27 `- [x]`** (grep-confirmed). No unchecked implementation task lines exist; no CRITICAL from checkboxes.

Remaining unchecked rows (parent-owned lifecycle, `sdd-owner: parent`, deferred — not implementation, not archive blockers from the checkbox rule alone):
- `- [ ] Start or reuse a bounded post-apply review: diff src/dqn_dash.rs drawing against the read-only src/viz_advanced.rs … confirm RED-first evidence exists for every seam in section B (failed-test-then-passed commit history per work-unit-commits).` (line 81)
- `- [ ] Delivery gate (ask-on-risk, NOT auto-resolved): … ~415–490 lines vs the 400 budget (risk High) … record the user's choice before applying/merging.` (line 82)
- `- [ ] User visual smoke (acceptance, verify phase): run the app …` (line 83)

**Archive is not ready**: clean verify is only one prerequisite — `sync` and `archive` are blocked until the parent-owned delivery decision, post-apply review, and user visual smoke complete.

## Strict-TDD compliance (strict_tdd: true)

### TDD Compliance
| Check | Result | Details |
|---|---|---|
| TDD evidence reported | ✅ | `TDD Cycle Evidence` tables present in `apply-progress.md` for Slice A (7 rows) and Slice B (4 rows) |
| Test files exist / RED confirmed | ✅ | Every RED row names a test file that exists: `view_dqn_train.rs`, `game_dqn.rs`, `view_ga_train.rs`, `dqn.rs`, `dqn_dash.rs` — all verified |
| GREEN confirmed on execution | ✅ | `cargo test` → 67/67; count chain 54 → 65 → 67 reproduced exactly |
| Triangulation adequate | ✅ | multi-case: render-target map (2), toggle+tick interleave, ring (3 scenarios), clamp (3 boundaries), fraction (0/bound/bound+50 + config pin), observation (2 setups), GA default (2 tests) |
| Safety net | ✅ | baseline 54/54 recorded before each slice |
| REFACTOR | ➖ | per support module, not verifiable; recorded rows are "None needed" or honest (view ring relocation, `unused mut`) |

**Note for parent task F1**: RED evidence is documentary (apply-progress command logs: E0425/E0433/E0599 compile-fails → GREEN, GA assertion-fails → GREEN). Each slice is a **single squashed commit**, so intermediate RED states are **not present in git history**; F1's "failed-test-then-passed commit history per work-unit-commits" cannot be confirmed from the branch alone — only from the apply records.

### Test Layer Distribution
| Layer | Tests | Files | Tools |
|---|---|---|---|
| Unit | 67 | 5 changed + pre-existing suites | `cargo test` |
| Integration | 0 | — | none (repo has no harness) |
| E2E / visual | 0 | — | none — **drawing is untested by repo convention** (macroquad-free seams only); visual acceptance deferred to user smoke |

### Coverage
Coverage analysis skipped — no coverage tool detected (`config.yaml` coverage commands empty). Informational.

### Assertion Quality Audit (Step 5f)
Audited all new/changed test code (`dqn_dash.rs` ×5, `view_dqn_train.rs` new ×3 + extended ×1, `game_dqn.rs` ×2, `dqn.rs` ×1, `view_ga_train.rs` ×2). No tautologies, no ghost loops (the index-alignment loop iterates a vector pre-asserted `len() == 50`), no type-only or smoke-only assertions, no implementation-detail CSS/mock assertions. Assertions exercise real production code and pin exact values (eviction `10.0/100` first + `59.0/590` last; `end_episode` exact `times == [71.0]`, `scores == [5]`; clamp `−0.5→0.0 / 0.5→0.5 / 1.5→1.0`; fraction `0→0.0 / 200→1.0 / 250→1.0`; hyperparameter pins; observation 12-input order/bounds/stability/side-effect-free).

**Assertion quality**: ✅ All assertions verify real behavior (0 CRITICAL, 0 WARNING).

**Coverage note (WARNING-level, pre-existing gap, no code change this change)**: spec scenario "replay training guard" (MODIFIED DQN requirement) has **no dedicated test anywhere** (`grep` over `src/` for guard/fewer-than/batch found none); the scenario is satisfied by unchanged `dqn.rs::train` code inspection only. Behavior is untouched by this change, so not a blocker — parent decides whether to add a pin.

## Review workload / PR boundary

- Forecast (tasks/design §7): ~415–490 changed lines; **actual code diff = 1,049 insertions / 42 deletions** across `src/` (`dqn_dash.rs` 638; `view_dqn_train.rs` +202 net; `game_dqn.rs` +87; `view_ga_train.rs` +46; `app.rs` +10; `dqn.rs` +23; `lib.rs` +1). Even excluding the ~380 test/doc lines, the drawing module alone (638 incl. docs + 5 pure tests) far exceeds the ~250–300 design estimate for Slice 2. Same scope, no feature additions beyond spec (all rows map to spec/design items), but **review workload materially exceeds the 400-line budget**.
- Slices were built as two commits on the feature branch, but the **PR/chain boundary is still undecided** — `chain strategy: pending`, delivery gate F2 (`ask-on-risk`) open. Nothing here invents a chain or a `size:exception`.
- Flag: WARNING (workload) — mitigation owned by F2. Verified scope matches the assigned tasks only; no out-of-task creep.

## Deviations (intended or documented; no requirement breach)

1. `dqn_dash.rs` is ~638 lines vs the design D-1/§7 ~300 estimate (verbatim geometry, module docs, 5 relocated+new pure tests) — recorded in apply-progress Slice B note; workload handled at F2.
2. Right column uses the reference's **absolute** panel slots (EPISODE@20, RUN@200, SCORE bar@350, BEST@470, charts@600) instead of incremental `y` accumulation — arithmetic-verified equal to the reference's final pixel positions (GA's increments land on the same slots); deliberate grammar-parity choice (apply-progress deviation 2).
3. Percent label `{:.0}%` (rounded) vs reference `(pct*100.0) as i32` truncation; geometry identical (apply-progress deviation 3).
4. `EpisodeHistory` staged in `view_dqn_train.rs` during Slice A then relocated to `dqn_dash.rs` in Slice B per design D-1/D-7 — final state matches design.
5. Cosmetic: several changed regions are not rustfmt-normalized (indentation inside `view_dqn_train.rs` enum docs, `app.rs` `draw_dqn_train`, `dqn_dash.rs` test asserts); compiles clean, 0 warnings. SUGGESTION only.

## What the user must smoke-test (cannot be verified headless)

Run `cargo run` (fullscreen; reference screenshot `ui/ui-version4.png` exists):

1. **Menu → 1 DQN Train**: opens on the dashboard by default — left: live single-snake grid over "DQN TRAIN" model-info (Architecture 12x8x4, Replay fill, Batch/Gamma, LR, Epsilon schedule, Step Limit 200, controls line); center: live "NEURAL NETWORK" with I0..I11 / H0..H7 and 4 output nodes LEFT/RIGHT/BOTTOM/TOP whose Q-colored intensity **changes mid-episode**; right: EPISODE STATS / RUN STATS / SCORE + BEST bars / EPISODE TIMES + EPISODE SCORES charts filling as episodes complete.
2. **Tab** in DQN train: toggles dashboard ↔ compact HUD and back; bottom-left shell hint `[R] fresh agent [ESC] menu` appears **only on the compact HUD**, never over the dashboard model-info panel.
3. **R** (fresh agent): resets agent/episode/best/history, keeps champion, does **not** flip the render target. **Esc** from either DQN target returns to the menu (trainer paused, not quit); re-entering (1) resumes the same session on the same target.
4. **Menu → 3 GA Train**: opens on the untouched `VizAdvanced` dashboard; **Tab** reaches the compact HUD (Generation/Gen Max/Best Ever/Elapsed/champion metrics + grid) and back; generation/persistence (every 10th generation writes `best_snake.json`/`sim_metadata.json`) unchanged; `V` internal VS still routes to the side-by-side versus renderer.
5. Both dashboards read as the same grammar as `ui/ui-version4.png` with their own data; **no overlapping or off-screen panels** at the default (fullscreen) window size.

## Exact blockers

None at verify level. Logic-level verification is a clean PASS; sync/archive remain blocked by the parent-owned gates: F1 post-apply review (RED-history confirmation), F2 delivery decision (`ask-on-risk`, risk High, ~1,049 vs 400-line budget), and F3 user visual smoke (steps above).

## Key Learnings

1. Verify kept read-only on the feature branch and reproduced 67/67 tests plus a clean warning-free build from scratch after cargo clean.
2. Geometry parity between dqn_dash.rs and viz_advanced.rs holds because absolute panel slots land on the reference's incremental-y final pixels.
3. The zero-diff guard list excludes view_ga_train.rs and lib.rs by design, which are the intended GA flip and module registration.
4. RED-first strict-TDD evidence is documentary only because each slice was committed squashed, so git history cannot confirm failed-test-then-passed cycles.
5. Visual dashboard acceptance cannot be verified headless, so the user smoke steps are the only gate for pixel-level grammar parity.

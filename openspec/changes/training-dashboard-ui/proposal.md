# Training Dashboard UI — Proposal

Change: `training-dashboard-ui` (follow-up on the archived `unified-snake-shell` MVP)

## Problem Statement

The user reported that the DQN training view in the unified app renders the *wrong* UI. The reference they want (screenshot `ui/ui-version4.png`, gitignored) is the GA **advanced training dashboard** rendered by `src/viz_advanced.rs` (the original GA app on branch `main`): left column big game grid + bottom model-info panel, center "NEURAL NETWORK" panel (I0..I11 → H0..H7 → LEFT/RIGHT/BOTTOM/TOP with colored activations), right column stats panels + VIZ SCORE/MAX SCORE bars + history bar charts.

Today, after the unified-shell MVP, both training views default to the *old compact* single-grid + text-HUD layout family: `DqnTrainView::draw` renders the `main_dqn`-style HUD unconditionally, and `GaTrainView` defaults to its DQN-style compact view with `Tab` toggling the real `VizAdvanced` dashboard (via `Simulation::draw_advanced`). The user decision: **both training views must show the advanced dashboard style by default** — DQN train gets a new dashboard mirroring the `viz_advanced` layout fed by live DQN data, and GA train defaults to the pre-existing advanced dashboard (as the original `main` app did), with `Tab` toggling to the compact view in both.

## Goals (in scope)

1. **DQN train advanced dashboard (new).** A new mirrored dashboard module rendering the `viz_advanced` visual grammar (same colors/panels/bars/charts/geometry family) filled with DQN data:
   - Left column: the live single-snake `GameDQN` grid (large) + a bottom model-info panel showing DQN hyperparameters and the architecture (12×8×4), replay-buffer fill, and controls.
   - Center column: "NEURAL NETWORK" panel I0..I11 → H0..H7 → LEFT/RIGHT/BOTTOM/TOP; input/hidden node labels as in the reference; the four output nodes colored by the live q_network Q-values on the current observation (same sigmoid-domain, so the reference color mapping carries over unchanged).
   - Right column: DQN stats panels (Episode, Score, Best, Epsilon), score bars, and per-episode history bar charts (a bounded ring, mirroring the 50-entry cap of `VizAdvanced`).
   - The dashboard is the **default** DQN-train render; `Tab` toggles to the legacy compact HUD (view stays reachable, nothing deleted).
2. **DQN rendering/state seams (additive only).** Expose the current 12-input observation from `GameDQN` (`get_state` is today private; visibility-only change or thin accessor) and make the DQN hyperparameter constants in `src/dqn.rs` `pub` (visibility-only) so the model-info panel shows real values, not literals. No logic change to learning/episode/champion behavior.
3. **GA train default flip.** `GaTrainView` defaults to `advanced = true` (the pre-existing `VizAdvanced` dashboard via `Simulation::draw_advanced`, byte-identical to the original `main` app); `Tab` toggles to the compact DQN-style HUD. The existing pure `render_target` rule (a running sim-internal VS sub-state always routes to the versus renderer) is unchanged.
4. **App shell wiring.** Forward `Tab` to the DQN train view (as today for GA), keep `R` (fresh agent) and `Esc` working, and keep the shell hint overlay off the dashboard's bottom model-info panel (either drawn by the dashboard itself like the reference's controls line, or overlaid only on the compact render).
5. **Behavior preservation.** DQN training loop, episode bookkeeping, champion snapshot/persistence, GA evolution, auto-VS cadence, and file persistence all behave exactly as today; `viz_advanced.rs`, `sim.rs`, `game.rs`, `pop.rs`, `stream.rs`, and the GA vs/cross views are **not modified**.

## Non-Goals (explicit, deferred)

- **Not touching verified GA rendering/evolution:** zero diffs to `viz_advanced.rs`, `sim.rs`, `pop.rs`, `stream.rs`, `game.rs`; their current output is the "correct" reference and stays byte-identical.
- **No refactor of `VizAdvanced`:** helpers may be duplicated locally in the new DQN dashboard module (preferred, lowest risk); extracting shared helpers is allowed only if strictly behavior-preserving — it is not required to satisfy this change.
- **No shared game-core refactor** (game.rs/game_dqn.rs duplication, vision encoding normalization) — out of scope here as in the archived MVP.
- **No pixel-exact clone of `ui-version4.png`:** the reference shows GA-only data (streams/agents counts, top-N ghost snakes) that has no DQN equivalent; the mirror keeps the layout/color/panel grammar and substitutes DQN data (single live snake grid — no translucent top-N ghost overlay). Visual parity means *grammar* parity plus DQN data, confirmed by eye.
- **No automated UI/screenshot tests**, no new metrics/logging infrastructure, no restyle of the menu, versus, or cross-match views.
- No persistence/format changes (`dqn_champion.json`, `best_snake.json`, `sim_metadata.json` untouched).

## Approach (summary)

- New module, e.g. `src/dqn_dash.rs` (name decided in design): a `DqnDashboard` mirroring `VizAdvanced`'s `draw_*` decomposition and color constants, borrowing the live `&GameDQN` plus view bookkeeping. Because it only reads, it can borrow the view's game each frame.
- Per-episode history: `DqnTrainView::end_episode` already centralizes episode-end; push `(time/score)` into the dashboard/ring there (capped ring, reset on `fresh_agent` with the rest of the session bookkeeping).
- `DqnTrainView` gains a small render-target decision (`Dashboard` default | `Compact`), a `Tab` toggle, and routes `draw()` through it; existing pure logic (`on_episode_end`, bookkeeping, tick) untouched.
- `GameDQN::get_state` becomes `pub` (or a thin public wrapper) — the single new data seam; `dqn.rs` consts become `pub`. All other dashboard inputs (`body`, `food`, `score`, `steps`, `agent.q_network`, `agent.replay_buffer.len()`, `epsilon`) are already public.
- `GaTrainView::new()` flips `advanced: false` → `true`; doc comments/tests updated; `render_target` pure function and versus-routing logic unchanged.
- `src/app.rs`: `handle_dqn_train_input` forwards `Tab` to the DQN view; hint overlay handling adjusted per render target.
- strict_tdd (`cargo test`): RED-first pure seams — DQN render-target decision, `Tab` toggle state, history ring push/cap/reset, q-value→intensity mapping if extracted pure; drawing stays untested by design (as throughout this repo).

## Affected Areas

- **New** `src/dqn_dash.rs` (or equivalent mirrored-dashboard module) + module wiring in `src/lib.rs`.
- `src/game_dqn.rs` — visibility-only: expose the current observation (`get_state` pub / accessor). No logic change.
- `src/dqn.rs` — visibility-only: `pub const` for hyperparameters shown in the model-info panel.
- `src/view_dqn_train.rs` — render-target enum + `Tab` toggle + history ring + dashboard draw path; existing tick/episode/champion code unchanged; tests extended for the new pure seams.
- `src/view_ga_train.rs` — default flip (`advanced: true`), docs/tests.
- `src/app.rs` — `Tab` forwarding for DQN train; hint-overlay gating.
- `openspec/changes/training-dashboard-ui/{design,tasks,specs,…}` produced by the following phases.
- Reused as-is (guard): `viz_advanced.rs`, `sim.rs`, `game.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `viz_vs.rs`, versus/cross views, `main.rs`, `Cargo.toml`.

## Delivery / Size Note (review budget 400)

- Estimated changed lines: **~380–550** (mirrored dashboard module ≈ 230–300 including drawing + doc comments; view/app seams ≈ 60–100; GA flip ≈ 20; tests ≈ 80–120). The midpoint sits at/over the 400-line review budget, so an **ask-on-risk** pause is likely at the delivery gate.
- If over budget, the executor MUST pause and ask the user to choose — it must NOT invent a chain strategy or claim an exception. Candidate options to present (pre-decision only): (a) explicit `size:exception` acceptance; (b) two sequential review slices (seams+module first, wiring+flip second); (c) trim the first slice (e.g., one history chart, or the model-info panel into a compact text block) and keep the rest for a follow-up change.

## Risks & Mitigations

- **Visual-only verification (highest).** Macroquad drawing has no automated tests in this repo; a mirrored layout can be logically correct yet visually broken (overlap, off-screen panels at different resolutions, color drift). Mitigations: reuse the reference's exact geometry math and color constants verbatim in the mirror; in-app side-by-side check — `Tab` into GA train's untouched `VizAdvanced` dashboard is the live ground truth next to the new DQN dashboard; final acceptance is a user smoke test of both views against `ui/ui-version4.png` (gitignored, in the repo UI folder). Success criteria below require that smoke.
- **Regression to the verified reference.** Constraint keeps `viz_advanced.rs`/`sim.rs`/GA behavior at zero diff; if design instead chooses to extract shared helpers into `viz_advanced`, that touch re-opens regression risk on the user-verified renderer and needs an explicit reason — default is local duplication.
- **Data-semantics mismatch in the mirror.** The reference's bars/charts normalize against GA constants (e.g., hardcoded `/20` bar caps, `NUM_GAMES_PER_STREAM`); the DQN mirror must pick documented equivalents (session best / max food bound) — wrong normalization would render misleading charts. Mitigation: document the chosen mapping in design; keep reference math where semantics carry over (sigmoid Q-values are already in 0..1, so the network-node color mapping transfers unchanged).
- **Layout drift from the screenshot.** The DQN grid shows one live snake (no top-N translucent ghosts) and GA-specific stat rows have no DQN analog; divergence is intended per the user decision but must be confirmed against the reference by eye so it reads as "same dashboard, DQN data".
- **Test regressions.** Existing unit tests in `view_dqn_train.rs`/`view_ga_train.rs`/`app.rs` must stay green; `GaTrainView` default flip is construction-only and touches no pure function, and `DqnTrainView` logic tests must not depend on render path.
- **Pacing/overlay interference.** The shell overlays a hint line at the bottom-left of DQN train today; the dashboard's bottom model-info panel occupies that zone — hint must be gated or drawn inside the panel (like the reference's controls line).

## Rollback Plan

- Revert this change's commits on `feature/dqn-vs-genetico` (local branch). Pure additive-render change: no data migration; champion/persistence file formats untouched. Reverting restores compact-default DQN train and compact-default GA train exactly as today. A stray `dqn_dash.rs`/render-state diff is removed with the revert.

## Success Criteria (verifiable)

1. `cargo build` clean and `cargo test` green (strict_tdd; RED-first evidence for the new pure seams: DQN render-target decision, Tab toggle, history ring cap/reset).
2. Menu → DQN train opens on the **dashboard** by default: left large grid + bottom model-info panel (real DQN hyperparameters, replay fill, architecture), center NEURAL NETWORK panel with I0..I11 / H0..H7 labels and live Q-value-colored LEFT/RIGHT/BOTTOM/TOP outputs that change as the episode runs, right column Episode/Score/Best/Epsilon stats, score bars, per-episode charts.
3. `Tab` in DQN train toggles dashboard ↔ legacy compact HUD and back; `R` fresh agent and `Esc` still work; episode bookkeeping/champion snapshot/persistence unchanged (existing tests green).
4. Menu → GA train opens on the pre-existing `VizAdvanced` dashboard by default; `Tab` toggles to the compact DQN-style HUD; generations advance and the every-10-generation persistence cadence is unchanged; sim-internal VS still routes to the versus renderer.
5. User smoke test (visual): DQN dashboard and GA dashboard read as the same dashboard grammar as `ui/ui-version4.png` with their respective data; no overlapping or off-screen panels at the default window size.
6. Guard: **zero diffs** to `viz_advanced.rs`, `sim.rs`, `game.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `viz_vs.rs`, versus/cross views (behavior-preservation of the verified reference).

## Assumptions & Open Questions for the Design Phase (auto mode — no live question round; validate at design/verify)

1. Dashboard default, `Tab`→compact in both views, per the user decision (authoritative; not re-litigated).
2. DQN dashboard shows a single live snake in the big grid (no ghost overlay — DQN has no top-N population view); model-info panel shows DQN hyperparameters + controls line, mirroring the reference.
3. Bar/chart normalization uses documented DQN-appropriate denominators (design to specify and call out in verify), not blindly the GA `/20` caps.
4. History ring cap mirrors `VizAdvanced`'s 50 entries; reset with the rest of session state on `fresh_agent`.
5. Which exact hyperparameter rows the model-info panel shows, and whether the shell hint overlay moves into the dashboard's controls line, are design-phase details.
6. Size handling at the delivery gate is the user's call (options listed under Delivery / Size Note); the executor will ask rather than infer.

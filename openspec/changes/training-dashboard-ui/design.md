# Training Dashboard UI — Design

Change: `training-dashboard-ui` · Artifact: design · Follows `proposal.md`

## Context (verified against code)

- The user-verified reference is `src/viz_advanced.rs` (the GA "advanced" dashboard): left column = big game grid (`x=20,y=20`, `grid_size = screen_h − 340`, `tile = grid_size / GRID_W`) over a bottom model-info panel (`y = screen_h − 300`, `w = screen_h − 320`, `h = 280`); center = full-height "NEURAL NETWORK" panel; right column = stacked stat panels, two bars, and two history bar charts. The whole layout is a pure function of `screen_h`/`screen_w`, and the app runs **fullscreen** (`main.rs`, AD-8), so at any desktop resolution the same formulas fit — the mirror inherits this for free.
- `DqnTrainView` (`src/view_dqn_train.rs`) owns a `GameDQN` and, today, renders the *compact* HUD unconditionally. It has no `Tab` key, no per-episode history, and no dashboard render path.
- `GaTrainView` (`src/view_ga_train.rs`) already owns a pure `render_target` seam and routes `Advanced` through `Simulation::draw_advanced` (untouched `VizAdvanced`). Its `new()` sets `advanced: false`.
- `GameDQN` already exposes `body`, `food`, `score`, `steps`, `is_complete`, `agent` (and `agent.q_network`, `agent.replay_buffer.len()`, `agent.get_epsilon()`). Only the 12-float observation accessor (`get_state`, private) is missing. All 7 DQN hyperparameters in `src/dqn.rs` are private `const`s.
- The app is fullscreen; both compact HUDs remain reachable today; the shell overlays a hint at the bottom-left of DQN train (`app.rs::draw_dqn_train`).

---

## 1. Architecture decisions

### D-1 Module/file placement: new `src/dqn_dash.rs`, mirror drawn as free functions + an `EpisodeHistory` ring

**Decision.** Create a new module `src/dqn_dash.rs` containing (a) the entire mirrored dashboard renderer as module-private free functions with one public entry point, and (b) a pure, macroquad-free `EpisodeHistory` ring type. Register `pub mod dqn_dash;` in `src/lib.rs` (alphabetical, after `dqn`). Do **not** grow `view_dqn_train.rs` with the drawing code.

**Rejected alternative.** Inlining the ~250-line renderer into `view_dqn_train.rs`. Rejected because the view file is the bookkeeping/testing surface (~330 lines today including pure tests); folding two full renderers (mirror + legacy compact) into it would bury the resumable-session logic and its RED-first tests under draw code, and would put the untouched compact draw in the same edit blast radius as the mirror.

**Rationale.**
1. Precedent: the repo keeps every dashboard family in its own module (`viz_advanced.rs`, `viz_vs.rs`); the DQN dashboard belongs to that family.
2. Isolation of the mirror: all geometry/color copies of the reference live in one self-contained file whose whole diff is the new dashboard — a reviewer diffs it against `viz_advanced.rs` line by line.
3. Zero-diff guard on the reference: no helper is exported from, extracted into, or shared with `viz_advanced.rs` (see D-2).
4. Draw entry needs no access to the view type — it borrows `&GameDQN` + two scalars + `&EpisodeHistory`, so `view_dqn_train.rs` stays decoupled from drawing internals.

### D-2 Drawing helpers are duplicated locally in `dqn_dash.rs` — verbatim copies where possible

**Decision.** Copy the reference's color constants and geometry arithmetic into `dqn_dash.rs` as private items with the same names/values (`PANEL_BG`, `PANEL_BORDER`, `TEXT_COLOR`, `ACCENT_COLOR`, `TITLE_SIZE`, `TEXT_SIZE`; geometry formulas verbatim). Do **not** make `viz_advanced.rs` items `pub(crate)`/shared and do **not** extract a shared helper module.

**Rationale.** `viz_advanced.rs`, `sim.rs` and the GA render path are the user-verified reference and are under a zero-diff guard. Touching them (even visibility-only exports) re-opens regression risk on verified output for zero functional gain. Duplication cost is ~15 const/geometry lines plus ~90 lines of node/panel/chart drawing — drawing is untested in this repo by design (no macroquad harness), so duplication risk is confined to the new module and caught by the visual smoke test (success criteria). This is the explicitly preferred, lowest-risk option in the proposal's Non-Goals.

### D-3 DQN render-target decision, `Tab` toggle, and draw routing (mirror of the GA seam)

In `view_dqn_train.rs`:

```rust
/// Which renderer draws the current DQN-train frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DqnRenderTarget { Dashboard, Hud }

/// Pure render-path decision (macroquad-free, RED-first seam).
pub fn dqn_render_target(dashboard_enabled: bool) -> DqnRenderTarget {
    if dashboard_enabled { DqnRenderTarget::Dashboard } else { DqnRenderTarget::Hud }
}
```

- New field `dashboard: bool` on `DqnTrainView`, initialized `true` in both `new()` and `at_path()` → **the dashboard is the default DQN-train render**. There is no DQN internal-VS sub-state (DQN versus is a separate transient view), so the decision is 2-state, unlike GA's 3-state.
- New methods: `pub fn toggle_dashboard(&mut self)` (flips the flag — shell `Tab`), `pub fn dashboard_enabled(&self) -> bool` (shell hint gating).
- `pub fn draw(&self)` becomes the dispatcher:

```rust
pub fn draw(&self) {
    match dqn_render_target(self.dashboard) {
        DqnRenderTarget::Dashboard => dqn_dash::draw(&self.game, self.episode, self.best_score, &self.history),
        DqnRenderTarget::Hud => self.draw_hud(),
    }
}
```

- The existing compact draw body (clear, HUD text, grid, `grid_layout`) moves **unchanged** into a private `fn draw_hud(&self)`. Nothing is deleted; the legacy HUD stays reachable via `Tab`. `dqn_dash::draw` starts with `clear_background(BLACK)` (mirroring `sim.rs::draw_advanced` doing the clear before `VizAdvanced::draw`).

### D-4 Entry point and data plumbing of the mirror

```rust
// dqn_dash.rs
pub fn draw(game: &GameDQN, episode: usize, best_score: usize, history: &EpisodeHistory)
```

Everything the dashboard needs is already public on `GameDQN`/`DQNAgent` plus the one new accessor (D-5): grid from `game.body`/`game.food`; run stats from `game.score`/`game.steps`; epsilon from `game.agent.get_epsilon()`; replay fill from `game.agent.replay_buffer.len()`; network outputs computed per frame as `game.agent.q_network.predict(&game.observation())` and `.last()` unwrapped (identical to the reference's `outputs.last().unwrap()` usage). The view passes its own bookkeeping (`episode`, `best_score`, `&history`). No `&DqnTrainView` crosses into the module.

### D-5 Observation seam: thin public accessor on `GameDQN`, named `observation`

**Decision.** In `src/game_dqn.rs`, keep the existing private `fn get_state(&self) -> Vec<f64>` untouched and add:

```rust
/// Current 12-input observation (4 directions × [wall, food, body] distances),
/// as consumed by the q-network (same feature vector `step()` trains on).
pub fn observation(&self) -> Vec<f64> { self.get_state() }
```

**Rationale.** The proposal names this the single data seam; the executor names the accessor. A visibility-only keyword change on `get_state` is equally small but (a) leaks a poor name into the crate API and (b) makes the private training-path method a public surface. The wrapper is strictly additive: internal callers in `step()` are untouched (zero behavior risk to learning), the name documents the "current observation" semantic used for live Q-value coloring, and the dashboard never sees a half-trained/other snapshot. +3 lines. `observation()` always returns 12 floats, so `predict` never hits its input-size panic.

### D-6 Hyperparameter constants in `src/dqn.rs` become `pub` (visibility-only)

**Decision.** Add `pub` to the seven `const`s: `REPLAY_BUFFER_SIZE`, `BATCH_SIZE`, `GAMMA`, `LEARNING_RATE`, `EPSILON_START`, `EPSILON_END`, `EPSILON_DECAY`. Architecture sizes already come from `configs::INP_LAYER_SIZE/HIDDEN_LAYER_SIZE/OUTPUT_LAYER_SIZE` (12×8×4, pub) and the episode step limit from `NUM_SIM_STEPS * 2` (GameDQN enforces it in `step()`). No logic change anywhere in `dqn.rs`.

### D-7 Per-episode history ring on `DqnTrainView`, fed at `end_episode`

`EpisodeHistory` lives in `dqn_dash.rs` (pure, macroquad-free — mirrors `VizAdvanced`'s twin-vector ring; the GA dashboard also duplicates the ring's cap of 50):

```rust
pub const EPISODE_HISTORY_CAP: usize = 50;

pub struct EpisodeHistory { pub times: Vec<f32>, pub scores: Vec<usize>, cap: usize }

impl EpisodeHistory {
    pub fn new() -> Self;                       // cap = 50, both vecs empty
    pub fn push(&mut self, time: f32, score: usize); // append; while len > cap remove(0) (oldest pair)
    pub fn clear(&mut self);                    // empty both vecs
    pub fn len(&self) -> usize;                 // scores.len() == times.len() invariant
}
```

On `DqnTrainView`:
- New fields: `history: EpisodeHistory`, `dashboard: bool`, `episode_started_at: std::time::Instant` (initialized in `new()`/`at_path()`).
- `end_episode` (already the single episode-end hub) gains one push before bookkeeping: `let elapsed = self.episode_started_at.elapsed().as_secs_f32(); self.history.push(elapsed, self.game.score);` and restarts the timer after `self.game.reset()`. `episode += 1` / `on_episode_end` / champion snapshot / persistence are byte-identical.
- `fresh_agent` additionally calls `self.history.clear()` and restarts the timer (session-reset semantics; champion retained as today).
- Documented skew (accepted, cosmetic): the timer is wall-clock `Instant`, so pausing mid-episode at the menu inflates that episode's recorded duration. Fine for a visual chart; the alternative (tie duration to `game.steps`) changes chart semantics away from the reference's wall-time "GEN TIMES".

### D-8 GA default flip and Tab model (both views symmetrical)

- `view_ga_train.rs::new()`: `advanced: false` → `true`. Update the module doc, the struct field doc ("Default on = `VizAdvanced` dashboard"), and the `new()` doc. `render_target`, versus-routing, pacing, and evolution are untouched — entering GA train now draws `sim.draw_advanced()` (byte-identical to the original `main` app's viz mode), `Tab` drops to the compact HUD, `Tab` again returns. Paused GA sessions keep their in-process flag (unchanged semantics).
- `app.rs::handle_dqn_train_input` gains the `Tab` forward (mirroring `handle_ga_train_input`):

```rust
if is_key_pressed(KeyCode::Tab) {
    if let Some(view) = &mut self.dqn { view.toggle_dashboard(); }
}
```

- Hint overlay gating in `app.rs::draw_dqn_train`: the shell overlay `"[R] fresh agent   [ESC] menu"` at `(10, screen_height() − 12)` is drawn **only when the compact HUD is active** (`if !view.dashboard_enabled()`). In dashboard mode the bottom model-info panel occupies that zone down to `screen_h − 20`; the panel instead carries its own controls line in accent (mirroring the reference's controls row) — see the panel spec below. `R` and `Esc` keep working in both render targets (shell-owned, unchanged).

---

## 2. Exact geometry mapping (viz_advanced → dqn_dash)

Same board (`GRID_W = GRID_H = 25`) and fullscreen window ⇒ the reference's screen-relative formulas transfer verbatim. All values below are the reference's, copied unchanged; only panel *titles/rows* differ.

| Reference element | Reference geometry | DQN mirror |
|---|---|---|
| Grid (left col) | `x=20, y=20, grid_size = screen_h−340`, `tile = grid_size/GRID_W`, border rect +8/+16, grid lines, white food (`tile−4` inset), snake head `(0.3,0.9,0.3)` body `(0.2,0.7,0.2)` | identical; only one live snake (no ghost/rank loop — DQN has no top-N population) |
| NEURAL NETWORK panel | `left_col_width = screen_h−320`; `panel_x = left_col_width+40`, `panel_w = 550`, `panel_h = screen_h−40`, `y = 20`; title at `+20/+28` accent | identical geometry & title "NEURAL NETWORK" |
| NN layers | input `x+80`, spacing `(panel_h−160)/11`, r=7 accent dots + `I{i}` at x−35; hidden `+160`, spacing `(panel_h−300)/7`, r=8 orange `(0.9,0.6,0.0)` + `H{i}`; output `+320`, spacing `(panel_h−500)/3`, r=10 | identical: 12→8→4 nodes, I0..I11 / H0..H7 labels |
| NN output coloring | intensity = `value.clamp(0,1)`; color `(intensity, intensity*0.3, intensity*0.9)`; labels `["LEFT","RIGHT","BOTTOM","TOP"]` at x+22 | identical, fed by `q_network.predict(observation()).last()`. Q-values are sigmoid outputs (same 0..1 domain as GA net outputs — `nn::Layer::predict` sigmoids every layer) so the mapping carries over unchanged. Label order matches `GameDQN::step` action mapping 0..3 → Left/Right/Bottom/Top |
| Model-info panel | `x=20, y=screen_h−300, w=screen_h−320, h=280` | identical geometry; see contents below |
| Right column | `x = left_col_width+550+60`, `w = screen_w − x − 20`, `y` stacking | identical |
| Stat panel 1 | `h=150`, title, rows at y+45 then +30 | "EPISODE STATS" — rows: `Episode: N`, `Best: M`, `Epsilon: {:.3}` |
| Stat panel 2 | `h=130`, rows y+45/+30/+30 | "RUN STATS" — rows: `Score: S`, `Steps: S` |
| Bar 1 | `h=100`, magenta bar `h=30` at panel_y+50, `w−40` track, `{:.0}%` label centered | "SCORE" — live `game.score` |
| Bar 2 | `h=100`, red bar, same shape | "BEST" — session best |
| Chart 1 | `chart_h = (screen_h−y−20)/2 −10`, SKYBLUE bars | "EPISODE TIMES" — `history.times` |
| Chart 2 | GREEN bars | "EPISODE SCORES" — `history.scores` (f32-cast) |

**Chart internals** (copied): `chart_x/y = +20/+50`, `chart_w = w−40`, `chart_h = h−70`, `max_val = data.max().max(1.0)`, `step = chart_w / len`, bar `w = step.max(4)−1`. Dynamic self-normalization needs no denominator.

**Model-info panel contents (DQN)** — title `"DQN TRAIN"` (accent, like the reference's "SNAKE AI"); rows at `panel_y + 55`, spacing 26, `TEXT_SIZE`:
1. `Architecture: 12x8x4` (`INP/HIDDEN/OUTPUT_LAYER_SIZE`)
2. `Replay: {fill}/{REPLAY_BUFFER_SIZE}` (live `replay_buffer.len()`)
3. `Batch: 32 | Gamma: 0.99` (`BATCH_SIZE`, `GAMMA`)
4. `LR: 0.001` (`LEARNING_RATE`)
5. `Epsilon: 1.00 -> 0.01 (x0.995)` (`EPSILON_START/END/DECAY`)
6. `Step Limit: 200` (`NUM_SIM_STEPS * 2`, the cap `GameDQN::step` enforces)
7. Controls line (accent, panel bottom, like the reference's): `Controls: [TAB] HUD  [R] Fresh Agent  [ESC] Menu`

All values come from the `pub` seams (D-5/D-6), never literals.

---

## 3. Documented bar denominators (DQN) — design's call-out for verify

Reference bars hardcode `/20.0` (GA-specific). The DQN mirror documents its own:

- **`SCORE` and `BEST` bars share `FULL_BAR = (NUM_SIM_STEPS * 2) as f32 = 200.0`.** Proof of bound: `GameDQN::step` increments `steps` every move and forces `done` at `steps >= NUM_SIM_STEPS*2`; `score` increments only when food is eaten, which also consumes a step ⇒ `score <= steps <= 200`. A full bar = a theoretically max-length episode. This is the tightest global constant cap available (board-area max food ≈ 528 is looser and less "grammar-true" than the reference's step-like cap).
- **Charts** normalize to the data's own max with floor 1.0 (reference semantics, copied) — no fixed denominator needed.
- GA's `/20` was tied to its per-generation metric; the DQN `200` bound is documented in the design and will be re-checked in verify (proposal Assumption 3).

Data-semantics substitutions vs the screenshot (intended divergence per proposal): single live snake grid (no translucent top-N ghosts), no GA stream/agent counts; "Gen/Sim/Fitness" rows have no DQN analog and are replaced by the rows above; panel titles carry DQN meaning while colors/geometry stay.

---

## 4. File-by-file change summary

| File | Change | ~Lines (net) |
|---|---|---|
| `src/dqn_dash.rs` | **new**: consts, `EpisodeHistory`, `output_intensity`, `draw` + private `draw_grid/draw_neural_network/draw_model_info/draw_stats_panels/draw_chart/draw_panel`, module doc, `#[cfg(test)]` pure-ring tests | ~300 |
| `src/game_dqn.rs` | `pub fn observation(&self)` wrapper (+3) | ~3 |
| `src/dqn.rs` | 7 consts → `pub` (visibility-only) | ~7 |
| `src/view_dqn_train.rs` | `DqnRenderTarget` + `dqn_render_target`, `dashboard`/`history`/`episode_started_at` fields, `toggle_dashboard`/`dashboard_enabled`, `draw` dispatcher + `draw_hud` move, `end_episode` push, `fresh_agent` reset, doc header, tests | ~75 net |
| `src/view_ga_train.rs` | `advanced: true` default + module/field/`new` docs | ~12 net |
| `src/app.rs` | `Tab` forward in `handle_dqn_train_input`; hint gating in `draw_dqn_train`; header/doc comments | ~16 |
| `src/lib.rs` | `pub mod dqn_dash;` | ~1 |
| **Total** | | **~415–490** |

**Zero-diff guard (MUST NOT change):** `viz_advanced.rs`, `sim.rs` (incl. `draw_advanced`, VS cadence, persistence), `game.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `viz_vs.rs`, `game.rs`, `view_dqn_versus.rs`, `view_ga_versus.rs`, `view_cross_match.rs`, `main.rs`, `Cargo.toml`. GA evolution logic and file formats (`dqn_champion.json`, `best_snake.json`, `sim_metadata.json`) untouched.

---

## 5. Test seams (pure logic only — drawing stays untested by design)

RED-first, macroquad-free, all existing tests stay green (they assert no render path):
1. `dqn_render_target(true) == Dashboard`, `(false) == Hud`.
2. Toggle state: construct a test view (existing `at_path` harness), `toggle_dashboard()`, assert `dashboard_enabled()` flips and maps through `dqn_render_target`.
3. `EpisodeHistory`: push past the 50 cap → `len == 50` and the *oldest* pair is evicted; `times`/`scores` stay index-aligned; `clear()` empties both.
4. `fresh_agent` resets history and bookkeeping while retaining the champion (extend the existing `fresh_agent_resets...` test with a history pre-fill assertion).
5. `output_intensity` clamps to [0,1] (extracted pure fn from the Q-value coloring so the sigmoid→color domain is pinned).

No new tests for `app.rs` (key handling is `is_key_pressed`, not headless-testable; GA flip is construction-only data). No macroquad draw calls in any test.

---

## 6. Rollout, risk, acceptance

**Rollout:** additive render change on `feature/dqn-vs-genetico`; revert of the change's commits restores compact-default DQN and compact-default GA exactly (no data migration; file formats untouched). **Risks:** visual-only verification (mitigated by verbatim geometry/color copies + in-app `Tab` ground truth against GA's untouched dashboard + user smoke vs `ui/ui-version4.png`); model-panel/hint overlap (mitigated by D-8 gating + panel controls line); chart-denominator semantics (mitigated by §3 documentation). **Acceptance:** `cargo build`/`cargo test` green; DQN train opens on the dashboard with live Q-colored outputs; `Tab`/`R`/`Esc` work in both targets; GA train opens on `VizAdvanced`; zero diffs on the guard list; user visual smoke (success criteria 2–5 of the proposal).

## 7. Delivery/size note

Forecast **~415–490 changed lines**, midpoint at/over the 400-line review budget. At the delivery gate the executor **must pause (ask-on-risk)** and present the proposal's three pre-decision options — (a) explicit `size:exception` single PR; (b) two chained review slices: slice 1 = seams + `EpisodeHistory` + render-target/toggle + app `Tab`/hint wiring + GA flip (headless-testable, ~150), slice 2 = `dqn_dash.rs` drawing (~250); (c) trim slice 1 content (e.g., defer the model-info panel to a follow-up). The design recommends (b) if the user will not grant (a), since slice 1 is independently testable and slice 2 is isolated drawing.

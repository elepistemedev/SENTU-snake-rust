# Exploration: unified-snake-shell

SDD explore phase (executed inline by the orchestrator because the subagent runtime was failing; read-only).

## Current State

The crate `snake` (macroquad 0.4, edition 2021) is "two projects in one": one library (`src/lib.rs`), two binaries.

### Binaries and loops

- `snake` (`src/main.rs`, window `snake-ai`, fullscreen): GA driver. Inner loop calls `Simulation::update(is_viz_enabled, slow_mode)`; slow mode runs one sim update per frame, fast mode batches up to 50. Keys: `Escape` quit, `Tab` viz on/off, `Space` slow/fast, `V` toggle VS mode (entering VS forces slow). `SIM_SLEEP_MILLIS` sleep in slow mode.
- `snake-dqn` (`src/main_dqn.rs`, window 800x600): DQN driver. One `GameDQN::step()` per frame; on `done`, prints episode line and calls `reset()`. Renders inline: single 25x25 grid (tile 20, offset 250,50) + text HUD (`Episode`, `Score`, `Best`, `Epsilon`). `Escape` quits.

### The "DQN-style" layout (the baseline the user wants GA to match)

main_dqn.rs drawing: one snake grid, dark background, HUD text column. Clean and minimal. The GA side has *richer* viz code today but a very different "wall" aesthetic:

- `VizAdvanced::draw(...)` (sim.rs training renderer): mini game grids + neural-network viewer + stats panels + charts.
- `viz.rs` (546 lines, older): best games, net drawing, stats — module compiled but effectively superseded by VizAdvanced; `Viz` itself is not used by `sim.rs` anymore (VizAdvanced is). Dead-ish weight for the shell.
- `viz_vs.rs`: side-by-side two-grid VS view with a center panel (used by GA VS mode).

So "GA train with the same UI distribution as DQN" means: one clean arena grid + text HUD (generation, gen max, best ever, elapsed, current champ score) instead of the multi-panel GA dashboard.

### Game/agent mechanics parity (key finding)

- Both agents act on the **same observation space**: 4-directional ray vision, 12 inputs (wall, food, body per direction) — `Game::get_four_dir_vision` (GA) vs `GameDQN::get_state` (DQN). Encodings are similar but **not identical**: GA pushes `wall = 1/dist`, `food = 0|1`, `body = 1/dist`; DQN pushes `wall = 1/dist`, `food = 0|1`, `body = 1/body_dist or 0`. Subtle mismatch in body-distance normalization.
- Same action space: 4 dirs `{Left, Right, Bottom, Top}` mapped identically, both with a 180°-turn guard.
- Same grid constants (`GRID_W/H = 25`, walls at borders).
- Both brains are literally the same type `crate::nn::Net` (12→8→4, sigmoid, serde): GA population nets and the DQN `q_network`/`target_network` are all `Net`. `Net` is serializable → champions can be saved/loaded (GA already does: `best_snake.json`, `sim_metadata.json`, gitignored; DQN does **not** persist anything today).
- Step limits differ: GA `Game` tiers by score (100/200/300/500/800); DQN `GameDQN` fixed at `NUM_SIM_STEPS*2` (200). Food placement/random-empty retries differ slightly (5 vs 10 tries). Reward shaping exists only in DQN.

### Versus modes today

- **GA internal VS exists**: `Simulation::toggle_vs_mode` pits `best_net_ever` vs `second_best_net_ever` (or current top) as two `Game::with_brain` instances on the shared grid rules; `viz_vs` renders side by side; auto-enters every 100 generations; returns to training when both die. Manual `V`.
- **DQN has no versus mode** and no champion persistence.
- A cross **DQN-vs-GA** match is structurally easy: both brains are `Net`; the GA arena (`Game::with_brain(net)`, GA rules + GA vision) can run any `Net`, including a DQN `q_network`/champion at epsilon 0. Caveat: the DQN policy learned under DQN vision/mechanics; running it inside the GA `Game` may degrade it slightly. Neutral referee (one shared game core) is the clean fix and is design territory.

### Tests

None (`cargo test` passes trivially; strict_tdd declared in openspec/config.yaml).

## Affected Areas

- `Cargo.toml` — bin layout (unify vs add a third binary).
- `src/main.rs` + `src/main_dqn.rs` — both loops collapse into one app-state shell (menu + mode views), or the DQN entry absorbs the shell and GA entry stays for A/B.
- New `src/ui/*` or `src/app/` module(s) — menu + shared layout primitives (HUD, grid, buttons/key hints).
- `src/sim.rs` — needs a public step/status API usable from the shell (currently `update()` owns generation lifecycle + drawing), plus exposing current GA champion as `Net`.
- `src/dqn.rs` / `src/game_dqn.rs` — add champion snapshot (clone current `q_network` when a new best score occurs) and possibly save/load; epsilon-0 policy for versus.
- `src/game.rs` + `src/viz_vs.rs` — GA VS arena is directly reusable for internal and cross versus.
- `src/viz_advanced.rs` / `src/viz.rs` — decide reuse vs retirement for the new GA train view.
- `src/pop.rs`, `src/stream.rs` — only touched if the shell exposes per-frame population metrics.

## Approaches

1. **Single unified binary, app-state enum shell (recommended)** — one `snake` bin: `enum AppMode { Menu, DqnTrain, DqnVersus, GaTrain, GaVersus, DqnVsGa }` driving one macroquad loop; each mode owns its state (Simulation / GameDQN / arenas); menu draws option cards; Esc returns to menu.
   - Pros: matches "two in one → one app" intent; one window/loop contract; GA VS arena reused for both internal and cross versus; later fixes land in one place.
   - Cons: merges two loops that pace differently (GA gen batch vs DQN per-episode frame) — needs a small per-mode tick abstraction; riskier than additive approach for regressions.
   - Effort: Medium.

2. **Add a third binary that hosts the shell; keep both originals untouched** — `snake-ui` (or similar) imports the lib and composes modes; original `snake` and `snake-dqn` remain for A/B/debug.
   - Pros: zero regression risk to existing entry points; cleanest MVP diff; per-algorithm issues can be fixed in lib code shared by both.
   - Cons: three bins with overlapping loops; "dos en uno" goal partially unmet (two old entries still there); doc/UX duplication.
   - Effort: Low–Medium.

3. **Full shared game-core refactor first, then shell** — extract one game core (rules, vision, step limit config) used by GA, DQN, and all versus modes, then build the shell on it.
   - Pros: kills the game.rs/game_dqn.rs duplication; encoding mismatch disappears; cleanest long term (aligns with "fix issues afterwards").
   - Cons: biggest diff; violates "mix first as MVP, fix issues later separately" ordering; review workload balloons past 400 lines.
   - Effort: High.

## Recommendation

Approach **1** (single unified binary, app-state enum) or **2** if the user wants zero risk to the existing entries — both are viable MVPs; the deciding question is whether old binaries may be replaced. Reuse the existing GA VS arena (`Game::with_brain` + `viz_vs`) for GA internal VS **and** for DQN-vs-GA cross match, since every brain is a `Net`. For "GA train with DQN layout", write a small single-grid+HUD view (mirror of main_dqn.rs drawing) fed by a light `Simulation` public step API; leave VizAdvanced untouched for now. Add a minimal DQN champion snapshot (clone `q_network` on record score) so DQN internal/cross versus has a player. Do **not** do approach 3 inside the MVP.

## Risks

- DQN policy vs GA-rule arena: vision-encoding and step-limit differences (DQN trained on DQN mechanics) may make cross-match scores misleading; acceptable for MVP if labeled, real fix later (shared core).
- Pacing mismatch between modes inside one loop (frame-locked DQN episode cadence vs GA batch updates) — needs a per-mode tick policy; GA in "slow mode" semantics per versus.
- GA training currently renders through VizAdvanced inside `Simulation::update`; decoupling draw from sim logic is a small refactor that must stay behavior-preserving.
- Randomness/seed parity between modes is untested; versus fairness (who moves first, food luck) is a design decision to document.
- No DQN persistence: cross-match after a fresh launch needs a GA champion file and either a saved DQN champion or a "train first" flow.
- strict_tdd: true and zero tests — the shell adds a UI-heavy change; MVP tests should target pure logic (arena outcomes, champion snapshot, state transitions) not pixels.

## Product decisions (user-confirmed, 2026-09-09)

1. **Binaries:** single unified binary — one `snake` app hosting menu + all views; old `snake`/`snake-dqn` entries replaced at archive (Approach 1).
2. **DQN internal versus:** session-best champion (q_network snapshot cloned on record score, ε=0) vs current live policy (ε=0).
3. **Cross-match players:** persist DQN champion to a gitignored JSON file (`dqn_champion.json`, Net is serde) + load GA champion (`best_snake.json`); cross-match always available.
4. **GA internal versus UI:** reuse existing side-by-side `viz_vs` panel; no DQN-style restyle for versus views in the MVP.

Ready for Proposal: Yes (scope: MVP integration; per-algorithm fixes deferred to later changes).

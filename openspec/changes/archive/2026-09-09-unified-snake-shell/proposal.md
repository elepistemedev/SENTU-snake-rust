# Unified Snake Shell — Proposal

## Problem Statement

The project is "two projects in one": the `snake` crate ships two binaries (`snake` = genetic algorithm, `snake-dqn` = DQN) that duplicate entry-point, loop, and drawing concerns. The DQN binary is the cleaner baseline (single grid + text HUD), but neither binary lets the user choose *what* to run, DQN has no versus mode at all, GA has no cross-match against DQN, and GA training renders through a heavy multi-panel dashboard that is visually unrelated to the DQN view. Running experiments therefore means switching binaries/windows and eyeballing unrelated UIs.

This change ships the MVP of a **unified single-binary app** with a welcome menu and five views (DQN train, DQN internal versus, GA train with a DQN-style layout, GA internal versus, DQN-vs-GA cross-match). It integrates what already exists; it does **not** fix each algorithm's pre-existing issues — that is explicitly deferred to later, separate changes.

## Goals (in scope — MVP)

1. **One binary, one app**: replace the two current entry points with a single `snake` binary hosting an app-state shell: `Menu | DqnTrain | DqnVersus | GaTrain | GaVersus | DqnVsGa`.
2. **Welcome menu**: at launch the user picks which mode to enter (keyboard navigation), and can return to the menu mid-session (Esc) without quitting.
3. **DQN trains view**: the existing DQN training loop + HUD, ported into the shell unchanged in behavior.
4. **DQN internal versus view**: session-best champion (q_network snapshot cloned whenever a new record score occurs, ε=0) vs the live current policy (ε=0), run on a versus arena; reuse the side-by-side `viz_vs` rendering.
5. **GA trains view**: GA training driven through the shell and rendered with the same clean single-grid + text-HUD layout family the DQN view uses (generation, generation max, best ever, elapsed seconds, current champion score). Existing GA behavior (populations, streams, persistence of `sim_metadata.json`/`best_snake.json`) stays intact; only the in-app rendering of the training mode changes to the DQN-style view, leaving `VizAdvanced` untouched elsewhere.
6. **GA internal versus view**: existing GA VS arena (best-ever vs second-best-ever on `Game::with_brain`, `viz_vs` side-by-side) exposed as a menu mode.
7. **DQN vs GA cross-match view**: both brains are the same serializable `Net` type (12→8→4). Load GA champion from `best_snake.json` (existing) and DQN champion from a new `dqn_champion.json` (gitignored), run both through the shared versus arena with ε=0, and show the side-by-side match.

## Non-Goals (explicit, deferred)

- Fixing GA issues (e.g., broken `fitness` scaling, `Viz` dead code, step-limit tiers) or DQN issues (e.g., simplified backprop, no optimizer, replay-buffer design) — these get their own later changes per algorithm.
- Shared game-core refactor: de-duplicating `game.rs` / `game_dqn.rs` and normalizing vision encodings stays out of this change (a design follow-up; cross-match scoring may slightly favor GA-rule familiarity and is acceptable for the MVP).
- Multiplayer, config screens, mouse UI, or pixel-perfect styling.

## Approach (summary)

- One `snake` binary with an `AppMode` enum driving a single macroquad loop; per-mode state structs (`Simulation`, `GameDQN` plus a small shell wrapper, versus arenas).
- Add a lightweight public step/status seam on `Simulation` (sim update without owning drawing; expose current GA champion `Net`), keeping `Simulation::update` semantics behavior-preserving.
- DQN champion: clone `q_network` on new record score; persist as `dqn_champion.json` (Net is serde); load on start when present. Add `dqn_champion.json` to `.gitignore`.
- Versus: reuse `Game::with_brain(net)` + `viz_vs` arena for GA internal VS, DQN internal VS, and the cross-match (players differ only in which `Net`s are loaded).
- GA train view: a new small view module mirroring main_dqn.rs drawing (one grid + HUD), fed by `Simulation` per-generation data.

## Affected Areas

- `Cargo.toml` — bin layout (collapse to one binary; `snake-dqn` bin removed).
- `src/main.rs`, `src/main_dqn.rs` — replaced by the shell entry (`main.rs`).
- `src/sim.rs` — expose step/status and champion access without changing evolution behavior.
- `src/game_dqn.rs` / `src/dqn.rs` — champion snapshot hook on record score + (de)serialization of champion.
- `src/lib.rs` — module wiring for new shell/ui/view modules.
- New: shell (`src/app/` or equivalent), menu view, DQN-style GA train view, versus orchestrator, DQN champion persistence helpers.
- `.gitignore` — add `dqn_champion.json`.
- Reused as-is: `game.rs`, `viz_vs.rs`, `viz_advanced.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `utils.rs`, `configs.rs`.

## User Flows

1. `cargo run --release` → menu: `1 DQN train · 2 DQN versus · 3 GA train · 4 GA versus · 5 DQN vs GA`.
2. In any view: Esc returns to the menu (second Esc quits from menu). Existing in-mode hotkeys (e.g., GA slow/fast, versus triggers) keep working where reused.
3. Cross-match and internal-versus views render both snakes side by side with live scores; when both games end, the winner (higher score) is shown and the user returns to the menu (or reruns).

## Risks & Mitigations

- **Loop pacing mismatch** (DQN step-per-frame vs GA batched generations): per-mode tick policy inside the shell; versus modes always slow/step-locked. Low risk, contained in the shell.
- **Cross-match fairness / encoding mismatch**: DQN trained under its own vision/mechanics may underperform on GA-rule arena. Mitigation: label the match as a "vs" experiment, use identical arena for both players, defer encoding normalization to the shared-core follow-up.
- **Regression in GA training from decoupling draw from sim**: keep sim logic and existing persistence behavior unchanged; new view is additive; `Simulation` gains public accessors behind the same code paths.
- **No DQN persistence previously**: new `dqn_champion.json` write/read is small; missing file degrades gracefully (cross-match prompts that DQN must train first, GA champion may still fight via best-ever from disk or a live population).
- **strict_tdd: true, zero tests**: apply with RED-GREEN evidence on pure logic (arena winner resolution, champion snapshot/round-trip, state transitions); UI drawing stays untested by design.

## Rollback Plan

- The change is additive-collapsing: the original `snake` (GA) and `snake-dqn` (DQN) entry behavior remains reachable inside the shell. Rollback = revert the commit(s) of this change on `feature/dqn-vs-genetico`; no data migration is involved. `sim_metadata.json`/`best_snake.json` formats are untouched; `dqn_champion.json` is new and only read by this app.

## Success Criteria (verifiable)

1. `cargo build` clean; single `snake` binary; `cargo run` opens the menu.
2. Each of the five views is reachable from the menu and Esc returns to the menu.
3. DQN train HUD shows Episode/Score/Best/Epsilon and behaves like today's `snake-dqn`.
4. A new DQN record score persists `dqn_champion.json`; on relaunch the cross-match and DQN-versus views can load it.
5. GA train view renders the population with the DQN-style single-grid + HUD layout; generations advance and metadata/best nets persist as today.
6. GA internal versus and DQN-vs-GA run both players side by side and resolve a winner by score when both games end.
7. `.gitignore` covers `dqn_champion.json`; no test regressions (`cargo test`), strict-TDD evidence recorded for new pure logic.

## Decisions & Assumptions (from the proposal question round — user-confirmed)

1. One unified binary; old entries replaced at archive.
2. DQN internal versus = session-best champion vs live policy, both ε=0.
3. Cross-match loads persisted champions (`best_snake.json` + new `dqn_champion.json`).
4. Versus views (GA internal + cross) reuse the existing side-by-side `viz_vs` panel; no restyle in MVP.
5. GA train view adopts the DQN-style layout; `VizAdvanced` untouched outside the shell.

# Unified Snake Shell — Design

Change: `unified-snake-shell`. Designs the MVP from `proposal.md` + `specs/app/spec.md`. No shared game-core refactor in this change.

## Architecture decisions (with rationale)

- **AD-1: One binary + state enum, per-view structs, single macroquad loop.** `src/main.rs` becomes the shell entry. `enum AppMode { Menu, DqnTrain, DqnVersus, GaTrain, GaVersus, DqnVsGa }`; the shell owns one `DqnTrainer` and one `Simulation`-backed GA trainer as long-lived paused state plus lightweight transient match state. Rationale: user decision 1; keeps GA evolution behavior untouched and makes "resume" semantics (AD-3) cheap.
- **AD-2: Versus = two `Game` instances built with `Game::with_brain(Net)`.** All three versus flavors (DQN internal, GA internal, cross) share one arena runner and one drawing path, because every brain — GA nets and DQN `q_network` — is the same `Net` (12→8→4, serde). Rationale: exploration finding; maximum reuse; no new game core needed.
- **AD-3: DQN training state pauses, not dies, when leaving the view.** The menu keeps the trainer alive (agent, episode counter, session best, champion). DQN internal versus then always has a real "live current policy" (AD-4). A paused run is resumable from the menu; the menu shows its state. GA training is naturally resume-able because `Simulation` persists via `sim_metadata.json`/`best_snake.json`; leaving and re-entering GA train simply reloads (today's `Simulation::new()` behavior).
- **AD-4: DQN internal versus = session champion snapshot vs live policy, both greedy.** "Epsilon 0" in the arena is automatic: `Game::get_brain_output` always argmaxes. Champion = latest snapshot (in-memory, then persisted `dqn_champion.json`). Live policy = the paused trainer's current `q_network`. If no live trainer exists yet, the view uses the persisted champion vs a fresh greedy agent and labels it accordingly.
- **AD-5: `VizVS` gains a flavor config with defaults that reproduce today's exact GA output.** Today `viz_vs` hardcodes "BEST EVER"/"2ND BEST", GA record semantics ("RECORD TO BEAT", "NEW!", "BEST EVER WINS!"). A `VsFlavor { player1_title, player2_title, colors, record: Option<usize>, winner_1/2/tie labels, back_label }` parametrizes it; the GA internal flavor uses the current constants and strings verbatim → no regression, and DQN internal / cross flavors label "CHAMPION vs CURRENT" / "GA vs DQN" with `record: None`.
- **AD-6: GA train renders DQN-style by default; `Tab` toggles the existing advanced dashboard.** `Simulation` gains a behavior-preserving tick/draw split so the shell can drive evolution and draw what it wants. Default view = DQN-style single grid + HUD; `Tab` keeps the old `VizAdvanced` full dashboard reachable (satisfies "VizAdvanced MUST NOT be modified / remains functional" and keeps power users happy). Hotkeys keep old meanings.
- **AD-7: No new game-rules code.** DQN policies may underperform slightly on the GA-rule arena (vision/step-limit nuances). Accepted for MVP and labeled in the versus UI ("GA rules arena"); the shared-core normalization is the deferred follow-up.
- **AD-8: Window = fullscreen (as the GA binary was).** `viz_vs` sizes grids from `screen_width()/height()`; DQN-style views center their grid with `screen_width()` instead of the old hardcoded `offset_x=250` so layouts survive any resolution. HUD text column anchored left like today.

## Module layout (new/changed files)

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Replaced: shell entry (`window_conf`, loop, mode dispatch). |
| `src/main_dqn.rs` | Deleted (behavior moves into `view_dqn_train.rs`); `snake-dqn` bin removed from `Cargo.toml`. |
| `src/app.rs` | `AppMode`, `App` (holds paused trainers + current view state), mode transitions, menu rendering/input, Esc handling. Pure transition logic extracted for tests. |
| `src/view_dqn_train.rs` | `DqnTrainView`: wraps `GameDQN`; per-frame step; episode/best bookkeeping moved from old `main_dqn`; record → champion snapshot (AD-4); DQN-style draw (grid + HUD) centered via screen size; `Esc` pauses → menu; `R` starts a fresh agent. |
| `src/view_ga_train.rs` | `GaTrainView`: wraps `Simulation`; pacing loop (slow batch/fast batches) + keys as today; draws DQN-style HUD by default, `VizAdvanced` dashboard when `Tab` enables advanced; detects the sim's internal VS sub-state and routes drawing to the shared versus view (GA flavor). |
| `src/versus.rs` | Shared arena: `MatchPlayers` (two `Net`s + `VsFlavor`), `run_match_tick`, `resolve_winner(score1, score2)` (pure), and the versus view renderer used by DQN internal, GA internal, and cross views. |
| `src/view_dqn_versus.rs` | Composes champion vs live policy (AD-4), feeds `versus.rs`. |
| `src/view_ga_versus.rs` | Loads GA champions (`sim_metadata.json` best/second-best, fallback `best_snake.json`) and runs GA-flavor match. |
| `src/view_cross_match.rs` | Loads `best_snake.json` (GA) + `dqn_champion.json` (DQN) and runs the cross-flavor match; missing-file messages per spec. |
| `src/champion_store.rs` | Pure serde encode/decode of `Net` + fs save/load helpers (`dqn_champion.json`; generic path). Pure fns testable without fs. |
| `src/lib.rs` | Register the new modules. |
| `.gitignore` | Add `dqn_champion.json`. |
| `src/sim.rs` | Additive seam: public tick (logic-only) + metrics accessor; **existing `update()` semantics unchanged** during this change (GaTrainView drives via the seam; old draw path remains reachable through advanced toggle). |
| `src/pop.rs` | Expose the existing best-net loader (`load_best_net`) as `pub` for the cross/GA-versus views (no behavior change). |
| `src/game_dqn.rs` | No change required (champion snapshot lives in `DqnTrainView` via `agent.q_network` clone). |
| `src/viz_vs.rs` | Add `VsFlavor` config; keep default output byte-for-byte identical for GA flavor. |

## State & transition model

- Shell `App` fields: `mode: AppMode`, `menu_selection: usize`, `dqn: Option<DqnTrainView>` (paused trainer; `None` until first DQN train entry), `ga: Option<GaTrainView>` (created lazily on first GA entry), transient match state for versus modes.
- Transitions (pure fn `fn next_mode(mode, action, has_dqn) -> Transition` for tests):
  - `Menu + select n` → target view (constructs fresh for versus/cross; resumes or creates paused trainer for DQN/GA train).
  - Any view + `Esc` → `Menu` (paused trainers kept). `Menu + Esc` → quit.
  - In-match `Esc` → return to the owning view (DQN-versus match started from DQN train keeps trainer paused → back to DQN train) — versus-from-menu returns to Menu.
  - Versus/cross views + both games complete → show winner; `Esc`/`Enter` → Menu (or DQN train if originated there).
- Entering versus/cross from the menu constructs a fresh match every time (spec). DQN-versus and cross both read persisted champions; missing champion → message state, `Esc` back.

## Per-mode tick policy (one loop)

- `DqnTrain`: 1 `GameDQN::step()` per frame (today's cadence).
- `GaTrain`: same batching policy as today's `main.rs` (slow = 1 batch/frame + `SIM_SLEEP_MILLIS`; fast = batches of ≤50/frame), via the sim seam.
- Versus/cross matches: always slow — 1 step per player per frame (both `Game::update()` each frame), regardless of trainer pacing; no sleep needed at 60 fps.
- Menu: only input handling; nothing steps.

## Simulation seam (behavior-preserving)

Add to `sim.rs` (existing `update`, `toggle_vs_mode`, persistence untouched):

- `pub fn tick_training(&mut self)` / `pub fn tick_vs(&mut self)` — the logic halves of today's `update()` arms (generation lifecycle, vs stepping, auto-VS cadence), extracted verbatim so behavior is identical.
- `pub fn mode(&self) -> SimMode` and `pub fn vs_state(&self) -> (&Game, &Game)` accessors for the VS sub-state.
- `pub fn snapshot(&self) -> SimSnapshot` — `{ gen_count, gen_max, best_ever, elapsed_secs, champ_score, champ_fitness, champ_steps, best_game: Option<&Game> }` derived from existing `get_gen_summary`/`get_top_games`.
- Old `update(is_viz, slow)` becomes `tick` + optional legacy draw internally so any remaining caller keeps behavior; `GaTrainView` uses the seam and draws itself.

## DQN champion snapshot & persistence

- `DqnTrainView` holds `best_score` and `champion: Option<Net>`. On episode end with `score > best_score`: `best_score = score`; `champion = Some(agent.q_network.clone())`; `champion_store::save("dqn_champion.json", champion)`.
- Startup (DQN-versus / cross / trainer): `champion_store::load("dqn_champion.json")`; corrupt/missing → `Ok(None)`/message, never panic.
- `champion_store` pure core: `encode(net) -> String`, `decode(s) -> Result<Net,_>` (unit-tested round trip); fs helpers thin.

## Versus arena (shared)

- `MatchPlayers { net_left: Net, net_right: Net, flavor: VsFlavor }`.
- Runner holds two `Game::with_brain(net)`; each frame updates both; when both `is_complete` → `resolve_winner(g1.score(), g2.score()) -> Winner { Left, Right, Tie }`; render via `VizVS::draw_flavored(g1, g2, flavor)`.
- DQN internal: left = champion Net, right = live policy Net, flavor record `None`, titles "CHAMPION"/"CURRENT".
- GA internal: left = best-ever, right = second-best (fallback current top), GA-default flavor incl. record = best-ever (byte-identical output to today).
- Cross: left = GA `best_snake.json`, right = DQN `dqn_champion.json`, flavor titles "GA"/"DQN", record `None`.

## Pure-logic seams for strict TDD (RED → GREEN)

No pixel/UI tests. Targets, all macroquad-free:

1. `champion_store` encode/decode round trip + corrupt input → `Err`/None (fs layer mocked by string fns).
2. `DqnTrainView::on_episode_end(score, q_network)` bookkeeping → new `best_score`, `champion` replaced exactly on record (pure fn operating on returned structs).
3. `app::next_mode(...)` transition table incl. fresh-match vs resume and paused-trainer cases.
4. `versus::resolve_winner(1,0|0,1|1,1)` → Left/Right/Tie.
5. Headless arena termination: two random-nets `Game` matches complete within a bounded tick budget (guaranteed by existing no-food step limits) and yield a winner — no macroquad import in `game.rs`/`versus.rs` core.

UI drawing (`VizVS`, DQN-style HUD, menu) is untested by design.

## Work sequence & risk note

1. Champion store (+tests) → 2. sim seam → 3. `viz_vs` flavor (default output preserved) → 4. versus arena (+tests) → 5. DQN trainer view (move logic) → 6. GA trainer view → 7. menu/App shell (+transition tests) → 8. wiring, `Cargo.toml` single bin, `.gitignore`, docs. Estimated changed lines likely exceed the 400 review budget → delivery decision (single PR `size:exception` vs chained slices) is taken with the user after the tasks forecast.

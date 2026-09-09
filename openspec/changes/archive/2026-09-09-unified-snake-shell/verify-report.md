# Verify Report — unified-snake-shell

Change: `unified-snake-shell` · Branch `feature/dqn-vs-genetico` (local, no remote) · Artifact store: openspec · Strict TDD active

## Status: PASS (no blockers)

All 8 delta requirements demonstrably met in code; 18/18 implementation tasks checked with zero unchecked lines; build clean (0 warnings); `cargo test` → 54 passed / 0 failed (re-run twice this session, stable). No CRITICAL issues. Two WARNING-level documentation/labeling observations and one spec-language observation recorded (none change the verdict).

---

## Verification evidence (commands run)

| Command | Result |
| --- | --- |
| `cargo metadata --no-deps --format-version 1` | targets: `[('snake',['lib']), ('snake',['bin'])]` — exactly one bin |
| `cargo build --all-targets` (forced rebuild via `touch src/lib.rs src/main.rs`) | 0 warnings / 0 errors |
| `cargo test` | **54 passed / 0 failed** (lib); bin 0; doc 0 — run twice, stable |
| `cargo build --release` | compiles clean |
| `cargo test <mod>::` per module | champion_store 5 · viz_vs 3 · versus 7 · view_dqn_train 6 · view_dqn_versus 6 · view_ga_train 5 · view_ga_versus 5 · view_cross_match 6 · app 11 → **sum 54** |
| `git check-ignore -v dqn_champion.json` | `.gitignore:90` matched |

## Spec coverage (per delta requirement)

1. **Unified single-binary shell — MET.** `Cargo.toml` ships one `[[bin]] name = "snake"` (`src/main.rs`); `snake-dqn` bin and `src/main_dqn.rs` deleted (commit 954f591); zero remaining `snake-dqn`/`main_dqn` code refs. `AppMode` at `src/app.rs:24-44` covers exactly the six states `Menu | DqnTrain | DqnVersus | GaTrain | GaVersus | DqnVsGa`. Each mode owns its state (`App.dqn`/`App.ga` paused holders + transient `match_view`, `src/app.rs:186-210`); match views are constructed fresh on entry and dropped on menu return without touching trainer state.

2. **Welcome menu + keyboard navigation — MET.** Menu lists rows 1–5 (`src/app.rs:375-388`), selectable by number keys 1–5 or arrows+Enter (`handle_menu_input`, `src/app.rs:232-262`). Pure table `next_mode(mode, action, has_dqn, has_ga)` (`src/app.rs:104-135`): Menu+Esc → `Quit` (only quit path); any view+Esc → `ToMenu` without resetting (trainers retained); DQN-train entry resumes a paused holder (`DqnTrainResume`) vs fresh (`DqnTrainNew`); versus/cross entries always `*New` (fresh transient match). Menu shows "paused at episode N - press 1 to resume" for a live DQN run (`draw_menu`, `src/app.rs:349-356`) — resumable-paused indicator per spec.

3. **DQN train view + champion persistence — MET.** `DqnTrainView` (`src/view_dqn_train.rs`) wraps one `GameDQN`; `tick()` = one `step()`/frame with same-tick episode reset; HUD Episode/Score/Best/Epsilon + centered grid (`draw`, lines 197-270). Pure record seam `on_episode_end(score, best, q_network) -> (usize, Option<Net>)` (lines 36-50) replaces champion exactly on `score > best`, never on tie/lower. On record, `champion_store::save(DQN_CHAMPION_FILE, net)` persists (`end_episode`, lines 95-125) and "NEW DQN RECORD" prints. `champion_store.rs` `load()` degrades corrupt/missing → `None` (never panics); `.gitignore:90` covers `dqn_champion.json`. Replay guard intact: `dqn.rs:106` `if self.replay_buffer.len() < BATCH_SIZE { return; }` — no weight update until ≥32 experiences.

4. **DQN internal versus — MET.** Pure `plan_dqn_versus(champion, live)` (`src/view_dqn_versus.rs:77-91`): MissingChampion / ChampionVsLive / ChampionVsFresh. Fresh-greedy fallback (`DQNAgent::new().q_network`) labeled "CURRENT (fresh)" via `live_is_fresh` (flavor fn lines 108-128). No-champion message state `CHAMPION_MISSING_MESSAGE` "Train DQN first…" with Esc-to-menu. Shell `build_dqn_versus` (`src/app.rs:318-329`) prefers paused-trainer champion + live q_network, falls back to `dqn_champion.json`. Arena is greedy: `game.rs:62-68 get_brain_output` always argmaxes (ε-zero equivalence), one `Game::update()` per tick per player (`versus.rs:113-121`). Winner resolved by higher score with documented Tie rule (`resolve_winner`, `versus.rs:25-31`).

5. **GA train DQN-style + unchanged evolution + advanced dashboard — MET.** `GaTrainView` (`src/view_ga_train.rs`) drives `Simulation` via the behavior-preserving seam: `tick()` selects `tick_training`/`tick_vs` from live `sim.mode()` per iteration with `frame_tick_budget` (slow/VS = 1 + `SIM_SLEEP_MILLIS` sleep; fast ≤ 50/frame, legacy cap). Default render = DQN-style single grid + HUD (Generation/Gen Max/Best Ever/Elapsed/Score/Fitness/Steps), `render_target` pure gate; Tab → `toggle_advanced` → `sim.draw_advanced()` (`sim.rs:308`) untouched `VizAdvanced`. Cadence unchanged in `sim.rs`: gen lifecycle (`tick_training` 175-189), auto-VS every-100 (`is_multiple_of(100)` line 182), `best_snake.json`+`sim_metadata.json` every-10 persistence (lines 302-304) — byte-comparable to pre-change `main` `sim.rs` modulo the documented `games_alive <= 0 → == 0` (usize) and `% → is_multiple_of` rewrites.

6. **GA internal versus standalone — MET.** `GaVersusView` + pure `plan_ga_versus(GaChampions)` (`src/view_ga_versus.rs:48-61`): best required; missing second-best falls back to clone of best (mirrors legacy `Simulation::toggle_vs_mode` degenerate tail); both missing → `GA_CHAMPIONS_MISSING_MESSAGE`. `load_ga_champions` (`sim.rs:78`) reads `sim_metadata.json` best/second-best + all-time record with `best_snake.json` fallback; missing/corrupt → no champions, no panic. `VsFlavor::ga_default(record)` (`viz_vs.rs:112-130`) reproduces legacy strings/colors byte-for-byte — pinned by unit tests (`ga_default_flavor_pins_legacy_strings/colors`).

7. **DQN vs GA cross-match — MET.** `CrossMatchView::new` loads GA from `Population::load_best_net()` (`best_snake.json`, `pop.rs:141-147` pub) and DQN from `champion_store::load(DQN_CHAMPION_FILE)`. Pure `plan_cross_match`/`cross_missing_message` (`src/view_cross_match.rs:46-70`) names the missing side (spec scenario pinned: GA present, DQN missing → `DQN_MISSING_MESSAGE`, no crash); both present → `Ready` match, cross flavor "GA"/"DQN", `record: None`. Brains are the shared serde `Net` (12→8→4, `configs.rs:29-31`); DQN `q_network` and GA arena inputs are both 12-dim so `Net::predict` shape guard never panics (encoding parity nuance is a documented, deferred MVP risk in proposal §Cross-match).

8. **No-regression reachability — MET.** Old GA entry behavior = menu → 3 GA train (`Simulation` seam, legacy batching/sleep, Space slow/fast, V internal VS forcing slow, Tab advanced dashboard; legacy auto-return from VS preserved in `tick_vs`). Old DQN entry behavior = menu → 1 DQN train (identical 1-step/frame + HUD cadence to deleted `main_dqn.rs`; only console-log cadence differs — legacy printed every episode end, new prints on records only). GA persistence cadence unchanged (see req 5). Legacy `VizVS::draw` still routes through GA-default so any remaining legacy caller output is unchanged.

## Task completion

All 18/18 tasks `[x]` in `tasks.md` (T0.1–T0.3, T1.1–T1.3, T2.1–T2.3, T3.1–T3.2, T4.1–T4.3, T5.1–T5.4). Zero unchecked `- [ ]` implementation lines (grep confirmed). Apply-progress records all slices 1–5 with no deferred parent-owned actions. **No archive blockers from unchecked tasks.**

## Structured status / actionContext findings

Native status: `artifactStore: openspec`, change `unified-snake-shell`, `applyState: all_done`, `verify: ready` (dependencies verified), `isNonAuthoritative: false`, `actionContext.mode: repo-local`, allowedEditRoots `[<repo-root>]`, no warnings. Working tree clean on `feature/dqn-vs-genetico`; change commits 66d2839 (s1), 05d5f15 (s2), b233f3e (s3), 17401fa (s4), 954f591 (s5) + planning commit 6b31e0d. All edited paths inside the workspace/allowed roots. Lint-only edits to `viz_advanced.rs`/`viz.rs`/`dqn.rs` predate the change (chore commit f45997a, branch-only) — the change commits themselves did not touch `VizAdvanced` (spec "MUST NOT be modified" satisfied for this change).

## Strict TDD compliance

- TDD Cycle Evidence table present in apply-progress (each slice + T5.1 RED→GREEN rows).
- RED→GREEN per slice coherent: E0425/E0432/E0433 unresolved-import failures on not-yet-implemented seams → implementation → green; GREEN-fix and triangulate rows record genuine flake/round-trip lessons. Slice baselines (0→8→15→27→43→54) are consistent arithmetic with the final suite.
- Reported test files exist in codebase and pass (`cargo test` 54/54, twice).
- Slice-4 per-module test-count documentation is internally swapped vs actual: claims "6 view_ga_versus + 5 view_cross_match", actual **5 + 6** (same sum 54). WARNING (documentation only — total correct, no evidence of fabrication).
- N/A rows (wiring/deletion/docs/verification tasks) are honest — no new pure behavior in those tasks.
- Assertion quality: ✅ All assertions verify real behavior. Audit of all 9 test modules: no tautologies, no type-only-only assertions, no ghost loops (the multi-trial random-brain loops run fixed 20×/5-view constant iterations and assert inside every iteration), no smoke-only tests, no CSS/impl-detail assertions. Randomness handled by bounded-termination budgets + suite re-runs (no flake observed here).

## Review workload / PR boundary

Chained plan implemented as 5 independently compilable/testable slices matching the forecast and the tasks.md `Review Workload Forecast` (chained PRs recommended → chained slices on a local branch, no remote/PR). Cumulative change ~4,058 insertions across the branch delta (well above the 400-line budget as forecast; chaining was the chosen mitigation — no `size:exception` was used). No scope creep beyond assigned tasks detected.

## Manual-smoke items that cannot be verified headless (for the user to run)

GUI rendering/input cannot be exercised in this headless environment. Per apply-progress checklist:

1. `cargo run --release` — fullscreen window `snake-ai` opens on the menu; rows 1–5 with selection highlight and footer key hint.
2. Menu `Esc` quits the process.
3. `1` (DQN train) → episodes step (Episode/Score/Best/Epsilon HUD + grid); `Esc` → menu shows "paused at episode N - press 1 to resume"; `1` resumes at N; `R` resets the agent (champion kept); on a record `dqn_champion.json` appears and a NEW DQN RECORD line prints.
4. `2` (DQN versus) with a champion → CHAMPION vs CURRENT runs to a winner; `Esc`/`Enter` after finish → menu; with no champion (fresh checkout) → "Train DQN first" message, `Esc` → menu.
5. `3` (GA train) → generation/gen-max/best-ever HUD; `Space` slow/fast, `Tab` advanced dashboard, `V` internal GA VS (auto-returns); `Esc` → menu (row shows paused).
6. `4` (GA versus) → BEST EVER vs 2ND BEST match (record panel); `Esc` after finish → menu.
7. `5` (DQN vs GA) with both `best_snake.json` + `dqn_champion.json` → cross match; missing one side → per-side message, `Esc` → menu.

## Observations (non-blocking)

- **WARNING (docs):** slice-4 apply-progress per-module test counts for view_ga_versus/view_cross_match are swapped (6+5 claimed vs 5+6 actual); totals unaffected.
- **OBSERVATION (spec language):** "both players observe the same arena state sequence" is implemented as two independent `Game::with_brain` instances (identical rules, deterministic argmax, one move/tick) — byte-identical to the legacy GA-VS arena; a truly shared single board would be the explicitly deferred game-core refactor (proposal Non-Goals).
- **OBSERVATION:** legacy GA driver's "Tab-off ⇒ auto-fast" coupling is not carried into the shell (Tab toggles only the dashboard; pacing stays on Space). Space/V/Tab meanings are preserved at the semantic level the spec pins.
- **OBSERVATION:** DQN console logging differs from the deleted driver (records-only vs every-episode print); not spec-governed.

## Exact blockers

None. Verify is clean → sync (`sdd-sync`) and archive are unblocked on the parent side.

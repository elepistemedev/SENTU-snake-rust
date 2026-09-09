# Apply Progress — unified-snake-shell (Slices 1–5 of 5)
    
Slice 5 (this file's top record): shell + wiring (T5.1–T5.4) — complete. Slices 1–4 records preserved verbatim below.
    
## Slice 5 — Menu shell + wiring + docs (completed)
    
Slice 5: `src/app.rs` shell + unified entry (`main.rs`), `main_dqn.rs` deleted, single bin, docs. Delivery: chained slices on local branch `feature/dqn-vs-genetico` (no remote/PR). Not committed. This is the final implementation slice.
    
## Completed tasks (persisted checkboxes updated)
    
| Task | Summary | Checkbox |
| --- | --- | --- |
| T5.1 | `src/app.rs` (new): `enum AppMode { Menu, DqnTrain, DqnVersus, GaTrain, GaVersus, DqnVsGa }`; pure seam `next_mode(mode, action, has_dqn, has_ga) -> Transition` (outcomes incl. `Quit`/`Stay`/`ToMenu`, `DqnTrainNew`/`DqnTrainResume`, `DqnVersusNew`, `GaTrainNew`/`GaTrainResume`, `GaVersusNew`, `CrossNew`) — **[RED-first]** 11 unit tests: menu Esc quits (the only quit path), any-view Esc → menu (paused trainers retained), DQN/GA train select fresh-vs-resume by `has_dqn`/`has_ga`, versus/cross entries always-fresh transient matches, invalid number keys ignored, Enter semantics (menu Enter is a shell translation into `Select`; match views dismiss finished results, trainer views ignore Enter), `Select` ignored inside views. `App` holds `mode`, `menu_selection`, paused `dqn: Option<DqnTrainView>` + `ga: Option<GaTrainView>` (AD-3 holders, created lazily, kept across menu visits) and a transient `match_view: Option<MatchView>` (DqnVersus/GaVersus/Cross) dropped on every menu return. Menu rendering: centered rows 1–5 with selection highlight (numbers or arrows+Enter), DQN row shows “paused at episode N - press 1 to resume” when a session is alive. Input routing: views are passive — the shell reads keys, forwards `R` → `fresh_agent()`, `Space` → GA `toggle_slow`, `Tab` → `toggle_advanced`, `V` → `trigger_vs`, and routes Esc via the pure table. Per-mode tick: DQN train 1 tick/frame, GA train via the view's pacing (its own sleep/batching), match views 1 tick/frame, menu no-op. `build_dqn_versus` composes champion + live policy from the paused trainer, falling back to `champion_store::load(dqn_champion.json)` and the view's fresh-greedy/message paths when no trainer exists (spec: “uses persisted champion or shows its message”). | `[x]` in tasks.md |
| T5.2 | Wiring: `src/main.rs` rewritten as the thin shell entry (window `snake-ai`, `fullscreen: true` per AD-8, macroquad loop calling `App::run_frame()` until it returns quit); `src/main_dqn.rs` deleted; `snake-dqn` bin removed from `Cargo.toml` (manifest now ships exactly one bin: `snake` — confirmed via `cargo metadata`); `pub mod app;` registered in `src/lib.rs`. | `[x]` in tasks.md |
| T5.3 | Docs: `README.md` gained a “Snake Rust — app unificada” usage section (single `cargo run --release`, menu table, per-view keys, champion files); `DQN_README.md` usage rewritten for the single binary (`snake-dqn` no longer exists) with unified menu + per-view key tables, controls updated (Esc pauses/back-to-menu on views, quits on menu; R fresh agent) and the “Guardar/cargar modelos entrenados” roadmap item checked with the `dqn_champion.json` note. | `[x]` in tasks.md |
| T5.4 | Full verification: `cargo build` clean (0 warnings via `--all-targets`), `cargo test` → **54 passed / 0 failed** (43 baseline + 11 new `app` tests; lib: 5 champion_store + 3 viz_vs + 7 versus + 6 view_dqn_train + 6 view_dqn_versus + 5 view_ga_train + 6 view_ga_versus + 5 view_cross_match + 11 app), `cargo build --release` compiles. Suite re-run 3× consecutively — 54/54 each run, no flake. GUI smoke not possible headless (checklist below). | `[x]` in tasks.md |
    
## Files changed (slice 5)
    
- `src/app.rs` — new (pure transition seam + `App` shell: menu, input routing, tick/draw dispatch, transient `MatchView` + 11 unit tests)
- `src/main.rs` — rewritten as the shell entry (window `snake-ai`, fullscreen, `App::run_frame()` loop)
- `src/main_dqn.rs` — deleted (behavior lives in `view_dqn_train.rs`)
- `Cargo.toml` — removed the `snake-dqn` `[[bin]]` block
- `src/lib.rs` — added `pub mod app;`
- `README.md`, `DQN_README.md` — usage docs (unified app + menu keys + `dqn_champion.json`)
- `openspec/changes/unified-snake-shell/tasks.md` — checked T5.1–T5.4 (all 18 tasks now `[x]`)
- `openspec/changes/unified-snake-shell/apply-progress.md` — this file (merged)
    
Not modified in this slice (as required): all slice-2/3/4 view/versus/viz/sim/pop modules, `src/viz_advanced.rs`, `src/game*.rs`, `src/nn.rs`, `src/champion_store.rs`, `src/configs.rs`, `.gitignore` (already carries `dqn_champion.json` from slice 1). No one-line API fixes were needed. No pixel tests.
    
## TDD Cycle Evidence
    
Runner: `cargo test`. Baseline: `cargo test` → 43 passed (slices 1–4), both bins 0.
    
| Task | RED | GREEN | Command |
| --- | --- | --- | --- |
| T5.1 | Registered `pub mod app;` in `lib.rs` and wrote the test module in `src/app.rs` referencing undefined seams (`next_mode`, `Action`, `AppMode`, `Transition`) → `cargo test app` failed: E0432 unresolved imports (super::*) | Implemented the pure transition table + full `App` (menu/input/tick/draw/MatchView) → `cargo test app` 11/11 pass | `cargo test app` |
| T5.1 (cleanup) | First GREEN draft kept a leftover unused-var hack in `draw_dqn_train` and a static `dqn_status_text` helper that made the DQN row uninformative | Rewrote `draw_menu` (dynamic `format!` labels incl. “paused at episode N”) and `draw_dqn_train` (direct `draw_text`, no dead code); then `rustfmt` on the two authored files | `cargo build` + `cargo test` |
| T5.2 | N/A (wiring/deletion of an entry point introduces no new pure behavior to RED-first) | Shell `main.rs` + bin removal + module registration → single-bin manifest (`cargo metadata` → bins `['snake']`), suite green | `cargo build` + `cargo test` + `cargo metadata` |
| T5.3 | N/A (docs) | `README.md`/`DQN_README.md` rewritten usage sections | (markdown only) |
| T5.4 | N/A (verification) | `cargo build` + `cargo test` green; suite re-run 3× → 54/54 every run; `cargo build --release` compiles | `cargo test` ×3, `cargo build --release` |
    
Final suite: `cargo test` → 54 passed / 0 failed. `cargo build --all-targets` clean, 0 warnings. Only the `snake` bin ships (lib 54 tests; bin 0). Pure seam (`next_mode`/`AppMode`/`Action`/`Transition`) is macroquad-free.
    
## Deviations from design
    
- Single `Transition` enum with named arms encoding both the destination mode and the construction decision (New/Resume), rather than the design sketch of a bare mode plus separate flags; the pure fn keeps the slice-5 mandated signature `next_mode(mode, action, has_dqn, has_ga)` (design.md's earlier `(mode, action, has_dqn)` sketch predates the `has_ga` instruction).
- `Menu + Enter` returns `Transition::Stay` in the pure table: the pure fn does not hold `menu_selection`, so arrows+Enter is the shell converting the highlighted row into `Action::Select(n)` (number keys stay first-class). `Up/Down` only move `menu_selection` in the shell.
- `Enter`-to-dismiss is gated by the shell: `next_mode` maps Enter from match views to `ToMenu`, but the shell only forwards Enter when the transient match reports `is_finished()`, so a stray Enter never aborts a running match.
- Versus-from-menu is the only versus origin in this slice (matches design “versus-from-menu returns to Menu”); the in-train-view versus origin was not reachable in the MVP menu, so `Esc` from any match view returns to the menu directly.
- DQN-versus entry prefers the paused trainer's in-memory champion and live q-network (AD-3/AD-4); with no trainer it falls back to `champion_store::load(DQN_CHAMPION_FILE)` and the `DqnVersusView` fresh-greedy/message paths — the “no trainer yet” spec branch is composed from already-tested seams (`plan_dqn_versus`, store load) rather than duplicated in `app.rs`.
- DQN-train hotkey hint (“[R] fresh agent  [ESC] menu”) is overlaid by the shell in the free bottom-left HUD column after `DqnTrainView::draw()` because the slice-3 view file was out of the allowed edit surfaces; the grid never occupies that corner (it starts at the HUD column x-offset).
- GA paused-row status in the menu is generic (“paused - press 3 to resume”) because `GaTrainView` exposes no generation-count accessor and its file was out of allowed surfaces; the spec's resumable-indicator requirement (episode-level) is fully met for the DQN row and met at the resumable level for GA.
    
## Remaining tasks
    
None — all 18 implementation tasks are `[x]` in `tasks.md`; no deferred parent-owned actions were listed for this change.
    
## Manual smoke checklist (headless verification impossible — for the user)
    
1. `cargo run --release` — fullscreen window `snake-ai` opens on the menu; rows 1–5 with selection highlight and a footer key hint.
2. Menu `Esc` quits the process.
3. `1` (DQN train) → episodes step (Episode/Score/Best/Epsilon HUD + grid); `Esc` returns to the menu and the row shows “paused at episode N - press 1 to resume”; `1` again resumes at N; `R` resets the agent (champion kept); on a record, `dqn_champion.json` appears and a NEW DQN RECORD line prints.
4. `2` (DQN versus) with a champion → CHAMPION vs CURRENT match runs to a winner; `Esc`/`Enter` after finish → menu; with no champion (fresh checkout) → “Train DQN first” message, `Esc` → menu.
5. `3` (GA train) → generation/gen-max/best-ever HUD; `Space` toggles slow/fast, `Tab` shows the advanced dashboard, `V` runs the internal GA VS (renders versus, then auto-returns); `Esc` → menu (row shows paused).
6. `4` (GA versus) → BEST EVER vs 2ND BEST match (record panel); `Esc` after finish → menu.
7. `5` (DQN vs GA) with both `best_snake.json` and `dqn_champion.json` → cross match; missing one side → per-side message, `Esc` → menu.
    
## Workload / PR boundary
    
Slice 5 is the final slice of the chained plan: ~700 added lines in `src/app.rs` + main/lib/Cargo churn + 2 doc files; `main_dqn.rs` deleted (−132). The complete change spans slices 1–5 (cumulative well above the 400-line review budget, as forecast); each slice was independently compilable/testable and this one completes the full `unified-snake-shell` change. Ready for parent-lifecycle (verify).
    
## Structured status consumed
    
Authoritative native status (change `unified-snake-shell`): `applyState: ready`, `artifactStore: openspec`, `actionContext.mode: repo-local`, allowed edit roots `[<repo-root>]`, no warnings; the parent prompt resolved the delivery path (chained slice 5 of 5, no commit). All edited paths inside the authoritative workspace and the slice's allowed edit surfaces. Strict TDD followed for the testable seam (T5.1 RED→GREEN→TRIANGULATE); wiring/docs/verification have no new pure behavior to RED-first (recorded per task).
    
---
    
## Slice 4 — GA side + cross-match views (completed)
    
## Completed tasks (persisted checkboxes updated)
    
| Task | Summary | Checkbox |
| --- | --- | --- |
| T3.1 | `src/view_ga_train.rs` (new): `GaTrainView` wraps a live `Simulation` through the slice-1 seam. Pure pacing seam `frame_tick_budget(slow_requested, vs_active) -> usize`: slow → 1 tick/frame; active sim-internal VS forces 1 (legacy `is_slow_mode || sim.is_vs_mode()` recomputed per frame); fast → `MAX_FAST_TICKS_PER_FRAME` (50, the legacy driver's cap). `tick()` picks `tick_training`/`tick_vs` per iteration from the sim's live mode so an auto-VS beginning mid-batch behaves like the driver, and sleeps `SIM_SLEEP_MILLIS` when the budget is 1. Pure `render_target(advanced, mode) -> GaRenderTarget` (Hud, Advanced, Versus) gates rendering: default DQN-style single grid (current best snake via `snapshot().best_game`) + HUD (Generation / Gen Max / Best Ever / Elapsed / Score / Fitness / Steps) centered from `screen_width()` (AD-8; grid_layout family as `DqnTrainView`); `Tab` → `toggle_advanced()` switches to the untouched `VizAdvanced` dashboard through the new `pub Simulation::draw_advanced()`; the sim's internal VS sub-state always routes to `VizVS::draw_flavored` + `VsFlavor::ga_default(best_ever)` (legacy byte-identical VS look; sim keeps auto-returning to Training). Key seams exposed for the slice-5 shell: `toggle_slow` (Space), `toggle_advanced` (Tab), `trigger_vs` (V; preserves the legacy “entering VS forces slow” side effect), plus `mode()/is_vs_active()/is_slow()/advanced_enabled()`. `Esc` stays shell-owned. | `[x]` in tasks.md |
| T3.2 | `src/pop.rs`: `Population::load_best_net` made `pub` (reads `best_snake.json`, missing/corrupt → `None`) — no behavior change; used by the cross view and as the sim fallback. | `[x]` in tasks.md |
| T4.2 | `src/sim.rs` + `src/view_ga_versus.rs` (new): `sim::GaChampions { best, second_best, record }` + `pub fn load_ga_champions()` reads `sim_metadata.json` best/second-best nets and the all-time record, falling back to `best_snake.json` for the best when the metadata lacks it (reuses the existing private `Simulation::load_metadata`; missing/corrupt → no champions, never panics). Pure seam `plan_ga_versus(GaChampions) -> GaVersusPlayers` (Missing, Match { best, second_best }) — best required; missing second-best falls back to a clone of the best (mirrors `Simulation::toggle_vs_mode`'s degenerate fallback). `GaVersusView::new()` = fresh match with `VsFlavor::ga_default(record)` (RECORD TO BEAT/NEW!/best-ever, byte-identical to today's GA VS); message state `GA_CHAMPIONS_MISSING_MESSAGE` (“Train GA first…”) exposed via `message()` when best is missing (both-missing case), plus `tick()/is_finished()/winner()/draw()`; `from_champions` private constructor injects champions so tests never touch the real JSON files. | `[x]` in tasks.md |
| T4.3 | `src/view_cross_match.rs` (new): pure `plan_cross_match(ga: Option<Net>, dqn: Option<Net>) -> CrossMatchPlayers` (Ready, MissingGa, MissingDqn, MissingBoth) + `cross_missing_message(&plan) -> Option<&'static str>` naming the missing side (spec scenario pinned: GA present, DQN missing → message names the DQN side, no crash). `CrossMatchView::new()` loads GA from `Population::load_best_net()` (`best_snake.json`) and DQN from `champion_store::load(DQN_CHAMPION_FILE)` (`dqn_champion.json`); both present → `VersusMatch` with cross `VsFlavor`: titles “GA”/“DQN”, distinct colors (GA green vs DQN blue), `record: None`, “GA WINS!/DQN WINS!/TIE!”, back/controls “[ESC] Menu”. Message state (per-side text, centered + “[ESC] Menu”) exposed via `message()`; `from_nets` private constructor for headless tests. | `[x]` in tasks.md |
    
## Files changed (slice 4)
    
- `src/view_ga_train.rs` — new (`GaTrainView` + pure `frame_tick_budget`/`render_target` seams + 5 unit tests)
- `src/view_ga_versus.rs` — new (`GaVersusView` + `plan_ga_versus`/`GaVersusPlayers` + 6 unit tests)
- `src/view_cross_match.rs` — new (`CrossMatchView` + `plan_cross_match`/`CrossMatchPlayers`/`cross_missing_message` + 5 unit tests)
- `src/sim.rs` — added `GaChampions` + `pub fn load_ga_champions()` (additive; reuses private `load_metadata`) and made `draw_advanced` `pub` (additive draw accessor) — evolution behavior + persistence untouched
- `src/pop.rs` — `load_best_net` → `pub` (no behavior change; + doc comment)
- `src/lib.rs` — registered `pub mod view_cross_match; pub mod view_ga_train; pub mod view_ga_versus;`
- `openspec/changes/unified-snake-shell/tasks.md` — checked T3.1, T3.2, T4.2, T4.3
- `openspec/changes/unified-snake-shell/apply-progress.md` — this file (merged)
    
Not modified in this slice (as required): `src/main.rs`, `src/main_dqn.rs`, the DQN view files, `src/viz_advanced.rs`, `src/viz_vs.rs`, `src/game.rs`, `src/nn.rs`, `src/versus.rs`, `src/champion_store.rs`, `Cargo.toml`. Both old binaries compile unchanged (wiring/deletion is slice 5). No pixel tests.
    
## TDD Cycle Evidence
    
Runner: `cargo test`. Baseline: `cargo test` → 27 passed (slices 1–3), both bins 0.
    
| Task | RED | GREEN | Command |
| --- | --- | --- | --- |
| T3.1 | Registered the 3 new modules in `lib.rs` and wrote test-only module files referencing undefined seams (`frame_tick_budget`, `render_target`, `GaRenderTarget`) → `cargo test --no-run` failed: E0432 unresolved imports (super::*) | Implemented the pure seams + full `GaTrainView` → suite green | `cargo test` |
| T4.2 | Same RED run also referenced `crate::sim::GaChampions` (not yet in sim.rs) → E0433 unresolved import | Added `GaChampions`/`load_ga_champions` to sim.rs + `GaVersusView` → green | `cargo test` |
| T4.3 | Same RED run referenced `plan_cross_match`/`CrossMatchPlayers`/message consts (undefined) → E0432 | Implemented planner/messages/`CrossMatchView` → green | `cargo test` |
| T3.2 / sim accessors | N/A (visibility-only change + additive accessor; no new behavior to RED-first) | `pub fn load_best_net`, `pub fn draw_advanced`, `GaChampions` load added | `cargo build` + `cargo test` green |
| GREEN fix | First GREEN run failed E0382 in the cross test `both_champions_present_produce_a_ready_match`: the `Ready { ga, dqn }` match arm moved the nets out of `plan`, then `cross_missing_message(&plan)` borrowed the partially moved value | Reordered the test (assert message first, then consume nets); refactored `from_nets` to an `if let` + `unwrap_or(BOTH_MISSING_MESSAGE)` so production code has no `expect` | `cargo test` |
| Triangulate | Random-brain match/finish tests (bounded 10 000-tick budgets) could flake on OS-entropy brains | Ran the full suite 3× consecutively → 43/43 every run; no flake | `cargo test` ×3 |
    
Final suite: `cargo test` → 43 passed / 0 failed (lib: 5 champion_store + 3 viz_vs + 7 versus + 6 view_dqn_train + 6 view_dqn_versus + 5 view_ga_train + 6 view_ga_versus + 5 view_cross_match; both bins 0 tests). `cargo build --all-targets` clean, 0 warnings. Pure seams (`frame_tick_budget`, `render_target`, `plan_ga_versus`, `plan_cross_match`, `cross_missing_message`) are macroquad-free; `Net` cloning only.
    
## Deviations from design
    
- Pacing/keys live on the view as seams, not inside a loop: slice 4 has no shell frame loop yet, so `GaTrainView` exposes `toggle_slow`/`toggle_advanced`/`trigger_vs` methods plus a `tick()` that already applies the legacy batching + sleep policy; the slice-5 shell just forwards key events. `trigger_vs` preserves the legacy “entering VS forces slow mode” side effect internally so the shell need not know GA pacing rules. `Esc` handling is deliberately absent (shell-owned per prior slices).
- Standalone GA-versus player-2 fallback: with no training running there is no “current generation top” to borrow, so when `sim_metadata.json` has a best net but no second-best, player 2 falls back to a clone of the best net (the same degenerate tail `Simulation::toggle_vs_mode` uses when both are missing). Message state only when no best-ever net exists anywhere.
- `GaChampions.record` is `usize` (0 when the metadata file is absent); the fallback-only `best_snake.json` case therefore shows record 0 rather than an unknown, keeping `VsFlavor::ga_default`'s `usize` signature (byte-identical layout is only guaranteed when metadata exists, which is the trained-GA case).
- Cross DQN accent is a distinct blue (GA green vs DQN blue) instead of reusing the arena's red second-player color, so cross and GA-internal matches read differently at a glance; all strings still match spec.
- One production `.expect` was removed during GREEN (see evidence) to keep panics-out-of-reach style; the missing-side message is derived via `cross_missing_message(...).unwrap_or(BOTH_MISSING_MESSAGE)` guarded by an `if let Ready` early return.
- `load_ga_champions`/`Population::load_best_net` read fixed relative paths (`sim_metadata.json`/`best_snake.json`); like the DQN champion store they degrade to `None` on missing/corrupt input and are deliberately not fs-unit-tested (CWD-dependent). Availability logic is tested via the pure planners.
    
## Remaining tasks (out of this slice — exact unchecked lines)
    
Slice 4 is complete; remaining implementation tasks belong to slice 5 and stay unchecked:
    
- [ ] T5.1 `src/app.rs`: `AppMode`, `App` (paused DQN trainer + GA trainer holders, transient match state), menu rendering/input (numbers/arrows+Enter/Esc); pure `next_mode(...)` transition table. **[RED-first]** tests: fresh-match vs resume, paused-trainer cases, Esc semantics.
- [ ] T5.2 Replace `src/main.rs` entry with the shell loop; delete `src/main_dqn.rs`; remove `snake-dqn` bin from `Cargo.toml`; register new modules in `src/lib.rs`.
- [ ] T5.3 Docs: update `README.md`/`DQN_README.md` usage to the unified app + menu keys; note `dqn_champion.json`.
- [ ] T5.4 Full `cargo build` + `cargo test` green; manual smoke of each menu → view → Esc path (no pixel tests, per design).
    
## Workload / PR boundary
    
Slice 4 is a clean, independently compilable/testable unit: ~950 added lines across 3 new view files + additive sim/pop accessors + lib.rs/docs churn; both bins still compile unchanged (`main.rs`/`main_dqn.rs` untouched until slice 5). Next PR slice boundary candidate: the menu shell + wiring + docs (T5.1–T5.4).
    
## Structured status consumed
    
Authoritative native status (change `unified-snake-shell`): `applyState: ready`, `artifactStore: openspec`, `actionContext.mode: repo-local`, allowed edit roots `[<repo-root>]`, no warnings. All edited paths inside the authoritative workspace and the slice's allowed edit surfaces (`src/view_ga_train.rs`, `src/view_ga_versus.rs`, `src/view_cross_match.rs`, `src/sim.rs`, `src/pop.rs`, `src/lib.rs`, `tasks.md`, `apply-progress.md`). Note: one accidental `cargo fmt` pass reformatted files outside the allowed surfaces; those were restored byte-identical from HEAD via `git checkout` and do not appear in the final diff.
    
---
    
## Slice 3 — DQN trainer view + champion + DQN versus (completed)

## Completed tasks (persisted checkboxes updated)

| Task | Summary | Checkbox |
| --- | --- | --- |
| T2.2 | Pure record seam `view_dqn_train::on_episode_end(score, best_score, &Net) -> (usize, Option<Net>)`: returns `(new_best, Some(q_network.clone()))` exactly when `score > best`; tie/lower keep best and return `None` (champion never replaced by an equal/worse run). **[RED-first]** tests pin exact snapshot on record and `None` on tie/lower. | `[x]` in tasks.md |
| T2.1 | `src/view_dqn_train.rs` (new): `DqnTrainView` owns a `GameDQN`; `tick()` = one `step()` per frame + episode/best bookkeeping via `on_episode_end` + same-tick reset; HUD (Episode/Score/Best/Epsilon, left-anchored as today) + grid sized/centered from `screen_width()/height()` (no hardcoded 800×600 offsets; `grid_layout()` reproduces the old family at 800×600). Passive/resumable per AD-3 (shell pauses by not ticking). Accessors: `score/episode/best_score/epsilon/live_net/champion`. `fresh_agent()` for the shell's `R` key (resets agent+bookkeeping, retains champion). | `[x]` in tasks.md |
| T2.3 | Champion persistence wired: `end_episode` saves the champion via `champion_store::save(self.champion_path, …)` on every record; `new()` loads an existing champion via `champion_store::load(DQN_CHAMPION_FILE)` (missing/corrupt → `None`, never panics — the fs layer is already covered by slice-1 store tests). Constant `DQN_CHAMPION_FILE = "dqn_champion.json"`. Private `at_path` constructor keeps the real file out of tests. | `[x]` in tasks.md |
| T4.1 | `src/view_dqn_versus.rs` (new): pure seam `plan_dqn_versus(Option<&Net>, Option<&Net>) -> DqnVersusPlayers { MissingChampion | ChampionVsLive{…} | ChampionVsFresh{…} }` — **[RED-first]** which nets are selected given trainer/champion presence. `DqnVersusView::new(champion: Option<Net>, live: Option<Net>)`: missing champion → message state (`CHAMPION_MISSING_MESSAGE`, exposed via `message()`); live `None` → fresh greedy agent (`DQNAgent::new().q_network`) labeled "CURRENT (fresh)". DQN `VsFlavor` "CHAMPION"/"CURRENT", `record: None`, "CHAMPION WINS!/CURRENT WINS!/TIE!", back label "[ESC] Menu". Drives `VersusMatch` per frame (`tick`); `draw` via the match (or the message notice); `is_finished()/winner()` exposed. No input handling (shell owns Esc, slice 5). | `[x]` in tasks.md |

## Files changed (slice 3)

- `src/view_dqn_train.rs` — new (`DqnTrainView` + `on_episode_end` + 6 unit tests)
- `src/view_dqn_versus.rs` — new (`DqnVersusView` + `plan_dqn_versus`/`DqnVersusPlayers` + 6 unit tests)
- `src/lib.rs` — registered `pub mod view_dqn_train;` + `pub mod view_dqn_versus;`
- `openspec/changes/unified-snake-shell/tasks.md` — checked T2.1, T2.2, T2.3, T4.1
- `openspec/changes/unified-snake-shell/apply-progress.md` — this file (merged)

Not modified in this slice (as required): `src/main_dqn.rs` (kept compiling; deleted in wiring slice 5), `Cargo.toml`, `src/main.rs`, `src/game_dqn.rs`, `src/dqn.rs`, `src/nn.rs`, `src/game.rs`, `src/versus.rs`, `src/viz_vs.rs`, `src/champion_store.rs`, any GA sources. No pixel tests.

## TDD Cycle Evidence

Runner: `cargo test`. Baseline: `cargo test` → 15 passed (slices 1–2), both bins 0.

| Task | RED | GREEN | Command |
| --- | --- | --- | --- |
| T2.2 | Wrote 3 tests in `view_dqn_train.rs` referencing undefined `on_episode_end` → compile failed E0425 (module registered in lib.rs so the seam was reachable) | Implemented `on_episode_end` (+ whole `DqnTrainView`) → `cargo test view_dqn_train` green | `cargo test` |
| T4.1 | Wrote 3 tests in `view_dqn_versus.rs` referencing undefined `plan_dqn_versus`/`DqnVersusPlayers` → compile failed E0425/E0433 (9 errors across both seams in one run) | Implemented planner + view → suite green | `cargo test` |
| T2.1/T2.3 (behavior pins) | First green run FAILED 2 tests: (a) `construction_loads_champion…` compared serde round trip with bit-exact weights — f64 JSON round trips differ at the last ulp (slice-1 lesson re-learned); (b) `bounded_ticks…` asserted `champion().is_none()` after 250 random ticks, but a random agent DID score (score 1 → record → champion set + saved to the real `dqn_champion.json`, racing the round-trip test on that shared file) | Fixes: serde comparisons use `nets_approx_eq` (1e-9); tests bind the champion file to private per-test paths via `DqnTrainView::at_path` (temp files removed after each); dropped the false no-record assumption (assert episode ≥ 1 + board reset only). 27/27 green | `cargo test` |
| T2.1/T4.1 (triangulate) | Random-brain match/finish tests (bounded 10 000-tick budget) and the record/epsilon behavior tests could flake on OS-entropy brains | Ran full suite 5× consecutively → 27/27 every run; no flake | `cargo test` ×5 |

Final suite: `cargo test` → 27 passed / 0 failed (lib: 5 champion_store + 3 viz_vs + 7 versus + 6 view_dqn_train + 6 view_dqn_versus; both bins 0 tests). `cargo build` clean, 0 warnings. Pure seams (`on_episode_end`, `plan_dqn_versus`) are macroquad-free; `Net` cloning only.

## Deviations from design

- Naming/state shape: T2.1's "Esc → pause (kept alive)" and "R" are not handled *inside* the view. Per the slice instruction ("Esc handled by the shell later — expose a paused/resumable struct") `DqnTrainView` is a passive resource: pause = the App stops calling `tick()` and keeps the struct (AD-3); `fresh_agent()` is the public seam the shell's `R` handler will call in slice 5. No `paused` bool field — the App's ownership already encodes pause.
- Champion file injection: `DqnTrainView::new()` hardcodes `dqn_champion.json` exactly as specced; a private `at_path(&'static str)` constructor exists purely so tests never read/write the real champion file (they use per-test temp files). No production behavior change.
- `DqnVersusPlayers` carries owned `Net` clones (deterministic planner) and `ChampionVsFresh { champion }` marks the fresh-fallback branch; the actual fallback net (`DQNAgent::new().q_network`) is allocated by `DqnVersusView::new` after planning, keeping the pure seam free of randomness.
- Extra accessor `live_is_fresh()` exposes the "CURRENT (fresh)" vs "CURRENT" labeling decision for tests and future shell HUD use.

## Remaining tasks (out of this slice — exact unchecked lines)

Slice 3 is complete; remaining implementation tasks belong to slices 4–5 and stay unchecked:

- [ ] T3.1 `src/view_ga_train.rs` `GaTrainView` around `Simulation` seam: pacing (slow batch/frame + sleep, fast ≤50/frame), keys keep meaning; DQN-style single-grid + HUD (generation, gen max, best ever, elapsed, champ score/fitness/steps) default; `Tab` toggles advanced VizAdvanced dashboard; routes sim internal VS sub-state to the versus renderer (GA flavor).
- [ ] T3.2 `src/pop.rs`: make best-net loader `pub` (no behavior change).
- [ ] T4.2 `src/view_ga_versus.rs`: GA champions from `sim_metadata.json` best/second-best (fallback `best_snake.json`), GA-default flavor incl. record; both missing → message.
- [ ] T4.3 `src/view_cross_match.rs`: load `best_snake.json` + `dqn_champion.json`; flavor "GA"/"DQN", `record: None`; per-side missing-file messages (spec scenario).
- [ ] T5.1–T5.4 (unchanged; see tasks.md).

## Workload / PR boundary

Slice 3 is a clean, independently compilable/testable unit: ~700 added lines across 2 new source files + 2-line `lib.rs` registration + docs churn; both bins still compile unchanged (main_dqn.rs untouched until slice 5). Next PR slice boundary candidates: GA train + GA/cross versus views (T3.1–T3.2, T4.2–T4.3) then the shell/wiring (T5.1–T5.4).

## Structured status consumed

Authoritative native status (change `unified-snake-shell`): `applyState: ready`, `artifactStore: openspec`, `actionContext.mode: repo-local`, allowed edit roots `[<repo-root>]`, no warnings. All edited paths inside the authoritative workspace and the slice's allowed edit surfaces (`src/view_dqn_train.rs`, `src/view_dqn_versus.rs`, `src/lib.rs`, `tasks.md`, `apply-progress.md`). Note: the pi-lens LSP watcher repeatedly reported a stale "file not found" for `view_dqn_versus.rs` this turn although the file exists on disk and compiles — cargo (authoritative) is green 5×.

---



## Slice 2 — Versus arena (completed)

## Completed tasks (persisted checkboxes updated)

| Task | Summary | Checkbox |
| --- | --- | --- |
| T1.2 | `src/versus.rs` (new): `pub enum Winner { Left, Right, Tie }`; `resolve_winner(s1, s2) -> Winner` (pure score comparison, ties → Tie); `run_headless_match(&Net, &Net, max_ticks) -> Option<Winner>` macroquad-free lockstep arena (two `Game::with_brain` stepped one `update()` each per tick until both `is_complete` or budget exhausted). | `[x]` in tasks.md |
| T1.3 | Same file: `VersusMatch { game1: Game, game2: Game, flavor: VsFlavor }` with `new(Net, Net, VsFlavor)`, `tick()` (one step per player, no-op when finished), `is_finished()`, `winner() -> Option<Winner>` (scores via `resolve_winner`), `draw()` delegating to `VizVS::draw_flavored` (winner banner + `back_label` overlay already handled by the renderer). No input handling (shell owns Esc, slice 5). | `[x]` in tasks.md |

## Files changed (slice 2)

- `src/versus.rs` — new (pure core + renderable match + 7 unit tests)
- `src/lib.rs` — registered `pub mod versus;`
- `openspec/changes/unified-snake-shell/tasks.md` — checked T1.2, T1.3
- `openspec/changes/unified-snake-shell/apply-progress.md` — this file (merged)

Not modified in this slice (as required): `src/viz_vs.rs`, `src/game.rs`, `src/nn.rs`, `src/sim.rs`, `Cargo.toml`, `src/main.rs`, `src/main_dqn.rs`, any DQN/GA sources. No pixel tests.

## TDD Cycle Evidence

Runner: `cargo test`. Baseline: `cargo test` → 8 passed (slice 1), both bins 0.

| Task | RED | GREEN | Command |
| --- | --- | --- | --- |
| T1.2 | Wrote 6 tests in `versus.rs` referencing undefined `Winner`/`resolve_winner`/`run_headless_match`/`VsFlavor`/`Color`/`VersusMatch` → compile failed (28 errors: E0422 x2, E0425 x11, E0433 x15) | Implemented `Winner`, `resolve_winner`, `run_headless_match`, `VersusMatch` → `cargo test versus` 7/7 pass | `cargo test versus` |
| T1.2 (triangulate) | N/A — random-termination tests (20 trials of two `Net::new()` brains, max 10 000 ticks) could flake if any random match outlives the budget | Ran the 7-test set 5× consecutively → 7/7 pass every run (each run draws fresh OS-entropy brains); no flake observed | `cargo test versus` ×5 |
| T1.3 | Behavior pin tests (finish is sticky across post-finish ticks; winner stable after finish) written with the T1.2 RED batch | Same GREEN as T1.2 (7 tests cover both tasks) | `cargo test versus` |

Final suite: `cargo test` → 15 passed / 0 failed (lib: 5 champion_store + 3 viz_vs + 7 versus; both bins 0 tests). `cargo build` clean. Pure core (`Winner`/`resolve_winner`/`run_headless_match`) makes no macroquad import or call; macroquad `Color` appears only inside `#[cfg(test)]` flavor literals (plain struct construction, no window calls).

## Deviations from design (slice 2)

- Naming: the slice executor's authoritative API is `VersusMatch { new/tick/is_finished/winner/draw }` + a free `run_headless_match` runner, whereas tasks.md/design.md sketched `MatchPlayers` + `run_match_tick`. Behavioral intent is identical (two `Net`s + `VsFlavor`, one step per player per tick, resolve winner from scores); executor-owned names win. If verify prefers `MatchPlayers`/`run_match_tick` aliases, that is a one-line follow-up.
- "Esc handling" on the T1.3 checkbox is deferred by explicit executor instruction: "No input handling here (shell owns Esc)". `Esc`/back semantics belong to `AppMode` dispatch in slice 5 (T5.1); VersusMatch is deliberately input-free so all three flavor views can compose it.
- `run_headless_match` returns `Option<Winner>` (`None` = budget exhausted before both games complete) rather than panicking on a pathological stalemate; the RED tests pin termination with a generous 10 000-tick budget across 20 random trials.
- VersusMatch fields (`game1`/`game2`/`flavor`) are private as specced; tests exercise behavior via public methods only (no pixel tests, per design).

## Remaining tasks (out of this slice — exact unchecked lines)

Slice 2 is complete; remaining implementation tasks belong to slices 3–5 and stay unchecked:

- [ ] T2.1 Move DQN train loop out of `src/main_dqn.rs` into `src/view_dqn_train.rs` (`DqnTrainView`): per-frame `GameDQN::step()`, episode/best bookkeeping, HUD (Episode/Score/Best/Epsilon) + grid centered via `screen_width()`; `Esc` → pause (kept alive); `R` → fresh agent.
- [ ] T2.2 Episode-end record logic as pure fn: `on_episode_end(score, best, q_network) -> (best, Option<Net>)`. **[RED-first]** test: snapshot replaced exactly on record, not on tie/lower.
- [ ] T2.3 Wire champion persistence: on record, `champion_store::save("dqn_champion.json", champion)`; on view construction, load existing champion (corrupt/missing → none).
- [ ] T3.1, T3.2, T4.1–T4.3, T5.1–T5.4 (unchanged; see tasks.md).

## Workload / PR boundary

Slice 2 is a clean, independently compilable/testable unit: ~300 added lines in `src/versus.rs` + 1-line `lib.rs` registration + docs churn. Slice 1 + 2 remain under the 400-line review budget combined. Next PR slice boundary candidate: DQN trainer view (T2.1–T2.3).

## Structured status consumed

Authoritative native status (change `unified-snake-shell`): `applyState: ready`, `artifactStore: openspec`, `actionContext.mode: repo-local`, allowed edit roots `[<repo-root>]`, no warnings. All edited paths inside the authoritative workspace and slice's allowed edit surfaces.

---

## Slice 1 record (preserved verbatim from the slice-1 apply run)

Slice: pure foundations (T0.1, T0.2, T0.3, T1.1). Delivery: chained slices on local branch `feature/dqn-vs-genetico` (no remote/PR). Not committed.

## Completed tasks (persisted checkboxes updated)

| Task | Summary | Checkbox |
| --- | --- | --- |
| T0.1 | `src/champion_store.rs` (new): pure `encode(&Net) -> String` / `decode(&str) -> Result<Net, String>` + thin fs `save(path, &Net) -> io::Result<()>` / `load(path) -> Option<Net>`; missing/corrupt → `None`, never panics. Registered `pub mod champion_store;` in `src/lib.rs`. | `[x]` in tasks.md |
| T0.2 | `dqn_champion.json` added to `.gitignore` beside `best_snake.json` / `sim_metadata.json`. | `[x]` in tasks.md |
| T0.3 | `src/sim.rs` seam: `update()` now delegates to extracted `tick_training()` (pop batch + generation lifecycle + auto-VS every-100 cadence) and `tick_vs()` (vs stepping + auto-return to Training); added `mode() -> SimMode`, `vs_state() -> Option<(&Game, &Game)>`, `snapshot() -> SimSnapshot` (gen_count, gen_max, best_ever, elapsed_secs, champ_score/fitness/steps, best_game). Legacy `update(is_viz, slow)` behavior preserved (advanced toggle still renders VizAdvanced; VS final winner frame still drawn before auto-return). | `[x]` in tasks.md |
| T1.1 | `src/viz_vs.rs`: added `VsFlavor` (player titles, colors, `record: Option<usize>`, record/winner/tie/eliminated/back/controls labels); `VsFlavor::ga_default(record)` reproduces the legacy strings byte-for-byte; existing `draw(g1, g2, max_score_ever)` routes through `draw_flavored` + GA-default so current GA VS output is unchanged. | `[x]` in tasks.md |

## Files changed

- `src/champion_store.rs` — new (pure serde encode/decode + fs save/load + 5 unit tests)
- `src/lib.rs` — registered `pub mod champion_store;`
- `src/sim.rs` — seam refactor + accessors + `SimSnapshot` (behavior-preserving)
- `src/viz_vs.rs` — `VsFlavor` + `draw_flavored`; `draw()` routes through GA default; 3 unit tests
- `.gitignore` — added `dqn_champion.json`
- `openspec/changes/unified-snake-shell/tasks.md` — checked T0.1, T0.2, T0.3, T1.1
- `openspec/changes/unified-snake-shell/apply-progress.md` — this file

Not modified in this slice (as required): `src/game_dqn.rs`, `src/dqn.rs`, `src/pop.rs`, `src/game.rs`, `Cargo.toml`, `src/main.rs`, `src/main_dqn.rs`. No pixel tests.

## TDD Cycle Evidence

Runner: `cargo test`. Baseline before slice: `cargo test` → 0 tests, all green.

| Task | RED | GREEN | Command |
| --- | --- | --- | --- |
| T0.1 | Wrote 5 tests in `champion_store.rs` referencing undefined `encode`/`decode`/`save`/`load` → compile failed E0425 (12 errors) | Implemented module → `cargo test champion_store` 5/5 pass | `cargo test champion_store` |
| T0.1 (triangulate) | Round-trip tests failed: comparing re-encoded JSON is not bit-exact (serde_json f64 decimal round trip differs at last ulp, e.g. `...842275` vs `...8422756`) | Test now asserts structural + weight-level equality within 1e-9 (`nets_approx_eq`) → 5/5 pass | `cargo test champion_store` |
| T1.1 | Added 3 tests in `viz_vs.rs` referencing undefined `VsFlavor` → compile failed E0422/E0433 (4 errors) | Implemented `VsFlavor`/`ga_default`/`draw_flavored` → `cargo test viz_vs` 3/3 pass | `cargo test viz_vs` |
| T0.3 | N/A (behavior-preserving refactor of existing code; no new behavior to RED-first) | Seam extracted + accessors added | `cargo build` + `cargo test` green |

Final suite: `cargo test` → 8 passed / 0 failed (lib: 5 champion_store + 3 viz_vs; both bins 0 tests). `cargo build` clean.

## Deviations from design

- `vs_state()` returns `Option<(&Game, &Game)>` instead of the design's bare `(&Game, &Game)`: a bare tuple would force a panic or dummy when the sim is in Training (games `None`). Callers check `mode()` first and handle `None`; documented in the accessor doc comment.
- Extracted code is behavior-identical, not character-verbatim: pi-lens clippy auto-fix had already rewritten `% 100 == 0` / `% 10 == 0` to `is_multiple_of(...)` (present on disk before this slice), and one flagged `games_alive <= 0` (usize) became `games_alive == 0` — equivalent for `usize` (can never be negative). No semantic difference.
- `VsFlavor` adds `eliminated1_label`/`eliminated2_label`/`record_beat_label`/`new_record_label`/`controls_label` beyond the design's listed fields so the center panel (record section, NEW! tags, eliminated messages, bottom controls) is fully flavor-driven; GA default strings unchanged.
- `decode` error type is `Result<Net, String>` (readable message) rather than an unspecified `Result<Net, _>`.

## Remaining tasks (out of this slice — exact unchecked lines)

Slice 1 is complete; remaining implementation tasks belong to slices 2–5 and stay unchecked:

- [ ] T1.2 `src/versus.rs` core (no macroquad): `MatchPlayers`, `run_match_tick`, `resolve_winner(s1, s2) -> Winner`. **[RED-first]** tests: Left/Right/Tie; headless arena of two random-nets `Game`s terminates within a bounded budget (existing no-food step limits guarantee it) and yields a winner.
- [ ] T1.3 Versus view renderer on `VizVS::draw_flavored` + winner overlay + Esc handling (composable for all three flavors).
- [ ] T2.1–T2.3, T3.1–T3.2, T4.1–T4.3, T5.1–T5.4 (unchanged; see tasks.md).

## Workload / PR boundary

Slice 1 is a clean, independently compilable/testable unit: ~430 added lines across 2 new/2 changed source files + config/docs churn, within review budget; the remaining ~800–1000 lines land in later slices. Next PR slice boundary candidate: versus arena + views (T1.2, T1.3).

## Structured status consumed

Authoritative native status (change `unified-snake-shell`): `applyState: ready`, `artifactStore: openspec`, `actionContext.mode: repo-local`, allowed edit roots `[<repo-root>]`, no warnings. All edited paths inside the authoritative workspace and slice's allowed edit surfaces.

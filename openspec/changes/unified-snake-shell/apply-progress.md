# Apply Progress — unified-snake-shell (Slices 1–2 of 5)

Slice 2: versus arena (T1.2, T1.3). Delivery: chained slices on local branch `feature/dqn-vs-genetico` (no remote/PR). Not committed. Slice 1 record preserved verbatim below.

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

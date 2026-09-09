# Apply Progress — unified-snake-shell (Slice 1 of 5)

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

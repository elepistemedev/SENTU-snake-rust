# Tasks — unified-snake-shell

STRICT TDD ACTIVE: test runner `cargo test`. Tasks marked **[RED-first]** start with a failing unit test on the pure-logic seam, then implement, then go green.

## Phase 0 — Pure foundations (RED-first)

- [x] T0.1 `src/champion_store.rs`: pure `encode(net) -> String` / `decode(s) -> Result<Net>` + thin fs `save(path, net)` / `load(path) -> Option<Net>`; corrupt/missing degrades to `None`, never panics. **[RED-first]** tests: round trip, corrupt input.
- [x] T0.2 Add `dqn_champion.json` to `.gitignore`.
- [x] T0.3 `src/sim.rs` seam: extract `tick_training()` / `tick_vs()` logic halves of `update()` verbatim; add `mode()`, `vs_state()`, `snapshot() -> SimSnapshot` accessors; keep `update()` behavior unchanged (advanced toggle still renders VizAdvanced). Evidence: `cargo build` + `cargo test`.

## Phase 1 — Versus arena

- [x] T1.1 `src/viz_vs.rs`: add `VsFlavor` (titles, colors, `record: Option<usize>`, winner/tie/back labels) with a GA-default whose output equals today's strings byte-for-byte; route existing `draw` through the default flavor.
- [x] T1.2 `src/versus.rs` core (no macroquad): `MatchPlayers`, `run_match_tick`, `resolve_winner(s1, s2) -> Winner`. **[RED-first]** tests: Left/Right/Tie; headless arena of two random-nets `Game`s terminates within a bounded budget (existing no-food step limits guarantee it) and yields a winner.
- [x] T1.3 Versus view renderer on `VizVS::draw_flavored` + winner overlay + Esc handling (composable for all three flavors).

## Phase 2 — DQN trainer + champion

- [x] T2.1 Move DQN train loop out of `src/main_dqn.rs` into `src/view_dqn_train.rs` (`DqnTrainView`): per-frame `GameDQN::step()`, episode/best bookkeeping, HUD (Episode/Score/Best/Epsilon) + grid centered via `screen_width()`; `Esc` → pause (kept alive); `R` → fresh agent.
- [x] T2.2 Episode-end record logic as pure fn: `on_episode_end(score, best, q_network) -> (best, Option<Net>)`. **[RED-first]** test: snapshot replaced exactly on record, not on tie/lower.
- [x] T2.3 Wire champion persistence: on record, `champion_store::save("dqn_champion.json", champion)`; on view construction, load existing champion (corrupt/missing → none).

## Phase 3 — GA trainer view (DQN-style)

- [x] T3.1 `src/view_ga_train.rs` `GaTrainView` around `Simulation` seam: pacing (slow batch/frame + sleep, fast ≤50/frame), keys keep meaning; DQN-style single-grid + HUD (generation, gen max, best ever, elapsed, champ score/fitness/steps) default; `Tab` toggles advanced VizAdvanced dashboard; routes sim internal VS sub-state to the versus renderer (GA flavor).
- [x] T3.2 `src/pop.rs`: make best-net loader `pub` (no behavior change).

## Phase 4 — Versus views (DQN internal / GA internal / cross)

- [x] T4.1 `src/view_dqn_versus.rs`: champion Net (memory/file) vs live policy Net (paused trainer q_network, else fresh greedy agent), flavor "CHAMPION"/"CURRENT", `record: None`; missing champion → message + Esc. **[RED-first]** pure part: which Nets are selected given trainer/champion presence.
- [x] T4.2 `src/view_ga_versus.rs`: GA champions from `sim_metadata.json` best/second-best (fallback `best_snake.json`), GA-default flavor incl. record; both missing → message.
- [x] T4.3 `src/view_cross_match.rs`: load `best_snake.json` + `dqn_champion.json`; flavor "GA"/"DQN", `record: None`; per-side missing-file messages (spec scenario).

## Phase 5 — Menu shell + wiring

- [ ] T5.1 `src/app.rs`: `AppMode`, `App` (paused DQN trainer + GA trainer holders, transient match state), menu rendering/input (numbers/arrows+Enter/Esc); pure `next_mode(...)` transition table. **[RED-first]** tests: fresh-match vs resume, paused-trainer cases, Esc semantics.
- [ ] T5.2 Replace `src/main.rs` entry with the shell loop; delete `src/main_dqn.rs`; remove `snake-dqn` bin from `Cargo.toml`; register new modules in `src/lib.rs`.
- [ ] T5.3 Docs: update `README.md`/`DQN_README.md` usage to the unified app + menu keys; note `dqn_champion.json`.
- [ ] T5.4 Full `cargo build` + `cargo test` green; manual smoke of each menu → view → Esc path (no pixel tests, per design).

## Review Workload Forecast

- Estimated changed lines: **~1200–1400** (added across ~8 new view/module files ≈ 900–1000; plus sim seam ~90, viz_vs flavor ~60, app/main rewrites, `main_dqn.rs` deletion ~132, Cargo/lib/gitignore/doc churn). All well above the 400-line review budget.
- Chained PRs recommended: **Yes** — natural slices exist (foundation → versus arena+views → trainer views+shell+wiring), each compilable/testable.
- Decision needed before apply: **Yes** (`delivery_strategy` = ask-on-risk): choose single MVP PR with `size:exception` vs chained slices; if chained, `chain_strategy` (stacked-to-main vs feature-branch-chain) is needed.

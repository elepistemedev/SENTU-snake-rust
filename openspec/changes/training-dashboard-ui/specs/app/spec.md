# Delta for App

Change `training-dashboard-ui`: both training views render the advanced `VizAdvanced`-grammar dashboard by default, DQN train gains a new live-data dashboard mirror, and the shell wires `Tab`/hint handling without perturbing training, evolution, versus routing, or the verified GA renderers. Drawing itself is untested by design in this repo; every requirement below is testable at the logic level (render-target decisions, toggle state, accessor behavior, ring bookkeeping, normalization functions) except where a scenario is explicitly a visual smoke check.

Domain-coverage note: the visibility seams on `game_dqn.rs` (observation accessor) and `dqn.rs` (hyperparameter constants) are specified here under the `app` domain because no per-module canonical specs exist; they are scoped to visibility-only changes and carry no learning/behavioral change.

## ADDED Requirements

### Requirement: GA train view with advanced dashboard default

The GA train view MUST default to the pre-existing advanced `VizAdvanced` dashboard — rendered through the unmodified `Simulation::draw_advanced` path, byte-identical to the original GA app — and MUST keep the compact DQN-style HUD (generation, generation max score, best-ever score, elapsed seconds, current champion score/fitness/steps, plus the best snake's grid) reachable as a `Tab`-toggled alternative that toggles back and forth without pausing or resetting the simulation. Genetic-algorithm training (population/streams, generation lifecycle, persistence of `sim_metadata.json` and `best_snake.json`) MUST behave exactly as today regardless of which render target is active. A fresh `GaTrainView` construction MUST select the advanced dashboard; a resumed (paused-then-reentered) view MUST keep whichever target was active when it paused.

#### Scenario: fresh GA train entry opens on the advanced dashboard and Tab round-trips

- Given a fresh `GaTrainView` is constructed while the sim is in `Training` mode
- When its render target is queried
- Then the advanced dashboard is selected by default
- And each `Tab` toggle flips the target between the advanced dashboard and the compact DQN-style HUD
- And a double toggle returns to the advanced dashboard
- And neither toggle advances, resets, or otherwise perturbs the simulation

#### Scenario: compact HUD shows the DQN-style text rows when Tabbed to

- Given the GA train view is running with the compact target active
- When a generation completes and the next one starts
- Then the HUD shows generation, generation max, best-ever, elapsed seconds, and current champion metrics
- And every tenth generation still persists `best_snake.json` and `sim_metadata.json` exactly as today

### Requirement: DQN live-network diagram and hyperparameter data seams

The DQN dashboard center panel MUST be fed by the live training state each frame, not by stale snapshots or literals: the network diagram MUST be computed from the current 12-input observation of the live `GameDQN` passed through the live `q_network`'s `predict`, and the model-info panel rows MUST reference the same hyperparameter constants the agent was constructed with. `GameDQN` MUST expose the current observation through a public read-only accessor whose output is identical to the vector `step()` feeds to action selection, and this MUST be a visibility-only change: values, ordering, bounds, and training behavior remain exactly as today. The DQN hyperparameter constants in `src/dqn.rs` (replay-buffer capacity, batch size, gamma, learning rate, epsilon schedule) MUST be `pub` so the model-info panel displays real values; their numeric values MUST NOT change, and no learning/episode/champion logic MAY be altered to expose them.

#### Scenario: observation accessor returns the live 12-input state without side effects

- Given a live `GameDQN` mid-episode
- When the public observation accessor is invoked twice without an intervening step
- Then both invocations return identical vectors
- And each vector has exactly 12 elements in the documented order (four directions × wall-distance reciprocal, food bit, body-distance reciprocal), with wall components in (0,1], body components in [0,1], and food components being 0.0 or 1.0
- And invoking the accessor does not advance steps, mutate the board, or change any training state

#### Scenario: visibility-only seams leave DQN behavior pinned

- Given the DQN hyperparameter constants are made `pub` and `GameDQN::get_state` becomes public
- When the existing DQN behavior tests run (replay-batch training guard, epsilon decay bounds, episode bookkeeping, champion snapshot rules)
- Then all existing tests stay green
- And the constants still hold their pre-change values (capacity 10000, batch 32, gamma 0.99, learning rate 0.001, epsilon start 1.0, end 0.01, decay 0.995)

#### Scenario: output-node color input is the sigmoid output layer of the live prediction

- Given the live q-network predicts on the current observation
- When the color-driving intensity for each of the four output nodes is derived
- Then the value is the corresponding element of the final (4-node) sigmoid output layer, already in (0,1)
- And the clamping used before mapping to color is pure: values below 0 clamp to 0, values above 1 clamp to 1, and interior values pass through unchanged

### Requirement: DQN per-episode history bookkeeping

The DQN train view MUST record one history entry per completed episode — the episode's score and its duration — appended exactly once in the same end-of-episode bookkeeping path that advances the episode counter and evaluates the champion record, before the board resets. The history MUST be a bounded FIFO ring of at most 50 entries (mirroring `VizAdvanced`'s 50-entry cap); overflow MUST drop the oldest entry and keep the newest. The duration recorded MUST be the completed episode's length in game steps taken from reset to completion (deterministic and unit-testable). The ring MUST survive menu pause/resume as part of the session state and MUST be emptied only by starting a fresh agent (`fresh_agent`), which resets it together with the episode counter and session best; the recorded entries MUST feed the dashboard's per-episode charts.

#### Scenario: exactly one entry per completed episode, in completion order

- Given a DQN train view stepping episodes
- When an episode completes and the board resets
- Then the ring gains exactly one entry holding that episode's final score and step count at completion
- And entries appear in completion order with the most recent episode last

#### Scenario: ring caps at 50 and drops the oldest

- Given a ring holding 50 entries after many completed episodes
- When one more episode completes
- Then the ring holds 50 entries, the newest episode is present, and the oldest entry is gone

#### Scenario: fresh agent clears the history

- Given a DQN train view whose ring holds recorded episodes
- When `fresh_agent` is invoked
- Then the ring is empty
- And the active render target is unchanged (a fresh agent resets learning session state, not the display choice)

### Requirement: DQN bar and chart normalization denominators

Every DQN dashboard bar and chart MUST normalize against a documented DQN-appropriate denominator; the GA reference's hardcoded `/20` caps MUST NOT be copied for DQN panels. The episode-score and session-best score bars MUST normalize against a documented DQN episode bound — the maximum food a single episode can collect, which is bounded by the episode step cap (`NUM_SIM_STEPS × 2` = 200 steps, since each food eaten requires at least one step) — clamped at 1.0. Per-episode history chart bars MUST use the reference's data-relative normalization (each bar proportional to the current series maximum, with an empty or all-zero series drawing no division by zero and no bars). Normalization functions MUST be pure and MUST clamp to [0,1].

#### Scenario: score-bar fractions are clamped against the documented episode bound

- Given a pure DQN score-bar fraction function over the documented episode bound (200)
- When scores 0, 200, and 250 are normalized
- Then 0 maps to 0.0, 200 maps to 1.0, and 250 clamps to 1.0
- And no DQN bar or chart code path uses the GA `/20` denominator

#### Scenario: empty history charts render safely

- Given a fresh DQN view whose history ring is empty
- When the history charts are drawn
- Then no division by zero occurs and no bars are drawn

### Requirement: Shell Tab routing and hint overlay for the DQN dashboard

The app shell MUST forward a `Tab` key press in DQN train mode to the DQN train view's render-target toggle, mirroring the existing GA `Tab` forwarding, and MUST keep `R` (fresh agent) and `Esc` (return to menu) working from either render target. `Esc` handling MUST keep routing through the existing pure transition table unchanged (any view + `Esc` → menu, menu `Esc` → quit). The shell's bottom-left hotkey hint MUST NOT overlap the DQN dashboard's bottom model-info panel: the hint MUST be drawn only while the compact HUD is the active target (or, alternatively, be replaced by the dashboard's own controls line inside the model-info panel, mirroring the reference); it MUST NOT be overlaid when the dashboard target is active.

#### Scenario: Tab toggles the DQN render target and Esc still returns to the menu

- Given the DQN train view with the dashboard target active
- When a `Tab` key press is forwarded to the view
- Then the active target flips to the compact HUD
- And a second forwarded `Tab` flips back to the dashboard
- And pressing `Esc` from either target routes through the unchanged transition table to `Transition::ToMenu` without quitting and without destroying the paused trainer

#### Scenario: hint overlay is gated off the dashboard

- Given the DQN train view whose pure render-target decision selects the dashboard
- When the shell composes the frame
- Then no shell-drawn hint text is placed over the bottom-left model-info panel zone
- And when the compact HUD is the active target the hint remains visible in its current bottom-left position

### Requirement: DQN dashboard visual grammar parity

The DQN advanced dashboard MUST present the same visual grammar family as `VizAdvanced` so it reads as "same dashboard, DQN data": a left column with the live single-snake `GameDQN` grid (large) and a bottom model-info panel showing DQN hyperparameters, replay-buffer fill, and architecture; a center "NEURAL NETWORK" panel with input nodes labeled I0..I11, hidden nodes H0..H7, and four output nodes labeled LEFT/RIGHT/BOTTOM/TOP colored by the live q-network's output activations; and a right column of DQN stats panels (Episode, Score, Best, Epsilon), score bars, and per-episode history bar charts. The mirror MUST reuse the reference's panel color constants and geometry relationships (dynamic sizing from `screen_width()`/`screen_height()`, no hardcoded 800×600 offsets) and MUST show a single live snake — DQN has no top-N ghost overlay. Output-node color mapping transfers unchanged because both nets share the sigmoid output domain. This requirement is satisfied by grammar-level code review plus the acceptance smoke; it is not unit-testable as pixels.

#### Scenario: smoke — DQN dashboard reads as the reference grammar with DQN data

- Given the app running at the default window size
- When DQN train is entered (dashboard default) and GA train is entered and `Tab`bed to its untouched advanced dashboard
- Then the two dashboards present the same panel/column/color grammar and neither has overlapping or off-screen panels
- And the user confirms the DQN dashboard against the reference screenshot with DQN data substituted (single live grid, DQN stats, live network, DQN history charts)

## MODIFIED Requirements

### Requirement: DQN train view

The DQN train view MUST run the existing DQN training behavior: one `GameDQN` stepping episodes, storing experiences, training on replay batches, decaying epsilon, and updating the target network. On entry (fresh construction or a fresh agent) the view MUST render the advanced dashboard — the `VizAdvanced`-grammar mirror with the live grid, model-info panel, live network panel, stats panels, score bars, and per-episode history charts — and the view MUST keep the legacy compact HUD (Episode/Score/Best/Epsilon text plus the snake grid) reachable as a `Tab`-toggled alternative that toggles back and forth without pausing, resetting, or otherwise perturbing the in-flight episode or session. Whenever an episode ends with a new session-best score, the view MUST snapshot the current q-network as the DQN champion (epsilon-zero policy source) and report the record; episode bookkeeping, champion replacement/persistence, and the one-`GameDQN::step()`-per-frame cadence MUST be identical regardless of which render target is active.
(Previously: the view rendered the DQN-style HUD (Episode, Score, Best, Epsilon) plus the snake grid unconditionally as its only display.)

#### Scenario: DQN train runs and tracks best score

- Given the user enters the DQN train view
- When an episode ends with score greater than the current session best
- Then the session best is updated
- And the DQN champion snapshot is replaced by the current q-network
- And the champion snapshot is persisted to `dqn_champion.json`
- And this holds whether the dashboard or the compact HUD is the active target

#### Scenario: dashboard is the default target and Tab round-trips without perturbing training

- Given a fresh DQN train view is constructed
- When its render-target decision is queried
- Then the advanced dashboard is selected by default
- And each `Tab` toggle flips between the dashboard and the compact HUD, returning to the dashboard after two toggles
- And ticking the view while toggling never changes the active target and never resets the in-flight episode, session best, or champion

#### Scenario: replay training guard

- Given the DQN replay buffer holds fewer than the batch size of experiences
- When the agent attempts to train
- Then no weight update occurs and the buffer keeps accumulating

### Requirement: No regression of GA evolution behavior

The shell MUST preserve the existing GA evolution semantics: population update batching, generation lifecycle, best/second-best ranking, auto-VS cadence hooks, and file persistence must behave as they do today whenever GA training runs inside the shell. Existing GA hotkeys that are reused in the GA views (slow/fast, versus trigger semantics) MUST keep their meaning. GA training MUST keep driving the unmodified `Simulation`/evolution pipeline, and the verified GA dashboard renderers MUST remain untouched by this change: `viz_advanced.rs`, `sim.rs`, `game.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `viz_vs.rs`, and the GA versus/cross views MUST show zero diffs. The pure render-path rule for the sim's internal VS sub-state MUST remain unchanged: a running internal VS match always routes to the versus renderer regardless of the dashboard toggle, and the advanced GA dashboard MUST still render through the unmodified `Simulation::draw_advanced` path.
(Previously: the guard named evolution semantics and reused hotkeys; the renderer-file preservation and versus-precedence rules are now explicit because the advanced dashboard became the GA default.)

#### Scenario: GA persistence cadence unchanged

- Given the GA train view has been running for N generations with N divisible by 10
- When generation N completes
- Then `best_snake.json` and `sim_metadata.json` are written exactly as the pre-change behavior writes them

#### Scenario: sim-internal VS still routes to the versus renderer under the advanced default

- Given the GA train view is running with the advanced dashboard as the default target
- When the sim enters its internal VS sub-state
- Then the frame routes to the side-by-side versus renderer and never to the advanced dashboard or the compact HUD
- And when the VS sub-state ends the sim returns to training on the advanced dashboard target

#### Scenario: verified GA renderer files are untouched by this change

- Given the `training-dashboard-ui` change is complete
- When the change diff is inspected for `viz_advanced.rs`, `sim.rs`, `game.rs`, `pop.rs`, `stream.rs`, `nn.rs`, `viz_vs.rs`, and the GA versus/cross views
- Then those files show zero diffs against the pre-change tree
- And `cargo build` is clean and `cargo test` is green

## REMOVED Requirements

### Requirement: GA train view with DQN-style layout

(Reason: superseded by "GA train view with advanced dashboard default" — the user decision flips the default: GA train opens on the pre-existing advanced `VizAdvanced` dashboard, and the DQN-style compact layout becomes the `Tab`-toggled alternative rather than the default.)
(Migration: the former requirement text, including the compact-HUD field list and generation/persistence semantics, is preserved inside the new "GA train view with advanced dashboard default" requirement; tests, doc comments, and README references that assert the compact layout is the GA default must be updated to assert the advanced-dashboard default with `Tab` reaching the compact HUD. No behavior, data, or persistence migration.)

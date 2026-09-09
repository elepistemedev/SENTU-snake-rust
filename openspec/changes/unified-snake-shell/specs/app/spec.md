# App Shell — Delta Spec

Change: `unified-snake-shell`

## ADDED Requirements

### Requirement: Unified single-binary app shell

The crate MUST ship exactly one runnable game binary (`snake`) that owns the whole app lifecycle and state machine: `Menu | DqnTrain | DqnVersus | GaTrain | GaVersus | DqnVsGa`. The previous `snake-dqn` binary entry MUST be removed; its training behavior MUST remain reachable inside the shell as the DQN train view. Each mode MUST own its state and MUST be constructible on entry and droppable on exit without affecting the other modes' state.

#### Scenario: single binary with full mode coverage

- Given the crate is built with `cargo build`
- When the produced binaries are enumerated
- Then exactly one game binary `snake` exists
- And each of the six shell states (`Menu`, `DqnTrain`, `DqnVersus`, `GaTrain`, `GaVersus`, `DqnVsGa`) is reachable through the shell state machine

### Requirement: Welcome menu with keyboard navigation

On launch, the app MUST show a welcome menu listing the five playable views (DQN train, DQN versus, GA train, GA versus, DQN vs GA). The user MUST be able to select and enter a view with keyboard input (number key or arrows+Enter). From any view, pressing `Esc` MUST return to the menu without resetting the process; pressing `Esc` on the menu MUST quit the app. Entering the versus/cross-match views MUST construct a fresh match on every entry. The DQN train view MUST start a fresh agent on first entry and MUST keep that agent paused (not destroyed) while the menu is shown, so that the DQN internal-versus view has a live current policy to compare against; the menu MUST indicate that a paused DQN run is resumable.

#### Scenario: navigate menu to view and back

- Given the app is running and showing the menu
- When the user presses the key mapped to "DQN train"
- Then the app enters the DQN train view and starts stepping episodes
- When the user presses `Esc`
- Then the app returns to the menu
- And when the user presses `Esc` again
- Then the app exits

### Requirement: DQN train view

The DQN train view MUST run the existing DQN training behavior: one `GameDQN` stepping episodes, storing experiences, training on replay batches, decaying epsilon, and updating the target network. The view MUST display the DQN-style HUD (Episode, Score, Best, Epsilon) and the snake grid. Whenever an episode ends with a new session-best score, the view MUST snapshot the current q-network as the DQN champion (epsilon-zero policy source) and report the record.

#### Scenario: DQN train runs and tracks best score

- Given the user enters the DQN train view
- When an episode ends with score greater than the current session best
- Then the session best is updated
- And the DQN champion snapshot is replaced by the current q-network
- And the champion snapshot is persisted to `dqn_champion.json`

#### Scenario: replay training guard

- Given the DQN replay buffer holds fewer than the batch size of experiences
- When the agent attempts to train
- Then no weight update occurs and the buffer keeps accumulating

### Requirement: DQN internal versus view

The DQN versus view MUST pit the session DQN champion (q-network snapshot, epsilon zero) against the live current policy (epsilon zero) on a versus arena using the side-by-side `viz_vs` rendering. Both players MUST observe the same arena state sequence and each MUST move at most once per tick. When both games end, the view MUST show the winner as the player with the higher score, and MUST offer return-to-menu.

#### Scenario: champion beats or ties current policy

- Given a session DQN champion snapshot and a live policy exist
- When the DQN versus view runs a match to completion
- Then both players move alternately on the shared arena
- And a winner is resolved by higher score (ties resolved by a documented tie rule)
- And pressing `Esc` returns to the menu

#### Scenario: versus without champion yet

- Given no champion snapshot exists (fresh session, no record yet)
- When the user enters the DQN versus view
- Then the view MUST show a message that a champion is required (train first) and return to the menu on `Esc`

### Requirement: GA train view with DQN-style layout

The GA train view MUST drive the existing genetic-algorithm training (population/streams, generation lifecycle, persistence of `sim_metadata.json` and `best_snake.json`) with unchanged evolution behavior, while rendering with the DQN-style layout family: one snake grid plus a text HUD (generation, generation max score, best-ever score, elapsed time, current champion score/fitness/steps). The pre-existing `VizAdvanced` multi-panel dashboard MUST remain functional where still used and MUST NOT be modified by this change.

#### Scenario: GA train advances generations with DQN-style HUD

- Given the user enters the GA train view
- When a generation completes and the next one starts
- Then generation count increments and the population resets as today
- And the HUD shows generation, generation max, best-ever, elapsed seconds, and current champion metrics
- And every tenth generation still persists `best_snake.json` and `sim_metadata.json`

### Requirement: GA internal versus view

The GA versus view MUST expose the existing GA VS arena — best-ever net vs second-best-ever net (falling back to the current generation top when no second best exists) as two `Game` instances — rendered with the existing side-by-side `viz_vs` panel. On completion the winner MUST be resolved by higher score and return-to-menu offered.

#### Scenario: GA internal versus uses best vs second-best

- Given GA training has produced a best-ever net and a second-best-ever net
- When the user enters the GA versus view
- Then a match runs between the two nets on the shared arena
- And the winner is the player with the higher score when both games end

### Requirement: DQN vs GA cross-match view

The cross-match view MUST load the GA champion from `best_snake.json` and the DQN champion from `dqn_champion.json` when present, and run both as epsilon-zero policies on the shared versus arena rendered by the `viz_vs` panel. When both games end, the view MUST resolve the winner by higher score. If a required champion file is missing, the view MUST state which side has no champion and allow returning to the menu.

#### Scenario: cross-match with both champions present

- Given `best_snake.json` and `dqn_champion.json` exist
- When the user enters the DQN vs GA view
- Then both champions play on the shared arena with epsilon-zero policies
- And a winner is resolved by higher score when both games end

#### Scenario: cross-match with a missing champion

- Given `best_snake.json` exists but `dqn_champion.json` does not
- When the user enters the DQN vs GA view
- Then the view indicates the DQN champion is missing and does not crash
- And `Esc` returns to the menu

### Requirement: DQN champion persistence

The app MUST persist the DQN champion q-network to `dqn_champion.json` (same serde `Net` format as `best_snake.json`) whenever a new DQN session record occurs, MUST load it on startup when present, and MUST tolerate a missing or corrupt file (start with no champion). The file MUST be covered by `.gitignore`.

#### Scenario: champion round-trip survives relaunch

- Given a DQN champion was persisted during a previous session
- When the app starts and the DQN versus or cross-match view is entered
- Then the champion is loaded from `dqn_champion.json` and is available as a player

#### Scenario: corrupt champion file degrades gracefully

- Given `dqn_champion.json` exists but is not valid JSON for a `Net`
- When the app starts
- Then the app runs normally with no champion and does not panic

### Requirement: No regression of GA evolution behavior

The shell MUST preserve the existing GA evolution semantics: population update batching, generation lifecycle, best/second-best ranking, auto-VS cadence hooks, and file persistence must behave as they do today whenever GA training runs inside the shell. Existing GA hotkeys that are reused in the GA views (slow/fast, versus trigger semantics) MUST keep their meaning.

#### Scenario: GA persistence cadence unchanged

- Given the GA train view has been running for N generations with N divisible by 10
- When generation N completes
- Then `best_snake.json` and `sim_metadata.json` are written exactly as the pre-change behavior writes them

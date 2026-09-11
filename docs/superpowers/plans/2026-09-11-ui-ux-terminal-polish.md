# UI/UX Terminal Polish & Cohesive Cyber-Retro Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Overhaul and unify the entire UI/UX of Snake AI in Rust with a polished retro/cyber-terminal aesthetic, responsive multi-resolution layouts, and reusable drawing primitives.

**Architecture:** Create a central `src/ui_kit.rs` module providing atomic retro-terminal primitives (panels, badges, progress bars, responsive charts, centered text, notices). Refactor the Main Menu and Theme Config in `src/app.rs`, modernize the training dashboards in `src/dqn_dash.rs`, and resolve Series HUD collisions and layout alignment in `src/viz_vs.rs` and versus views.

**Tech Stack:** Rust 2021, `macroquad 0.4.5`, `serde`, `serde_json`.

**Spec:** `docs/superpowers/specs/2026-09-11-ui-ux-terminal-polish-design.md`

## Global Constraints
- Do not break existing headless navigation state machine tests in `src/app.rs`.
- All drawing code must adapt gracefully to different screen heights and widths using `screen_width()` and `screen_height()`.
- Theme colors selected in `theme_config.json` must continue to apply to snake and food rendering.
- Maintain fast frame rates: no expensive allocations inside per-frame drawing loops.

---

### Task 1: Shared UI Kit (`src/ui_kit.rs`)

**Files:**
- Create: `src/ui_kit.rs`
- Modify: `src/lib.rs`
- Test: unit tests within `src/ui_kit.rs`

**Interfaces:**
- Produces:
  - `pub const COLOR_BG: Color`
  - `pub const PANEL_BG: Color`
  - `pub const PANEL_BORDER: Color`
  - `pub const PANEL_BORDER_FOCUSED: Color`
  - `pub const ACCENT_CYAN: Color`
  - `pub const ACCENT_GOLD: Color`
  - `pub const ACCENT_GREEN: Color`
  - `pub const ACCENT_RED: Color`
  - `pub const TEXT_MUTED: Color`
  - `pub fn draw_terminal_box(x: f32, y: f32, w: f32, h: f32, title: &str, focused: bool)`
  - `pub fn draw_badge(text: &str, x: f32, y: f32, badge_color: Color)`
  - `pub fn draw_progress_bar(x: f32, y: f32, w: f32, h: f32, fraction: f32, label: &str, fill_color: Color)`
  - `pub fn draw_responsive_chart(x: f32, y: f32, w: f32, h: f32, title: &str, max_label: &str, data: &[f32], max_cap: usize, bar_color: Color)`
  - `pub fn draw_centered_text(text: &str, cx: f32, y: f32, font_size: f32, color: Color)`
  - `pub fn draw_missing_champion_notice(w: f32, h: f32, title: &str, detail: &str)`
  - Pure calculation helpers for testing: `calculate_chart_slot_width(total_w: f32, count: usize, max_cap: usize) -> f32`

- [ ] **Step 1: Write the failing tests in `src/ui_kit.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_slot_width_calculation() {
        // When data is small, slots should not stretch absurdly; when full, fill width
        let w = 500.0;
        let slot_w = calculate_chart_slot_width(w, 10, 50);
        assert!(slot_w > 0.0 && slot_w <= 10.0);

        let full_slot = calculate_chart_slot_width(w, 50, 50);
        assert_eq!(full_slot, 10.0);
    }

    #[test]
    fn test_progress_bar_clamp() {
        assert_eq!(clamp_fraction(-0.5), 0.0);
        assert_eq!(clamp_fraction(1.5), 1.0);
        assert_eq!(clamp_fraction(0.75), 0.75);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test ui_kit::tests`
Expected: FAIL (module or functions not found)

- [ ] **Step 3: Implement `src/ui_kit.rs` and export in `src/lib.rs`**

Implement the constants, calculation functions, and macroquad drawing primitives. Add `pub mod ui_kit;` to `src/lib.rs`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test ui_kit::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/ui_kit.rs src/lib.rs
git commit -m "feat(ui): add shared ui_kit with retro-terminal primitives and tests"
```

---

### Task 2: Main Menu & Theme Configuration Overhaul (`src/app.rs`)

**Files:**
- Modify: `src/app.rs`
- Test: `cargo test app::tests`

**Interfaces:**
- Consumes:
  - `crate::ui_kit::{draw_terminal_box, draw_badge, draw_centered_text, PANEL_BORDER_FOCUSED, ACCENT_CYAN, ACCENT_GOLD, ACCENT_GREEN, TEXT_MUTED}`
  - `crate::theme::{load_theme, save_theme, GameTheme}`
- Produces:
  - Enhanced `draw_menu(&self)` with retro terminal header box, dynamic badges (`[PAUSADO - EP. X]`, `[CHAMPION LISTO]`, etc.), interactive focused row indicator, and terminal status bar footer.
  - Enhanced `draw_theme_config(&self)` with card layout, focused highlight, color swatches, and animated live snake preview on the board.

- [ ] **Step 1: Verify existing navigation tests pass before touching code**

Run: `cargo test app::tests`
Expected: PASS

- [ ] **Step 2: Implement enhanced `draw_menu` in `src/app.rs`**

- Draw ASCII-framed / terminal box title `SNAKE AI` at top center.
- For each menu item 1..=6:
  - Draw key badge `[ 1 ]` .. `[ 6 ]`.
  - Draw item label.
  - Check state: if DQN paused -> badge `[PAUSADO - EP. X]`; if champion exists on disk -> badge `[CHAMPION LISTO]`; if GA paused -> badge `[PAUSADO - GEN. X]`; if versus has no champion -> badge `[REQUIERE CHAMPION]`.
  - When row is selected: draw focused container with `PANEL_BORDER_FOCUSED`, retro cursor `▶  ◀` and yellow/cyan highlighted text.
- Draw bottom status bar across the screen width: `[1-6] / [↑↓] + [ENTER] Iniciar   [ESC] Salir   |   TEMA: [Nombre]`.

- [ ] **Step 3: Implement enhanced `draw_theme_config` in `src/app.rs`**

- Render theme options in cards with `[ACTIVO]` badge, color swatches (head, body, apple), and description.
- Render right-hand live preview: animated snake moving in a loop or showing curved body, directional eyes, digestion bulge, and shiny apple.
- Footer with instructions: `[1-4] / [↑↓] Seleccionar   [ENTER] Guardar   [ESC] Cancelar`.

- [ ] **Step 4: Run tests to verify no regressions in app logic**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat(ui): overhaul main menu with live badges and enhance theme config screen"
```

---

### Task 3: Training Dashboards Overhaul (`src/dqn_dash.rs`)

**Files:**
- Modify: `src/dqn_dash.rs`
- Test: `cargo test dqn_dash::tests`

**Interfaces:**
- Consumes:
  - `crate::ui_kit::{draw_terminal_box, draw_progress_bar, draw_responsive_chart, ACCENT_CYAN, ACCENT_GOLD, TEXT_MUTED}`
- Produces:
  - Fully responsive vertical layout in right stats column (dynamic chart height calculation based on available screen space).
  - Clean progress bars for Score and Best with real percentage based on session record or step limit.
  - Polished neural network diagram: softer synapse lines, active node glow, argmax action ring and neon cyan action text.
  - Model info panel at bottom left formatted with retro key tags `[TAB] HUD   [R] Nuevo Agente   [ESC] Menú`.

- [ ] **Step 1: Run existing tests in `dqn_dash`**

Run: `cargo test dqn_dash::tests`
Expected: PASS

- [ ] **Step 2: Refactor right column vertical layout & charts**

- Calculate available height dynamically:
  ```rust
  let stats_h = 160.0;
  let run_h = 95.0;
  let bar_h = 55.0;
  let fixed_total = 20.0 + stats_h + 15.0 + run_h + 15.0 + bar_h + 10.0 + bar_h + 20.0;
  let remaining_h = (screen_h - fixed_total - 20.0).max(120.0);
  let chart_h = (remaining_h / 2.0) - 10.0;
  ```
- Use `ui_kit::draw_progress_bar` and `ui_kit::draw_responsive_chart`.

- [ ] **Step 3: Enhance neural network visualization**

- Make inactive synapse lines more subtle (`Color::new(0.2, 0.2, 0.25, 0.15)`).
- Highlight output node for argmax action with neon cyan glow and border.
- Align labels and values cleanly.

- [ ] **Step 4: Update bottom model-info panel controls**

- Draw structured key shortcuts with badge frames.

- [ ] **Step 5: Run tests to verify everything passes**

Run: `cargo test dqn_dash::tests`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/dqn_dash.rs
git commit -m "feat(ui): responsive layout, progress bars, and neural network polish in dqn_dash"
```

---

### Task 4: Versus & Cross-Match Arenas Overhaul (`src/viz_vs.rs` & `src/view_*_versus.rs`)

**Files:**
- Modify: `src/viz_vs.rs`
- Modify: `src/view_dqn_versus.rs`
- Modify: `src/view_ga_versus.rs`
- Modify: `src/view_cross_match.rs`
- Test: `cargo test`

**Interfaces:**
- Consumes:
  - `crate::ui_kit::{draw_terminal_box, draw_centered_text, draw_missing_champion_notice, ACCENT_GOLD, ACCENT_CYAN, ACCENT_RED, TEXT_MUTED}`
- Produces:
  - Fixed Series HUD layout in `viz_vs.rs`: dedicated top margin (`y = 65.0..75.0`) so the series winner banner never collides with the center panel border or `"VS"` title.
  - Perfectly centered player titles and scores using `measure_text`.
  - Distinct glowing circular win pips for Best-of-5.
  - Unified missing champion notices across `view_dqn_versus.rs`, `view_ga_versus.rs`, and `view_cross_match.rs`.

- [ ] **Step 1: Check existing versus tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 2: Fix Series HUD geometry and scoreboard centering in `src/viz_vs.rs`**

- Shift center panel `y` start from `50.0` to `75.0` (or give top margin 80px) to provide clear room for:
  - Game number: `"GAME {n} / 5"` at `y = 18.0`.
  - Win pips: Player 1 (left) and Player 2 (right) at `y = 36.0`.
  - Series Winner Banner: `"🏆 {TITLE} WINS SERIES!"` at `y = 56.0` completely outside the center panel.
- Center player titles over each grid with `draw_centered_text`.
- Symmetrically align Player 1 and Player 2 scores in the center panel.

- [ ] **Step 3: Unify missing champion screens**

In `view_dqn_versus.rs`, `view_ga_versus.rs`, and `view_cross_match.rs`:
- Replace duplicated text drawing with `ui_kit::draw_missing_champion_notice(screen_width(), screen_height(), title, instruction)`.

- [ ] **Step 4: Run tests to verify all tests pass**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/viz_vs.rs src/view_dqn_versus.rs src/view_ga_versus.rs src/view_cross_match.rs
git commit -m "feat(ui): fix series HUD collision, center versus scoreboard, and unify empty states"
```

---

### Task 5: Verification and Final Polish

**Files:**
- Modify: any minor visual tweaks across modified files if needed
- Test: full test suite and build verification

- [ ] **Step 1: Run full test suite**

Run: `cargo test -- --nocapture`
Expected: All tests PASS.

- [ ] **Step 2: Run release check**

Run: `cargo check --release`
Expected: No errors or warnings.

- [ ] **Step 3: Verification of theme loading and persistence**

Run: `cargo test theme::tests`
Expected: PASS.

- [ ] **Step 4: Commit any final adjustments**

```bash
git commit --allow-empty -m "chore(ui): complete UI/UX terminal polish implementation"
```

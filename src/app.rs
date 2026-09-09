//! Unified app shell — one window, one loop, a menu plus five playable views.
//!
//! [`App`] owns the whole lifecycle and state machine: `Menu | DqnTrain |
//! DqnVersus | GaTrain | GaVersus | DqnVsGa` (AD-1). The shell keeps the DQN
//! trainer ([`DqnTrainView`]) and the GA trainer ([`GaTrainView`]) alive as
//! *paused* holders across menu visits (AD-3): leaving a training view pauses it
//! by simply not ticking it, and the menu shows which paused runs are resumable.
//! Versus/cross views are transient — every entry constructs a fresh match from
//! the paused trainers and/or the persisted champion files.
//!
//! Navigation is a PURE seam ([`next_mode`]) with no macroquad dependency, so the
//! transition table (menu select → view; fresh vs resume for trainers; versus =
//! always-fresh; `Esc` semantics; finished-match `Enter` dismissal) is
//! unit-testable headless. The views themselves are passive: they never poll
//! keys; the shell reads keys each frame, forwards view-only keys (`R` fresh DQN
//! agent, `Space` slow/fast GA, `Tab` advanced GA dashboard, `V` GA internal VS)
//! to the active view, and routes navigation keys through [`next_mode`].
//!
//! Per-mode tick policy (design "Per-mode tick policy"): `DqnTrain` steps its
//! trainer once per frame; `GaTrain` steps through the view's own pacing/batching
//! (which already sleeps); versus/cross matches step one tick per frame; the menu
//! steps nothing.

use macroquad::prelude::*;

use crate::champion_store;
use crate::view_cross_match::CrossMatchView;
use crate::view_dqn_train::{DqnTrainView, DQN_CHAMPION_FILE};
use crate::view_dqn_versus::DqnVersusView;
use crate::view_ga_train::GaTrainView;
use crate::view_ga_versus::GaVersusView;

/// Number of playable menu entries (number keys 1..=5).
pub const MENU_ENTRY_COUNT: usize = 5;

/// The six shell states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Welcome menu; nothing steps here.
    Menu,
    /// DQN training (one `GameDQN::step()` per frame).
    DqnTrain,
    /// DQN internal versus (champion vs live policy / fresh greedy agent).
    DqnVersus,
    /// GA training (legacy evolution semantics, DQN-style HUD).
    GaTrain,
    /// GA internal versus (best-ever vs second-best).
    GaVersus,
    /// Cross match: GA champion vs DQN champion.
    DqnVsGa,
}

impl AppMode {
    /// Is this mode one of the five playable views (not the menu)?
    pub fn is_view(self) -> bool {
        self != AppMode::Menu
    }
}

/// A navigation action the shell can hand the transition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// A menu number key (`1`..=`5`) selecting a view.
    Select(u8),
    /// `Esc`: quit from the menu, back-to-menu from any view.
    Esc,
    /// `Enter`/`Return`: on a finished match view, dismiss back to the menu.
    Enter,
}

/// The pure outcome of evaluating an [`Action`] in a given [`AppMode`].
///
/// Each arm names the destination mode *and* the construction decision (fresh vs
/// resume), so the shell can apply it without re-deriving state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// The app should exit (only `Esc` on the menu produces this).
    Quit,
    /// Nothing changes (an invalid or view-only key for the current mode).
    Stay,
    /// Return to the menu; paused trainers stay alive; transient matches drop.
    ToMenu,
    /// Enter DQN train with a brand-new session (no paused trainer yet).
    DqnTrainNew,
    /// Enter DQN train, resuming the paused session (AD-3).
    DqnTrainResume,
    /// Enter DQN-versus with a fresh transient match (every entry is fresh).
    DqnVersusNew,
    /// Enter GA train with a brand-new session.
    GaTrainNew,
    /// Enter GA train, resuming the paused session.
    GaTrainResume,
    /// Enter GA-versus with a fresh transient match.
    GaVersusNew,
    /// Enter DQN-vs-GA with a fresh transient match.
    CrossNew,
}

/// Pure transition table (macroquad-free; unit-tested headless).
///
/// * `Menu + Esc` → [`Transition::Quit`]; `Menu + Select(n)` → the numbered view
///   (`1` DQN train, `2` DQN-versus, `3` GA train, `4` GA-versus, `5` cross),
///   where the train entries resume when a paused holder exists and start fresh
///   otherwise, and versus/cross entries are always fresh transient matches.
/// * `Menu + Enter` → [`Transition::Stay`]: the arrows+Enter flow is the shell
///   translating the highlighted row into an `Action::Select`.
/// * Any view + `Esc` → [`Transition::ToMenu`] (trainers paused, never destroyed;
///   quitting happens only from the menu).
/// * Match views (DqnVersus/GaVersus/DqnVsGa) + `Enter` → [`Transition::ToMenu`]
///   (dismiss a finished result — the shell only forwards `Enter` when the match
///   is finished). `Enter` is not a trainer key.
/// * Anything else → [`Transition::Stay`].
pub fn next_mode(mode: AppMode, action: Action, has_dqn: bool, has_ga: bool) -> Transition {
    match (mode, action) {
        (AppMode::Menu, Action::Esc) => Transition::Quit,
        (AppMode::Menu, Action::Enter) => Transition::Stay,
        (AppMode::Menu, Action::Select(1)) if has_dqn => Transition::DqnTrainResume,
        (AppMode::Menu, Action::Select(1)) => Transition::DqnTrainNew,
        (AppMode::Menu, Action::Select(2)) => Transition::DqnVersusNew,
        (AppMode::Menu, Action::Select(3)) if has_ga => Transition::GaTrainResume,
        (AppMode::Menu, Action::Select(3)) => Transition::GaTrainNew,
        (AppMode::Menu, Action::Select(4)) => Transition::GaVersusNew,
        (AppMode::Menu, Action::Select(5)) => Transition::CrossNew,
        (AppMode::Menu, Action::Select(_)) => Transition::Stay,
        // Esc from any view: pause (don't destroy) trainers and return to menu.
        (view, Action::Esc) if view.is_view() => Transition::ToMenu,
        // Enter dismisses finished versus/cross results; it is not a trainer key.
        (AppMode::DqnVersus | AppMode::GaVersus | AppMode::DqnVsGa, Action::Enter) => {
            Transition::ToMenu
        }
        _ => Transition::Stay,
    }
}

/// The transient versus/cross match holder: exactly one is alive when the shell
/// is in a match mode, freshly constructed on every entry.
enum MatchView {
    DqnVersus(DqnVersusView),
    GaVersus(GaVersusView),
    Cross(CrossMatchView),
}

impl MatchView {
    fn tick(&mut self) {
        match self {
            MatchView::DqnVersus(v) => v.tick(),
            MatchView::GaVersus(v) => v.tick(),
            MatchView::Cross(v) => v.tick(),
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            MatchView::DqnVersus(v) => v.is_finished(),
            MatchView::GaVersus(v) => v.is_finished(),
            MatchView::Cross(v) => v.is_finished(),
        }
    }

    fn draw(&self) {
        match self {
            MatchView::DqnVersus(v) => v.draw(),
            MatchView::GaVersus(v) => v.draw(),
            MatchView::Cross(v) => v.draw(),
        }
    }
}

/// The unified app shell: menu + mode dispatch + paused trainer holders.
pub struct App {
    mode: AppMode,
    /// Highlighted menu row (index into the five entries; 0-based).
    menu_selection: usize,
    /// Paused DQN trainer; `None` until the first DQN-train entry (AD-3).
    dqn: Option<DqnTrainView>,
    /// Paused GA trainer; `None` until the first GA-train entry.
    ga: Option<GaTrainView>,
    /// Active transient match, `Some` only in a match mode.
    match_view: Option<MatchView>,
    /// Set when the user asked to quit (`Esc` on the menu).
    quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// A fresh shell: menu shown, no trainers or matches running yet.
    pub fn new() -> Self {
        Self {
            mode: AppMode::Menu,
            menu_selection: 0,
            dqn: None,
            ga: None,
            match_view: None,
            quit: false,
        }
    }

    /// Run one shell frame: read keys, tick the active mode, draw it. Returns
    /// `true` when the app should exit (menu `Esc`).
    pub fn run_frame(&mut self) -> bool {
        if !self.quit {
            self.handle_input();
        }
        if !self.quit {
            self.tick_active();
            self.draw();
        }
        self.quit
    }

    /// Whether the app has been asked to quit.
    pub fn quit_requested(&self) -> bool {
        self.quit
    }

    // --- input ---------------------------------------------------------------

    fn handle_input(&mut self) {
        match self.mode {
            AppMode::Menu => self.handle_menu_input(),
            AppMode::DqnTrain => self.handle_dqn_train_input(),
            AppMode::GaTrain => self.handle_ga_train_input(),
            AppMode::DqnVersus | AppMode::GaVersus | AppMode::DqnVsGa => self.handle_match_input(),
        }
    }

    fn handle_menu_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.navigate(Action::Esc);
            return;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Right) {
            self.menu_selection = (self.menu_selection + 1) % MENU_ENTRY_COUNT;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Left) {
            self.menu_selection = (self.menu_selection + MENU_ENTRY_COUNT - 1) % MENU_ENTRY_COUNT;
        }
        // Number keys 1..=5 jump straight to a view.
        let number = self.pressed_menu_number();
        if let Some(n) = number {
            self.navigate(Action::Select(n));
        } else if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            // Arrows+Enter: convert the highlighted row into its Select action.
            let n = self.menu_selection as u8 + 1;
            self.navigate(Action::Select(n));
        }
    }

    fn pressed_menu_number(&self) -> Option<u8> {
        if is_key_pressed(KeyCode::Key1) {
            Some(1)
        } else if is_key_pressed(KeyCode::Key2) {
            Some(2)
        } else if is_key_pressed(KeyCode::Key3) {
            Some(3)
        } else if is_key_pressed(KeyCode::Key4) {
            Some(4)
        } else if is_key_pressed(KeyCode::Key5) {
            Some(5)
        } else {
            None
        }
    }

    fn handle_dqn_train_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.navigate(Action::Esc);
            return;
        }
        if is_key_pressed(KeyCode::R) {
            if let Some(view) = &mut self.dqn {
                view.fresh_agent();
            }
        }
    }

    fn handle_ga_train_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.navigate(Action::Esc);
            return;
        }
        if is_key_released(KeyCode::Space) {
            if let Some(view) = &mut self.ga {
                view.toggle_slow();
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            if let Some(view) = &mut self.ga {
                view.toggle_advanced();
            }
        }
        if is_key_pressed(KeyCode::V) {
            if let Some(view) = &mut self.ga {
                view.trigger_vs();
            }
        }
    }

    fn handle_match_input(&mut self) {
        // Esc always returns to the menu (aborts a running match or leaves a
        // finished one). Enter dismisses only a *finished* result — the shell
        // gates it so a stray Enter never aborts a running match.
        let finished = self.match_view.as_ref().is_some_and(|m| m.is_finished());
        if is_key_pressed(KeyCode::Escape) {
            self.navigate(Action::Esc);
        } else if finished && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter)) {
            self.navigate(Action::Enter);
        }
    }

    /// Route a navigation action through the pure transition table and apply it.
    fn navigate(&mut self, action: Action) {
        let transition = next_mode(self.mode, action, self.dqn.is_some(), self.ga.is_some());
        self.apply(transition);
    }

    fn apply(&mut self, transition: Transition) {
        match transition {
            Transition::Quit => self.quit = true,
            Transition::Stay => {}
            Transition::ToMenu => {
                self.mode = AppMode::Menu;
                // Paused trainers (self.dqn/self.ga) are retained; the transient
                // match is dropped so the next entry builds a fresh one.
                self.match_view = None;
            }
            Transition::DqnTrainNew => {
                self.mode = AppMode::DqnTrain;
                self.match_view = None;
                self.dqn = Some(DqnTrainView::new());
            }
            Transition::DqnTrainResume => {
                self.mode = AppMode::DqnTrain;
                self.match_view = None;
            }
            Transition::DqnVersusNew => {
                self.mode = AppMode::DqnVersus;
                self.match_view = Some(MatchView::DqnVersus(self.build_dqn_versus()));
            }
            Transition::GaTrainNew => {
                self.mode = AppMode::GaTrain;
                self.match_view = None;
                self.ga = Some(GaTrainView::new());
            }
            Transition::GaTrainResume => {
                self.mode = AppMode::GaTrain;
                self.match_view = None;
            }
            Transition::GaVersusNew => {
                self.mode = AppMode::GaVersus;
                self.match_view = Some(MatchView::GaVersus(GaVersusView::new()));
            }
            Transition::CrossNew => {
                self.mode = AppMode::DqnVsGa;
                self.match_view = Some(MatchView::Cross(CrossMatchView::new()));
            }
        }
    }

    /// Build a fresh DQN-versus match from the paused trainer (champion + live
    /// policy when one exists) or, with no paused trainer yet, from the persisted
    /// `dqn_champion.json`. The view itself falls into its message state when no
    /// champion exists anywhere (spec: "train DQN first").
    fn build_dqn_versus(&self) -> DqnVersusView {
        let champion = self
            .dqn
            .as_ref()
            .and_then(|view| view.champion().cloned())
            .or_else(|| champion_store::load(DQN_CHAMPION_FILE));
        // Live policy only exists while a paused trainer is alive; without one
        // the view pits the champion against a fresh greedy agent.
        let live = self.dqn.as_ref().map(|view| view.live_net().clone());
        DqnVersusView::new(champion, live)
    }

    // --- per-mode tick ---------------------------------------------------------

    fn tick_active(&mut self) {
        match self.mode {
            AppMode::Menu => {}
            AppMode::DqnTrain => {
                if let Some(view) = &mut self.dqn {
                    view.tick();
                }
            }
            AppMode::GaTrain => {
                if let Some(view) = &mut self.ga {
                    view.tick();
                }
            }
            AppMode::DqnVersus | AppMode::GaVersus | AppMode::DqnVsGa => {
                if let Some(match_) = &mut self.match_view {
                    match_.tick();
                }
            }
        }
    }

    // --- drawing ---------------------------------------------------------------

    fn draw(&mut self) {
        match self.mode {
            AppMode::Menu => self.draw_menu(),
            AppMode::DqnTrain => self.draw_dqn_train(),
            AppMode::GaTrain => self.draw_ga_train(),
            AppMode::DqnVersus | AppMode::GaVersus | AppMode::DqnVsGa => self.draw_match(),
        }
    }

    fn draw_menu(&self) {
        clear_background(BLACK);

        let (w, h) = (screen_width(), screen_height());
        let center_x = w * 0.5;

        self.centered_text("SNAKE AI", center_x, h * 0.07, 48.0, WHITE);
        self.centered_text("choose a mode", center_x, h * 0.07 + 42.0, 20.0, GRAY);

        let dqn_label = match &self.dqn {
            // AD-3 / spec: a paused DQN run is resumable from the menu, and the
            // menu says so (with its episode counter) instead of hiding it.
            Some(view) => format!(
                "1) DQN Train  (paused at episode {} - press 1 to resume)",
                view.episode()
            ),
            None => "1) DQN Train  (start fresh agent)".to_owned(),
        };
        let ga_label = if self.ga.is_some() {
            "3) GA Train  (paused - press 3 to resume)".to_owned()
        } else {
            "3) GA Train  (start fresh)".to_owned()
        };
        let entries: [String; 5] = [
            dqn_label,
            "2) DQN Versus".to_owned(),
            ga_label,
            "4) GA Versus".to_owned(),
            "5) DQN vs GA".to_owned(),
        ];

        let start_y = h * 0.30;
        let row_h = h * 0.10;
        for (i, label) in entries.iter().enumerate() {
            let y = start_y + i as f32 * row_h;
            let selected = i == self.menu_selection;
            let color = if selected { YELLOW } else { WHITE };
            self.centered_text(label, center_x, y, 30.0, color);
            if selected {
                // Selection chevrons either side of the highlighted row.
                self.centered_text("<", center_x - 260.0, y, 30.0, YELLOW);
                self.centered_text(">", center_x + 260.0, y, 30.0, YELLOW);
            }
        }

        self.centered_text(
            "[1-5] or [Up/Down]+[Enter] select    [ESC] Quit",
            center_x,
            h * 0.93,
            18.0,
            GRAY,
        );
    }

    fn centered_text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        let dims = measure_text(text, None, font_size as u16, 1.0);
        draw_text(text, x - dims.width * 0.5, y, font_size, color);
    }

    fn draw_dqn_train(&mut self) {
        if let Some(view) = &self.dqn {
            view.draw();
            // DQN-style HUD leaves the bottom of the left text column free; the
            // shell overlays the view hotkeys there (the view is passive).
            let hint = "[R] fresh agent   [ESC] menu";
            draw_text(hint, 10.0, screen_height() - 12.0, 18.0, GRAY);
        }
    }

    fn draw_ga_train(&mut self) {
        if let Some(view) = &self.ga {
            view.draw();
        }
    }

    fn draw_match(&mut self) {
        if let Some(match_) = &self.match_view {
            match_.draw();
        }
    }
}

// ---------------------------------------------------------------------------
// Pure navigation tests (T5.1, RED-first)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{next_mode, Action, AppMode, Transition};

    // The five playable views, in menu order (number keys 1..=5).
    const ALL_VIEWS: [AppMode; 5] = [
        AppMode::DqnTrain,
        AppMode::DqnVersus,
        AppMode::GaTrain,
        AppMode::GaVersus,
        AppMode::DqnVsGa,
    ];

    // --- Menu Esc semantics ---------------------------------------------------

    #[test]
    fn escape_on_the_menu_quits_the_app() {
        assert_eq!(
            next_mode(AppMode::Menu, Action::Esc, false, false),
            Transition::Quit
        );
        // has_dqn/has_ga must not change the menu-quit decision.
        assert_eq!(
            next_mode(AppMode::Menu, Action::Esc, true, true),
            Transition::Quit
        );
    }

    #[test]
    fn escape_from_any_view_returns_to_the_menu_never_quits() {
        // Esc in a view pauses the trainers and comes back to the menu; quitting
        // is a menu-only action.
        for view in ALL_VIEWS {
            assert_eq!(
                next_mode(view, Action::Esc, false, false),
                Transition::ToMenu,
                "Esc from {view:?} must return to the menu"
            );
        }
    }

    // --- Menu select: fresh vs resume (paused-trainer cases) ------------------

    #[test]
    fn selecting_dqn_train_without_a_paused_trainer_starts_fresh() {
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(1), false, false),
            Transition::DqnTrainNew
        );
    }

    #[test]
    fn selecting_dqn_train_with_a_paused_trainer_resumes_it() {
        // The paused trainer is kept alive across menu visits (AD-3): selecting
        // DQN train again must resume the same session, not start a new agent.
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(1), true, false),
            Transition::DqnTrainResume
        );
    }

    #[test]
    fn selecting_ga_train_without_a_paused_trainer_starts_fresh() {
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(3), false, false),
            Transition::GaTrainNew
        );
    }

    #[test]
    fn selecting_ga_train_with_a_paused_trainer_resumes_it() {
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(3), false, true),
            Transition::GaTrainResume
        );
    }

    // --- Menu select: versus/cross are always fresh transient matches ----------

    #[test]
    fn versus_entries_always_build_fresh_matches_even_when_trainers_are_paused() {
        // Versus and cross views are transient: every entry constructs a brand-new
        // arena state (spec), regardless of paused trainers.
        for has_dqn in [false, true] {
            for has_ga in [false, true] {
                assert_eq!(
                    next_mode(AppMode::Menu, Action::Select(2), has_dqn, has_ga),
                    Transition::DqnVersusNew,
                    "DQN-versus entry must always be a fresh match (dqn={has_dqn}, ga={has_ga})"
                );
                assert_eq!(
                    next_mode(AppMode::Menu, Action::Select(4), has_dqn, has_ga),
                    Transition::GaVersusNew,
                    "GA-versus entry must always be a fresh match (dqn={has_dqn}, ga={has_ga})"
                );
                assert_eq!(
                    next_mode(AppMode::Menu, Action::Select(5), has_dqn, has_ga),
                    Transition::CrossNew,
                    "cross entry must always be a fresh match (dqn={has_dqn}, ga={has_ga})"
                );
            }
        }
    }

    #[test]
    fn invalid_menu_selection_numbers_are_ignored() {
        // Only keys 1..=5 map to a view; stray number keys must not move the app.
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(0), false, false),
            Transition::Stay
        );
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(6), false, false),
            Transition::Stay
        );
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(9), false, false),
            Transition::Stay
        );
    }

    // --- Enter semantics ------------------------------------------------------

    #[test]
    fn enter_on_the_menu_is_a_shell_translation_not_a_direct_transition() {
        // The arrows+Enter flow works by the shell converting the highlighted row
        // into an Action::Select; a bare Enter on the menu changes nothing by itself.
        assert_eq!(
            next_mode(AppMode::Menu, Action::Enter, false, false),
            Transition::Stay
        );
    }

    #[test]
    fn enter_dismisses_a_finished_match_view_but_is_not_a_trainer_key() {
        // Match views (DqnVersus/GaVersus/DqnVsGa): Enter dismisses the finished
        // result and returns to the menu. Trainer views have no Enter meaning.
        for match_view in [AppMode::DqnVersus, AppMode::GaVersus, AppMode::DqnVsGa] {
            assert_eq!(
                next_mode(match_view, Action::Enter, false, false),
                Transition::ToMenu,
                "Enter from {match_view:?} must return to the menu"
            );
        }
        assert_eq!(
            next_mode(AppMode::DqnTrain, Action::Enter, false, false),
            Transition::Stay
        );
        assert_eq!(
            next_mode(AppMode::GaTrain, Action::Enter, false, false),
            Transition::Stay
        );
    }

    // --- Cross-mode noise is ignored -------------------------------------------

    #[test]
    fn number_select_is_ignored_while_a_view_is_active() {
        // Number keys only mean something on the menu.
        for view in ALL_VIEWS {
            assert_eq!(
                next_mode(view, Action::Select(1), false, false),
                Transition::Stay,
                "Select must be ignored in {view:?}"
            );
        }
    }
}

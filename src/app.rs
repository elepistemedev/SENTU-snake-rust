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
//! keys; the shell reads keys each frame, forwards view-only keys (`R` fresh
//! DQN agent, `Tab` toggles the DQN dashboard, `Space` slow/fast GA, `Tab`
//! toggles the advanced GA dashboard, `V` GA internal VS) to the active view,
//! and routes navigation keys through [`next_mode`].
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

/// Number of playable menu entries (number keys 1..=6).
pub const MENU_ENTRY_COUNT: usize = 6;

/// The seven shell states.
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
    /// Theme and visual configuration.
    ThemeConfig,
}

impl AppMode {
    /// Is this mode one of the playable or settings views (not the menu)?
    pub fn is_view(self) -> bool {
        self != AppMode::Menu
    }
}

/// A navigation action the shell can hand the transition table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// A menu number key (`1`..=`6`) selecting a view.
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
    /// Enter theme configuration view.
    ThemeConfig,
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
        (AppMode::Menu, Action::Select(6)) => Transition::ThemeConfig,
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
    /// Currently active visual theme.
    active_theme: crate::theme::GameTheme,
    /// Currently highlighted theme index in the ThemeConfig screen (0..3).
    theme_selection: usize,
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
        let active_theme = crate::theme::load_theme();
        let theme_selection = match active_theme {
            crate::theme::GameTheme::Retro => 0,
            crate::theme::GameTheme::Arcade => 1,
            crate::theme::GameTheme::Pleasant => 2,
            crate::theme::GameTheme::Meadow => 3,
        };
        Self {
            mode: AppMode::Menu,
            menu_selection: 0,
            active_theme,
            theme_selection,
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
            AppMode::ThemeConfig => self.handle_theme_config_input(),
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
        // Number keys 1..=6 jump straight to a view.
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
        } else if is_key_pressed(KeyCode::Key6) {
            Some(6)
        } else {
            None
        }
    }

    fn handle_theme_config_input(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.navigate(Action::Esc);
            return;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Left) {
            if self.theme_selection == 0 {
                self.theme_selection = 3;
            } else {
                self.theme_selection -= 1;
            }
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Right) {
            self.theme_selection = (self.theme_selection + 1) % 4;
        }
        if is_key_pressed(KeyCode::Key1) {
            self.theme_selection = 0;
        } else if is_key_pressed(KeyCode::Key2) {
            self.theme_selection = 1;
        } else if is_key_pressed(KeyCode::Key3) {
            self.theme_selection = 2;
        } else if is_key_pressed(KeyCode::Key4) {
            self.theme_selection = 3;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            let selected_theme = match self.theme_selection {
                0 => crate::theme::GameTheme::Retro,
                1 => crate::theme::GameTheme::Arcade,
                2 => crate::theme::GameTheme::Pleasant,
                _ => crate::theme::GameTheme::Meadow,
            };
            self.active_theme = selected_theme;
            crate::theme::save_theme(selected_theme).ok();
            self.navigate(Action::Esc);
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
        if is_key_pressed(KeyCode::Tab) {
            if let Some(view) = &mut self.dqn {
                view.toggle_dashboard();
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
            Transition::ThemeConfig => {
                self.mode = AppMode::ThemeConfig;
                self.match_view = None;
                self.theme_selection = match self.active_theme {
                    crate::theme::GameTheme::Retro => 0,
                    crate::theme::GameTheme::Arcade => 1,
                    crate::theme::GameTheme::Pleasant => 2,
                    crate::theme::GameTheme::Meadow => 3,
                };
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
            AppMode::ThemeConfig => {}
        }
    }

    // --- drawing ---------------------------------------------------------------

    fn draw(&mut self) {
        match self.mode {
            AppMode::Menu => self.draw_menu(),
            AppMode::DqnTrain => self.draw_dqn_train(),
            AppMode::GaTrain => self.draw_ga_train(),
            AppMode::DqnVersus | AppMode::GaVersus | AppMode::DqnVsGa => self.draw_match(),
            AppMode::ThemeConfig => self.draw_theme_config(),
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
        let entries: [String; 6] = [
            dqn_label,
            "2) DQN Versus".to_owned(),
            ga_label,
            "4) GA Versus".to_owned(),
            "5) DQN vs GA".to_owned(),
            format!("6) Configuración y Temas  [Tema: {}]", self.active_theme.name()),
        ];

        let start_y = h * 0.25;
        let row_h = h * 0.09;
        for (i, label) in entries.iter().enumerate() {
            let y = start_y + i as f32 * row_h;
            let selected = i == self.menu_selection;
            let color = if selected { YELLOW } else { WHITE };
            self.centered_text(label, center_x, y, 30.0, color);
            if selected {
                let dims = measure_text(label, None, 30, 1.0);
                self.centered_text("<", center_x - dims.width * 0.5 - 25.0, y, 30.0, YELLOW);
                self.centered_text(">", center_x + dims.width * 0.5 + 25.0, y, 30.0, YELLOW);
            }
        }

        self.centered_text(
            "[1-6] or [Up/Down]+[Enter] select    [ESC] Quit",
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
                // The shell hotkey hint must not overlap the dashboard's bottom
                // model-info panel (design D-8): it is drawn only while the compact
                // HUD is the active target. The dashboard carries its own controls
                // line inside the model-info panel.
                if !view.dashboard_enabled() {
                    let hint = "[R] fresh agent   [ESC] menu";
                    draw_text(hint, 10.0, screen_height() - 12.0, 18.0, GRAY);
                }
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

    fn draw_theme_config(&self) {
        clear_background(BLACK);

        let (w, h) = (screen_width(), screen_height());
        let center_x = w * 0.5;

        self.centered_text("CONFIGURACIÓN Y TEMAS", center_x, h * 0.08, 40.0, WHITE);
        self.centered_text(
            "Personaliza los colores de la serpiente y la manzana",
            center_x,
            h * 0.08 + 36.0,
            18.0,
            GRAY,
        );

        let themes = crate::theme::GameTheme::all();
        let preview_theme = match self.theme_selection {
            0 => crate::theme::GameTheme::Retro,
            1 => crate::theme::GameTheme::Arcade,
            2 => crate::theme::GameTheme::Pleasant,
            _ => crate::theme::GameTheme::Meadow,
        };
        let preview_colors = preview_theme.colors();

        // Left column: Theme list cards (4 themes)
        let list_x = w * 0.08;
        let list_y = h * 0.17;
        let row_h = h * 0.18;

        for (i, &theme) in themes.iter().enumerate() {
            let y = list_y + i as f32 * row_h;
            let is_selected = i == self.theme_selection;
            let is_active = theme == self.active_theme;

            let card_w = w * 0.44;
            let card_h = row_h * 0.88;

            let bg_color = if is_selected {
                Color::new(0.12, 0.15, 0.22, 1.0)
            } else {
                Color::new(0.06, 0.06, 0.08, 1.0)
            };
            let border_color = if is_selected {
                YELLOW
            } else if is_active {
                Color::new(0.3, 0.8, 0.4, 1.0)
            } else {
                Color::new(0.2, 0.2, 0.25, 1.0)
            };

            draw_rectangle(list_x, y, card_w, card_h, bg_color);
            draw_rectangle_lines(list_x, y, card_w, card_h, 2.0, border_color);

            // Number + Name
            let title_color = if is_selected { YELLOW } else { WHITE };
            let title = format!("{}) Tema {}", i + 1, theme.name());
            draw_text(&title, list_x + 18.0, y + 32.0, 24.0, title_color);

            // Active badge
            if is_active {
                draw_text("[ACTIVO]", list_x + card_w - 95.0, y + 32.0, 18.0, Color::new(0.3, 0.9, 0.4, 1.0));
            }

            // Description
            draw_text(theme.description(), list_x + 18.0, y + 62.0, 16.0, GRAY);

            // Swatches inside card
            let tc = theme.colors();
            let swatch_y = y + card_h - 26.0;
            draw_rectangle(list_x + 18.0, swatch_y, 14.0, 14.0, tc.head);
            draw_text("Cabeza", list_x + 38.0, swatch_y + 11.0, 14.0, tc.head);

            draw_rectangle(list_x + 110.0, swatch_y, 14.0, 14.0, tc.body);
            draw_text("Cuerpo", list_x + 130.0, swatch_y + 11.0, 14.0, tc.body);

            draw_rectangle(list_x + 200.0, swatch_y, 14.0, 14.0, tc.food);
            draw_text("Manzana", list_x + 220.0, swatch_y + 11.0, 14.0, tc.food);
        }

        // Right column: Live preview panel
        let preview_x = w * 0.56;
        let preview_y = h * 0.17;
        let preview_w = w * 0.36;
        let preview_h = row_h * 3.88;

        draw_rectangle(preview_x, preview_y, preview_w, preview_h, Color::new(0.05, 0.05, 0.07, 1.0));
        draw_rectangle_lines(preview_x, preview_y, preview_w, preview_h, 2.0, Color::new(0.3, 0.35, 0.45, 1.0));

        let preview_title = format!("VISTA PREVIA: {}", preview_theme.name().to_uppercase());
        self.centered_text(&preview_title, preview_x + preview_w * 0.5, preview_y + 35.0, 20.0, YELLOW);

        // Draw mini board grid in the preview
        let grid_size = 12;
        let cell_size = (preview_w.min(preview_h) * 0.50 / grid_size as f32).floor();
        let board_w = cell_size * grid_size as f32;
        let board_h = cell_size * grid_size as f32;
        let board_x = preview_x + (preview_w - board_w) * 0.5;
        let board_y = preview_y + 55.0;

        draw_rectangle(board_x, board_y, board_w, board_h, Color::new(0.02, 0.02, 0.03, 1.0));
        draw_rectangle_lines(board_x, board_y, board_w, board_h, 1.0, Color::new(0.2, 0.2, 0.25, 1.0));

        // Subtle grid lines
        for g in 1..grid_size {
            let gx = board_x + g as f32 * cell_size;
            let gy = board_y + g as f32 * cell_size;
            draw_line(gx, board_y, gx, board_y + board_h, 1.0, Color::new(0.1, 0.1, 0.12, 1.0));
            draw_line(board_x, gy, board_x + board_w, gy, 1.0, Color::new(0.1, 0.1, 0.12, 1.0));
        }

        // Draw food
        let food_pos = (8, 3);
        crate::render_snake::draw_apple(
            board_x + food_pos.0 as f32 * cell_size,
            board_y + food_pos.1 as f32 * cell_size,
            cell_size,
            preview_theme,
            preview_colors.food,
        );

        // Draw snake (head + 4 body segments)
        let snake_cells = [
            (5, 5), // head (facing Right)
            (4, 5), // body 1
            (3, 5), // body 2 (with preview digestion bulge!)
            (3, 6), // body 3
            (3, 7), // body 4
        ];
        for (idx, &(cx, cy)) in snake_cells.iter().enumerate() {
            let seg_x = board_x + cx as f32 * cell_size;
            let seg_y = board_y + cy as f32 * cell_size;
            if idx == 0 {
                crate::render_snake::draw_snake_head(
                    seg_x,
                    seg_y,
                    cell_size,
                    crate::utils::FourDirs::Right,
                    preview_theme,
                    preview_colors.head,
                    0.0,
                );
            } else {
                let bulge = if idx == 2 { 0.35 } else { 0.0 };
                crate::render_snake::draw_snake_body(
                    seg_x,
                    seg_y,
                    cell_size,
                    preview_theme,
                    preview_colors.body,
                    bulge,
                );
            }
        }

        // Explanatory note inside preview
        let note_y = board_y + board_h + 30.0;
        self.centered_text(
            "Se aplica a todos los modos de juego y entrenamiento",
            preview_x + preview_w * 0.5,
            note_y,
            14.0,
            GRAY,
        );

        // Footer instructions
        self.centered_text(
            "[1-4] o [Arriba/Abajo] Seleccionar    [Enter] Guardar y Salir    [ESC] Cancelar",
            center_x,
            h * 0.94,
            18.0,
            YELLOW,
        );
    }
}

// ---------------------------------------------------------------------------
// Pure navigation tests (T5.1, RED-first)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{next_mode, Action, AppMode, Transition};

    // The six playable/configurable views, in menu order (number keys 1..=6).
    const ALL_VIEWS: [AppMode; 6] = [
        AppMode::DqnTrain,
        AppMode::DqnVersus,
        AppMode::GaTrain,
        AppMode::GaVersus,
        AppMode::DqnVsGa,
        AppMode::ThemeConfig,
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
    fn selecting_theme_config_transitions_to_theme_config() {
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(6), false, false),
            Transition::ThemeConfig
        );
    }

    #[test]
    fn invalid_menu_selection_numbers_are_ignored() {
        // Only keys 1..=6 map to a view; stray number keys must not move the app.
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(0), false, false),
            Transition::Stay
        );
        assert_eq!(
            next_mode(AppMode::Menu, Action::Select(7), false, false),
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

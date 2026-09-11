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
use crate::ui_kit::{
    draw_badge, draw_brand_watermark, draw_centered_text, draw_terminal_box, ACCENT_CYAN,
    ACCENT_GOLD, ACCENT_GREEN, ACCENT_RED, COLOR_BG, PANEL_BG, PANEL_BORDER, PANEL_BORDER_FOCUSED,
    TEXT_MUTED,
};
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

enum MatchInner {
    DqnVersus(DqnVersusView),
    GaVersus(GaVersusView),
    Cross(CrossMatchView),
}

/// The transient versus/cross match holder: exactly one is alive when the shell
/// is in a match mode, freshly constructed on every entry.
pub struct MatchView {
    inner: MatchInner,
    slow: bool,
}

impl MatchView {
    pub fn dqn_versus(v: DqnVersusView) -> Self {
        Self {
            inner: MatchInner::DqnVersus(v),
            slow: true,
        }
    }

    pub fn ga_versus(v: GaVersusView) -> Self {
        Self {
            inner: MatchInner::GaVersus(v),
            slow: true,
        }
    }

    pub fn cross(v: CrossMatchView) -> Self {
        Self {
            inner: MatchInner::Cross(v),
            slow: true,
        }
    }

    pub fn is_slow(&self) -> bool {
        self.slow
    }

    pub fn toggle_slow(&mut self) {
        self.slow = !self.slow;
    }

    fn tick(&mut self) {
        let budget = if self.slow {
            1
        } else {
            *crate::view_ga_train::MAX_FAST_TICKS_PER_FRAME
        };
        for _ in 0..budget {
            match &mut self.inner {
                MatchInner::DqnVersus(v) => v.tick(),
                MatchInner::GaVersus(v) => v.tick(),
                MatchInner::Cross(v) => v.tick(),
            }
            if self.is_finished() {
                break;
            }
        }
        if self.slow {
            std::thread::sleep(std::time::Duration::from_millis(*crate::configs::SIM_SLEEP_MILLIS));
        }
    }

    fn is_finished(&self) -> bool {
        match &self.inner {
            MatchInner::DqnVersus(v) => v.is_finished(),
            MatchInner::GaVersus(v) => v.is_finished(),
            MatchInner::Cross(v) => v.is_finished(),
        }
    }

    fn restart(&mut self) {
        match &mut self.inner {
            MatchInner::DqnVersus(v) => v.restart(),
            MatchInner::GaVersus(v) => v.restart(),
            MatchInner::Cross(v) => v.restart(),
        }
    }

    fn draw(&self) {
        match &self.inner {
            MatchInner::DqnVersus(v) => v.draw(),
            MatchInner::GaVersus(v) => v.draw(),
            MatchInner::Cross(v) => v.draw(),
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
        let old_sel = self.theme_selection;

        if is_key_pressed(KeyCode::Escape) {
            let selected_theme = match self.theme_selection {
                0 => crate::theme::GameTheme::Retro,
                1 => crate::theme::GameTheme::Arcade,
                2 => crate::theme::GameTheme::Pleasant,
                _ => crate::theme::GameTheme::Meadow,
            };
            self.active_theme = selected_theme;
            crate::theme::save_theme(selected_theme).ok();
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

        let selected_theme = match self.theme_selection {
            0 => crate::theme::GameTheme::Retro,
            1 => crate::theme::GameTheme::Arcade,
            2 => crate::theme::GameTheme::Pleasant,
            _ => crate::theme::GameTheme::Meadow,
        };

        if self.theme_selection != old_sel {
            self.active_theme = selected_theme;
            crate::theme::save_theme(selected_theme).ok();
        }

        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
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
        if is_key_released(KeyCode::Space) {
            if let Some(view) = &mut self.dqn {
                view.toggle_slow();
            }
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
        // R restarts the series if finished (rematch).
        let finished = self.match_view.as_ref().is_some_and(|m| m.is_finished());
        if is_key_pressed(KeyCode::Escape) {
            self.navigate(Action::Esc);
        } else if finished && is_key_pressed(KeyCode::R) {
            if let Some(m) = &mut self.match_view {
                m.restart();
            }
        } else if finished && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter)) {
            self.navigate(Action::Enter);
        } else if is_key_released(KeyCode::Space) {
            if let Some(m) = &mut self.match_view {
                m.toggle_slow();
            }
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
                if let Some(dqn) = &self.dqn {
                    dqn.sync_save();
                }
                if let Some(ga) = &self.ga {
                    ga.sync_save();
                }
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
                self.match_view = Some(MatchView::dqn_versus(self.build_dqn_versus()));
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
                self.match_view = Some(MatchView::ga_versus(self.build_ga_versus()));
            }
            Transition::CrossNew => {
                self.mode = AppMode::DqnVsGa;
                self.match_view = Some(MatchView::cross(self.build_cross_match()));
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

    /// Build a fresh GA-versus match from the paused GA trainer or disk fallback.
    fn build_ga_versus(&self) -> GaVersusView {
        let champions = self
            .ga
            .as_ref()
            .map(|view| view.champions())
            .unwrap_or_else(crate::sim::load_ga_champions);
        GaVersusView::from_champions(champions)
    }

    /// Build a fresh Cross-Match arena using best available GA and DQN champions.
    fn build_cross_match(&self) -> CrossMatchView {
        let ga = self
            .ga
            .as_ref()
            .and_then(|view| view.best_net())
            .or_else(crate::pop::Population::load_best_net);
        let dqn = self
            .dqn
            .as_ref()
            .and_then(|view| view.champion().cloned())
            .or_else(|| champion_store::load(DQN_CHAMPION_FILE));
        CrossMatchView::from_nets(ga, dqn)
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
        draw_brand_watermark();
    }

    fn has_dqn_champion(&self) -> bool {
        self.dqn.as_ref().and_then(|v| v.champion()).is_some()
            || std::path::Path::new(DQN_CHAMPION_FILE).is_file()
    }

    fn has_ga_champion(&self) -> bool {
        self.ga.as_ref().and_then(|v| v.best_net()).is_some()
            || std::path::Path::new("best_snake.json").is_file()
            || std::path::Path::new("sim_metadata.json").is_file()
    }

    fn draw_menu(&self) {
        clear_background(COLOR_BG);

        let (w, h) = (screen_width(), screen_height());
        let center_x = w * 0.5;

        // Top terminal title box
        let title_w = (w * 0.58).clamp(420.0, 560.0);
        let title_h = 72.0;
        let title_x = center_x - title_w * 0.5;
        let title_y = (h * 0.04).max(12.0);

        draw_terminal_box(title_x, title_y, title_w, title_h, "SNAKE AI", false);
        draw_centered_text(
            "AUTONOMOUS DEEP RL & GA SYSTEM",
            center_x,
            title_y + 48.0,
            14.0,
            TEXT_MUTED,
        );

        // Bottom status bar
        let status_bar_h = 32.0;
        let status_bar_y = h - status_bar_h;

        // Check live champion status via lightweight file existence / memory checks
        let has_dqn_champ = self.has_dqn_champion();
        let has_ga_champ = self.has_ga_champion();

        // 6 Menu Entries with live badges
        let dqn_train_badge = self
            .dqn
            .as_ref()
            .map(|view| (format!("[PAUSADO - EP. {}]", view.episode()), ACCENT_GOLD));

        let dqn_versus_badge = if has_dqn_champ {
            Some(("[CHAMPION LISTO]".to_string(), ACCENT_GREEN))
        } else {
            Some(("[REQUIERE CHAMPION]".to_string(), ACCENT_RED))
        };

        let ga_train_badge = if self.ga.is_some() {
            Some(("[PAUSADO]".to_string(), ACCENT_GOLD))
        } else {
            None
        };

        let ga_versus_badge = if has_ga_champ {
            Some(("[CHAMPION LISTO]".to_string(), ACCENT_GREEN))
        } else {
            Some(("[REQUIERE CHAMPION]".to_string(), ACCENT_RED))
        };

        let cross_badge = if has_dqn_champ && has_ga_champ {
            Some(("[CHAMPION LISTO]".to_string(), ACCENT_GREEN))
        } else {
            Some(("[REQUIERE CHAMPION]".to_string(), ACCENT_RED))
        };

        let theme_badge = Some((
            format!("[TEMA: {}]", self.active_theme.name().to_uppercase()),
            ACCENT_CYAN,
        ));

        let items: [(&str, Option<(String, Color)>); 6] = [
            ("DQN Train", dqn_train_badge),
            ("DQN Versus", dqn_versus_badge),
            ("Algoritmo Genético (Train)", ga_train_badge),
            ("Algoritmo Genético (Versus)", ga_versus_badge),
            ("DQN vs Algoritmo Genético", cross_badge),
            ("Configuración y Temas", theme_badge),
        ];

        let menu_w = (w * 0.72).clamp(560.0, 720.0);
        let card_x = center_x - menu_w * 0.5;
        let start_y = title_y + title_h + (h * 0.03).max(14.0);
        let avail_h = status_bar_y - start_y - 14.0;
        let gap = 10.0;
        let card_h = ((avail_h - gap * 5.0) / 6.0).clamp(42.0, 56.0);

        for (i, (label, badge_info)) in items.iter().enumerate() {
            let card_y = start_y + i as f32 * (card_h + gap);
            let is_selected = i == self.menu_selection;

            if is_selected {
                draw_rectangle(
                    card_x,
                    card_y,
                    menu_w,
                    card_h,
                    Color::new(0.08, 0.12, 0.16, 0.95),
                );
                draw_rectangle_lines(
                    card_x + 1.0,
                    card_y + 1.0,
                    menu_w - 2.0,
                    card_h - 2.0,
                    1.0,
                    PANEL_BORDER_FOCUSED,
                );
            }
            draw_terminal_box(card_x, card_y, menu_w, card_h, "", is_selected);

            // Retro cursor on left
            if is_selected {
                draw_text("▶", card_x + 12.0, card_y + card_h * 0.5 + 6.0, 18.0, ACCENT_CYAN);
            }

            // Key badge [ 1 ] .. [ 6 ]
            let key_str = format!("[ {} ]", i + 1);
            let key_color = if is_selected { ACCENT_CYAN } else { PANEL_BORDER };
            let key_y = card_y + (card_h - 20.0) * 0.5;
            draw_badge(&key_str, card_x + 34.0, key_y, key_color);

            // Item label
            let label_color = if is_selected { ACCENT_GOLD } else { WHITE };
            draw_text(label, card_x + 92.0, card_y + card_h * 0.5 + 6.0, 19.0, label_color);

            // Status badge (right-aligned)
            if let Some((badge_str, badge_color)) = badge_info {
                let b_dims = measure_text(badge_str, None, 13, 1.0);
                let b_w = b_dims.width + 16.0;
                let b_x = card_x + menu_w - 38.0 - b_w;
                let b_y = card_y + (card_h - 20.0) * 0.5;
                draw_badge(badge_str, b_x, b_y, *badge_color);
            }

            // Retro cursor on right
            if is_selected {
                draw_text("◀", card_x + menu_w - 24.0, card_y + card_h * 0.5 + 6.0, 18.0, ACCENT_CYAN);
            }
        }

        // Bottom status bar across screen width
        draw_rectangle(0.0, status_bar_y, w, status_bar_h, PANEL_BG);
        draw_line(0.0, status_bar_y, w, status_bar_y, 1.5, PANEL_BORDER);
        let status_text = format!(
            "[1-6] / [↑↓] + [ENTER] Iniciar   [ESC] Salir   |   TEMA: {}",
            self.active_theme.name()
        );
        draw_centered_text(
            &status_text,
            center_x,
            status_bar_y + 20.0,
            14.0,
            TEXT_MUTED,
        );
    }

    fn draw_dqn_train(&mut self) {
        if let Some(view) = &self.dqn {
            view.draw(self.active_theme);
            // The shell hotkey hint must not overlap the dashboard's bottom
            // model-info panel (design D-8): it is drawn only while the compact
            // HUD is the active target. The dashboard carries its own controls
            // line inside the model-info panel.
            if !view.dashboard_enabled() {
                let hint = "[R] fresh agent   [ESC] menu";
                draw_text(hint, 220.0, screen_height() - 12.0, 18.0, GRAY);
            }
        }
    }

    fn draw_ga_train(&mut self) {
        if let Some(view) = &self.ga {
            view.draw(self.active_theme);
        }
    }


    fn draw_match(&mut self) {
        if let Some(match_) = &self.match_view {
            match_.draw();
        }
    }

    fn draw_theme_config(&self) {
        clear_background(COLOR_BG);

        let (w, h) = (screen_width(), screen_height());
        let center_x = w * 0.5;

        // Top terminal title box
        let title_w = (w * 0.60).clamp(420.0, 580.0);
        let title_h = 68.0;
        let title_x = center_x - title_w * 0.5;
        let title_y = (h * 0.03).max(10.0);

        draw_terminal_box(title_x, title_y, title_w, title_h, "CONFIGURACIÓN Y TEMAS", false);
        draw_centered_text(
            "PERSONALIZACIÓN VISUAL // PALETAS DE COLOR",
            center_x,
            title_y + 45.0,
            14.0,
            TEXT_MUTED,
        );

        let themes = crate::theme::GameTheme::all();
        let preview_theme = match self.theme_selection {
            0 => crate::theme::GameTheme::Retro,
            1 => crate::theme::GameTheme::Arcade,
            2 => crate::theme::GameTheme::Pleasant,
            _ => crate::theme::GameTheme::Meadow,
        };
        let preview_colors = preview_theme.colors();

        let bottom_bar_h = 32.0;
        let bottom_bar_y = h - bottom_bar_h;

        let content_y = title_y + title_h + (h * 0.025).max(10.0);
        let content_h = bottom_bar_y - content_y - 12.0;

        let col_gap = 18.0;
        let margin_x = (w * 0.05).max(16.0);
        let col_w = ((w - margin_x * 2.0 - col_gap) * 0.52).clamp(320.0, 520.0);
        let preview_x = margin_x + col_w + col_gap;
        let preview_w = w - preview_x - margin_x;

        let card_gap = 10.0;
        let card_h = ((content_h - card_gap * 3.0) / 4.0).clamp(65.0, 95.0);

        // Left column: 4 theme cards
        for (i, &theme) in themes.iter().enumerate() {
            let y = content_y + i as f32 * (card_h + card_gap);
            let is_selected = i == self.theme_selection;
            let is_active = theme == self.active_theme;

            if is_selected {
                draw_rectangle(margin_x, y, col_w, card_h, Color::new(0.08, 0.12, 0.16, 0.95));
                draw_rectangle_lines(
                    margin_x + 1.0,
                    y + 1.0,
                    col_w - 2.0,
                    card_h - 2.0,
                    1.0,
                    PANEL_BORDER_FOCUSED,
                );
            }
            draw_terminal_box(margin_x, y, col_w, card_h, "", is_selected);

            // Retro cursor if selected
            if is_selected {
                draw_text("▶", margin_x + 12.0, y + 27.0, 16.0, ACCENT_CYAN);
            }

            // Key badge [ 1 ] .. [ 4 ] - stable offset across all cards
            let key_badge = format!("[ {} ]", i + 1);
            let key_color = if is_selected { ACCENT_CYAN } else { PANEL_BORDER };
            let key_badge_x = margin_x + 32.0;
            draw_badge(&key_badge, key_badge_x, y + 12.0, key_color);

            let key_dims = measure_text(&key_badge, None, 13, 1.0);
            let name_x = key_badge_x + key_dims.width + 16.0 + 10.0;

            let name_color = if is_selected { ACCENT_GOLD } else { WHITE };
            draw_text(&format!("Tema {}", theme.name()), name_x, y + 27.0, 19.0, name_color);

            // Active badge
            if is_active {
                let badge_text = "[ACTIVO]";
                let b_dims = measure_text(badge_text, None, 13, 1.0);
                draw_badge(badge_text, margin_x + col_w - b_dims.width - 28.0, y + 12.0, ACCENT_GREEN);
            }

            // Description
            draw_text(theme.description(), margin_x + 16.0, y + 49.0, 13.0, TEXT_MUTED);

            // Swatches inside card
            let tc = theme.colors();
            let swatch_y = y + card_h - 22.0;
            let sz = 13.0;

            // Head swatch
            draw_rectangle(margin_x + 16.0, swatch_y, sz, sz, tc.head);
            draw_rectangle_lines(margin_x + 16.0, swatch_y, sz, sz, 1.0, PANEL_BORDER);
            draw_text("Cabeza", margin_x + 34.0, swatch_y + 11.0, 12.0, TEXT_MUTED);

            // Body swatch
            draw_rectangle(margin_x + 104.0, swatch_y, sz, sz, tc.body);
            draw_rectangle_lines(margin_x + 104.0, swatch_y, sz, sz, 1.0, PANEL_BORDER);
            draw_text("Cuerpo", margin_x + 122.0, swatch_y + 11.0, 12.0, TEXT_MUTED);

            // Apple swatch
            draw_rectangle(margin_x + 190.0, swatch_y, sz, sz, tc.food);
            draw_rectangle_lines(margin_x + 190.0, swatch_y, sz, sz, 1.0, PANEL_BORDER);
            draw_text("Manzana", margin_x + 208.0, swatch_y + 11.0, 12.0, TEXT_MUTED);
        }

        // Right column: Live preview panel
        draw_terminal_box(preview_x, content_y, preview_w, content_h, "VISTA PREVIA", false);

        let preview_header = format!("TEMA: {}", preview_theme.name().to_uppercase());
        draw_centered_text(
            &preview_header,
            preview_x + preview_w * 0.5,
            content_y + 36.0,
            16.0,
            ACCENT_GOLD,
        );

        // Mini board grid
        let grid_size = 12;
        let avail_board_h = content_h - 100.0;
        let avail_board_w = preview_w - 40.0;
        let cell_size = (avail_board_w.min(avail_board_h) / grid_size as f32).floor().max(12.0);
        let board_w = cell_size * grid_size as f32;
        let board_h = cell_size * grid_size as f32;
        let board_x = preview_x + (preview_w - board_w) * 0.5;
        let board_y = content_y + 52.0;

        draw_rectangle(board_x, board_y, board_w, board_h, Color::new(0.02, 0.02, 0.03, 1.0));
        draw_rectangle_lines(board_x, board_y, board_w, board_h, 1.5, PANEL_BORDER);

        // Grid lines
        for g in 1..grid_size {
            let gx = board_x + g as f32 * cell_size;
            let gy = board_y + g as f32 * cell_size;
            draw_line(gx, board_y, gx, board_y + board_h, 1.0, Color::new(0.10, 0.12, 0.15, 1.0));
            draw_line(board_x, gy, board_x + board_w, gy, 1.0, Color::new(0.10, 0.12, 0.15, 1.0));
        }

        // Food / Apple
        let food_pos = (8, 3);
        crate::render_snake::draw_apple(
            board_x + food_pos.0 as f32 * cell_size,
            board_y + food_pos.1 as f32 * cell_size,
            cell_size,
            preview_theme,
            preview_colors.food,
            1.0,
        );

        // Snake preview (head at (5,5), body 1 at (4,5), corner at (3,5), body 2 at (3,6), tail at (3,7))
        let snake_points = [
            crate::utils::Point { x: 5, y: 5 }, // head (facing Right)
            crate::utils::Point { x: 4, y: 5 }, // body 1 (straight horizontal)
            crate::utils::Point { x: 3, y: 5 }, // body 2 (corner turning down)
            crate::utils::Point { x: 3, y: 6 }, // body 3 (straight vertical)
            crate::utils::Point { x: 3, y: 7 }, // tail
        ];

        let time = get_time();
        let active_bulge_idx = 1 + ((time * 2.5) as usize % 4);

        for (idx, pt) in snake_points.iter().enumerate() {
            let seg_x = board_x + pt.x as f32 * cell_size;
            let seg_y = board_y + pt.y as f32 * cell_size;
            let color = if idx == 0 {
                preview_colors.head
            } else {
                preview_colors.body
            };
            let bulge = if idx == active_bulge_idx { 0.35 } else { 0.0 };
            let head_scale = if idx == 0 && active_bulge_idx == 1 {
                ((time * 5.0).sin().abs() * 0.15) as f32
            } else {
                0.0
            };
            crate::render_snake::draw_connected_segment(
                &snake_points,
                idx,
                crate::utils::FourDirs::Right,
                seg_x,
                seg_y,
                cell_size,
                preview_theme,
                color,
                bulge,
                head_scale,
            );
        }

        // Explanatory note inside preview
        let note_y = board_y + board_h + 18.0;
        if note_y + 14.0 < content_y + content_h {
            draw_centered_text(
                "Vista en vivo con ojos direccionales y onda digestiva",
                preview_x + preview_w * 0.5,
                note_y,
                13.0,
                TEXT_MUTED,
            );
        }

        // Instructions footer in a terminal status bar
        draw_rectangle(0.0, bottom_bar_y, w, bottom_bar_h, PANEL_BG);
        draw_line(0.0, bottom_bar_y, w, bottom_bar_y, 1.5, PANEL_BORDER);
        draw_centered_text(
            "[1-4] / [↑↓] Seleccionar   [ENTER] Guardar   [ESC] Cancelar",
            center_x,
            bottom_bar_y + 20.0,
            14.0,
            TEXT_MUTED,
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

    #[test]
    fn app_menu_champion_helpers_dont_panic() {
        let app = super::App::new();
        let _ = app.has_dqn_champion();
        let _ = app.has_ga_champion();
    }

    #[test]
    fn match_view_slow_toggle() {
        let mut mv = super::MatchView::dqn_versus(crate::view_dqn_versus::DqnVersusView::new(None, None));
        assert!(mv.is_slow());
        mv.toggle_slow();
        assert!(!mv.is_slow());
        mv.toggle_slow();
        assert!(mv.is_slow());
    }
}

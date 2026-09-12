//! DQN-vs-GA cross-match view — [`CrossMatchView`].
//!
//! Loads the GA champion from `best_snake.json` (`pop::Population::load_best_net`)
//! and the DQN champion from `dqn_champion.json` (`champion_store::load`) and
//! runs both as epsilon-zero policies on the shared arena with a cross flavor:
//! titles "GA"/"DQN", distinct accent colors, `record: None`, winner labels
//! "GA WINS!"/"DQN WINS!"/"TIE!" and a "[ESC] Menu" back hint (spec). Every
//! construction is a fresh match.
//!
//! When a required champion file is missing the view MUST state which side has
//! no champion and must not crash — the per-side message logic is the pure
//! [`plan_cross_match`]/[`cross_missing_message`] seam, unit-tested headless.
//! `Esc`-to-menu is owned by the shell, which reads the message state via
//! [`CrossMatchView::message`].

use macroquad::prelude::*;

use crate::champion_store;
use crate::nn::Net;
use crate::pop::Population;
use crate::versus::{BestOfSeries, VersusMatch, Winner};
use crate::view_dqn_train::DQN_CHAMPION_FILE;
use crate::viz_vs::VsFlavor;

/// Message shown when the GA side (`best_snake.json`) has no champion.
pub const GA_MISSING_MESSAGE: &str =
    "Falta el campeón del Algoritmo Genético - entrena primero (necesita best_snake.json)";
/// Message shown when the DQN side (`dqn_champion.json`) has no champion.
pub const DQN_MISSING_MESSAGE: &str =
    "Falta el campeón DQN - entrena DQN primero (necesita dqn_champion.json)";
/// Message shown when neither side has a champion.
pub const BOTH_MISSING_MESSAGE: &str =
    "Faltan ambos campeones - entrena Algoritmo Genético y DQN primero";

/// Which players were available for a cross match, and which side is missing.
pub enum CrossMatchPlayers {
    /// Both champions present: a match can start.
    Ready { ga: Net, dqn: Net },
    /// The GA champion file is missing.
    MissingGa,
    /// The DQN champion file is missing.
    MissingDqn,
    /// Both champion files are missing.
    MissingBoth,
}

impl std::fmt::Debug for CrossMatchPlayers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready { .. } => f.write_str("Ready"),
            Self::MissingGa => f.write_str("MissingGa"),
            Self::MissingDqn => f.write_str("MissingDqn"),
            Self::MissingBoth => f.write_str("MissingBoth"),
        }
    }
}

/// Pure availability decision for a cross match (macroquad-free).
pub fn plan_cross_match(ga: Option<Net>, dqn: Option<Net>) -> CrossMatchPlayers {
    match (ga, dqn) {
        (Some(ga), Some(dqn)) => CrossMatchPlayers::Ready { ga, dqn },
        (Some(_), None) => CrossMatchPlayers::MissingDqn,
        (None, Some(_)) => CrossMatchPlayers::MissingGa,
        (None, None) => CrossMatchPlayers::MissingBoth,
    }
}

/// The message that states which side lacks a champion, or `None` when both are
/// present and a match can run.
pub fn cross_missing_message(players: &CrossMatchPlayers) -> Option<&'static str> {
    match players {
        CrossMatchPlayers::Ready { .. } => None,
        CrossMatchPlayers::MissingGa => Some(GA_MISSING_MESSAGE),
        CrossMatchPlayers::MissingDqn => Some(DQN_MISSING_MESSAGE),
        CrossMatchPlayers::MissingBoth => Some(BOTH_MISSING_MESSAGE),
    }
}

/// Accent color for player 1 (the GA champion) — the arena's GA green.
const GA_COLOR: Color = Color::new(0.3, 0.9, 0.3, 1.0);
/// Accent color for player 2 (the DQN champion) — a distinct blue.
const DQN_COLOR: Color = Color::new(0.35, 0.65, 0.95, 1.0);

/// The cross flavor: "ALGORITMO GENÉTICO" (green, left) vs "DQN" (blue, right), no record
/// semantics, winner/back labels per spec.
fn cross_flavor() -> VsFlavor {
    VsFlavor {
        player1_title: "ALGORITMO GENÉTICO",
        player2_title: "DQN",
        player1_color: GA_COLOR,
        player2_color: DQN_COLOR,
        record: None,
        record_beat_label: "",
        new_record_label: "",
        winner1_label: "ALGORITMO GENÉTICO GANA!",
        winner2_label: "DQN GANA!",
        tie_label: "EMPATE!",
        eliminated1_label: "Algoritmo Genético eliminado!",
        eliminated2_label: "DQN eliminado!",
        back_label: "[ESC] Menú",
        controls_label: "[SPACE] Vel  [ESC] Menú",
        arena_title: "DQN VS ALGORITMO GENÉTICO",
    }
}

/// Type alias for the cross series match builder closure.
type CrossSeries = BestOfSeries<Box<dyn FnMut() -> VersusMatch>>;

/// The renderable DQN-vs-GA cross-match view. See module docs.
pub struct CrossMatchView {
    inner: CrossInner,
}

enum CrossInner {
    /// A champion is missing: message state. The shell shows the message and
    /// owns `Esc`.
    NeedsChampions { message: &'static str },
    /// A running/finished [`BestOfSeries`] between both champions.
    Series {
        series: CrossSeries,
        flavor: VsFlavor,
    },
}

impl CrossMatchView {
    /// Build a fresh cross series from the champions currently on disk: GA from
    /// `best_snake.json`, DQN from `dqn_champion.json`. Every game in the 5-game
    /// series is built fresh from the loaded nets.
    pub fn new() -> Self {
        let ga = Population::load_best_net();
        let dqn = champion_store::load(DQN_CHAMPION_FILE);
        Self::from_nets(ga, dqn)
    }

    /// Same as [`CrossMatchView::new`] from explicit per-side champions.
    pub fn from_nets(ga: Option<Net>, dqn: Option<Net>) -> Self {
        let dqn = dqn.filter(|n| n.matches_arch(&crate::dqn::DQN_ARCH));
        let plan = plan_cross_match(ga, dqn);
        if let CrossMatchPlayers::Ready { ga, dqn } = plan {
            let flavor = cross_flavor();
            let builder: Box<dyn FnMut() -> VersusMatch> =
                Box::new(move || VersusMatch::new_cross(ga.clone(), dqn.clone()));
            return Self {
                inner: CrossInner::Series {
                    series: BestOfSeries::new(builder),
                    flavor,
                },
            };
        }
        // A non-Ready plan is exactly "some side is missing", which always
        // carries a per-side message; `BOTH_MISSING_MESSAGE` is only the
        // defensive tail for a logically impossible Ready here.
        let message = cross_missing_message(&plan).unwrap_or(BOTH_MISSING_MESSAGE);
        Self {
            inner: CrossInner::NeedsChampions { message },
        }
    }

    /// `Some(message)` exactly when a champion file is missing and the view is in
    /// its message state (nothing to tick or draw as a match). The shell reads
    /// this to decide whether `Esc` returns to the menu immediately.
    pub fn message(&self) -> Option<&'static str> {
        match &self.inner {
            CrossInner::NeedsChampions { message } => Some(message),
            CrossInner::Series { .. } => None,
        }
    }

    /// Advance the series one tick. No-op in the message state and once the
    /// series is over.
    pub fn tick(&mut self) {
        if let CrossInner::Series { series, .. } = &mut self.inner {
            series.tick();
        }
    }

    /// Reset the series for a rematch if in active series state.
    pub fn restart(&mut self) {
        if let CrossInner::Series { series, .. } = &mut self.inner {
            series.restart();
        }
    }

    /// True once the whole 5-game series has ended (message state is never
    /// finished).
    pub fn is_finished(&self) -> bool {
        match &self.inner {
            CrossInner::NeedsChampions { .. } => false,
            CrossInner::Series { series, .. } => series.is_series_over(),
        }
    }

    /// The resolved series winner, or `None` while running (or in the message
    /// state).
    pub fn winner(&self) -> Option<Winner> {
        match &self.inner {
            CrossInner::NeedsChampions { .. } => None,
            CrossInner::Series { series, .. } => series.series_winner(),
        }
    }

    /// Draw the arena with the series HUD, or the missing-champion notice.
    pub fn draw(&self) {
        match &self.inner {
            CrossInner::NeedsChampions { message } => {
                crate::ui_kit::draw_missing_champion_notice(
                    screen_width(),
                    screen_height(),
                    message,
                    "[ESC] Menu",
                );
            }
            CrossInner::Series { series, flavor } => {
                crate::viz_vs::VizVS::new().draw_series_match(series, flavor);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cross_flavor, cross_missing_message, plan_cross_match, CrossMatchPlayers, CrossMatchView,
        BOTH_MISSING_MESSAGE, DQN_MISSING_MESSAGE, GA_MISSING_MESSAGE,
    };
    use crate::nn::Net;

    /// `Net` has no `PartialEq`; the planner must hand back exact clones.
    fn nets_eq(a: &Net, b: &Net) -> bool {
        if a.layers.len() != b.layers.len() {
            return false;
        }
        for (la, lb) in a.layers.iter().zip(b.layers.iter()) {
            if la.nodes.len() != lb.nodes.len() {
                return false;
            }
            for (na, nb) in la.nodes.iter().zip(lb.nodes.iter()) {
                if na.len() != nb.len() {
                    return false;
                }
                for (wa, wb) in na.iter().zip(nb.iter()) {
                    if wa != wb {
                        return false;
                    }
                }
            }
        }
        true
    }

    // --- plan_cross_match: pure per-side availability seam (RED-first) ----------

    #[test]
    fn both_champions_present_produce_a_ready_match() {
        let ga = Net::new();
        let dqn = Net::new();
        let plan = plan_cross_match(Some(ga.clone()), Some(dqn.clone()));
        assert_eq!(
            cross_missing_message(&plan),
            None,
            "both champions present means no missing-side message"
        );
        match plan {
            CrossMatchPlayers::Ready { ga: g, dqn: d } => {
                assert!(nets_eq(&g, &ga), "left player must be the GA champion");
                assert!(nets_eq(&d, &dqn), "right player must be the DQN champion");
            }
            other => panic!("expected Ready, got {other:?}"),
        }
    }

    #[test]
    fn missing_dqn_champion_names_the_dqn_side() {
        // Spec scenario: best_snake.json exists but dqn_champion.json does not.
        let ga = Net::new();
        let plan = plan_cross_match(Some(ga), None);
        assert!(
            matches!(plan, CrossMatchPlayers::MissingDqn),
            "the DQN side must be reported missing"
        );
        assert_eq!(
            cross_missing_message(&plan),
            Some(DQN_MISSING_MESSAGE),
            "the message must name the missing side"
        );
    }

    #[test]
    fn missing_ga_champion_names_the_ga_side() {
        let dqn = Net::new();
        let plan = plan_cross_match(None, Some(dqn));
        assert!(matches!(plan, CrossMatchPlayers::MissingGa));
        assert_eq!(cross_missing_message(&plan), Some(GA_MISSING_MESSAGE));
    }

    #[test]
    fn both_champions_missing_reports_both_sides() {
        let plan = plan_cross_match(None, None);
        assert!(matches!(plan, CrossMatchPlayers::MissingBoth));
        assert_eq!(cross_missing_message(&plan), Some(BOTH_MISSING_MESSAGE));
    }

    // --- CrossMatchView composition (behavior pins, no drawing) -----------------

    #[test]
    fn view_with_missing_side_is_a_message_state() {
        let dqn = Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        let view = CrossMatchView::from_nets(None, Some(dqn));
        assert_eq!(view.message(), Some(GA_MISSING_MESSAGE));
        assert!(!view.is_finished(), "message state is never finished");
        assert_eq!(view.winner(), None);
    }

    #[test]
    fn view_with_both_champions_runs_to_a_winner_within_budget() {
        let view = CrossMatchView::from_nets(
        Some(Net::new()),
        Some(Net::new_with_sizes(&crate::dqn::DQN_ARCH)),
        );
        assert_eq!(
            view.message(),
            None,
            "both champions exist, so a match starts"
        );

        let mut view = view;
        let mut ticks = 0;
        while !view.is_finished() && ticks < 10_000 {
            view.tick();
            ticks += 1;
        }
        assert!(
            view.is_finished(),
            "random-brain cross match must finish within the tick budget"
        );
        assert!(view.winner().is_some());
    }

    #[test]
    fn cross_match_rejects_dqn_champion_with_outdated_architecture() {
        let ga = Net::new();
        let old_dqn = Net::new(); // 12x8x4, pre-relative-action DQN arch
        let view = CrossMatchView::from_nets(Some(ga), Some(old_dqn));
        assert_eq!(
            view.message(),
            Some(DQN_MISSING_MESSAGE),
            "old-arch DQN champion must be treated as missing"
        );
    }

    #[test]
    fn cross_flavor_has_expanded_arena_title() {
        let flavor = cross_flavor();
        assert_eq!(flavor.arena_title, "DQN VS ALGORITMO GENÉTICO");
    }
}

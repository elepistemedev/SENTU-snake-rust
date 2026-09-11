//! GA internal versus view — [`GaVersusView`].
//!
//! Standalone GA versus (no training running): loads the GA champions persisted
//! by a previous run via `sim::load_ga_champions` (`sim_metadata.json`
//! best/second-best nets, with a `best_snake.json` fallback for the best) and
//! runs the two brains on the shared arena with the GA-default flavor — the same
//! nets, record semantics ("RECORD TO BEAT"/"NEW!"/best-ever) and strings the
//! legacy in-sim VS rendered (byte-identical look). Every construction is a
//! fresh match (spec). With no best-ever champion anywhere the view enters a
//! message state ("train GA first"); `Esc`-to-menu is owned by the shell, which
//! reads that state via [`GaVersusView::message`].
//!
//! The pure player-selection seam ([`plan_ga_versus`]) decides which nets play
//! given what was loaded and is unit-tested headless; the renderable match
//! ([`VersusMatch`]) performs no input handling here.

use macroquad::prelude::*;

use crate::nn::Net;
use crate::sim::{load_ga_champions, GaChampions};
use crate::versus::{BestOfSeries, VersusMatch, Winner};
use crate::viz_vs::VsFlavor;

/// Message shown when no best-ever GA champion exists anywhere (no
/// `sim_metadata.json` best net and no `best_snake.json` fallback).
pub const GA_CHAMPIONS_MISSING_MESSAGE: &str =
    "Entrena el Algoritmo Genético primero para crear campeones (necesita best_snake.json)";

/// Pure decision of who plays a standalone GA match given the champions loaded
/// off disk. The best-ever net is required; when the metadata carried no second
/// best, player 2 falls back to a clone of the best net — mirroring
/// `Simulation::toggle_vs_mode`'s degenerate fallback when no generation top is
/// available to borrow.
pub enum GaVersusPlayers {
    /// No best-ever champion exists anywhere: the view must show a message.
    Missing,
    /// Best-ever vs second-best (or best-as-second when none was recorded).
    Match { best: Net, second_best: Net },
}

impl std::fmt::Debug for GaVersusPlayers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str("Missing"),
            Self::Match { .. } => f.write_str("Match"),
        }
    }
}

/// Decide who plays, pure and macroquad-free. `best: None` forces the message
/// state even when a second-best is present (a match needs a best-ever brain).
pub fn plan_ga_versus(champions: GaChampions) -> GaVersusPlayers {
    match champions.best {
        None => GaVersusPlayers::Missing,
        Some(best) => GaVersusPlayers::Match {
            second_best: champions.second_best.unwrap_or_else(|| best.clone()),
            best,
        },
    }
}

/// The renderable GA-internal versus view. See module docs.
pub struct GaVersusView {
    inner: GaVersusInner,
}

/// Type alias for the GA series match builder closure.
type GaSeries = BestOfSeries<Box<dyn FnMut() -> VersusMatch>>;

enum GaVersusInner {
    /// No champion: message state. The shell shows the message and owns `Esc`.
    NeedsChampions,
    /// A running/finished [`BestOfSeries`] between the loaded champions.
    Series {
        series: GaSeries,
        flavor: VsFlavor,
    },
}

impl GaVersusView {
    /// Build a fresh standalone match from the champions currently on disk
    /// (`sim_metadata.json` best/second-best, `best_snake.json` fallback).
    /// Every construction is a fresh arena state (spec: versus-from-menu
    /// constructs a fresh match every time).
    pub fn new() -> Self {
        Self::from_champions(load_ga_champions())
    }

    /// Same as [`GaVersusView::new`] from an explicit champion set.
    pub fn from_champions(champions: GaChampions) -> Self {
        let record = champions.record;
        match plan_ga_versus(champions) {
            GaVersusPlayers::Missing => Self {
                inner: GaVersusInner::NeedsChampions,
            },
            GaVersusPlayers::Match { best, second_best } => {
                let flavor = VsFlavor::ga_default(record);
                let builder: Box<dyn FnMut() -> VersusMatch> = {
                    let best = best;
                    let second_best = second_best;
                    Box::new(move || VersusMatch::new(best.clone(), second_best.clone()))
                };
                Self {
                    inner: GaVersusInner::Series {
                        series: BestOfSeries::new(builder),
                        flavor,
                    },
                }
            }
        }
    }

    /// `Some(message)` exactly when no champions exist and the view is in its
    /// message state (nothing to tick or draw as a match). The shell reads this
    /// to decide whether `Esc` returns to the menu immediately.
    pub fn message(&self) -> Option<&'static str> {
        match &self.inner {
            GaVersusInner::NeedsChampions => Some(GA_CHAMPIONS_MISSING_MESSAGE),
            GaVersusInner::Series { .. } => None,
        }
    }

    /// Advance the series one tick. No-op in the message state and once the
    /// series is over.
    pub fn tick(&mut self) {
        if let GaVersusInner::Series { series, .. } = &mut self.inner {
            series.tick();
        }
    }

    /// Reset the series for a rematch if in active series state.
    pub fn restart(&mut self) {
        if let GaVersusInner::Series { series, .. } = &mut self.inner {
            series.restart();
        }
    }

    /// True once the whole 5-game series has ended (message state is never
    /// finished).
    pub fn is_finished(&self) -> bool {
        match &self.inner {
            GaVersusInner::NeedsChampions => false,
            GaVersusInner::Series { series, .. } => series.is_series_over(),
        }
    }

    /// The resolved series winner, or `None` while running (or in the message
    /// state).
    pub fn winner(&self) -> Option<Winner> {
        match &self.inner {
            GaVersusInner::NeedsChampions => None,
            GaVersusInner::Series { series, .. } => series.series_winner(),
        }
    }

    /// Draw the arena with the series HUD, or the "train GA first" notice.
    pub fn draw(&self) {
        match &self.inner {
            GaVersusInner::NeedsChampions => {
                crate::ui_kit::draw_missing_champion_notice(
                    screen_width(),
                    screen_height(),
                    GA_CHAMPIONS_MISSING_MESSAGE,
                    "[ESC] Menu",
                );
            }
            GaVersusInner::Series { series, flavor } => {
                crate::viz_vs::VizVS::new().draw_series_match(series, flavor);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{plan_ga_versus, GaVersusPlayers, GaVersusView, GA_CHAMPIONS_MISSING_MESSAGE};
    use crate::nn::Net;
    use crate::sim::GaChampions;

    /// `Net` has no `PartialEq`; the planner must hand back exact clones of the
    /// passed nets, so compare exact layer weights.
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

    fn champs(best: Option<Net>, second_best: Option<Net>) -> GaChampions {
        GaChampions {
            best,
            second_best,
            record: 42,
        }
    }

    // --- plan_ga_versus: pure player selection (RED-first seam) -----------------

    #[test]
    fn missing_best_net_means_no_match_even_with_second_best_present() {
        let second = Net::new();
        let plan = plan_ga_versus(champs(None, Some(second)));
        assert!(
            matches!(plan, GaVersusPlayers::Missing),
            "a missing best-ever net must force the message state"
        );
    }

    #[test]
    fn best_and_second_best_produce_a_match_with_the_exact_nets() {
        let best = Net::new();
        let second = Net::new();
        let plan = plan_ga_versus(champs(Some(best.clone()), Some(second.clone())));
        match plan {
            GaVersusPlayers::Match {
                best: b,
                second_best: s,
            } => {
                assert!(nets_eq(&b, &best), "player 1 must be the best-ever net");
                assert!(nets_eq(&s, &second), "player 2 must be the second-best net");
            }
            other => panic!("expected a Match, got {other:?}"),
        }
    }

    #[test]
    fn missing_second_best_falls_back_to_the_best_net_as_player_two() {
        // Mirrors Simulation::toggle_vs_mode's degenerate fallback (no current
        // generation top to borrow when training is not running).
        let best = Net::new();
        let plan = plan_ga_versus(champs(Some(best.clone()), None));
        match plan {
            GaVersusPlayers::Match {
                best: b,
                second_best: s,
            } => {
                assert!(nets_eq(&b, &best));
                assert!(
                    nets_eq(&s, &best),
                    "second best must fall back to a clone of the best net"
                );
            }
            other => panic!("expected a Match, got {other:?}"),
        }
    }

    // --- GaVersusView composition (behavior pins, no drawing) ------------------

    #[test]
    fn view_without_champions_is_message_state() {
        let view = GaVersusView::from_champions(champs(None, None));
        assert_eq!(view.message(), Some(GA_CHAMPIONS_MISSING_MESSAGE));
        assert!(!view.is_finished(), "message state is never finished");
        assert_eq!(view.winner(), None);
    }

    #[test]
    fn view_with_champions_runs_to_a_winner_within_budget() {
        let view = GaVersusView::from_champions(champs(Some(Net::new()), Some(Net::new())));
        assert_eq!(view.message(), None, "champions exist, so a match starts");

        let mut view = view;
        let mut ticks = 0;
        while !view.is_finished() && ticks < 10_000 {
            view.tick();
            ticks += 1;
        }
        assert!(
            view.is_finished(),
            "random-brain GA match must finish within the tick budget"
        );
        assert!(view.winner().is_some());
    }
}

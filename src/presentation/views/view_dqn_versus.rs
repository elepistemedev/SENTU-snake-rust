//! DQN internal versus view — [`DqnVersusView`].
//!
//! Pits the session DQN champion (q-network snapshot; epsilon-zero in the
//! arena because every arena brain argmaxes) against the live current policy
//! on the shared `viz_vs` arena (AD-4). If no live trainer exists yet the view
//! falls back to a fresh greedy agent and labels player 2 "CURRENT (fresh)".
//! With no champion at all the view enters a message state; `Esc`-to-menu is
//! owned by the shell, which reads that state via [`DqnVersusView::message`].
//!
//! The pure player-selection seam ([`plan_dqn_versus`]) decides which nets are
//! picked given trainer/champion presence and is unit-tested headless; the
//! renderable composition ([`VersusMatch`]) needs no input handling here.

use macroquad::prelude::*;

use crate::dqn::DQNAgent;
use crate::nn::Net;
use crate::versus::{BestOfSeries, VersusMatch, Winner};
use crate::viz_vs::VsFlavor;

/// Accent color for player 1 (the champion) — matches the GA arena's green.
const CHAMPION_COLOR: Color = Color::new(0.3, 0.9, 0.3, 1.0);
/// Accent color for player 2 (the live/current policy) — matches the GA
/// arena's red.
const CURRENT_COLOR: Color = Color::new(0.9, 0.3, 0.3, 1.0);

/// Message shown when no DQN champion exists yet.
pub const CHAMPION_MISSING_MESSAGE: &str = "Train DQN first to create a champion";

/// Pure decision of whom the DQN versus arena pits against whom, given what the
/// caller has on hand (AD-4). Returns owned nets only when a match can start;
/// `MissingChampion` means the view must show a message instead.
pub enum DqnVersusPlayers {
    /// No champion snapshot exists anywhere (fresh session, nothing recorded,
    /// nothing on disk): a match cannot start.
    MissingChampion,
    /// Champion vs the given live policy (a paused trainer's q_network).
    ChampionVsLive {
        champion: Net,
        live: Net,
    },
    /// Champion vs a fresh greedy agent: no live policy exists yet.
    ChampionVsFresh {
        champion: Net,
    },
}

impl std::fmt::Debug for DqnVersusPlayers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingChampion => f.write_str("MissingChampion"),
            Self::ChampionVsLive { .. } => f.write_str("ChampionVsLive"),
            Self::ChampionVsFresh { .. } => f.write_str("ChampionVsFresh"),
        }
    }
}

/// Decide who plays, pure and macroquad-free. The champion must always exist
/// for a match; the live policy is optional and its absence selects the fresh
/// greedy-agent fallback (the fallback net itself is allocated by the caller,
/// keeping this function deterministic).
pub fn plan_dqn_versus(champion: Option<&Net>, live: Option<&Net>) -> DqnVersusPlayers {
    match (champion, live) {
        (None, _) => DqnVersusPlayers::MissingChampion,
        (Some(c), Some(l)) => DqnVersusPlayers::ChampionVsLive {
            champion: c.clone(),
            live: l.clone(),
        },
        (Some(c), None) => DqnVersusPlayers::ChampionVsFresh {
            champion: c.clone(),
        },
    }
}

/// The renderable DQN-internal versus view. See module docs.
pub struct DqnVersusView {
    inner: DqnVersusInner,
}

/// Type alias for the DQN series match builder closure.
type DqnSeries = BestOfSeries<Box<dyn FnMut() -> VersusMatch>>;

enum DqnVersusInner {
    /// No champion: message state. The shell shows the message and owns `Esc`.
    NeedsChampion,
    /// A running/finished [`BestOfSeries`]; `live_is_fresh` records whether
    /// player 2 is the fresh greedy fallback (for labeling).
    Series {
        series: DqnSeries,
        flavor: VsFlavor,
        live_is_fresh: bool,
    },
}

/// The DQN internal flavor: champion (green, left) vs current policy (red,
/// right), no record semantics, `[ESC] Menu` back hint.
fn dqn_flavor(live_is_fresh: bool) -> VsFlavor {
    VsFlavor {
        player1_title: "CHAMPION",
        player2_title: if live_is_fresh { "CURRENT (fresh)" } else { "CURRENT" },
        player1_color: CHAMPION_COLOR,
        player2_color: CURRENT_COLOR,
        record: None,
        record_beat_label: "",
        new_record_label: "",
        winner1_label: "CHAMPION WINS!",
        winner2_label: "CURRENT WINS!",
        tie_label: "TIE!",
        eliminated1_label: "Champion eliminated!",
        eliminated2_label: if live_is_fresh {
            "Current (fresh) eliminated!"
        } else {
            "Current eliminated!"
        },
        back_label: "[ESC] Menu",
        controls_label: "[SPACE] Vel  [ESC] Menu",
    }
}

impl DqnVersusView {
    /// Compose a fresh series from what the shell has on hand: the DQN champion
    /// (`None` when nothing was recorded/persisted yet) and the live current
    /// policy from a paused trainer (`None` → fresh greedy fallback, labeled
    /// "CURRENT (fresh)"). Every game in the 5-game series is built fresh.
    pub fn new(champion: Option<Net>, live: Option<Net>) -> Self {
        match plan_dqn_versus(champion.as_ref(), live.as_ref()) {
            DqnVersusPlayers::MissingChampion => Self {
                inner: DqnVersusInner::NeedsChampion,
            },
            DqnVersusPlayers::ChampionVsLive { champion, live } => {
                let flavor = dqn_flavor(false);
                let builder: Box<dyn FnMut() -> VersusMatch> =
                    Box::new(move || VersusMatch::new_relative(champion.clone(), live.clone()));
                Self {
                    inner: DqnVersusInner::Series {
                        series: BestOfSeries::new(builder),
                        flavor,
                        live_is_fresh: false,
                    },
                }
            }
            DqnVersusPlayers::ChampionVsFresh { champion } => {
                let flavor = dqn_flavor(true);
                let builder: Box<dyn FnMut() -> VersusMatch> = Box::new(move || {
                    let fresh_net = DQNAgent::new().q_network;
                    VersusMatch::new_relative(champion.clone(), fresh_net)
                });
                Self {
                    inner: DqnVersusInner::Series {
                        series: BestOfSeries::new(builder),
                        flavor,
                        live_is_fresh: true,
                    },
                }
            }
        }
    }

    /// `Some(message)` exactly when the champion is missing and the view is in
    /// its message state (nothing to tick or draw as a match). The shell reads
    /// this to decide whether `Esc` returns to the menu immediately.
    pub fn message(&self) -> Option<&'static str> {
        match &self.inner {
            DqnVersusInner::NeedsChampion => Some(CHAMPION_MISSING_MESSAGE),
            DqnVersusInner::Series { .. } => None,
        }
    }

    /// True when player 2 is the fresh greedy fallback ("CURRENT (fresh)").
    pub fn live_is_fresh(&self) -> bool {
        match &self.inner {
            DqnVersusInner::NeedsChampion => false,
            DqnVersusInner::Series { live_is_fresh, .. } => *live_is_fresh,
        }
    }

    /// Advance the series one tick. No-op in the message state and once the
    /// series is over.
    pub fn tick(&mut self) {
        if let DqnVersusInner::Series { series, .. } = &mut self.inner {
            series.tick();
        }
    }

    /// Reset the series for a rematch if in active series state.
    pub fn restart(&mut self) {
        if let DqnVersusInner::Series { series, .. } = &mut self.inner {
            series.restart();
        }
    }

    /// True once the whole 5-game series has ended (message state is never
    /// finished).
    pub fn is_finished(&self) -> bool {
        match &self.inner {
            DqnVersusInner::NeedsChampion => false,
            DqnVersusInner::Series { series, .. } => series.is_series_over(),
        }
    }

    /// The resolved series winner, or `None` while running (or in the message
    /// state).
    pub fn winner(&self) -> Option<Winner> {
        match &self.inner {
            DqnVersusInner::NeedsChampion => None,
            DqnVersusInner::Series { series, .. } => series.series_winner(),
        }
    }

    /// Draw the arena with the series HUD, or the "train DQN first" notice.
    pub fn draw(&self) {
        match &self.inner {
            DqnVersusInner::NeedsChampion => {
                crate::ui_kit::draw_missing_champion_notice(
                    screen_width(),
                    screen_height(),
                    CHAMPION_MISSING_MESSAGE,
                    "[ESC] Menu",
                );
            }
            DqnVersusInner::Series { series, flavor, .. } => {
                crate::viz_vs::VizVS::new().draw_series_match(series, flavor);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Net` has no `PartialEq`; the pure planner must hand back exact clones
    /// of the passed nets, so compare exact layer weights.
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

    // --- plan_dqn_versus: pure player selection (RED-first seam) ---------------

    #[test]
    fn plan_requires_champion_even_when_live_policy_exists() {
        let live = Net::new();
        let plan = plan_dqn_versus(None, Some(&live));
        assert!(
            matches!(plan, DqnVersusPlayers::MissingChampion),
            "a missing champion must force the message state, live policy or not"
        );
    }

    #[test]
    fn plan_pits_champion_against_live_policy_when_both_present() {
        let champion = Net::new();
        let live = Net::new();
        let plan = plan_dqn_versus(Some(&champion), Some(&live));
        match plan {
            DqnVersusPlayers::ChampionVsLive { champion: c, live: l } => {
                assert!(nets_eq(&c, &champion), "left net must be the champion");
                assert!(nets_eq(&l, &live), "right net must be the live policy");
            }
            other => panic!("expected ChampionVsLive, got {other:?}"),
        }
    }

    #[test]
    fn plan_falls_back_to_fresh_greedy_agent_when_live_policy_is_absent() {
        let champion = Net::new();
        let plan = plan_dqn_versus(Some(&champion), None);
        match plan {
            DqnVersusPlayers::ChampionVsFresh { champion: c } => {
                assert!(
                    nets_eq(&c, &champion),
                    "left net must be the champion when live policy is absent"
                );
            }
            other => panic!("expected ChampionVsFresh, got {other:?}"),
        }
    }

    // --- DqnVersusView composition (behavior pins, no drawing) -----------------

    #[test]
    fn view_without_champion_is_message_state() {
        let view = DqnVersusView::new(None, None);
        assert_eq!(view.message(), Some(CHAMPION_MISSING_MESSAGE));
        assert!(!view.is_finished(), "message state is never finished");
        assert_eq!(view.winner(), None);
        assert!(!view.live_is_fresh());
    }

    #[test]
    fn view_falls_back_to_fresh_agent_and_runs_to_a_winner() {
        // Both players are relative-action DQN brains (9-in, 3-out).
        let champion = Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        let mut view = DqnVersusView::new(Some(champion), None);
        assert_eq!(view.message(), None, "a champion exists, so a match starts");
        assert!(
            view.live_is_fresh(),
            "no live policy must select the fresh fallback label"
        );

        let mut ticks = 0;
        while !view.is_finished() && ticks < 10_000 {
            view.tick();
            ticks += 1;
        }
        assert!(
            view.is_finished(),
            "random-brain match must finish within the tick budget"
        );
        assert!(view.winner().is_some(), "finished match must resolve a winner");
    }

    #[test]
    fn view_with_live_policy_is_not_fresh_and_runs_to_a_winner() {
        // Both players are relative-action DQN brains (9-in, 3-out).
        let champion = Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        let live = Net::new_with_sizes(&crate::dqn::DQN_ARCH);
        let mut view = DqnVersusView::new(Some(champion), Some(live));
        assert_eq!(view.message(), None);
        assert!(
            !view.live_is_fresh(),
            "a real live policy must not be labeled fresh"
        );

        let mut ticks = 0;
        while !view.is_finished() && ticks < 10_000 {
            view.tick();
            ticks += 1;
        }
        assert!(view.is_finished());
        assert!(view.winner().is_some());
    }
}

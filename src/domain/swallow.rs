//! Snake digestion and food bulge tracking.
//!
//! Pure domain state machine with zero UI/rendering dependencies.
//! Used by [`crate::snake_core::SnakeCore`] and animated by
//! [`crate::render_snake`].

/// Tracks food moving down the snake's digestive tract and head chew animation.
#[derive(Debug, Clone, Default)]
pub struct SwallowTracker {
    /// Active body segment indices containing a swallowed food bulge.
    bulges: Vec<usize>,
    /// Frame counter for head chew expansion (e.g. 6 ticks).
    chew_timer: u8,
}

impl SwallowTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset tracker state.
    pub fn reset(&mut self) {
        self.bulges.clear();
        self.chew_timer = 0;
    }

    /// Call when a food item is eaten.
    pub fn push_eating(&mut self) {
        self.chew_timer = 6;
        self.bulges.push(0);
    }

    /// Advance active bulges down the snake body and evict any exceeding `max_len`.
    pub fn advance(&mut self, max_len: usize) {
        if self.chew_timer > 0 {
            self.chew_timer -= 1;
        }
        for b in self.bulges.iter_mut() {
            *b += 1;
        }
        self.bulges.retain(|&idx| idx < max_len);
    }

    /// Scale multiplier for head during eating/chewing (0.0 to 0.25).
    pub fn head_scale(&self) -> f32 {
        if self.chew_timer > 0 {
            (self.chew_timer as f32 / 6.0) * 0.25
        } else {
            0.0
        }
    }

    /// Bulge expansion for a specific segment index (0.0 to 0.35).
    pub fn bulge_at(&self, index: usize) -> f32 {
        if self.bulges.contains(&index) {
            0.35
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_swallow_tracker_has_no_bulges_and_no_chew() {
        let tracker = SwallowTracker::new();
        assert_eq!(tracker.bulge_at(0), 0.0);
        assert_eq!(tracker.bulge_at(1), 0.0);
        assert_eq!(tracker.head_scale(), 0.0);
    }

    #[test]
    fn eating_triggers_head_chew_and_starts_bulge_at_zero() {
        let mut tracker = SwallowTracker::new();
        tracker.push_eating();
        assert!(tracker.head_scale() > 0.0);
        assert!(tracker.bulge_at(0) > 0.0);
        assert_eq!(tracker.bulge_at(1), 0.0);
    }

    #[test]
    fn advance_shifts_bulge_along_body_and_evicts_at_max_len() {
        let mut tracker = SwallowTracker::new();
        tracker.push_eating();
        assert!(tracker.bulge_at(0) > 0.0);

        // Advance 1 step with body length 3: bulge moves to index 1
        tracker.advance(3);
        assert_eq!(tracker.bulge_at(0), 0.0);
        assert!(tracker.bulge_at(1) > 0.0);
        assert_eq!(tracker.bulge_at(2), 0.0);

        // Advance step 2: bulge moves to index 2
        tracker.advance(3);
        assert_eq!(tracker.bulge_at(1), 0.0);
        assert!(tracker.bulge_at(2) > 0.0);

        // Advance step 3: index 3 >= body length 3, bulge gets evicted
        tracker.advance(3);
        assert_eq!(tracker.bulge_at(2), 0.0);
        assert_eq!(tracker.bulge_at(3), 0.0);
    }

    #[test]
    fn multiple_bulges_can_travel_simultaneously() {
        let mut tracker = SwallowTracker::new();
        tracker.push_eating();
        tracker.advance(5);
        tracker.push_eating();

        // One bulge at index 0 and one at index 1
        assert!(tracker.bulge_at(0) > 0.0);
        assert!(tracker.bulge_at(1) > 0.0);
        assert_eq!(tracker.bulge_at(2), 0.0);
    }
}

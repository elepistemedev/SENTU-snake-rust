//! Unified snake and food visual rendering module.
//!
//! Provides retro flat-square rendering for [`crate::theme::GameTheme::Retro`],
//! and rich vectorized rendering (directional eyes, apple stem/leaf/shine,
//! chew scale and digestion wave bulge) for [`crate::theme::GameTheme::Arcade`]
//! and [`crate::theme::GameTheme::Pleasant`].

use macroquad::prelude::*;
use crate::theme::GameTheme;
use crate::utils::FourDirs;

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

/// Calculated relative eye placement for directional eyes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeOffset {
    pub center_x: f32,
    pub center_y: f32,
    pub radius: f32,
    pub pupil_x: f32,
    pub pupil_y: f32,
    pub pupil_radius: f32,
}

/// Computes the position and pupil bias for two eyes looking in direction `dir`.
pub fn compute_eye_offsets(dir: FourDirs, tile_size: f32) -> (EyeOffset, EyeOffset) {
    let eye_r = tile_size * 0.16;
    let pupil_r = eye_r * 0.55;
    let inset = tile_size * 0.28;
    let front = tile_size * 0.26;
    let pupil_offset = eye_r * 0.40;

    let (e1_pos, e2_pos, p_dx, p_dy) = match dir {
        FourDirs::Top => (
            (inset, front),
            (tile_size - inset, front),
            0.0,
            -pupil_offset,
        ),
        FourDirs::Bottom => (
            (inset, tile_size - front),
            (tile_size - inset, tile_size - front),
            0.0,
            pupil_offset,
        ),
        FourDirs::Left => (
            (front, inset),
            (front, tile_size - inset),
            -pupil_offset,
            0.0,
        ),
        FourDirs::Right => (
            (tile_size - front, inset),
            (tile_size - front, tile_size - inset),
            pupil_offset,
            0.0,
        ),
    };

    (
        EyeOffset {
            center_x: e1_pos.0,
            center_y: e1_pos.1,
            radius: eye_r,
            pupil_x: e1_pos.0 + p_dx,
            pupil_y: e1_pos.1 + p_dy,
            pupil_radius: pupil_r,
        },
        EyeOffset {
            center_x: e2_pos.0,
            center_y: e2_pos.1,
            radius: eye_r,
            pupil_x: e2_pos.0 + p_dx,
            pupil_y: e2_pos.1 + p_dy,
            pupil_radius: pupil_r,
        },
    )
}

/// Draws food: classic solid square for Retro, or an apple with stem, leaf and shine for Arcade/Pleasant.
pub fn draw_apple(x: f32, y: f32, tile_size: f32, theme: GameTheme, color: Color) {
    if theme == GameTheme::Retro {
        draw_rectangle(x + 2.0, y + 2.0, tile_size - 4.0, tile_size - 4.0, color);
        return;
    }

    let cx = x + tile_size * 0.5;
    let cy = y + tile_size * 0.54;
    let r = (tile_size - 4.0) * 0.44;

    // Apple main body
    draw_circle(cx, cy, r, color);

    // Stem (tallo marrón)
    let stem_color = Color::new(0.42, 0.24, 0.12, 1.0);
    draw_line(
        cx,
        cy - r * 0.75,
        cx + tile_size * 0.08,
        y + tile_size * 0.10,
        tile_size * 0.09,
        stem_color,
    );

    // Leaf (hojita verde)
    let leaf_color = Color::new(0.25, 0.85, 0.35, 1.0);
    draw_circle(
        cx + tile_size * 0.16,
        y + tile_size * 0.15,
        tile_size * 0.09,
        leaf_color,
    );

    // Specular shine (brillo de luz)
    let shine_color = Color::new(1.0, 1.0, 1.0, 0.60);
    draw_circle(
        cx - r * 0.35,
        cy - r * 0.35,
        r * 0.26,
        shine_color,
    );
}

/// Draws snake head: classic solid square for Retro, or organic rounded head with directional eyes for Arcade/Pleasant.
pub fn draw_snake_head(
    x: f32,
    y: f32,
    tile_size: f32,
    dir: FourDirs,
    theme: GameTheme,
    color: Color,
    head_scale: f32,
) {
    if theme == GameTheme::Retro {
        draw_rectangle(x + 1.0, y + 1.0, tile_size - 2.0, tile_size - 2.0, color);
        return;
    }

    let size = (tile_size - 2.0) * (1.0 + head_scale);
    let cx = x + tile_size * 0.5;
    let cy = y + tile_size * 0.5;
    let hx = cx - size * 0.5;
    let hy = cy - size * 0.5;

    // Head base (smooth organic circle)
    draw_circle(cx, cy, size * 0.50, color);

    // Directional eyes
    let (e1, e2) = compute_eye_offsets(dir, size);
    for eye in [e1, e2] {
        // Eye sclera (white)
        draw_circle(hx + eye.center_x, hy + eye.center_y, eye.radius, WHITE);
        // Eye pupil (black)
        draw_circle(hx + eye.pupil_x, hy + eye.pupil_y, eye.pupil_radius, BLACK);
        // Eye reflection glint
        draw_circle(
            hx + eye.pupil_x - eye.pupil_radius * 0.3,
            hy + eye.pupil_y - eye.pupil_radius * 0.3,
            eye.pupil_radius * 0.35,
            WHITE,
        );
    }
}

/// Draws snake body segment: classic solid square for Retro, or rounded segment with swallow bulge for Arcade/Pleasant.
pub fn draw_snake_body(
    x: f32,
    y: f32,
    tile_size: f32,
    theme: GameTheme,
    color: Color,
    bulge_scale: f32,
) {
    if theme == GameTheme::Retro {
        draw_rectangle(x + 1.0, y + 1.0, tile_size - 2.0, tile_size - 2.0, color);
        return;
    }

    let size = (tile_size - 2.0) * (1.0 + bulge_scale);
    let cx = x + tile_size * 0.5;
    let cy = y + tile_size * 0.5;

    if bulge_scale > 0.0 {
        // Digestion wave bulge (enlarged with inner digestion highlight)
        draw_circle(cx, cy, size * 0.52, color);
        draw_circle(cx, cy, size * 0.28, Color::new(1.0, 1.0, 1.0, 0.35));
    } else {
        draw_circle(cx, cy, size * 0.48, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::FourDirs;

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

    #[test]
    fn directional_eyes_offset_towards_heading() {
        let tile_size = 20.0;
        let (e1_top, e2_top) = compute_eye_offsets(FourDirs::Top, tile_size);
        let (e1_bottom, e2_bottom) = compute_eye_offsets(FourDirs::Bottom, tile_size);

        // In Top direction, eyes and pupils are higher (smaller y) than Bottom direction
        assert!(e1_top.center_y < e1_bottom.center_y);
        assert!(e2_top.center_y < e2_bottom.center_y);
        assert!(e1_top.pupil_y < e1_bottom.pupil_y);

        let (e1_left, _e2_left) = compute_eye_offsets(FourDirs::Left, tile_size);
        let (e1_right, _e2_right) = compute_eye_offsets(FourDirs::Right, tile_size);
        // In Left direction, eyes are further left (smaller x) than Right direction
        assert!(e1_left.center_x < e1_right.center_x);
        assert!(e1_left.pupil_x < e1_right.pupil_x);
    }
}

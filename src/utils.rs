use macroquad::color::Color;
use rand::Rng;

use crate::*;

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FourDirs {
    #[default]
    Left,
    Right,
    Bottom,
    Top,
}

pub fn map_to_unit_interval(value: f32, range: f32) -> f32 {
    let x_abs = range.abs();
    let clamped_value = value.clamp(-x_abs, x_abs);
    (clamped_value + x_abs) / (2.0 * x_abs)
}

pub fn grid_to_world(x: i32, y: i32, tile_size: f32, scale: f32) -> (f32, f32) {
    (x as f32 * tile_size * scale, y as f32 * tile_size * scale)
}

pub fn color_with_a(color: Color, a: f32) -> Color {
    Color::new(color.r, color.g, color.b, a)
}

pub fn are_colors_equal(c1: Color, c2: Color) -> bool {
    c1.r == c2.r && c1.g == c2.g && c1.b == c2.b
}

impl FourDirs {
    pub fn get_rand_dir() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Bottom,
            _ => Self::Top,
        }
    }

    pub fn get_all_dirs() -> [(i32, i32); 4] {
        [
            Self::Left.value(),
            Self::Right.value(),
            Self::Bottom.value(),
            Self::Top.value(),
        ]
    }

    pub fn get_rand_horizontal() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..2) {
            0 => Self::Left,
            _ => Self::Right,
        }
    }

    pub fn get_rand_vertical() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..2) {
            0 => Self::Top,
            _ => Self::Bottom,
        }
    }

    pub fn value(&self) -> (i32, i32) {
        match self {
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Bottom => (0, 1),
            Self::Top => (0, -1),
        }
    }

    pub fn is_horizontal(&self) -> bool {
        match self {
            FourDirs::Left => true,
            FourDirs::Right => true,
            _ => false,
        }
    }

    pub fn is_vertical(&self) -> bool {
        match self {
            FourDirs::Top => true,
            FourDirs::Bottom => true,
            _ => false,
        }
    }
}

/// Heading-relative frame (forward, left-turn, right-turn) for a given absolute
/// heading. Pure; precomputed table verified by tests.
pub fn relative_frame(heading: FourDirs) -> (FourDirs, FourDirs, FourDirs) {
    match heading {
        FourDirs::Left => (FourDirs::Left, FourDirs::Bottom, FourDirs::Top),
        FourDirs::Right => (FourDirs::Right, FourDirs::Top, FourDirs::Bottom),
        FourDirs::Bottom => (FourDirs::Bottom, FourDirs::Right, FourDirs::Left),
        FourDirs::Top => (FourDirs::Top, FourDirs::Left, FourDirs::Right),
    }
}

/// Map a relative action (0 = forward, 1 = left turn, 2 = right turn) to an
/// absolute direction given the current heading. Pure; never returns a
/// backward direction (guaranteed by construction).
pub fn relative_dir(heading: FourDirs, action: usize) -> FourDirs {
    let (fwd, left, right) = relative_frame(heading);
    match action {
        1 => left,
        2 => right,
        _ => fwd,
    }
}

/// Rotate a 12-input absolute-direction vision (LEFT,RIGHT,BOTTOM,TOP order,
/// 3 features each) into a 9-input heading-relative frame, dropping the
/// backward ray. Pure; no side effects.
pub fn rotate_vision_to_relative(abs: &[f64], heading: FourDirs) -> Vec<f64> {
    debug_assert_eq!(
        abs.len(), 12,
        "rotate_vision_to_relative expects exactly 12 absolute inputs"
    );
    let (fwd, left, right) = relative_frame(heading);
    let group = |d: FourDirs| match d {
        FourDirs::Left => 0,
        FourDirs::Right => 1,
        FourDirs::Bottom => 2,
        FourDirs::Top => 3,
    };
    let mut rel = Vec::with_capacity(9);
    for d in [fwd, left, right] {
        let base = group(d) * 3;
        rel.push(abs[base]);
        rel.push(abs[base + 1]);
        rel.push(abs[base + 2]);
    }
    rel
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x: rng.gen_range(1..GRID_W - 1),
            y: rng.gen_range(1..GRID_H - 1),
        }
    }
}

impl Into<Point> for (i32, i32) {
    fn into(self) -> Point {
        Point {
            x: self.0,
            y: self.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_frame_matches_hand_verified_table() {
        assert_eq!(relative_frame(FourDirs::Left),  (FourDirs::Left,  FourDirs::Bottom, FourDirs::Top));
        assert_eq!(relative_frame(FourDirs::Right), (FourDirs::Right, FourDirs::Top,    FourDirs::Bottom));
        assert_eq!(relative_frame(FourDirs::Bottom),(FourDirs::Bottom,FourDirs::Right,  FourDirs::Left));
        assert_eq!(relative_frame(FourDirs::Top),   (FourDirs::Top,   FourDirs::Left,   FourDirs::Right));
    }

    #[test]
    fn relative_dir_matches_frame_for_all_12_combinations() {
        for heading in [FourDirs::Left, FourDirs::Right, FourDirs::Bottom, FourDirs::Top] {
            let (f, l, r) = relative_frame(heading);
            assert_eq!(relative_dir(heading, 0), f, "action 0 = forward for {heading:?}");
            assert_eq!(relative_dir(heading, 1), l, "action 1 = left-turn for {heading:?}");
            assert_eq!(relative_dir(heading, 2), r, "action 2 = right-turn for {heading:?}");
        }
    }

    #[test]
    fn relative_dir_never_returns_backward() {
        // For every heading, the backward direction must never be the result of any relative action.
        use std::collections::HashSet;
        for heading in [FourDirs::Left, FourDirs::Right, FourDirs::Bottom, FourDirs::Top] {
            let all_dirs: HashSet<_> = [FourDirs::Left, FourDirs::Right, FourDirs::Bottom, FourDirs::Top].into();
            let (f, l, r) = relative_frame(heading);
            let backward = all_dirs.into_iter().find(|d| *d != f && *d != l && *d != r).unwrap();
            for action in 0..3 {
                let d = relative_dir(heading, action);
                assert_ne!(d, backward, "heading {heading:?} action {action} must not be backward {backward:?}");
            }
        }
    }

    #[test]
    fn rotate_vision_identity_for_heading_top() {
        // Absolute groups: LEFT=(1,0,0), RIGHT=(0,1,0), BOTTOM=(0,0,1), TOP=(1,1,1)
        let abs = vec![
            1.0, 0.0, 0.0, // LEFT
            0.0, 1.0, 0.0, // RIGHT
            0.0, 0.0, 1.0, // BOTTOM
            1.0, 1.0, 1.0, // TOP
        ];
        // Heading Top => frame (Top, Left, Right)
        let rel = rotate_vision_to_relative(&abs, FourDirs::Top);
        assert_eq!(rel.len(), 9);
        assert_eq!(rel, vec![
            1.0, 1.0, 1.0, // forward = TOP
            1.0, 0.0, 0.0, // left    = LEFT
            0.0, 1.0, 0.0, // right   = RIGHT
        ]);
    }

    #[test]
    fn rotate_vision_drops_backward_group() {
        let abs = vec![
            1.0, 0.0, 0.0, // LEFT
            0.0, 1.0, 0.0, // RIGHT
            0.0, 0.0, 1.0, // BOTTOM
            1.0, 1.0, 1.0, // TOP
        ];
        for heading in [FourDirs::Left, FourDirs::Right, FourDirs::Bottom, FourDirs::Top] {
            let rel = rotate_vision_to_relative(&abs, heading);
            assert_eq!(rel.len(), 9, "heading {heading:?} must drop the backward ray");
            // The backward group's data must NOT appear in the output.
            let all_dirs = [FourDirs::Left, FourDirs::Right, FourDirs::Bottom, FourDirs::Top];
            let (f, l, r) = relative_frame(heading);
            let backward = all_dirs.iter().find(|d| **d != f && **d != l && **d != r).unwrap();
            let b_idx = match backward {
                FourDirs::Left => 0,
                FourDirs::Right => 1,
                FourDirs::Bottom => 2,
                FourDirs::Top => 3,
            };
            let backward_triple = &abs[b_idx * 3..b_idx * 3 + 3];
            assert_ne!(&rel[0..3], backward_triple, "backward triple of {heading:?} must not be in forward slot");
            assert_ne!(&rel[3..6], backward_triple, "backward triple of {heading:?} must not be in left slot");
            assert_ne!(&rel[6..9], backward_triple, "backward triple of {heading:?} must not be in right slot");
        }
    }
}

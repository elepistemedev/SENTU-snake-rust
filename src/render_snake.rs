//! Unified snake and food visual rendering module.
//!
//! Provides retro flat-square rendering for [`crate::theme::GameTheme::Retro`],
//! and rich vectorized rendering (directional eyes, apple stem/leaf/shine,
//! chew scale and digestion wave bulge) for [`crate::theme::GameTheme::Arcade`]
//! and [`crate::theme::GameTheme::Pleasant`].

use macroquad::prelude::*;
use crate::theme::GameTheme;
use crate::utils::FourDirs;

// Re-export SwallowTracker from the domain module (Fase 5 clean architecture).
pub use crate::swallow::SwallowTracker;

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

    // Apple outline / shadow if Meadow theme
    if theme == GameTheme::Meadow {
        draw_circle(cx, cy, r + 1.2, Color::new(0.60, 0.10, 0.16, 1.0));
    }

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

use crate::Point;

/// Topological connectivity and shape of a snake body segment in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    /// Snake head pointing in heading direction, with optional neck connection to body[1].
    Head {
        heading: FourDirs,
        neck_dir: Option<FourDirs>,
    },
    /// Snake tail pointing away from the connected body.
    Tail(FourDirs),
    /// Straight segment spanning horizontally between left and right edges.
    StraightHorizontal,
    /// Straight segment spanning vertically between top and bottom edges.
    StraightVertical,
    /// 90-degree corner connecting top and right edges.
    CornerTopRight,
    /// 90-degree corner connecting top and left edges.
    CornerTopLeft,
    /// 90-degree corner connecting bottom and right edges.
    CornerBottomRight,
    /// 90-degree corner connecting bottom and left edges.
    CornerBottomLeft,
}

/// Classifies the segment connection based on its neighbors in the snake body.
pub fn classify_segment(body: &[Point], index: usize, heading: FourDirs) -> SegmentKind {
    if index == 0 {
        let neck_dir = if body.len() > 1 {
            let curr = body[0];
            let next = body[1];
            let dx = next.x - curr.x;
            let dy = next.y - curr.y;
            if dx > 0 {
                Some(FourDirs::Right)
            } else if dx < 0 {
                Some(FourDirs::Left)
            } else if dy > 0 {
                Some(FourDirs::Bottom)
            } else if dy < 0 {
                Some(FourDirs::Top)
            } else {
                None
            }
        } else {
            None
        };
        return SegmentKind::Head {
            heading,
            neck_dir,
        };
    }
    let curr = body[index];
    let prev = body[index - 1];

    if index == body.len() - 1 {
        // Tail: only connected to prev
        let dx = curr.x - prev.x;
        let dy = curr.y - prev.y;
        let tail_dir = if dx > 0 {
            FourDirs::Right
        } else if dx < 0 {
            FourDirs::Left
        } else if dy > 0 {
            FourDirs::Bottom
        } else {
            FourDirs::Top
        };
        return SegmentKind::Tail(tail_dir);
    }

    let next = body[index + 1];
    let d_prev = (prev.x - curr.x, prev.y - curr.y);
    let d_next = (next.x - curr.x, next.y - curr.y);

    let has_top = d_prev == (0, -1) || d_next == (0, -1);
    let has_bottom = d_prev == (0, 1) || d_next == (0, 1);
    let has_left = d_prev == (-1, 0) || d_next == (-1, 0);
    let has_right = d_prev == (1, 0) || d_next == (1, 0);

    if has_left && has_right {
        SegmentKind::StraightHorizontal
    } else if has_top && has_bottom {
        SegmentKind::StraightVertical
    } else if has_top && has_right {
        SegmentKind::CornerTopRight
    } else if has_top && has_left {
        SegmentKind::CornerTopLeft
    } else if has_bottom && has_right {
        SegmentKind::CornerBottomRight
    } else if has_bottom && has_left {
        SegmentKind::CornerBottomLeft
    } else if has_left || has_right {
        SegmentKind::StraightHorizontal
    } else {
        SegmentKind::StraightVertical
    }
}

/// Draws snake segment with continuous edge-to-edge connectivity and curved corners.
pub fn draw_snake_segment(
    x: f32,
    y: f32,
    tile_size: f32,
    kind: SegmentKind,
    theme: GameTheme,
    color: Color,
    bulge_scale: f32,
    head_scale: f32,
) {
    if theme == GameTheme::Retro {
        draw_rectangle(x + 1.0, y + 1.0, tile_size - 2.0, tile_size - 2.0, color);
        return;
    }

    let cx = x + tile_size * 0.5;
    let cy = y + tile_size * 0.5;
    let outline = theme.colors().outline;
    let contour_w = 2.0;

    match kind {
        SegmentKind::Head { heading, neck_dir } => {
            let size = (tile_size - 2.0) * (1.0 + head_scale);
            let half = size * 0.5;
            let hx = cx - half;
            let hy = cy - half;

            let neck_w = tile_size * 0.74;
            let half_neck = neck_w * 0.5;

            // 1. Draw outline for head and neck
            if let Some(ol) = outline {
                draw_circle(cx, cy, half + 1.5, ol);
                if let Some(nd) = neck_dir {
                    match nd {
                        FourDirs::Left => {
                            draw_line(x - 0.5, cy - half_neck, cx, cy - half_neck, contour_w, ol);
                            draw_line(x - 0.5, cy + half_neck, cx, cy + half_neck, contour_w, ol);
                        }
                        FourDirs::Right => {
                            draw_line(cx, cy - half_neck, x + tile_size + 0.5, cy - half_neck, contour_w, ol);
                            draw_line(cx, cy + half_neck, x + tile_size + 0.5, cy + half_neck, contour_w, ol);
                        }
                        FourDirs::Top => {
                            draw_line(cx - half_neck, y - 0.5, cx - half_neck, cy, contour_w, ol);
                            draw_line(cx + half_neck, y - 0.5, cx + half_neck, cy, contour_w, ol);
                        }
                        FourDirs::Bottom => {
                            draw_line(cx - half_neck, cy, cx - half_neck, y + tile_size + 0.5, contour_w, ol);
                            draw_line(cx + half_neck, cy, cx + half_neck, y + tile_size + 0.5, contour_w, ol);
                        }
                    }
                }
            }

            // 2. Draw neck fill flush to adjacent edge and into the head center (covers any inner outline cut)
            if let Some(nd) = neck_dir {
                match nd {
                    FourDirs::Left => draw_rectangle(x - 0.5, cy - half_neck, cx - x + 0.5, neck_w, color),
                    FourDirs::Right => draw_rectangle(cx, cy - half_neck, x + tile_size - cx + 0.5, neck_w, color),
                    FourDirs::Top => draw_rectangle(cx - half_neck, y - 0.5, neck_w, cy - y + 0.5, color),
                    FourDirs::Bottom => draw_rectangle(cx - half_neck, cy, neck_w, y + tile_size - cy + 0.5, color),
                }
            }

            // 3. Draw head circular body (fills head and seamlessly welds with neck)
            draw_circle(cx, cy, half, color);

            // 4. Directional eyes
            let (e1, e2) = compute_eye_offsets(heading, size);
            for eye in [e1, e2] {
                draw_circle(hx + eye.center_x, hy + eye.center_y, eye.radius, WHITE);
                draw_circle(hx + eye.pupil_x, hy + eye.pupil_y, eye.pupil_radius, BLACK);
                draw_circle(
                    hx + eye.pupil_x - eye.pupil_radius * 0.3,
                    hy + eye.pupil_y - eye.pupil_radius * 0.3,
                    eye.pupil_radius * 0.35,
                    WHITE,
                );
            }
        }
        SegmentKind::Tail(tail_dir) => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;

            // 1. Draw outline first: cap circle and side lines
            if let Some(ol) = outline {
                draw_circle(cx, cy, half_t + 1.0, ol);
                match tail_dir {
                    FourDirs::Right => {
                        draw_line(x - 0.5, cy - half_t, cx, cy - half_t, contour_w, ol);
                        draw_line(x - 0.5, cy + half_t, cx, cy + half_t, contour_w, ol);
                    }
                    FourDirs::Left => {
                        draw_line(cx, cy - half_t, x + tile_size + 0.5, cy - half_t, contour_w, ol);
                        draw_line(cx, cy + half_t, x + tile_size + 0.5, cy + half_t, contour_w, ol);
                    }
                    FourDirs::Bottom => {
                        draw_line(cx - half_t, y - 0.5, cx - half_t, cy, contour_w, ol);
                        draw_line(cx + half_t, y - 0.5, cx + half_t, cy, contour_w, ol);
                    }
                    FourDirs::Top => {
                        draw_line(cx - half_t, cy, cx - half_t, y + tile_size + 0.5, contour_w, ol);
                        draw_line(cx + half_t, cy, cx + half_t, y + tile_size + 0.5, contour_w, ol);
                    }
                }
            }

            // 2. Draw body rectangle fill (covers inner half of cap outline)
            match tail_dir {
                FourDirs::Right => draw_rectangle(x - 0.5, cy - half_t, cx - x + 0.5, t, color),
                FourDirs::Left => draw_rectangle(cx, cy - half_t, x + tile_size - cx + 0.5, t, color),
                FourDirs::Bottom => draw_rectangle(cx - half_t, y - 0.5, t, cy - y + 0.5, color),
                FourDirs::Top => draw_rectangle(cx - half_t, cy, t, y + tile_size - cy + 0.5, color),
            }

            // 3. Draw rounded cap fill
            draw_circle(cx, cy, half_t, color);
        }
        SegmentKind::StraightHorizontal => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;
            draw_rectangle(x - 0.5, cy - half_t, tile_size + 1.0, t, color);
            if let Some(ol) = outline {
                draw_line(x - 0.5, cy - half_t, x + tile_size + 0.5, cy - half_t, contour_w, ol);
                draw_line(x - 0.5, cy + half_t, x + tile_size + 0.5, cy + half_t, contour_w, ol);
            }
            if bulge_scale > 0.0 {
                draw_circle(cx, cy, half_t * 1.25, color);
                draw_circle(cx, cy, half_t * 0.65, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
        SegmentKind::StraightVertical => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;
            draw_rectangle(cx - half_t, y - 0.5, t, tile_size + 1.0, color);
            if let Some(ol) = outline {
                draw_line(cx - half_t, y - 0.5, cx - half_t, y + tile_size + 0.5, contour_w, ol);
                draw_line(cx + half_t, y - 0.5, cx + half_t, y + tile_size + 0.5, contour_w, ol);
            }
            if bulge_scale > 0.0 {
                draw_circle(cx, cy, half_t * 1.25, color);
                draw_circle(cx, cy, half_t * 0.65, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
        SegmentKind::CornerTopRight => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;
            draw_rectangle(cx - half_t, y, t, cy - y + half_t, color);
            draw_rectangle(cx - half_t, cy - half_t, x + tile_size - (cx - half_t), t, color);
            draw_circle(cx, cy, half_t, color);

            if let Some(ol) = outline {
                draw_line(cx - half_t, y, cx - half_t, cy + half_t, contour_w, ol);
                draw_line(cx - half_t, cy + half_t, x + tile_size, cy + half_t, contour_w, ol);
                draw_line(cx + half_t, y, cx + half_t, cy - half_t, contour_w, ol);
                draw_line(cx + half_t, cy - half_t, x + tile_size, cy - half_t, contour_w, ol);
            }
            if bulge_scale > 0.0 {
                draw_circle(cx, cy, half_t * 1.25, color);
                draw_circle(cx, cy, half_t * 0.65, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
        SegmentKind::CornerTopLeft => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;
            draw_rectangle(cx - half_t, y, t, cy - y + half_t, color);
            draw_rectangle(x, cy - half_t, cx + half_t - x, t, color);
            draw_circle(cx, cy, half_t, color);

            if let Some(ol) = outline {
                draw_line(cx + half_t, y, cx + half_t, cy + half_t, contour_w, ol);
                draw_line(x, cy + half_t, cx + half_t, cy + half_t, contour_w, ol);
                draw_line(cx - half_t, y, cx - half_t, cy - half_t, contour_w, ol);
                draw_line(x, cy - half_t, cx - half_t, cy - half_t, contour_w, ol);
            }
            if bulge_scale > 0.0 {
                draw_circle(cx, cy, half_t * 1.25, color);
                draw_circle(cx, cy, half_t * 0.65, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
        SegmentKind::CornerBottomRight => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;
            draw_rectangle(cx - half_t, cy - half_t, t, y + tile_size - (cy - half_t), color);
            draw_rectangle(cx - half_t, cy - half_t, x + tile_size - (cx - half_t), t, color);
            draw_circle(cx, cy, half_t, color);

            if let Some(ol) = outline {
                draw_line(cx - half_t, y + tile_size, cx - half_t, cy - half_t, contour_w, ol);
                draw_line(cx - half_t, cy - half_t, x + tile_size, cy - half_t, contour_w, ol);
                draw_line(cx + half_t, y + tile_size, cx + half_t, cy + half_t, contour_w, ol);
                draw_line(cx + half_t, cy + half_t, x + tile_size, cy + half_t, contour_w, ol);
            }
            if bulge_scale > 0.0 {
                draw_circle(cx, cy, half_t * 1.25, color);
                draw_circle(cx, cy, half_t * 0.65, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
        SegmentKind::CornerBottomLeft => {
            let t = tile_size * 0.74 * (1.0 + bulge_scale);
            let half_t = t * 0.5;
            draw_rectangle(cx - half_t, cy - half_t, t, y + tile_size - (cy - half_t), color);
            draw_rectangle(x, cy - half_t, cx + half_t - x, t, color);
            draw_circle(cx, cy, half_t, color);

            if let Some(ol) = outline {
                draw_line(cx + half_t, y + tile_size, cx + half_t, cy - half_t, contour_w, ol);
                draw_line(x, cy - half_t, cx + half_t, cy - half_t, contour_w, ol);
                draw_line(cx - half_t, y + tile_size, cx - half_t, cy + half_t, contour_w, ol);
                draw_line(x, cy + half_t, cx - half_t, cy + half_t, contour_w, ol);
            }
            if bulge_scale > 0.0 {
                draw_circle(cx, cy, half_t * 1.25, color);
                draw_circle(cx, cy, half_t * 0.65, Color::new(1.0, 1.0, 1.0, 0.35));
            }
        }
    }
}

/// Draws one segment of the snake connecting seamlessly with adjacent segments.
pub fn draw_connected_segment(
    body: &[Point],
    index: usize,
    heading: FourDirs,
    x: f32,
    y: f32,
    tile_size: f32,
    theme: GameTheme,
    color: Color,
    bulge_scale: f32,
    head_scale: f32,
) {
    if theme == GameTheme::Retro {
        draw_rectangle(x + 1.0, y + 1.0, tile_size - 2.0, tile_size - 2.0, color);
        return;
    }
    let kind = classify_segment(body, index, heading);
    draw_snake_segment(x, y, tile_size, kind, theme, color, bulge_scale, head_scale);
}

/// Backward-compatible wrapper for drawing isolated head.
pub fn draw_snake_head(
    x: f32,
    y: f32,
    tile_size: f32,
    dir: FourDirs,
    theme: GameTheme,
    color: Color,
    head_scale: f32,
) {
    draw_snake_segment(
        x,
        y,
        tile_size,
        SegmentKind::Head {
            heading: dir,
            neck_dir: None,
        },
        theme,
        color,
        0.0,
        head_scale,
    );
}

/// Backward-compatible wrapper for drawing isolated body.
pub fn draw_snake_body(
    x: f32,
    y: f32,
    tile_size: f32,
    theme: GameTheme,
    color: Color,
    bulge_scale: f32,
) {
    draw_snake_segment(
        x,
        y,
        tile_size,
        SegmentKind::StraightHorizontal,
        theme,
        color,
        bulge_scale,
        0.0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::FourDirs;

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

    #[test]
    fn segment_classification_identifies_straight_and_corners() {
        use crate::Point;

        // Snake shape:
        // (10, 5) [Head Right]
        // (9, 5)  [Straight Horizontal]
        // (8, 5)  [Corner Bottom-Right: connects to (9,5) right and (8,6) bottom]
        // (8, 6)  [Straight Vertical]
        // (8, 7)  [Tail Bottom: points Down]
        let body = vec![
            Point::new(10, 5),
            Point::new(9, 5),
            Point::new(8, 5),
            Point::new(8, 6),
            Point::new(8, 7),
        ];

        assert_eq!(
            classify_segment(&body, 0, FourDirs::Right),
            SegmentKind::Head {
                heading: FourDirs::Right,
                neck_dir: Some(FourDirs::Left),
            }
        );
        assert_eq!(
            classify_segment(&[Point::new(10, 5)], 0, FourDirs::Right),
            SegmentKind::Head {
                heading: FourDirs::Right,
                neck_dir: None,
            }
        );
        assert_eq!(classify_segment(&body, 1, FourDirs::Right), SegmentKind::StraightHorizontal);
        assert_eq!(classify_segment(&body, 2, FourDirs::Right), SegmentKind::CornerBottomRight);
        assert_eq!(classify_segment(&body, 3, FourDirs::Right), SegmentKind::StraightVertical);
        assert_eq!(classify_segment(&body, 4, FourDirs::Right), SegmentKind::Tail(FourDirs::Bottom));
    }
}


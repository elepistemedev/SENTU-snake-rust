//! Shared retro-terminal UI Kit (`src/ui_kit.rs`).
//!
//! Provides atomic retro-terminal primitives (panels, badges, progress bars,
//! responsive charts, centered text, notices) and styling constants.
//!
//! Visual Grammar:
//! - Deep dark background (`COLOR_BG`)
//! - Framed terminal boxes with bracketed titles `[ TITLE ]` and corner accents
//! - High-contrast neon accents (`ACCENT_CYAN`, `ACCENT_GOLD`, `ACCENT_GREEN`, `ACCENT_RED`)
//! - Muted secondary labels (`TEXT_MUTED`)

use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// Base Color Palette (Retro-Terminal / Cyberpunk)
// ---------------------------------------------------------------------------

pub const COLOR_BG: Color = Color::new(0.04, 0.04, 0.06, 1.0);
pub const PANEL_BG: Color = Color::new(0.07, 0.08, 0.10, 0.95);
pub const PANEL_BORDER: Color = Color::new(0.25, 0.30, 0.38, 1.0);
pub const PANEL_BORDER_FOCUSED: Color = Color::new(0.0, 0.85, 0.85, 1.0);
pub const ACCENT_CYAN: Color = Color::new(0.0, 0.90, 0.90, 1.0);
pub const ACCENT_GOLD: Color = Color::new(1.0, 0.80, 0.20, 1.0);
pub const ACCENT_GREEN: Color = Color::new(0.30, 0.90, 0.40, 1.0);
pub const ACCENT_RED: Color = Color::new(0.95, 0.30, 0.30, 1.0);
pub const TEXT_MUTED: Color = Color::new(0.60, 0.65, 0.70, 1.0);

#[inline]
pub fn color_with_a(color: Color, a: f32) -> Color {
    Color::new(color.r, color.g, color.b, a)
}

#[inline]
pub fn are_colors_equal(c1: Color, c2: Color) -> bool {
    c1.r == c2.r && c1.g == c2.g && c1.b == c2.b
}

// ---------------------------------------------------------------------------
// Brand Watermark (ASCII Art)
// ---------------------------------------------------------------------------

pub const BRAND_WATERMARK_LINES: [&str; 2] = [
    "  █▀ █▀▀ █▄░█ ▀█▀ █░█",
    "  ▄█ ██▄ █░▀█ ░█░ █▄█",
];

pub const BRAND_WATERMARK_FONT_SIZE: f32 = 14.0;
pub const BRAND_WATERMARK_LINE_HEIGHT: f32 = 14.0;
pub const BRAND_WATERMARK_MARGIN_X: f32 = 16.0;
pub const BRAND_WATERMARK_MARGIN_BOTTOM: f32 = 10.0;

/// Calculate the (x, y) coordinates for line `line_idx` (0 or 1) of the brand watermark.
#[inline]
pub fn brand_watermark_pos(screen_h: f32, line_idx: usize) -> (f32, f32) {
    let base_y = screen_h - BRAND_WATERMARK_MARGIN_BOTTOM;
    let y = if line_idx == 0 {
        base_y - BRAND_WATERMARK_LINE_HEIGHT
    } else {
        base_y
    };
    (BRAND_WATERMARK_MARGIN_X, y)
}

/// Draw the brand ASCII watermark at the bottom-left corner of the screen.
pub fn draw_brand_watermark() {
    let screen_h = screen_height();
    let color = Color::new(ACCENT_CYAN.r, ACCENT_CYAN.g, ACCENT_CYAN.b, 0.80);
    for (i, line) in BRAND_WATERMARK_LINES.iter().enumerate() {
        let (x, y) = brand_watermark_pos(screen_h, i);
        draw_text(line, x, y, BRAND_WATERMARK_FONT_SIZE, color);
    }
}

// ---------------------------------------------------------------------------
// Font Management
// ---------------------------------------------------------------------------

/// Embedded TTF font bytes supporting full Unicode block elements, arrows, and accented characters.
pub const EMBEDDED_FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/font.ttf");

/// Initialize and set the global default font for Macroquad.
/// Replaces the default ASCII-only ProggyClean font with our embedded Unicode monospace font.
pub fn init_default_font() -> Result<(), macroquad::Error> {
    let font = macroquad::text::load_ttf_font_from_bytes(EMBEDDED_FONT_BYTES)?;
    macroquad::text::set_default_font(font);
    Ok(())
}


// ---------------------------------------------------------------------------
// Pure Calculation Helpers
// ---------------------------------------------------------------------------

/// Clamp a progress bar fraction to [0.0, 1.0].
#[inline]
pub fn clamp_fraction(val: f32) -> f32 {
    val.clamp(0.0, 1.0)
}

/// Calculate dynamic bar slot width for responsive charts.
///
/// When data count is below `max_cap`, slots maintain a stable width based on `max_cap`
/// so that few bars do not stretch absurdly across the entire screen. When data count
/// reaches or exceeds `max_cap`, slots distribute evenly across `total_w`.
#[inline]
pub fn calculate_chart_slot_width(total_w: f32, count: usize, max_cap: usize) -> f32 {
    if total_w <= 0.0 {
        return 0.0;
    }
    let slots = count.max(max_cap).max(1) as f32;
    total_w / slots
}

// ---------------------------------------------------------------------------
// Drawing Primitives
// ---------------------------------------------------------------------------

/// Helper to draw horizontally centered text at (`cx`, `y`) with exact measurement.
pub fn draw_centered_text(text: &str, cx: f32, y: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, font_size, color);
}

/// Format a terminal box title with brackets if not already bracketed.
/// Avoids heap allocation when `title` already starts with `[`.
#[inline]
pub fn format_box_title<'a>(title: &'a str) -> std::borrow::Cow<'a, str> {
    if title.starts_with('[') {
        std::borrow::Cow::Borrowed(title)
    } else {
        std::borrow::Cow::Owned(format!("[ {} ]", title))
    }
}

/// Draw a retro-terminal panel box with corner accent ticks, background, border,
/// and an optional bracketed title.
pub fn draw_terminal_box(x: f32, y: f32, w: f32, h: f32, title: &str, focused: bool) {
    let border_color = if focused { PANEL_BORDER_FOCUSED } else { PANEL_BORDER };
    let title_color = if focused { ACCENT_CYAN } else { ACCENT_GOLD };

    draw_rectangle(x, y, w, h, PANEL_BG);
    draw_rectangle_lines(x, y, w, h, 2.0, border_color);

    // Subtle corner accents (retro terminal feel)
    let corner = 6.0f32.min(w * 0.1).min(h * 0.1);
    if corner > 1.0 {
        // Top-left
        draw_line(x - 1.0, y, x + corner, y, 3.0, border_color);
        draw_line(x, y - 1.0, x, y + corner, 3.0, border_color);
        // Top-right
        draw_line(x + w - corner, y, x + w + 1.0, y, 3.0, border_color);
        draw_line(x + w, y - 1.0, x + w, y + corner, 3.0, border_color);
        // Bottom-left
        draw_line(x - 1.0, y + h, x + corner, y + h, 3.0, border_color);
        draw_line(x, y + h - corner, x, y + h + 1.0, 3.0, border_color);
        // Bottom-right
        draw_line(x + w - corner, y + h, x + w + 1.0, y + h, 3.0, border_color);
        draw_line(x + w, y + h - corner, x + w, y + h + 1.0, 3.0, border_color);
    }

    if !title.is_empty() {
        let formatted = format_box_title(title);
        draw_text(&formatted, x + 16.0, y + 24.0, 20.0, title_color);
    }
}

/// Draw a compact status or key badge with dark tinted background and colored border.
pub fn draw_badge(text: &str, x: f32, y: f32, badge_color: Color) {
    let font_size = 13.0;
    let dims = measure_text(text, None, font_size as u16, 1.0);
    let pad_x = 8.0;
    let badge_w = dims.width + pad_x * 2.0;
    let badge_h = 20.0;

    let bg = Color::new(
        badge_color.r * 0.15,
        badge_color.g * 0.15,
        badge_color.b * 0.15,
        0.85,
    );
    draw_rectangle(x, y, badge_w, badge_h, bg);
    draw_rectangle_lines(x, y, badge_w, badge_h, 1.5, badge_color);
    draw_text(text, x + pad_x, y + 14.5, font_size, badge_color);
}

/// Draw a progress bar with border, fill fraction, and centered percentage.
/// If `label` is provided, renders label above the progress track.
pub fn draw_progress_bar(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fraction: f32,
    label: &str,
    fill_color: Color,
) {
    let clamped = clamp_fraction(fraction);
    let (track_y, track_h) = if label.is_empty() {
        (y, h)
    } else {
        draw_text(label, x + 2.0, y + 14.0, 14.0, TEXT_MUTED);
        (y + 20.0, (h - 22.0).max(12.0))
    };

    // Track background
    draw_rectangle(x, track_y, w, track_h, PANEL_BG);

    // Fill
    if clamped > 0.0 {
        draw_rectangle(x, track_y, w * clamped, track_h, fill_color);
    }

    // Border
    draw_rectangle_lines(x, track_y, w, track_h, 2.0, PANEL_BORDER);

    // Exact percentage centered
    let pct_str = format!("{:.0}%", clamped * 100.0);
    let font_size = (track_h * 0.55).clamp(12.0, 16.0);
    draw_centered_text(
        &pct_str,
        x + w * 0.5,
        track_y + track_h * 0.5 + font_size * 0.35,
        font_size,
        WHITE,
    );
}

/// Draw a responsive bar chart with terminal frame, dynamic slot width, local max scaling,
/// and optional max value label at top right.
pub fn draw_responsive_chart(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    title: &str,
    max_label: &str,
    data: &[f32],
    max_cap: usize,
    bar_color: Color,
) {
    draw_terminal_box(x, y, w, h, title, false);

    if !max_label.is_empty() {
        let dims = measure_text(max_label, None, 14, 1.0);
        draw_text(max_label, x + w - dims.width - 16.0, y + 24.0, 14.0, TEXT_MUTED);
    }

    let chart_x = x + 16.0;
    let chart_y = y + 42.0;
    let chart_w = (w - 32.0).max(10.0);
    let chart_h = (h - 56.0).max(10.0);

    // Baseline reference line
    draw_line(
        chart_x,
        chart_y + chart_h,
        chart_x + chart_w,
        chart_y + chart_h,
        1.0,
        PANEL_BORDER,
    );

    if data.is_empty() {
        draw_centered_text(
            "NO DATA",
            chart_x + chart_w * 0.5,
            chart_y + chart_h * 0.5 + 4.0,
            14.0,
            TEXT_MUTED,
        );
        return;
    }

    let max_val = data.iter().fold(0.0f32, |a, &b| a.max(b)).max(1.0);
    let step = calculate_chart_slot_width(chart_w, data.len(), max_cap);
    let bar_w = (step - 1.0).max(1.0);

    for (i, &val) in data.iter().enumerate() {
        let bh = ((val / max_val) * chart_h).clamp(0.0, chart_h);
        let bx = chart_x + i as f32 * step;
        let by = chart_y + chart_h - bh;
        draw_rectangle(bx, by, bar_w, bh, bar_color);
    }
}

/// Draw a centered missing champion notice screen with terminal styling, title, detail,
/// and exit instruction.
pub fn draw_missing_champion_notice(w: f32, h: f32, title: &str, detail: &str) {
    clear_background(COLOR_BG);

    let box_w = (w * 0.6).clamp(380.0, 560.0);
    let box_h = 200.0;
    let box_x = (w - box_w) * 0.5;
    let box_y = (h - box_h) * 0.5;

    draw_terminal_box(box_x, box_y, box_w, box_h, "AVISO // NOTICE", false);

    // Centered Title
    draw_centered_text(title, w * 0.5, box_y + 75.0, 22.0, ACCENT_GOLD);

    // Centered Detail
    if !detail.is_empty() {
        draw_centered_text(detail, w * 0.5, box_y + 115.0, 16.0, TEXT_MUTED);
    }

    // Centered exit shortcut if detail does not already mention ESC
    let has_esc = detail.to_uppercase().contains("ESC");
    if !has_esc {
        draw_centered_text("[ESC] Menu", w * 0.5, box_y + 155.0, 16.0, ACCENT_CYAN);
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_slot_width_calculation() {
        // When data is small, slots should not stretch absurdly; when full, fill width
        let w = 500.0;
        let slot_w = calculate_chart_slot_width(w, 10, 50);
        assert!(slot_w > 0.0 && slot_w <= 10.0);

        let full_slot = calculate_chart_slot_width(w, 50, 50);
        assert_eq!(full_slot, 10.0);
    }

    #[test]
    fn test_chart_slot_width_edge_cases() {
        // Zero width
        assert_eq!(calculate_chart_slot_width(0.0, 10, 50), 0.0);
        assert_eq!(calculate_chart_slot_width(-100.0, 10, 50), 0.0);

        // Count 0
        assert_eq!(calculate_chart_slot_width(500.0, 0, 50), 10.0);

        // Count > max_cap
        assert_eq!(calculate_chart_slot_width(500.0, 100, 50), 5.0);

        // max_cap 0 and count 0
        assert_eq!(calculate_chart_slot_width(100.0, 0, 0), 100.0);
    }

    #[test]
    fn test_progress_bar_clamp() {
        assert_eq!(clamp_fraction(-0.5), 0.0);
        assert_eq!(clamp_fraction(1.5), 1.0);
        assert_eq!(clamp_fraction(0.75), 0.75);
        assert_eq!(clamp_fraction(0.0), 0.0);
        assert_eq!(clamp_fraction(1.0), 1.0);
        assert_eq!(clamp_fraction(-100.0), 0.0);
        assert_eq!(clamp_fraction(100.0), 1.0);
    }

    #[test]
    fn test_color_constants_pinned() {
        assert_eq!(COLOR_BG, Color::new(0.04, 0.04, 0.06, 1.0));
        assert_eq!(PANEL_BG, Color::new(0.07, 0.08, 0.10, 0.95));
        assert_eq!(PANEL_BORDER, Color::new(0.25, 0.30, 0.38, 1.0));
        assert_eq!(PANEL_BORDER_FOCUSED, Color::new(0.0, 0.85, 0.85, 1.0));
        assert_eq!(ACCENT_CYAN, Color::new(0.0, 0.90, 0.90, 1.0));
        assert_eq!(ACCENT_GOLD, Color::new(1.0, 0.80, 0.20, 1.0));
        assert_eq!(ACCENT_GREEN, Color::new(0.30, 0.90, 0.40, 1.0));
        assert_eq!(ACCENT_RED, Color::new(0.95, 0.30, 0.30, 1.0));
        assert_eq!(TEXT_MUTED, Color::new(0.60, 0.65, 0.70, 1.0));
    }

    #[test]
    fn test_format_box_title_borrowed_vs_owned() {
        use std::borrow::Cow;

        let bracketed = "[ SYSTEM ]";
        match format_box_title(bracketed) {
            Cow::Borrowed(s) => assert_eq!(s, "[ SYSTEM ]"),
            Cow::Owned(_) => panic!("Expected Cow::Borrowed for bracketed title"),
        }

        let unbracketed = "TITLE";
        match format_box_title(unbracketed) {
            Cow::Owned(s) => assert_eq!(s, "[ TITLE ]"),
            Cow::Borrowed(_) => panic!("Expected Cow::Owned for unbracketed title"),
        }
    }

    #[test]
    fn test_brand_watermark_lines_and_content() {
        assert_eq!(BRAND_WATERMARK_LINES.len(), 2);
        assert_eq!(BRAND_WATERMARK_LINES[0], "  █▀ █▀▀ █▄░█ ▀█▀ █░█");
        assert_eq!(BRAND_WATERMARK_LINES[1], "  ▄█ ██▄ █░▀█ ░█░ █▄█");
    }

    #[test]
    fn test_brand_watermark_positioning() {
        let screen_h = 600.0;
        let (x0, y0) = brand_watermark_pos(screen_h, 0);
        let (x1, y1) = brand_watermark_pos(screen_h, 1);

        assert_eq!(x0, BRAND_WATERMARK_MARGIN_X);
        assert_eq!(x1, BRAND_WATERMARK_MARGIN_X);
        assert!(y0 < y1, "Line 0 must be above Line 1");
        assert_eq!(y1 - y0, BRAND_WATERMARK_LINE_HEIGHT);
        assert_eq!(y1, screen_h - BRAND_WATERMARK_MARGIN_BOTTOM);
    }

    #[test]
    fn test_embedded_font_has_required_glyphs() {
        let bytes = include_bytes!("../assets/fonts/font.ttf");
        let font = fontdue::Font::from_bytes(
            bytes.as_slice(),
            fontdue::FontSettings::default(),
        ).expect("Font file must be valid TTF");

        let required_chars = [
            '█', '▀', '▄', '░', // ASCII art blocks
            '▶', '◀', '↑', '↓', // UI indicators
            'á', 'é', 'í', 'ó', 'ú', 'ñ', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Ñ', // Spanish accents
        ];

        for &c in &required_chars {
            let idx = font.lookup_glyph_index(c);
            assert!(
                idx > 0,
                "Font is missing glyph for character '{}' (U+{:04X})",
                c,
                c as u32
            );
        }
    }
}



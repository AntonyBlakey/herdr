//! Colour primitives and the inactive-pane haze blend.

use ratatui::style::Color;

pub(super) type Rgb = (u8, u8, u8);

/// xterm-256 cube component levels.
const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Standard RGB for ANSI indices 0-15. Matches the named-colour constants that
/// `color_to_rgb` already used, so the two paths cannot disagree.
fn ansi_base_rgb(index: u8) -> Rgb {
    match index {
        0 => (0, 0, 0),
        1 => (128, 0, 0),
        2 => (0, 128, 0),
        3 => (128, 128, 0),
        4 => (0, 0, 128),
        5 => (128, 0, 128),
        6 => (0, 128, 128),
        7 => (192, 192, 192),
        8 => (128, 128, 128),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (0, 0, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        _ => (255, 255, 255),
    }
}

/// Resolve an xterm-256 index to RGB. Indices 16-255 are fixed by specification:
/// a 6x6x6 cube, then a greyscale ramp.
fn indexed_to_rgb(index: u8) -> Rgb {
    match index {
        0..=15 => ansi_base_rgb(index),
        16..=231 => {
            let offset = index - 16;
            (
                CUBE_LEVELS[usize::from(offset / 36)],
                CUBE_LEVELS[usize::from((offset % 36) / 6)],
                CUBE_LEVELS[usize::from(offset % 6)],
            )
        }
        _ => {
            let level = 8 + 10 * (index - 232);
            (level, level, level)
        }
    }
}

pub(super) fn color_to_rgb(color: Color) -> Option<Rgb> {
    match color {
        Color::Reset => None,
        Color::Black => Some(ansi_base_rgb(0)),
        Color::Red => Some(ansi_base_rgb(1)),
        Color::Green => Some(ansi_base_rgb(2)),
        Color::Yellow => Some(ansi_base_rgb(3)),
        Color::Blue => Some(ansi_base_rgb(4)),
        Color::Magenta => Some(ansi_base_rgb(5)),
        Color::Cyan => Some(ansi_base_rgb(6)),
        Color::Gray => Some(ansi_base_rgb(7)),
        Color::DarkGray => Some(ansi_base_rgb(8)),
        Color::LightRed => Some(ansi_base_rgb(9)),
        Color::LightGreen => Some(ansi_base_rgb(10)),
        Color::LightYellow => Some(ansi_base_rgb(11)),
        Color::LightBlue => Some(ansi_base_rgb(12)),
        Color::LightMagenta => Some(ansi_base_rgb(13)),
        Color::LightCyan => Some(ansi_base_rgb(14)),
        Color::White => Some(ansi_base_rgb(15)),
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(index) => Some(indexed_to_rgb(index)),
    }
}

pub(super) fn mix_rgb(base: Rgb, target: Rgb, amount: f32) -> Rgb {
    fn channel(base: u8, target: u8, amount: f32) -> u8 {
        (f32::from(base) + (f32::from(target) - f32::from(base)) * amount).round() as u8
    }
    (
        channel(base.0, target.0, amount),
        channel(base.1, target.1, amount),
        channel(base.2, target.2, amount),
    )
}

/// Foreground blends at this fraction of the background ratio, so text recedes
/// with its background while staying readable. Deliberately not configurable.
pub(super) const HAZE_FG_FACTOR: f32 = 0.6;

#[derive(Debug, Clone, Copy)]
pub(super) struct HazeContext {
    pub haze: Rgb,
    pub ratio: f32,
    pub default_fg: Rgb,
    pub default_bg: Rgb,
}

/// The ANSI palette slot a colour occupies, if any. `Rgb` and `Reset` have none.
fn ansi_index(color: Color) -> Option<u8> {
    match color {
        Color::Black => Some(0),
        Color::Red => Some(1),
        Color::Green => Some(2),
        Color::Yellow => Some(3),
        Color::Blue => Some(4),
        Color::Magenta => Some(5),
        Color::Cyan => Some(6),
        Color::Gray => Some(7),
        Color::DarkGray => Some(8),
        Color::LightRed => Some(9),
        Color::LightGreen => Some(10),
        Color::LightYellow => Some(11),
        Color::LightBlue => Some(12),
        Color::LightMagenta => Some(13),
        Color::LightCyan => Some(14),
        Color::White => Some(15),
        Color::Indexed(index) => Some(index),
        Color::Reset | Color::Rgb(..) => None,
    }
}

/// Resolve a cell colour to RGB. The host palette wins wherever it reported a
/// value; otherwise the specification constants apply, and `fallback` covers
/// `Reset` and anything still unresolved.
pub(super) fn resolve(
    color: Color,
    theme: &crate::terminal_theme::TerminalTheme,
    fallback: Rgb,
) -> Rgb {
    if let Some(index) = ansi_index(color) {
        if let Some(reported) = theme.palette[usize::from(index)] {
            return (reported.r, reported.g, reported.b);
        }
    }
    color_to_rgb(color).unwrap_or(fallback)
}

pub(super) fn haze_style(
    style: ratatui::style::Style,
    theme: &crate::terminal_theme::TerminalTheme,
    ctx: HazeContext,
) -> ratatui::style::Style {
    let bg = resolve(style.bg.unwrap_or(Color::Reset), theme, ctx.default_bg);
    let fg = resolve(style.fg.unwrap_or(Color::Reset), theme, ctx.default_fg);
    let bg = mix_rgb(bg, ctx.haze, ctx.ratio);
    let fg = mix_rgb(fg, ctx.haze, ctx.ratio * HAZE_FG_FACTOR);
    style
        .fg(Color::Rgb(fg.0, fg.1, fg.2))
        .bg(Color::Rgb(bg.0, bg.1, bg.2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_theme::{RgbColor, TerminalTheme};
    use ratatui::style::Style;

    #[test]
    fn indexed_cube_resolves_to_specified_levels() {
        // 196 = 16 + 36*5 -> pure red corner of the cube.
        assert_eq!(color_to_rgb(Color::Indexed(196)), Some((255, 0, 0)));
        // 16 is the cube origin, 231 the opposite corner.
        assert_eq!(color_to_rgb(Color::Indexed(16)), Some((0, 0, 0)));
        assert_eq!(color_to_rgb(Color::Indexed(231)), Some((255, 255, 255)));
        // 33 = 16 + 17 -> r=0, g=2, b=5
        assert_eq!(color_to_rgb(Color::Indexed(33)), Some((0, 135, 255)));
    }

    #[test]
    fn indexed_greyscale_ramp_resolves() {
        assert_eq!(color_to_rgb(Color::Indexed(232)), Some((8, 8, 8)));
        assert_eq!(color_to_rgb(Color::Indexed(255)), Some((238, 238, 238)));
    }

    #[test]
    fn indexed_low_range_matches_named_colors() {
        assert_eq!(color_to_rgb(Color::Indexed(1)), color_to_rgb(Color::Red));
        assert_eq!(color_to_rgb(Color::Indexed(15)), color_to_rgb(Color::White));
    }

    #[test]
    fn reset_stays_unresolved() {
        assert_eq!(color_to_rgb(Color::Reset), None);
    }

    #[test]
    fn mix_rgb_endpoints_are_exact() {
        let base = (10, 20, 30);
        let target = (250, 240, 230);
        assert_eq!(mix_rgb(base, target, 0.0), base);
        assert_eq!(mix_rgb(base, target, 1.0), target);
        assert_eq!(mix_rgb((0, 0, 0), (100, 200, 50), 0.5), (50, 100, 25));
    }

    fn theme_with(index: u8, color: Rgb) -> TerminalTheme {
        TerminalTheme::default().with_palette_color(
            index,
            RgbColor {
                r: color.0,
                g: color.1,
                b: color.2,
            },
        )
    }

    #[test]
    fn resolve_prefers_host_palette_over_standard_constants() {
        let theme = theme_with(1, (200, 30, 40));
        // Both the named colour and its index must consult the palette.
        assert_eq!(resolve(Color::Red, &theme, (0, 0, 0)), (200, 30, 40));
        assert_eq!(resolve(Color::Indexed(1), &theme, (0, 0, 0)), (200, 30, 40));
    }

    #[test]
    fn resolve_falls_back_to_specification_values() {
        let theme = TerminalTheme::default();
        // Nothing reported: cube and ramp are still exact.
        assert_eq!(resolve(Color::Indexed(196), &theme, (0, 0, 0)), (255, 0, 0));
        assert_eq!(resolve(Color::Indexed(232), &theme, (0, 0, 0)), (8, 8, 8));
        assert_eq!(resolve(Color::Red, &theme, (0, 0, 0)), (128, 0, 0));
    }

    #[test]
    fn resolve_maps_reset_to_the_supplied_fallback() {
        let theme = TerminalTheme::default();
        assert_eq!(resolve(Color::Reset, &theme, (17, 18, 19)), (17, 18, 19));
    }

    #[test]
    fn resolve_passes_through_true_color() {
        let theme = theme_with(1, (200, 30, 40));
        assert_eq!(resolve(Color::Rgb(1, 2, 3), &theme, (0, 0, 0)), (1, 2, 3));
    }

    #[test]
    fn haze_style_blends_bg_more_than_fg() {
        let theme = TerminalTheme::default();
        let ctx = HazeContext {
            haze: (0, 0, 0),
            ratio: 1.0,
            default_fg: (255, 255, 255),
            default_bg: (255, 255, 255),
        };
        let style = Style::default()
            .fg(Color::Rgb(200, 200, 200))
            .bg(Color::Rgb(100, 100, 100));
        let hazed = haze_style(style, &theme, ctx);

        // ratio 1.0 takes bg fully to the haze colour.
        assert_eq!(hazed.bg, Some(Color::Rgb(0, 0, 0)));
        // fg blends at ratio * HAZE_FG_FACTOR = 0.6, so 200 -> 80.
        assert_eq!(hazed.fg, Some(Color::Rgb(80, 80, 80)));
    }

    #[test]
    fn haze_style_uses_defaults_for_reset_cells() {
        let theme = TerminalTheme::default();
        let ctx = HazeContext {
            haze: (0, 0, 0),
            ratio: 1.0,
            default_fg: (10, 10, 10),
            default_bg: (20, 20, 20),
        };
        // A cell with no explicit colours must still resolve and blend.
        let hazed = haze_style(Style::default(), &theme, ctx);
        assert_eq!(hazed.bg, Some(Color::Rgb(0, 0, 0)));
        assert_eq!(hazed.fg, Some(Color::Rgb(4, 4, 4)));
    }
}

#![allow(dead_code)]
//! Terminal image rendering using half-block characters
//!
//! Uses the Unicode half-block character (▀) to display two vertical
//! pixels per terminal character:
//!   - Foreground color = top pixel
//!   - Background color = bottom pixel
//!
//! This gives 2x vertical resolution compared to a single character.
//!
//! Optimizations:
//!   - Skips redundant ANSI escape codes when colors don't change
//!   - Handles transparency (composites against terminal background)
//!   - Batches output for performance

use super::decoder::{DecodedImage, Pixel};
use crate::output::Output;

const HALF_UPPER: &str = "▀";
const HALF_LOWER: &str = "▄";
const RESET: &str = "\x1b[0m";
const RESET_BG: &str = "\x1b[49m";

#[allow(unused_assignments)]
pub fn render_halfblock(img: &DecodedImage, _out: &Output) {
    let mut buf = String::with_capacity(img.display_width as usize * 64);

    // Skip redundant ANSI color codes when adjacent pixels match
    let mut last_fg: Option<(u8, u8, u8)> = None;
    let mut last_bg: Option<(u8, u8, u8)> = None;

    // Two rows per character cell (▀ = top fg, bottom bg)
    for y in (0..img.display_height).step_by(2) {
        buf.clear();
        buf.push_str("  ");
        last_fg = None;
        last_bg = None;

        for x in 0..img.display_width {
            let top = img.get_pixel(x, y);
            let bot = if y + 1 < img.display_height {
                img.get_pixel(x, y + 1)
            } else {
                Pixel {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                }
            };

            let top_trans = top.is_transparent();
            let bot_trans = bot.is_transparent();

            if top_trans && bot_trans {
                // Both transparent → space
                buf.push_str(RESET);
                buf.push(' ');
                last_fg = None;
                last_bg = None;
            } else if top_trans {
                // Only bottom visible → ▄ with fg=bottom
                let fg = (bot.r, bot.g, bot.b);
                if last_fg != Some(fg) {
                    write_fg(&mut buf, bot.r, bot.g, bot.b);
                    last_fg = Some(fg);
                }
                buf.push_str(RESET_BG);
                last_bg = None;
                buf.push_str(HALF_LOWER);
            } else if bot_trans {
                // Only top visible → ▀ with fg=top
                let fg = (top.r, top.g, top.b);
                if last_fg != Some(fg) {
                    write_fg(&mut buf, top.r, top.g, top.b);
                    last_fg = Some(fg);
                }
                buf.push_str(RESET_BG);
                last_bg = None;
                buf.push_str(HALF_UPPER);
            } else {
                // Both visible → ▀ with fg=top, bg=bottom
                let top_c = composite_for_display(top);
                let bot_c = composite_for_display(bot);

                let fg = (top_c.r, top_c.g, top_c.b);
                let bg = (bot_c.r, bot_c.g, bot_c.b);

                if last_fg != Some(fg) {
                    write_fg(&mut buf, top_c.r, top_c.g, top_c.b);
                    last_fg = Some(fg);
                }
                if last_bg != Some(bg) {
                    write_bg(&mut buf, bot_c.r, bot_c.g, bot_c.b);
                    last_bg = Some(bg);
                }
                buf.push_str(HALF_UPPER);
            }
        }

        buf.push_str(RESET);
        buf.push('\n');
        print!("{}", buf);
    }
}

/// Composites against assumed terminal bg (#1a1a2e)
fn composite_for_display(px: Pixel) -> Pixel {
    if px.a >= 250 {
        return px;
    }
    px.composite_over(26, 26, 46)
}

#[inline]
fn write_fg(buf: &mut String, r: u8, g: u8, b: u8) {
    use std::fmt::Write;
    let _ = write!(buf, "\x1b[38;2;{};{};{}m", r, g, b);
}

#[inline]
fn write_bg(buf: &mut String, r: u8, g: u8, b: u8) {
    use std::fmt::Write;
    let _ = write!(buf, "\x1b[48;2;{};{};{}m", r, g, b);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_opaque() {
        let px = Pixel { r: 255, g: 0, b: 0, a: 255 };
        let result = composite_for_display(px);
        assert_eq!(result.r, 255);
        assert_eq!(result.g, 0);
    }

    #[test]
    fn test_composite_semi_transparent() {
        let px = Pixel { r: 255, g: 255, b: 255, a: 128 };
        let result = composite_for_display(px);
        // Should be lighter than background but not pure white
        assert!(result.r > 100);
        assert!(result.a == 255);
    }
}

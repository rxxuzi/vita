use crossterm::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::output::Output;
use crate::theme::Theme;

pub fn render(content: &str, lang: &str, line_numbers: bool, theme: &Theme, out: &Output) {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ss
        .find_syntax_by_name(lang)
        .or_else(|| ss.find_syntax_by_extension(&lang.to_lowercase()))
        .or_else(|| ss.find_syntax_by_token(&lang.to_lowercase()))
        .or_else(|| {
            let fb = crate::detect::syntax_fallback(lang);
            ss.find_syntax_by_name(fb)
                .or_else(|| ss.find_syntax_by_token(fb))
        })
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let highlight_theme = ts
        .themes
        .get(theme.syntect_theme)
        .or_else(|| ts.themes.get("Monokai Extended"))
        .unwrap_or_else(|| ts.themes.values().next().unwrap());

    let mut h = HighlightLines::new(syntax, highlight_theme);

    let lines: Vec<&str> = LinesWithEndings::from(content).collect();
    let line_count = lines.len();
    let num_width = if line_numbers {
        format!("{}", line_count).len()
    } else {
        0
    };

    for (i, line) in lines.iter().enumerate() {
        if line_numbers {
            out.dim(&format!(" {:>width$} │ ", i + 1, width = num_width), theme.line_number);
        }

        match h.highlight_line(line, &ss) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    let color = syntect_to_crossterm(style);
                    if style.font_style.contains(FontStyle::BOLD) {
                        out.bold_colored(text, color);
                    } else if style.font_style.contains(FontStyle::ITALIC) {
                        out.italic_colored(text, color);
                    } else {
                        out.colored(text, color);
                    }
                }
            }
            Err(_) => print!("{}", line),
        }
    }

    // Ensure final newline
    if !content.ends_with('\n') {
        println!();
    }
}

fn syntect_to_crossterm(style: Style) -> Color {
    Color::Rgb {
        r: style.foreground.r,
        g: style.foreground.g,
        b: style.foreground.b,
    }
}

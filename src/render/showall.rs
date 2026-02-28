//! Show-all renderer: visualizes invisible characters with symbolic replacements.
//! Tab → ⇥, space → ·, CR → ←, LF → ↵, control chars → ^X, NBSP → ⍽,
//! zero-width chars → [U+XXXX]. Always displays line numbers.

use crate::output::Output;
use crate::theme::Theme;
use crossterm::style::Color;

pub fn render(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = format!("{}", lines.len().max(1)).len();

    for (i, line) in lines.iter().enumerate() {
        print_line_number(i + 1, width, theme, out);
        render_line(line, theme, out);

        out.dim("↵", theme.line_number);
        println!();
    }
}

fn print_line_number(num: usize, width: usize, theme: &Theme, out: &Output) {
    let (r, g, b) = rgb(theme.line_number);
    print!(
        "\x1b[38;2;{};{};{}m {:>w$} │ \x1b[0m",
        r,
        g,
        b,
        num,
        w = width
    );
    let _ = out;
}

fn render_line(line: &str, theme: &Theme, out: &Output) {
    let mut col = 0usize;

    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\t' => {
                out.dim("⇥", theme.line_number);
                col += 1;
                let pad = 8 - (col % 8);
                for _ in 0..pad {
                    out.dim(" ", theme.line_number);
                }
                col += pad;
            }
            ' ' => {
                out.dim("·", theme.line_number);
                col += 1;
            }
            '\r' => {
                out.colored("←", theme.alert_warning);
            }
            '\0' => {
                out.colored("^@", theme.alert_warning);
                col += 2;
            }
            '\x01'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' => {
                let symbol = (b'@' + ch as u8) as char;
                out.colored(&format!("^{}", symbol), theme.alert_warning);
                col += 2;
            }
            '\u{00A0}' => {
                out.colored("⍽", theme.alert_warning);
                col += 1;
            }
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {
                out.colored(&format!("[U+{:04X}]", ch as u32), theme.alert_caution);
                col += 8;
            }
            _ => {
                print!("{}", ch);
                col += 1;
            }
        }
    }
}

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (128, 128, 128),
    }
}

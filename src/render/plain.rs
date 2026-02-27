use crate::output::Output;
use crate::theme::Theme;

pub fn render(content: &str, line_numbers: bool, theme: &Theme, _out: &Output) {
    if !line_numbers {
        print!("{}", content);
        if !content.ends_with('\n') {
            println!();
        }
        return;
    }

    let lines: Vec<&str> = content.lines().collect();
    let width = format!("{}", lines.len()).len();

    for (i, line) in lines.iter().enumerate() {
        print!(
            "\x1b[38;2;{};{};{}m {:>w$} │ \x1b[0m{}",
            color_r(theme.line_number),
            color_g(theme.line_number),
            color_b(theme.line_number),
            i + 1,
            line,
            w = width
        );
        println!();
    }
}

fn color_r(c: crossterm::style::Color) -> u8 {
    match c {
        crossterm::style::Color::Rgb { r, .. } => r,
        _ => 128,
    }
}
fn color_g(c: crossterm::style::Color) -> u8 {
    match c {
        crossterm::style::Color::Rgb { g, .. } => g,
        _ => 128,
    }
}
fn color_b(c: crossterm::style::Color) -> u8 {
    match c {
        crossterm::style::Color::Rgb { b, .. } => b,
        _ => 128,
    }
}

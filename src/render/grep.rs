use crate::output::Output;
use crate::theme::Theme;

pub fn render(content: &str, pattern: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();
    let num_width = format!("{}", line_count).len();

    for (i, line) in lines.iter().enumerate() {
        if !line.contains(pattern) {
            continue;
        }

        out.dim(&format!(" {:>width$} │ ", i + 1, width = num_width), theme.line_number);

        let mut rest = *line;
        while let Some(pos) = rest.find(pattern) {
            if pos > 0 {
                out.colored(&rest[..pos], theme.text);
            }
            out.colored_bg(pattern, theme.grep_match_fg, theme.grep_match_bg);
            rest = &rest[pos + pattern.len()..];
        }
        if !rest.is_empty() {
            out.colored(rest, theme.text);
        }
        println!();
    }
}

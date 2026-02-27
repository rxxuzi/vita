//! CSV/TSV renderer with pastel column coloring
//!
//! Each column gets its own pastel color, making it easy to
//! visually track data across rows in wide tables.

use crate::output::Output;
use crate::theme::Theme;
use crossterm::style::Color;

const COLUMN_COLORS: &[(u8, u8, u8)] = &[
    (182, 215, 252), // pastel blue
    (199, 243, 185), // pastel green
    (255, 209, 163), // pastel orange
    (228, 187, 249), // pastel purple
    (255, 179, 186), // pastel pink
    (162, 233, 222), // pastel teal
    (255, 236, 158), // pastel yellow
    (200, 200, 240), // pastel lavender
    (186, 230, 200), // pastel mint
    (252, 196, 218), // pastel rose
    (180, 220, 255), // pastel sky
    (220, 210, 180), // pastel sand
];

pub fn render(content: &str, theme: &Theme, out: &Output) {
    let delimiter = detect_delimiter(content);
    let rows = parse_csv(content, delimiter);

    if rows.is_empty() {
        print!("{}", content);
        return;
    }

    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        print!("{}", content);
        return;
    }

    let mut widths = vec![0usize; col_count];
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Cap column widths to something reasonable
    let max_col_width = ((out.term_width as usize).saturating_sub(col_count * 3 + 4)) / col_count.max(1);
    let max_col_width = max_col_width.max(8).min(40);
    for w in widths.iter_mut() {
        *w = (*w).min(max_col_width);
    }

    let border_color = theme.table_border;

    print_border_top(&widths, col_count, border_color, out);

    for (r, row) in rows.iter().enumerate() {
        print!("  ");
        out.colored("│", border_color);

        for (c, w) in widths.iter().enumerate() {
            let cell = row.get(c).map(|s| s.as_str()).unwrap_or("");
            let truncated = truncate_str(cell, *w);
            let padding = w.saturating_sub(truncated.len());

            let col_color = column_color(c);

            print!(" ");
            if r == 0 {
                out.bold_colored(&truncated, col_color);
            } else {
                out.colored(&truncated, col_color);
            }
            for _ in 0..padding {
                print!(" ");
            }
            print!(" ");
            out.colored("│", border_color);
        }
        println!();

        if r == 0 && rows.len() > 1 {
            print_border_mid(&widths, col_count, border_color, out);
        }
    }

    print_border_bottom(&widths, col_count, border_color, out);

    let data_rows = if rows.len() > 1 { rows.len() - 1 } else { rows.len() };
    out.dim(
        &format!("  {} rows × {} columns\n", data_rows, col_count),
        theme.hr,
    );
}

fn column_color(index: usize) -> Color {
    let (r, g, b) = COLUMN_COLORS[index % COLUMN_COLORS.len()];
    Color::Rgb { r, g, b }
}

fn detect_delimiter(content: &str) -> char {
    let first_lines: Vec<&str> = content.lines().take(5).collect();
    let sample = first_lines.join("\n");

    let tabs = sample.matches('\t').count();
    let commas = sample.matches(',').count();
    let semicolons = sample.matches(';').count();
    let pipes = sample.matches('|').count();

    let max = tabs.max(commas).max(semicolons).max(pipes);
    if max == 0 {
        return ',';
    }
    if max == tabs {
        '\t'
    } else if max == commas {
        ','
    } else if max == pipes {
        '|'
    } else {
        ';'
    }
}

fn parse_csv(content: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = trimmed.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_quotes {
                if ch == '"' {
                    if chars.peek() == Some(&'"') {
                        // Escaped quote
                        current.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    current.push(ch);
                }
            } else if ch == '"' {
                in_quotes = true;
            } else if ch == delimiter {
                fields.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(ch);
            }
        }
        fields.push(current.trim().to_string());
        rows.push(fields);
    }

    rows
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 2 {
        format!("{}…", &s[..max_len - 1])
    } else {
        s[..max_len].to_string()
    }
}

fn print_border_top(widths: &[usize], col_count: usize, color: Color, out: &Output) {
    print!("  ");
    out.colored("┌", color);
    for (i, w) in widths.iter().enumerate() {
        out.colored(&"─".repeat(*w + 2), color);
        out.colored(if i < col_count - 1 { "┬" } else { "┐" }, color);
    }
    println!();
}

fn print_border_mid(widths: &[usize], col_count: usize, color: Color, out: &Output) {
    print!("  ");
    out.colored("├", color);
    for (i, w) in widths.iter().enumerate() {
        out.colored(&"─".repeat(*w + 2), color);
        out.colored(if i < col_count - 1 { "┼" } else { "┤" }, color);
    }
    println!();
}

fn print_border_bottom(widths: &[usize], col_count: usize, color: Color, out: &Output) {
    print!("  ");
    out.colored("└", color);
    for (i, w) in widths.iter().enumerate() {
        out.colored(&"─".repeat(*w + 2), color);
        out.colored(if i < col_count - 1 { "┴" } else { "┘" }, color);
    }
    println!();
}

//! Git blame renderer — grouped annotations with syntax-highlighted code.
//!
//! Runs `git blame --porcelain` and parses the output. Consecutive lines from
//! the same commit show metadata only on the first line; subsequent lines leave
//! the metadata columns blank for a clean, grouped layout.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

use crossterm::style::Color;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style, ThemeSet};
use syntect::parsing::SyntaxSet;

use crate::output::Output;
use crate::theme::Theme;

struct BlameLine {
    hash: String,
    author: String,
    timestamp: i64,
    content: String,
}

pub fn render(
    path: &Path,
    lang: &str,
    head: Option<usize>,
    tail: Option<usize>,
    theme: &Theme,
    out: &Output,
) {
    let cmd = match Command::new("git")
        .args(["blame", "--porcelain"])
        .arg(path)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        Ok(o) => {
            eprintln!(
                "vita: git blame failed: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            );
            return;
        }
        Err(e) => {
            eprintln!("vita: failed to run git: {}", e);
            return;
        }
    };

    let mut lines = parse_porcelain(&cmd);
    if lines.is_empty() {
        return;
    }

    if let Some(n) = head {
        lines.truncate(n);
    } else if let Some(n) = tail {
        let skip = lines.len().saturating_sub(n);
        lines = lines.split_off(skip);
    }

    let max_author = lines.iter().map(|l| l.author.len()).max().unwrap_or(0);
    let line_count = lines.len();
    let num_width = format!("{}", line_count).len();

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
    let mut prev_hash = String::new();

    let dates: Vec<String> = lines
        .iter()
        .map(|l| {
            let time = UNIX_EPOCH + Duration::from_secs(l.timestamp as u64);
            crate::info::format_relative_time(time)
        })
        .collect();
    let max_date = dates.iter().map(|d| d.len()).max().unwrap_or(0);

    for (i, line) in lines.iter().enumerate() {
        let same_commit = line.hash == prev_hash;

        if same_commit {
            let meta_width = 7 + 2 + max_author + 2 + max_date;
            print!("{:width$}", "", width = meta_width);
        } else {
            out.colored(&line.hash, theme.blame_hash);
            print!("  ");
            out.colored(
                &format!("{:<width$}", line.author, width = max_author),
                theme.blame_author,
            );
            print!("  ");
            out.dim(&format!("{:<width$}", dates[i], width = max_date), theme.blame_date);
        }

        out.dim(
            &format!(" {:>width$} \u{2502} ", i + 1, width = num_width),
            theme.line_number,
        );

        let code_line = format!("{}\n", line.content);
        match h.highlight_line(&code_line, &ss) {
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
            Err(_) => print!("{}\n", line.content),
        }

        prev_hash.clone_from(&line.hash);
    }
}

fn syntect_to_crossterm(style: Style) -> Color {
    Color::Rgb {
        r: style.foreground.r,
        g: style.foreground.g,
        b: style.foreground.b,
    }
}

fn parse_porcelain(input: &str) -> Vec<BlameLine> {
    let mut results = Vec::new();
    let mut iter = input.lines().peekable();
    let mut cache: HashMap<String, (String, i64)> = HashMap::new();

    while let Some(header) = iter.next() {
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.is_empty() || parts[0].len() < 40 {
            continue;
        }

        let full_hash = parts[0];
        let short_hash = full_hash[..7].to_string();

        let mut author = String::new();
        let mut timestamp: i64 = 0;

        // Read metadata lines until tab-prefixed content line
        while let Some(&next) = iter.peek() {
            if next.starts_with('\t') {
                break;
            }
            let line = iter.next().unwrap();
            if let Some(name) = line.strip_prefix("author ") {
                author = name.to_string();
            } else if let Some(ts) = line.strip_prefix("author-time ") {
                timestamp = ts.parse().unwrap_or(0);
            }
        }

        if author.is_empty() {
            if let Some((a, t)) = cache.get(full_hash) {
                author = a.clone();
                timestamp = *t;
            }
        } else {
            cache.insert(full_hash.to_string(), (author.clone(), timestamp));
        }

        let content = iter
            .next()
            .and_then(|l| l.strip_prefix('\t'))
            .unwrap_or("")
            .to_string();

        results.push(BlameLine {
            hash: short_hash,
            author,
            timestamp,
            content,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_porcelain() {
        let input = "\
abc1234567890abcdef1234567890abcdef12345678 1 1 2
author Alice
author-mail <alice@example.com>
author-time 1700000000
author-tz +0000
committer Alice
committer-mail <alice@example.com>
committer-time 1700000000
committer-tz +0000
summary initial commit
filename src/main.rs
\tfn main() {
abc1234567890abcdef1234567890abcdef12345678 2 2
filename src/main.rs
\t    println!(\"hello\");
def4567890abcdef1234567890abcdef1234567890 3 3 1
author Bob
author-mail <bob@example.com>
author-time 1710000000
author-tz +0000
committer Bob
committer-mail <bob@example.com>
committer-time 1710000000
committer-tz +0000
summary add x
filename src/main.rs
\t    let x = 42;
abc1234567890abcdef1234567890abcdef12345678 4 4
filename src/main.rs
\t}";

        let lines = parse_porcelain(input);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].hash, "abc1234");
        assert_eq!(lines[0].author, "Alice");
        assert_eq!(lines[0].timestamp, 1_700_000_000);
        assert_eq!(lines[0].content, "fn main() {");
        assert_eq!(lines[1].hash, "abc1234");
        assert_eq!(lines[1].author, "Alice");
        assert_eq!(lines[1].content, "    println!(\"hello\");");
        assert_eq!(lines[2].hash, "def4567");
        assert_eq!(lines[2].author, "Bob");
        assert_eq!(lines[2].timestamp, 1_710_000_000);
        assert_eq!(lines[2].content, "    let x = 42;");
        assert_eq!(lines[3].hash, "abc1234");
        assert_eq!(lines[3].author, "Alice");
        assert_eq!(lines[3].content, "}");
    }

    #[test]
    fn test_parse_porcelain_empty() {
        let lines = parse_porcelain("");
        assert!(lines.is_empty());
    }
}

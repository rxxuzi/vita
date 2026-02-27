//! JSON renderer with rainbow brackets and pastel highlighting
//!
//! Nested brackets `{}[]` cycle through pastel rainbow colors,
//! making nesting depth instantly visible.

use crate::output::Output;
use crate::theme::Theme;

const RAINBOW: &[(u8, u8, u8)] = &[
    (255, 154, 162), // pastel red
    (255, 183, 148), // pastel orange
    (255, 218, 145), // pastel yellow
    (182, 231, 160), // pastel green
    (146, 220, 229), // pastel cyan
    (159, 188, 249), // pastel blue
    (199, 164, 247), // pastel purple
    (248, 165, 212), // pastel pink
];

pub fn render(content: &str, theme: &Theme, out: &Output) {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(value) => {
            let pretty =
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string());
            render_highlighted(&pretty, theme, out);
        }
        Err(_) => {
            render_highlighted(content, theme, out);
        }
    }
}

fn render_highlighted(json: &str, theme: &Theme, out: &Output) {
    let mut in_string = false;
    let mut is_key = false;
    let mut escaped = false;
    let mut after_colon = false;
    let mut depth: usize = 0;
    let chars: Vec<char> = json.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if escaped {
            let color = if is_key {
                theme.json_key
            } else {
                theme.json_string
            };
            out.colored(&ch.to_string(), color);
            escaped = false;
            i += 1;
            continue;
        }

        if ch == '\\' && in_string {
            escaped = true;
            let color = if is_key {
                theme.json_key
            } else {
                theme.json_string
            };
            out.colored("\\", color);
            i += 1;
            continue;
        }

        if ch == '"' {
            if !in_string {
                in_string = true;
                is_key = !after_colon && is_likely_key(json, i);
                let color = if is_key {
                    theme.json_key
                } else {
                    theme.json_string
                };
                out.colored("\"", color);
            } else {
                let color = if is_key {
                    theme.json_key
                } else {
                    theme.json_string
                };
                out.colored("\"", color);
                in_string = false;
                is_key = false;
                after_colon = false;
            }
            i += 1;
            continue;
        }

        if in_string {
            let color = if is_key {
                theme.json_key
            } else {
                theme.json_string
            };
            out.colored(&ch.to_string(), color);
            i += 1;
            continue;
        }

        match ch {
            ':' => {
                out.colored(":", theme.json_bracket);
                after_colon = true;
                i += 1;
            }
            ',' => {
                let (r, g, b) = rainbow_color(depth);
                out.colored(",", crossterm::style::Color::Rgb { r, g, b });
                after_colon = false;
                i += 1;
            }
            '{' | '[' => {
                let (r, g, b) = rainbow_color(depth);
                out.colored(
                    &ch.to_string(),
                    crossterm::style::Color::Rgb { r, g, b },
                );
                depth += 1;
                after_colon = false;
                i += 1;
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                let (r, g, b) = rainbow_color(depth);
                out.colored(
                    &ch.to_string(),
                    crossterm::style::Color::Rgb { r, g, b },
                );
                i += 1;
            }
            _ if ch.is_ascii_digit() || ch == '-' || ch == '.' => {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_ascii_digit()
                        || chars[i] == '.'
                        || chars[i] == '-'
                        || chars[i] == '+'
                        || chars[i] == 'e'
                        || chars[i] == 'E')
                {
                    i += 1;
                }
                let num: String = chars[start..i].iter().collect();
                out.colored(&num, theme.json_number);
            }
            't' | 'f' => {
                let word: String = chars[i..].iter().take(5).collect();
                if word.starts_with("true") {
                    out.colored("true", theme.json_bool);
                    i += 4;
                } else if word.starts_with("false") {
                    out.colored("false", theme.json_bool);
                    i += 5;
                } else {
                    print!("{}", ch);
                    i += 1;
                }
            }
            'n' => {
                let word: String = chars[i..].iter().take(4).collect();
                if word == "null" {
                    out.colored("null", theme.json_null);
                    i += 4;
                } else {
                    print!("{}", ch);
                    i += 1;
                }
            }
            '\n' => {
                println!();
                after_colon = false;
                i += 1;
            }
            _ => {
                print!("{}", ch);
                i += 1;
            }
        }
    }

    if !json.ends_with('\n') {
        println!();
    }
}

fn rainbow_color(depth: usize) -> (u8, u8, u8) {
    RAINBOW[depth % RAINBOW.len()]
}

fn is_likely_key(json: &str, quote_pos: usize) -> bool {
    let chars: Vec<char> = json[quote_pos + 1..].chars().collect();
    let mut i = 0;
    let mut escaped = false;

    while i < chars.len() {
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if chars[i] == '\\' {
            escaped = true;
            i += 1;
            continue;
        }
        if chars[i] == '"' {
            for j in (i + 1)..chars.len() {
                if chars[j] == ':' {
                    return true;
                }
                if !chars[j].is_whitespace() {
                    return false;
                }
            }
            return false;
        }
        i += 1;
    }
    false
}

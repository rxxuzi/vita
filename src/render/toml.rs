//! TOML renderer with colored keys, values, and section headers
//!
//! Line-based parser that preserves original formatting.
//! Section headers `[name]`/`[[name]]` get bracket color + bold key,
//! key-value pairs are colored by value type.

use crate::output::Output;
use crate::theme::Theme;

pub fn render(content: &str, theme: &Theme, out: &Output) {
    for line in content.lines() {
        render_line(line, theme, out);
        println!();
    }
}

fn render_line(line: &str, theme: &Theme, out: &Output) {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return;
    }

    // Comment
    if trimmed.starts_with('#') {
        let indent = &line[..line.len() - trimmed.len()];
        print!("{}", indent);
        out.dim(trimmed, theme.line_number);
        return;
    }

    // Section header: [[array]] or [table]
    if trimmed.starts_with('[') {
        let indent = &line[..line.len() - trimmed.len()];
        print!("{}", indent);
        render_section_header(trimmed, theme, out);
        return;
    }

    // Key = value
    if let Some(eq_pos) = find_equals(trimmed) {
        let indent = &line[..line.len() - trimmed.len()];
        print!("{}", indent);
        let key = &trimmed[..eq_pos];
        let rest = &trimmed[eq_pos..];
        out.colored(key.trim_end(), theme.json_key);
        // Print spacing between key and '='
        let key_trimmed_len = key.trim_end().len();
        if key_trimmed_len < key.len() {
            print!("{}", &key[key_trimmed_len..]);
        }
        out.colored("=", theme.json_bracket);
        if rest.len() > 1 {
            let value_part = &rest[1..];
            // Preserve leading space after '='
            let value_trimmed = value_part.trim_start();
            let spaces = &value_part[..value_part.len() - value_trimmed.len()];
            print!("{}", spaces);
            render_value(value_trimmed, theme, out);
        }
        return;
    }

    // Fallback
    out.colored(line, theme.text);
}

fn render_section_header(trimmed: &str, theme: &Theme, out: &Output) {
    let double = trimmed.starts_with("[[");
    if double {
        out.bold_colored("[[", theme.json_bracket);
        if let Some(end) = trimmed.find("]]") {
            let name = &trimmed[2..end];
            out.bold_colored(name, theme.json_key);
            out.bold_colored("]]", theme.json_bracket);
            // Trailing comment
            let after = trimmed[end + 2..].trim();
            if !after.is_empty() {
                print!(" ");
                out.dim(after, theme.line_number);
            }
        } else {
            out.bold_colored(&trimmed[2..], theme.json_key);
        }
    } else {
        out.bold_colored("[", theme.json_bracket);
        if let Some(end) = trimmed[1..].find(']') {
            let name = &trimmed[1..1 + end];
            out.bold_colored(name, theme.json_key);
            out.bold_colored("]", theme.json_bracket);
            let after = trimmed[2 + end..].trim();
            if !after.is_empty() {
                print!(" ");
                out.dim(after, theme.line_number);
            }
        } else {
            out.bold_colored(&trimmed[1..], theme.json_key);
        }
    }
}

/// Find the position of `=` that is not inside a string.
fn find_equals(line: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if ch == '=' && !in_string {
            return Some(i);
        }
    }
    None
}

fn render_value(value: &str, theme: &Theme, out: &Output) {
    // Inline comment after value
    let (val, comment) = split_trailing_comment(value);
    let val = val.trim_end();

    render_value_core(val, theme, out);

    if let Some(c) = comment {
        print!(" ");
        out.dim(c, theme.line_number);
    }
}

fn render_value_core(val: &str, theme: &Theme, out: &Output) {
    if val.is_empty() {
        return;
    }

    // String (basic or literal)
    if (val.starts_with("\"\"\"") && val.ends_with("\"\"\""))
        || (val.starts_with("'''") && val.ends_with("'''"))
        || (val.starts_with('"') && val.ends_with('"'))
        || (val.starts_with('\'') && val.ends_with('\''))
    {
        out.colored(val, theme.json_string);
        return;
    }

    // Boolean
    if val == "true" || val == "false" {
        out.colored(val, theme.json_bool);
        return;
    }

    // Number (integer, float, hex, oct, bin, inf, nan, with optional sign/underscores)
    if is_toml_number(val) {
        out.colored(val, theme.json_number);
        return;
    }

    // Date/time values — treat as strings
    if is_toml_datetime(val) {
        out.colored(val, theme.json_string);
        return;
    }

    // Inline array
    if val.starts_with('[') && val.ends_with(']') {
        render_inline_array(val, theme, out);
        return;
    }

    // Inline table
    if val.starts_with('{') && val.ends_with('}') {
        render_inline_table(val, theme, out);
        return;
    }

    // Bare value fallback
    out.colored(val, theme.json_string);
}

fn render_inline_array(val: &str, theme: &Theme, out: &Output) {
    out.colored("[", theme.json_bracket);
    let inner = &val[1..val.len() - 1];
    let parts = split_top_level(inner, ',');
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.colored(",", theme.json_bracket);
        }
        let trimmed = part.trim();
        if trimmed.is_empty() {
            print!("{}", part);
        } else {
            let leading = &part[..part.len() - part.trim_start().len()];
            let trailing = &part[part.trim_end().len()..];
            print!("{}", leading);
            render_value_core(trimmed, theme, out);
            print!("{}", trailing);
        }
    }
    out.colored("]", theme.json_bracket);
}

fn render_inline_table(val: &str, theme: &Theme, out: &Output) {
    out.colored("{", theme.json_bracket);
    let inner = &val[1..val.len() - 1];
    let parts = split_top_level(inner, ',');
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.colored(",", theme.json_bracket);
        }
        let trimmed = part.trim();
        if let Some(eq) = trimmed.find('=') {
            let leading = &part[..part.len() - part.trim_start().len()];
            print!("{}", leading);
            let key = trimmed[..eq].trim_end();
            let v = trimmed[eq + 1..].trim();
            out.colored(key, theme.json_key);
            print!(" ");
            out.colored("=", theme.json_bracket);
            print!(" ");
            render_value_core(v, theme, out);
        } else {
            print!("{}", part);
        }
    }
    out.colored("}", theme.json_bracket);
}

/// Split by delimiter at top level (not inside strings, brackets, or braces).
fn split_top_level(s: &str, delim: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    let mut start = 0;

    for (i, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            if ch == '[' || ch == '{' {
                depth += 1;
            } else if ch == ']' || ch == '}' {
                depth -= 1;
            } else if ch == delim && depth == 0 {
                parts.push(&s[start..i]);
                start = i + ch.len_utf8();
            }
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Split trailing `# comment` that's not inside a string.
fn split_trailing_comment(value: &str) -> (&str, Option<&str>) {
    let mut in_string = false;
    let mut escaped = false;
    let mut quote_char = ' ';
    let bytes = value.as_bytes();

    for i in 0..bytes.len() {
        let ch = bytes[i] as char;
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if !in_string && (ch == '"' || ch == '\'') {
            in_string = true;
            quote_char = ch;
            continue;
        }
        if in_string && ch == quote_char {
            in_string = false;
            continue;
        }
        if !in_string && ch == '#' {
            return (value[..i].trim_end(), Some(&value[i..]));
        }
    }
    (value, None)
}

fn is_toml_number(val: &str) -> bool {
    let s = val.replace('_', "");
    if s.is_empty() {
        return false;
    }
    // Special float values
    if matches!(s.as_str(), "inf" | "+inf" | "-inf" | "nan" | "+nan" | "-nan") {
        return true;
    }
    // Hex, Oct, Bin
    if s.starts_with("0x") || s.starts_with("0o") || s.starts_with("0b") {
        return s.len() > 2 && s[2..].chars().all(|c| c.is_ascii_hexdigit());
    }
    // Regular number
    let s = s.strip_prefix(['+', '-']).unwrap_or(&s);
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-')
        && s.chars().next().map_or(false, |c| c.is_ascii_digit())
}

fn is_toml_datetime(val: &str) -> bool {
    // Simple heuristic: contains digit-digit-digit-digit and either '-' or ':'
    val.len() >= 10
        && val.as_bytes()[4] == b'-'
        && val.chars().take(4).all(|c| c.is_ascii_digit())
}

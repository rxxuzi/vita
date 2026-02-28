//! YAML renderer with colored keys, values, and document markers
//!
//! Line-based parser that preserves original formatting.
//! Keys get key color, values are colored by detected type
//! (bool, null, number, string).

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

    // Document separator
    if trimmed == "---" || trimmed == "..." {
        let indent = &line[..line.len() - trimmed.len()];
        print!("{}", indent);
        out.colored(trimmed, theme.hr);
        return;
    }

    let indent = &line[..line.len() - line.trim_start().len()];
    let rest = line.trim_start();

    // Directive (e.g. %YAML 1.2)
    if rest.starts_with('%') {
        print!("{}", indent);
        out.dim(rest, theme.line_number);
        return;
    }

    // List item: "- ..." or "-\n"
    if rest.starts_with("- ") || rest == "-" {
        print!("{}", indent);
        out.colored("- ", theme.list_bullet);
        if rest.len() > 2 {
            let item = &rest[2..];
            render_key_or_value(item, theme, out);
        }
        return;
    }

    // Key: value
    if let Some(colon_pos) = find_colon(rest) {
        print!("{}", indent);
        let key = &rest[..colon_pos];
        out.colored(key, theme.json_key);
        out.colored(":", theme.json_key);

        let after = &rest[colon_pos + 1..];
        if after.is_empty() {
            return;
        }

        // Check for trailing comment
        let (value_part, comment) = split_comment(after);
        let value_trimmed = value_part.trim();

        if value_trimmed.is_empty() {
            // Might just be spaces before a comment
            if let Some(c) = comment {
                print!(" ");
                out.dim(c, theme.line_number);
            }
            return;
        }

        // Preserve the single space after colon
        print!(" ");
        render_typed_value(value_trimmed, theme, out);

        if let Some(c) = comment {
            print!(" ");
            out.dim(c, theme.line_number);
        }
        return;
    }

    // Bare value line (continuation, block scalar, etc.)
    print!("{}", indent);
    out.colored(rest, theme.text);
}

/// Handle content after a list marker that might be `key: value` or just a value.
fn render_key_or_value(item: &str, theme: &Theme, out: &Output) {
    if let Some(colon_pos) = find_colon(item) {
        let key = &item[..colon_pos];
        out.colored(key, theme.json_key);
        out.colored(":", theme.json_key);

        let after = &item[colon_pos + 1..];
        if after.is_empty() {
            return;
        }

        let (value_part, comment) = split_comment(after);
        let value_trimmed = value_part.trim();

        if !value_trimmed.is_empty() {
            print!(" ");
            render_typed_value(value_trimmed, theme, out);
        }

        if let Some(c) = comment {
            print!(" ");
            out.dim(c, theme.line_number);
        }
    } else {
        let (value_part, comment) = split_comment(item);
        let value_trimmed = value_part.trim();
        if !value_trimmed.is_empty() {
            render_typed_value(value_trimmed, theme, out);
        }
        if let Some(c) = comment {
            print!(" ");
            out.dim(c, theme.line_number);
        }
    }
}

fn render_typed_value(val: &str, theme: &Theme, out: &Output) {
    // Flow sequence [...] or mapping {...}
    if (val.starts_with('[') && val.ends_with(']'))
        || (val.starts_with('{') && val.ends_with('}'))
    {
        render_flow(val, theme, out);
        return;
    }

    // Block scalar indicators
    if val == "|" || val == ">" || val == "|-" || val == ">-" || val == "|+" || val == ">+" {
        out.colored(val, theme.json_bracket);
        return;
    }

    // Anchor/alias
    if val.starts_with('&') || val.starts_with('*') {
        out.colored(val, theme.json_key);
        return;
    }

    // Tag
    if val.starts_with('!') {
        out.colored(val, theme.json_bracket);
        return;
    }

    let lower = val.to_lowercase();

    // Boolean
    if matches!(lower.as_str(), "true" | "false" | "yes" | "no" | "on" | "off") {
        out.colored(val, theme.json_bool);
        return;
    }

    // Null
    if matches!(lower.as_str(), "null" | "~") {
        out.colored(val, theme.json_null);
        return;
    }

    // Quoted string
    if (val.starts_with('"') && val.ends_with('"'))
        || (val.starts_with('\'') && val.ends_with('\''))
    {
        out.colored(val, theme.json_string);
        return;
    }

    // Number
    if is_yaml_number(val) {
        out.colored(val, theme.json_number);
        return;
    }

    // Bare string
    out.colored(val, theme.text);
}

fn render_flow(val: &str, theme: &Theme, out: &Output) {
    let open = &val[..1];
    let close = &val[val.len() - 1..];
    let is_mapping = open == "{";

    out.colored(open, theme.json_bracket);

    let inner = &val[1..val.len() - 1];
    let parts = split_flow_top_level(inner);

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.colored(",", theme.json_bracket);
        }
        let trimmed = part.trim();
        if trimmed.is_empty() {
            print!("{}", part);
            continue;
        }
        let leading = &part[..part.len() - part.trim_start().len()];
        print!("{}", leading);

        if is_mapping {
            if let Some(cp) = trimmed.find(':') {
                let k = trimmed[..cp].trim();
                out.colored(k, theme.json_key);
                out.colored(":", theme.json_key);
                let v = trimmed[cp + 1..].trim();
                if !v.is_empty() {
                    print!(" ");
                    render_typed_value(v, theme, out);
                }
            } else {
                render_typed_value(trimmed, theme, out);
            }
        } else {
            render_typed_value(trimmed, theme, out);
        }
    }

    out.colored(close, theme.json_bracket);
}

/// Split by comma at top level (not inside strings, brackets, or braces).
fn split_flow_top_level(s: &str) -> Vec<&str> {
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
        if !in_string && (ch == '"' || ch == '\'') {
            in_string = true;
            continue;
        }
        if in_string && (ch == '"' || ch == '\'') {
            in_string = false;
            continue;
        }
        if !in_string {
            if ch == '[' || ch == '{' {
                depth += 1;
            } else if ch == ']' || ch == '}' {
                depth -= 1;
            } else if ch == ',' && depth == 0 {
                parts.push(&s[start..i]);
                start = i + 1;
            }
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Find the first `:` that is a YAML key separator (followed by space, end, or newline)
/// and not inside a quoted string.
fn find_colon(s: &str) -> Option<usize> {
    let mut in_string = false;
    let mut quote_char = ' ';
    let bytes = s.as_bytes();

    for i in 0..bytes.len() {
        let ch = bytes[i] as char;
        if !in_string && (ch == '"' || ch == '\'') {
            in_string = true;
            quote_char = ch;
            continue;
        }
        if in_string && ch == quote_char {
            in_string = false;
            continue;
        }
        if !in_string && ch == ':' {
            // Colon must be followed by space, end of string, or nothing
            let next = bytes.get(i + 1).map(|&b| b as char);
            if next.is_none() || next == Some(' ') || next == Some('\t') {
                return Some(i);
            }
        }
    }
    None
}

/// Split trailing `# comment` that's not inside a string.
fn split_comment(value: &str) -> (&str, Option<&str>) {
    let mut in_string = false;
    let mut quote_char = ' ';
    let bytes = value.as_bytes();

    for i in 0..bytes.len() {
        let ch = bytes[i] as char;
        if !in_string && (ch == '"' || ch == '\'') {
            in_string = true;
            quote_char = ch;
            continue;
        }
        if in_string && ch == quote_char {
            in_string = false;
            continue;
        }
        // `#` preceded by space (YAML comment rule)
        if !in_string && ch == '#' && i > 0 && bytes[i - 1] == b' ' {
            return (value[..i].trim_end(), Some(&value[i..]));
        }
    }
    (value, None)
}

fn is_yaml_number(val: &str) -> bool {
    if val.is_empty() {
        return false;
    }
    // Special float values
    if matches!(val, ".inf" | "-.inf" | "+.inf" | ".Inf" | "-.Inf" | "+.Inf"
                    | ".INF" | "-.INF" | "+.INF" | ".nan" | ".NaN" | ".NAN") {
        return true;
    }
    // Hex, Oct
    if val.starts_with("0x") || val.starts_with("0o") {
        return val.len() > 2 && val[2..].chars().all(|c| c.is_ascii_hexdigit());
    }
    let s = val.strip_prefix(['+', '-']).unwrap_or(val);
    if s.is_empty() {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' || c == '_')
        && s.chars().next().map_or(false, |c| c.is_ascii_digit())
}

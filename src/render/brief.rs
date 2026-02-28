//! Structural outline renderer (brief mode)
//!
//! Extracts and displays structural elements from source files:
//! function definitions, class declarations, headings, section headers, etc.
//! Uses keyword-based line matching for code and format-specific logic for
//! data files (JSON, CSV, YAML, TOML, Markdown, HTML).

use crate::detect::FileFormat;
use crate::output::Output;
use crate::theme::Theme;

pub fn render(content: &str, format: &FileFormat, theme: &Theme, out: &Output) {
    match format {
        FileFormat::Markdown => brief_markdown(content, theme, out),
        FileFormat::Json => brief_json(content, theme, out),
        FileFormat::Csv => brief_csv(content, theme, out),
        FileFormat::Toml => brief_toml(content, theme, out),
        FileFormat::Yaml => brief_yaml(content, theme, out),
        FileFormat::Code(lang) => brief_code(content, lang, theme, out),
        FileFormat::Plain => brief_plain(content, theme, out),
        FileFormat::Image => {}
    }
}

// ─── Markdown ───

fn brief_markdown(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            print_line(i + 1, width, line, theme, out);
        }
    }
}

// ─── JSON ───

fn brief_json(content: &str, theme: &Theme, out: &Output) {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
        brief_json_value(&val, theme, out);
    } else {
        // Fallback: scan for top-level keys by looking for lines with `"key":`
        let lines: Vec<&str> = content.lines().collect();
        let width = line_num_width(lines.len());
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('"') && trimmed.contains("\":") {
                print_line(i + 1, width, line, theme, out);
            }
        }
    }
}

fn brief_json_value(val: &serde_json::Value, theme: &Theme, out: &Output) {
    match val {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let summary = json_type_summary(value);
                out.colored("  ", theme.text);
                out.bold_colored(key, theme.json_key);
                out.dim(&format!(": {}\n", summary), theme.line_number);
            }
        }
        serde_json::Value::Array(arr) => {
            let summary = if arr.is_empty() {
                "[] (empty)".to_string()
            } else {
                let first_type = json_type_name(&arr[0]);
                format!("[{}] ({} items, {})", first_type, arr.len(), first_type)
            };
            out.dim(&format!("  {}\n", summary), theme.line_number);
        }
        _ => {
            out.dim(&format!("  {}\n", json_type_name(val)), theme.line_number);
        }
    }
}

fn json_type_summary(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Object(map) => format!("{{}} ({} keys)", map.len()),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                "[] (empty)".to_string()
            } else {
                format!("[{}] ({} items)", json_type_name(&arr[0]), arr.len())
            }
        }
        serde_json::Value::String(s) => {
            if s.len() > 40 {
                format!("\"{}...\"", &s[..37])
            } else {
                format!("\"{}\"", s)
            }
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
    }
}

fn json_type_name(val: &serde_json::Value) -> &'static str {
    match val {
        serde_json::Value::Object(_) => "object",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Null => "null",
    }
}

// ─── CSV ───

fn brief_csv(content: &str, theme: &Theme, out: &Output) {
    let delimiter = super::csv::detect_delimiter(content);
    let mut lines = content.lines();

    let header = match lines.next() {
        Some(h) if !h.trim().is_empty() => h,
        _ => return,
    };

    let cols: Vec<&str> = header.split(delimiter).map(|s| s.trim()).collect();
    out.bold_colored("  Columns: ", theme.table_header);
    out.colored(&cols.join(", "), theme.text);
    println!();

    let data_lines: Vec<&str> = lines.filter(|l| !l.trim().is_empty()).collect();
    let total = data_lines.len();

    let preview_count = total.min(3);
    for line in &data_lines[..preview_count] {
        let fields: Vec<&str> = line.split(delimiter).map(|s| s.trim()).collect();
        out.dim("  ", theme.line_number);
        out.colored(&fields.join(", "), theme.text);
        println!();
    }

    if total > 3 {
        out.dim(&format!("  ... ({} more rows)\n", total - 3), theme.line_number);
    }

    out.dim(&format!("  {} rows × {} columns\n", total, cols.len()), theme.line_number);
}

// ─── Code (keyword-based) ───

fn brief_code(content: &str, lang: &str, theme: &Theme, out: &Output) {
    let normalized = lang.to_lowercase();

    match normalized.as_str() {
        "yaml" | "yml" => return brief_yaml(content, theme, out),
        "toml" => return brief_toml(content, theme, out),
        "html" | "html (rails)" | "html (tcl)" => return brief_html(content, theme, out),
        "css" | "scss" | "sass" | "less" => return brief_css(content, theme, out),
        "batch file" | "bat" | "cmd" => return brief_batch(content, theme, out),
        "asm" | "nasm" | "assembly" => return brief_asm(content, theme, out),
        _ => {}
    }

    let keywords = keywords_for(&normalized);
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if matches_keyword(trimmed, keywords) {
            print_line(i + 1, width, line, theme, out);
            found = true;
            continue;
        }

        // Language-specific heuristics (use original line for indentation checks)
        match normalized.as_str() {
            "c" | "c++" | "objective-c" | "objective-c++" if is_c_func_def(line) => {
                print_line(i + 1, width, line, theme, out);
                found = true;
            }
            "haskell" if has_haskell_sig(line) => {
                print_line(i + 1, width, line, theme, out);
                found = true;
            }
            "bash" | "sh" | "zsh" | "fish" | "shell"
            | "bourne again shell (bash)" if is_shell_func(line) => {
                print_line(i + 1, width, line, theme, out);
                found = true;
            }
            _ => {}
        }
    }

    if !found {
        out.dim(&format!("  (no brief outline for {})\n", lang), theme.line_number);
    }
}

fn matches_keyword(trimmed: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| trimmed.starts_with(kw))
}

fn keywords_for(lang: &str) -> &'static [&'static str] {
    match lang {
        "rust" => &[
            "fn ", "pub fn ", "pub(crate) fn ", "pub(super) fn ",
            "struct ", "pub struct ", "pub(crate) struct ",
            "enum ", "pub enum ", "pub(crate) enum ",
            "trait ", "pub trait ",
            "impl ", "mod ", "pub mod ", "pub(crate) mod ",
        ],
        "python" => &[
            "import ", "from ", "class ", "def ", "async def ", "if __name__",
        ],
        "javascript" | "jsx" | "javascriptreact" => &[
            "import ", "export ", "function ", "async function ", "class ",
            "const ", "let ",
        ],
        "typescript" | "tsx" | "typescriptreact" => &[
            "import ", "export ", "function ", "async function ", "class ",
            "const ", "let ", "interface ", "type ", "enum ",
        ],
        "go" => &["package ", "import ", "type ", "func "],
        "java" => &[
            "package ", "import ", "class ", "interface ", "enum ",
            "public ", "private ", "protected ",
        ],
        "c" => &["#include ", "typedef ", "struct ", "union ", "enum "],
        "c++" | "objective-c" | "objective-c++" => &[
            "#include ", "typedef ", "struct ", "union ", "enum ",
            "class ", "namespace ", "template ",
        ],
        "c#" => &[
            "using ", "namespace ", "class ", "interface ", "struct ",
            "enum ", "public ", "private ", "protected ", "internal ",
        ],
        "ruby" => &[
            "require ", "require_relative ", "module ", "class ", "def ",
        ],
        "php" => &[
            "namespace ", "use ", "class ", "function ",
            "public function ", "private function ", "protected function ",
        ],
        "kotlin" | "kt" => &[
            "package ", "import ", "class ", "data class ", "sealed class ",
            "object ", "fun ",
        ],
        "swift" => &[
            "import ", "protocol ", "struct ", "class ", "func ", "enum ", "extension ",
        ],
        "lua" => &["function ", "local function "],
        "scala" | "sbt" => &[
            "package ", "import ", "trait ", "class ", "case class ", "object ", "def ",
        ],
        "zig" => &["const ", "pub const ", "fn ", "pub fn "],
        "elixir" | "ex" => &["defmodule ", "def ", "defp "],
        "haskell" | "hs" => &["module ", "import ", "data ", "type ", "class "],
        "sql" | "ddl" | "dml" => &[
            "CREATE ", "ALTER ", "DROP ",
            "create ", "alter ", "drop ",
        ],
        "bash" | "sh" | "zsh" | "fish" | "shell"
        | "powershell" | "ps1"
        | "bourne again shell (bash)" => &["#!", "source ", "function "],
        "r" => &["library(", "require(", "source("],
        "perl" => &["use ", "package ", "sub "],
        "d" => &[
            "import ", "module ", "class ", "struct ", "interface ", "void ", "auto ",
        ],
        "ocaml" | "ml" => &["let ", "module ", "type ", "val ", "open "],
        "clojure" => &["(ns ", "(def ", "(defn ", "(defmacro "],
        "erlang" => &["-module(", "-export(", "-import("],
        "lisp" | "scheme" => &["(define ", "(defun ", "(defmacro ", "(defvar "],
        "groovy" | "gradle" => &[
            "package ", "import ", "class ", "interface ", "def ",
        ],
        "pascal" | "delphi" => &[
            "program ", "unit ", "uses ", "type ", "procedure ", "function ",
        ],
        "batch file" | "bat" | "cmd" => &[],
        "makefile" => &[".PHONY", "define "],
        "dockerfile" => &["FROM ", "RUN ", "CMD ", "ENTRYPOINT ", "COPY ", "ADD ", "ENV ", "EXPOSE "],
        "terraform" | "tf" | "hcl" => &["resource ", "data ", "variable ", "output ", "module ", "provider "],
        "graphql" | "gql" => &["type ", "input ", "enum ", "interface ", "query ", "mutation ", "subscription "],
        "protocol buffers" | "proto" => &["syntax ", "package ", "message ", "service ", "enum ", "rpc "],
        _ => &[],
    }
}

// ─── C/C++ function definition heuristic ───
// Matches lines like `int main(` or `void* foo(` but not `if (`, `while (`, etc.
fn is_c_func_def(line: &str) -> bool {
    if line.starts_with(' ') || line.starts_with('\t') {
        return false;
    }
    if !line.contains('(') {
        return false;
    }
    let control = ["if ", "if(", "else ", "while ", "while(", "for ", "for(",
                    "switch ", "switch(", "return ", "return(", "//", "/*", "#"];
    if control.iter().any(|kw| line.starts_with(kw)) {
        return false;
    }
    let paren_pos = line.find('(').unwrap();
    let before = line[..paren_pos].trim();
    // Must have at least two tokens (return type + name) before `(`
    before.contains(' ') || before.contains('*')
}

// ─── CSS selector heuristic ───

fn brief_css(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if is_css_selector(trimmed) {
            print_line(i + 1, width, line, theme, out);
            found = true;
        }
    }

    if !found {
        out.dim("  (no brief outline for CSS)\n", theme.line_number);
    }
}

fn is_css_selector(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("/*") || trimmed.starts_with("//") {
        return false;
    }
    // @import, @media, @keyframes, etc.
    if trimmed.starts_with('@') {
        return true;
    }
    trimmed.ends_with('{')
}

// ─── Batch File ───

fn brief_batch(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Labels like `:main`, `:parse_args` (but not `::` comments)
        if trimmed.starts_with(':') && !trimmed.starts_with("::") {
            print_line(i + 1, width, line, theme, out);
            found = true;
        }
    }

    if !found {
        out.dim("  (no brief outline for Batch File)\n", theme.line_number);
    }
}

// ─── Assembly ───

fn brief_asm(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if is_asm_structural(trimmed) {
            print_line(i + 1, width, line, theme, out);
            found = true;
        }
    }

    if !found {
        out.dim("  (no brief outline for ASM)\n", theme.line_number);
    }
}

fn is_asm_structural(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(';') {
        return false;
    }
    // Labels: `word:`
    if trimmed.contains(':') && !trimmed.starts_with('.') {
        let colon_pos = trimmed.find(':').unwrap();
        let before = &trimmed[..colon_pos];
        if !before.is_empty() && before.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return true;
        }
    }
    let lower = trimmed.to_lowercase();
    lower.starts_with("section ") || lower.starts_with("global ") || lower.starts_with(".section ")
}

// ─── Haskell type signature ───

fn has_haskell_sig(line: &str) -> bool {
    if line.starts_with(' ') || line.starts_with('\t') || line.starts_with("--") {
        return false;
    }
    line.contains(" :: ")
}

// ─── Shell function pattern: `name() {` ───

fn is_shell_func(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.ends_with("() {") || trimmed.ends_with("(){")
}

// ─── YAML ───

fn brief_yaml(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        // Top-level keys (indent 0) or second-level (indent <= 2), must contain ':'
        if indent <= 2 && line.contains(':') {
            print_line(i + 1, width, line, theme, out);
            found = true;
        }
    }

    if !found {
        out.dim("  (no brief outline for YAML)\n", theme.line_number);
    }
}

// ─── TOML ───

fn brief_toml(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Section headers: [section] or [[array]]
        if trimmed.starts_with('[') {
            print_line(i + 1, width, line, theme, out);
            found = true;
        }
    }

    if !found {
        out.dim("  (no brief outline for TOML)\n", theme.line_number);
    }
}

// ─── HTML ───

fn brief_html(content: &str, theme: &Theme, out: &Output) {
    let lines: Vec<&str> = content.lines().collect();
    let width = line_num_width(lines.len());
    let mut found = false;

    for (i, line) in lines.iter().enumerate() {
        let lower = line.trim_start().to_lowercase();
        if lower.starts_with("<title")
            || lower.starts_with("<h1")
            || lower.starts_with("<h2")
            || lower.starts_with("<h3")
            || lower.starts_with("<h4")
            || lower.starts_with("<h5")
            || lower.starts_with("<h6")
        {
            print_line(i + 1, width, line, theme, out);
            found = true;
        }
    }

    if !found {
        out.dim("  (no brief outline for HTML)\n", theme.line_number);
    }
}

// ─── Plain text fallback ───

fn brief_plain(content: &str, theme: &Theme, out: &Output) {
    let line_count = content.lines().count();
    out.dim(&format!("  {} lines (plain text — no structure to outline)\n", line_count), theme.line_number);
}

// ─── Structural line extraction (for -b + -g combo) ───

/// Returns (1-based line number, line text) for lines considered structural.
/// For JSON/CSV/Plain (no line-mapped structure), returns empty vec.
pub fn structural_lines<'a>(content: &'a str, format: &FileFormat) -> Vec<(usize, &'a str)> {
    let lines: Vec<&str> = content.lines().collect();
    match format {
        FileFormat::Markdown => lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.trim_start().starts_with('#'))
            .map(|(i, l)| (i + 1, *l))
            .collect(),
        FileFormat::Toml => lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.trim().starts_with('['))
            .map(|(i, l)| (i + 1, *l))
            .collect(),
        FileFormat::Yaml => lines
            .iter()
            .enumerate()
            .filter(|(_, l)| !l.is_empty() && l.len() - l.trim_start().len() <= 2 && l.contains(':'))
            .map(|(i, l)| (i + 1, *l))
            .collect(),
        FileFormat::Code(lang) => collect_code_structural(&lines, lang),
        _ => Vec::new(),
    }
}

fn collect_code_structural<'a>(lines: &[&'a str], lang: &str) -> Vec<(usize, &'a str)> {
    let normalized = lang.to_lowercase();
    let mut result = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if is_structural_code_line(line, &normalized) {
            result.push((i + 1, *line));
        }
    }
    result
}

fn is_structural_code_line(line: &str, lang: &str) -> bool {
    let trimmed = line.trim_start();

    match lang {
        "yaml" | "yml" => {
            !line.is_empty() && line.len() - line.trim_start().len() <= 2 && line.contains(':')
        }
        "toml" => trimmed.starts_with('['),
        "html" | "html (rails)" | "html (tcl)" => {
            let lower = trimmed.to_lowercase();
            lower.starts_with("<title")
                || lower.starts_with("<h1")
                || lower.starts_with("<h2")
                || lower.starts_with("<h3")
                || lower.starts_with("<h4")
                || lower.starts_with("<h5")
                || lower.starts_with("<h6")
        }
        "css" | "scss" | "sass" | "less" => is_css_selector(trimmed),
        "batch file" | "bat" | "cmd" => trimmed.starts_with(':') && !trimmed.starts_with("::"),
        "asm" | "nasm" | "assembly" => is_asm_structural(trimmed),
        _ => {
            let keywords = keywords_for(lang);
            if matches_keyword(trimmed, keywords) {
                return true;
            }
            match lang {
                "c" | "c++" | "objective-c" | "objective-c++" => is_c_func_def(line),
                "haskell" => has_haskell_sig(line),
                "bash" | "sh" | "zsh" | "fish" | "shell"
                | "bourne again shell (bash)" => is_shell_func(line),
                _ => false,
            }
        }
    }
}

// ─── Output helpers ───

fn print_line(num: usize, width: usize, text: &str, theme: &Theme, out: &Output) {
    out.dim(&format!(" {:>w$} │ ", num, w = width), theme.line_number);
    out.colored(text, theme.text);
    println!();
}

fn line_num_width(total: usize) -> usize {
    format!("{}", total).len()
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords_for_rust() {
        let kw = keywords_for("rust");
        assert!(kw.contains(&"fn "));
        assert!(kw.contains(&"pub fn "));
        assert!(kw.contains(&"struct "));
        assert!(kw.contains(&"impl "));
        assert!(kw.contains(&"mod "));
    }

    #[test]
    fn test_keywords_for_unknown() {
        let kw = keywords_for("brainfuck");
        assert!(kw.is_empty());
    }

    #[test]
    fn test_brief_markdown_extracts_headings() {
        let content = "# Title\nSome text\n## Section\nMore text\n### Sub\n";
        let lines: Vec<&str> = content.lines().collect();
        let headings: Vec<(usize, &str)> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| l.trim_start().starts_with('#'))
            .map(|(i, l)| (i + 1, *l))
            .collect();
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0], (1, "# Title"));
        assert_eq!(headings[1], (3, "## Section"));
        assert_eq!(headings[2], (5, "### Sub"));
    }

    #[test]
    fn test_c_function_detection() {
        assert!(is_c_func_def("int main(int argc, char *argv[]) {"));
        assert!(is_c_func_def("void foo(void) {"));
        assert!(is_c_func_def("static int helper(int x)"));
        assert!(!is_c_func_def("if (x > 0) {"));
        assert!(!is_c_func_def("while (true) {"));
        assert!(!is_c_func_def("// comment with parens()"));
        assert!(!is_c_func_def("    indented_func(x)"));
    }

    #[test]
    fn test_format_json_brief() {
        let json = r#"{"name":"test","items":[1,2,3],"nested":{"a":1}}"#;
        let val: serde_json::Value = serde_json::from_str(json).unwrap();
        if let serde_json::Value::Object(map) = &val {
            assert_eq!(map.len(), 3);
            assert!(map.contains_key("name"));
            assert!(map.contains_key("items"));
            assert!(map.contains_key("nested"));
        } else {
            panic!("expected object");
        }
    }

    #[test]
    fn test_csv_brief_structure() {
        let csv = "name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,SF\nDave,28,CHI\n";
        let delimiter = crate::render::csv::detect_delimiter(csv);
        assert_eq!(delimiter, ',');
        let mut lines = csv.lines();
        let header = lines.next().unwrap();
        let cols: Vec<&str> = header.split(',').collect();
        assert_eq!(cols.len(), 3);
        let data: Vec<&str> = lines.filter(|l| !l.is_empty()).collect();
        assert_eq!(data.len(), 4);
    }

    #[test]
    fn test_haskell_sig() {
        assert!(has_haskell_sig("map :: (a -> b) -> [a] -> [b]"));
        assert!(!has_haskell_sig("  where helper :: Int -> Int"));
        assert!(!has_haskell_sig("-- comment :: not a sig"));
    }

    #[test]
    fn test_shell_func() {
        assert!(is_shell_func("my_func() {"));
        assert!(is_shell_func("build(){"));
        assert!(!is_shell_func("echo hello"));
    }

    #[test]
    fn test_css_selector() {
        assert!(is_css_selector(".container {"));
        assert!(is_css_selector("@media screen {"));
        assert!(!is_css_selector("  color: red;"));
        assert!(!is_css_selector("/* comment */"));
    }

    #[test]
    fn test_asm_structural() {
        assert!(is_asm_structural("main:"));
        assert!(is_asm_structural("section .text"));
        assert!(is_asm_structural("global _start"));
        assert!(!is_asm_structural("; comment"));
        assert!(!is_asm_structural("  mov eax, 1"));
    }

    #[test]
    fn test_matches_keyword() {
        let kw = &["fn ", "pub fn "];
        assert!(matches_keyword("fn main() {", kw));
        assert!(matches_keyword("pub fn new() -> Self {", kw));
        assert!(!matches_keyword("let x = fn_ptr;", kw));
    }
}

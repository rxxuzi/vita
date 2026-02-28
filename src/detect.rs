use std::path::Path;

#[derive(Debug, Clone)]
pub enum FileFormat {
    Markdown,
    Json,
    Csv,
    Code(String), // language name
    Image,
    Plain,
}

/// Map a language name to one that syntect actually recognizes.
///
/// syntect's default SyntaxSet (load_defaults) contains ~75 syntaxes.
/// Languages like TypeScript, TSX, JSX, PowerShell, etc. are NOT included.
/// This function maps them to the closest available syntax for highlighting.
pub fn syntax_fallback(lang: &str) -> &str {
    match lang {
        // TypeScript family → JavaScript (close enough for highlighting)
        "TypeScript" | "tsx" | "TSX" | "jsx" | "JSX" | "TypeScriptReact"
        | "JavaScriptReact" | "Svelte" | "Vue" => "JavaScript",

        // Shell variants
        "PowerShell" | "ps1" | "pwsh" => "Bourne Again Shell (bash)",
        "Fish" | "fish" => "Bourne Again Shell (bash)",
        "Bash" | "bash" | "sh" | "zsh" => "Bourne Again Shell (bash)",

        // Batch
        "Batch" | "bat" | "cmd" => "Batch File",

        // CSS preprocessors → CSS
        "SCSS" | "Sass" | "Less" | "Stylus" => "CSS",

        // Config formats → YAML / INI
        "TOML" | "toml" => "YAML",
        "INI" | "ini" => "Java Properties",
        "Dockerfile" | "dockerfile" => "Bourne Again Shell (bash)",

        // Build tools
        "CMake" | "cmake" => "Makefile",

        // Modern languages without syntect support → closest match
        "Kotlin" | "kt" => "Java",
        "Swift" | "swift" => "Objective-C",
        "Dart" | "dart" => "Java",
        "ASM" | "asm" | "nasm" | "NASM" => "Plain Text",
        "Zig" | "zig" => "C",
        "Elixir" | "ex" => "Ruby",
        "Terraform" | "tf" => "YAML",
        "GraphQL" | "gql" => "JavaScript",
        "Protocol Buffers" | "proto" => "Java",
        "VimL" | "vim" => "Bourne Again Shell (bash)",

        // Already supported - pass through
        _ => lang,
    }
}

pub fn detect_format(path: &Path) -> FileFormat {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // Markdown
        "md" | "markdown" | "mdown" | "mkd" => FileFormat::Markdown,

        // JSON
        "json" | "jsonc" | "geojson" | "jsonl" => FileFormat::Json,

        // CSV/TSV
        "csv" | "tsv" => FileFormat::Csv,

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tga" | "ppm" | "webp" | "ico"
        | "tiff" | "tif" | "qoi" | "exr" | "hdr" | "pgm" | "pbm" | "pam" | "ff" => {
            FileFormat::Image
        }

        // ── Languages with native syntect support ──────────────
        "rs" => FileFormat::Code("Rust".into()),
        "py" | "pyw" | "pyi" | "pyx" => FileFormat::Code("Python".into()),
        "js" | "mjs" | "cjs" => FileFormat::Code("JavaScript".into()),
        "c" => FileFormat::Code("C".into()),
        "h" => FileFormat::Code("C".into()), // could be C or C++, default to C
        "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hxx" | "h++" | "hh" | "ipp" | "inl" => {
            FileFormat::Code("C++".into())
        }
        "m" => FileFormat::Code("Objective-C".into()),
        "mm" => FileFormat::Code("Objective-C++".into()),
        "java" | "bsh" => FileFormat::Code("Java".into()),
        "go" => FileFormat::Code("Go".into()),
        "rb" | "rake" | "gemspec" => FileFormat::Code("Ruby".into()),
        "php" | "php3" | "php4" | "php5" | "phtml" => FileFormat::Code("PHP".into()),
        "cs" | "csx" => FileFormat::Code("C#".into()),
        "scala" | "sbt" => FileFormat::Code("Scala".into()),
        "lua" => FileFormat::Code("Lua".into()),
        "r" | "rmd" => FileFormat::Code("R".into()),
        "pl" | "pm" | "pod" => FileFormat::Code("Perl".into()),
        "d" | "di" => FileFormat::Code("D".into()),
        "hs" | "lhs" => FileFormat::Code("Haskell".into()),
        "ml" | "mli" => FileFormat::Code("OCaml".into()),
        "clj" | "cljs" | "cljc" | "edn" => FileFormat::Code("Clojure".into()),
        "erl" | "hrl" => FileFormat::Code("Erlang".into()),
        "lisp" | "cl" | "el" | "scm" | "ss" => FileFormat::Code("Lisp".into()),
        "groovy" | "gvy" | "gradle" => FileFormat::Code("Groovy".into()),
        "pas" | "dpr" => FileFormat::Code("Pascal".into()),
        "tcl" => FileFormat::Code("Tcl".into()),
        "tex" | "ltx" | "sty" | "cls" => FileFormat::Code("LaTeX".into()),
        "rst" | "rest" => FileFormat::Code("reStructuredText".into()),
        "html" | "htm" | "shtml" | "xhtml" => FileFormat::Code("HTML".into()),
        "erb" | "rhtml" => FileFormat::Code("HTML (Rails)".into()),
        "haml" => FileFormat::Code("Ruby Haml".into()),
        "css" => FileFormat::Code("CSS".into()),
        "xml" | "xsd" | "xslt" | "svg" | "rss" | "opml" => FileFormat::Code("XML".into()),
        "sql" | "ddl" | "dml" => FileFormat::Code("SQL".into()),
        "yaml" | "yml" => FileFormat::Code("YAML".into()),
        "json5" => FileFormat::Code("JSON".into()),
        "diff" | "patch" => FileFormat::Code("Diff".into()),
        "dot" | "gv" => FileFormat::Code("Graphviz (DOT)".into()),
        "bat" | "cmd" => FileFormat::Code("Batch File".into()),
        "makefile" | "mk" | "mak" => FileFormat::Code("Makefile".into()),
        "textile" => FileFormat::Code("Textile".into()),

        // ── Languages needing fallback (not in syntect defaults) ──
        "ts" | "mts" | "cts" => FileFormat::Code("TypeScript".into()),
        "tsx" => FileFormat::Code("TSX".into()),
        "jsx" => FileFormat::Code("JSX".into()),
        "svelte" => FileFormat::Code("Svelte".into()),
        "vue" => FileFormat::Code("Vue".into()),
        "sh" | "bash" | "zsh" => FileFormat::Code("Bash".into()),
        "fish" => FileFormat::Code("Fish".into()),
        "ps1" | "psm1" | "psd1" => FileFormat::Code("PowerShell".into()),
        "scss" | "sass" | "less" | "styl" => FileFormat::Code("SCSS".into()),
        "toml" => FileFormat::Code("TOML".into()),
        "ini" | "cfg" | "conf" => FileFormat::Code("INI".into()),
        "dockerfile" => FileFormat::Code("Dockerfile".into()),
        "cmake" => FileFormat::Code("CMake".into()),
        "zig" => FileFormat::Code("Zig".into()),
        "dart" => FileFormat::Code("Dart".into()),
        "swift" => FileFormat::Code("Swift".into()),
        "kt" | "kts" => FileFormat::Code("Kotlin".into()),
        "ex" | "exs" | "heex" => FileFormat::Code("Elixir".into()),
        "tf" | "tfvars" | "hcl" => FileFormat::Code("Terraform".into()),
        "proto" => FileFormat::Code("Protocol Buffers".into()),
        "graphql" | "gql" => FileFormat::Code("GraphQL".into()),
        "vim" => FileFormat::Code("VimL".into()),

        "asm" | "s" | "nasm" => FileFormat::Code("ASM".into()),

        "txt" | "text" | "log" => FileFormat::Plain,

        _ => {
            // Check filename (no extension)
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            match name.as_str() {
                "makefile" | "gnumakefile" => FileFormat::Code("Makefile".into()),
                "dockerfile" => FileFormat::Code("Dockerfile".into()),
                "cmakelists.txt" => FileFormat::Code("CMake".into()),
                "gemfile" | "rakefile" => FileFormat::Code("Ruby".into()),
                "cargo.toml" | "cargo.lock" => FileFormat::Code("TOML".into()),
                "package.json" | "tsconfig.json" | "deno.json" => FileFormat::Json,
                "go.sum" | "go.mod" => FileFormat::Code("Go".into()),
                ".gitignore" | ".gitattributes" | ".editorconfig" | ".env" => FileFormat::Plain,
                ".bashrc" | ".zshrc" | ".profile" | ".bash_profile" | ".zprofile" => {
                    FileFormat::Code("Bash".into())
                }
                _ => FileFormat::Plain,
            }
        }
    }
}

/// Used for stdin/pipes where we have no file extension.
pub fn detect_from_content(content: &str) -> FileFormat {
    let bytes = content.as_bytes();

    // Check for binary image formats via magic bytes
    if bytes.len() >= 12 {
        // PNG
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return FileFormat::Image;
        }
        // JPEG
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return FileFormat::Image;
        }
        // GIF
        if bytes.starts_with(b"GIF8") {
            return FileFormat::Image;
        }
        // WebP (RIFF....WEBP)
        if &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            return FileFormat::Image;
        }
        // BMP
        if bytes.starts_with(b"BM") && bytes.len() > 14 {
            return FileFormat::Image;
        }
    }

    let trimmed = content.trim_start();

    // JSON
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        if serde_json::from_str::<serde_json::Value>(content).is_ok() {
            return FileFormat::Json;
        }
    }

    // HTML
    if trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<!doctype")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<HTML")
    {
        return FileFormat::Code("HTML".into());
    }

    // XML
    if trimmed.starts_with("<?xml") {
        return FileFormat::Code("XML".into());
    }

    // Shebang
    if trimmed.starts_with("#!") {
        let first_line = trimmed.lines().next().unwrap_or("");
        if first_line.contains("python") {
            return FileFormat::Code("Python".into());
        }
        if first_line.contains("ruby") {
            return FileFormat::Code("Ruby".into());
        }
        if first_line.contains("node") || first_line.contains("deno") || first_line.contains("bun")
        {
            return FileFormat::Code("JavaScript".into());
        }
        if first_line.contains("bash") || first_line.contains("/sh") {
            return FileFormat::Code("Bash".into());
        }
        if first_line.contains("perl") {
            return FileFormat::Code("Perl".into());
        }
    }

    // Diff
    if trimmed.starts_with("diff --git")
        || trimmed.starts_with("--- ")
        || trimmed.starts_with("+++ ")
    {
        return FileFormat::Code("Diff".into());
    }

    // Markdown heuristics
    let lines: Vec<&str> = trimmed.lines().take(20).collect();
    let md_score = lines.iter().filter(|l| {
        l.starts_with('#')
            || l.starts_with("- ")
            || l.starts_with("* ")
            || l.starts_with("> ")
            || l.starts_with("```")
            || l.contains("**")
            || l.contains("](")
    }).count();

    if md_score >= 2 {
        return FileFormat::Markdown;
    }

    FileFormat::Plain
}

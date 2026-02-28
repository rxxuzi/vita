use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::process;

mod detect;
mod info;
mod output;
mod render;
mod theme;

use detect::{detect_format, FileFormat};
use output::Output;
use theme::Theme;

/// vita - Universal File Viewer
/// cat with eyes. See everything beautifully.
#[derive(Parser, Debug)]
#[command(name = "vita", about, long_about = None, disable_version_flag = true)]
struct Cli {
    /// Files to display
    #[arg()]
    files: Vec<PathBuf>,

    /// Print version
    #[arg(short = 'v', long = "version")]
    show_version: bool,

    /// Show line numbers
    #[arg(short = 'n', long = "number")]
    line_numbers: bool,

    /// Color theme (--list-themes to see all)
    #[arg(short = 't', long = "theme", default_value = "dracula")]
    theme: String,

    /// Max image width in characters
    #[arg(short = 'w', long = "width", default_value_t = 60)]
    width: u32,

    /// Force language for syntax highlighting
    #[arg(short = 'l', long = "lang")]
    lang: Option<String>,

    /// Plain output (no formatting)
    #[arg(short = 'p', long = "plain")]
    plain: bool,

    /// Raw mode: syntax coloring without format rendering
    #[arg(short = 'r', long = "raw")]
    raw: bool,

    /// Show file info header
    #[arg(short = 'i', long = "info")]
    info: bool,

    /// Brief: show structural outline
    #[arg(short = 'b', long = "brief")]
    brief: bool,

    /// Grep: show only lines matching PATTERN with highlight
    #[arg(short = 'g', long = "grep")]
    grep: Option<String>,

    /// Show only the first N lines
    #[arg(long = "head")]
    head: Option<usize>,

    /// Show only the last N lines
    #[arg(long = "tail")]
    tail: Option<usize>,

    /// List available themes
    #[arg(long = "list-themes")]
    list_themes: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.show_version {
        println!("vita {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if cli.list_themes {
        Theme::list_all();
        return;
    }

    if cli.head.is_some() && cli.tail.is_some() {
        eprintln!("vita: --head and --tail cannot be used together");
        process::exit(1);
    }

    if cli.brief && cli.grep.is_some() {
        eprintln!("vita: --brief and --grep cannot be used together");
        process::exit(1);
    }

    let theme = match Theme::from_name(&cli.theme) {
        Some(t) => t,
        None => {
            eprintln!("vita: unknown theme '{}'", cli.theme);
            eprintln!();
            Theme::list_all_to(&mut io::stderr());
            process::exit(1);
        }
    };
    let out = Output::new(!cli.plain && io::stdout().is_terminal());

    if cli.brief {
        return run_brief(&cli, &theme, &out);
    }

    if let Some(ref pattern) = cli.grep {
        return run_grep(&cli, pattern, &theme, &out);
    }

    if cli.files.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("vita: no input. Use 'vita --help' for usage.");
            process::exit(1);
        }

        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("vita: failed to read stdin");
            process::exit(1);
        }

        let buf = truncate_lines(&buf, cli.head, cli.tail);

        let format = cli
            .lang
            .as_deref()
            .map(|l| FileFormat::Code(l.to_string()))
            .unwrap_or_else(|| detect::detect_from_content(&buf));

        if cli.info {
            info::print_header(None, Some(&format), Some(&buf), &theme, &out);
        }
        render_content(&buf, &format, &cli, &theme, &out);
        return;
    }

    let multi = cli.files.len() > 1;

    for path in &cli.files {
        if path.to_str() == Some("-") {
            let mut buf = String::new();
            if io::stdin().read_to_string(&mut buf).is_ok() {
                let buf = truncate_lines(&buf, cli.head, cli.tail);
                let format = detect::detect_from_content(&buf);
                if cli.info {
                    info::print_header(None, Some(&format), Some(&buf), &theme, &out);
                }
                render_content(&buf, &format, &cli, &theme, &out);
            }
            continue;
        }

        if !path.exists() {
            eprintln!("vita: '{}': No such file or directory", path.display());
            continue;
        }

        if multi {
            out.file_separator(&path.display().to_string(), &theme);
        }

        let format = cli
            .lang
            .as_deref()
            .map(|l| FileFormat::Code(l.to_string()))
            .unwrap_or_else(|| detect_format(path));

        match &format {
            FileFormat::Image => {
                if cli.info {
                    info::print_header(Some(path), Some(&format), None, &theme, &out);
                }
                render::image::render(path, cli.width, &theme, &out);
            }
            _ => match std::fs::read_to_string(path) {
                Ok(content) => {
                    let content = truncate_lines(&content, cli.head, cli.tail);
                    if cli.info {
                        info::print_header(Some(path), Some(&format), Some(&content), &theme, &out);
                    }
                    render_content(&content, &format, &cli, &theme, &out);
                }
                Err(e) => {
                    eprintln!("vita: '{}': {}", path.display(), e);
                }
            },
        }
    }
}

fn run_brief(cli: &Cli, theme: &Theme, out: &Output) {
    if cli.files.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("vita: no input. Use 'vita --help' for usage.");
            process::exit(1);
        }

        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("vita: failed to read stdin");
            process::exit(1);
        }

        let buf = truncate_lines(&buf, cli.head, cli.tail);
        let format = cli
            .lang
            .as_deref()
            .map(|l| FileFormat::Code(l.to_string()))
            .unwrap_or_else(|| detect::detect_from_content(&buf));

        if cli.info {
            info::print_header(None, Some(&format), Some(&buf), theme, out);
        }
        render::brief::render(&buf, &format, theme, out);
        return;
    }

    let multi = cli.files.len() > 1;

    for path in &cli.files {
        if path.to_str() == Some("-") {
            let mut buf = String::new();
            if io::stdin().read_to_string(&mut buf).is_ok() {
                let buf = truncate_lines(&buf, cli.head, cli.tail);
                let format = detect::detect_from_content(&buf);
                if cli.info {
                    info::print_header(None, Some(&format), Some(&buf), theme, out);
                }
                render::brief::render(&buf, &format, theme, out);
            }
            continue;
        }

        if !path.exists() {
            eprintln!("vita: '{}': No such file or directory", path.display());
            continue;
        }

        if multi {
            out.file_separator(&path.display().to_string(), theme);
        }

        let format = cli
            .lang
            .as_deref()
            .map(|l| FileFormat::Code(l.to_string()))
            .unwrap_or_else(|| detect_format(path));

        if matches!(format, FileFormat::Image) {
            continue;
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let content = truncate_lines(&content, cli.head, cli.tail);
                if cli.info {
                    info::print_header(Some(path), Some(&format), Some(&content), theme, out);
                }
                render::brief::render(&content, &format, theme, out);
            }
            Err(e) => eprintln!("vita: '{}': {}", path.display(), e),
        }
    }
}

fn run_grep(cli: &Cli, pattern: &str, theme: &Theme, out: &Output) {
    if cli.files.is_empty() {
        if io::stdin().is_terminal() {
            eprintln!("vita: no input. Use 'vita --help' for usage.");
            process::exit(1);
        }

        let mut buf = String::new();
        if io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("vita: failed to read stdin");
            process::exit(1);
        }

        let buf = truncate_lines(&buf, cli.head, cli.tail);
        if cli.info {
            info::print_header(None, None, Some(&buf), theme, out);
        }
        render::grep::render(&buf, pattern, theme, out);
        return;
    }

    let multi = cli.files.len() > 1;

    for path in &cli.files {
        if path.to_str() == Some("-") {
            let mut buf = String::new();
            if io::stdin().read_to_string(&mut buf).is_ok() {
                let buf = truncate_lines(&buf, cli.head, cli.tail);
                if cli.info {
                    info::print_header(None, None, Some(&buf), theme, out);
                }
                render::grep::render(&buf, pattern, theme, out);
            }
            continue;
        }

        if !path.exists() {
            eprintln!("vita: '{}': No such file or directory", path.display());
            continue;
        }

        if multi {
            out.file_separator(&path.display().to_string(), theme);
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let content = truncate_lines(&content, cli.head, cli.tail);
                if cli.info {
                    let format = detect_format(path);
                    info::print_header(Some(path), Some(&format), Some(&content), theme, out);
                }
                render::grep::render(&content, pattern, theme, out);
            }
            Err(e) => eprintln!("vita: '{}': {}", path.display(), e),
        }
    }
}

fn truncate_lines(content: &str, head: Option<usize>, tail: Option<usize>) -> String {
    if let Some(n) = head {
        content.lines().take(n).collect::<Vec<_>>().join("\n")
    } else if let Some(n) = tail {
        let lines: Vec<&str> = content.lines().collect();
        let skip = lines.len().saturating_sub(n);
        lines[skip..].join("\n")
    } else {
        content.to_string()
    }
}

fn render_content(content: &str, format: &FileFormat, cli: &Cli, theme: &Theme, out: &Output) {
    if cli.plain {
        print!("{}", content);
        return;
    }

    if cli.raw {
        let lang = match format {
            FileFormat::Markdown => "Markdown",
            FileFormat::Json => "JSON",
            FileFormat::Csv => "Plain Text",
            FileFormat::Code(lang) => lang.as_str(),
            FileFormat::Plain => "Plain Text",
            FileFormat::Image => return,
        };
        render::code::render(content, lang, cli.line_numbers, theme, out);
        return;
    }

    match format {
        FileFormat::Markdown => render::markdown::render(content, theme, out),
        FileFormat::Json => render::json::render(content, theme, out),
        FileFormat::Csv => render::csv::render(content, theme, out),
        FileFormat::Code(lang) => {
            render::code::render(content, lang, cli.line_numbers, theme, out)
        }
        FileFormat::Image => {}
        FileFormat::Plain => render::plain::render(content, cli.line_numbers, theme, out),
    }
}

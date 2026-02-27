use clap::Parser;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::process;

mod detect;
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

    /// Color theme
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

    let theme = Theme::from_name(&cli.theme);
    let out = Output::new(!cli.plain && io::stdout().is_terminal());

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

        let format = cli
            .lang
            .as_deref()
            .map(|l| FileFormat::Code(l.to_string()))
            .unwrap_or_else(|| detect::detect_from_content(&buf));

        render_content(&buf, &format, &cli, &theme, &out);
        return;
    }

    let multi = cli.files.len() > 1;

    for path in &cli.files {
        if path.to_str() == Some("-") {
            let mut buf = String::new();
            if io::stdin().read_to_string(&mut buf).is_ok() {
                let format = detect::detect_from_content(&buf);
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
                render::image::render(path, cli.width, &theme, &out);
            }
            _ => match std::fs::read_to_string(path) {
                Ok(content) => render_content(&content, &format, &cli, &theme, &out),
                Err(e) => {
                    // Might be binary
                    eprintln!("vita: '{}': {}", path.display(), e);
                }
            },
        }
    }
}

fn render_content(content: &str, format: &FileFormat, cli: &Cli, theme: &Theme, out: &Output) {
    if cli.plain {
        print!("{}", content);
        return;
    }

    // Raw mode: syntax coloring only, skip format-specific rendering
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
        FileFormat::Image => {} // handled separately
        FileFormat::Plain => render::plain::render(content, cli.line_numbers, theme, out),
    }
}
use std::path::Path;
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;

use crate::detect::FileFormat;
use crate::output::Output;
use crate::theme::Theme;

pub fn print_header(
    path: Option<&Path>,
    format: Option<&FileFormat>,
    content: Option<&str>,
    theme: &Theme,
    out: &Output,
) {
    let filename = path
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("stdin");

    let mut segments: Vec<String> = Vec::new();

    if let Some(fmt) = format {
        segments.push(format_language(fmt).to_string());
    }

    match format {
        Some(FileFormat::Image) => {
            if let Some(p) = path {
                if let Some((w, h)) = image_dimensions(p) {
                    segments.push(format!("{}x{}", w, h));
                }
            }
        }
        _ => {
            if let Some(c) = content {
                let n = c.lines().count();
                segments.push(if n == 1 {
                    "1 line".to_string()
                } else {
                    format!("{} lines", n)
                });
            }
        }
    }

    if let Some(p) = path {
        if let Ok(meta) = std::fs::metadata(p) {
            segments.push(format_size(meta.len()));
        }
    } else if let Some(c) = content {
        segments.push(format_size(c.len() as u64));
    }

    // Modified time — only for real files
    if let Some(p) = path {
        if let Ok(meta) = std::fs::metadata(p) {
            if let Ok(modified) = meta.modified() {
                segments.push(format_relative_time(modified));
            }
        }
    }

    let sep = " \u{2502} ";
    let mut display_width: usize = 2; // "─ "
    display_width += UnicodeWidthStr::width(filename);
    let joined = segments.join(sep);
    if !segments.is_empty() {
        display_width += 3; // " │ " between filename and segments
        display_width += UnicodeWidthStr::width(joined.as_str());
    }
    display_width += 1; // trailing space before fill

    let fill_count = (out.term_width as usize).saturating_sub(display_width);
    let fill: String = "\u{2500}".repeat(fill_count);

    out.colored("\u{2500} ", theme.hr);
    out.bold_colored(filename, theme.file_header);
    if !segments.is_empty() {
        out.colored(" \u{2502} ", theme.hr);
        out.dim(&joined, theme.line_number);
    }
    out.colored(&format!(" {}", fill), theme.hr);
    println!();
}

fn format_language(format: &FileFormat) -> &str {
    match format {
        FileFormat::Markdown => "Markdown",
        FileFormat::Json => "JSON",
        FileFormat::Csv => "CSV",
        FileFormat::Toml => "TOML",
        FileFormat::Yaml => "YAML",
        FileFormat::Code(lang) => lang.as_str(),
        FileFormat::Image => "Image",
        FileFormat::Plain => "Plain Text",
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn format_relative_time(modified: SystemTime) -> String {
    let elapsed = match modified.elapsed() {
        Ok(d) => d,
        Err(_) => return "just now".to_string(),
    };
    let secs = elapsed.as_secs();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 2_592_000 {
        format!("{}d ago", secs / 86400)
    } else if secs < 31_536_000 {
        format!("{}mo ago", secs / 2_592_000)
    } else {
        format!("{}y ago", secs / 31_536_000)
    }
}

fn image_dimensions(path: &Path) -> Option<(u32, u32)> {
    image::image_dimensions(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(4300), "4.2 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_format_language() {
        assert_eq!(format_language(&FileFormat::Markdown), "Markdown");
        assert_eq!(format_language(&FileFormat::Json), "JSON");
        assert_eq!(format_language(&FileFormat::Csv), "CSV");
        assert_eq!(format_language(&FileFormat::Image), "Image");
        assert_eq!(format_language(&FileFormat::Plain), "Plain Text");
        assert_eq!(format_language(&FileFormat::Code("Rust".into())), "Rust");
    }

    #[test]
    fn test_format_relative_time() {
        use std::time::Duration;

        let now = SystemTime::now();
        assert_eq!(format_relative_time(now), "just now");

        let ago_5m = now - Duration::from_secs(300);
        assert_eq!(format_relative_time(ago_5m), "5m ago");

        let ago_2h = now - Duration::from_secs(7200);
        assert_eq!(format_relative_time(ago_2h), "2h ago");

        let ago_3d = now - Duration::from_secs(259200);
        assert_eq!(format_relative_time(ago_3d), "3d ago");

        let ago_14d = now - Duration::from_secs(1_209_600);
        assert_eq!(format_relative_time(ago_14d), "14d ago");

        let ago_3mo = now - Duration::from_secs(7_776_000);
        assert_eq!(format_relative_time(ago_3mo), "3mo ago");

        let ago_2y = now - Duration::from_secs(63_072_000);
        assert_eq!(format_relative_time(ago_2y), "2y ago");
    }
}

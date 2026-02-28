use crossterm::style::{self, Color, Stylize};
use std::io::{self, Write};

use crate::theme::Theme;

pub struct Output {
    pub use_colors: bool,
    pub term_width: u16,
}

impl Output {
    pub fn new(use_colors: bool) -> Self {
        let term_width = terminal_size::terminal_size()
            .map(|(w, _)| w.0)
            .unwrap_or(80);

        Self {
            use_colors,
            term_width,
        }
    }

    pub fn colored(&self, text: &str, color: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(color));
        } else {
            print!("{}", text);
        }
    }

    pub fn bold_colored(&self, text: &str, color: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(color).bold());
        } else {
            print!("{}", text);
        }
    }

    pub fn italic_colored(&self, text: &str, color: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(color).italic());
        } else {
            print!("{}", text);
        }
    }

    pub fn underline_colored(&self, text: &str, color: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(color).underlined());
        } else {
            print!("{}", text);
        }
    }

    pub fn strike_colored(&self, text: &str, color: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(color).crossed_out());
        } else {
            print!("{}", text);
        }
    }

    pub fn dim(&self, text: &str, color: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(color).dim());
        } else {
            print!("{}", text);
        }
    }

    pub fn colored_bg(&self, text: &str, fg: Color, bg: Color) {
        if self.use_colors {
            print!("{}", style::style(text).with(fg).on(bg));
        } else {
            print!("{}", text);
        }
    }

    pub fn hyperlink_start(&self, url: &str) {
        if self.use_colors {
            print!("\x1b]8;;{}\x1b\\", url);
        }
    }

    pub fn hyperlink_end(&self) {
        if self.use_colors {
            print!("\x1b]8;;\x1b\\");
        }
    }

    #[allow(dead_code)]
    pub fn reset(&self) {
        if self.use_colors {
            print!("{}", style::style("").reset());
        }
    }

    /// File separator for multi-file output
    pub fn file_separator(&self, filename: &str, theme: &Theme) {
        println!();
        self.colored("━━━ ", theme.hr);
        self.bold_colored(filename, theme.file_header);
        self.colored(" ━━━", theme.hr);
        println!();
        println!();
    }

    #[allow(dead_code)]
    pub fn flush(&self) {
        let _ = io::stdout().flush();
    }
}

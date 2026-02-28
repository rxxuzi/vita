//! Markdown renderer - Claude Code inspired style
//!
//! Design principles:
//!   - Clean, minimal, readable
//!   - Headings: `# ` prefix with bold color, no decorative underlines
//!   - Code blocks: full-width background with language label
//!   - Lists: consistent `- ` or `N. ` style
//!   - Blockquotes: thin `│` bar with dimmed text
//!   - Tables: simple box-drawing borders

use crossterm::style::Color;
use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use unicode_width::UnicodeWidthStr;

use crate::output::Output;
use crate::theme::Theme;

pub fn render(content: &str, theme: &Theme, out: &Output) {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;

    let parser = Parser::new_ext(content, options);
    let mut ctx = RenderContext::new(theme, out);
    ctx.process(parser);
}

struct RenderContext<'a> {
    theme: &'a Theme,
    out: &'a Output,

    in_heading: Option<u8>,
    in_bold: bool,
    in_italic: bool,
    in_strike: bool,
    in_code_block: bool,
    in_block_quote: u32, // nesting depth
    in_link: bool,
    link_url: String,
    code_lang: String,
    code_buffer: String,
    list_stack: Vec<ListState>,
    need_newline: bool,
    in_table: bool,
    table_alignments: Vec<Alignment>,
    table_row: Vec<String>,
    table_rows: Vec<Vec<String>>,
    table_is_header: bool,
    in_table_cell: bool,
    cell_buffer: String,
    in_paragraph: bool,
}

#[derive(Clone)]
enum ListState {
    Ordered(u64),
    Unordered,
}

impl<'a> RenderContext<'a> {
    fn new(theme: &'a Theme, out: &'a Output) -> Self {
        Self {
            theme,
            out,
            in_heading: None,
            in_bold: false,
            in_italic: false,
            in_strike: false,
            in_code_block: false,
            in_block_quote: 0,
            in_link: false,
            link_url: String::new(),
            code_lang: String::new(),
            code_buffer: String::new(),
            list_stack: Vec::new(),
            need_newline: false,
            in_table: false,
            table_alignments: Vec::new(),
            table_row: Vec::new(),
            table_rows: Vec::new(),
            table_is_header: false,
            in_table_cell: false,
            cell_buffer: String::new(),
            in_paragraph: false,
        }
    }

    fn process(&mut self, parser: Parser) {
        for event in parser {
            match event {
                Event::Start(tag) => self.start_tag(tag),
                Event::End(tag) => self.end_tag(tag),
                Event::Text(text) => self.text(&text),
                Event::Code(code) => self.inline_code(&code),
                Event::SoftBreak => self.soft_break(),
                Event::HardBreak => {
                    println!();
                    self.print_indent();
                }
                Event::Rule => self.rule(),
                Event::TaskListMarker(checked) => self.task_marker(checked),
                Event::FootnoteReference(name) => {
                    self.out.colored("[", self.theme.link);
                    self.out.bold_colored(&name, self.theme.link);
                    self.out.colored("]", self.theme.link);
                }
                _ => {}
            }
        }
        // Ensure final newline
        println!();
    }

    // ─── Tag Start ────────────────────────────────────────────

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Heading { level, .. } => {
                let lvl = heading_level(level);
                self.in_heading = Some(lvl);
                self.ensure_blank_line();

                // Print `# `, `## `, etc. prefix
                let color = self.heading_color(lvl);
                let prefix = "#".repeat(lvl as usize);
                self.out.bold_colored(&prefix, color);
                self.out.bold_colored(" ", color);
            }

            Tag::Paragraph => {
                if self.in_block_quote > 0 {
                    self.print_quote_bar();
                } else if self.list_stack.is_empty() {
                    self.ensure_blank_line();
                }
                self.in_paragraph = true;
            }

            Tag::BlockQuote => {
                self.in_block_quote += 1;
                if self.in_block_quote == 1 {
                    self.ensure_blank_line();
                }
            }

            Tag::CodeBlock(kind) => {
                self.in_code_block = true;
                self.code_buffer.clear();
                self.code_lang = match &kind {
                    CodeBlockKind::Fenced(lang) => {
                        let l = lang.split_whitespace().next().unwrap_or("").to_string();
                        l
                    }
                    _ => String::new(),
                };
                self.ensure_blank_line();
            }

            Tag::List(start) => {
                if self.list_stack.is_empty() {
                    self.ensure_blank_line();
                }
                match start {
                    Some(n) => self.list_stack.push(ListState::Ordered(n)),
                    None => self.list_stack.push(ListState::Unordered),
                }
            }

            Tag::Item => {
                if self.need_newline {
                    println!();
                }
                self.print_list_indent();

                if let Some(state) = self.list_stack.last_mut() {
                    match state {
                        ListState::Ordered(n) => {
                            self.out.colored(&format!("{}. ", n), self.theme.list_bullet);
                            *n += 1;
                        }
                        ListState::Unordered => {
                            self.out.colored("- ", self.theme.list_bullet);
                        }
                    }
                }
            }

            Tag::Emphasis => self.in_italic = true,
            Tag::Strong => self.in_bold = true,
            Tag::Strikethrough => self.in_strike = true,

            Tag::Link { dest_url, .. } => {
                self.in_link = true;
                self.link_url = dest_url.to_string();
                self.out.hyperlink_start(&self.link_url);
            }

            Tag::Image { dest_url, .. } => {
                self.in_link = true;
                self.link_url = dest_url.to_string();
                self.out.colored("[image: ", self.theme.link_url);
            }

            Tag::Table(alignments) => {
                self.in_table = true;
                self.table_alignments = alignments;
                self.table_rows.clear();
                self.ensure_blank_line();
            }

            Tag::TableHead => {
                self.table_is_header = true;
                self.table_row = Vec::new();
            }

            Tag::TableRow => {
                self.table_row = Vec::new();
            }

            Tag::TableCell => {
                self.in_table_cell = true;
                self.cell_buffer.clear();
            }

            _ => {}
        }
    }

    // ─── Tag End ──────────────────────────────────────────────

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                println!();
                self.in_heading = None;
                self.need_newline = true;
            }

            TagEnd::Paragraph => {
                println!();
                self.in_paragraph = false;
                self.need_newline = true;
            }

            TagEnd::BlockQuote => {
                self.in_block_quote = self.in_block_quote.saturating_sub(1);
                self.need_newline = true;
            }

            TagEnd::CodeBlock => {
                self.render_code_block();
                self.in_code_block = false;
                self.code_lang.clear();
                self.code_buffer.clear();
                self.need_newline = true;
            }

            TagEnd::List(_) => {
                self.list_stack.pop();
                if self.list_stack.is_empty() {
                    self.need_newline = true;
                }
            }

            TagEnd::Item => {
                println!();
                self.need_newline = false;
            }

            TagEnd::Emphasis => self.in_italic = false,
            TagEnd::Strong => self.in_bold = false,
            TagEnd::Strikethrough => self.in_strike = false,

            TagEnd::Link => {
                self.out.hyperlink_end();
                self.in_link = false;
                self.link_url.clear();
            }

            TagEnd::Image => {
                if !self.link_url.is_empty() {
                    self.out.dim(&self.link_url, self.theme.link_url);
                }
                self.out.colored("]", self.theme.link_url);
                self.in_link = false;
                self.link_url.clear();
            }

            TagEnd::Table => {
                self.render_table();
                self.in_table = false;
                self.table_rows.clear();
                self.need_newline = true;
            }

            TagEnd::TableHead => {
                self.table_rows.push(self.table_row.clone());
                self.table_is_header = false;
            }

            TagEnd::TableRow => {
                self.table_rows.push(self.table_row.clone());
            }

            TagEnd::TableCell => {
                self.in_table_cell = false;
                self.table_row.push(self.cell_buffer.clone());
            }

            _ => {}
        }
    }

    // ─── Text Content ─────────────────────────────────────────

    fn text(&mut self, text: &str) {
        if self.in_table_cell {
            self.cell_buffer.push_str(text);
            return;
        }

        if self.in_code_block {
            self.code_buffer.push_str(text);
            return;
        }

        // Block quote: render inline (bar printed at paragraph start)
        if self.in_block_quote > 0 && self.in_heading.is_none() {
            self.out.italic_colored(text, self.theme.quote);
            return;
        }

        let color = if let Some(lvl) = self.in_heading {
            self.heading_color(lvl)
        } else if self.in_link {
            self.theme.link
        } else {
            self.theme.text
        };

        if self.in_heading.is_some() {
            self.out.bold_colored(text, color);
        } else if self.in_bold && self.in_italic {
            self.out.bold_colored(text, self.theme.bold);
        } else if self.in_bold {
            self.out.bold_colored(text, self.theme.bold);
        } else if self.in_italic {
            self.out.italic_colored(text, color);
        } else if self.in_strike {
            self.out.strike_colored(text, self.theme.strike);
        } else if self.in_link {
            self.out.underline_colored(text, color);
        } else {
            self.out.colored(text, color);
        }
    }

    fn inline_code(&mut self, code: &str) {
        if self.in_table_cell {
            self.cell_buffer.push('`');
            self.cell_buffer.push_str(code);
            self.cell_buffer.push('`');
            return;
        }

        // ` code ` with background
        self.out.colored_bg(
            &format!(" {} ", code),
            self.theme.code_fg,
            self.theme.code_bg,
        );
    }

    fn soft_break(&mut self) {
        if self.in_table_cell {
            self.cell_buffer.push(' ');
            return;
        }
        if self.in_code_block {
            self.code_buffer.push('\n');
            return;
        }
        if self.in_block_quote > 0 {
            println!();
            self.print_quote_bar();
        } else {
            println!();
        }
    }

    fn rule(&mut self) {
        self.ensure_blank_line();
        let width = self.content_width().min(60);
        self.out.colored(&"─".repeat(width), self.theme.hr);
        println!();
        self.need_newline = true;
    }

    fn task_marker(&mut self, checked: bool) {
        if checked {
            self.out.colored("[✓] ", self.theme.alert_tip);
        } else {
            self.out.colored("[ ] ", self.theme.hr);
        }
    }

    // ─── Code Block Rendering ─────────────────────────────────

    fn render_code_block(&self) {
        let content = &self.code_buffer;
        let width = self.content_width();
        let bg = self.theme.code_block_bg;

        let (bg_r, bg_g, bg_b) = if let Color::Rgb { r, g, b } = bg {
            (r, g, b)
        } else {
            (30, 30, 46) // fallback dark bg
        };

        if !self.code_lang.is_empty() {
            self.out.dim(&format!("  {}", self.code_lang), self.theme.code_lang);
            println!();
        }

        let highlighted = if !self.code_lang.is_empty() {
            self.highlight_code(content, &self.code_lang)
        } else {
            None
        };

        let (def_r, def_g, def_b) = if let Color::Rgb { r, g, b } = self.theme.code_block_fg {
            (r, g, b)
        } else {
            (205, 214, 244)
        };

        // Raw ANSI only — crossterm styled() resets bg between fragments
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            print!("  \x1b[48;2;{};{};{}m", bg_r, bg_g, bg_b);

            if let Some(ref hl_lines) = highlighted {
                if i < hl_lines.len() {
                    print!(" ");
                    for (fg, text) in &hl_lines[i] {
                        // Set fg+bg together, no reset between fragments
                        if let Color::Rgb { r, g, b } = fg {
                            print!("\x1b[38;2;{};{};{}m{}", r, g, b, text);
                        } else {
                            print!("{}", text);
                        }
                    }
                } else {
                    print!("\x1b[38;2;{};{};{}m {}", def_r, def_g, def_b, line);
                }
            } else {
                print!("\x1b[38;2;{};{};{}m {}", def_r, def_g, def_b, line);
            }

            let visible_len = line.len() + 1;
            if visible_len < width {
                for _ in 0..(width - visible_len) {
                    print!(" ");
                }
            }
            print!(" \x1b[0m");
            println!();
        }
    }

    fn highlight_code(&self, code: &str, lang: &str) -> Option<Vec<Vec<(Color, String)>>> {
        let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
        let ts = syntect::highlighting::ThemeSet::load_defaults();

        let syntax = ss
            .find_syntax_by_token(lang)
            .or_else(|| ss.find_syntax_by_extension(lang))
            .or_else(|| ss.find_syntax_by_name(lang))
            .or_else(|| {
                // Fallback for unsupported languages (e.g. TypeScript → JavaScript)
                let fb = crate::detect::syntax_fallback(lang);
                ss.find_syntax_by_name(fb)
                    .or_else(|| ss.find_syntax_by_token(fb))
            })?;

        let st = ts
            .themes
            .get(self.theme.syntect_theme)
            .or_else(|| ts.themes.get("base16-eighties.dark"))
            .unwrap_or_else(|| ts.themes.values().next().unwrap());

        let mut h = syntect::easy::HighlightLines::new(syntax, st);
        let mut result = Vec::new();

        for line in syntect::util::LinesWithEndings::from(code) {
            if let Ok(ranges) = h.highlight_line(line, &ss) {
                let fragments: Vec<(Color, String)> = ranges
                    .iter()
                    .map(|(style, text)| {
                        let (mut r, mut g, mut b) = (
                            style.foreground.r,
                            style.foreground.g,
                            style.foreground.b,
                        );
                        // Perceived brightness (ITU-R BT.601)
                        let fg_bright = (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
                        let bg_bright = if let Color::Rgb { r: br, g: bg, b: bb } = self.theme.code_block_bg {
                            (br as u32 * 299 + bg as u32 * 587 + bb as u32 * 114) / 1000
                        } else { 35 };
                        let contrast = if fg_bright > bg_bright {
                            fg_bright - bg_bright
                        } else {
                            bg_bright - fg_bright
                        };
                        if contrast < 60 {
                            if let Color::Rgb { r: dr, g: dg, b: db } = self.theme.code_block_fg {
                                r = dr; g = dg; b = db;
                            } else {
                                r = 205; g = 214; b = 244;
                            }
                        }
                        let color = Color::Rgb { r, g, b };
                        (color, text.trim_end_matches('\n').to_string())
                    })
                    .collect();
                result.push(fragments);
            } else {
                result.push(vec![(self.theme.code_block_fg, line.to_string())]);
            }
        }

        Some(result)
    }

    // ─── Table Rendering ──────────────────────────────────────

    fn render_table(&self) {
        if self.table_rows.is_empty() {
            return;
        }

        let col_count = self.table_rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut widths = vec![3usize; col_count];

        for row in &self.table_rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_count {
                    widths[i] = widths[i].max(cell.width());
                }
            }
        }

        let border = self.theme.table_border;

        // Top border
        print!("  ");
        self.out.colored("┌", border);
        for (i, w) in widths.iter().enumerate() {
            self.out.colored(&"─".repeat(*w + 2), border);
            self.out
                .colored(if i < col_count - 1 { "┬" } else { "┐" }, border);
        }
        println!();

        // Rows
        for (r, row) in self.table_rows.iter().enumerate() {
            print!("  ");
            self.out.colored("│", border);

            for (c, w) in widths.iter().enumerate() {
                let cell = row.get(c).map(|s| s.as_str()).unwrap_or("");
                let padding = w.saturating_sub(cell.width());

                let (left, right) = match self.table_alignments.get(c) {
                    Some(Alignment::Center) => (padding / 2, padding - padding / 2),
                    Some(Alignment::Right) => (padding, 0),
                    _ => (0, padding),
                };

                print!(" ");
                for _ in 0..left {
                    print!(" ");
                }

                if r == 0 {
                    self.out.bold_colored(cell, self.theme.table_header);
                } else {
                    self.out.colored(cell, self.theme.text);
                }

                for _ in 0..right {
                    print!(" ");
                }
                print!(" ");
                self.out.colored("│", border);
            }
            println!();

            // Separator after header
            if r == 0 && self.table_rows.len() > 1 {
                print!("  ");
                self.out.colored("├", border);
                for (i, w) in widths.iter().enumerate() {
                    self.out.colored(&"─".repeat(*w + 2), border);
                    self.out.colored(
                        if i < col_count - 1 { "┼" } else { "┤" },
                        border,
                    );
                }
                println!();
            }
        }

        // Bottom border
        print!("  ");
        self.out.colored("└", border);
        for (i, w) in widths.iter().enumerate() {
            self.out.colored(&"─".repeat(*w + 2), border);
            self.out
                .colored(if i < col_count - 1 { "┴" } else { "┘" }, border);
        }
        println!();
    }

    // ─── Helpers ──────────────────────────────────────────────

    fn heading_color(&self, level: u8) -> Color {
        match level {
            1 => self.theme.heading1,
            2 => self.theme.heading2,
            3 => self.theme.heading3,
            4 => self.theme.heading4,
            5 => self.theme.heading3,
            _ => self.theme.heading4,
        }
    }

    fn content_width(&self) -> usize {
        (self.out.term_width as usize).saturating_sub(4).min(100)
    }

    fn ensure_blank_line(&mut self) {
        if self.need_newline {
            println!();
            self.need_newline = false;
        }
    }

    fn print_quote_bar(&self) {
        for _ in 0..self.in_block_quote {
            print!("  ");
            self.out.colored("│ ", self.theme.quote_bar);
        }
    }

    fn print_list_indent(&self) {
        let depth = self.list_stack.len().saturating_sub(1);
        for _ in 0..depth {
            print!("    ");
        }
    }

    fn print_indent(&self) {
        if self.in_block_quote > 0 {
            self.print_quote_bar();
        }
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

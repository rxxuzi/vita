# vita

> `cat` with eyes. See everything beautifully.

A universal file viewer for the terminal, written in Rust.  
vita doesn't just print files — it **understands** them and renders each format beautifully.

## Install

```bash
git clone https://github.com/rxxuzi/vita.git
cd vita
cargo build --release
```

The binary will be at `target/release/vita`. Add it to your PATH.

## Usage

```bash
vita README.md           # Rendered Markdown
vita main.rs             # Syntax highlighted code
vita data.json           # Rainbow brackets + colored values
vita table.csv           # Pastel colored table
vita photo.png           # Image in terminal

vita a.txt b.txt         # Multiple files
cat log.txt | vita       # Pipe support (auto-detects format)
git diff | vita          # Colored diff
```

## Supported Formats

| Format | What vita does |
|--------|----------------|
| **Markdown** | Full GFM rendering — headings, tables, code blocks, task lists, alerts |
| **Source Code** | Syntax highlighting for 90+ languages |
| **JSON** | Pretty-print with rainbow brackets by nesting depth |
| **CSV / TSV** | Colored table with auto-delimiter detection |
| **Images** | Terminal rendering using half-block characters (▀▄) |
| **Plain Text** | Clean display with optional line numbers |

## Themes

Five built-in themes:

- **dracula** (default)
- **tokyonight**
- **catppuccin**
- **nord**
- **gruvbox**

```bash
vita --list-themes
vita -t catppuccin README.md
```

## Language Support

90+ languages with syntax highlighting. Native support for C, C++, Rust, Python, JavaScript, Go, Java, Ruby, PHP, Haskell, Scala, and more.

## Cross-Platform

vita runs on **Linux**, **macOS**, and **Windows**. Built with Rust and cross-platform crates.

## License

[GPL-3.0](LICENSE)

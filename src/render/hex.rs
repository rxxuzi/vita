use crate::output::Output;
use crate::theme::Theme;

const BYTES_PER_LINE: usize = 16;

pub fn render(data: &[u8], head: Option<usize>, tail: Option<usize>, theme: &Theme, out: &Output) {
    let total_lines = (data.len() + BYTES_PER_LINE - 1) / BYTES_PER_LINE;

    let (start_line, end_line) = if let Some(n) = head {
        (0, n.min(total_lines))
    } else if let Some(n) = tail {
        (total_lines.saturating_sub(n), total_lines)
    } else {
        (0, total_lines)
    };

    for line_idx in start_line..end_line {
        let offset = line_idx * BYTES_PER_LINE;
        let chunk_end = (offset + BYTES_PER_LINE).min(data.len());
        let chunk = &data[offset..chunk_end];

        out.dim(&format!("{:08x}", offset), theme.hex_offset);
        out.colored(" │ ", theme.line_number);

        for i in 0..BYTES_PER_LINE {
            if i > 0 && i % 4 == 0 {
                print!(" ");
            }
            if i < chunk.len() {
                let b = chunk[i];
                if b == 0 {
                    out.dim("00 ", theme.hex_byte);
                } else {
                    out.colored(&format!("{:02x} ", b), theme.hex_byte);
                }
            } else {
                print!("   ");
                if i > 0 && i % 4 == 0 {
                    // already printed group space above
                }
            }
        }

        out.colored("│ ", theme.line_number);

        for &b in chunk {
            let ch = if (0x20..=0x7e).contains(&b) {
                b as char
            } else {
                '.'
            };
            out.colored(&ch.to_string(), theme.hex_ascii);
        }

        println!();
    }
}

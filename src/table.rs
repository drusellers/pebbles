use colored::Colorize;

pub struct SimpleTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl SimpleTable {
    pub fn new(headers: Vec<String>) -> Self {
        let column_widths = headers.iter().map(|h| strip_ansi(h).len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            column_widths,
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        // Update column widths based on content
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                let content_width = strip_ansi(cell).len();
                if content_width > self.column_widths[i] {
                    self.column_widths[i] = content_width;
                }
            }
        }
        self.rows.push(row);
    }

    pub fn print(&self) {
        // Calculate total width for the border
        // Each cell has: " content " = width + 2 spaces
        // Plus separators between cells, plus left/right borders
        let content_width: usize = self.column_widths.iter().map(|w| w + 2).sum();
        let separator_width = self.column_widths.len().saturating_sub(1);
        let total_width = content_width + separator_width + 2;

        // Print top border
        println!("╭{}╮", "─".repeat(total_width - 2));

        // Print headers
        let header_row = self.format_row(&self.headers);
        println!("│{}│", header_row);

        // Print separator
        let sep_parts: Vec<String> = self
            .column_widths
            .iter()
            .map(|w| "─".repeat(w + 2))
            .collect();
        println!("├{}┤", sep_parts.join("┼"));

        // Print data rows
        for (i, row) in self.rows.iter().enumerate() {
            let data_row = self.format_row(row);
            println!("│{}│", data_row);

            // Print row separator (except for last row)
            if i < self.rows.len() - 1 {
                let sep_parts: Vec<String> = self
                    .column_widths
                    .iter()
                    .map(|w| "─".repeat(w + 2))
                    .collect();
                println!("├{}┤", sep_parts.join("┼"));
            }
        }

        // Print bottom border
        println!("╰{}╯", "─".repeat(total_width - 2));
    }

    fn format_row(&self, row: &[String]) -> String {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let width = self.column_widths.get(i).copied().unwrap_or(10);
                let content_width = strip_ansi(cell).len();
                let padding = width.saturating_sub(content_width);
                format!(" {}{} ", cell, " ".repeat(padding))
            })
            .collect();
        cells.join("│")
    }
}

fn strip_ansi(s: &str) -> String {
    // Simple ANSI escape code stripper
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            // Skip the escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // skip '['
                              // Skip until we find a letter (end of sequence)
                while let Some(c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        chars.next();
                        break;
                    }
                    chars.next();
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#![allow(clippy::inherent_to_string, dead_code)]

pub struct SimpleTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
    borderless: bool,
}

impl SimpleTable {
    pub fn new(headers: Vec<String>) -> Self {
        let column_widths = headers.iter().map(|h| strip_ansi(h).len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            column_widths,
            borderless: false,
        }
    }

    pub fn borderless(headers: Vec<String>) -> Self {
        let column_widths = headers.iter().map(|h| strip_ansi(h).len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            column_widths,
            borderless: true,
        }
    }

    pub fn set_borderless(&mut self, borderless: bool) {
        self.borderless = borderless;
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
        if self.borderless {
            self.print_borderless();
        } else {
            self.print_bordered();
        }
    }

    fn print_bordered(&self) {
        // Calculate total width for the border
        // Each cell has: " content " = width + 2 spaces
        // Plus separators between cells, plus left/right borders
        let content_width: usize = self.column_widths.iter().map(|w| w + 2).sum();
        let separator_width = self.column_widths.len().saturating_sub(1);
        let total_width = content_width + separator_width + 2;

        // Print top border
        println!("╭{}╮", "─".repeat(total_width - 2));

        // Print headers
        let header_row = self.format_row_bordered(&self.headers);
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
            let data_row = self.format_row_bordered(row);
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

    fn print_borderless(&self) {
        // Print headers
        let header_row = self.format_row_borderless(&self.headers);
        println!("{}", header_row);

        // Print separator line under headers
        let total_width: usize = self.column_widths.iter().map(|w| w + 2).sum::<usize>()
            + self.column_widths.len().saturating_sub(1);
        println!("{}", "─".repeat(total_width));

        // Print data rows
        for row in self.rows.iter() {
            let data_row = self.format_row_borderless(row);
            println!("{}", data_row);
        }
    }

    fn format_row_bordered(&self, row: &[String]) -> String {
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

    fn format_row_borderless(&self, row: &[String]) -> String {
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
        cells.join(" ")
    }

    /// Generate the table output as a string (for testing)
    pub fn to_string(&self) -> String {
        let mut output = String::new();

        if self.borderless {
            self.to_string_borderless(&mut output);
        } else {
            self.to_string_bordered(&mut output);
        }

        output
    }

    fn to_string_bordered(&self, output: &mut String) {
        let content_width: usize = self.column_widths.iter().map(|w| w + 2).sum();
        let separator_width = self.column_widths.len().saturating_sub(1);
        let total_width = content_width + separator_width + 2;

        output.push_str(&format!("╭{}╮\n", "─".repeat(total_width - 2)));

        let header_row = self.format_row_bordered(&self.headers);
        output.push_str(&format!("│{}│\n", header_row));

        let sep_parts: Vec<String> = self
            .column_widths
            .iter()
            .map(|w| "─".repeat(w + 2))
            .collect();
        output.push_str(&format!("├{}┤\n", sep_parts.join("┼")));

        for (i, row) in self.rows.iter().enumerate() {
            let data_row = self.format_row_bordered(row);
            output.push_str(&format!("│{}│\n", data_row));

            if i < self.rows.len() - 1 {
                let sep_parts: Vec<String> = self
                    .column_widths
                    .iter()
                    .map(|w| "─".repeat(w + 2))
                    .collect();
                output.push_str(&format!("├{}┤\n", sep_parts.join("┼")));
            }
        }

        output.push_str(&format!("╰{}╯\n", "─".repeat(total_width - 2)));
    }

    fn to_string_borderless(&self, output: &mut String) {
        let header_row = self.format_row_borderless(&self.headers);
        output.push_str(&format!("{}\n", header_row));

        let total_width: usize = self.column_widths.iter().map(|w| w + 2).sum::<usize>()
            + self.column_widths.len().saturating_sub(1);
        output.push_str(&format!("{}\n", "─".repeat(total_width)));

        for row in self.rows.iter() {
            let data_row = self.format_row_borderless(row);
            output.push_str(&format!("{}\n", data_row));
        }
    }

    #[cfg(test)]
    pub fn get_column_widths(&self) -> &Vec<usize> {
        &self.column_widths
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_table_bordered() {
        let mut table = SimpleTable::new(vec![
            "ID".to_string(),
            "Status".to_string(),
            "Title".to_string(),
        ]);

        table.add_row(vec![
            "abc1".to_string(),
            "open".to_string(),
            "First issue".to_string(),
        ]);
        table.add_row(vec![
            "def2".to_string(),
            "done".to_string(),
            "Second issue".to_string(),
        ]);

        let output = table.to_string();

        // Check for bordered table structure
        assert!(output.contains("╭"), "Should have top border");
        assert!(output.contains("╮"), "Should have top right corner");
        assert!(output.contains("│"), "Should have vertical borders");
        assert!(output.contains("├"), "Should have left separator");
        assert!(output.contains("┤"), "Should have right separator");
        assert!(output.contains("╰"), "Should have bottom left corner");
        assert!(output.contains("╯"), "Should have bottom right corner");

        // Check content
        assert!(output.contains("ID"), "Should contain header ID");
        assert!(output.contains("abc1"), "Should contain first row ID");
        assert!(output.contains("def2"), "Should contain second row ID");
    }

    #[test]
    fn test_simple_table_borderless() {
        let mut table = SimpleTable::borderless(vec![
            "ID".to_string(),
            "Status".to_string(),
            "Title".to_string(),
        ]);

        table.add_row(vec![
            "abc1".to_string(),
            "open".to_string(),
            "First issue".to_string(),
        ]);
        table.add_row(vec![
            "def2".to_string(),
            "done".to_string(),
            "Second issue".to_string(),
        ]);

        let output = table.to_string();

        // Check for borderless table structure
        assert!(!output.contains("│"), "Should not have vertical borders");
        assert!(!output.contains("╭"), "Should not have box corners");
        assert!(!output.contains("╮"), "Should not have box corners");
        assert!(!output.contains("╰"), "Should not have box corners");
        assert!(!output.contains("╯"), "Should not have box corners");

        // Should have header separator line
        assert!(output.contains("─"), "Should have header separator");

        // Check content
        assert!(output.contains("ID"), "Should contain header ID");
        assert!(output.contains("abc1"), "Should contain first row ID");
        assert!(output.contains("def2"), "Should contain second row ID");
    }

    #[test]
    fn test_column_widths_auto_adjust() {
        let mut table = SimpleTable::new(vec!["ID".to_string(), "Title".to_string()]);

        // Initial width based on headers
        assert_eq!(table.get_column_widths()[0], 2); // "ID"
        assert_eq!(table.get_column_widths()[1], 5); // "Title"

        // Add a row with longer content
        table.add_row(vec!["abc123".to_string(), "A very long title".to_string()]);

        // Widths should be updated to fit content
        assert_eq!(table.get_column_widths()[0], 6); // "abc123"
        assert_eq!(table.get_column_widths()[1], 17); // "A very long title"
    }

    #[test]
    fn test_strip_ansi() {
        let colored_text = "\x1B[32mGreen\x1B[0m";
        let stripped = strip_ansi(colored_text);
        assert_eq!(stripped, "Green");
        assert_eq!(stripped.len(), 5);
        assert_ne!(colored_text.len(), stripped.len());
    }

    #[test]
    fn test_empty_table_bordered() {
        let table = SimpleTable::new(vec!["ID".to_string(), "Status".to_string()]);

        let output = table.to_string();

        // Even empty table should have structure
        assert!(output.contains("ID"), "Should contain header");
        assert!(output.contains("Status"), "Should contain header");
        assert!(output.contains("╭"), "Should have borders");
    }

    #[test]
    fn test_empty_table_borderless() {
        let table = SimpleTable::borderless(vec!["ID".to_string(), "Status".to_string()]);

        let output = table.to_string();

        // Even empty table should have structure
        assert!(output.contains("ID"), "Should contain header");
        assert!(output.contains("Status"), "Should contain header");
        assert!(output.contains("─"), "Should have header separator");
        assert!(!output.contains("│"), "Should not have vertical borders");
    }

    #[test]
    fn test_set_borderless_toggle() {
        let mut table = SimpleTable::new(vec!["ID".to_string(), "Status".to_string()]);

        // Initially bordered
        assert!(!table.borderless);

        // Toggle to borderless
        table.set_borderless(true);
        assert!(table.borderless);

        // Toggle back
        table.set_borderless(false);
        assert!(!table.borderless);
    }

    #[test]
    fn test_row_with_colored_content() {
        let mut table = SimpleTable::borderless(vec!["S".to_string()]);

        // Add row with ANSI colors
        let colored_status = "\x1B[32mdone\x1B[0m".to_string();
        table.add_row(vec![colored_status]);

        // Width should be calculated based on visible text only (4 for "done")
        // Header "S" is 1, so content wins
        assert_eq!(table.get_column_widths()[0], 4); // "done", not including ANSI codes
    }
}

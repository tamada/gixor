use terminal_size::{Width, Height};
/// Represents a terminal that can format strings in columns.
pub(crate) struct Terminal {
    pub(crate) width: usize,
    pub(crate) padding: String,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new("  ")
    }
}

impl Terminal {
    pub fn new<S: AsRef<str>>(padding: S) -> Self {
        let (cols, _rows) = match terminal_size::terminal_size() {
            Some((Width(w), Height(h))) => (w, h),
            _ => (80, 24),
        };
        Self::new_with(cols as usize, padding)
    }

    pub fn new_with<S: AsRef<str>>(width: usize, padding: S) -> Self {
        let padding = padding.as_ref().to_string();
        Self { width, padding }
    }

    fn get_max_length_and_number_of_columns(&self, list: &[String]) -> (usize, usize) {
        let max_length = list.iter().map(|item| item.len()).max().unwrap_or(0);
        let number_of_columns: usize = (self.width + 1) / (max_length + self.padding.len());
        if number_of_columns == 0 {
            (max_length, 1)
        } else {
            (max_length, number_of_columns)
        }
    }

    pub(crate) fn format_header(&self, header: String) -> String {
        let deco_len = self.width - header.len() - 2;
        let deco_prefix = "=".repeat(deco_len / 2);
        let deco_suffix = "=".repeat(deco_len - deco_prefix.len());
        format!("{} {} {}", deco_prefix, header, deco_suffix)
    }

    pub(crate) fn format_in_column(&self, items: Vec<String>) -> Vec<String> {
        let (max_length, number_of_columns) = self.get_max_length_and_number_of_columns(&items);
        let targets = padding_list(items, max_length);
        let mut result = vec![];
        let mut line = Vec::<u8>::new();
        for (i, item) in targets.iter().enumerate() {
            line.extend(item.as_bytes());
            if i % number_of_columns == (number_of_columns - 1) || i == targets.len() - 1 {
                let r = String::from_utf8(line.clone()).unwrap();
                result.push(r.trim().to_string());
                line.clear();
            } else {
                line.extend(self.padding.as_bytes());
            }
        }
        result
    }
}

fn padding_list(list: Vec<String>, max_length: usize) -> Vec<String> {
    let mut result = vec![];
    for item in list {
        let mut padded_item = item.clone();
        for _ in 0..max_length - item.len() {
            padded_item.push(' ');
        }
        result.push(padded_item);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_print_in_column_string() {
        let v1 = vec![
            "macOS",
            "Linux",
            "Windows",
            "Go",
            "VisualStudioCode",
            "JetBrains",
        ];
        let t = Terminal::new_with(125, " ");
        let r1 = t.format_in_column(v1.iter().map(|s| s.to_string()).collect());
        assert_eq!(r1.len(), 1);
        assert_eq!(r1[0], "macOS            Linux            Windows          Go               VisualStudioCode JetBrains");

        let v2 = vec![
            "macOS",
            "Linux",
            "Windows",
            "Go",
            "VisualStudioCode",
            "JetBrains",
            "Rust",
            "NetBeans",
        ];
        let t = Terminal::new_with(125, " ");
        let r2 = t.format_in_column(v2.iter().map(|s| s.to_string()).collect());
        assert_eq!(r2.len(), 2);
        assert_eq!(r2[0], "macOS            Linux            Windows          Go               VisualStudioCode JetBrains        Rust");
        assert_eq!(r2[1], "NetBeans");
    }

    #[test]
    pub fn test_format_header() {
        let t = Terminal::new_with(80, " ");
        assert_eq!(
            t.format_header("Hello, World!".to_string()),
            "================================ Hello, World! ================================="
        );
    }
}

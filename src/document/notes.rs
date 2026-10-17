//! How a note is written.

/// A note as the one-line items it is written as: each line its own item, in
/// order. A line ends at `\n` or `\r`, the breaks `-n/--note` refuses. A
/// blank line only ever separated paragraphs inside an old block scalar, so
/// blank lines are dropped, and a CRLF pair leaves no item between. A note
/// with no other line stays one item with its text as it was, so no note
/// disappears.
pub(super) fn lines(note: &str) -> Vec<&str> {
    let lines: Vec<&str> = note
        .split(['\n', '\r'])
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.is_empty() { vec![note] } else { lines }
}

#[cfg(test)]
mod tests {
    use super::lines;

    #[test]
    fn a_note_is_one_item_per_line_without_blank_lines() {
        assert_eq!(lines("Short context."), ["Short context."]);
        assert_eq!(
            lines("Lead: the design.\n1. First.\n\n  \n2. Second."),
            ["Lead: the design.", "1. First.", "2. Second."]
        );
        assert_eq!(
            lines("First.\r\nSecond.\rThird."),
            ["First.", "Second.", "Third."]
        );
        assert_eq!(lines(" \n\n"), [" \n\n"]);
    }
}

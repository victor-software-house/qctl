//! How a note is written.

use ctl_core::input::Input;
use yamled::TextStyle;

/// A note with line breaks is a literal block, so each break stays where it
/// was typed. A one-line note is plain when plain text reads back as itself,
/// and otherwise a folded block, so a colon or a quote in prose never turns
/// into escaped text.
pub(super) fn style(note: &str) -> TextStyle {
    if note.contains('\n') {
        TextStyle::Literal
    } else if reads_back_plain(note) {
        TextStyle::Auto
    } else {
        TextStyle::Folded
    }
}

fn reads_back_plain(note: &str) -> bool {
    Input::new("note", format!("- {note}"))
        .parse::<Vec<String>>()
        .is_ok_and(|notes| notes.as_slice() == [note])
}

#[cfg(test)]
mod tests {
    use super::style;
    use yamled::TextStyle;

    #[test]
    fn a_note_is_plain_folded_or_literal() {
        assert_eq!(style("Short context."), TextStyle::Auto);
        assert_eq!(
            style("Source: the colon stays readable."),
            TextStyle::Folded
        );
        assert_eq!(style("First line.\nSecond line."), TextStyle::Literal);
    }
}

pub(super) fn id_of(text: &str) -> Option<&str> {
    text.lines().find_map(|line| {
        line.trim_start()
            .trim_start_matches("- ")
            .strip_prefix("id:")
            .map(|value| value.trim().trim_matches(['"', '\'']))
    })
}

pub(super) fn dedent(text: &str, spaces: usize) -> String {
    text.lines()
        .map(|line| line.get(spaces..).unwrap_or(line.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn indent_by(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{pad}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

//! Hidden `--usage-spec` and the served-task mount line.

mod common;

use common::{qctl, stderr, stdout};
use std::fs;
use std::path::Path;

fn crate_file(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

#[test]
fn usage_spec_is_the_mounted_q_grammar() {
    let output = qctl(&["--usage-spec=q"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let spec = stdout(&output);
    assert!(spec.lines().any(|line| line == "name q"), "{spec}");
    assert!(spec.lines().any(|line| line == "bin q"), "{spec}");
    insta::with_settings!({
        filters => vec![(r#"version "\d+\.\d+\.\d+""#, r#"version "<version>""#)],
        omit_expression => true,
    }, {
        insta::assert_snapshot!(spec);
    });
}

#[test]
fn usage_spec_bare_defaults_to_q() {
    let output = qctl(&["--usage-spec"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), stdout(&qctl(&["--usage-spec=q"])));
}

#[test]
fn template_mount_line_is_ctl_core() {
    let template = crate_file(".ctl/templates/q.jinja");
    let line = template
        .lines()
        .find(|line| line.starts_with("#USAGE mount"))
        .unwrap_or_else(|| panic!("no #USAGE mount in template:\n{template}"));
    assert_eq!(line, ctl_core::mount_line("q"));
}

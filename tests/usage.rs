//! Hidden `--usage-spec` and the served-task mount line.

mod common;

use common::{qctl, stderr, stdout};

#[test]
fn usage_spec_prints_kdl_named_q() {
    let output = qctl(&["--usage-spec=q"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("name"), "{out}");
    assert!(out.contains("status"), "{out}");
    assert!(out.contains("check"), "{out}");
}

#[test]
fn usage_spec_bare_defaults_to_q() {
    let output = qctl(&["--usage-spec"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("name"), "{out}");
}

#[test]
fn served_task_mount_matches_ctl_core() {
    let template = include_str!("../.ctl/templates/q.jinja");
    assert!(template.contains(&ctl_core::mount_line("q")), "{template}");
}

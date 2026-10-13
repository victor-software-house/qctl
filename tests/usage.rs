//! Hidden `--usage-spec` and the served-task mount line.

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use common::{qctl, stderr, stdout};
use indoc::indoc;

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
        filters => vec![(r#"version "[^"]+""#, r#"version "<version>""#)],
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
fn template_mount_is_ctl_core() {
    let template = crate_file(".ctl/templates/q.jinja");
    let directives = ctl_core::mount_directives("q");
    assert!(
        template.contains(&directives),
        "template must carry ctl_core::mount_directives(\"q\"):\n{directives}\n\n{template}"
    );
}

/// In prefix output mode, which mise uses for several tasks at once or when
/// `MISE_TASK_OUTPUT=prefix` is set, mise relays a task's output itself unless the task is `raw`, and appends an
/// SGR reset to every relayed line whenever its own colour is on, which
/// `CLICOLOR_FORCE=1` turns on even into a pipe. The served task, rendered
/// from its template, runs in a scrubbed environment with a stub `qctl` on
/// PATH, so neither the caller's mise configuration nor a download is
/// involved.
#[test]
fn the_served_task_pipes_json_without_an_escape_byte() {
    let mut environment = minijinja::Environment::new();
    environment.set_keep_trailing_newline(true);
    let task = environment
        .render_str(
            &crate_file(".ctl/templates/q.jinja"),
            minijinja::context! { versions => minijinja::context! { qctl => "0.0.0" } },
        )
        .unwrap_or_else(|error| panic!("render q.jinja: {error}"));
    let root = tempfile::TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let write = |relative: &str, body: &str| {
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap_or(root.path()))
            .unwrap_or_else(|error| panic!("mkdir for {relative}: {error}"));
        fs::write(&path, body).unwrap_or_else(|error| panic!("write {relative}: {error}"));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .unwrap_or_else(|error| panic!("chmod {relative}: {error}"));
    };
    write(
        "mise.toml",
        indoc! {r#"
            [task_config]
            includes = ["tasks"]
        "#},
    );
    write("tasks/q", &task);
    write(
        "bin/qctl",
        indoc! {r#"
            #!/bin/sh
            printf '{"ok":true}\n'
        "#},
    );
    let path = std::env::join_paths(std::iter::once(root.path().join("bin")).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap_or_else(|error| panic!("PATH: {error}"));
    let output = std::process::Command::new("mise")
        .args(["run", "q", "status", "--format", "json"])
        .current_dir(root.path())
        .env_clear()
        .env("PATH", path)
        .env("HOME", std::env::var_os("HOME").unwrap_or_default())
        .env("MISE_CONFIG_DIR", root.path().join("mise-config"))
        .env("MISE_TRUSTED_CONFIG_PATHS", root.path())
        .env("MISE_DISABLE_TOOLS", "github:victor-software-house/qctl")
        .env("MISE_TASK_OUTPUT", "prefix")
        .env("CLICOLOR_FORCE", "1")
        .env("FORCE_COLOR", "3")
        .env("COLORTERM", "truecolor")
        .output()
        .unwrap_or_else(|error| panic!("this test needs mise on PATH: {error}"));
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "{\"ok\":true}\n",
        "stdout must be exactly what the CLI printed"
    );
}

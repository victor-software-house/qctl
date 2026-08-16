use indoc::indoc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

pub struct LedgerDir {
    _root: TempDir,
    pub path: PathBuf,
}

impl LedgerDir {
    pub fn empty() -> Self {
        let root = TempDir::new().expect("tempdir");
        let path = root.path().join("tasks.yaml");
        Self { _root: root, path }
    }

    pub fn write(&self, body: &str) {
        fs::write(&self.path, body).expect("write ledger");
    }

    pub fn parent(&self) -> &Path {
        self.path.parent().expect("parent")
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.path).expect("read ledger")
    }
}

pub fn qctl(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qctl"))
        .args(args)
        .env_remove("TASKS_LEDGER")
        .output()
        .expect("spawn qctl")
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub const MINIMAL: &str = indoc! {"
    schema_version: 1
    prefix: QCTL
    active: null
    queue: []
    archive: []
    horizon: []
"};

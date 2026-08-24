use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn domain_modules_return_data_and_ctl_core_owns_presentation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = rust_files(&root.join("src"));
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        for print in ["println!(", "eprintln!(", "print!(", "eprint!("] {
            assert!(
                !source.contains(print),
                "{} prints through {print}",
                path.display()
            );
        }
        let name = path.file_name().unwrap().to_string_lossy();
        if matches!(
            name.as_ref(),
            "main.rs" | "cli.rs" | "operator_docs.rs" | "presentation.rs"
        ) {
            continue;
        }
        assert!(
            !source.contains("ctl_core::"),
            "{} crosses the presentation boundary",
            path.display()
        );
    }

    let cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    for dependency in [
        "anstream =",
        "anstyle =",
        "comfy-table =",
        "unicode-width =",
    ] {
        assert!(
            !cargo.contains(dependency),
            "qctl directly depends on {dependency}"
        );
    }
}

fn rust_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files
}

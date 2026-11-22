//! `layout` is the only Layout package other engines may depend on
//! (AGENTS.md §1.3). Cargo cannot forbid a dependency, so this guard scans the
//! workspace manifests: no package outside `crates/layout` may name an
//! internal Layout crate, directly or through a `package = "..."` alias.

use std::path::{Path, PathBuf};

const INTERNAL: &[&str] = &[
    "layout_assembly",
    "layout_core",
    "layout_commit",
    "layout_invalidation",
    "layout_reuse",
    "layout_algo_",
];

/// Manifests outside `crates/layout` temporarily allowed to name a Layout
/// internal. Keep it empty; an entry needs a recorded migration stage.
const ALLOWED: &[&str] = &[];

fn manifests(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !matches!(name.as_ref(), "target" | ".git" | ".claude" | "node_modules") {
                manifests(&path, out);
            }
        } else if name == "Cargo.toml" {
            out.push(path);
        }
    }
}

fn names_internal_crate(line: &str) -> Option<&'static str> {
    let line = line.split('#').next().unwrap_or("");
    INTERNAL.iter().copied().find(|internal| {
        let key = line.trim_start();
        key.starts_with(internal) || line.contains(&format!("package = \"{internal}"))
    })
}

#[test]
fn only_the_facade_depends_on_layout_internals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut found = Vec::new();
    for dir in ["crates", "apps"] {
        manifests(&root.join(dir), &mut found);
    }
    assert!(!found.is_empty(), "no manifests found under {}", root.display());
    let mut violations = Vec::new();
    for manifest in found {
        let relative = manifest
            .strip_prefix(&root)
            .unwrap_or(&manifest)
            .to_string_lossy()
            .replace('\\', "/");
        if relative.starts_with("crates/layout/") || ALLOWED.contains(&relative.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).unwrap();
        for (number, line) in text.lines().enumerate() {
            if let Some(internal) = names_internal_crate(line) {
                violations.push(format!("{relative}:{}: {internal}: {}", number + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "depend on the `layout` facade instead of Layout internals:\n{}",
        violations.join("\n")
    );
}

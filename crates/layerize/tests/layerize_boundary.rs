//! Main/impl compositor split guard (crates/compositor/MAIN_IMPL_SPLIT.md).
//!
//! `layerize` is the main-thread half and never depends on `compositor`.
//! The impl half (`crates/compositor`) may name only `layerize::commit` and
//! must not name Blink paint property nodes, which stay on the main thread.
//! Cargo cannot express either rule, so this scans manifests and sources.

use std::path::{Path, PathBuf};

const PAINT_PROPERTY_NODES: &[&str] = &[
    "TransformPaintPropertyNode",
    "ClipPaintPropertyNode",
    "EffectPaintPropertyNode",
    "ScrollPaintPropertyNode",
];

/// Compositor files that still hold main-half code naming paint property
/// nodes. Each entry moves into `layerize` in stage S1a; remove it then.
/// Do not add entries.
const PROPERTY_NODE_ALLOWED: &[&str] = &[];

/// Temporary (S1a only): EngineCore / EngineAssembly hold the main-half
/// engine and call it where they ran the main half; removed in S1b, when
/// Page owns `layerize::LayerizeEngine`.
const LAYERIZE_ENGINE_ALLOWED: &[&str] = &[
    "crates/compositor/src/runtime/engine_core.rs",
    "crates/compositor/src/runtime/engine_assembly.rs",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !matches!(name.as_ref(), "target" | ".git" | ".claude") {
                files(&path, out);
            }
        } else if name.ends_with(".rs") || name == "Cargo.toml" {
            out.push(path);
        }
    }
}

fn relative(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Code text of a line: `//` comments carry source citations, not paths.
fn code(line: &str) -> &str {
    line.split("//").next().unwrap_or("")
}

#[test]
fn layerize_does_not_depend_on_compositor() {
    let manifest = std::fs::read_to_string(root().join("crates/layerize/Cargo.toml")).unwrap();
    let violations: Vec<_> = manifest
        .lines()
        .map(|line| line.split('#').next().unwrap_or(""))
        .filter(|line| {
            let key = line.trim_start();
            key.starts_with("compositor ")
                || key.starts_with("compositor=")
                || line.contains("package = \"compositor\"")
                || line.contains("path = \"../compositor\"")
        })
        .collect();
    assert!(
        violations.is_empty(),
        "layerize must not depend on the compositor facade: {violations:?}"
    );
}

#[test]
fn compositor_names_only_the_commit_vocabulary() {
    let mut found = Vec::new();
    files(&root().join("crates/compositor"), &mut found);
    assert!(!found.is_empty(), "no compositor sources found");
    let mut violations = Vec::new();
    for path in found {
        let relative = relative(&path);
        let text = std::fs::read_to_string(&path).unwrap();
        for (number, line) in text.lines().enumerate() {
            let line = code(line);
            for (index, _) in line.match_indices("layerize::") {
                let preceded = line[..index]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_');
                if !preceded
                    && !line[index..].starts_with("layerize::commit")
                    && !(line[index..].starts_with("layerize::LayerizeEngine")
                        && LAYERIZE_ENGINE_ALLOWED.contains(&relative.as_str()))
                {
                    violations.push(format!("{relative}:{}: {}", number + 1, line.trim()));
                }
            }
            if !PROPERTY_NODE_ALLOWED.contains(&relative.as_str()) {
                for node in PAINT_PROPERTY_NODES {
                    if line.contains(node) {
                        violations.push(format!("{relative}:{}: {node}", number + 1));
                    }
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "the impl compositor may use only layerize::commit and no paint property nodes:\n{}",
        violations.join("\n")
    );
}

#[test]
fn property_node_allowance_has_no_stale_entries() {
    for entry in PROPERTY_NODE_ALLOWED {
        let path = root().join(entry);
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            PAINT_PROPERTY_NODES.iter().any(|node| text.contains(node)),
            "{entry} no longer names a paint property node; remove it from the allowance"
        );
    }
}

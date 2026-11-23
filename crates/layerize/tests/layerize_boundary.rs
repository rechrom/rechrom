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

/// Temporary (S1a-3 B2 until B3): compositor re-export stubs of the moved
/// main-half modules (Blink compositing/paint translations, their adapter
/// families and the Paint-value bindings), which EngineCore and
/// paint_services still name at their old paths. Each file is a single
/// `pub use layerize::...` line; delete it when its last user moves.
/// Do not add entries.
const MAIN_HALF_REEXPORT_ALLOWED: &[&str] = &[
    "crates/compositor/src/adapter/content_layer_client_impl.rs",
    "crates/compositor/src/adapter/paint_artifact_compositor.rs",
    "crates/compositor/src/adapter/paint_chunks_to_cc_layer.rs",
    "crates/compositor/src/adapter/paint_geometry.rs",
    "crates/compositor/src/adapter/pending_layer.rs",
    "crates/compositor/src/adapter/property_tree_manager.rs",
    "crates/compositor/src/adapter/raster_invalidator.rs",
    "crates/compositor/src/runtime/paint.rs",
    // Re-exports the main paint services next to the impl lease (B3).
    "crates/compositor/src/runtime/paint_services.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/adjust_mask_layer_geometry.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/chunk_to_layer_mapper.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/content_layer_client_impl.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/layers_as_json.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/paint_artifact_compositor.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/paint_chunks_to_cc_layer.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/pending_layer.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/compositing/property_tree_manager.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/display_item_raster_invalidator.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/float_clip_rect.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/geometry_mapper.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/geometry_mapper_clip_cache.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/geometry_mapper_transform_cache.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/raster_invalidation_tracking.rs",
    "crates/compositor/src/third_party/blink/renderer/platform/graphics/paint/raster_invalidator.rs",
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
                    && !MAIN_HALF_REEXPORT_ALLOWED.contains(&relative.as_str())
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

#[test]
fn main_half_reexport_allowance_has_no_stale_entries() {
    for entry in MAIN_HALF_REEXPORT_ALLOWED {
        let text = std::fs::read_to_string(root().join(entry)).unwrap_or_default();
        let names_main_half = text.lines().map(code).any(|line| {
            line.match_indices("layerize::")
                .any(|(index, _)| !line[index..].starts_with("layerize::commit"))
        });
        assert!(
            names_main_half,
            "{entry} no longer names the layerize main half; remove it from the allowance"
        );
    }
}

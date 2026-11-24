//! Main/impl compositor split guard (crates/compositor/MAIN_IMPL_SPLIT.md).
//!
//! `layerize` is the main-thread half and never depends on `compositor`.
//! The impl half (`crates/compositor`) may name only `layerize::commit` and
//! must not name Blink paint property nodes, which stay on the main thread.
//! Cargo cannot express either rule, so this scans manifests and sources.
//!
//! The S4 package classification (crates/compositor/S4_CRATE_CLASSIFICATION.md)
//! extends this: every package under `crates/compositor/` and
//! `crates/layerize/` is a cc shared value root, an impl package, a main
//! package or one of the two facades, and dependencies follow the class.

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
                if !preceded && !line[index..].starts_with("layerize::commit") {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    /// cc/gfx/viz values and algorithms both sides use; no lifecycle.
    Shared,
    /// Code only LayerTreeHostImpl, ProxyImpl, the Scheduler, tiles, raster
    /// or draw run.
    Impl,
    /// Code only the main thread runs (LayerTreeHost, Blink layerization).
    Main,
    /// The impl side's Page-facing engine package.
    ImplFacade,
    /// The main side's Page-facing engine package.
    MainFacade,
}

use Class::*;

/// The S4 classification (S4_CRATE_CLASSIFICATION.md section 2). Every
/// package whose manifest lives under crates/compositor/ or crates/layerize/
/// appears exactly once.
const CLASSIFICATION: &[(&str, Class)] = &[
    ("scroll_snap", Shared),
    ("scroll_animation", Shared),
    ("compositor_tree", Shared),
    ("compositor_animation", Shared),
    ("compositor_tree_sync", Impl),
    ("compositor_input", Impl),
    ("compositor_draw", Impl),
    ("compositor_scheduler", Impl),
    ("compositor_tile", Impl),
    ("compositor_frame", Impl),
    ("compositor_resource", Impl),
    ("compositor_raster", Impl),
    ("compositor_layerize", Main),
    ("compositor", ImplFacade),
    ("layerize", MainFacade),
];

/// Main-side dependencies on impl packages that S4 is still cutting:
/// `(from, to, step that removes it)`. Do not add entries.
const LAYERIZE_IMPL_DEPENDENCY_ALLOWED: &[(&str, &str, &str)] = &[
    ("layerize", "compositor_input", "S4-4c"),
    ("layerize", "compositor_draw", "S4-4c"),
    ("layerize", "compositor_scheduler", "S4-4b"),
    ("layerize", "compositor_tile", "S4-4b"),
    ("layerize", "compositor_frame", "S4-4b"),
    ("layerize", "compositor_resource", "S4-4b"),
];

/// Shared-root dependencies on impl packages that S4 is still cutting.
/// Do not add entries.
const SHARED_ROOT_IMPL_DEPENDENCY_ALLOWED: &[(&str, &str, &str)] = &[];

/// Non-compositor value crates a shared root may name (gfx geometry, tracing,
/// Skia values, base types, image values).
const SHARED_ROOT_VALUE_DEPENDENCIES: &[&str] =
    &["paint_model", "browser_tracing", "skia", "foundation", "image_resource"];

/// Value modules of the mixed `compositor_input` that layerize may name
/// until S4-4 cuts them. Remove with `layerize_names_only_shared_modules`.
const COMPOSITOR_INPUT_VALUE_MODULES: &[&str] = &[
    "actively_scrolling_type",
    "browser_controls_params",
    "browser_controls_state",
    "compositor_commit_data",
    "event_listener_properties",
    "hit_test_opaqueness",
    "input_types",
    "layer_selection_bound",
    "main_thread_scrolling_reason",
    "overscroll_behavior",
    "page_scale_animation",
    "scroll_decision",
    "scroll_hit_test_rect",
    "scroll_snap_data",
    "scroll_source_type",
    "scrollbar",
    "snap_selection_strategy",
    "touch_action",
    "touch_action_region",
    "ScrollGranularity",
    // The ScrollbarTransform facet; it moves with gfx::Transform in S4-4c.
    "scrollbar_controller_boundary",
];

struct Manifest {
    path: String,
    package: String,
    dependencies: Vec<String>,
}

fn manifests() -> Vec<Manifest> {
    let mut found = Vec::new();
    for dir in ["crates", "apps"] {
        files(&root().join(dir), &mut found);
    }
    let mut out = Vec::new();
    for path in found {
        if path.file_name().is_some_and(|name| name == "Cargo.toml") {
            let text = std::fs::read_to_string(&path).unwrap();
            if let Some(package) = package_name(&text) {
                out.push(Manifest {
                    path: relative(&path),
                    package,
                    dependencies: dependencies(&text),
                });
            }
        }
    }
    assert!(!out.is_empty(), "no manifests found under {}", root().display());
    out
}

fn string_value(line: &str, key: &str) -> Option<String> {
    let rest = line.split_once(key)?.1.trim_start().strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    Some(rest.split('"').next()?.to_string())
}

/// `[package] name` of a manifest; `None` for a virtual workspace manifest.
fn package_name(text: &str) -> Option<String> {
    let mut in_package = false;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package && line.starts_with("name") {
            return string_value(line, "name");
        }
    }
    None
}

/// Package names of `[dependencies]`, `[dev-dependencies]` and
/// `[build-dependencies]` (also target-specific and `[dependencies.<name>]`
/// tables), resolving `package = "..."` renames.
fn dependencies(text: &str) -> Vec<String> {
    const KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
    let mut out = Vec::new();
    let mut in_list = false;
    let mut table: Option<String> = None;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            out.extend(table.take());
            let header = line.trim_matches(|c| c == '[' || c == ']');
            in_list = KINDS
                .iter()
                .any(|kind| header == *kind || header.ends_with(&format!(".{kind}")));
            table = KINDS
                .iter()
                .find_map(|kind| header.strip_prefix(&format!("{kind}.")))
                .map(str::to_string);
            continue;
        }
        if let Some(name) = table.as_mut() {
            if let Some(package) = line.starts_with("package").then(|| string_value(line, "package")).flatten() {
                *name = package;
            }
        } else if in_list {
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim().trim_matches('"').to_string();
            out.push(string_value(value, "package").unwrap_or(key));
        }
    }
    out.extend(table);
    out
}

fn class(package: &str) -> Option<Class> {
    CLASSIFICATION
        .iter()
        .find(|(name, _)| *name == package)
        .map(|(_, class)| *class)
}

fn in_compositor_family(path: &str) -> bool {
    path.starts_with("crates/compositor/") || path.starts_with("crates/layerize/")
}

fn allowed(list: &[(&str, &str, &str)], from: &str, to: &str) -> bool {
    list.iter().any(|(a, b, _)| *a == from && *b == to)
}

#[test]
fn every_compositor_package_is_classified() {
    let manifests = manifests();
    let mut violations = Vec::new();
    for manifest in manifests.iter().filter(|m| in_compositor_family(&m.path)) {
        match class(&manifest.package) {
            None => violations.push(format!(
                "{}: `{}` is not classified",
                manifest.path, manifest.package
            )),
            Some(Main | MainFacade) if !manifest.path.starts_with("crates/layerize/") => {
                violations.push(format!(
                    "{}: main package `{}` must live under crates/layerize/",
                    manifest.path, manifest.package
                ))
            }
            Some(_) => {}
        }
    }
    for (package, _) in CLASSIFICATION {
        let count = manifests
            .iter()
            .filter(|m| in_compositor_family(&m.path) && m.package == *package)
            .count();
        if count != 1 {
            violations.push(format!(
                "classified `{package}` has {count} manifests in the compositor family"
            ));
        }
    }
    assert!(violations.is_empty(), "S4 classification:\n{}", violations.join("\n"));
}

#[test]
fn main_side_depends_only_on_shared_roots() {
    let mut violations = Vec::new();
    for manifest in manifests() {
        if !matches!(class(&manifest.package), Some(Main | MainFacade)) {
            continue;
        }
        for dependency in &manifest.dependencies {
            if let Some(other @ (Impl | ImplFacade | MainFacade)) = class(dependency) {
                if !allowed(LAYERIZE_IMPL_DEPENDENCY_ALLOWED, &manifest.package, dependency) {
                    violations.push(format!(
                        "{}: `{}` depends on `{dependency}` ({other:?})",
                        manifest.path, manifest.package
                    ));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "layerize packages may depend only on cc shared roots and main packages:\n{}",
        violations.join("\n")
    );
}

#[test]
fn shared_roots_do_not_depend_on_impl() {
    let mut violations = Vec::new();
    for manifest in manifests() {
        if class(&manifest.package) != Some(Shared) {
            continue;
        }
        for dependency in &manifest.dependencies {
            let ok = match class(dependency) {
                Some(Shared) => true,
                Some(_) => {
                    allowed(SHARED_ROOT_IMPL_DEPENDENCY_ALLOWED, &manifest.package, dependency)
                }
                None => SHARED_ROOT_VALUE_DEPENDENCIES.contains(&dependency.as_str()),
            };
            if !ok {
                violations.push(format!(
                    "{}: `{}` depends on `{dependency}`",
                    manifest.path, manifest.package
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "cc shared roots may depend only on shared roots and value crates:\n{}",
        violations.join("\n")
    );
}

#[test]
fn impl_side_does_not_depend_on_main() {
    let mut violations = Vec::new();
    for manifest in manifests() {
        if !matches!(class(&manifest.package), Some(Impl | ImplFacade)) {
            continue;
        }
        for dependency in &manifest.dependencies {
            let main = matches!(class(dependency), Some(Main | MainFacade));
            // The commit vocabulary; sources may name only layerize::commit
            // (compositor_names_only_the_commit_vocabulary).
            let commit = manifest.package == "compositor" && dependency == "layerize";
            if main && !commit {
                violations.push(format!(
                    "{}: `{}` depends on `{dependency}`",
                    manifest.path, manifest.package
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "impl packages must not depend on main packages:\n{}",
        violations.join("\n")
    );
}

#[test]
fn outside_crates_use_only_shared_roots_or_facades() {
    let mut violations = Vec::new();
    for manifest in manifests() {
        if in_compositor_family(&manifest.path) {
            continue;
        }
        for dependency in &manifest.dependencies {
            if matches!(class(dependency), Some(Impl | Main)) {
                violations.push(format!(
                    "{}: `{}` depends on `{dependency}`",
                    manifest.path, manifest.package
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "crates outside the compositor family may use only cc shared roots or a facade:\n{}",
        violations.join("\n")
    );
}

#[test]
fn allowances_have_no_stale_entries() {
    let manifests = manifests();
    for (from, to, step) in LAYERIZE_IMPL_DEPENDENCY_ALLOWED
        .iter()
        .chain(SHARED_ROOT_IMPL_DEPENDENCY_ALLOWED)
    {
        assert!(
            manifests
                .iter()
                .any(|m| m.package == *from && m.dependencies.iter().any(|d| d == to)),
            "`{from}` no longer depends on `{to}`; remove its {step} allowance"
        );
    }
}

/// First path segments named after `prefix::` in `text` (code without `//`
/// comments), expanding one `{...}` group.
fn named_modules(text: &str, prefix: &str) -> Vec<String> {
    let code: String = text.lines().map(|line| format!("{}\n", code(line))).collect();
    let needle = format!("{prefix}::");
    let mut out = Vec::new();
    for (index, _) in code.match_indices(&needle) {
        let preceded = code[..index]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        if preceded {
            continue;
        }
        let rest = &code[index + needle.len()..];
        let Some(group) = rest.strip_prefix('{') else {
            out.push(rest.to_string());
            continue;
        };
        let mut depth = 0;
        let mut item = String::new();
        for c in group.chars() {
            match c {
                '{' => depth += 1,
                '}' if depth == 0 => break,
                '}' => depth -= 1,
                ',' if depth == 0 => {
                    out.push(std::mem::take(&mut item));
                    continue;
                }
                _ => {}
            }
            item.push(c);
        }
        out.push(item);
    }
    out.into_iter()
        .map(|item| {
            item.trim()
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>()
        })
        .filter(|item| !item.is_empty())
        .collect()
}

/// S4 transition guard: no new impl use of the mixed `compositor_input`
/// appears while its values are being cut.
#[test]
fn layerize_names_only_shared_modules() {
    let mut found = Vec::new();
    files(&root().join("crates/layerize"), &mut found);
    let mut violations = Vec::new();
    for path in found.iter().filter(|p| {
        p.extension().is_some_and(|e| e == "rs") && !relative(p).starts_with("crates/layerize/tests/")
    }) {
        let text = std::fs::read_to_string(path).unwrap();
        for module in named_modules(&text, "compositor_input") {
            if !COMPOSITOR_INPUT_VALUE_MODULES.contains(&module.as_str()) {
                violations.push(format!("{}: compositor_input::{module}", relative(path)));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "layerize may name only the shared modules of the mixed packages:\n{}",
        violations.join("\n")
    );
}

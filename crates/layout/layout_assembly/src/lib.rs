// LayoutNG and the fragment-tree types that still call LayoutObject behaviour
// are mutually recursive C++ packages. Cargo cannot express that cycle
// between two crates, so compile their translated modules once in one
// assembly crate. The include paths point to the owning crates' source files
// and do not copy or relocate implementations. Fragment data and the
// fragment tree are base modules (`layout_core/src/fragment/`); the inline
// algorithm package is the real `layout_algo_inline` crate above it.
//
// R7a turned base's link-time pairs into direct calls; the callers' former
// `unsafe { Extern(...) }` blocks now wrap safe functions.
#![allow(unused_unsafe)]
extern crate self as layout_core;
extern crate self as layout_assembly;

pub use foundation::UnsupportedLayout;

include!("../../layout_core/src/lib.rs");

/// Keep mutually recursive translated sources in their owning module paths.
/// Root re-exports preserve the existing assembly API while profiler symbols
/// identify the source package instead of flattening everything into assembly.
pub mod layout_fragment_package;
pub use layout_fragment_package::*;

// Chromium's core/paint/ decorating-box, inline-paint-context, line-relative
// geometry and text-decoration metrics are consumed by Layout's own
// fragment-tree code (FragmentItem, InlineCursor, PhysicalBoxFragment and
// ink_overflow) and name FragmentItem/UsedFont directly. Their only compile
// unit is this assembly; the sources physically live beside those
// fragment-tree users in `layout_core/src/fragment/`. Paint reaches them
// through `layout::paint::<module>` re-exports and does not compile them.
#[path = "../../layout_core/src/fragment/decorating_box.rs"]
pub mod decorating_box;
#[path = "../../layout_core/src/fragment/inline_paint_context.rs"]
pub mod inline_paint_context;
#[path = "../../layout_core/src/fragment/line_relative_rect.rs"]
pub mod line_relative_rect;
#[path = "../../layout_core/src/fragment/text_decoration_info.rs"]
pub mod text_decoration_info;

// The CSS colour parser is owned and compiled by `css_parser` (feature
// `color_parser`). It names only `paint_input_types` values, so Layout
// re-exports that compile unit instead of compiling the file a second time.
pub use css_parser::color_parser as css_color_parser;

mod block_virtual_dispatch;
pub use block_virtual_dispatch::{
    RegisterImageStyleDidChange, RegisterReplacedRecalcVisualOverflow,
    RegisterReplacedStyleDidChange, RegisterVideoStyleDidChange,
};
mod untranslated_symbols;
mod out_of_flow_element_data;

#[cfg(test)]
mod package_contract_tests {
    use std::path::{Path, PathBuf};

    const LIFECYCLE_OWNERS: [&str; 3] = ["layout_invalidation", "layout_reuse", "layout_commit"];

    fn manifest_section_value<'a>(contents: &'a str, section: &str, key: &str) -> Option<&'a str> {
        let mut current = "";
        for raw_line in contents.lines() {
            let line = raw_line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                current = &line[1..line.len() - 1];
                continue;
            }
            if current != section {
                continue;
            }
            let Some((candidate, value)) = line.split_once('=') else {
                continue;
            };
            if candidate.trim() == key {
                return value.trim().strip_prefix('"')?.strip_suffix('"');
            }
        }
        None
    }

    fn is_performance_owner(name: &str) -> bool {
        LIFECYCLE_OWNERS.contains(&name)
            || (name.starts_with("layout_algo_") && !name.ends_with("_assembly"))
    }

    fn rust_sources(directory: &Path, output: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).expect("read Layout package directory") {
            let entry = entry.expect("read Layout package entry");
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|name| name == "target") {
                    continue;
                }
                rust_sources(&path, output);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                output.push(path);
            }
        }
    }

    #[test]
    fn lifecycle_trace_names_are_declared_only_by_their_owner_package() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layout package root");
        let mut sources = Vec::new();
        rust_sources(root, &mut sources);
        // (trace-name prefix, the only source path allowed to declare it).
        let owners = [
            ("layout_invalidation", Path::new("layout_invalidation")),
            ("layout_reuse", Path::new("layout_reuse")),
            // `layout_fragment.` is the historical category of fragment
            // publication and Paint-sidecar work. Fragments are base now and
            // that package is gone, but recorded traces, DevTools owner
            // attribution and the diagnostics phase map keep the name, so its
            // phases are declared only by `layout_commit`'s fragment_trace.rs.
            (
                "layout_fragment",
                Path::new("layout_commit/src/fragment_trace.rs"),
            ),
            ("layout_commit", Path::new("layout_commit")),
        ];
        let mut violations = Vec::new();
        for source in sources {
            let relative = source.strip_prefix(root).expect("relative Layout source");
            let contents = std::fs::read_to_string(&source).expect("read Layout source");
            for (owner, allowed) in owners {
                if contents.contains(&format!("\"{owner}."))
                    && !relative.starts_with(allowed)
                {
                    violations.push(format!(
                        "{} declares {owner} trace names",
                        relative.display()
                    ));
                }
            }
        }
        assert!(
            violations.is_empty(),
            "lifecycle trace ownership violations:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn algorithm_trace_names_are_declared_only_by_their_owner_package() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layout package root");
        let mut sources = Vec::new();
        rust_sources(root, &mut sources);
        let mut violations = Vec::new();
        for source in sources {
            let relative = source.strip_prefix(root).expect("relative Layout source");
            let contents = std::fs::read_to_string(&source).expect("read Layout source");
            for line in contents.lines() {
                let mut remainder = line;
                while let Some(start) = remainder.find("\"layout_algo_") {
                    let literal_start = &remainder[start + 1..];
                    let Some(end) = literal_start.find('"') else {
                        break;
                    };
                    let literal = &literal_start[..end];
                    remainder = &literal_start[end + 1..];
                    let Some((owner, _)) = literal.split_once('.') else {
                        continue;
                    };
                    // Source paths in ownership-contract fixtures can start with an
                    // algorithm package name as well. They are not trace literals.
                    if owner.contains('/') {
                        continue;
                    }
                    if owner.ends_with("_test") {
                        continue;
                    }
                    if !root.join(owner).is_dir() || !relative.starts_with(Path::new(owner)) {
                        violations.push(format!(
                            "{} declares {owner} trace names",
                            relative.display()
                        ));
                    }
                }
            }
        }
        assert!(
            violations.is_empty(),
            "algorithm trace ownership violations:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn performance_owner_identity_matches_directory_and_dependency_key() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layout package root");
        let mut owners = Vec::new();
        let mut manifests = Vec::new();
        for entry in std::fs::read_dir(root).expect("read Layout package directory") {
            let entry = entry.expect("read Layout package entry");
            let path = entry.path();
            if !path.is_dir() || !path.join("Cargo.toml").is_file() {
                continue;
            }
            let name = path
                .file_name()
                .expect("package directory name")
                .to_string_lossy()
                .into_owned();
            if is_performance_owner(&name) {
                owners.push(name);
            }
            manifests.push(path.join("Cargo.toml"));
        }
        owners.sort();

        let mut violations = Vec::new();
        for owner in &owners {
            let manifest_path = root.join(owner).join("Cargo.toml");
            let contents =
                std::fs::read_to_string(&manifest_path).expect("read performance owner manifest");
            let package_name = manifest_section_value(&contents, "package", "name");
            if package_name != Some(owner.as_str()) {
                violations.push(format!(
                    "{owner}/Cargo.toml declares package {package_name:?}"
                ));
            }
            if let Some(library_name) = manifest_section_value(&contents, "lib", "name") {
                if library_name != owner {
                    violations.push(format!(
                        "{owner}/Cargo.toml overrides its library name as {library_name}"
                    ));
                }
            }
        }

        for manifest_path in manifests {
            let contents = std::fs::read_to_string(&manifest_path).expect("read Layout manifest");
            for raw_line in contents.lines() {
                let line = raw_line.trim();
                let Some((dependency_key, declaration)) = line.split_once('=') else {
                    continue;
                };
                let Some(path_start) = declaration.find("path = \"") else {
                    continue;
                };
                let path_value = &declaration[path_start + "path = \"".len()..];
                let Some(path_end) = path_value.find('"') else {
                    continue;
                };
                let Some(target) = Path::new(&path_value[..path_end])
                    .file_name()
                    .map(|value| value.to_string_lossy())
                else {
                    continue;
                };
                if !is_performance_owner(&target) {
                    continue;
                }
                let dependency_key = dependency_key.trim();
                if dependency_key != target {
                    violations.push(format!(
                        "{} aliases performance owner {target} as {dependency_key}",
                        manifest_path
                            .strip_prefix(root)
                            .expect("relative manifest path")
                            .display()
                    ));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "performance owner identity violations:\n{}",
            violations.join("\n")
        );
    }
}

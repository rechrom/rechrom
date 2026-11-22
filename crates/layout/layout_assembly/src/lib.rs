// LayoutNG and its fragment tree are mutually recursive C++ packages. Cargo
// cannot express that cycle between two crates, so compile their translated
// modules once in one assembly crate. The include paths point to the owning
// crates' source files and do not copy or relocate implementations.
extern crate self as layout_algo_inline;
extern crate self as layout_fragment;
extern crate self as layout_core;
extern crate self as layout_assembly;

pub use foundation::UnsupportedLayout;

include!("../../layout_core/src/lib.rs");

/// Keep mutually recursive translated sources in their owning module paths.
/// Root re-exports preserve the existing assembly API while profiler symbols
/// identify the source package instead of flattening everything into assembly.
pub mod layout_fragment_package {
    include!("../../layout_fragment/src/lib.rs");
}
pub use layout_fragment_package::*;

pub mod layout_algo_inline_package {
    include!("../../layout_algo_inline/src/lib.rs");
}
pub use layout_algo_inline_package::*;

// Chromium's core/paint/ decorating-box, inline-paint-context, line-relative
// geometry and text-decoration metrics are consumed by Layout's own
// fragment-tree code (FragmentItem, InlineCursor, PhysicalBoxFragment and
// ink_overflow) and name FragmentItem/UsedFont directly. Their only compile
// unit is this assembly; the sources physically live beside those
// fragment-tree users in `layout_fragment/src/` so samples attribute to the
// fragment package. Paint reaches them through `layout::paint::<module>`
// re-exports and does not compile them.
#[path = "../../layout_fragment/src/decorating_box.rs"]
pub mod decorating_box;
#[path = "../../layout_fragment/src/inline_paint_context.rs"]
pub mod inline_paint_context;
#[path = "../../layout_fragment/src/line_relative_rect.rs"]
pub mod line_relative_rect;
#[path = "../../layout_fragment/src/text_decoration_info.rs"]
pub mod text_decoration_info;

// The CSS colour parser is owned and compiled by `css_parser` (feature
// `color_parser`). It names only `paint_input_types` values, so Layout
// re-exports that compile unit instead of compiling the file a second time.
pub use css_parser::color_parser as css_color_parser;

mod block_virtual_dispatch;
pub use block_virtual_dispatch::{
    GridObjectVirtuals, RegisterGridObjectVirtuals, RegisterGridStyleDidChange,
    RegisterImageStyleDidChange, RegisterReplacedRecalcVisualOverflow,
    RegisterReplacedStyleDidChange, RegisterSvgRootChildren, RegisterVideoStyleDidChange,
};
pub use block_virtual_dispatch::{
    RegisterFlexStitchedRowGapIndex, RegisterFlexTotalRowGapCount, RegisterGridTotalRowGapCount,
};
mod layout_invalidation_reason_symbols;
mod untranslated_symbols;
mod out_of_flow_element_data;

// Grid-owned sources whose only compile unit is this assembly. They stay in
// `layout_algo_grid/src/` for profiler attribution; `layout_algo_grid/src/lib.rs`
// re-exports these modules and does not compile the files. Core needs them:
// gap_segment_state_aggregator.rs holds trait and inherent impls on the
// core-declared `GapSegmentStateAggregator` (coherence requires the declaring
// crate), and grid_track_collection.rs defines the `#[no_mangle]` bodies that
// core's `internal::grid_track_collection` declares as `extern "Rust"`, so
// they must link with every layout_assembly user, including its own tests.
#[path = "../../layout_algo_grid/src/gap_segment_state_aggregator.rs"]
pub mod grid_gap_segment_state_aggregator;

#[path = "../../layout_algo_grid/src/grid_track_collection.rs"]
pub mod grid_track_collection_implementation;

#[cfg(test)]
mod package_contract_tests {
    use std::path::{Path, PathBuf};

    const LIFECYCLE_OWNERS: [&str; 4] = [
        "layout_invalidation",
        "layout_reuse",
        "layout_fragment",
        "layout_commit",
    ];

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
        let owners = [
            "layout_invalidation",
            "layout_reuse",
            "layout_fragment",
            "layout_commit",
        ];
        let mut violations = Vec::new();
        for source in sources {
            let relative = source.strip_prefix(root).expect("relative Layout source");
            let contents = std::fs::read_to_string(&source).expect("read Layout source");
            for owner in owners {
                if contents.contains(&format!("\"{owner}."))
                    && !relative.starts_with(Path::new(owner))
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
                let assembly_fragment_boundary = manifest_path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|name| name.to_string_lossy().ends_with("_assembly"))
                    && dependency_key == "layout_fragment_boundary"
                    && target == "layout_fragment";
                if dependency_key != target && !assembly_fragment_boundary {
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

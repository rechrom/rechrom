// LayoutNG and its fragment tree are mutually recursive C++ packages. Cargo
// cannot express that cycle between two crates, so compile their translated
// modules once in one assembly crate. The include paths point to the owning
// crates' source files and do not copy or relocate implementations.
extern crate self as layout_algo_inline;
extern crate self as layout_fragment;
extern crate self as layoutng;
extern crate self as layoutng_assembly;

pub use foundation::UnsupportedLayout;

include!("../../layoutng/src/lib.rs");

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

// Inline ink-overflow traversal is mutually recursive with Paint's decorating
// box context in Chromium. Keep the Paint-owned sources at their physical path
// and compile them once at this existing reciprocal assembly boundary.
#[path = "../../../paint/paint_record/src/decorating_box.rs"]
pub mod decorating_box;
#[path = "../../../paint/paint_record/src/inline_paint_context.rs"]
pub mod inline_paint_context;
#[path = "../../../paint/paint_record/src/line_relative_rect.rs"]
pub mod line_relative_rect;
#[path = "../../../paint/paint_record/src/text_decoration_info.rs"]
pub mod text_decoration_info;

// CSS value parsers refer to LayoutNG-owned value types. Compile the owning
// css_parser source once at this reciprocal assembly boundary.
#[path = "../../../css_parser/src/color_parser.rs"]
pub mod css_color_parser;

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
mod out_of_flow_element_data;

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
            .expect("layoutng package root");
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
            .expect("layoutng package root");
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
    fn core_sampling_frames_are_explicit_shared_boundaries_or_algorithm_trampolines() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let core = root.join("layoutng/src/internal");
        let mut sources = Vec::new();
        rust_sources(&core, &mut sources);
        let mut violations = Vec::new();
        for source in sources {
            let contents = std::fs::read_to_string(&source).expect("read Layout core source");
            if !contents.contains("#[inline(never)]") {
                continue;
            }
            let relative = source.strip_prefix(&core).expect("relative core source");
            if relative == Path::new("algorithm_entry.rs")
                && contents.contains("pub fn RunAlgorithm<A: LayoutAlgorithmImplementation>")
                && contents.contains("pub fn MeasureAlgorithm<A: LayoutAlgorithmImplementation>")
                && contents.contains("pub fn TraceAlgorithmRoot<A: LayoutAlgorithmImplementation>")
            {
                continue;
            }
            if matches!(
                relative,
                path if path == Path::new("layout_font_resolver.rs")
                    || path == Path::new("boundary/font_from_selector.rs")
                    || path == Path::new("boundary/layout_environment.rs")
            ) {
                continue;
            }
            violations.push(relative.display().to_string());
        }
        assert!(
            violations.is_empty(),
            "core introduced an unreviewed sampling boundary; either assign functional ownership or document it as shared orchestration:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn selected_shared_fixed_cost_core_helpers_are_forced_inline() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        for (relative, helpers) in [
            (
                "relative_utils.rs",
                &[
                    "fn ResolveInset",
                    "pub fn ComputeRelativeOffset",
                    "pub fn ComputeRelativeOffsetForBoxFragment",
                ][..],
            ),
            (
                "layout_alignment_utils.rs",
                &["pub fn InlineStaticPositionEdgeDefault"][..],
            ),
        ] {
            let source = std::fs::read_to_string(root.join("layoutng/src/internal").join(relative))
                .unwrap_or_else(|_| panic!("read fixed-cost core helper {relative}"));
            for helper in helpers {
                assert!(
                    source.contains(&format!("#[inline(always)]\n{helper}")),
                    "fixed-cost {helper} in {relative} must fold into its package-owned caller"
                );
            }
        }
    }

    #[test]
    fn performance_owner_identity_matches_directory_and_dependency_key() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
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

    #[test]
    fn shared_fragmentation_policy_resolves_to_its_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "fragmentation/src/fragmentation_utils.rs"
        )))
        .expect("read fragmentation algorithm owner");
        for sampled_entry in [
            "SetupFragmentBuilderForFragmentation",
            "FinishFragmentation",
            "FinishFragmentationForFragmentainer",
            "BreakBeforeChildIfNeededFull",
            "MovePastBreakpointFull",
            "MovePastBreakpointWithoutChildFull",
            "AttemptSoftBreak",
            "CreateConstraintSpaceForFragmentainer",
            "OffsetInStitchedFragments",
        ] {
            assert!(
                owner.contains(&format!("#[inline(never)]\npub fn {sampled_entry}")),
                "fragmentation work lost its {sampled_entry} sampling frame"
            );
        }

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/fragmentation_utils.rs"))
                .expect("read fragmentation compatibility facade");
        assert!(
            facade.contains("fragmentation/src/fragmentation_utils.rs")
                && facade.contains("pub use layout_algo_fragmentation_fragmentation_utils::*;")
                && !facade.contains("pub fn FinishFragmentation"),
            "core fragmentation_utils must remain a re-export-only facade"
        );
        for (facade_file, owner_file, module_name, owned_type) in [
            (
                "early_break.rs",
                "early_break.rs",
                "layout_algo_fragmentation_early_break",
                "pub struct EarlyBreak",
            ),
            (
                "break_appeal.rs",
                "break_appeal.rs",
                "layout_algo_fragmentation_break_appeal",
                "pub enum BreakAppeal",
            ),
        ] {
            let owner = std::fs::read_to_string(
                root.join(concat!("layout_algo_", "fragmentation/src/"))
                    .join(owner_file),
            )
            .unwrap_or_else(|_| panic!("read fragmentation state owner {owner_file}"));
            assert!(
                owner.contains(owned_type),
                "fragmentation owner lost {owned_type}"
            );
            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read fragmentation facade {facade_file}"));
            assert!(
                facade.contains(&format!("fragmentation/src/{owner_file}"))
                    && facade.contains(&format!("pub use {module_name}::*;"))
                    && !facade.contains(owned_type),
                "{facade_file} must remain a re-export-only layout_algo_fragmentation facade"
            );
        }
    }

    #[test]
    fn specialized_constraint_space_clones_resolve_to_their_owners() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let core = std::fs::read_to_string(root.join("layoutng/src/internal/constraint_space.rs"))
            .expect("read shared constraint-space schema");
        assert!(
            core.contains("#[inline(always)]\n    fn clone(&self) -> Self")
                && core.contains("fragmentation/src/constraint_space_fragmentation.rs")
                && core.contains("inline/src/constraint_space_block_in_inline.rs")
                && !core.contains("pub fn CloneWithoutFragmentation")
                && !core.contains("pub fn CloneForBlockInInlineIfNeeded"),
            "ConstraintSpace must keep only its inline shared clone and owner-module loaders"
        );

        let fragmentation = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "fragmentation/src/constraint_space_fragmentation.rs"
        )))
        .expect("read fragmentation constraint-space projection");
        assert!(
            fragmentation.contains("#[inline(always)]\n    pub fn CloneWithoutFragmentation")
                && fragmentation.contains("#[inline(never)]\nfn clone_without_fragmentation"),
            "fragmentation projection must forward to a package-qualified sampling frame"
        );

        let inline = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "inline/src/constraint_space_block_in_inline.rs"
        )))
        .expect("read block-in-inline constraint-space projection");
        assert!(
            inline.contains("#[inline(always)]\n    pub fn CloneForBlockInInlineIfNeeded")
                && inline.contains("#[inline(never)]\nfn clone_for_block_in_inline"),
            "block-in-inline projection must keep an inline fast path and package-qualified sampled slow path"
        );

        let float = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/exclusion_space.rs")),
        )
        .expect("read float exclusion-space owner");
        assert!(
            float.contains("impl Clone for ExclusionSpace {")
                && float.contains("#[inline(always)]\n    fn clone(&self) -> Self")
                && float.contains("#[inline(never)]\nfn clone_exclusion_space"),
            "allocated exclusion-state cloning must retain a package-qualified float sampling frame"
        );
    }

    #[test]
    fn shared_sizing_policy_resolves_to_its_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "sizing/src/length_utils.rs")),
        )
        .expect("read sizing algorithm owner");
        for sampled_entry in [
            "#[inline(never)]\npub fn ComputeMinMaxBlockSizes",
            "#[inline(never)]\npub fn ComputeMinMaxInlineSizes",
            "#[inline(never)]\npub fn ComputeInlineSizeForFragmentInternal",
            "#[inline(never)]\nfn ComputeBlockSizeForFragmentInternal",
            "#[inline(never)]\nfn ComputeReplacedSizeInternal",
            "#[inline(never)]\npub fn CalculateInitialFragmentGeometryWithMinMaxSizes",
            "#[inline(never)]\npub fn ComputeMinAndMaxContentContributionInternal",
        ] {
            assert!(
                owner.contains(sampled_entry),
                "shared sizing work lost its {sampled_entry} sampling frame"
            );
        }

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/length_utils.rs"))
            .expect("read sizing compatibility facade");
        assert!(
            facade.contains("sizing/src/length_utils.rs")
                && facade.contains("pub use layout_algo_sizing_length_utils::*;")
                && !facade.contains("pub fn ComputeMinMaxBlockSizes"),
            "core length_utils must remain a re-export-only layout_algo_sizing facade"
        );

        let space_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "sizing/src/space_utils.rs")),
        )
        .expect("read constraint-space sizing owner");
        assert!(
            space_owner.contains("#[inline(never)]\npub fn SetOrthogonalFallbackInlineSize"),
            "orthogonal fallback sizing must retain a layout_algo_sizing sampling frame"
        );
        let baseline_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "sizing/src/baseline_utils.rs")),
        )
        .expect("read baseline sizing owner");
        for inline_helper in [
            "DetermineBaselineWritingMode",
            "DetermineBaselineGroup",
            "DetermineBaselineGroupDefault",
        ] {
            assert!(
                baseline_owner.contains(&format!("#[inline(always)]\npub fn {inline_helper}")),
                "fixed-cost {inline_helper} must fold into its flex/grid caller"
            );
        }
        for (facade_file, owner_file, module_name) in [
            (
                "space_utils.rs",
                "space_utils.rs",
                "layout_algo_sizing_space_utils",
            ),
            (
                "baseline_utils.rs",
                "baseline_utils.rs",
                "layout_algo_sizing_baseline_utils",
            ),
        ] {
            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read sizing facade {facade_file}"));
            assert!(
                facade.contains(&format!("sizing/src/{owner_file}"))
                    && facade.contains(&format!("pub use {module_name}::*;")),
                "{facade_file} must remain a re-export-only layout_algo_sizing facade"
            );
        }

        let builder_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "sizing/src/min_max_constraint_space_builder.rs"
        )))
        .expect("read min/max constraint-space builder owner");
        assert!(
            builder_owner.contains("pub struct MinMaxConstraintSpaceBuilder")
                && builder_owner.contains("SetOrthogonalFallbackInlineSizeIfNeeded"),
            "min/max constraint-space sizing must remain in layout_algo_sizing"
        );
        let builder_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/constraint_space_builder_style.rs"),
        )
        .expect("read min/max constraint-space builder facade");
        assert!(
            builder_facade.contains("sizing/src/min_max_constraint_space_builder.rs")
                && builder_facade
                    .contains("pub use layout_algo_sizing_min_max_constraint_space_builder::*;")
                && !builder_facade.contains("pub struct MinMaxConstraintSpaceBuilder"),
            "core min/max constraint-space builder must remain a re-export-only facade"
        );
    }

    #[test]
    fn fragment_boundary_stays_independent_of_concrete_layoutng() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let package = root.join("layout_fragment");
        let manifest =
            std::fs::read_to_string(package.join("Cargo.toml")).expect("read fragment manifest");
        assert!(
            !manifest
                .lines()
                .map(str::trim)
                .any(|line| line.starts_with("layoutng =")),
            "layout_fragment must not depend on the concrete layoutng crate"
        );

        for relative in [
            "src/package.rs",
            "src/boundary_types.rs",
            "src/revision.rs",
            "src/trace.rs",
        ] {
            let contents = std::fs::read_to_string(package.join(relative))
                .unwrap_or_else(|_| panic!("read fragment boundary source {relative}"));
            assert!(
                !contents.contains("layoutng::") && !contents.contains("use layoutng"),
                "{relative} imports the concrete layoutng implementation"
            );
        }
    }

    #[test]
    fn support_callback_groups_publish_their_performance_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let groups = [
            ("layout_algo_inline", "inline_support"),
            ("layout_algo_table", "table_support"),
            ("layout_algo_out_of_flow", "out_of_flow_support"),
            ("layout_algo_float", "float_support"),
            ("layout_algo_replaced", "replaced_sizing"),
            ("layout_algo_list", "list_support"),
            ("layout_algo_forms", "forms_support"),
            ("layout_algo_svg", "svg_support"),
        ];
        for (owner, group) in groups {
            let source = root.join(owner).join("src/assembly.rs");
            let contents = std::fs::read_to_string(&source)
                .unwrap_or_else(|_| panic!("read {} support installer", source.display()));
            let compact = contents
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>();
            let assignment = format!("{group}.performance_owner=Some(\"{owner}\")");
            let struct_field = format!("performance_owner:Some(\"{owner}\")");
            assert!(
                compact.contains(&assignment) || compact.contains(&struct_field),
                "{owner} does not assign {group}'s performance owner"
            );
        }
    }

    #[test]
    fn custom_layout_state_resolves_to_the_custom_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let payload = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "custom/src/custom_layout_payload.rs"
        )))
        .expect("read custom layout payload owner");
        assert!(
            payload.contains("pub struct CustomLayoutPayload")
                && payload.contains("#[inline(never)]\n    pub fn Copy")
                && payload.contains("bytes.to_vec().into_boxed_slice()")
                && payload.contains("pub type SerializedScriptValue"),
            "input-sized custom payload copying must retain a layout_algo_custom sampling frame"
        );

        let custom_data = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "custom/src/constraint_space_custom_data.rs"
        )))
        .expect("read custom constraint-space state owner");
        for owned_policy in [
            "pub struct CustomData",
            "pub fn MaySkipLayout",
            "pub fn IsInitialForMaySkipLayout",
            "pub fn SetCustomLayoutData",
            "impl ConstraintSpaceBuilder",
        ] {
            assert!(
                custom_data.contains(owned_policy),
                "layout_algo_custom constraint-space state lost {owned_policy}"
            );
        }
        assert!(
            !custom_data.contains("#[inline(never)]"),
            "fixed-cost custom constraint state must remain inlineable"
        );

        for (facade_file, owner_file, leaked_policy) in [
            (
                "custom_layout_payload.rs",
                "custom_layout_payload.rs",
                "pub struct CustomLayoutPayload",
            ),
            (
                "constraint_space_custom_data.rs",
                "constraint_space_custom_data.rs",
                "pub struct CustomData",
            ),
        ] {
            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read custom compatibility facade {facade_file}"));
            assert!(
                facade.contains(&format!("custom/src/{owner_file}"))
                    && facade.contains("pub use layout_algo_custom_")
                    && !facade.contains(leaked_policy),
                "{facade_file} must remain a re-export-only layout_algo_custom facade"
            );
        }
    }

    #[test]
    fn frame_set_layout_data_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/frame_set_layout_data.rs"))
                .expect("read source frame-set layout-data owner");
        for owned_policy in [
            "pub struct FrameSetLayoutData",
            "col_sizes: self.col_sizes.clone()",
            "row_sizes: self.row_sizes.clone()",
            "col_allow_border: self.col_allow_border.clone()",
            "row_allow_border: self.row_allow_border.clone()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment frame-set data lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]\n    fn clone"),
            "input-sized frame-set vector copying must retain a fragment sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/frame_set_layout_data.rs"))
                .expect("read frame-set layout-data facade");
        assert!(
            facade.contains("layout_fragment/src/frame_set_layout_data.rs")
                && facade.contains("pub use layout_fragment_frame_set_layout_data::*")
                && !facade.contains("pub struct FrameSetLayoutData"),
            "frame-set layout data must be re-exported from layout_fragment"
        );
    }

    #[test]
    fn replaced_sizing_callback_keeps_runtime_and_sampling_ownership() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let source =
            std::fs::read_to_string(root.join("layout_algo_replaced").join("src/assembly.rs"))
                .expect("read replaced algorithm installer");
        assert!(
            source.contains("#[inline(never)]\nfn NaturalSizingInfo")
                && source.contains(concat!("layout_algo_", "replaced.NaturalSizingInfo"))
                && source.contains("replaced_sizing.natural_sizing_info = Some(NaturalSizingInfo)")
                && source.contains("replaced_respects_css_overflow")
                && source.contains("LayoutReplacedRespectsCSSOverflow")
                && source.contains("replaced_image_state_updater")
                && source.contains("UpdateImageState")
                && source.contains("replaced_video_state_updater")
                && source.contains("UpdateVideoState"),
            "replaced intrinsic sizing must retain a package span and out-of-line sampling frame"
        );

        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "replaced/src/layout_replaced_interface.rs"
        )))
        .expect("read source LayoutReplaced owner");
        for owned_policy in [
            "pub struct LayoutReplaced",
            "pub struct LayoutInputReplaced",
            "fn LayoutReplacedComputeNaturalSizingInfo",
            "fn ComputeReplacedContentRect",
            "fn ComputeObjectFitAndPositionRect",
            "cached_resource_priority_",
            "fn CachedSpeculativeDecodeSize",
            "fn ComputeSpeculativeDecodeQuality",
            "fn IsUnsizedImage",
            "\"LayoutImage\"",
            "fn DrawsBackgroundOntoContentLayer",
            "fn IsInSelfHitTestingPhase",
            "fn PositionForPoint",
            "image_device_pixel_ratio_",
            "natural_dimensions_",
            "is_generated_content_",
            "fn ImageDevicePixelRatio",
            "fn NaturalDimensions",
            "fn IsGeneratedContent",
            "DowncastFrom<LayoutObject> for LayoutInputReplaced",
        ] {
            assert!(
                owner.contains(owned_policy),
                "replaced algorithm interface lost {owned_policy}"
            );
        }
        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_replaced.rs"))
            .expect("read LayoutReplaced compatibility facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "replaced/src/layout_replaced_interface.rs"
            )) && facade.contains("pub use layout_algo_replaced_layout_replaced_interface::*;")
                && !facade.contains("pub struct LayoutReplaced"),
            "LayoutReplaced must be re-exported from the replaced algorithm owner"
        );

        let lifecycle = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "replaced/src/layout_replaced.rs")),
        )
        .expect("read LayoutReplaced lifecycle owner");
        for owned_policy in [
            "fn LayoutReplacedStyleDidChange",
            "fn LayoutReplacedNaturalSizeChanged",
            "fn LayoutReplacedComputeVisualEffectOverflowOutsets",
            "fn LayoutReplacedAddVisualEffectOverflow",
            "fn LayoutReplacedRecalcVisualOverflow",
            "fn LayoutReplacedComputeSpeculativeDecodeSize",
            "fn LayoutReplacedLocalSelectionVisualRect",
            "fn LayoutReplacedPositionForPoint",
            "fn LayoutReplacedHitTestChildren",
            "fn SelectionTopAndBottom",
            ".SlowLastChild()",
            ".HasSelfPaintingLayer()",
            ".NodeAtPoint(",
            "fn LayoutInputReplacedStyleDidChange",
            "diff.border_radius_changed()",
            "new_style.EffectiveZoom() != old_zoom",
            "kSizeChanged",
        ] {
            assert!(
                lifecycle.contains(owned_policy),
                "replaced algorithm lifecycle lost {owned_policy}"
            );
        }
        let cursor = std::fs::read_to_string(root.join("layout_fragment/src/inline_cursor.rs"))
            .expect("read replaced selection fragment owner");
        assert!(
            cursor.contains("fn CurrentLocalSelectionRectForReplaced")
                && cursor.contains("ExpandSelectionRectToLineHeight(logical_rect"),
            "replaced inline selection geometry must remain owned by layout_fragment"
        );
        let priority = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_box_resource_priority.rs"),
        )
        .expect("read LayoutBox resource-priority owner");
        for owned_policy in [
            "fn LayoutBoxComputeResourcePriorityProvider",
            "LocalToAbsolutePointPhysical",
            "VisibilityStatus::kVisible",
            "screen_rect.Width() * screen_rect.Height()",
        ] {
            assert!(
                priority.contains(owned_policy),
                "layout_fragment resource priority lost {owned_policy}"
            );
        }
        let dispatch =
            std::fs::read_to_string(root.join("layoutng_assembly/src/block_virtual_dispatch.rs"))
                .expect("read LayoutObject virtual dispatch");
        assert!(
            dispatch.contains("RegisterReplacedStyleDidChange")
                && dispatch.contains("REPLACED_STYLE_DID_CHANGE")
                && dispatch.contains("LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot"),
            "assembly must remain a lifecycle trampoline to layout_algo_replaced"
        );
        assert!(
            dispatch.contains("RegisterImageStyleDidChange")
                && dispatch.contains("IMAGE_STYLE_DID_CHANGE")
                && dispatch.contains("LayoutObjectClass::Replaced if IsLayoutImage(object)")
                && dispatch.contains(
                    "LayoutObjectClass::Replaced if IsLayoutImage(object) => \"LayoutImage\""
                ),
            "assembly must preserve LayoutImage's derived style lifecycle before the replaced base fallback"
        );
        assert!(
            dispatch.contains("RegisterReplacedRecalcVisualOverflow")
                && dispatch.contains("REPLACED_RECALC_VISUAL_OVERFLOW")
                && dispatch.contains("runtime_class == LayoutObjectClass::Replaced")
                && !dispatch.contains(
                    "runtime_class == LayoutObjectClass::Replaced\n            || runtime_class == LayoutObjectClass::SvgRoot"
                ),
            "assembly must dispatch replaced visual overflow to its package without swallowing the SVG root override"
        );

        let paint = std::fs::read_to_string(
            root.parent()
                .expect("crates root")
                .join("paint/paint_record/src/box_fragment_painter.rs"),
        )
        .expect("read replaced-image paint consumer");
        let paint_replaced = paint
            .split_once("fn PaintReplaced")
            .and_then(|(_, tail)| tail.split_once("fn PaintOutline"))
            .map(|(body, _)| body)
            .expect("isolate replaced-image paint body");
        assert!(
            paint_replaced.contains("source replaced-content geometry"),
            "replaced-image paint must require LayoutReplaced geometry"
        );
        for leaked_layout_policy in ["ObjectFit::", "object_position", "resolution_scale"] {
            assert!(
                !paint_replaced.contains(leaked_layout_policy),
                "paint_record must not recompute LayoutReplaced policy {leaked_layout_policy}"
            );
        }
    }

    #[test]
    fn multicol_break_token_data_resolves_to_the_multicol_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "multicol/src/multicol_break_token_data.rs"
        )))
        .expect("read multicol break-token data owner");
        assert!(
            owner.contains("pub struct MulticolBreakTokenData")
                && owner.contains("BreakTokenAlgorithmDataType::kMulticolData")
                && owner.contains("impl BreakTokenAlgorithmDataVirtual"),
            "multicol break-token state must remain physically owned by the multicol algorithm"
        );
        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/multicol_break_token_data.rs"),
        )
        .expect("read multicol break-token compatibility facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "multicol/src/multicol_break_token_data.rs"
            )) && facade.contains("pub use layout_algo_multicol_multicol_break_token_data::*;")
                && !facade.contains("pub struct MulticolBreakTokenData"),
            "multicol break-token data must be re-exported from its algorithm owner"
        );
    }

    #[test]
    fn form_state_resolves_to_the_forms_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        for (owner_file, facade_file, owned_policy, reexport) in [
            (
                "form_control_types.rs",
                "form_control_types.rs",
                "pub enum AutofillState",
                concat!("pub use layout_algo_", "forms_form_control_types::*;"),
            ),
            (
                "form_control_sizing_service.rs",
                "form_control_sizing_service.rs",
                "impl ComputedStyleControlSizingExt for ComputedStyle",
                concat!(
                    "pub use layout_algo_",
                    "forms_form_control_sizing_service::*;"
                ),
            ),
            (
                "form_node_metadata_interface.rs",
                "form_node_metadata.rs",
                "pub struct HTMLFormControlElement",
                concat!(
                    "pub use layout_algo_",
                    "forms_form_node_metadata_interface::*;"
                ),
            ),
        ] {
            let owner = std::fs::read_to_string(
                root.join(concat!("layout_algo_", "forms/src"))
                    .join(owner_file),
            )
            .unwrap_or_else(|_| panic!("read forms owner {owner_file}"));
            assert!(
                owner.contains(owned_policy),
                "forms algorithm owner {owner_file} lost {owned_policy}"
            );

            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read forms facade {facade_file}"));
            assert!(
                facade.contains(&format!("forms/src/{owner_file}"))
                    && facade.contains(reexport)
                    && !facade.contains(owned_policy),
                "forms policy {owned_policy} must be re-exported from {owner_file}"
            );
        }

        let metadata = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "forms/src/form_node_metadata_interface.rs"
        )))
        .expect("read forms metadata owner");
        assert!(
            metadata.contains("pub trait FormNodeMetadataExternal")
                && metadata.contains("fn CreateFormElementMetadata"),
            "forms metadata provider seam must remain forms-owned"
        );
        let assembly =
            std::fs::read_to_string(root.join(concat!("layout_algo_", "forms/src/assembly.rs")))
                .expect("read forms algorithm assembly");
        assert!(
            assembly.contains("fn CreateFormsMetadata")
                && assembly.contains(concat!("layout_algo_", "forms.CreateMetadata"))
                && assembly.contains("assembly.objects.forms_metadata = Some(CreateFormsMetadata)"),
            "form metadata construction must retain its forms trace and callback owner"
        );
    }

    #[test]
    fn algorithm_sampling_frames_retain_the_concrete_package_type() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let source = std::fs::read_to_string(root.join("layoutng/src/internal/algorithm_entry.rs"))
            .expect("read source algorithm entry");
        assert!(
            source.contains(
                "#[inline(never)]\npub fn RunAlgorithm<A: LayoutAlgorithmImplementation>"
            ) && source.contains(
                "#[inline(never)]\npub fn MeasureAlgorithm<A: LayoutAlgorithmImplementation>",
            ) && source.contains("layout: Some(RunAlgorithm::<A>)")
                && source.contains("measure: Some(MeasureAlgorithm::<A>)"),
            "source algorithm entries must remain out-of-line monomorphizations of the package type"
        );
    }

    #[test]
    fn algorithm_input_environments_keep_concrete_owners() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let inline = std::fs::read_to_string(
            root.join("layout_algo_inline")
                .join("src/text_input_environment.rs"),
        )
        .expect("read inline text-input environment owner");
        for owned_policy in [
            "fn ProviderLastHyphenLocation",
            "pub(super) struct InputHyphenationResolver",
            "fn TransformWithProvider",
            "pub(super) struct InputTextTransformResolver",
            "fn NextPhraseBreak",
            "pub(super) struct InputPhraseBreakResolver",
        ] {
            assert!(
                inline.contains(owned_policy),
                "layout_algo_inline text environment lost {owned_policy}"
            );
        }
        assert!(
            inline.matches("#[inline(never)]").count() >= 3,
            "hyphenation, transform and phrase-break provider calls must retain sampling frames"
        );

        let float = std::fs::read_to_string(
            root.join("layout_algo_float")
                .join("src/shape_image_environment.rs"),
        )
        .expect("read float shape-image environment owner");
        for owned_policy in [
            "pub(super) struct InputShapeImageResolver",
            "impl ShapeImageResolver",
            "BitmapPixels",
            "shape image requires dimensions, scale and RGBA pixels",
        ] {
            assert!(
                float.contains(owned_policy),
                "layout_algo_float shape-image environment lost {owned_policy}"
            );
        }

        let float_services = std::fs::read_to_string(
            root.join("layout_algo_float")
                .join("src/shape_image_services_interface.rs"),
        )
        .expect("read float shape-image services owner");
        for owned_policy in [
            "pub trait ShapeImageResolver",
            "pub struct ShapeImageResolverScope",
            "CURRENT_SHAPE_IMAGE_RESOLVER",
            "pub fn CurrentShapeImageResolver",
            "pub fn CurrentShapeImage",
        ] {
            assert!(
                float_services.contains(owned_policy),
                "layout_algo_float shape-image services lost {owned_policy}"
            );
        }
        assert!(
            !float_services.contains("#[inline(never)]"),
            "constant-time shape-image TLS access must remain inlineable"
        );

        let float_services_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/shapes/shape_image_services.rs"),
        )
        .expect("read shape-image services compatibility facade");
        assert!(
            float_services_facade.contains(concat!(
                "layout_algo_",
                "float/src/shape_image_services_interface.rs"
            )) && float_services_facade
                .contains("pub use layout_algo_float_shape_image_services_interface::*"),
            "core shape-image services must remain a facade over layout_algo_float"
        );
        for leaked_policy in [
            "pub trait ShapeImageResolver",
            "pub struct ShapeImageResolverScope",
            "CURRENT_SHAPE_IMAGE_RESOLVER",
            "pub fn CurrentShapeImageResolver",
        ] {
            assert!(
                !float_services_facade.contains(leaked_policy),
                "core shape-image services facade regained float policy {leaked_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source layout boundary adapter");
        for owner in [
            concat!("layout_algo_", "inline/src/text_input_environment.rs"),
            concat!("layout_algo_", "float/src/shape_image_environment.rs"),
        ] {
            assert!(
                adapter.contains(owner),
                "layout boundary must import {owner}"
            );
        }
        for leaked_policy in [
            "fn ProviderLastHyphenLocation(",
            "struct InputHyphenationResolver",
            "struct InputTextTransformResolver",
            "struct InputPhraseBreakResolver",
            "struct InputShapeImageResolver",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout boundary still owns algorithm input policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn bfc_reuse_policy_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let policy =
            std::fs::read_to_string(root.join("layout_reuse/src/block_formatting_context.rs"))
                .expect("read BFC reuse policy");
        assert!(
            policy.contains("#[inline(never)]")
                && policy.contains("pub fn calculate_block_formatting_context_reuse"),
            "BFC reuse policy must stay out of line in layout_reuse for sampling attribution"
        );

        let adapter = std::fs::read_to_string(root.join("layout_reuse/src/layout_utils.rs"))
            .expect("read source layout reuse adapter owner");
        assert!(
            adapter.contains("layout_reuse::calculate_block_formatting_context_reuse")
                && adapter.contains("#[inline(never)]\npub fn CalculateSizeBasedLayoutCacheStatus")
                && adapter
                    .contains("#[inline(never)]\npub fn MaySkipLayoutWithinBlockFormattingContext"),
            "source BFC and size-based reuse admission must retain layout_reuse sampling frames"
        );
        for misplaced_helper in [
            "ResolveContentAlignment",
            "ComputeContentAlignmentForTableCell",
            "InlineStaticPositionEdge",
            "BlockStaticPositionEdge",
        ] {
            assert!(
                !adapter.contains(&format!("pub fn {misplaced_helper}")),
                "normal layout helper {misplaced_helper} must not be owned by cache reuse"
            );
        }

        let alignment =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_alignment_utils.rs"))
                .expect("read shared alignment policy");
        for inline_helper in [
            "ResolveContentAlignment",
            "ComputeContentAlignmentForTableCell",
            "InlineStaticPositionEdge",
            "BlockStaticPositionEdge",
        ] {
            assert!(
                alignment.contains(&format!("pub fn {inline_helper}")),
                "shared alignment policy lost {inline_helper}"
            );
        }

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_utils.rs"))
            .expect("read source layout reuse facade");
        assert!(
            facade.contains("layout_reuse/src/layout_utils.rs")
                && facade.contains("CalculateSizeBasedLayoutCacheStatus")
                && facade.contains("pub use super::layout_alignment_utils")
                && !facade.contains("calculate_block_formatting_context_reuse"),
            "core layout_utils must expose reuse admission separately from shared alignment policy"
        );
    }

    #[test]
    fn min_max_size_values_resolve_to_the_geometry_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let geometry =
            std::fs::read_to_string(root.join("layoutng_geometry/src/geometry/min_max_sizes.rs"))
                .expect("read shared min/max size model");
        for owned_policy in [
            "pub struct MinMaxSizes",
            "pub struct MinMaxSizesResult",
            "pub fn ShrinkToFit",
            "impl AddAssign for MinMaxSizes",
        ] {
            assert!(
                geometry.contains(owned_policy),
                "layoutng_geometry min/max size model lost {owned_policy}"
            );
        }

        let compatibility = std::fs::read_to_string(root.join("layout_reuse/src/min_max_sizes.rs"))
            .expect("read reuse compatibility export");
        assert!(
            compatibility.contains("layoutng_geometry::geometry::min_max_sizes"),
            "reuse compatibility export must use the geometry-owned value"
        );
        for misplaced_policy in ["pub struct MinMaxSizes", "impl AddAssign for MinMaxSizes"] {
            assert!(
                !compatibility.contains(misplaced_policy),
                "cache reuse must not own shared sizing policy {misplaced_policy}"
            );
        }

        let core = std::fs::read_to_string(root.join("layoutng/src/internal/min_max_sizes.rs"))
            .expect("read core min/max size facade");
        assert!(
            core.contains("layoutng_geometry::geometry::min_max_sizes"),
            "core must expose the geometry-owned min/max size model directly"
        );
    }

    #[test]
    fn cache_lookup_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let lookup = std::fs::read_to_string(root.join("layout_reuse/src/cache_lookup.rs"))
            .expect("read source cache lookup");
        assert!(
            lookup.contains("#[inline(never)]") && lookup.contains("pub fn cached_layout_result"),
            "source cache lookup must stay out of line in the layout_reuse owner"
        );
        for policy_marker in [
            "NeedsOOFPositionedInfoPropagation",
            "HasFragmentainerBreak",
            "CalculateSizeBasedLayoutCacheStatus",
            "MaySkipLayoutWithinBlockFormattingContext",
        ] {
            assert!(
                lookup.contains(policy_marker),
                "layout_reuse source lookup lost {policy_marker}"
            );
        }

        let adapter = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox cache adapter");
        assert!(
            adapter.contains("crate::layout_reuse_package::cached_layout_result"),
            "LayoutBox cache entry must delegate to layout_reuse"
        );
        for leaked_policy in [
            "NeedsOOFPositionedInfoPropagation",
            "HasFragmentainerBreak",
            "CalculateSizeBasedLayoutCacheStatus",
            "MaySkipLayoutWithinBlockFormattingContext",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutBox cache adapter still owns reuse policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn measure_cache_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_reuse/src/measure_cache.rs"))
            .expect("read source measure-cache owner");
        for owned_policy in [
            "pub struct MeasureCache",
            "#[inline(never)]\n    pub fn Find",
            ".touch_where(",
            "CalculateSizeBasedLayoutCacheStatus(",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse source measurement cache lost {owned_policy}"
            );
        }

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/measure_cache.rs"))
            .expect("read source measure-cache facade");
        assert!(
            facade.contains("layout_reuse/src/measure_cache.rs")
                && facade.contains("pub use layout_reuse_measure_cache::MeasureCache"),
            "core measurement cache must be a reuse-owner facade"
        );
        for leaked_policy in [".touch_where(", "CalculateSizeBasedLayoutCacheStatus("] {
            assert!(
                !facade.contains(leaked_policy),
                "core measurement-cache facade still owns reuse policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_result_cache_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_reuse/src/layout_box_result_cache.rs"))
                .expect("read source per-box layout-result cache owner");
        for owned_policy in [
            "fn LayoutBoxAddMeasureLayoutResult",
            "fn LayoutBoxSetCachedLayoutResult",
            "fn LayoutBoxGetCachedLayoutResult",
            "fn LayoutBoxGetCachedMeasureResult",
            "CacheSlot() == LayoutResultCacheSlot::kMeasure",
            "SetFragmentChildrenInvalid",
            "measure_cache }.Find",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse per-box result cache lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 6,
            "per-box result admission and lookup entries must retain reuse sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_reuse/src/layout_box_result_cache.rs")
                && facade.contains("LayoutBoxSetCachedLayoutResult(self, result, index)")
                && facade.contains("LayoutBoxGetCachedMeasureResult("),
            "LayoutBox must delegate per-box result caching to layout_reuse"
        );
        for leaked_policy in [
            "SetFragmentChildrenInvalid(result)",
            "CacheSlot() == LayoutResultCacheSlot::kMeasure",
            "GetLastForTesting()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns result-cache policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn managed_input_index_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_reuse/src/managed_input_index.rs"))
            .expect("read source managed-input index owner");
        for owned_policy in [
            "fn BuildManagedInputIndex",
            "fn remove_managed_input",
            "fn insert_managed_input",
            "for (key, record) in &storage.managed",
            "keys.retain",
            ".by_id",
            ".children",
            "key.clone()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse managed-input index lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            3,
            "managed-input rebuild, removal, and insertion must retain distinct reuse frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder facade");
        assert!(
            facade.contains("layout_reuse/src/managed_input_index.rs")
                && facade.contains("if let Some(index) = self.input_index.get()")
                && facade.contains("BuildManagedInputIndex(self)"),
            "LayoutObject builder must keep an inline cache hit and delegate cache construction to layout_reuse"
        );
        assert!(
            !facade.contains("for (key, record) in &self.managed")
                && !facade.contains("keys.retain")
                && !facade.contains("impl ManagedInputIndex"),
            "LayoutObject builder still owns managed-input index policy"
        );
    }

    #[test]
    fn managed_object_compatibility_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_reuse/src/managed_object_compatibility.rs"))
                .expect("read managed-object compatibility owner");
        for owned_policy in [
            "pub(super) fn Compatible",
            "pub(super) fn StyleUpdateCompatible",
            "fn object_kind_compatible",
            "fn managed_object_compatible",
            "fn managed_style_update_compatible",
            "extended.timeline_trigger_names",
            "StyleComputedStyle::NeedsReinsertLayoutTree",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse managed-object compatibility lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            3,
            "object-kind, complete-update, and targeted-style reuse admission need distinct frames"
        );

        let core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder facade");
        assert!(
            core.contains("layout_reuse/src/managed_object_compatibility.rs"),
            "LayoutObject builder must load reuse-owned compatibility policy"
        );
        for leaked_policy in [
            "fn ObjectKindCompatible(",
            "fn Compatible(",
            "fn StyleUpdateCompatible(",
            "NeedsReinsertLayoutTree(old_style, new_style)",
        ] {
            assert!(
                !core.contains(leaked_policy),
                "LayoutObject builder still owns reuse compatibility policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn collapsible_whitespace_resolves_to_the_inline_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "inline/src/collapsible_whitespace.rs"
        )))
        .expect("read source collapsible-whitespace owner");
        for owned_policy in [
            "fn needs_collapsible_whitespace",
            "while !previous.is_null()",
            "IsFloatingOrOutOfFlowPositioned",
            "TransformedText",
            "parent.IsLayoutInline()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "inline algorithm collapsible-whitespace lookup lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "collapsible-whitespace sibling lookup must retain an inline-algorithm sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "inline/src/collapsible_whitespace.rs"
            )) && facade.contains(
                "layout_algo_inline_collapsible_whitespace::needs_collapsible_whitespace(self, parent)"
            ),
            "LayoutObject builder must delegate collapsible-whitespace lookup to the inline algorithm"
        );
        for leaked_policy in [
            "while !previous.is_null()",
            "IsFloatingOrOutOfFlowPositioned()",
            "TransformedText()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject builder still owns inline whitespace policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn inline_state_resolves_to_the_inline_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        for (owner_file, facade_file, owned_type, reexport) in [
            (
                "hyphen_result_interface.rs",
                "hyphen_result.rs",
                "pub struct HyphenResult {",
                concat!("pub use layout_algo_", "inline_hyphen_result_interface::*;"),
            ),
            (
                "inline_item_interface.rs",
                "inline_item.rs",
                "pub struct InlineItem {",
                concat!("pub use layout_algo_", "inline_inline_item_interface::*;"),
            ),
            (
                "inline_item_result_interface.rs",
                "inline_item_result.rs",
                "pub struct InlineItemResult {",
                concat!(
                    "pub use layout_algo_",
                    "inline_inline_item_result_interface::*;"
                ),
            ),
            (
                "inline_item_segment_interface.rs",
                "inline_item_segment.rs",
                "pub struct InlineItemSegment {",
                concat!(
                    "pub use layout_algo_",
                    "inline_inline_item_segment_interface::*;"
                ),
            ),
            (
                "inline_item_span.rs",
                "inline_item_span.rs",
                "pub struct InlineItemSpan {",
                concat!("pub use layout_algo_", "inline_inline_item_span::*;"),
            ),
            (
                "inline_item_text_index.rs",
                "inline_item_text_index.rs",
                "pub struct InlineItemTextIndex {",
                concat!("pub use layout_algo_", "inline_inline_item_text_index::*;"),
            ),
            (
                "inline_node_interface.rs",
                "inline_node.rs",
                "pub struct InlineNode {",
                concat!("pub use layout_algo_", "inline_inline_node_interface::*;"),
            ),
            (
                "inline_node_data.rs",
                "inline_node_data.rs",
                "pub struct InlineNodeData {",
                concat!("pub use layout_algo_", "inline_inline_node_data::*;"),
            ),
            (
                "layout_text_combine_interface.rs",
                "layout_text_combine.rs",
                "pub struct LayoutTextCombine {",
                concat!(
                    "pub use layout_algo_",
                    "inline_layout_text_combine_interface::*;"
                ),
            ),
            (
                "layout_inline.rs",
                "layout_inline.rs",
                "pub struct LayoutInline {",
                concat!("pub use layout_algo_", "inline_layout_inline::*;"),
            ),
            (
                "layout_text_interface.rs",
                "layout_text.rs",
                "pub struct LayoutText {",
                concat!("pub use layout_algo_", "inline_layout_text_interface::*;"),
            ),
            (
                "text_fit_scale.rs",
                "text_fit_scale.rs",
                "pub struct TextFitScale {",
                concat!("pub use layout_algo_", "inline_text_fit_scale::*;"),
            ),
            (
                "text_item_type.rs",
                "text_item_type.rs",
                "pub enum TextItemType",
                concat!("pub use layout_algo_", "inline_text_item_type::*;"),
            ),
            (
                "used_font_interface.rs",
                "used_font.rs",
                "pub struct UsedFont {",
                concat!("pub use layout_algo_", "inline_used_font_interface::*;"),
            ),
            (
                "variable_length_transform_result.rs",
                "variable_length_transform_result.rs",
                "pub struct VariableLengthTransformResult {",
                concat!(
                    "pub use layout_algo_",
                    "inline_variable_length_transform_result::*;"
                ),
            ),
        ] {
            let owner = std::fs::read_to_string(
                root.join(concat!("layout_algo_", "inline/src"))
                    .join(owner_file),
            )
            .unwrap_or_else(|_| panic!("read inline owner {owner_file}"));
            assert!(
                owner.contains(owned_type),
                "inline algorithm owner {owner_file} lost {owned_type}"
            );

            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read inline facade {facade_file}"));
            assert!(
                facade.contains(&format!("inline/src/{owner_file}"))
                    && facade.contains(reexport)
                    && !facade.contains(owned_type),
                "inline type {owned_type} must be re-exported from {owner_file}"
            );
        }

        let core_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "inline/src/inline_node_core.rs")),
        )
        .expect("read source InlineNode core owner");
        assert!(
            core_owner.contains("impl InlineNode")
                && core_owner.contains("LayoutPassScope::Algorithms()"),
            "InlineNode core behavior must remain physically inline-owned"
        );
        let core_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/inline_node_core.rs"))
                .expect("read InlineNode core compatibility loader");
        assert!(
            core_facade.contains(concat!("layout_algo_", "inline/src/inline_node_core.rs"))
                && !core_facade.contains("impl InlineNode"),
            "InlineNode core loader must not regain inline algorithm behavior"
        );

        let text_core = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "inline/src/layout_text_core.rs")),
        )
        .expect("read source LayoutText core owner");
        assert!(
            text_core.contains("impl LayoutText")
                && text_core.contains("fn DetachAxHooks")
                && text_core.contains("fn ClearBlockFlowCachedData"),
            "LayoutText core behavior must remain physically inline-owned"
        );
        let text_core_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_text_core.rs"))
                .expect("read LayoutText core compatibility loader");
        assert!(
            text_core_facade.contains(concat!("layout_algo_", "inline/src/layout_text_core.rs"))
                && !text_core_facade.contains("impl LayoutText"),
            "LayoutText core loader must not regain inline behavior"
        );

        let hyphen_provider = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "inline/src/hyphen_result.rs")),
        )
        .expect("read inline hyphen shaping provider");
        assert!(
            hyphen_provider.contains("#[inline(never)]")
                && hyphen_provider.contains("fn ShapeHyphenResult"),
            "hyphen shaping must retain an inline-algorithm sampling frame"
        );
        let font_provider =
            std::fs::read_to_string(root.join(concat!("layout_algo_", "inline/src/used_font.rs")))
                .expect("read inline used-font metric provider");
        assert!(
            font_provider.contains("fn UsedFontFixedAscentForBaseline")
                && font_provider.contains("fn UsedFontUnderlineThickness"),
            "used-font metric queries must remain implemented by the inline owner"
        );
    }

    #[test]
    fn variable_length_transform_index_resolves_to_the_inline_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "inline/src/layout_view_variable_length_transform_index.rs"
        )))
        .expect("read variable-length transform index owner");
        for owned_policy in [
            "fn RegisterVariableLengthTransformResult",
            "fn UnregisterVariableLengthTransformResult",
            "fn GetVariableLengthTransformResult",
            "text_to_variable_length_transform_result_",
            "WeakMember::from_ptr",
        ] {
            assert!(
                owner.contains(owned_policy),
                "inline variable-length transform index lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "variable-length transform registration changes must retain inline sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView inline index loader");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "inline/src/layout_view_variable_length_transform_index.rs"
            )) && !loader.contains("fn RegisterVariableLengthTransformResult")
                && !loader.contains("fn UnregisterVariableLengthTransformResult")
                && !loader.contains("fn GetVariableLengthTransformResult"),
            "LayoutView must load variable-length transform indexing from the inline owner"
        );
    }

    #[test]
    fn layout_object_editing_queries_resolve_to_the_inline_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "inline/src/layout_object_editing_queries.rs"
        )))
        .expect("read LayoutObject editing-query owner");
        for owned_policy in [
            "fn CanBeSelectionLeafInternalBase",
            "fn CanBeSelectionLeafInternal(",
            "fn PositionForPoint",
            "fn CreatePositionWithAffinity(",
            "fn CreatePositionWithAffinityDefault",
            "fn FindPosition",
            "fn FirstPositionInOrBeforeThis",
            "fn LastPositionInOrAfterThis",
            "fn PositionAfterThis",
            "fn PositionBeforeThis",
            "fn AssociatedLayoutObjectOf(",
            "fn AssociatedLayoutObjectOfDefault",
            "DispatchLayoutObjectPositionForPoint",
            "AssociatedLayoutObjectOfProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "inline LayoutObject editing query lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            9,
            "eight position providers and node/object association must retain inline sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "inline/src/layout_object_editing_queries.rs"
            )) && loader.contains("pub use layout_algo_inline_layout_object_editing_queries::{"),
            "LayoutObject must load and re-export editing queries from the inline owner"
        );
        for leaked_policy in [
            "fn CanBeSelectionLeafInternalBase",
            "fn CanBeSelectionLeafInternal(",
            "fn PositionForPoint",
            "fn CreatePositionWithAffinity(",
            "fn CreatePositionWithAffinityDefault",
            "fn FindPosition",
            "fn FirstPositionInOrBeforeThis",
            "fn LastPositionInOrAfterThis",
            "fn PositionAfterThis",
            "fn PositionBeforeThis",
            "fn AssociatedLayoutObjectOf(",
            "fn AssociatedLayoutObjectOfDefault",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns inline editing query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_list_marker_queries_resolve_to_the_list_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "list/src/layout_object_list_marker_queries.rs"
        )))
        .expect("read LayoutObject list-marker-query owner");
        for owned_policy in [
            "fn IsListMarkerForSummary",
            "fn IsInListMarker",
            "LayoutObjectIsListMarkerForSummaryProvider",
            "LayoutObjectIsInListMarkerProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "list LayoutObject marker query lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "fixed-cost list-marker predicates must remain inlineable"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "list/src/layout_object_list_marker_queries.rs"
            )),
            "LayoutObject must load marker queries from the list owner"
        );
        for leaked_policy in ["fn IsListMarkerForSummary", "fn IsInListMarker"] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns list marker query {leaked_policy}"
            );
        }
    }

    #[test]
    fn scroll_input_update_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/scroll_input_update.rs"))
                .expect("read source scroll-input update owner");
        for owned_policy in [
            "fn update_scroll_offset",
            "fn update_scrollbars_hidden_if_overlay",
            "for key in storage.InputKeys(id).to_vec()",
            "element.scroll_offset = offset",
            "UpdateInputScrollOffset(offset)",
            "SetScrollbarsHiddenIfOverlayFromOwner(hidden)",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment scroll-input update lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "scroll-input fan-out must retain a fragment sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder facade");
        assert!(
            facade.contains("layout_fragment/src/scroll_input_update.rs")
                && facade.contains(
                    "layout_fragment_scroll_input::update_scroll_offset(self, id, offset)"
                )
                && facade.contains(
                    "layout_fragment_scroll_input::update_scrollbars_hidden_if_overlay(self, id, hidden)"
                ),
            "LayoutObject builder must delegate scroll-input mutation to layout_fragment"
        );
        for leaked_policy in [
            "element.scroll_offset = offset",
            "UpdateInputScrollOffset(offset)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject builder still owns scroll-input policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragmentation_context_resolves_to_the_block_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/fragmentation_context.rs"
        )))
        .expect("read source fragmentation-context owner");
        for owned_policy in [
            "fn fragmentation_context",
            "while !ancestor.is_null()",
            "ComputedStyleRef().ColumnCount() > 1",
            "FragmentationType::kFragmentColumn",
            "FragmentationType::kFragmentPage",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block algorithm fragmentation-context lookup lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "fragmentation-context ancestor lookup must retain a block-algorithm sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder facade");
        let transaction =
            std::fs::read_to_string(root.join("layout_invalidation/src/managed_object_update.rs"))
                .expect("read invalidation-owned managed-object update");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "block/src/fragmentation_context.rs"
            )) && transaction
                .contains("layout_algo_block_fragmentation_context::fragmentation_context(parent)"),
            "managed-object update must delegate fragmentation-context lookup to the block algorithm"
        );
        for leaked_policy in [
            "let mut ancestor = parent",
            "InputStyle().column_count > 1",
            "FragmentationType::kFragmentColumn",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject builder still owns fragmentation policy {leaked_policy}"
            );
        }
        assert!(
            !transaction.contains("let mut ancestor = parent")
                && !transaction.contains("InputStyle().column_count > 1"),
            "managed-object update must not absorb block fragmentation traversal policy"
        );
    }

    #[test]
    fn block_node_traversal_resolves_to_the_block_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/block_node_traversal.rs")),
        )
        .expect("read source BlockNode traversal owner");
        for owned_policy in [
            "fn BlockNodeNextBlockSibling",
            "fn BlockNodeFirstChild",
            "fn BlockNodeGetRenderedLegend",
            "while !next_sibling.is_null()",
            "while !child.is_null()",
            "IsRenderedLegendCandidate",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block algorithm BlockNode traversal lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "sibling, first-child, and legend scans must retain block-algorithm sampling frames"
        );

        let node_owner =
            std::fs::read_to_string(root.join(concat!("layout_algo_", "block/src/block_node.rs")))
                .expect("read source BlockNode owner");
        assert!(
            node_owner.contains("pub struct BlockNode")
                && node_owner.contains("block_node_traversal.rs")
                && node_owner.contains("BlockNodeNextBlockSibling(self)")
                && node_owner.contains("BlockNodeFirstChild(self)")
                && node_owner.contains("BlockNodeGetRenderedLegend(self)"),
            "BlockNode must delegate traversal policy to the block algorithm"
        );
        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/block_node.rs"))
            .expect("read BlockNode compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "block/src/block_node.rs"))
                && facade.contains(concat!("pub use layout_algo_", "block_block_node::*;"))
                && !facade.contains("pub struct BlockNode"),
            "BlockNode interface must be re-exported from the block algorithm owner"
        );
        for leaked_policy in [
            "while !next_sibling.is_null()",
            "while !child.is_null()",
            "IsRenderedLegendCandidate()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "BlockNode facade still owns traversal policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_block_traversal_resolves_to_the_block_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/layout_block_traversal.rs"
        )))
        .expect("read source LayoutBlock traversal owner");
        for owned_policy in [
            "fn LayoutBlockFirstLineStyleParentBlock",
            "fn LayoutBlockNearestInnerBlockWithFirstLine",
            "while unsafe { &*first_child }.IsFloatingOrOutOfFlowPositioned()",
            "while !child.is_null()",
            "FirstLineOnListItemEnabled",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block algorithm LayoutBlock traversal lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "first-line sibling and nested-block walks must retain block-algorithm sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block.rs")),
        )
        .expect("read source LayoutBlock owner");
        assert!(
            facade.contains("layout_block_traversal.rs")
                && facade.contains("LayoutBlockFirstLineStyleParentBlock(self)")
                && facade.contains("LayoutBlockNearestInnerBlockWithFirstLine(self)"),
            "LayoutBlock must delegate first-line traversal to the block algorithm"
        );
        for leaked_policy in [
            "while unsafe { &*first_child }.IsFloatingOrOutOfFlowPositioned()",
            "while !child.is_null()",
            "FirstLineOnListItemEnabled()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBlock facade still owns first-line traversal {leaked_policy}"
            );
        }
    }

    #[test]
    fn block_object_interfaces_resolve_to_the_block_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        for (owner_file, facade_file, owned_type, reexport) in [
            (
                "layout_block.rs",
                "layout_block.rs",
                "pub struct LayoutBlock {",
                concat!("pub use layout_algo_", "block_layout_block::*;"),
            ),
            (
                "layout_block_flow.rs",
                "layout_block_flow.rs",
                "pub struct LayoutBlockFlow {",
                concat!("pub use layout_algo_", "block_layout_block_flow::*;"),
            ),
        ] {
            let owner = std::fs::read_to_string(
                root.join(concat!("layout_algo_", "block/src"))
                    .join(owner_file),
            )
            .unwrap_or_else(|_| panic!("read block interface owner {owner_file}"));
            assert!(
                owner.contains(owned_type),
                "block interface owner {owner_file} lost {owned_type}"
            );
            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read block interface facade {facade_file}"));
            assert!(
                facade.contains(&format!("block/src/{owner_file}"))
                    && facade.contains(reexport)
                    && !facade.contains(owned_type),
                "block interface {owned_type} must be re-exported from {owner_file}"
            );
        }
    }

    #[test]
    fn line_clamp_data_resolves_to_the_block_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/line_clamp_data.rs")),
        )
        .expect("read source line-clamp owner");
        for owned_policy in [
            "pub enum State",
            "pub struct LineClampData",
            "pub struct LineClampAncestorChain",
            "fn LinesUntilClamp",
            "fn FinalLineClampBlockSize",
            "fn WithResolvedBfcOffset",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block algorithm line-clamp state lost {owned_policy}"
            );
        }

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/line_clamp_data.rs"))
            .expect("read line-clamp compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "block/src/line_clamp_data.rs"))
                && facade.contains("pub use layout_algo_block_line_clamp_data::*;"),
            "line-clamp state must be re-exported from the block algorithm owner"
        );
        for leaked_policy in [
            "pub struct LineClampData",
            "pub struct LineClampAncestorChain",
            "fn FinalLineClampBlockSize",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "line-clamp facade still owns block-algorithm policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn grid_style_projection_resolves_to_the_grid_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "grid/src/grid_style_projection.rs")),
        )
        .expect("read source Grid style-projection owner");
        for owned_policy in [
            "fn TrackBreadth",
            "fn Tracks",
            "fn AutoTracks",
            "fn GridPosition",
            "fn GridTemplateAreas",
            "for repeater in &tracks.repeaters",
            "for repetition in 0..repetitions",
            "SortNamedLines",
            "UntranslatedDefiniteGridSpan",
        ] {
            assert!(
                owner.contains(owned_policy),
                "Grid algorithm source style projection lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "track expansion, auto-track projection, and template areas must retain Grid sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source style input");
        assert!(
            facade.contains(concat!("layout_algo_", "grid/src/grid_style_projection.rs"))
                && facade.contains("AutoTracks, GridPosition, GridTemplateAreas, Tracks"),
            "source style-input boundary must import Grid projection from its algorithm owner"
        );
        for leaked_policy in [
            "fn TrackBreadth(",
            "fn Tracks(",
            "fn GridTemplateAreas(",
            "for repeater in &tracks.repeaters",
            "UntranslatedDefiniteGridSpan",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "source style-input facade still owns Grid projection policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn grid_track_list_input_resolves_to_the_grid_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "grid/src/grid_track_list_input.rs")),
        )
        .expect("read Grid track-list input owner");
        for owned_policy in [
            "impl GridTrackListInput",
            "#[inline(always)]\n    pub fn from_tracks",
            "#[inline(never)]\nfn copy_grid_track_list_input",
            "#[inline(never)]\nfn take_grid_track_list_input",
            "#[inline(never)]\nfn clear_grid_track_list_input",
            "tracks.to_vec()",
            "result.repeaters.push",
        ] {
            assert!(
                owner.contains(owned_policy),
                "Grid track-list input owner lost {owned_policy}"
            );
        }

        let core = std::fs::read_to_string(root.join("layoutng/src/internal/layout_input.rs"))
            .expect("read shared layout-input schema");
        assert!(
            core.contains(concat!("layout_algo_", "grid/src/grid_track_list_input.rs")),
            "layout_input must load the Grid-owned track-list implementation"
        );
        for leaked_policy in [
            "pub fn from_tracks",
            "impl From<Vec<GridTrack>> for GridTrackListInput",
            "tracks.to_vec()",
        ] {
            assert!(
                !core.contains(leaked_policy),
                "layout_input still owns Grid track-list policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn grid_layout_data_resolves_to_the_grid_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "grid/src/grid_layout_data.rs")),
        )
        .expect("read source Grid layout-data owner");
        for owned_policy in [
            "pub struct GridLayoutData",
            "pub struct GridTreeNode",
            "pub struct GridLayoutTree",
            "fn AreSubtreesEqual",
            "for index in 0..subtree_size",
            "fn from_subtree",
        ] {
            assert!(
                owner.contains(owned_policy),
                "Grid algorithm layout data lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "Grid subtree comparison must retain a Grid sampling frame"
        );

        let item_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "grid/src/grid_item_data.rs")),
        )
        .expect("read source Grid item-data owner");
        for owned_type in [
            "pub struct GridPlacementData",
            "pub struct GridItemData",
            "pub struct GridItems",
            "pub struct OutOfFlowItemPlacement",
        ] {
            assert!(
                item_owner.contains(owned_type),
                "Grid algorithm item data lost {owned_type}"
            );
        }
        let track_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "grid/src/grid_track_collection_types.rs"
        )))
        .expect("read source Grid track-collection owner");
        for owned_type in [
            "pub struct GridRangeBuilder",
            "pub struct GridLayoutTrackCollection",
            "pub struct GridSizingTrackCollection",
            "pub struct GridTrackBaselines",
        ] {
            assert!(
                track_owner.contains(owned_type),
                "Grid algorithm track collection lost {owned_type}"
            );
        }
        let lanes_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "grid/src/grid_lanes_item_group.rs")),
        )
        .expect("read source Grid-lanes item-group owner");
        assert!(
            lanes_owner.contains("pub struct GridLanesItemGroupProperties")
                && lanes_owner.contains("pub struct GridLanesItemGroup")
                && lanes_owner.contains("pub struct VirtualItems"),
            "Grid-lanes item grouping must remain Grid-owned"
        );
        let subtree_owner =
            std::fs::read_to_string(root.join(concat!("layout_algo_", "grid/src/grid_subtree.rs")))
                .expect("read source Grid subtree owner");
        assert!(
            subtree_owner.contains("pub trait GridSubtreeTree")
                && subtree_owner.contains("pub struct GridSubtree"),
            "Grid subtree cursor must remain Grid-owned"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/grid_layout_data.rs"))
                .expect("read Grid layout-data facade");
        assert!(
            facade.contains(concat!("layout_algo_", "grid/src/grid_layout_data.rs"))
                && facade.contains("pub use layout_algo_grid_grid_layout_data::*;"),
            "Grid layout data must be re-exported from the Grid algorithm owner"
        );
        for leaked_policy in [
            "pub struct GridLayoutData",
            "pub struct GridLayoutTree",
            "for index in 0..subtree_size",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "Grid layout-data facade still owns algorithm policy {leaked_policy}"
            );
        }
        let item_facade = std::fs::read_to_string(root.join("layoutng/src/internal/grid_item.rs"))
            .expect("read Grid item-data facade");
        let track_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/grid_track_collection.rs"))
                .expect("read Grid track-collection facade");
        assert!(
            item_facade.contains(concat!("layout_algo_", "grid/src/grid_item_data.rs"))
                && item_facade.contains("pub use layout_algo_grid_grid_item_data::*;")
                && track_facade.contains(concat!(
                    "layout_algo_",
                    "grid/src/grid_track_collection_types.rs"
                ))
                && track_facade
                    .contains("pub use layout_algo_grid_grid_track_collection_types::*;")
                && !item_facade.contains("pub struct GridItemData")
                && !track_facade.contains("pub struct GridLayoutTrackCollection"),
            "Grid item and track types must be re-exported from the Grid algorithm owner"
        );
        let lanes_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/grid_lanes_item_group.rs"))
                .expect("read Grid-lanes item-group facade");
        let subtree_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/grid_subtree.rs"))
                .expect("read Grid subtree facade");
        assert!(
            lanes_facade.contains(concat!("layout_algo_", "grid/src/grid_lanes_item_group.rs"))
                && lanes_facade.contains("pub use layout_algo_grid_grid_lanes_item_group::*;")
                && subtree_facade.contains(concat!("layout_algo_", "grid/src/grid_subtree.rs"))
                && subtree_facade.contains("pub use layout_algo_grid_grid_subtree::*;")
                && !lanes_facade.contains("pub struct GridLanesItemGroup")
                && !subtree_facade.contains("pub struct GridSubtree"),
            "Grid-lanes groups and subtree cursor must be re-exported from the Grid owner"
        );
    }

    #[test]
    fn table_constraint_data_resolves_to_the_table_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "table/src/table_constraint_space_data.rs"
        )))
        .expect("read source table constraint-space data owner");
        for owned_policy in [
            "pub struct TableConstraintSpaceData",
            "fn MaySkipRowLayout",
            "fn MaySkipSectionLayout",
            "while new_cell_index < new_end_cell_index",
            "while new_row_index < new_end_row_index",
        ] {
            assert!(
                owner.contains(owned_policy),
                "table algorithm constraint-space data lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "row and section reuse comparisons must retain table sampling frames"
        );

        let border_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "table/src/table_borders_types.rs")),
        )
        .expect("read source table-border type owner");
        for owned_type in [
            "pub enum EdgeSource",
            "pub enum EdgeSide",
            "pub struct Edge",
            "pub struct TableBorders",
        ] {
            assert!(
                border_owner.contains(owned_type),
                "table algorithm border state lost {owned_type}"
            );
        }
        let column_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "table/src/table_column_location.rs"
        )))
        .expect("read source table-column location owner");
        assert!(
            column_owner.contains("pub struct TableColumnLocation"),
            "table algorithm column-location value was lost"
        );
        let types_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "table/src/table_layout_algorithm_types_interface.rs"
        )))
        .expect("read source table layout-type owner");
        for owned_type in [
            "pub struct CellInlineConstraint",
            "pub struct Column",
            "pub struct Row",
            "pub struct Section",
            "pub struct TableGroupedChildren",
        ] {
            assert!(
                types_owner.contains(owned_type),
                "table algorithm state lost {owned_type}"
            );
        }
        let node_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "table/src/table_node_interface.rs")),
        )
        .expect("read source table-node owner");
        assert!(
            node_owner.contains("pub struct TableNode")
                && node_owner.contains("fn ComputeTableInlineSize")
                && node_owner.contains("fn GetColumnConstraints"),
            "table node interface must remain table-owned"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/table_constraint_space_data.rs"),
        )
        .expect("read table constraint-space data facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "table/src/table_constraint_space_data.rs"
            )) && facade.contains("pub use layout_algo_table_table_constraint_space_data::*;"),
            "table constraint-space data must be re-exported from the table algorithm owner"
        );
        for leaked_policy in [
            "pub struct TableConstraintSpaceData",
            "while new_cell_index <",
            "while new_row_index <",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "table constraint-data facade still owns algorithm policy {leaked_policy}"
            );
        }
        let border_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/table_borders.rs"))
                .expect("read table-border facade");
        let column_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/table_column_location.rs"))
                .expect("read table-column location facade");
        assert!(
            border_facade.contains(concat!("layout_algo_", "table/src/table_borders_types.rs"))
                && border_facade.contains("pub use layout_algo_table_table_borders_types::*;")
                && column_facade.contains(concat!(
                    "layout_algo_",
                    "table/src/table_column_location.rs"
                ))
                && column_facade.contains("pub use layout_algo_table_table_column_location::*;")
                && !border_facade.contains("pub struct TableBorders")
                && !column_facade.contains("pub struct TableColumnLocation"),
            "table border and column values must be re-exported from the table algorithm owner"
        );
        let types_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/table_layout_algorithm_types.rs"),
        )
        .expect("read table layout-type facade");
        let node_facade = std::fs::read_to_string(root.join("layoutng/src/internal/table_node.rs"))
            .expect("read table-node facade");
        assert!(
            types_facade.contains(concat!(
                "layout_algo_",
                "table/src/table_layout_algorithm_types_interface.rs"
            )) && types_facade
                .contains("pub use layout_algo_table_table_layout_algorithm_types_interface::*;")
                && node_facade
                    .contains(concat!("layout_algo_", "table/src/table_node_interface.rs"))
                && node_facade.contains("pub use layout_algo_table_table_node_interface::*;")
                && !types_facade.contains("pub struct CellInlineConstraint")
                && !node_facade.contains("pub struct TableNode"),
            "table algorithm state and node interface must be re-exported from the table owner"
        );
    }

    #[test]
    fn shape_style_projection_resolves_to_the_float_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "float/src/shape_style_projection.rs"
        )))
        .expect("read source shape style-projection owner");
        for owned_policy in [
            "fn ApplyStyleShape",
            "for point in &shape.contour",
            "for radius in [&shape.radius_x, &shape.radius_y]",
            "for inset in &shape.insets",
            "ShapeValue::from_box",
            "SetShapeImageThreshold",
        ] {
            assert!(
                owner.contains(owned_policy),
                "float algorithm source shape projection lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "shape contour projection must retain a float-algorithm sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source style input");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "float/src/shape_style_projection.rs"
            )) && facade.contains("layout_algo_float_shape_style_projection::ApplyStyleShape"),
            "source style-input boundary must import shape projection from its float owner"
        );
        for leaked_policy in [
            "fn ApplyStyleShape(",
            "for point in &shape.contour",
            "for inset in &shape.insets",
            "ShapeValue::from_box",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "source style-input facade still owns shape projection policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn exclusion_space_resolves_to_the_float_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/exclusion_space.rs")),
        )
        .expect("read source exclusion-space owner");
        for owned_policy in [
            "pub struct DerivedGeometry",
            "pub struct ExclusionSpaceInternal",
            "pub struct ExclusionSpace",
            "fn InsertClosedArea",
            "fn RebuildDerivedGeometry",
            "fn IterateAllLayoutOpportunities",
            "while index < self.shelves_.len()",
            "for member in exclusions.iter().take(count).skip(index)",
            "fn MergeExclusionSpaces",
        ] {
            assert!(
                owner.contains(owned_policy),
                "float algorithm exclusion space lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 6,
            "shelf insertion, opportunity iteration, cache rebuild, add, update, and merge must retain float sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/exclusions/exclusion_space.rs"),
        )
        .expect("read exclusion-space compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "float/src/exclusion_space.rs"))
                && facade.contains("pub use layout_algo_float_exclusion_space::*;"),
            "exclusion_space must re-export its implementation from the float algorithm"
        );
        for leaked_policy in [
            "pub struct DerivedGeometry",
            "pub struct ExclusionSpaceInternal",
            "while index <",
            "for member in",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "exclusion-space facade still owns float algorithm policy {leaked_policy}"
            );
        }

        let interval_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/shape_interval.rs")),
        )
        .expect("read float shape-interval owner");
        assert!(
            interval_owner.contains("pub struct ShapeInterval")
                && interval_owner.contains("pub type IntShapeInterval")
                && interval_owner.contains("pub fn Unite")
                && !interval_owner.contains("#[inline(never)]"),
            "constant-time shape intervals must stay inlineable in layout_algo_float"
        );
        let interval_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/shapes/shape_interval.rs"))
                .expect("read shape-interval compatibility facade");
        assert!(
            interval_facade.contains(concat!("layout_algo_", "float/src/shape_interval.rs"))
                && interval_facade.contains("pub use layout_algo_float_shape_interval::*")
                && !interval_facade.contains("pub struct ShapeInterval"),
            "shape intervals must be re-exported from layout_algo_float"
        );

        let shape_owner =
            std::fs::read_to_string(root.join(concat!("layout_algo_", "float/src/shape.rs")))
                .expect("read float shape interface owner");
        assert!(
            shape_owner.contains("pub struct LineSegment")
                && shape_owner.contains("pub struct ShapeBase")
                && shape_owner.contains("pub trait Shape")
                && shape_owner.contains("fn GetExcludedInterval")
                && !shape_owner.contains("#[inline(never)]"),
            "the constant-time shape interface must stay inlineable in layout_algo_float"
        );
        let shape_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/shapes/shape.rs"))
                .expect("read shape compatibility facade");
        assert!(
            shape_facade.contains(concat!("layout_algo_", "float/src/shape.rs"))
                && shape_facade.contains("pub use layout_algo_float_shape::*")
                && !shape_facade.contains("pub trait Shape"),
            "the shape interface must be re-exported from layout_algo_float"
        );

        let shapes_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/shape_exclusions.rs")),
        )
        .expect("read float shape-exclusions owner");
        assert!(
            shapes_owner.contains("pub struct ShapeExclusions")
                && shapes_owner.contains("#[inline(never)]\n    fn clone")
                && shapes_owner.contains("Member::<ExclusionArea>::from_ptr(area.Get())"),
            "shape exclusion cloning must retain a float-owned sampling frame"
        );
        let shapes_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/exclusions/shape_exclusions.rs"),
        )
        .expect("read shape-exclusions compatibility facade");
        assert!(
            shapes_facade.contains(concat!("layout_algo_", "float/src/shape_exclusions.rs"))
                && shapes_facade.contains("pub use layout_algo_float_shape_exclusions::*")
                && !shapes_facade.contains("pub struct ShapeExclusions"),
            "shape exclusions must be re-exported from layout_algo_float"
        );

        for (owner_file, facade_file, owned_type, reexport) in [
            (
                "positioned_float.rs",
                "positioned_float.rs",
                "pub struct PositionedFloat",
                concat!("pub use layout_algo_", "float_positioned_float::*;"),
            ),
            (
                "unpositioned_float.rs",
                "unpositioned_float.rs",
                "pub struct UnpositionedFloat",
                concat!("pub use layout_algo_", "float_unpositioned_float::*;"),
            ),
        ] {
            let state_owner = std::fs::read_to_string(
                root.join(concat!("layout_algo_", "float/src"))
                    .join(owner_file),
            )
            .unwrap_or_else(|_| panic!("read float owner {owner_file}"));
            assert!(
                state_owner.contains(owned_type),
                "float algorithm owner {owner_file} lost {owned_type}"
            );
            let state_facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read float facade {facade_file}"));
            assert!(
                state_facade.contains(&format!("float/src/{owner_file}"))
                    && state_facade.contains(reexport)
                    && !state_facade.contains(owned_type),
                "float state {owned_type} must be re-exported from {owner_file}"
            );
        }
    }

    #[test]
    fn exclusion_area_resolves_to_the_float_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/exclusion_area.rs")),
        )
        .expect("read source exclusion-area owner");
        for owned_policy in [
            "pub struct ExclusionShapeData",
            "pub struct ExclusionArea",
            "pub fn Create(",
            "pub fn CreateForInitialLetterBox",
            "pub fn CopyWithOffset",
            "MakeGarbageCollected",
            "impl PartialEq for ExclusionArea",
        ] {
            assert!(
                owner.contains(owned_policy),
                "float algorithm exclusion-area owner lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            3,
            "float exclusion allocation, initial-letter allocation, and offset copy need distinct frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/exclusions/exclusion_area.rs"),
        )
        .expect("read exclusion-area compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "float/src/exclusion_area.rs"))
                && facade.contains("pub use layout_algo_float_exclusion_area::*;")
                && !facade.contains("pub struct ExclusionArea"),
            "exclusion_area must be a re-export-only float facade"
        );
    }

    #[test]
    fn layout_opportunity_resolves_to_the_float_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/layout_opportunity.rs")),
        )
        .expect("read source layout-opportunity owner");
        for owned_policy in [
            "pub struct LayoutOpportunity",
            "fn excluded_segment",
            "fn IsBlockDeltaBelowShapes",
            "fn compute_line_left_offset",
            "fn compute_line_right_offset",
            "fn ComputeLineLayoutOpportunity",
            "GetExcludedInterval",
            "line_left_shapes",
            "line_right_shapes",
        ] {
            assert!(
                owner.contains(owned_policy),
                "float algorithm layout opportunity lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            4,
            "shape projection and left/right/below-shape scans must retain float sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/exclusions/layout_opportunity.rs"),
        )
        .expect("read layout-opportunity compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "float/src/layout_opportunity.rs"))
                && facade.contains("pub use layout_algo_float_layout_opportunity::*;"),
            "layout_opportunity must re-export its implementation from the float algorithm"
        );
        for leaked_policy in [
            "pub struct LayoutOpportunity",
            "fn excluded_segment",
            "line_left_shapes",
            "line_right_shapes",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "layout-opportunity facade still owns float policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn shape_outside_info_resolves_to_the_float_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "float/src/shape_outside_info.rs")),
        )
        .expect("read source shape-outside info owner");
        for owned_policy in [
            "pub struct ShapeOutsideInfo",
            "pub type ShapeOutsideInfoMap",
            "fn EnsureInfo",
            "fn ComputedShape",
            "RequireFloatSupport",
            "support.create_shape",
            "WeakHeapHashMap",
            "ShapeMarginLogicalBoundingBox",
        ] {
            assert!(
                owner.contains(owned_policy),
                "float shape-outside info lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            2,
            "shape-info map admission and shape construction must retain separate float sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/shapes/shape_outside_info.rs"),
        )
        .expect("read shape-outside info compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "float/src/shape_outside_info.rs"))
                && facade.contains("pub use layout_algo_float_shape_outside_info::*;"),
            "shape_outside_info must re-export state from the float algorithm"
        );
        for leaked_policy in [
            "pub struct ShapeOutsideInfo",
            "WeakHeapHashMap",
            "support.create_shape",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "shape-outside info facade still owns float policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn svg_metadata_validation_resolves_to_the_svg_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "svg/src/svg_metadata_validation.rs"
        )))
        .expect("read source SVG metadata-validation owner");
        for owned_policy in [
            "fn ValidateSvgElementInput",
            "for command in &shape.path",
            "for point in &path.points",
            "for value in transform.values",
            "PaintPathVerb::kMoveTo",
            "SvgShapeGeometry::kPath",
        ] {
            assert!(
                owner.contains(owned_policy),
                "SVG algorithm metadata validation lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "SVG geometry validation must retain an SVG sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read LayoutNode metadata facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "svg/src/svg_metadata_validation.rs"
            )),
            "LayoutNode metadata must load SVG validation from its algorithm owner"
        );
        let input_owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/element_metadata_input.rs"))
                .expect("read element metadata input owner");
        assert!(
            input_owner.contains("ValidateSvgElementInput(input.kind, data)?"),
            "element input transaction must delegate SVG validation to its algorithm owner"
        );
        for leaked_policy in [
            "for command in &shape.path",
            "for point in &path.points",
            "SVG path must start with move-to",
            "SVG textPath points must be finite source coordinates",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutNode metadata still owns SVG validation policy {leaked_policy}"
            );
        }

        for (owner_file, facade_file, owned_type, reexport) in [
            (
                "svg_character_data.rs",
                "svg_character_data.rs",
                "pub struct SvgCharacterData",
                "pub use layout_algo_svg_svg_character_data::*;",
            ),
            (
                "svg_inline_node_data.rs",
                "svg_inline_node_data.rs",
                "pub struct SvgInlineNodeData",
                "pub use layout_algo_svg_svg_inline_node_data::*;",
            ),
            (
                "svg_layout_info.rs",
                "svg_layout_info.rs",
                "pub struct SVGLayoutInfo",
                "pub use layout_algo_svg_svg_layout_info::*;",
            ),
            (
                "svg_length_adjust_type.rs",
                "svg_length_adjust_type.rs",
                "pub enum SVGLengthAdjustType",
                "pub use layout_algo_svg_svg_length_adjust_type::*;",
            ),
            (
                "svg_text_attributes_request.rs",
                "svg_text_attributes_request.rs",
                "pub struct SvgTextAttributesBuildRequest",
                "pub use layout_algo_svg_svg_text_attributes_request::*;",
            ),
        ] {
            let svg_owner = std::fs::read_to_string(
                root.join(concat!("layout_algo_", "svg/src"))
                    .join(owner_file),
            )
            .unwrap_or_else(|_| panic!("read SVG owner {owner_file}"));
            assert!(
                svg_owner.contains(owned_type),
                "SVG algorithm owner {owner_file} lost {owned_type}"
            );

            let svg_facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read SVG facade {facade_file}"));
            assert!(
                svg_facade.contains(&format!("svg/src/{owner_file}"))
                    && svg_facade.contains(reexport)
                    && !svg_facade.contains(owned_type),
                "SVG type {owned_type} must be re-exported from {owner_file}"
            );
        }

        let iterator = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "svg/src/resolved_text_layout_attributes_iterator.rs"
        )))
        .expect("read resolved SVG text-attribute iterator owner");
        assert!(
            iterator.contains("pub struct ResolvedTextLayoutAttributesIterator")
                && iterator.contains("pub fn AdvanceTo")
                && iterator.contains("#[inline(never)]\n    fn AdvanceToSlow")
                && iterator.contains(".position(|pair| addressable_index <= pair.0)"),
            "SVG attribute scanning must retain a slow-path SVG sampling frame"
        );
        let iterator_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/resolved_text_layout_attributes_iterator.rs"),
        )
        .expect("read resolved SVG text-attribute iterator facade");
        assert!(
            iterator_facade.contains(concat!(
                "layout_algo_",
                "svg/src/resolved_text_layout_attributes_iterator.rs"
            )) && iterator_facade
                .contains("pub use layout_algo_svg_resolved_text_layout_attributes_iterator::*",)
                && !iterator_facade.contains("pub struct ResolvedTextLayoutAttributesIterator"),
            "resolved SVG text attributes must be re-exported from layout_algo_svg"
        );
    }

    #[test]
    fn svg_text_descendant_index_resolves_to_the_svg_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "svg/src/svg_text_descendant_index.rs"
        )))
        .expect("read source SVG text-descendant index owner");
        for owned_policy in [
            "fn AddSvgTextDescendant",
            "fn RemoveSvgTextDescendant",
            "fn SetContainsNonScalingStroke",
            "fn ContainsNonScalingStroke",
            "fn SvgTextDescendantsMap",
            "SvgTextDescendantsMap",
            "GCedHeapHashSet",
            "SetHasSVGTextDescendants",
        ] {
            assert!(
                owner.contains(owned_policy),
                "SVG text-descendant index owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "SVG text descendant add/remove must retain SVG sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView SVG index loader");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "svg/src/svg_text_descendant_index.rs"
            )) && !loader.contains("fn AddSvgTextDescendant")
                && !loader.contains("fn RemoveSvgTextDescendant")
                && !loader.contains("fn SetContainsNonScalingStroke")
                && !loader.contains("fn ContainsNonScalingStroke")
                && !loader.contains("fn SvgTextDescendantsMap"),
            "LayoutView must load SVG text descendant indexing from the SVG algorithm owner"
        );
    }

    #[test]
    fn list_marker_lookup_resolves_to_the_list_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "list/src/list_marker_lookup.rs")),
        )
        .expect("read source list-marker lookup owner");
        for owned_policy in [
            "fn InputListMarker",
            "while !child.is_null()",
            "NodeKind::kListMarker",
            "child_ref.nextSibling()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "list algorithm marker lookup lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "generated marker child scan must retain a list-algorithm sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read LayoutNode metadata facade");
        assert!(
            facade.contains(concat!("layout_algo_", "list/src/list_marker_lookup.rs"))
                && facade.contains("layout_algo_list_marker_lookup::InputListMarker(self)"),
            "LayoutNode metadata must delegate marker lookup to the list algorithm"
        );
        assert!(
            !facade.contains("while !child.is_null()"),
            "LayoutNode metadata still owns a child-list scan"
        );
    }

    #[test]
    fn layout_view_list_index_resolves_to_the_list_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "list/src/layout_view_list_index.rs"
        )))
        .expect("read LayoutView list-index owner");
        for owned_policy in [
            "fn AddLayoutCounter",
            "fn RemoveLayoutCounter",
            "fn HasLayoutCounters",
            "fn AddLayoutListItem",
            "fn RemoveLayoutListItem",
            "fn HasLayoutListItems",
            "layout_counter_count_",
            "layout_list_item_count_",
        ] {
            assert!(
                owner.contains(owned_policy),
                "list algorithm root index lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "O(1) root list counters must not force sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView list-index loader");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "list/src/layout_view_list_index.rs"
            )),
            "LayoutView must load counter/list indexes from the list algorithm owner"
        );
        for leaked_policy in [
            "fn AddLayoutCounter",
            "fn RemoveLayoutCounter",
            "fn HasLayoutCounters",
            "fn AddLayoutListItem",
            "fn RemoveLayoutListItem",
            "fn HasLayoutListItems",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns list index {leaked_policy}"
            );
        }
    }

    #[test]
    fn style_effect_projection_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/style_effect_projection.rs"))
                .expect("read source style-effect projection owner");
        for owned_policy in [
            "fn ApplyStylePaintGrouping",
            "fn ApplyStyleTransforms",
            "for filter in projected_filters",
            "for operation in &transform.operations",
            "SetBackdropFilterOwned",
            "PushMatrix3D",
            "SetTransformOriginX",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source style-effect projection lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "filter and transform projection must retain separate fragment sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source style input");
        assert!(
            facade.contains("layout_fragment/src/style_effect_projection.rs")
                && facade.contains("ApplyStylePaintGrouping, ApplyStyleTransforms"),
            "source style-input boundary must import effect projection from layout_fragment"
        );
        for leaked_policy in [
            "fn ApplyStylePaintGrouping(",
            "fn ApplyStyleTransforms(",
            "for filter in projected_filters",
            "for operation in &transform.operations",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "source style-input facade still owns effect projection policy {leaked_policy}"
            );
        }

        let clip_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/clip_path_operation_projection.rs"),
        )
        .expect("read source clip-path projection owner");
        for owned_policy in [
            "fn ProjectClipPathOperation",
            "fn PathByteStream",
            "for command in &input.commands",
            "BasicShapePolygon",
            "input.commands.iter().zip(&input.percentage_commands)",
            "ShapeClipPathOperation::new",
        ] {
            assert!(
                clip_owner.contains(owned_policy),
                "layout_fragment clip-path projection lost {owned_policy}"
            );
        }
        assert_eq!(
            clip_owner.matches("#[inline(never)]").count(),
            1,
            "one clip-path conversion must retain one fragment sampling frame"
        );

        let clip_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/clip_path_projection.rs"),
        )
        .expect("read source clip-path facade");
        assert!(
            clip_facade.contains("layout_fragment/src/clip_path_operation_projection.rs")
                && clip_facade.contains(
                    "pub(super) use layout_fragment_clip_path_operation_projection::ProjectClipPathOperation;"
                ),
            "clip-path boundary must re-export projection from layout_fragment"
        );
        for leaked_policy in [
            "fn PathByteStream",
            "for command in &input.commands",
            "BasicShapePolygon::default",
        ] {
            assert!(
                !clip_facade.contains(leaked_policy),
                "clip-path facade still owns fragment policy {leaked_policy}"
            );
        }

        let mask_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/mask_image_style_projection.rs"),
        )
        .expect("read source mask-image projection owner");
        for owned_policy in [
            "fn ApplyMaskImages",
            "fn gradient",
            ".stops.iter()",
            ".collect::<Option<Vec<_>>>()",
            "mask_images.iter().zip(values)",
            "CSSLinearGradientValue::FromResolved",
            "AdjustMaskLayers",
        ] {
            assert!(
                mask_owner.contains(owned_policy),
                "layout_fragment mask-image projection lost {owned_policy}"
            );
        }
        assert_eq!(
            mask_owner.matches("#[inline(never)]").count(),
            1,
            "one mask-chain projection must retain one fragment sampling frame"
        );

        let mask_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/mask_image_projection.rs"),
        )
        .expect("read source mask-image facade");
        assert!(
            mask_facade.contains("layout_fragment/src/mask_image_style_projection.rs")
                && mask_facade.contains(
                    "pub(super) use layout_fragment_mask_image_style_projection::ApplyMaskImages;"
                ),
            "mask-image boundary must load projection from layout_fragment"
        );
        for leaked_policy in [
            "fn gradient",
            ".stops.iter()",
            ".collect::<Option<Vec<_>>>()",
            "mask_images.iter().zip(values)",
        ] {
            assert!(
                !mask_facade.contains(leaked_policy),
                "mask-image facade still owns fragment-sized policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn style_font_resolution_resolves_to_the_input_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/style_font_resolution.rs"))
                .expect("read source style-font resolution owner");
        for owned_policy in [
            "fn ResolveStyleFont",
            "for family in &extra.font_families",
            "FontRequest::new_auto",
            "fonts.Resolve(&request)",
            "builder.SetFont",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source style-font resolution lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "font input validation and request projection must retain an input sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source style input");
        let transaction =
            std::fs::read_to_string(root.join("layout_invalidation/src/input_projection.rs"))
                .expect("read source style-input transaction owner");
        assert!(
            facade.contains("style_font_resolution.rs")
                && facade.contains("ResolveStyleFont(builder, extra, fonts)")
                && transaction.contains("if !node.style.style.is_null()"),
            "source style-input must keep its identity fast path and own font request projection"
        );
        for leaked_policy in [
            "for family in &extra.font_families",
            "FontRequest::new_auto(",
            "fonts.Resolve(&request)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "source style-input facade still owns font-resolution policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn paint_only_font_binding_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_reuse/src/paint_only_font_binding.rs"))
                .expect("read paint-only font-binding owner");
        for owned_policy in [
            "#[inline(never)]\npub(super) fn PreservePaintOnlyFontBinding",
            "ComputedStyleBuilder::from_style",
            "builder.SetFont",
            "builder.SetContainerFont",
            "builder.TakeStyle()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse paint-only font binding lost {owned_policy}"
            );
        }

        let builder =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read layout-object tree builder");
        assert!(
            builder.contains("layout_reuse/src/paint_only_font_binding.rs")
                && builder.contains(
                    "use layout_reuse_paint_only_font_binding::PreservePaintOnlyFontBinding;"
                )
                && !builder.contains("fn PreservePaintOnlyFontBinding("),
            "layout-object builder must delegate paint-only font preservation to layout_reuse"
        );
    }

    #[test]
    fn font_resolver_boundary_separates_service_from_reuse_policy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let interface =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_font_resolver.rs"))
                .expect("read source layout font-resolver interface owner");
        for owned_policy in [
            "pub struct FontRequest",
            "fn new_auto",
            "fn WithBaseDescription",
            "pub trait FontResolver",
            "impl FontResolver for font_engine::FontEngine",
            "FontEngine::ResolveFont",
            "FontEngine::FaceIndex",
            "FontEngine::Variations",
        ] {
            assert!(
                interface.contains(owned_policy),
                "shared source font-resolver interface lost {owned_policy}"
            );
        }
        assert!(
            interface.contains("#[inline(never)]\n    fn Resolve"),
            "direct FontEngine resolution must retain a service sampling frame"
        );

        let construction = std::fs::read_to_string(root.join("layout_reuse/src/font_resolver.rs"))
            .expect("read source font-resolver construction owner");
        assert!(
            construction.contains("impl FontResolver for SharedEngineFontResolver")
                && construction.contains("#[inline(never)]\n    fn Resolve"),
            "shared FontEngine resolution must retain a reuse sampling frame"
        );

        let scope =
            std::fs::read_to_string(root.join("layout_invalidation/src/font_resolver_scope.rs"))
                .expect("read source font-resolver scope owner");
        for owned_policy in [
            "CURRENT_FONT_RESOLVER",
            "pub struct FontResolverScope",
            "pub fn new",
            "pub(crate) unsafe fn new_from_raw",
            "pub fn CurrentFontResolver",
        ] {
            assert!(
                scope.contains(owned_policy),
                "source input font-resolver scope lost {owned_policy}"
            );
        }
        assert!(
            !scope.contains("#[inline(never)]"),
            "constant-time font-resolver TLS binding must remain inlineable"
        );

        let input_facade = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source input");
        assert!(
            input_facade.contains("font_resolver_scope.rs")
                && input_facade.contains("CurrentFontResolver, FontResolverScope",),
            "source input must own and re-export its font-resolver scope"
        );
        for leaked_policy in [
            "static CURRENT_FONT_RESOLVER",
            "pub struct FontResolverScope",
            "impl Drop for FontResolverScope",
        ] {
            assert!(
                !input_facade.contains(leaked_policy),
                "source input still owns font-resolver scope policy {leaked_policy}"
            );
        }

        assert!(
            !interface.contains("layout_reuse/src/layout_font_resolver.rs"),
            "shared font service must not be sourced from the reuse package"
        );
    }

    #[test]
    fn layout_object_font_validity_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_reuse/src/layout_object_font_validity.rs"))
                .expect("read LayoutObject font-validity owner");
        for owned_policy in [
            "fn IsFontFallbackValid",
            "StyleRef().GetFont()",
            "FirstLineStyleRef().GetFont()",
            "IsFallbackValid()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse LayoutObject font admission lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "font-fallback admission must retain a reuse sampling frame"
        );

        let loader = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services loader");
        assert!(
            loader.contains("layout_reuse/src/layout_object_font_validity.rs"),
            "LayoutObject core services must load font admission from layout_reuse"
        );
        assert!(
            !loader.contains("fn IsFontFallbackValid"),
            "LayoutObject core services still own font-fallback admission"
        );
    }

    #[test]
    fn css_zoom_slow_path_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_invalidation/src/css_zoom.rs"))
            .expect("read non-unit CSS zoom owner");
        for owned_policy in [
            "fn ZoomedStyleNonUnit",
            "style.clone()",
            "fn TracksScale",
            "for repeat in &mut list.repeaters",
            "Arc::make_mut",
            ".box_shadows",
            ".background_images",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation non-unit CSS zoom lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "variable-size CSS zoom conversion must retain one invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/css_zoom.rs"))
            .expect("read CSS zoom facade");
        assert!(
            facade.contains("layout_invalidation/src/css_zoom.rs")
                && facade.contains("if zoom == 1.0")
                && facade.contains("Cow::Borrowed(style)")
                && facade.contains("ZoomedStyleNonUnit(style, zoom)"),
            "CSS zoom facade must preserve its borrowed fast path and delegate non-unit work"
        );
        for leaked_policy in [
            "style.clone()",
            "for repeat in &mut list.repeaters",
            "Arc::make_mut",
            ".box_shadows",
            ".background_images",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "CSS zoom facade still owns slow-path policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_svg_dispatch_resolves_to_the_svg_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "svg/src/layout_object_svg_dispatch.rs"
        )))
        .expect("read LayoutObject SVG dispatch owner");
        for owned_policy in [
            "fn UpdateSVGLayout",
            "svg_support.update_object",
            "fn ObjectBoundingBox",
            "fn StrokeBoundingBox",
            "fn DecoratedBoundingBox",
            "fn VisualRectInLocalSVGCoordinates",
            "fn LocalSVGTransform",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_algo_svg LayoutObject dispatch lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "only SVG provider dispatch should retain an algorithm sampling frame"
        );

        let loader = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services loader");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "svg/src/layout_object_svg_dispatch.rs"
            )),
            "LayoutObject core services must load SVG dispatch from layout_algo_svg"
        );
        for leaked_policy in [
            "fn UpdateSVGLayout",
            "fn ObjectBoundingBox",
            "fn StrokeBoundingBox",
            "fn DecoratedBoundingBox",
            "fn VisualRectInLocalSVGCoordinates",
            "fn LocalSVGTransform",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core services still own SVG policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn trigger_scope_projection_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/trigger_scope_projection.rs"),
        )
        .expect("read source trigger-scope projection owner");
        for owned_policy in [
            "fn ApplyNonDefaultTriggerScope",
            "fn TriggerScopeNames",
            "for value in &extra.trigger_scope_names",
            "for name in names",
            "SetTriggerScopeOwned",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation trigger-scope projection lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "non-default trigger-scope construction must retain an invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source style input");
        assert!(
            facade.contains("trigger_scope_projection.rs")
                && facade.contains(
                    "if !extra.trigger_scope_all && extra.trigger_scope_names.is_empty()"
                )
                && facade.contains("ApplyNonDefaultTriggerScope(builder, extra)"),
            "source style-input must keep the default trigger fast path and delegate non-default construction"
        );
        for leaked_policy in [
            "fn TriggerScopeNames(",
            "for value in &extra.trigger_scope_names",
            "for name in names",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "source style-input facade still owns trigger-scope policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn trigger_scoped_name_state_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/trigger_scoped_name.rs"))
                .expect("read source trigger scoped-name owner");
        for owned_policy in [
            "pub type TriggerScopedName = NamingScope",
            "pub fn ToTriggerScopedName",
            "pub type TriggerScopedNameMap",
            "TriggerScopedName::FindScopeElement",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment trigger scoped-name state lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "trigger scoped-name wrapper must leave ancestor sampling to NamingScope"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/trigger_scoped_name.rs"))
                .expect("read trigger scoped-name facade");
        assert!(
            facade.contains("layout_fragment/src/trigger_scoped_name.rs")
                && facade.contains("pub use layout_fragment_trigger_scoped_name::*")
                && !facade.contains("pub type TriggerScopedName = NamingScope"),
            "trigger scoped-name state must be re-exported from layout_fragment"
        );
    }

    #[test]
    fn naming_scope_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_invalidation/src/naming_scope.rs"))
            .expect("read source naming-scope owner");
        for owned_policy in [
            "pub struct NamingScope",
            "fn FindScopeElement",
            "while !element.is_null()",
            "fn IsWithinScope",
            "for scoped_name in",
            "pub struct NamingScopeKey",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation naming scope lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "scoped-name ancestry and name-list lookup must retain one invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/naming_scope.rs"))
            .expect("read naming-scope compatibility facade");
        assert!(
            facade.contains("layout_invalidation/src/naming_scope.rs")
                && facade.contains("pub use layout_invalidation_naming_scope::*;"),
            "naming_scope must re-export its implementation from layout_invalidation"
        );
        for leaked_policy in [
            "pub struct NamingScope",
            "fn FindScopeElement",
            "while !element.is_null()",
            "for scoped_name in",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "naming-scope facade still owns invalidation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn input_projection_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/input_projection.rs"))
                .expect("read source input-projection owner");
        for owned_policy in [
            "fn PrepareStyle",
            "fn PrepareStyleProjection",
            "fn PrepareStyleDefault",
            "fn PrepareConstraints",
            "fn PrepareConstraintsDefault",
            "if !node.style.style.is_null()",
            "BeginStyle",
            "ApplyStyleListAndPosition",
            "ApplyStyleShape",
            "ApplyStyleGridTableAndBreaks",
            "ApplyStylePaintGrouping",
            "ApplyMaskImages",
            "ApplyTriggerScope",
            "ConstraintSpaceBuilder::new_without_parent_space",
            "SetContainsAnnotations",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source input projection lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            2,
            "slow style reconstruction and root-constraint projection must retain separate invalidation frames"
        );

        let implementation = std::fs::read_to_string(root.join("layout_invalidation/src/input.rs"))
            .expect("read invalidation-owned source input implementation");
        assert!(
            implementation.contains("input_projection.rs")
                && implementation
                    .contains("PrepareConstraints, PrepareConstraintsDefault, PrepareStyle,")
                && implementation.contains("fn BeginStyle")
                && implementation.contains("fn FinishStyle"),
            "source input conversion and projection entries must remain in layout_invalidation"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/boundary/input.rs"))
            .expect("read source input compatibility facade");
        assert!(
            facade.contains("layout_invalidation/src/input.rs")
                && facade.contains("pub use layout_invalidation_input::*;")
                && !facade.contains("fn BeginStyle")
                && !facade.contains("pub fn PrepareStyle("),
            "core input must remain a re-export-only invalidation facade"
        );
    }

    #[test]
    fn input_equivalence_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let style_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/computed_style_equivalence.rs"),
        )
        .expect("read computed-style equivalence owner");
        for owned_policy in [
            "impl PartialEq for ComputedStyle",
            "#[inline(always)]\n    fn eq",
            "#[inline(never)]\nfn computed_style_eq",
            "#[inline(always)]\n    pub fn LayoutEquivalent",
            "#[inline(never)]\nfn layout_equivalent",
            "fn geometry_inputs_equal",
            "ComputedStyleBase::FieldInvalidationDiff",
            "std::sync::Arc::ptr_eq",
            "grid_template_columns",
        ] {
            assert!(
                style_owner.contains(owned_policy),
                "layout_invalidation computed-style equivalence lost {owned_policy}"
            );
        }

        let element_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/element_data_equivalence.rs"),
        )
        .expect("read element-data equivalence owner");
        for owned_policy in [
            "impl PartialEq for ElementData",
            "#[inline(always)]\n    fn eq",
            "#[inline(never)]\nfn element_data_eq",
            "element.frame_rows == other.frame_rows",
            "element.svg_x == other.svg_x",
            "element.svg_rotate == other.svg_rotate",
            "element.custom_layout, &other.custom_layout",
        ] {
            assert!(
                element_owner.contains(owned_policy),
                "layout_invalidation element-data equivalence lost {owned_policy}"
            );
        }

        let core = std::fs::read_to_string(root.join("layoutng/src/internal/layout_input.rs"))
            .expect("read shared layout-input schema");
        assert!(
            core.contains("layout_invalidation/src/computed_style_equivalence.rs")
                && core.contains("layout_invalidation/src/element_data_equivalence.rs"),
            "layout_input must load both invalidation-owned equivalence implementations"
        );
        for leaked_policy in [
            "pub fn LayoutEquivalent",
            "fn geometry_inputs_equal",
            "impl PartialEq for ElementData",
        ] {
            assert!(
                !core.contains(leaked_policy),
                "layout_input schema still owns variable-size equivalence policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn managed_input_admission_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/managed_input_admission.rs"),
        )
        .expect("read managed-input admission owner");
        for owned_policy in [
            "fn GeometryMetadataEqual",
            "fn NonStyleInputsEqual",
            "fn RetainInputStyles",
            "#[inline(always)]\n",
            "pub(super) fn ControlPaintStateChanged",
            "let mut normalized = new.clone()",
            "for style in [",
            "roots.iter().any",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation managed-input admission lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            3,
            "metadata normalization, descriptor comparison, and style retention need separate invalidation frames"
        );

        let builder =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read layout-object tree builder");
        assert!(
            builder.contains("layout_invalidation/src/managed_input_admission.rs")
                && [
                    "ControlPaintStateChanged",
                    "GeometryMetadataEqual",
                    "ImagePaintStateChanged",
                    "LayoutImageNaturalSizeChangeIsPaintOnly",
                    "LayoutImageNaturalSizeChanged",
                    "NonStyleInputsEqual",
                    "RetainInputStyles",
                ]
                .into_iter()
                .all(|name| builder.contains(name)),
            "layout-object builder must import managed-input admission from invalidation"
        );
        for leaked_policy in [
            "fn GeometryMetadataEqual(",
            "fn NonStyleInputsEqual(",
            "fn RetainInputStyles(",
        ] {
            assert!(
                !builder.contains(leaked_policy),
                "layout-object builder still owns invalidation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn managed_object_update_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/managed_object_update.rs"))
                .expect("read managed-object update owner");
        for owned_policy in [
            "#[inline(always)]\n    #[allow(clippy::too_many_arguments)]\n    pub(super) fn Create",
            "#[inline(never)]\n#[allow(clippy::too_many_arguments)]\nfn create_managed_object",
            "let same_non_style_inputs",
            "let paint_only",
            "PrepareStyle(",
            "PreservePaintOnlyFontBinding",
            "CreateElementMetadata",
            "CreateLayoutObject(",
            "RetainInputStyles",
            "storage.managed.insert",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation managed-object update lost {owned_policy}"
            );
        }

        let core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read layout-object tree builder");
        assert!(
            core.contains("layout_invalidation/src/managed_object_update.rs"),
            "layout-object builder must load the invalidation-owned managed-object transaction"
        );
        for leaked_policy in [
            "fn Create(\n",
            "let same_non_style_inputs",
            "let paint_only",
            "let mut prepare_style",
            "storage.managed.insert(key, record)",
        ] {
            assert!(
                !core.contains(leaked_policy),
                "layout-object builder still owns managed-object update policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn node_input_state_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/node_input_state.rs"))
                .expect("read retained node-input state owner");
        for owned_policy in [
            "#[inline(never)]\npub(super) fn NewStandaloneNodeData",
            "#[inline(never)]\npub(super) fn UpdateNodeInputData",
            "input_first_line_style_data: input.first_line_style.clone()",
            "element_data: input.element.clone()",
            "node.input_data_.element_data = input.element.clone()",
            "node.input_data_.debug_name = input.debug_name.clone()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation retained node-input state lost {owned_policy}"
            );
        }

        let core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read shared node-metadata schema");
        assert!(
            core.contains("layout_invalidation/src/node_input_state.rs")
                && core
                    .contains("layout_invalidation_node_input_state::NewStandaloneNodeData(input)")
                && core.contains(
                    "layout_invalidation_node_input_state::UpdateNodeInputData(self, input)"
                ),
            "node metadata must delegate retained-input construction and replacement to invalidation"
        );
        for leaked_policy in [
            "element_data: input.element.clone()",
            "self.input_data_.element_data = input.element.clone()",
            "self.input_data_.debug_name = input.debug_name.clone()",
        ] {
            assert!(
                !core.contains(leaked_policy),
                "node-metadata schema still owns retained-input copy policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn timeline_trigger_metadata_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/timeline_trigger_metadata.rs"),
        )
        .expect("read source timeline-trigger metadata owner");
        for owned_policy in [
            "fn PopulateTimelineTriggerNames",
            "for value in &style.timeline_trigger_names",
            "name.encode_utf16()",
            "ScopedCSSName::new",
            "timeline_trigger_names_",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation timeline-trigger metadata lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "nonempty timeline-trigger construction must retain an invalidation sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read LayoutNode metadata facade");
        assert!(
            facade.contains("layout_invalidation/src/timeline_trigger_metadata.rs"),
            "LayoutNode metadata must load timeline-trigger construction"
        );
        let input_owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/element_metadata_input.rs"))
                .expect("read element metadata input owner");
        assert!(
            input_owner.contains("!style.timeline_trigger_names.is_empty()")
                && input_owner.contains("PopulateTimelineTriggerNames(&mut element, input)?"),
            "element input transaction must keep the empty trigger fast path and delegate construction"
        );
        for leaked_policy in [
            "for value in &style.timeline_trigger_names",
            "Timeline trigger requires a nonempty UTF-8 name",
            "ScopedCSSName::new(&atom",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutNode metadata still owns timeline-trigger policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn element_metadata_input_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/element_metadata_input.rs"))
                .expect("read element metadata input owner");
        for owned_policy in [
            "pub fn new(",
            "fn UpdateElementInput",
            "fn validate_element_input_prefix",
            "fn prepare_element_dimensions",
            "ValidateSvgElementInput",
            "PopulateTimelineTriggerNames",
            "text_area_sizing",
            "autofill_state",
            "used_canvas_transform",
            "replaced_respects_css_overflow",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation element metadata input lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            2,
            "element construction and incremental refresh must retain separate invalidation frames"
        );

        let loader =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read node metadata compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/element_metadata_input.rs"),
            "node metadata must load element input policy from layout_invalidation"
        );
        for leaked_policy in [
            "fn validate_element_input_prefix",
            "fn prepare_element_dimensions",
            "pub(crate) fn UpdateElementInput",
            "replaced_respects_css_overflow;",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "node metadata core still owns element input policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn metadata_append_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/metadata_append.rs"))
                .expect("read source metadata append owner");
        for owned_policy in [
            "fn AppendChild",
            "fn DetachForTreeUpdate",
            "fn InsertForTreeUpdate",
            "while !ancestor.is_null()",
            "AppendChildError::Cycle",
            "child.parent_ = Member::from_ptr",
            "parent.last_child_ = Member::from_ptr",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation metadata append lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "metadata append, detach, and insertion must retain invalidation sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read LayoutNode metadata facade");
        assert!(
            facade.contains("layout_invalidation/src/metadata_append.rs")
                && facade.contains("layout_invalidation_metadata_append::AppendChild(self, child)")
                && facade
                    .contains("layout_invalidation_metadata_append::DetachForTreeUpdate(self)")
                && facade.contains(
                    "layout_invalidation_metadata_append::InsertForTreeUpdate(self, child, before)"
                ),
            "LayoutNode metadata must delegate link mutation to layout_invalidation"
        );
        for leaked_policy in [
            "while !ancestor.is_null()",
            "child.parent_ = Member::from_ptr(self)",
            "(*self.last_child_.Get()).next_",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutNode metadata still owns append policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn node_metadata_finalize_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/node_metadata_finalize.rs"))
                .expect("read source metadata-finalization owner");
        for owned_policy in [
            "fn FinalizeTreeRelations",
            "while let Some(node) = pending.pop()",
            "for &candidate_node in &nodes",
            "DuplicateViewportDefiningElement",
            "ImplicitAnchorIdNotUnique",
            "SetNeedsLayoutAndIntrinsicWidthsRecalc",
            "seal_input_tree",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation metadata finalization lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "whole-tree metadata finalization must retain an invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/node_metadata_input.rs"),
        )
        .expect("read node-metadata input facade");
        assert!(
            facade.contains("layout_invalidation/src/node_metadata_finalize.rs")
                && facade.contains(
                    "layout_invalidation_node_metadata_finalize::FinalizeTreeRelations(root)"
                ),
            "node-metadata input must delegate whole-tree finalization to layout_invalidation"
        );
        for leaked_policy in [
            "while let Some(node) = pending.pop()",
            "for &candidate_node in &nodes",
            "SetNeedsLayoutAndIntrinsicWidthsRecalc(",
            "seal_input_tree()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "node-metadata input facade still owns finalization policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn node_metadata_incremental_relations_resolve_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/node_metadata_incremental_relations.rs"),
        )
        .expect("read source incremental metadata-relations owner");
        for owned_policy in [
            "fn AttachIncrementalRelations",
            "while distance != 0 && !host.is_null()",
            "FormControlType::kInputFile",
            "FormControlType::kInputRange",
            "upload_button_ = Member::from_ptr",
            "spin_button_ = Member::from_ptr",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation incremental metadata relations lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "incremental metadata relation admission must retain an invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/node_metadata_input.rs"),
        )
        .expect("read node-metadata input facade");
        assert!(
            facade.contains(
                "layout_invalidation/src/node_metadata_incremental_relations.rs"
            ) && facade.contains(
                "layout_invalidation_node_metadata_incremental_relations::AttachIncrementalRelations("
            ),
            "node-metadata input must delegate incremental relations to layout_invalidation"
        );
        for leaked_policy in [
            "while distance != 0 && !host.is_null()",
            "File upload button requires input-button identity",
            "Slider thumb requires an input-range control owner",
            "Inner editor requires one actual text-control host",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "node-metadata input facade still owns incremental relation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn node_metadata_tree_query_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/node_metadata_tree.rs"))
                .expect("read source node-metadata tree owner");
        for owned_policy in [
            "#[inline(never)]",
            "fn parentElement",
            "while !parent.is_null()",
            "node.IsElementNode()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "node-metadata tree owner lost {owned_policy}"
            );
        }

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_metadata.rs"))
                .expect("read node-metadata core record");
        assert!(
            facade.contains("layout_invalidation/src/node_metadata_tree.rs")
                && !facade.contains("fn parentElement"),
            "node parent-element traversal must load from layout_invalidation"
        );
    }

    #[test]
    fn client_traversals_resolve_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_commit/src/client_commit.rs"))
            .expect("read source client commit owner");
        assert!(
            owner.contains("#[inline(never)]")
                && owner.contains("pub fn commit_paint_client_state")
                && owner.contains("while !object.is_null()")
                && owner.contains("unsafe fn refresh_clients"),
            "source client traversals must stay out of line in layout_commit"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/paint_client_commit.rs"))
                .expect("read source client commit adapter");
        assert!(
            adapter.contains("crate::layout_commit_package::commit_paint_client_state"),
            "source paint-client entry must delegate to layout_commit"
        );
        for leaked_policy in [
            "while !object.is_null()",
            "unsafe fn refresh_clients",
            "Arc::make_mut",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "paint_client_commit adapter still owns traversal policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn engine_paint_commit_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_commit/src/engine_paint_commit.rs"))
            .expect("read source engine paint-commit owner");
        for owned_policy in [
            "struct PaintCommit",
            "impl layout_commit::PaintCommitBackend",
            "fn ValidateAuxiliaryClients",
            "fn CommitClients",
            "fn CommitPrePaintInputs",
            "fn PublishLayout",
            "CommitPaintClientState",
            "CommitClients",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_commit source engine backend lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "each source Paint commit stage must retain a commit sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_commit/src/engine_paint_commit.rs")
                && adapter.contains("use layout_commit_engine_paint_commit::PaintCommit"),
            "LayoutEngine must import its concrete Paint backend from layout_commit"
        );
        for leaked_policy in [
            "impl layout_commit::PaintCommitBackend for PaintCommit",
            "caret.ValidateForCommittedPaint",
            "tree.CommitPaintClientState",
            ".CommitClients(commit.UsedClients()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns Paint commit policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn engine_layout_publish_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_commit/src/engine_layout_publish.rs"))
                .expect("read source engine layout-publication owner");
        for owned_policy in [
            "struct LayoutOutputPreparation",
            "fn begin_layout_output",
            "fn run_layout_without_publication",
            "fn publish_layout_output",
            "scroll_paint_index_spare_",
            "pending_subtree_layout_id_ = None",
            "std::mem::take(&mut engine.pre_paint_inputs_)",
            "TracePhase::RunLayoutWithoutPublication",
            "engine.pre_paint_inputs_.invalidate()",
            "UpdatePrePaintInputs",
            "TracePhase::CertifyPaintInputs",
            "TracePhase::BuildScrollIndex",
            "ScrollPaintPropertyIndex::new",
            "engine.fragments = Some(fragments)",
            "engine.pre_paint_inputs_ = inputs",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_commit engine layout publication lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "layout transaction begin, commit-only run and successful publication must retain commit sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_commit/src/engine_layout_publish.rs")
                && adapter.contains(
                    "layout_commit_engine_layout_publish::begin_layout_output(self, purpose)"
                )
                && adapter.contains("layout_commit_engine_layout_publish::publish_layout_output")
                && adapter.contains(
                    "layout_commit_engine_layout_publish::run_layout_without_publication(self)"
                ),
            "LayoutEngine must delegate successful result publication to layout_commit"
        );
        for leaked_policy in [
            "TracePhase::CertifyPaintInputs",
            "TracePhase::BuildScrollIndex",
            "self.pending_subtree_layout_id_ = None",
            "std::mem::take(&mut self.pre_paint_inputs_)",
            "LayoutEngine.RunLayoutWithoutPublication",
            "self.fragments = Some(fragments);\n            self.pre_paint_inputs_ = inputs",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns successful layout publication {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_environment_is_shared_pass_orchestration() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/layout_environment.rs"),
        )
        .expect("read source layout environment owner");
        for owned_policy in [
            "struct InputTextAndImageScopes",
            "struct LayoutEnvironment",
            "struct LayoutEnvironmentStorage",
            "FontBinding::new",
            "InputHyphenationResolver::new",
            "HyphenationResolverScope::new_from_raw",
            "InputPhraseBreakResolver::new",
            "PhraseBreakResolverScope::new_from_raw",
            "InputTextTransformResolver::new",
            "TextTransformResolverScope::new_from_raw",
            "InputShapeImageResolver::new",
            "ShapeImageResolverScope::new_from_raw",
            "impl LayoutBoundaryEnvironment for LayoutEnvironment",
            "fn ReusesPreparedFonts",
            "fn Commit",
        ] {
            assert!(
                owner.contains(owned_policy),
                "source layout environment lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "layout environment creation and teardown must retain shared boundary frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source boundary adapter");
        assert!(
            adapter.contains("layout_environment.rs")
                && adapter.contains(
                    "pub(crate) use layout_boundary_layout_environment::LayoutEnvironment"
                ),
            "source boundary must load and re-export shared pass orchestration"
        );
        for leaked_policy in [
            "struct InputTextAndImageScopes",
            "struct LayoutEnvironmentStorage",
            "impl LayoutBoundaryEnvironment for LayoutEnvironment",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "source boundary still owns layout environment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_result_commit_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_commit/src/layout_box_result_commit.rs"))
                .expect("read source per-box layout-result commit owner");
        for owned_policy in [
            "fn check_did_add_fragment",
            "fn LayoutBoxSetLayoutResult",
            "fn LayoutBoxAppendLayoutResult",
            "fn LayoutBoxReplaceLayoutResult",
            "fn LayoutBoxFinalizeLayoutResults",
            "fn LayoutBoxRebuildFragmentTreeSpine",
            "fn LayoutBoxShrinkLayoutResults",
            "IsCausedByColumnSpanner",
            "FragmentItems::FinalizeAfterLayout",
            "CloneWithPostLayoutFragments",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_commit per-box result publication lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 7,
            "per-box result publication stages must retain commit sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_commit/src/layout_box_result_commit.rs")
                && facade.contains("LayoutBoxSetLayoutResult(self, result, index)")
                && facade.contains("LayoutBoxRebuildFragmentTreeSpine(self)"),
            "LayoutBox must delegate per-box result publication to layout_commit"
        );
        for leaked_policy in [
            "FragmentItems::FinalizeAfterLayout",
            "CloneWithPostLayoutFragments",
            "IsCausedByColumnSpanner()",
            "fn CheckDidAddFragment",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns result publication policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_post_layout_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_commit/src/layout_box_post_layout_commit.rs"),
        )
        .expect("read LayoutBox post-layout commit owner");
        for owned_policy in [
            "#[inline(never)]",
            "fn SizeChanged",
            "fn UpdateAfterLayout",
            "SetScrollableAreaSizeChanged(true)",
            "SetScrollableOverflowFromLayoutResults",
            "UpdateTransform",
            "UpdateScrollingAfterLayout",
            "ClearNeedsLayout",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutBox post-layout commit owner lost {owned_policy}"
            );
        }

        let loader =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_layout.rs"))
                .expect("read LayoutBox layout compatibility module");
        assert!(
            loader.contains("layout_commit/src/layout_box_post_layout_commit.rs")
                && loader.contains("fn OverrideIntrinsicContentInlineSize")
                && !loader.contains("fn UpdateAfterLayout"),
            "post-layout publication must load from layout_commit without moving intrinsic-size queries"
        );
    }

    #[test]
    fn layout_view_post_layout_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_commit/src/layout_view_commit.rs"))
            .expect("read LayoutView commit owner");
        for owned_policy in [
            "fn UpdateAfterLayout",
            "fn CommitPendingSelection",
            "fn CacheScrollDimensions",
            "LayoutViewCommitPendingSelectionProvider",
            "LayoutViewCacheScrollDimensionsProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutView commit owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "post-layout, selection, and scroll-dimension commits must retain separate sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains("layout_commit/src/layout_view_commit.rs"),
            "LayoutView must load post-layout publication from layout_commit"
        );
        for leaked_policy in [
            "fn UpdateAfterLayout",
            "fn CommitPendingSelection",
            "fn CacheScrollDimensions",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns commit policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn container_query_sizes_resolve_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_commit/src/container_query_sizes.rs"))
                .expect("read source container-query size owner");
        for owned_policy in [
            "fn collect_container_query_sizes",
            "LayoutObjectFactoryScope::new",
            "object.NextInPreOrder(root)",
            "ContainerType() & 3",
            "PhysicalContentBoxRect",
            "EffectiveZoom",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_commit container-query publication lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "container-query publication must retain a commit sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_commit/src/container_query_sizes.rs")
                && adapter.contains(
                    "layout_commit_container_query_sizes::collect_container_query_sizes(self)"
                ),
            "LayoutEngine must delegate container-query publication to layout_commit"
        );
        for leaked_policy in [
            "while let Some(object) = unsafe { current.as_ref() }",
            "PhysicalContentBoxRect().size",
            "object.NextInPreOrder(root)",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns container-query publication {leaked_policy}"
            );
        }
    }

    #[test]
    fn output_node_lookup_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_commit/src/output_node_lookup.rs"))
            .expect("read source output-node lookup owner");
        for owned_policy in [
            "fn output_node_ptr",
            ".InputKeys(id)",
            ".filter_map(|key| storage.managed.get(key))",
            ".filter(|record| !record.input.style_generated)",
            ".min_by_key(|record| record.input.kind as i32)",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_commit output-node lookup lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "output candidate selection needs one commit sampling frame"
        );

        let core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObjectTree adapter");
        assert!(
            core.contains("layout_commit/src/output_node_lookup.rs")
                && core.contains("layout_commit_output_node_lookup::output_node(self, id)")
                && core.contains("layout_commit_output_node_lookup::output_node_mut(self, id)"),
            "LayoutObjectTree must delegate published object lookup to layout_commit"
        );
        for leaked_policy in [
            ".filter_map(|key| storage.managed.get(key))",
            ".min_by_key(|record| record.input.kind as i32)",
        ] {
            assert!(
                !core.contains(leaked_policy),
                "LayoutObjectTree still owns output lookup policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn scroll_offset_snapshot_resolves_to_the_commit_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_commit/src/scroll_offset_snapshot.rs"))
                .expect("read source scroll-offset snapshot owner");
        for owned_policy in [
            "fn output_scroll_offsets",
            "storage",
            ".managed",
            ".values()",
            "style_generated",
            "InputElementData",
            "data.scroll_offset",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_commit scroll-offset publication lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "scroll-offset snapshot scan must retain a commit sampling frame"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObjectTree adapter");
        assert!(
            adapter.contains("layout_commit/src/scroll_offset_snapshot.rs")
                && adapter
                    .contains("layout_commit_scroll_offset_snapshot::output_scroll_offsets(self)"),
            "LayoutObjectTree must delegate scroll-offset publication to layout_commit"
        );
        for leaked_policy in [
            "Some((record.input.id, data.scroll_offset))",
            ".filter_map(|record| {\n                if record.input.style_generated",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutObjectTree still owns scroll-offset publication {leaked_policy}"
            );
        }
    }

    #[test]
    fn object_index_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_boundary_object_index.rs"),
        )
        .expect("read source fragment object index owner");
        for owned_policy in [
            "pub(crate) fn ensure_layout_boundary_object_index",
            "cache.objects.push(object)",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source object index lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "fragment indexing must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment_object_index::ensure_layout_boundary_object_index"),
            "source object indexing must delegate through the fragment-owned adapter"
        );
        for leaked_policy in [
            "cache.objects.push(object)",
            "cache.source_objects = Arc::new(source_objects)",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary adapter still owns index policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn sticky_offset_loop_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/paint_layer_scrollable_area_sticky_offsets.rs"),
        )
        .expect("read source fragment sticky-offset owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(crate) fn update_sticky_offsets",
            "for object in objects",
            "while !ancestor.is_null()",
            "ComputeStickyPositionConstraintsForLayout",
            "ComputeStickyOffset",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source sticky-offset loop lost {owned_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment_sticky_offsets::update_sticky_offsets"),
            "fragment boundary must delegate source sticky refresh to layout_fragment"
        );
        for leaked_policy in [
            "let constrained_axes = LayoutBoxModelObject::StickyConstrainedAxes",
            "ComputeStickyPositionConstraintsForLayout(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns sticky-offset policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn scroll_refresh_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/scroll_paint_property_refresh.rs"),
        )
        .expect("read source fragment scroll-refresh owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn refresh_scroll_paint_properties",
            "TracePhase::RefreshScrollPaintProperties",
            "while !object.is_null()",
            "for child in children",
            "SetScrollbarThumb(",
            "MarkScrollPropertyChange",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source scroll refresh lost {owned_policy}"
            );
        }
        for reconstructed_state in [
            "scroll.scroll_origin = Offset::default()",
            "scroll.scroll_position = scroll_offset",
        ] {
            assert!(
                !owner.contains(reconstructed_state),
                "scroll refresh must consume the retained ScrollableArea, not reconstruct {reconstructed_state}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains(
                "layout_fragment_scroll_paint_property_refresh::refresh_scroll_paint_properties"
            ),
            "fragment boundary must delegate source scroll refresh to layout_fragment"
        );
        for leaked_policy in [
            "old_scroll = fragment.paint.scroll_offset",
            "old_sticky_constraint = fragment.paint.sticky_constraint",
            "RefreshFragmentScrollPaintProperties",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns scroll-refresh policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn scroll_index_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/scroll_paint_property_index.rs"),
        )
        .expect("read source fragment scroll-index owner");
        for owned_policy in [
            "pub(crate) struct ScrollPaintPropertyIndex",
            "pub(crate) fn new",
            "pub(crate) fn refresh",
            "TracePhase::RefreshScrollPaintIndex",
            "for fragment in fragment.Children()",
            "for id in changed_ids",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source scroll index lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "scroll-index rebuild and refresh must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/scroll_paint_property_index.rs")
                && adapter.contains(
                    "layout_fragment_scroll_paint_property_index::ScrollPaintPropertyIndex"
                ),
            "fragment boundary must import its source scroll index from layout_fragment"
        );
        assert!(
            !root
                .join("layoutng/src/internal/boundary/scroll_paint_property_index.rs")
                .exists(),
            "the scroll paint index must not keep a duplicate core implementation"
        );
    }

    #[test]
    fn paint_style_projection_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_fragment/src/paint_style_export.rs"))
            .expect("read fragment source paint-style owner");
        for owned_entry in [
            "pub(crate) fn ExportGeometryStyle",
            "pub(crate) fn ExportStyle",
            "pub(crate) fn ExportStyleWithProjectedSvg",
            "pub(crate) fn SetResolvedOffsetTransform",
            "clip_path_paint_export.rs",
        ] {
            assert!(
                owner.contains(owned_entry),
                "layout_fragment paint-style projection lost {owned_entry}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "fragment paint-style projection entries must remain visible to sampling profilers"
        );
        assert!(
            root.join("layout_fragment/src/clip_path_paint_export.rs")
                .is_file(),
            "clip-path projection must be physically owned by layout_fragment"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/paint_style_export.rs"),
        )
        .expect("read source paint-style facade");
        assert!(
            facade.contains("layout_fragment/src/paint_style_export.rs")
                && facade.contains("pub use layout_fragment_paint_style_export::*"),
            "core paint-style module must be a fragment-owner facade"
        );
        for leaked_policy in ["fn Backgrounds(", "fn Masks(", "fn Transform("] {
            assert!(
                !facade.contains(leaked_policy),
                "core paint-style facade still owns projection policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn paint_input_update_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let schema = std::fs::read_to_string(root.join("layout_fragment/src/paint_input.rs"))
            .expect("read fragment-owned paint-input schema");
        for owned_type in [
            "pub struct ClipPathPaint",
            "pub struct PaintShader",
            "pub struct PaintTransform",
            "pub struct PaintStyleData",
        ] {
            assert!(
                schema.contains(owned_type),
                "fragment paint-input schema lost {owned_type}"
            );
        }
        let schema_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/paint_input.rs"))
                .expect("read paint-input compatibility facade");
        assert!(
            schema_facade.contains("layout_fragment/src/paint_input.rs")
                && schema_facade.contains("pub use layout_fragment_paint_input::*;")
                && !schema_facade.contains("pub struct PaintStyleData"),
            "paint-input schema must be re-exported from layout_fragment"
        );

        let owner = std::fs::read_to_string(root.join("layout_fragment/src/paint_input_update.rs"))
            .expect("read source fragment paint-input owner");
        for owned_policy in [
            "pub(super) fn update_paint_style",
            "pub(super) fn paint_input_plan",
            "pub(super) fn update_paint_input",
            "TracePhase::ValidatePaintInput",
            "ComputedStyleBase::FieldInvalidationDiff",
            "SetShouldDoFullPaintInvalidationWithoutLayoutChange",
            "node.UpdateInputData",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source paint-input update lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "paint-style, paint-input validation and mutation must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder adapter");
        for delegation in [
            "layout_fragment_paint_input::update_paint_style",
            "layout_fragment_paint_input::paint_input_plan",
            "layout_fragment_paint_input::update_paint_input",
        ] {
            assert!(
                adapter.contains(delegation),
                "LayoutObject builder must delegate through {delegation}"
            );
        }
        for leaked_policy in [
            "svg_clip_resources.snapshot.version != clip_resource_version",
            "Arc::make_mut(&mut record.effective_style).style",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutObject builder still owns fragment paint mutation policy {leaked_policy}"
            );
        }
        for leaked_policy in [
            "let fields =",
            "metadata_requires_export",
            "node.UpdateInputData(&record.input)",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutObject builder still owns paint-input policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn gap_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_fragment/src/gap_geometry.rs"))
            .expect("read fragment-owned gap geometry");
        for owned_policy in [
            "pub struct GapGeometry",
            "pub struct GridLanesMainGapSegmentWalker",
            "fn Equals",
            "fn GetCrossingGapSize",
            "fn GenerateIntersectionListForGap",
            "fn GenerateMainIntersectionListForFlex",
            "fn AdjustCrossGapsRangesForFragmentation",
        ] {
            assert!(
                owner.contains(owned_policy),
                "fragment gap-geometry owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "gap equality, crossing-size, intersection generation, and fragmentation adjustment must retain fragment sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/gap/gap_geometry.rs"))
                .expect("read gap-geometry compatibility facade");
        assert!(
            facade.contains("layout_fragment/src/gap_geometry.rs")
                && facade.contains("pub use layout_fragment_gap_geometry::*;")
                && !facade.contains("fn GenerateIntersectionListForGap"),
            "retained gap geometry must be re-exported from layout_fragment"
        );

        for (owner_file, facade_file, owned_type) in [
            ("cross_gap.rs", "cross_gap.rs", "pub struct CrossGap"),
            ("main_gap.rs", "main_gap.rs", "pub struct MainGap"),
            (
                "gap_intersection.rs",
                "gap_intersection.rs",
                "pub struct GapIntersection",
            ),
            ("gap_utils.rs", "gap_utils.rs", "pub struct GapSegmentState"),
            (
                "gap_decoration_utils.rs",
                "gap_decoration_utils.rs",
                "pub struct CSSGapDecorationUtils",
            ),
        ] {
            let companion =
                std::fs::read_to_string(root.join("layout_fragment/src").join(owner_file))
                    .unwrap_or_else(|_| panic!("read fragment gap companion {owner_file}"));
            assert!(
                companion.contains(owned_type),
                "fragment gap companion {owner_file} lost {owned_type}"
            );
            let compatibility =
                std::fs::read_to_string(root.join("layoutng/src/internal/gap").join(facade_file))
                    .unwrap_or_else(|_| panic!("read gap companion facade {facade_file}"));
            assert!(
                compatibility.contains("layout_fragment/src/")
                    && compatibility.contains("pub use layout_fragment_")
                    && !compatibility.contains(owned_type),
                "gap companion {facade_file} must remain a fragment-owner facade"
            );
        }
        let cross_gap = std::fs::read_to_string(root.join("layout_fragment/src/cross_gap.rs"))
            .expect("read fragment cross-gap owner");
        assert!(
            cross_gap.contains("#[inline(never)]")
                && cross_gap.contains("fn AdjustGapSegmentStateRangesForFragmentation"),
            "cross-gap fragmentation range adjustment must retain a fragment sampling frame"
        );
    }

    #[test]
    fn retained_paint_refresh_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/retained_paint_refresh.rs"))
                .expect("read retained paint-refresh owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn refresh_retained_paint_inputs",
            "let mut ink_items",
            "InvalidateInkOverflow",
            "unsafe fn refresh",
            "ExportPaintProperties",
            "ExportCanvasBackground",
            "ExportGlobalMainThreadScrollFacts",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment retained paint refresh lost {owned_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter
                .contains("layout_fragment_retained_paint_refresh::refresh_retained_paint_inputs"),
            "fragment boundary must delegate retained paint refresh to layout_fragment"
        );
        for leaked_policy in [
            "let mut ink_items",
            "InvalidateInkOverflow()",
            "unsafe fn refresh(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns retained paint-refresh policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn prepaint_input_update_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/prepaint_input_update.rs"))
                .expect("read source PrePaint input owner");
        for owned_policy in [
            "pub(super) fn UpdateFragmentPaintStyle",
            "pub(super) fn VisitMutablePaintInputs",
            "pub(super) fn UpdatePrePaintInputs",
            "reference_viewport",
            "FlatPrePaintInput",
            "tracker.update_flat",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source PrePaint input update lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "style, traversal and PrePaint collection must retain sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_fragment/src/prepaint_input_update.rs")
                && adapter.contains("layout_fragment_prepaint_input_update::{"),
            "LayoutEngine must import PrePaint updates from layout_fragment"
        );
        for leaked_policy in [
            "fn UpdateFragmentPaintStyle(",
            "fn VisitMutablePaintInputs(",
            "fn UpdatePrePaintInputs(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns PrePaint input policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn engine_paint_update_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/engine_paint_update.rs"))
                .expect("read source engine paint-update owner");
        for owned_policy in [
            "trait EnginePaintUpdate",
            "fn populate_paint_data",
            "fn refresh_targeted_paint_data",
            "fn refresh_targeted_paint_inputs",
            "fn refresh_scroll_cull_data",
            "fn commit_scroll_origins",
            "RefreshRetainedPaintInputs",
            "ScrollPaintPropertyIndex::new",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source engine paint update lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "paint publication, retained refreshes and CullRect refresh must retain fragment sampling frames"
        );
        assert!(
            !owner
                .rsplit_once("fn refresh_scroll_cull_data")
                .is_some_and(|(_, body)| body.contains("PublishLayoutResult")),
            "CullRect refresh must retain the source fragment graph"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_fragment/src/engine_paint_update.rs")
                && adapter.contains("EnginePaintUpdate::populate_paint_data(self)")
                && adapter.contains("EnginePaintUpdate::refresh_targeted_paint_data(self)")
                && adapter.contains("EnginePaintUpdate::refresh_targeted_paint_inputs(self)")
                && adapter.contains("EnginePaintUpdate::refresh_scroll_cull_data(self, fragments)"),
            "LayoutEngine must delegate paint publication and refresh to layout_fragment"
        );
        for leaked_policy in [
            "TracePhase::PopulatePaintData",
            "TracePhase::RefreshTargetedPaintData",
            "TracePhase::RefreshTargetedPaintInputs",
            "TracePhase::RefreshScrollCullData",
            "RefreshRetainedPaintInputs(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns source paint-update policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_paint_output_traversals_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_paint_output.rs"))
                .expect("read source LayoutPaintOutput traversal owner");
        for owned_policy in [
            "fn paint_cull_rect_has_scrolled_enough",
            "fn changed(cursor: PaintFragmentCursor",
            "fn visit_paint_inputs_mut",
            "unsafe fn visit(",
            "paint_cull_scroll_offset",
            "PaintDataMut",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutPaintOutput traversal lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "public and recursive paint-output traversals must retain fragment sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutPaintOutput adapter");
        assert!(
            adapter.contains("layout_fragment/src/layout_paint_output.rs")
                && adapter.contains(
                    "layout_fragment_layout_paint_output::paint_cull_rect_has_scrolled_enough"
                )
                && adapter.contains("layout_fragment_layout_paint_output::visit_paint_inputs_mut"),
            "LayoutPaintOutput must delegate fragment traversals to layout_fragment"
        );
        for leaked_policy in [
            "const CHANGED_ENOUGH_DISTANCE",
            "unsafe fn visit(\n            cursor: PaintFragmentCursor",
            "cursor.PaintDataMut()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutPaintOutput still owns fragment traversal policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_cursor_children_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_cursor_children.rs"))
                .expect("read source fragment cursor child owner");
        for owned_policy in [
            "fn paint_fragment_children",
            "fn fragment_children",
            "fn append_svg_children",
            "fn append_box_children",
            "SlowFirstChild",
            "PhysicalFragments",
            "DescendantsCount",
            "OffsetInContainerFragment",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment cursor child expansion lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "paint/source cursor entries and child expansion stages must retain fragment sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_output.rs"))
            .expect("read source fragment cursor adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_cursor_children.rs")
                && adapter
                    .contains("layout_fragment_fragment_cursor_children::fragment_children(self)")
                && adapter.contains(
                    "layout_fragment_fragment_cursor_children::paint_fragment_children(self)"
                ),
            "FragmentCursor must delegate child expansion to layout_fragment"
        );
        for leaked_policy in [
            "fn append_svg_children",
            "fn append_box_children",
            "child_index += child.DescendantsCount()",
            ".filter(|child| child.PaintInput().is_some())",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout output facade still owns fragment child expansion {leaked_policy}"
            );
        }
    }

    #[test]
    fn tree_traversal_utils_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/tree_traversal_utils.rs"))
                .expect("read source fragment traversal owner");
        for owned_policy in [
            "trait PhysicalFragmentTraversalListener",
            "fn HandleBoxFragment",
            "fn ForAllBoxFragmentDescendants",
            "fragment.Children()",
            "InlineCursor::from_container_and_items",
            "MoveToIncludingCulledInline",
            "HandleCulledInline",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source traversal lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "physical/inline fragment recursion must retain fragment sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/tree_traversal_utils.rs"))
                .expect("read source fragment traversal facade");
        assert!(
            facade.contains("layout_fragment/src/tree_traversal_utils.rs")
                && facade.contains("pub use layout_fragment_tree_traversal_utils::*"),
            "core traversal module must re-export the layout_fragment owner"
        );
        for leaked_policy in [
            "fn HandleBoxFragment",
            "fn ForAllBoxFragmentDescendants",
            "while cursor.IsNotNull()",
            "MoveToIncludingCulledInline",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "core traversal facade still owns fragment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn pagination_fragment_utils_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/pagination_fragment_utils.rs"))
                .expect("read source pagination fragment owner");
        for owned_policy in [
            "fn GetPageBorderBoxLink",
            "fn GetPageBorderBox",
            "fn GetPageArea",
            "fn StitchedPageContentRect",
            "BoxType::kPageContainer",
            "BoxType::kPageArea",
            "ConsumedBlockSize",
            "GetWritingDirection().BlockEnd",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment pagination geometry lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "page fragment discovery and stitched geometry must retain fragment sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/pagination_utils.rs"))
                .expect("read pagination fragment facade");
        assert!(
            facade.contains("layout_fragment/src/pagination_fragment_utils.rs")
                && facade.contains("pub use layout_fragment_pagination_fragment_utils::*"),
            "core pagination module must re-export the layout_fragment owner"
        );
        for leaked_policy in [
            "fn GetPageBorderBoxLink",
            "fn GetPageArea",
            "fn StitchedPageContentRect",
            "ConsumedBlockSize",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "core pagination facade still owns fragment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_location_update_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_box_location_update.rs"))
                .expect("read source box-location owner");
        for owned_policy in [
            "struct OutOfFlowDescendant",
            "fn SetChildLocation",
            "fn UpdateBoxChildLocations",
            "struct TraversalListener",
            "fn UpdateOffsetsFromRootFragmentationContext",
            "fn UpdateChildLayoutBoxLocations",
            "ForAllBoxFragmentDescendants",
            "StitchedPageContentRect",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment box-location update lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "box, OOF and fragmentation location stages must retain fragment sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_utils.rs"))
                .expect("read source box-location facade");
        assert!(
            facade.contains("layout_fragment/src/layout_box_location_update.rs")
                && facade.contains("pub use layout_fragment_layout_box_location_update::*"),
            "core box-location module must re-export the layout_fragment owner"
        );
        for leaked_policy in [
            "struct OutOfFlowDescendant",
            "fn SetChildLocation",
            "struct TraversalListener",
            "fn UpdateChildLayoutBoxLocations",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "core box-location facade still owns fragment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn scrollable_overflow_recalc_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/scrollable_overflow_recalc.rs"))
                .expect("read source scrollable-overflow owner");
        for owned_policy in [
            "fn RecalcFragmentChildren",
            "fn LayoutBoxRecalcScrollableOverflowNG",
            "fn LayoutBoxRecalcChildScrollableOverflowNG",
            "PostLayoutChildren",
            "RecalculateScrollableOverflowForFragment",
            "InlineCursor::from_fragment",
            "SetScrollableOverflow",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment scrollable-overflow recalc lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "overflow recursion and box/child recalculation must retain fragment sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_core_services.rs"))
                .expect("read LayoutBox core-services facade");
        assert!(
            facade.contains("layout_fragment/src/scrollable_overflow_recalc.rs")
                && facade.contains("LayoutBoxRecalcChildScrollableOverflowNG")
                && facade.contains("LayoutBoxRecalcScrollableOverflowNG"),
            "LayoutBox core services must re-export overflow recalc from layout_fragment"
        );
        for leaked_policy in [
            "fn RecalcFragmentChildren",
            "RecalculateScrollableOverflowForFragment",
            "InlineCursor::from_fragment",
            "SetScrollableOverflow(new_overflow)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox core services still own overflow recalc {leaked_policy}"
            );
        }
    }

    #[test]
    fn scrollable_overflow_calculation_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/scrollable_overflow_calculation.rs"),
        )
        .expect("read source scrollable-overflow calculation owner");
        for owned_policy in [
            "fn RecalculateScrollableOverflowForFragment",
            "fn AddItemsInternal",
            "fn ScrollableOverflowForPropagation",
            "for child in fragment.PostLayoutChildren().iter()",
            "for item in items",
            "while walker != container",
            "GetTransformForChildFragment",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment scrollable-overflow calculation lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "fragment recursion, item traversal, and propagation must retain fragment sampling frames"
        );

        let calculator = std::fs::read_to_string(
            root.join("layout_fragment/src/scrollable_overflow_calculator.rs"),
        )
        .expect("read fragment-owned scrollable-overflow calculator");
        for owned_policy in [
            "pub struct ScrollableOverflowCalculator",
            "fn Result",
            "fn AddChild",
            "fn AddBuilderItems",
            "fn AddFragmentItems",
            "fn AdjustOverflowForScrollOrigin",
            "#[path = \"scrollable_overflow_calculation.rs\"]",
        ] {
            assert!(
                calculator.contains(owned_policy),
                "scrollable-overflow calculator owner lost {owned_policy}"
            );
        }

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/scrollable_overflow_calculator.rs"),
        )
        .expect("read scrollable-overflow calculator facade");
        assert!(
            facade.contains("layout_fragment/src/scrollable_overflow_calculator.rs")
                && facade.contains("pub use layout_fragment_scrollable_overflow_calculator::*;")
                && !facade.contains("pub struct ScrollableOverflowCalculator"),
            "scrollable-overflow calculator must be re-exported from layout_fragment"
        );
        for leaked_policy in [
            "for child in fragment.PostLayoutChildren().iter()",
            "for item in items",
            "while walker != container",
            "GetTransformForChildFragment(child_fragment",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "scrollable-overflow calculator still owns traversal policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn sticky_constraints_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let geometry_owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_sticky_constraints.rs"))
                .expect("read source sticky-constraint geometry owner");
        for owned_policy in [
            "fn MapRectToAncestorIgnoringPaintGeometry",
            "fn ComputeStickyPositionConstraintsForLayout",
            "fn FindFirstStickyContainer",
            "while current != ancestor",
            "while unsafe { &*sticky_container }.IsAnonymous()",
            "while ancestor != below",
        ] {
            assert!(
                geometry_owner.contains(owned_policy),
                "layout_fragment sticky geometry lost {owned_policy}"
            );
        }
        assert!(
            geometry_owner.matches("#[inline(never)]").count() >= 3,
            "coordinate mapping, constraint construction, and sticky-container lookup must retain fragment sampling frames"
        );

        let data_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/sticky_position_scrolling_constraints.rs"),
        )
        .expect("read source sticky scrolling-constraint data owner");
        for owned_policy in [
            "pub struct PerAxisData",
            "fn ComputeOffset",
            "pub struct StickyPositionScrollingConstraints",
            "pub struct StickyConstraintsData",
        ] {
            assert!(
                data_owner.contains(owned_policy),
                "layout_fragment sticky constraint data lost {owned_policy}"
            );
        }

        let geometry_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_sticky_constraints.rs"),
        )
        .expect("read sticky-geometry facade");
        let data_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/sticky_position_scrolling_constraints.rs"),
        )
        .expect("read sticky constraint-data facade");
        assert!(
            geometry_facade.contains("layout_fragment/src/layout_sticky_constraints.rs")
                && geometry_facade
                    .contains("pub use layout_fragment_layout_sticky_constraints::*;")
                && data_facade
                    .contains("layout_fragment/src/sticky_position_scrolling_constraints.rs")
                && data_facade
                    .contains("pub use layout_fragment_sticky_position_scrolling_constraints::*;"),
            "sticky geometry and data must be re-exported from layout_fragment"
        );
        for leaked_policy in [
            "fn ComputeStickyPositionConstraintsForLayout",
            "while unsafe { &*sticky_container }.IsAnonymous()",
            "pub struct PerAxisData",
        ] {
            assert!(
                !geometry_facade.contains(leaked_policy) && !data_facade.contains(leaked_policy),
                "sticky facade still owns fragment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_scrollbars_resolve_to_the_sizing_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_algo_sizing/src/layout_box_scrollbars.rs"))
                .expect("read source LayoutBox scrollbar owner");
        for owned_policy in [
            "fn HypotheticalScrollbarThickness",
            "fn CanSkipComputeScrollbars",
            "fn ComputeScrollbarsInternal",
            "fn UsesOverlayScrollbars",
            "fn HasScrollbarGutters",
            "VerticalScrollbarWidth",
            "HorizontalScrollbarHeight",
            "ShouldPlaceVerticalScrollbarOnLeft",
            "IsScrollbarGutterBothEdges",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_algo_sizing LayoutBox scrollbar sizing lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]\n    pub fn ComputeScrollbarsInternal"),
            "complete scrollbar strut calculation must retain a sizing sampling frame"
        );

        let custom_owner =
            std::fs::read_to_string(root.join("layout_fragment/src/custom_scrollbar.rs"))
                .expect("read source custom scrollbar owner");
        for owned_policy in [
            "pub struct CustomScrollbar",
            "fn SetPartStyle",
            "fn ButtonRect",
            "fn TrackRect",
            "fn TrackPieceRectWithMargins",
            "fn MinimumThumbLength",
            "MakeGarbageCollected(LayoutCustomScrollbarPart::new",
        ] {
            assert!(
                custom_owner.contains(owned_policy),
                "layout_fragment custom scrollbar lost {owned_policy}"
            );
        }
        assert!(
            custom_owner.contains("#[inline(never)]\n    pub fn SetPartStyle"),
            "custom scrollbar part allocation/style mutation must retain a fragment sampling frame"
        );
        let part_identity =
            std::fs::read_to_string(root.join("layout_fragment/src/scrollbar_part.rs"))
                .expect("read source scrollbar-part identity owner");
        assert!(
            part_identity.contains("pub enum ScrollbarPart")
                && part_identity.contains("kAllParts = 0xffff_ffff")
                && part_identity.contains("StrongHeapMapKey for ScrollbarPart")
                && !part_identity.contains("#[inline(never)]"),
            "scrollbar-part identity must stay inlineable in layout_fragment"
        );
        let custom_part_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_custom_scrollbar_part.rs"),
        )
        .expect("read source custom scrollbar part owner");
        for owned_policy in [
            "pub struct LayoutCustomScrollbarPart",
            "fn compute_size",
            "fn compute_width",
            "fn compute_height",
            "fn ComputeThickness",
            "fn ComputeLength",
            "fn MarginOutsets",
        ] {
            assert!(
                custom_part_owner.contains(owned_policy),
                "layout_fragment custom scrollbar part lost {owned_policy}"
            );
        }

        let loader = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_box_scrollbar_gutter.rs"),
        )
        .expect("read LayoutBox scrollbar compatibility loader");
        assert!(
            loader.contains("layout_algo_sizing/src/layout_box_scrollbars.rs"),
            "LayoutBox scrollbar compatibility module must load its sizing owner"
        );
        let custom_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/custom_scrollbar.rs"))
                .expect("read custom scrollbar facade");
        let custom_part_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_custom_scrollbar_part.rs"),
        )
        .expect("read custom scrollbar part facade");
        let part_identity_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/scrollbar_part.rs"))
                .expect("read scrollbar-part identity facade");
        assert!(
            custom_facade.contains("layout_fragment/src/custom_scrollbar.rs")
                && custom_facade.contains("pub use layout_fragment_custom_scrollbar::*;")
                && custom_part_facade
                    .contains("layout_fragment/src/layout_custom_scrollbar_part.rs")
                && custom_part_facade
                    .contains("pub use layout_fragment_layout_custom_scrollbar_part::*;"),
            "custom scrollbar facades must re-export their fragment owners"
        );
        assert!(
            part_identity_facade.contains("layout_fragment/src/scrollbar_part.rs")
                && part_identity_facade.contains("pub use layout_fragment_scrollbar_part::*")
                && !part_identity_facade.contains("pub enum ScrollbarPart"),
            "scrollbar-part identity must be re-exported from layout_fragment"
        );
        let old_layout =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_layout.rs"))
                .expect("read LayoutBox layout module");
        for leaked_policy in [
            "fn HypotheticalScrollbarThickness",
            "fn CanSkipComputeScrollbars",
            "fn ComputeScrollbarsInternal",
            "fn UsesOverlayScrollbars",
            "fn HasScrollbarGutters",
        ] {
            assert!(
                !loader.contains(leaked_policy) && !old_layout.contains(leaked_policy),
                "core LayoutBox modules still own scrollbar policy {leaked_policy}"
            );
        }
        assert!(
            !custom_facade.contains("pub struct CustomScrollbar")
                && !custom_part_facade.contains("pub struct LayoutCustomScrollbarPart"),
            "custom scrollbar facades must not retain fragment state"
        );
    }

    #[test]
    fn scrollable_area_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let area_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/paint_layer_scrollable_area.rs"),
        )
        .expect("read source scrollable-area owner");
        for owned_policy in [
            "pub struct PaintLayerScrollableArea",
            "fn ComputeScrollbarExistence",
            "fn UpdateAfterLayout",
            "fn ClampScrollOffsetAfterOverflowChange",
            "fn UpdateScrollDimensions",
            "fn SetHasHorizontalScrollbar",
            "fn SetHasVerticalScrollbar",
            "fn SetHasHorizontalScrollbarInManager",
            "fn SetHasVerticalScrollbarInManager",
            "fn DestroyDetachedScrollbars",
            "fn DestroyScrollbar",
            "fn ShouldAvoidHidingOverlayScrollbars",
            "fn ShouldSupplyScrollbarsForVisualViewport",
            "fn DidUpdateVisualViewport",
            "fn IsScrollCornerVisible",
            "fn ScrollbarFrameRectChanged",
            "fn ScrollbarOffset",
            "fn PositionOverflowControlsForPrePaint",
            "fn UsedColorSchemeScrollbarsChanged",
            "fn ScrollbarsHiddenIfOverlay",
            "fn SetScrollbarsHiddenIfOverlayFromOwner",
        ] {
            assert!(
                area_owner.contains(owned_policy),
                "layout_fragment scrollable area lost {owned_policy}"
            );
        }
        assert!(
            area_owner.contains("if self.has_horizontal_scrollbar_")
                && area_owner.contains("if self.has_vertical_scrollbar_")
                && area_owner.contains("DelayScrollOffsetClampScope::ClampingIsDelayed()")
                && area_owner.contains("self.DestroyDetachedScrollbars();")
                && area_owner.contains("SetOnlyThisNeedsPaintPropertyUpdate"),
            "scrollbar manager must hide detached bars, retain them during delayed clamp, and destroy them at the clamp boundary"
        );

        let identity_owner =
            std::fs::read_to_string(root.join("layout_fragment/src/scrollbar_identity.rs"))
                .expect("read source scrollbar client identity owner");
        for owned_policy in [
            "pub struct Scrollbar",
            "pub struct ScrollCornerDisplayItemClient",
            "DisplayItemClient",
            "fn DisconnectFromScrollableArea",
            "fn StyleChanged",
            "fn ShouldParticipateInHitTesting",
            "impl Traceable for Scrollbar",
            "impl Traceable for ScrollCornerDisplayItemClient",
        ] {
            assert!(
                identity_owner.contains(owned_policy),
                "layout_fragment source scrollbar identity lost {owned_policy}"
            );
        }

        let scope_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/paint_layer_scrollable_area_scope.rs"),
        )
        .expect("read source scroll-layout scope owner");
        for owned_policy in [
            "pub struct FreezeScrollbarsScope",
            "pub struct FreezeScrollbarsRootScope",
            "pub struct DelayScrollOffsetClampScope",
            "fn ClampScrollableAreas",
            "for index in 0..len",
        ] {
            assert!(
                scope_owner.contains(owned_policy),
                "layout_fragment scroll layout scope lost {owned_policy}"
            );
        }
        assert!(
            scope_owner.contains("#[inline(never)]"),
            "delayed clamp queue traversal must retain a fragment sampling frame"
        );

        let model_owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_box_model_scrollable_area.rs"),
        )
        .expect("read LayoutBoxModelObject scrollable-area owner");
        for owned_policy in [
            "fn CreateScrollableAreaForPaintLayer",
            "fn DisposeScrollableAreaForPaintLayer",
            "PaintLayerScrollableArea::new",
            "InitializeScrollingBackgroundDisplayItemClient",
        ] {
            assert!(
                model_owner.contains(owned_policy),
                "LayoutBoxModelObject scrollable-area owner lost {owned_policy}"
            );
        }
        assert!(
            model_owner.matches("#[inline(never)]").count() >= 2,
            "scrollable-area admission and disposal must retain fragment sampling frames"
        );

        let paint_layer_owner =
            std::fs::read_to_string(root.join("layout_fragment/src/paint_layer.rs"))
                .expect("read PaintLayer owner");
        for owned_policy in [
            "fn RequiresScrollableArea",
            "fn UpdateScrollableArea",
            "fn UpdateScrollingAfterLayout",
            "IsScrollContainer",
            "IsOverscrollContainer",
            "CanResize",
            "CreateScrollableAreaForPaintLayer",
            "DisposeScrollableAreaForPaintLayer",
        ] {
            assert!(
                paint_layer_owner.contains(owned_policy),
                "PaintLayer scrollable-area lifecycle lost {owned_policy}"
            );
        }

        let area_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_scrollable_area.rs"))
                .expect("read scrollable-area facade");
        let scope_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/scroll_layout_scope.rs"))
                .expect("read scroll-layout scope facade");
        let identity_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/scrollbar.rs"))
                .expect("read source scrollbar identity facade");
        assert!(
            area_facade.contains("layout_fragment/src/paint_layer_scrollable_area.rs")
                && area_facade.contains("pub use layout_fragment_paint_layer_scrollable_area::*;")
                && scope_facade
                    .contains("layout_fragment/src/paint_layer_scrollable_area_scope.rs")
                && scope_facade
                    .contains("pub use layout_fragment_paint_layer_scrollable_area_scope::*;")
                && identity_facade.contains("layout_fragment/src/scrollbar_identity.rs")
                && identity_facade.contains("pub use layout_fragment_scrollbar_identity::*;"),
            "scrollable area, scopes, and client identity must be re-exported from layout_fragment"
        );
        let model_loader =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_model_object.rs"))
                .expect("read LayoutBoxModelObject loader");
        assert!(
            model_loader.contains("layout_fragment/src/layout_box_model_scrollable_area.rs")
                && !model_loader.contains("fn CreateScrollableAreaForPaintLayer")
                && !model_loader.contains("fn DisposeScrollableAreaForPaintLayer"),
            "LayoutBoxModelObject must load scrollable-area lifecycle from layout_fragment"
        );
        for leaked_policy in [
            "pub struct PaintLayerScrollableArea",
            "pub struct DelayScrollOffsetClampScope",
            "pub struct ScrollCornerDisplayItemClient",
            "for index in 0..len",
        ] {
            assert!(
                !area_facade.contains(leaked_policy)
                    && !scope_facade.contains(leaked_policy)
                    && !identity_facade.contains(leaked_policy),
                "scroll facade still owns fragment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn motion_path_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let geometry = std::fs::read_to_string(
            root.join("layout_fragment/src/motion_path_geometry_helpers.rs"),
        )
        .expect("read source motion-path geometry owner");
        for owned_policy in [
            "fn MotionPathLength",
            "fn MotionPathPointAndTangentAtLength",
            "fn MotionPathPolygon",
            "fn MotionPathCommands",
            "SkContourMeasureIter::new",
            "for index in 0..count",
            "for command in commands",
        ] {
            assert!(
                geometry.contains(owned_policy),
                "layout_fragment motion-path geometry lost {owned_policy}"
            );
        }
        assert!(
            geometry.matches("#[inline(never)]").count() >= 4,
            "contour measurement, sampling, polygon, and command walks must retain fragment sampling frames"
        );

        let projection = std::fs::read_to_string(
            root.join("layout_fragment/src/motion_path_shape_projection.rs"),
        )
        .expect("read source motion-path projection owner");
        for owned_policy in [
            "fn circle_radius",
            "fn ellipse_path",
            "fn inset_path",
            "fn basic_shape_path",
            "fn resolved_sample",
            "fn PointAndTangentForShape",
            "#[path = \"motion_path_geometry_helpers.rs\"]",
        ] {
            assert!(
                projection.contains(owned_policy),
                "layout_fragment motion-path projection lost {owned_policy}"
            );
        }
        assert!(
            projection.contains("#[inline(never)]"),
            "complete motion-path projection must retain a fragment sampling frame"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/motion_path_geometry.rs"))
                .expect("read motion-path geometry facade");
        assert!(
            facade.contains("layout_fragment/src/motion_path_shape_projection.rs")
                && facade
                    .contains("pub(crate) use layout_fragment_motion_path_shape_projection::*;")
                && !facade.contains("fn PointAndTangentForShape"),
            "motion-path projection must be re-exported from layout_fragment"
        );
        for leaked_policy in [
            "SkContourMeasureIter::new(path",
            "for point in vertices.iter()",
            "for index in 0..count",
            "for command in commands",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "motion-path facade still owns input-sized geometry {leaked_policy}"
            );
        }
    }

    #[test]
    fn transform_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_fragment/src/transform_utils.rs"))
            .expect("read source transform-geometry owner");
        for owned_policy in [
            "impl MotionPathBox for LayoutBox",
            "fn ComputeReferenceBoxInternal",
            "fn GetTransformForChildFragment",
            "fn UpdateTransformState",
            "ApplyTransformPhysicalRect",
            "transform_state.ApplyTransform",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment transform geometry lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "child-fragment transform construction and state projection must retain fragment sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/transform_utils.rs"))
            .expect("read transform-geometry facade");
        assert!(
            facade.contains("layout_fragment/src/transform_utils.rs")
                && facade.contains("pub use layout_fragment_transform_utils::*;")
                && !facade.contains("fn UpdateTransformState"),
            "transform geometry must be re-exported from layout_fragment"
        );

        let retained =
            std::fs::read_to_string(root.join("layout_fragment/src/paint_layer_transform.rs"))
                .expect("read retained layout-transform owner");
        for owned_policy in [
            "fn UpdateTransform",
            "fn GetTransformFromContainer",
            "fn NearestAncestorForElement",
            "fn PerspectiveOrigin",
            "ApplyPerspectiveDepth",
        ] {
            assert!(
                retained.contains(owned_policy),
                "retained layout-transform owner lost {owned_policy}"
            );
        }
        assert!(
            retained.matches("#[inline(never)]").count() >= 3,
            "transform rebuild, container projection, and ancestor lookup must retain fragment sampling frames"
        );
        let retained_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_transform.rs"))
                .expect("read retained layout-transform facade");
        assert!(
            retained_facade.contains("layout_fragment/src/paint_layer_transform.rs")
                && retained_facade.contains("pub use layout_fragment_paint_layer_transform::*;")
                && !retained_facade.contains("fn GetTransformFromContainer"),
            "retained layout transforms must be re-exported from layout_fragment"
        );
    }

    #[test]
    fn client_rects_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/fragment_client_rect_queries.rs"),
        )
        .expect("read source client-rect owner");
        for owned_policy in [
            "fn find_path",
            "fn visit<'a>",
            "fn transform_to_root",
            "fn client_rects",
            "fn map_rect",
            "StickyPositionOffset",
            "ApplyTransformRectF",
            "ClientRectsInContainers",
            "trace::client_rects",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment client-rect query lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "client-rect path, transform, collection and mapping must retain fragment sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_output.rs"))
            .expect("read source client-rect adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_client_rect_queries.rs")
                && adapter
                    .contains("layout_fragment_fragment_client_rect_queries::client_rects(self)"),
            "LayoutNode must delegate client-rect queries to layout_fragment"
        );
        for leaked_policy in [
            "fn FindPath",
            "fn TransformToRoot",
            "struct Step<'a>",
            "ClientRectsInContainers().rects",
            "fn MapRect",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout output facade still owns client-rect policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn inline_client_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/inline_client_geometry.rs"))
                .expect("read source inline client-geometry owner");
        for owned_policy in [
            "pub(crate) struct InlineClientGeometry",
            "pub(crate) fn LayoutInlineClientRectsInContainers",
            "MoveToIncludingCulledInline",
            "IsInlineRubyText",
            "LocalRectForBoxQuad",
            "CurrentRectInFirstContainerFragment",
            "MoveToNextForSameLayoutObject",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment inline client geometry lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "inline client-geometry enumeration must retain a fragment sampling frame"
        );

        let adapter = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "inline/src/layout_inline.rs")),
        )
        .expect("read LayoutInline owner");
        assert!(
            adapter.contains("layout_fragment/src/inline_client_geometry.rs")
                && adapter.contains("LayoutInlineClientRectsInContainers(self)"),
            "LayoutInline must delegate client geometry to layout_fragment"
        );
        for leaked_policy in [
            "MoveToIncludingCulledInline(source)",
            "IsInlineRubyText()",
            "CurrentRectInFirstContainerFragment()",
            "MoveToNextForSameLayoutObject()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutInline owner still contains fragment client-geometry policy {leaked_policy}"
            );
        }
        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_inline.rs"))
            .expect("read LayoutInline compatibility facade");
        assert!(
            facade.contains(concat!("layout_algo_", "inline/src/layout_inline.rs"))
                && facade.contains(concat!("pub use layout_algo_", "inline_layout_inline::*;"))
                && !facade.contains("pub struct LayoutInline"),
            "LayoutInline must be re-exported from the inline algorithm owner"
        );
    }

    #[test]
    fn layout_box_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_box_fragment_geometry.rs"),
        )
        .expect("read source LayoutBox fragment-geometry owner");
        for owned_policy in [
            "fn LayoutBoxLocationContainer",
            "fn LayoutBoxFindPreviousBreakToken",
            "fn fragment_for_edge",
            "fn LayoutBoxStitchedBlockSize",
            "fn LayoutBoxPhysicalContractedBoxRect",
            "fn LayoutBoxComputeSize",
            "fn LayoutBoxBoundingBoxRelativeToFirstFragment",
            "fn LayoutBoxQuadsInAncestorInternal",
            "fn LayoutBoxResolvedDirection",
            "SequenceNumber",
            "ConsumedBlockSize",
            "PhysicalBoxSides::new",
            "WritingModeConverter::without_outer_size",
            "OffsetFromRootFragmentationContext",
            "IsAtBlockEnd",
            "LocalRectForBoxQuad",
            "LocalRectToAncestorQuad",
            "MoveToLayoutObject",
            "ResolvedDirection",
            "while !container.is_null()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutBox geometry lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 8,
            "fragment sizing, bounding-box, ancestor-quad and direction queries must retain fragment sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_fragment/src/layout_box_fragment_geometry.rs")
                && facade.contains("LayoutBoxLocationContainer(self)")
                && facade.contains("LayoutBoxFindPreviousBreakToken(fragment)")
                && facade.contains("LayoutBoxStitchedBlockSize(self)")
                && facade.contains("LayoutBoxPhysicalContractedBoxRect(self, edge)")
                && facade.contains("LayoutBoxComputeSize(self)")
                && facade.contains("LayoutBoxBoundingBoxRelativeToFirstFragment(self)")
                && facade.contains("LayoutBoxQuadsInAncestorInternal(")
                && facade.contains("LayoutBoxResolvedDirection(self)"),
            "LayoutBox must delegate fragment sizing and geometry to layout_fragment"
        );
        for leaked_policy in [
            "while !container.is_null()",
            "fn FragmentForEdge(",
            "let mut previously_consumed_block_size",
            "let mut previous_break_token",
            "fragment.HasScrollbar()",
            "let mut bounding_rect = PhysicalRect::default()",
            "let mut first_fragment: Option<&PhysicalBoxFragment>",
            "quads.push(self.LocalRectToAncestorQuad",
            "cursor.MoveToLayoutObject(self)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns fragment geometry policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_background_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_box_background.rs"))
                .expect("read source LayoutBox background owner");
        for owned_policy in [
            "fn LayoutBoxPhysicalBackgroundRectProvider",
            "fn LayoutBoxBackgroundPaintedExtentProvider",
            "fn LayoutBoxBackgroundIsKnownToBeOpaqueInRectProvider",
            "fn LayoutBoxForegroundIsKnownToBeOpaqueInRectProvider",
            "fn LayoutBoxComputeBackgroundIsKnownToBeObscuredProvider",
            "fn LayoutBoxIntersectsVisibleViewportProvider",
            "fn LayoutBoxEnsureIsReadyForPaintInvalidationProvider",
            "fn LayoutBoxBackgroundClipBorderBoxIsEquivalentToPaddingBoxProvider",
            "KnownToBeOpaque(style)",
            "EnsureIsReadyForPaintInvalidationBase",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutBox background lost {owned_policy}"
            );
        }

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_fragment/src/layout_box_background.rs"),
            "LayoutBox must load background/obscuration policy from layout_fragment"
        );
    }

    #[test]
    fn column_spanner_admission_resolves_to_the_block_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/column_spanner_admission.rs"
        )))
        .expect("read source block column-spanner owner");
        for owned_policy in [
            "fn LayoutBoxIsValidColumnSpannerInTree",
            "fn LayoutBoxIsSelfValidColumnSpanner",
            "fn LayoutBoxDoesAncestryAllowColumnSpanner",
            "fn LayoutBoxShouldPreventColumnSpannerDescendants",
            "ShouldBeHandledAsFloating",
            "kIsOutOfFlowPositioned",
            "IsMulticolContainer",
            "CreatesNewFormattingContext",
            "CanContainFixedPositionObjects",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block column-spanner admission lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "column-spanner admission and ancestor scanning must retain block-algorithm sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "block/src/column_spanner_admission.rs"
            )) && facade.contains("LayoutBoxIsValidColumnSpannerInTree(self, style)")
                && facade.contains("LayoutBoxIsSelfValidColumnSpanner(self, style)")
                && facade.contains("LayoutBoxDoesAncestryAllowColumnSpanner(self)")
                && facade.contains("LayoutBoxShouldPreventColumnSpannerDescendants(self)"),
            "LayoutBox must delegate column-spanner admission to the block algorithm owner"
        );
        for leaked_policy in [
            "let mut ancestor =",
            "ShouldBeHandledAsFloating(style)",
            "GetColumnSpan() != EColumnSpan::kAll",
            "CanContainFixedPositionObjects()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns column-spanner policy {leaked_policy}"
            );
        }

        let path_owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/column_spanner_path.rs")),
        )
        .expect("read block column-spanner path owner");
        assert!(
            path_owner.contains("pub struct ColumnSpannerPath")
                && path_owner.contains("Member<ColumnSpannerPath>"),
            "column-spanner traversal state must remain block-owned"
        );
        let path_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/column_spanner_path.rs"))
                .expect("read column-spanner path facade");
        assert!(
            path_facade.contains(concat!("layout_algo_", "block/src/column_spanner_path.rs"))
                && path_facade.contains(concat!(
                    "pub use layout_algo_",
                    "block_column_spanner_path::*;"
                ))
                && !path_facade.contains("pub struct ColumnSpannerPath"),
            "column-spanner path must be re-exported from the block owner"
        );

        let marker_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/unpositioned_list_marker_interface.rs"
        )))
        .expect("read block-level list-marker state owner");
        assert!(
            marker_owner.contains("pub struct UnpositionedListMarker")
                && marker_owner.contains("UnpositionedListMarkerAddToBoxFromBlock"),
            "block-level list-marker placement state must remain block-owned"
        );
        let marker_provider = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/unpositioned_list_marker.rs"
        )))
        .expect("read block-level list-marker provider");
        assert!(
            marker_provider.matches("#[inline(never)]").count() >= 4,
            "list-marker layout, placement, and float intrusion must retain block sampling frames"
        );
        let marker_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/unpositioned_list_marker.rs"))
                .expect("read block-level list-marker facade");
        assert!(
            marker_facade.contains(concat!(
                "layout_algo_",
                "block/src/unpositioned_list_marker_interface.rs"
            )) && marker_facade.contains(concat!(
                "pub use layout_algo_",
                "block_unpositioned_list_marker_interface::*;"
            )) && !marker_facade.contains("pub struct UnpositionedListMarker"),
            "block-level list-marker state must be re-exported from the block owner"
        );
    }

    #[test]
    fn devtools_flex_info_resolves_to_the_flex_algorithm_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "flex/src/devtools_flex_info.rs")),
        )
        .expect("read flex DevTools geometry owner");
        assert!(
            owner.contains("pub struct DevtoolsFlexInfo")
                && owner.contains("pub struct DevtoolsFlexInfoLine")
                && owner.contains("pub struct DevtoolsFlexInfoItem"),
            "flex diagnostic geometry must remain flex-owned"
        );
        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/devtools_flex_info.rs"))
                .expect("read flex DevTools geometry facade");
        assert!(
            facade.contains(concat!("layout_algo_", "flex/src/devtools_flex_info.rs"))
                && facade.contains(concat!(
                    "pub use layout_algo_",
                    "flex_devtools_flex_info::*;"
                ))
                && !facade.contains("pub struct DevtoolsFlexInfo"),
            "flex diagnostic geometry must be re-exported from its algorithm owner"
        );
    }

    #[test]
    fn layout_result_overflow_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_result_overflow.rs"))
                .expect("read source layout-result overflow owner");
        for owned_policy in [
            "fn LayoutBoxSetScrollableOverflowFromLayoutResults",
            "layout_results_.iter()",
            "ConsumedBlockSize",
            "IsRepeated",
            "IsFlippedBlocksWritingMode",
            "PhysicalPaddingBoxRect",
            "BoxScrollableOverflowModel::new",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment result-overflow aggregation lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "layout-result overflow aggregation must retain a fragment sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_fragment/src/layout_result_overflow.rs")
                && facade.contains("LayoutBoxSetScrollableOverflowFromLayoutResults(self)"),
            "LayoutBox must delegate result-overflow aggregation to layout_fragment"
        );
        for leaked_policy in [
            "let mut consumed_block_size = LayoutUnit::default()",
            "fragment.ScrollableOverflow()",
            "BoxScrollableOverflowModel::new(&scrollable_overflow)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns result-overflow policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn snap_area_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_fragment/src/snap_area.rs"))
            .expect("read source snap-area owner");
        for owned_policy in [
            "pub struct SnapArea",
            "fn from_element",
            "fn GetElementIfConsumed",
            "fn IsPending",
            "fn ConsumedAxes",
            "fn PendingAxes",
            "fn ContainerWritingDirectionMode",
            "impl PartialEq for SnapArea",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment snap-area state lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "scalar snap-area state must remain inlineable"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/snap_area.rs"))
            .expect("read source snap-area facade");
        assert!(
            facade.contains("layout_fragment/src/snap_area.rs")
                && facade.contains("pub use layout_fragment_snap_area::*;"),
            "snap-area state must be re-exported from layout_fragment"
        );
        assert!(
            !facade.contains("pub struct SnapArea"),
            "snap-area facade must not retain fragment state"
        );
    }

    #[test]
    fn physical_fragment_list_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/physical_fragment_list.rs"))
                .expect("read source physical-fragment list owner");
        for owned_policy in [
            "pub struct PhysicalFragmentList",
            "pub struct PhysicalFragmentIterator",
            "fn SlowHasFragmentItems",
            "fn IndexOf",
            "impl<'a> Iterator for PhysicalFragmentIterator",
            "GetPhysicalFragment",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment physical-fragment list lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "linear fragment item and index scans must retain sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_fragment/src/physical_fragment_list.rs")
                && facade.contains("PhysicalFragmentIterator, PhysicalFragmentList"),
            "LayoutBox must re-export physical-fragment list types from layout_fragment"
        );
        for leaked_policy in [
            "pub struct PhysicalFragmentIterator<'a>",
            "impl<'a> Iterator for PhysicalFragmentIterator",
            "self.layout_results_.iter().enumerate()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns physical-fragment list policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_visual_overflow_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_visual_overflow.rs"))
                .expect("read source fragment visual-overflow owner");
        for owned_policy in [
            "fn LayoutBoxCopyVisualOverflowFromFragmentsWithoutInvalidations",
            "PhysicalFragmentCount",
            "SelfInkOverflowRect",
            "ContentsInkOverflowRect",
            "ConsumedBlockSize",
            "IsHorizontalWritingMode",
            "IsRepeated",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment visual-overflow stitching lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "multi-fragment visual-overflow stitching must retain a fragment sampling frame"
        );

        let box_owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_box_visual_overflow.rs"))
                .expect("read LayoutBox visual-overflow owner");
        for owned_policy in [
            "fn AddSelfVisualOverflow",
            "fn AddContentsVisualOverflow",
            "fn SetVisualOverflow",
            "fn CopyVisualOverflowFromFragmentsWithoutInvalidations",
            "fn BorderOutsetsForClipping",
            "fn VisualOverflowRect",
            "#[path = \"fragment_visual_overflow.rs\"]",
        ] {
            assert!(
                box_owner.contains(owned_policy),
                "LayoutBox visual-overflow owner lost {owned_policy}"
            );
        }
        let model = std::fs::read_to_string(root.join("layout_fragment/src/overflow_model.rs"))
            .expect("read retained overflow-model owner");
        for owned_type in [
            "pub struct BoxScrollableOverflowModel",
            "pub struct BoxVisualOverflowModel",
            "pub struct PreviousOverflowData",
            "pub struct BoxOverflowModel",
        ] {
            assert!(
                model.contains(owned_type),
                "retained overflow-model owner lost {owned_type}"
            );
        }
        let model_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/overflow_model.rs"))
                .expect("read overflow-model facade");
        assert!(
            model_facade.contains("layout_fragment/src/overflow_model.rs")
                && model_facade.contains("pub use layout_fragment_overflow_model::*;")
                && !model_facade.contains("pub struct BoxOverflowModel"),
            "overflow storage must be re-exported from layout_fragment"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_box_visual_overflow.rs"),
        )
        .expect("read LayoutBox visual-overflow facade");
        assert!(
            facade.contains("layout_fragment/src/layout_box_visual_overflow.rs")
                && facade.contains("pub use layout_fragment_layout_box_visual_overflow::*;")
                && !facade.contains("fn VisualOverflowRect"),
            "LayoutBox visual overflow must be re-exported from layout_fragment"
        );
        for leaked_policy in [
            "for index in 0..fragment_count",
            "fragment.SelfInkOverflowRect()",
            "break_token }.ConsumedBlockSize()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox visual-overflow facade still owns fragment stitching {leaked_policy}"
            );
        }
    }

    #[test]
    fn retained_paint_support_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        for (owner_file, facade_file, owned_policy) in [
            ("paint_layer.rs", "paint_layer.rs", "pub struct PaintLayer"),
            (
                "outline_info.rs",
                "outline_info.rs",
                "pub struct LayoutOutlineInfo",
            ),
            (
                "outline_geometry.rs",
                "outline_geometry.rs",
                "fn OutlineOutsetExtent",
            ),
            (
                "outline_rect_collector.rs",
                "outline_rect_collector.rs",
                "pub trait OutlineRectCollector",
            ),
            (
                "text_overflow_post_layout_snapshot.rs",
                "text_overflow_post_layout_snapshot.rs",
                "pub struct TextOverflowPostLayoutSnapshot",
            ),
        ] {
            let owner = std::fs::read_to_string(root.join("layout_fragment/src").join(owner_file))
                .unwrap_or_else(|_| panic!("read retained paint owner {owner_file}"));
            assert!(
                owner.contains(owned_policy),
                "retained paint owner {owner_file} lost {owned_policy}"
            );
            let facade =
                std::fs::read_to_string(root.join("layoutng/src/internal").join(facade_file))
                    .unwrap_or_else(|_| panic!("read retained paint facade {facade_file}"));
            assert!(
                facade.contains("layout_fragment/src/")
                    && facade.contains("pub use layout_fragment_")
                    && !facade.contains(owned_policy),
                "retained paint support {facade_file} must remain a fragment-owner facade"
            );
        }
        let layer_owner = std::fs::read_to_string(root.join("layout_fragment/src/paint_layer.rs"))
            .expect("read retained PaintLayer owner");
        for source_method in [
            "fn DebugName",
            "fn OwnerNodeId",
            "fn PaintsWithFilters",
            "fn FirstChild",
            "fn NextSibling",
            "fn UpdateDescendantDependentFlags",
            "has_visible_content_",
            "has_visible_self_painting_descendant_",
            "fn HasVisibleSelfPaintingDescendant",
        ] {
            assert!(
                layer_owner.contains(source_method),
                "retained PaintLayer lost Chromium method {source_method}"
            );
        }
        for undefined_provider in [
            "DispatchLayoutObjectRecalcVisualOverflow",
            "HasNonIsolatedBlendingDescendants()",
        ] {
            assert!(
                !layer_owner.contains(undefined_provider),
                "PaintLayer must not revive undefined provider {undefined_provider}"
            );
        }
        let snapshot = std::fs::read_to_string(
            root.join("layout_fragment/src/text_overflow_post_layout_snapshot.rs"),
        )
        .expect("read text-overflow snapshot owner");
        assert!(
            snapshot.contains("#[inline(never)]")
                && snapshot.contains("fn UpdateSnapshot")
                && snapshot.contains("SetNeedsLayout"),
            "text-overflow snapshot refresh must retain a fragment sampling frame"
        );
    }

    #[test]
    fn layout_object_overflow_recalc_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_overflow_recalc.rs"),
        )
        .expect("read source LayoutObject overflow-recalc owner");
        for owned_policy in [
            "fn RecalcNormalFlowChildVisualOverflowIfNeeded",
            "fn RecalcScrollableOverflow",
            "fn RecalcVisualOverflow",
            "fn RecalcVisualOverflowBase",
            "fn LayoutObjectRecalcScrollableOverflow",
            "fn LayoutObjectRecalcVisualOverflow",
            "self.RecalcVisualOverflow()",
            "ChildNeedsScrollableOverflowRecalc",
            "DispatchLayoutObjectRecalcScrollableOverflow",
            "DispatchLayoutObjectRecalcVisualOverflow",
            "while !child.is_null()",
            "current.HasLayer()",
            "HasSelfPaintingLayer()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject overflow traversal lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "normal-flow, base scrollable, and visual overflow traversals must retain fragment sampling frames"
        );
        assert!(
            !owner.contains("LayoutObjectRecalcNormalFlowChildVisualOverflowIfNeededProvider"),
            "normal-flow visual overflow must not retain its dead provider"
        );
        let layout_object =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
                .expect("read LayoutObject declarations");
        assert!(
            !layout_object
                .contains("LayoutObjectRecalcNormalFlowChildVisualOverflowIfNeededProvider"),
            "LayoutObject declarations must not retain the dead normal-flow provider"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services facade");
        assert!(
            facade.contains("layout_fragment/src/layout_object_overflow_recalc.rs"),
            "LayoutObject core services must load overflow traversal from layout_fragment"
        );
        for leaked_policy in [
            "fn RecalcScrollableOverflow",
            "fn RecalcVisualOverflow",
            "fn RecalcNormalFlowChildVisualOverflowIfNeeded",
            "changed |= unsafe { DispatchLayoutObjectRecalcScrollableOverflow",
            "unsafe { DispatchLayoutObjectRecalcVisualOverflow(child) }",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject core services still own overflow traversal {leaked_policy}"
            );
        }

        let dispatcher =
            std::fs::read_to_string(root.join("layoutng_assembly/src/block_virtual_dispatch.rs"))
                .expect("read LayoutObject virtual dispatcher");
        for dispatch_policy in [
            "fn DispatchLayoutObjectRecalcVisualOverflow",
            "LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot",
            "IsLayoutBlockClass(runtime_class)",
            "object.cast::<LayoutBlock>() }.RecalcVisualOverflow()",
            "object.cast::<LayoutText>() }.RecalcVisualOverflow()",
            "object.cast::<LayoutBoxModelObject>() }.RecalcVisualOverflow()",
        ] {
            assert!(
                dispatcher.contains(dispatch_policy),
                "LayoutObject visual-overflow dispatch lost {dispatch_policy}"
            );
        }

        let block = std::fs::read_to_string(root.join("layout_algo_block/src/layout_block.rs"))
            .expect("read LayoutBlock visual-overflow owner");
        assert!(
            block.contains("Source: core/layout/layout_block.cc:629-641")
                && block.contains("self.ClearVisualOverflow()")
                && block.contains("fragment.GetMutableForPainting().RecalcInkOverflow()")
                && !block.contains("object.RecalcVisualOverflowBase()"),
            "LayoutBlock must recalculate resident fragment ink overflow instead of substituting base traversal"
        );

        let fragment =
            std::fs::read_to_string(root.join("layout_fragment/src/physical_box_fragment.rs"))
                .expect("read PhysicalBoxFragment ink-overflow owner");
        for owned_policy in [
            "pub fn RecalcInkOverflow(&self)",
            "fn RecalcContentsInkOverflow(&self)",
            "CopyVisualOverflowFromFragments()",
            "table_virtuals.computed_row_span",
            "recalc_text_combine_contents_ink_overflow",
            "Source: core/layout/physical_box_fragment.cc:1001-1036",
            "Source: core/layout/physical_box_fragment.cc:1040-1095",
        ] {
            assert!(
                fragment.contains(owned_policy),
                "PhysicalBoxFragment ink-overflow owner lost {owned_policy}"
            );
        }
        assert!(
            !fragment.contains("PhysicalBoxFragmentComputeSelfInkOverflow")
                && !fragment.contains(
                    "table-row self ink overflow requires LayoutTableCell row-span dispatch"
                )
                && !fragment.contains("combined-text ink overflow requires LayoutTextCombine"),
            "PhysicalBoxFragment must not retain dead or substitute ink-overflow providers"
        );

        let fragment_item =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_item.rs"))
                .expect("read FragmentItem ink-overflow owner");
        assert!(
            fragment_item.contains("adjust_text_combine_bounding_box")
                && fragment_item.contains(".SetTextInkOverflow(")
                && fragment_item.contains(".SetSvgTextInkOverflow(")
                && !fragment_item
                    .contains("text ink-overflow recomputation requires InlinePaintContext")
                && !fragment_item.contains("AdjustRectForBoundingBox"),
            "FragmentItem must retain source-owned text overflow and cross the installed inline-owner callback"
        );

        let ink_overflow =
            std::fs::read_to_string(root.join("layout_fragment/src/ink_overflow.rs"))
                .expect("read InkOverflow owner");
        for owned_policy in [
            "Source: core/layout/ink_overflow.cc:351-370",
            "Source: core/layout/ink_overflow.cc:372-436",
            "Source: core/layout/ink_overflow.cc:439-489",
            "used_font.TextInkBounds(text_info)",
            "ExpandForShadowOverflow(",
            "self.SetSelf(type_, &ink_overflow",
        ] {
            assert!(
                ink_overflow.contains(owned_policy),
                "InkOverflow text recomputation lost {owned_policy}"
            );
        }

        let inline_installer =
            std::fs::read_to_string(root.join("layout_algo_inline/src/assembly.rs"))
                .expect("read inline algorithm installer");
        assert!(
            inline_installer.contains("adjust_text_combine_bounding_box =")
                && inline_installer.contains("recalc_text_combine_contents_ink_overflow =")
                && inline_installer.contains("adjust_combined_text_style =")
                && inline_installer.contains("is_text_fragment_selected ="),
            "the inline package must install text-combine overflow, style, and selection callbacks"
        );

        let layout_text_interface =
            std::fs::read_to_string(root.join("layout_algo_inline/src/layout_text_interface.rs"))
                .expect("read LayoutText SVG font dispatch");
        let svg_installer = std::fs::read_to_string(root.join("layout_algo_svg/src/assembly.rs"))
            .expect("read SVG algorithm installer");
        assert!(
            layout_text_interface.contains("RegisterSvgInlineTextFontVirtuals")
                && layout_text_interface.contains("SVG_INLINE_TEXT_SCALED_FONT")
                && layout_text_interface.contains("SVG_INLINE_TEXT_SCALING_FACTOR")
                && svg_installer.contains(
                    "RegisterSvgInlineTextFontVirtuals(SvgInlineTextScaledFont, SvgInlineTextScalingFactor)"
                ),
            "SVG scaled-font access must remain installed beyond LayoutPassScope for retained overflow recomputation"
        );
        assert!(
            !layout_text_interface.contains("LayoutTextScaledFont(")
                && !inline_installer.contains("LayoutTextScaledFont"),
            "the removed free-function scaled-font substitute must not return"
        );

        let outline =
            std::fs::read_to_string(root.join("layout_fragment/src/outline_rect_collector.rs"))
                .expect("read outline collector owner");
        assert!(
            outline.contains("current.UniteEvenIfEmpty(rect)")
                && outline.contains("self.rects_.extend(rects)")
                && !outline.contains("UnionOutlineRectCollectorAddRect")
                && !outline.contains("VectorOutlineRectCollectorCombineWithOffset"),
            "outline collectors must keep their source-owned local operations without dead providers"
        );
    }

    #[test]
    fn layout_object_geometry_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_geometry_queries.rs"),
        )
        .expect("read source LayoutObject geometry-query owner");
        for owned_policy in [
            "fn ContainingScrollContainer(",
            "fn LocalToAncestorPoint(",
            "fn LayoutObjectContainingScrollContainer",
            "fn LayoutObjectLocalToAncestorPoint",
            "IsOverflowValueScrollableY",
            "IsOverflowValueScrollableX",
            "while !ancestor.is_null()",
            "OffsetFromContainer",
            "point.Offset",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject geometry query lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "scroll-owner and coordinate ancestor walks must retain fragment sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services facade");
        assert!(
            facade.contains("layout_fragment/src/layout_object_geometry_queries.rs"),
            "LayoutObject core services must load geometry walks from layout_fragment"
        );
        for leaked_policy in [
            "fn ContainingScrollContainer(",
            "fn LocalToAncestorPoint(",
            "IsOverflowValueScrollableY()",
            "point.Offset(offset.left.ToFloat()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject core services still own geometry query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_scroll_container_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_scroll_container_queries.rs"),
        )
        .expect("read source LayoutObject scroll-container-query owner");
        for owned_policy in [
            "fn IsContentMovingOverscrollContainer",
            "fn IsScrollContainerWithScrollMarkerGroup",
            "fn IsBackdropForOverscrollAreaParent",
            "fn IsScrollMarker(",
            "fn IsScrollMarkerGroupBefore",
            "fn IsScrollContainer(",
            "fn IsOverscrollContainer(",
            "fn ContainingScrollContainerAnyAxis(",
            "fn ContainingScrollContainerAnyAxisDefault",
            "fn ContainingScrollContainerLayerAnyAxis(",
            "fn ContainingScrollContainerLayerAnyAxisDefault",
            "fn ContainingScrollContainerLayerForAxis(",
            "fn ContainingScrollContainerLayerForAxisDefault",
            "self.EnclosingLayer()",
            ".ContainingScrollContainerLayer(&mut is_fixed_to_view",
            "style.IsOverflowValueScrollableY()",
            "LayoutObjectIsBackdropForOverscrollAreaParentProvider",
            "LayoutObjectIsScrollMarkerGroupBeforeProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment scroll-container query lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            3,
            "scroll box and layer ancestry queries must retain separate fragment frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_scroll_container_queries.rs"),
            "LayoutObject must load scroll-container queries from layout_fragment"
        );
        for leaked_policy in [
            "fn IsContentMovingOverscrollContainer",
            "fn IsScrollContainerWithScrollMarkerGroup",
            "fn IsBackdropForOverscrollAreaParent",
            "fn IsScrollMarker(",
            "fn IsScrollMarkerGroupBefore",
            "fn IsScrollContainer(",
            "fn IsOverscrollContainer(",
            "fn ContainingScrollContainerAnyAxis(",
            "fn ContainingScrollContainerAnyAxisDefault",
            "fn ContainingScrollContainerLayerAnyAxis(",
            "fn ContainingScrollContainerLayerAnyAxisDefault",
            "fn ContainingScrollContainerLayerForAxis(",
            "fn ContainingScrollContainerLayerForAxisDefault",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns scroll-container query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_fragment_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_fragment_queries.rs"),
        )
        .expect("read source LayoutObject fragment-query owner");
        for owned_policy in [
            "fn HitTestForOcclusion",
            "fn HitTestAllPhases",
            "fn NodeForHitTest",
            "fn UpdateHitTestResult",
            "fn NodeAtPoint",
            "fn MapToVisualRectInAncestorSpacePhysical",
            "fn MapToVisualRectInAncestorSpaceFloat",
            "fn MapToVisualRectInAncestorSpaceInternal(",
            "fn GetPropertyContainer",
            "fn LocalCaretRect",
            "fn VisibleToHitTestRequest",
            "fn MapLocalToAncestor",
            "fn MapAncestorToLocal",
            "fn MapToVisualRectInAncestorSpaceInternalFastPath",
            "LayoutObjectHitTestForOcclusionProvider",
            "LayoutObjectHitTestAllPhasesProvider",
            "MapVisualRectInternalDispatch",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject query owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 9,
            "hit-test, property, and ancestor traversals must retain fragment sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_fragment_queries.rs"),
            "LayoutObject must load hit-test and mapping queries from layout_fragment"
        );
        for leaked_policy in [
            "fn HitTestForOcclusion",
            "fn HitTestAllPhases",
            "fn NodeForHitTest",
            "fn UpdateHitTestResult",
            "fn NodeAtPoint",
            "fn MapToVisualRectInAncestorSpacePhysical",
            "fn MapToVisualRectInAncestorSpaceFloat",
            "fn MapToVisualRectInAncestorSpaceInternal(",
            "fn GetPropertyContainer",
            "fn LocalCaretRect",
            "fn VisibleToHitTestRequest",
            "fn MapLocalToAncestor",
            "fn MapAncestorToLocal",
            "fn MapToVisualRectInAncestorSpaceInternalFastPath",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns fragment query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_selection_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_selection_queries.rs"),
        )
        .expect("read source LayoutObject selection-query owner");
        for owned_policy in [
            "fn CanUpdateSelectionOnRootLineBoxes",
            "fn AbsoluteSelectionRect",
            "fn CanBeSelectionLeaf",
            "fn IsSelected",
            "fn IsSelectable",
            "self.ContainingBlock().as_ref()",
            "self.MapToVisualRectInAncestorSpacePhysical",
            "self.SlowFirstChild().is_null()",
            "self.StyleRef().Visibility() != EVisibility::kVisible",
            "self.HasLockedAncestorPreventingPaint()",
            "self.CanBeSelectionLeafInternal()",
            "self.GetSelectionState() != SelectionState::kNone",
            "factories.is_text_fragment_selected",
            "self.StyleRef().IsSelectable()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject selection query lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "only retained selection-rectangle aggregation should force a fragment sampling frame"
        );
        assert!(
            !owner.contains("LayoutObjectIsSelectedProvider")
                && !owner.contains("LayoutObjectIsSelectableProvider"),
            "fixed-cost selection queries must not retain dead providers"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_selection_queries.rs"),
            "LayoutObject must load selection queries from layout_fragment"
        );
        assert!(
            !loader.contains("LayoutObjectIsSelectedProvider")
                && !loader.contains("LayoutObjectIsSelectableProvider"),
            "LayoutObject declarations must not retain dead selection providers"
        );
        for leaked_policy in [
            "fn CanUpdateSelectionOnRootLineBoxes",
            "fn AbsoluteSelectionRect",
            "fn CanBeSelectionLeaf",
            "fn IsSelected",
            "fn IsSelectable",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns selection query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_coordinate_conversion_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_coordinate_conversion.rs"),
        )
        .expect("read source LayoutObject coordinate-conversion owner");
        for owned_policy in [
            "fn OffsetFromContainerInternalBase",
            "fn OffsetFromScrollableContainer",
            "fn OffsetFromOverscrollContainer",
            "fn OffsetFromAncestor",
            "fn OffsetFromContainer(",
            "fn OffsetFromContainerInternal",
            "fn AncestorToLocalRect",
            "fn AncestorToLocalQuad",
            "fn AncestorToLocalPointF",
            "fn LocalToAncestorRect",
            "fn MapLocalToAncestorForGeometry",
            "fn LocalRectToAncestorQuad",
            "fn LocalToAncestorQuad",
            "fn LocalToAncestorPointPhysical",
            "fn LocalToAncestorTransform",
            "fn LocalToAbsoluteTransform",
            "fn LocalToAbsoluteRect",
            "fn LocalToAbsoluteQuad",
            "fn AbsoluteToLocalRect",
            "fn AbsoluteToLocalQuad",
            "fn AbsoluteToLocalPointF",
            "DispatchLayoutObjectOffsetFromContainerInternal",
            "LayoutObjectAncestorToLocalPointFProvider",
            "LayoutObjectLocalToAncestorTransformProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment coordinate conversion lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            7,
            "scroll offset plus provider-backed offset, quad, point, rect, and transform ancestry operations must retain fragment frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_coordinate_conversion.rs"),
            "LayoutObject must load coordinate conversion from layout_fragment"
        );
        for leaked_policy in [
            "fn OffsetFromContainerInternalBase",
            "fn OffsetFromScrollableContainer",
            "fn OffsetFromOverscrollContainer",
            "fn OffsetFromAncestor",
            "fn OffsetFromContainer(",
            "fn OffsetFromContainerInternal",
            "fn AncestorToLocalRect",
            "fn AncestorToLocalQuad",
            "fn AncestorToLocalPointF",
            "fn LocalToAncestorRect",
            "fn LocalRectToAncestorQuad",
            "fn LocalToAncestorQuad",
            "fn LocalToAncestorPointPhysical",
            "fn LocalToAncestorTransform",
            "fn LocalToAbsoluteTransform",
            "fn LocalToAbsoluteRect",
            "fn LocalToAbsoluteQuad",
            "fn AbsoluteToLocalRect",
            "fn AbsoluteToLocalQuad",
            "fn AbsoluteToLocalPointF",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns coordinate conversion {leaked_policy}"
            );
        }
        let core_services = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core services");
        for leaked_policy in [
            "fn OffsetFromContainerInternalBase",
            "fn OffsetFromScrollableContainer",
            "fn OffsetFromOverscrollContainer",
        ] {
            assert!(
                !core_services.contains(leaked_policy),
                "LayoutObject core services still own coordinate conversion {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_bounding_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_bounding_geometry.rs"),
        )
        .expect("read source LayoutObject bounding-geometry owner");
        for owned_policy in [
            "fn AddAbsoluteRectForLayer",
            "fn QuadsInAncestorInternal",
            "fn QuadsInAncestor(",
            "fn AbsoluteQuads",
            "fn AbsoluteBoundingBoxRectF(",
            "fn AbsoluteBoundingBoxRect(",
            "fn AbsoluteBoundingBoxRectForUnboundedElement",
            "fn AbsoluteBoundingBoxRectHandlingEmptyInline",
            "fn AbsoluteBoundingBoxRectForScrollIntoView",
            "fn AbsoluteBoundingBoxRectIncludingDescendants",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment bounding geometry lost {owned_policy}"
            );
        }
        for replaced_bypass in [
            "LayoutObjectAbsoluteBoundingBoxRectFProvider",
            "LayoutObjectAbsoluteBoundingBoxRectIncludingDescendantsProvider",
            "LayoutObjectAddAbsoluteRectForLayerProvider",
        ] {
            assert!(
                !owner.contains(replaced_bypass),
                "native bounds still bypass source owner through {replaced_bypass}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            7,
            "layer rectangle and six absolute bounds operations must retain fragment sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_bounding_geometry.rs"),
            "LayoutObject must load bounding geometry from layout_fragment"
        );
        for leaked_policy in [
            "fn AddAbsoluteRectForLayer",
            "fn QuadsInAncestorInternal",
            "fn QuadsInAncestor(",
            "fn AbsoluteQuads",
            "fn AbsoluteBoundingBoxRectF(",
            "fn AbsoluteBoundingBoxRect(",
            "fn AbsoluteBoundingBoxRectForUnboundedElement",
            "fn AbsoluteBoundingBoxRectHandlingEmptyInline",
            "fn AbsoluteBoundingBoxRectForScrollIntoView",
            "fn AbsoluteBoundingBoxRectIncludingDescendants",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns bounding geometry {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_paint_dispatch_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_paint_dispatch.rs"),
        )
        .expect("read source LayoutObject paint-dispatch owner");
        for owned_policy in [
            "fn IsCanvasOrInCanvasSubtree",
            "fn IsInCanvasSubtree",
            "fn Paint(",
            "fn AddDraggableRegions",
            "fn CanHaveAdditionalCompositingReasons",
            "fn AdditionalCompositingReasons",
            "fn CanvasForDrawingLayoutObject",
            "fn AddOutlineRectsBase",
            "fn AddOutlineRects(",
            "fn CollectOutlineRectsAndAdvance",
            "fn OutlineRects(",
            "fn DebugRect",
            "LayoutObjectPaintProvider",
            "InputCanvasLayoutSubtree",
            "InputHasDrawableAttribute",
            "DispatchLayoutObjectAddOutlineRects",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject paint dispatch lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            4,
            "the three multi-fragment outline entries and canvas ancestry lookup must retain paint-dispatch sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_paint_dispatch.rs"),
            "LayoutObject must load paint dispatch from layout_fragment"
        );
        assert!(
            !loader.contains("LayoutObjectCanvasForDrawingLayoutObjectProvider"),
            "LayoutObject declarations must not retain the translated canvas ancestry provider"
        );
        let dispatcher =
            std::fs::read_to_string(root.join("layoutng_assembly/src/block_virtual_dispatch.rs"))
                .expect("read LayoutObject virtual dispatcher");
        assert!(
            dispatcher.contains("fn DispatchLayoutObjectIsCanvas")
                && dispatcher.contains("InputIsHTMLCanvasElement")
                && !dispatcher
                    .contains("block_false_predicates!(\n    DispatchLayoutObjectIsCanvas"),
            "LayoutHTMLCanvas identity must not remain a false base predicate"
        );
        for leaked_policy in [
            "fn IsCanvasOrInCanvasSubtree",
            "fn IsInCanvasSubtree",
            "fn Paint(",
            "fn AddDraggableRegions",
            "fn CanHaveAdditionalCompositingReasons",
            "fn AdditionalCompositingReasons",
            "fn CanvasForDrawingLayoutObject",
            "fn AddOutlineRectsBase",
            "fn AddOutlineRects(",
            "fn CollectOutlineRectsAndAdvance",
            "fn OutlineRects(",
            "fn DebugRect",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns paint dispatch {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_paint_state_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_object_paint_state.rs"))
                .expect("read source LayoutObject paint-state owner");
        for owned_policy in [
            "fn SetIsBackgroundAttachmentFixedObject",
            "fn SetCanCompositeBackgroundAttachmentFixed",
            "fn BackgroundIsKnownToBeObscured",
            "fn SetBackgroundIsKnownToBeObscured",
            "fn GetImageOrientation",
            "LayoutObjectSetIsBackgroundAttachmentFixedObjectProvider",
            "LayoutObjectSetCanCompositeBackgroundAttachmentFixedProvider",
            "LayoutObjectBackgroundIsKnownToBeObscuredProvider",
            "LayoutObjectGetImageOrientationProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject paint state lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            0,
            "fixed-cost per-object paint state must remain inlineable"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_paint_state.rs"),
            "LayoutObject must load paint state from layout_fragment"
        );
        for leaked_policy in [
            "fn SetIsBackgroundAttachmentFixedObject",
            "fn SetCanCompositeBackgroundAttachmentFixed",
            "fn BackgroundIsKnownToBeObscured",
            "fn SetBackgroundIsKnownToBeObscured",
            "fn GetImageOrientation",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns paint state {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_effective_style_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_effective_style.rs"),
        )
        .expect("read source LayoutObject effective-style owner");
        for owned_policy in [
            "fn EffectiveStyle",
            "fn SlowEffectiveStyle",
            "fn ResolveColorWithStyle",
            "fn ResolveColor(",
            "fn CreatesGroup",
            "fn HasClipRelatedProperty",
            "StyleVariant::kFirstLineEllipsis",
            "ContainingBlock()",
            "LayoutObjectHasClipRelatedPropertyProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment effective style lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "only non-standard style-variant resolution should force a fragment sampling frame"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_effective_style.rs"),
            "LayoutObject must load effective style from layout_fragment"
        );
        for leaked_policy in [
            "fn EffectiveStyle",
            "fn ResolveColorWithStyle",
            "fn ResolveColor(",
            "fn CreatesGroup",
            "fn HasClipRelatedProperty",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns effective-style policy {leaked_policy}"
            );
        }

        let core_services = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core services");
        assert!(
            !core_services.contains("fn SlowEffectiveStyle"),
            "LayoutObject core services still own slow effective-style selection"
        );
    }

    #[test]
    fn layout_object_gap_queries_preserve_algorithm_attribution() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_object_gap_queries.rs"))
                .expect("read source LayoutObject gap-query owner");
        for owned_policy in [
            "fn StitchedRowGapIndexBase",
            "fn StitchedRowGapIndex(",
            "DispatchLayoutObjectStitchedRowGapIndex",
            "PhysicalBoxFragment",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment gap query lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "generic gap dispatch must not hide the concrete algorithm frame"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_gap_queries.rs"),
            "LayoutObject must load gap queries from layout_fragment"
        );
        for leaked_policy in ["fn StitchedRowGapIndexBase", "fn StitchedRowGapIndex("] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns gap query {leaked_policy}"
            );
        }

        for (package, source) in [
            (
                "flex",
                concat!("layout_algo_", "flex/src/layout_flexible_box.rs"),
            ),
            ("grid", concat!("layout_algo_", "grid/src/layout_grid.rs")),
        ] {
            let algorithm = std::fs::read_to_string(root.join(source))
                .unwrap_or_else(|_| panic!("read {package} stitched-gap owner"));
            assert!(
                algorithm.contains("#[inline(never)]\n    pub fn StitchedRowGapIndex"),
                "{package} stitched-gap implementation must retain an algorithm sampling frame"
            );
        }
    }

    #[test]
    fn layout_box_fragment_access_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_box_fragment_access.rs"))
                .expect("read source LayoutBox fragment-access owner");
        for owned_policy in [
            "pub fn GetLayoutResult",
            "pub fn InvalidateCachedGeometry",
            "pub fn GetPhysicalFragment",
            "pub fn PhysicalFragmentCount",
            "pub fn HasInlineFragments",
            "pub fn ClearFirstInlineFragmentItemIndex",
            "pub fn SetFirstInlineFragmentItemIndex",
            "pub fn OutOfFlowInsetsForGetComputedStyle",
            "pub fn NeedsAnchorPositionScrollAdjustmentInX",
            "pub fn NeedsAnchorPositionScrollAdjustmentInY",
            "pub fn FirstInlineFragmentItemIndex",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutBox access lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "constant-time fragment access and debug-only consistency checks must remain inlineable"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_box_fragment_access.rs"),
            "LayoutBox must load fragment result access from layout_fragment"
        );
        for leaked_policy in [
            "pub fn GetLayoutResult(&self",
            "pub fn GetPhysicalFragment(&self",
            "pub fn PhysicalFragmentCount(&self",
            "pub fn OutOfFlowInsetsForGetComputedStyle(&self",
            "pub fn NeedsAnchorPositionScrollAdjustmentInX(&self",
            "pub fn NeedsAnchorPositionScrollAdjustmentInY(&self",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutBox core still owns fragment access policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_inline_fragment_state_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_inline_fragment_state.rs"),
        )
        .expect("read source LayoutObject inline-fragment-state owner");
        for owned_policy in [
            "fn IsFragmentLessBoxBase",
            "fn IsFragmentLessBox(",
            "fn HasInlineFragmentsBase",
            "fn HasInlineFragments(",
            "fn FirstInlineFragmentItemIndexBase",
            "fn FirstInlineFragmentItemIndex(",
            "fn ClearFirstInlineFragmentItemIndexBase",
            "fn ClearFirstInlineFragmentItemIndex(",
            "fn SetFirstInlineFragmentItemIndexBase",
            "fn SetFirstInlineFragmentItemIndex(",
            "DispatchLayoutObjectHasInlineFragments",
            "DispatchLayoutObjectIsFragmentLessBox",
            "DispatchLayoutObjectSetFirstInlineFragmentItemIndex",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment inline-fragment state lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "per-object inline-fragment bookkeeping must remain inlineable"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_inline_fragment_state.rs"),
            "LayoutObject must load inline-fragment state from layout_fragment"
        );
        for leaked_policy in [
            "fn IsFragmentLessBoxBase",
            "fn IsFragmentLessBox(",
            "fn HasInlineFragmentsBase",
            "fn HasInlineFragments(",
            "fn FirstInlineFragmentItemIndexBase",
            "fn FirstInlineFragmentItemIndex(",
            "fn ClearFirstInlineFragmentItemIndexBase",
            "fn ClearFirstInlineFragmentItemIndex(",
            "fn SetFirstInlineFragmentItemIndexBase",
            "fn SetFirstInlineFragmentItemIndex(",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns inline-fragment state {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_layer_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_object_layer_queries.rs"),
        )
        .expect("read source LayoutObject layer-query owner");
        for owned_policy in [
            "fn HasDistortingVisualEffects",
            "fn HasNonZeroEffectiveOpacity",
            "requires source LocalBorderBoxProperties",
            "fn EnclosingLayer",
            "fn PaintingLayer(",
            "fn PaintingLayerDefault",
            "current_ref.HasLayer()",
            "current_ref.ContainerForColumnSpanner()",
            "current_ref.ContainingBlock() as *mut LayoutObject",
            "IsSelfPaintingLayer()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment LayoutObject layer query lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            2,
            "enclosing and painting-layer ancestry queries must retain separate fragment frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_object_layer_queries.rs"),
            "LayoutObject must load layer queries from layout_fragment"
        );
        for leaked_policy in [
            "fn HasDistortingVisualEffects",
            "fn HasNonZeroEffectiveOpacity",
            "fn EnclosingLayer",
            "fn PaintingLayer(",
            "fn PaintingLayerDefault",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns layer query {leaked_policy}"
            );
        }
    }

    #[test]
    fn transform_change_admission_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/transform_invalidation.rs"))
                .expect("read source transform invalidation owner");
        assert!(
            owner.contains("#[inline(never)]")
                && owner.contains("fn LayoutBoxTransformsChangeMayRequireLayout")
                && owner.contains("PhysicalFragments")
                && owner.contains("HasAnchorsToPropagate"),
            "layout_invalidation must own retained-anchor transform admission"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_invalidation/src/transform_invalidation.rs")
                && facade.contains("LayoutBoxTransformsChangeMayRequireLayout(self)"),
            "LayoutBox must delegate transform admission to layout_invalidation"
        );
        assert!(
            !facade.contains("fragment.HasAnchorsToPropagate()"),
            "LayoutBox still owns transform-change invalidation admission"
        );
    }

    #[test]
    fn layout_box_fragment_retirement_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_box_fragment_retirement.rs"),
        )
        .expect("read source LayoutBox fragment retirement owner");
        for owned_policy in [
            "fn LayoutBoxDisassociatePhysicalFragments",
            "FragmentItems::LayoutObjectWillBeDestroyed",
            "measure_cache }.LayoutObjectWillBeDestroyed",
            "layout_results_.iter()",
            ".LayoutObjectWillBeDestroyed();",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation fragment retirement lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "fragment retirement must retain an invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_box.rs"))
            .expect("read LayoutBox facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_box_fragment_retirement.rs")
                && facade.contains("LayoutBoxDisassociatePhysicalFragments(self)"),
            "LayoutBox must delegate fragment retirement to layout_invalidation"
        );
        for leaked_policy in [
            "FragmentItems::LayoutObjectWillBeDestroyed(self)",
            "GetPhysicalFragment()\n                .LayoutObjectWillBeDestroyed()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox still owns fragment retirement policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_model_lifecycle_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_box_model_lifecycle.rs"),
        )
        .expect("read source LayoutBoxModelObject lifecycle owner");
        for owned_policy in [
            "fn NeedsAnchorPositionScrollData",
            "fn WillBeDestroyed",
            "fn StyleWillChange",
            "fn StyleDidChange",
            "DisposeScrollableAreaForPaintLayer",
            "DestroyLayer",
            "DispatchLayoutBoxModelUpdateFromStyle",
            "UpdateTransformAfterStyleChange",
            "UpdateLayerAfterStyleChange",
            "EnsureAnchorPositionScrollData",
            "RemoveAnchorPositionScrollData",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation LayoutBoxModelObject lifecycle lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "destruction and completed style update must retain invalidation sampling frames"
        );

        let loader = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_box_model_lifecycle.rs"),
        )
        .expect("read LayoutBoxModelObject lifecycle loader");
        assert!(
            loader.contains("layout_invalidation/src/layout_box_model_lifecycle.rs"),
            "LayoutBoxModelObject lifecycle must load from layout_invalidation"
        );
        for leaked_policy in [
            "fn NeedsAnchorPositionScrollData",
            "fn WillBeDestroyed",
            "fn StyleDidChange",
            "DispatchLayoutBoxModelUpdateFromStyle",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "core LayoutBoxModelObject lifecycle still owns {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_node_style_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/layout_node_style.rs"))
                .expect("read source layout-node style owner");
        for owned_policy in [
            "fn ComputeIsFixedContainer",
            "fn ComputeIsAbsoluteContainer",
            "fn ShouldBeHandledAsInline",
            "fn ShouldBeHandledAsFloating",
            "fn UpdateFromStyle",
            "fn ComputeOverflowClipAxes",
            "SetCanContainFixedPositionObjects",
            "SetCanContainAbsolutePositionObjects",
            "SetHasTransformRelatedProperty",
            "SetHasNonVisibleOverflow",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source layout-node style lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("UpdateScrollableAreaForLayout"),
            "LayoutBox::UpdateFromStyle must not preempt PaintLayer::UpdateScrollableArea"
        );
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            2,
            "box-model and box UpdateFromStyle stages must retain separate invalidation frames"
        );

        let loader =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_node_style.rs"))
                .expect("read source layout-node style loader");
        assert!(
            loader.contains("layout_invalidation/src/layout_node_style.rs"),
            "source layout-node style must load from layout_invalidation"
        );
        for leaked_policy in [
            "fn ComputeIsFixedContainer",
            "fn ShouldBeHandledAsInline",
            "fn UpdateFromStyle",
            "fn ComputeOverflowClipAxes",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "core source layout-node style still owns {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_box_style_change_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_box_style_change.rs"),
        )
        .expect("read source LayoutBox style-change owner");
        for owned_policy in [
            "fn LayoutBoxStyleWillChange",
            "fn LayoutBoxStyleDidChange",
            "MarkContainerChainForLayout",
            "GridColumnStart",
            "SetGridPlacementDirty",
            "ComputeOverflowClipAxes",
            "ScrollContainerStatusChanged",
            "SetNeedsCollectInlines",
            "invalidate_column_spanners",
            "TransformsChangeMayRequireLayout",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation LayoutBox style change lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "pre/post LayoutBox style-change decisions must retain invalidation sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_lifecycle.rs"))
                .expect("read LayoutBox lifecycle facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_box_style_change.rs")
                && facade.contains("LayoutBoxStyleWillChange(self,")
                && facade.contains("LayoutBoxStyleDidChange(self,"),
            "LayoutBox lifecycle must delegate style invalidation to layout_invalidation"
        );
        for leaked_policy in [
            "GridColumnStart()",
            "SetGridPlacementDirty(true)",
            "invalidate_column_spanners",
            "TransformsChangeMayRequireLayout()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox lifecycle still owns style invalidation {leaked_policy}"
            );
        }
    }

    #[test]
    fn derived_block_style_changes_resolve_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let block_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_block_style_change.rs"),
        )
        .expect("read source LayoutBlock style-change owner");
        for owned_policy in [
            "fn LayoutBlockStyleDidChange",
            "HasSVGTextDescendants",
            "svg_block_text_scale",
            "RemovePositionedObjects",
            "PropagateStyleToAnonymousChildren",
            "update_svg_block_style",
        ] {
            assert!(
                block_owner.contains(owned_policy),
                "layout_invalidation LayoutBlock style change lost {owned_policy}"
            );
        }
        assert!(
            block_owner.contains("#[inline(never)]"),
            "LayoutBlock style changes must retain an invalidation sampling frame"
        );

        let flow_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_block_flow_style_change.rs"),
        )
        .expect("read source LayoutBlockFlow style-change owner");
        for owned_policy in [
            "fn LayoutBlockFlowStyleDidChange",
            "UpdateForMulticol",
            "kPseudoIdColumn",
            "needs_reshape",
            "update_inline_style",
        ] {
            assert!(
                flow_owner.contains(owned_policy),
                "layout_invalidation LayoutBlockFlow style change lost {owned_policy}"
            );
        }
        assert!(
            flow_owner.contains("#[inline(never)]"),
            "LayoutBlockFlow style changes must retain an invalidation sampling frame"
        );

        let block_facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block.rs")),
        )
        .expect("read source LayoutBlock owner");
        let flow_facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block_flow.rs")),
        )
        .expect("read source LayoutBlockFlow owner");
        assert!(
            block_facade.contains("layout_invalidation/src/layout_block_style_change.rs")
                && block_facade.contains("LayoutBlockStyleDidChange(self,"),
            "LayoutBlock must delegate style invalidation to layout_invalidation"
        );
        assert!(
            flow_facade.contains("layout_invalidation/src/layout_block_flow_style_change.rs")
                && flow_facade.contains("LayoutBlockFlowStyleDidChange(self,"),
            "LayoutBlockFlow must delegate style invalidation to layout_invalidation"
        );
        for leaked_policy in [
            "svg_block_text_scale.unwrap()",
            "PropagateStyleToAnonymousChildren();",
        ] {
            assert!(
                !block_facade.contains(leaked_policy),
                "LayoutBlock still owns style invalidation {leaked_policy}"
            );
        }
        for leaked_policy in ["UpdateForMulticol();", "update_inline_style(self"] {
            assert!(
                !flow_facade.contains(leaked_policy),
                "LayoutBlockFlow still owns style invalidation {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_style_update_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_style_update.rs"),
        )
        .expect("read source LayoutObject style-update owner");
        for owned_policy in [
            "fn SetStyle(",
            "fn SetStyleWithChanges",
            "fn SetPseudoElementStyle(",
            "fn SetPseudoElementStyleWithMatchParentSize",
            "fn StyleWillChange",
            "Visibility()",
            "DirtyVisibleContentStatus",
            "fn StyleDidChange",
            "fn transform_style_difference",
            "fn AnonymousHasStylePropagationOverrideBase",
            "fn AnonymousHasStylePropagationOverride(",
            "fn UpdateAnonymousChildStyleBase",
            "fn UpdateAnonymousChildStyle(",
            "fn AdjustStyleDifference",
            "fn ApplyPseudoElementStyleChanges",
            "fn ApplyFirstLineChanges",
            "fn LayoutObjectSetStyleWithChanges",
            "DispatchLayoutObjectUpdateAnonymousChildStyle",
            "DispatchLayoutObjectAnonymousHasStylePropagationOverride",
            "LayoutObjectAdjustStyleDifferenceProvider",
            "LayoutObjectApplyPseudoElementStyleChangesProvider",
            "LayoutObjectApplyFirstLineChangesProvider",
            "OffsetPath",
            "HasCurrentTransformAnimation",
            "set_transform_data_changed",
            "set_only_transform_property_changed",
            "DispatchLayoutObjectStyleWillChange",
            "UpdateMaskImageObservers",
            "DispatchLayoutObjectStyleDidChange",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation LayoutObject style update lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 7,
            "transform classification, style and pseudo transactions, anonymous-child dispatch, difference adjustment, and pseudo/first-line application must retain invalidation sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_object_style_update.rs"),
            "LayoutObject core services must load style transactions from layout_invalidation"
        );
        for leaked_policy in [
            "fn SetStyle(",
            "fn SetStyleWithChanges",
            "fn SetPseudoElementStyle(",
            "fn SetPseudoElementStyleWithMatchParentSize",
            "fn StyleWillChange",
            "fn StyleDidChange",
            "fn TransformStyleDifference(",
            "set_transform_data_changed(",
            "DispatchLayoutObjectStyleWillChange(self,",
            "UpdateMaskImageObservers(old_style, style)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject core services still own style-update policy {leaked_policy}"
            );
        }

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        for leaked_policy in [
            "fn AnonymousHasStylePropagationOverrideBase",
            "fn AnonymousHasStylePropagationOverride(",
            "fn UpdateAnonymousChildStyleBase",
            "fn UpdateAnonymousChildStyle(",
            "fn AdjustStyleDifference",
            "fn ApplyPseudoElementStyleChanges",
            "fn ApplyFirstLineChanges",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns style-update entry {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_style_services_resolve_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_style_services.rs"),
        )
        .expect("read source LayoutObject style-service owner");
        for owned_policy in [
            "fn UpdateFromElementBase",
            "fn UpdateFromElement(",
            "fn FirstLineStyleWithoutFallback",
            "fn GetCachedPseudoElementStyle",
            "fn GetUncachedPseudoElementStyle",
            "fn SetModifiedStyleOutsideStyleRecalc",
            "DispatchLayoutObjectUpdateFromElement",
            "LayoutObjectGetUncachedPseudoElementStyleProvider",
            "LayoutObjectSetModifiedStyleOutsideStyleRecalcProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation style service lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            3,
            "element refresh, uncached pseudo resolution, and explicit style mutation must retain invalidation frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_object_style_services.rs"),
            "LayoutObject must load style services from layout_invalidation"
        );
        for leaked_policy in [
            "fn UpdateFromElementBase",
            "fn UpdateFromElement(",
            "fn FirstLineStyleWithoutFallback",
            "fn GetCachedPseudoElementStyle",
            "fn GetUncachedPseudoElementStyle",
            "fn SetModifiedStyleOutsideStyleRecalc",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns style service {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_prepaint_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_prepaint_invalidation.rs"),
        )
        .expect("read source LayoutObject prepaint-invalidation owner");
        for owned_policy in [
            "fn EffectiveAllowedTouchAction",
            "fn HasEffectiveAllowedTouchAction",
            "fn SetShouldDelayFullPaintInvalidation",
            "fn SetShouldInheritContainerTimingRoot",
            "fn ClearPaintFlags",
            "fn EnsureIsReadyForPaintInvalidation",
            "fn InvalidateSubtreePositionTry",
            "fn SetShouldInvalidatePaintForHitTest",
            "fn ClearPaintInvalidationFlags",
            "fn SetMayNeedPaintInvalidationAnimatedBackgroundImage",
            "fn ClearShouldDelayFullPaintInvalidation",
            "fn SetShouldInvalidateSelection",
            "fn InvalidateSelectedChildrenOnStyleChange",
            "fn InvalidateSelectionOnStyleChange",
            "fn SetNeedsPrePaintSubtreeWalk",
            "fn SetDescendantNeedsPrePaintSubtreeWalk",
            "fn MarkEffectiveAllowedTouchActionChanged",
            "fn MarkBlockingWheelEventHandlerChanged",
            "fn MarkSoftNavigationContextChanged",
            "fn MarkContainerTimingChanged",
            "fn MaybeClearIsScrollAnchorObject",
            "fn SetSVGDescendantMayHaveTransformRelatedOperations",
            "fn UpdateAfterReinsert",
            "fn MarkSelfPaintingLayerForVisualOverflowRecalc",
            "LayoutObjectSetNeedsPrePaintSubtreeWalkProvider",
            "LayoutObjectEnsureIsReadyForPaintInvalidationProvider",
            "LayoutObjectInvalidateSelectedChildrenOnStyleChangeProvider",
            "LayoutObjectUpdateAfterReinsertProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation prepaint service lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            14,
            "position, selection, prepaint, interaction, anchor, SVG, reinsert, and layer propagation must retain invalidation frames"
        );
        assert!(
            !owner.contains("LayoutObjectMarkSelfPaintingLayerForVisualOverflowRecalcProvider",),
            "visual-overflow layer marking must be implemented directly by layout_invalidation",
        );

        let reasons = std::fs::read_to_string(
            root.join("layout_invalidation/src/pre_paint_subtree_walk_reasons.rs"),
        )
        .expect("read prepaint subtree-walk reason owner");
        assert!(
            reasons.contains("pub enum PrePaintSubtreeWalkReason")
                && reasons.contains("pub struct PrePaintSubtreeWalkReasons")
                && reasons.contains("pub const fn CrossFramePrePaintSubtreeWalkReasons")
                && reasons.contains("core/paint/pre_paint_subtree_walk_reasons.h")
                && !reasons.contains("layoutng/internal/pre_paint_subtree_walk_reasons.h")
                && !reasons.contains("#[inline(never)]"),
            "prepaint reason bit operations must stay inlineable in layout_invalidation and cite their real core/paint source"
        );
        let reasons_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/pre_paint_subtree_walk_reasons.rs"),
        )
        .expect("read prepaint subtree-walk reason facade");
        assert!(
            reasons_facade.contains("layout_invalidation/src/pre_paint_subtree_walk_reasons.rs",)
                && reasons_facade
                    .contains("pub use layout_invalidation_pre_paint_subtree_walk_reasons::*",)
                && !reasons_facade.contains("pub enum PrePaintSubtreeWalkReason"),
            "prepaint walk reasons must be re-exported from layout_invalidation"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_object_prepaint_invalidation.rs"),
            "LayoutObject must load prepaint invalidation from layout_invalidation"
        );
        for leaked_policy in [
            "fn EffectiveAllowedTouchAction",
            "fn HasEffectiveAllowedTouchAction",
            "LayoutObjectSetShouldDelayFullPaintInvalidationProvider(self)",
            "LayoutObjectSetShouldInheritContainerTimingRootProvider(self, inherit)",
            "self.ClearPaintInvalidationFlags();",
            "LayoutObjectEnsureIsReadyForPaintInvalidationProvider(self)",
            "fn InvalidateSubtreePositionTry",
            "fn SetShouldInvalidatePaintForHitTest",
            "fn ClearPaintInvalidationFlags",
            "fn SetMayNeedPaintInvalidationAnimatedBackgroundImage",
            "fn ClearShouldDelayFullPaintInvalidation",
            "fn SetShouldInvalidateSelection",
            "fn InvalidateSelectedChildrenOnStyleChange",
            "fn InvalidateSelectionOnStyleChange",
            "fn SetNeedsPrePaintSubtreeWalk",
            "fn SetDescendantNeedsPrePaintSubtreeWalk",
            "fn MarkEffectiveAllowedTouchActionChanged",
            "fn MarkBlockingWheelEventHandlerChanged",
            "fn MarkSoftNavigationContextChanged",
            "fn MarkContainerTimingChanged",
            "fn MaybeClearIsScrollAnchorObject",
            "fn SetSVGDescendantMayHaveTransformRelatedOperations",
            "fn UpdateAfterReinsert",
            "fn MarkSelfPaintingLayerForVisualOverflowRecalc",
            "LayoutObjectMarkSelfPaintingLayerForVisualOverflowRecalcProvider",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns prepaint invalidation {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_display_lock_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_display_lock.rs"),
        )
        .expect("read source LayoutObject display-lock owner");
        for owned_policy in [
            "fn ChildLayoutBlockedByDisplayLock",
            "fn ChildPrePaintBlockedByDisplayLock",
            "fn ChildPaintBlockedByDisplayLock",
            "fn HasLockedAncestorPreventingPaint",
            "fn GetDisplayLockContext",
            "ContentLockBlocksChildLayout",
            "ContentLockBlocksChildPrePaint",
            "ContentLockBlocksChildPaint",
            "LayoutObjectGetDisplayLockContextProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation display-lock query lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "fixed-cost display-lock admission must remain inlineable"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_object_display_lock.rs"),
            "LayoutObject must load display-lock queries from layout_invalidation"
        );
        for leaked_policy in [
            "fn ChildLayoutBlockedByDisplayLock",
            "fn ChildPrePaintBlockedByDisplayLock",
            "fn ChildPaintBlockedByDisplayLock",
            "fn GetDisplayLockContext",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns display-lock query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_tree_lifecycles_resolve_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let object_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_tree_lifecycle.rs"),
        )
        .expect("read source LayoutObject lifecycle owner");
        for owned_policy in [
            "fn InsertedIntoTree",
            "fn InsertedIntoTreeBase",
            "fn WillBeRemovedFromTree",
            "fn WillBeRemovedFromTreeBase",
            "fn WillBeDestroyed",
            "fn LayoutObjectInsertedIntoTreeBase",
            "fn LayoutObjectWillBeRemovedFromTreeBase",
            "fn LayoutObjectWillBeDestroyed",
            "DirtyLinesFromChangedChild",
            "InputMayBeImplicitAnchor",
            "MarkMayContainAnchor",
            "DestroyLeftoverChildren",
        ] {
            assert!(
                object_owner.contains(owned_policy),
                "layout_invalidation LayoutObject lifecycle lost {owned_policy}"
            );
        }
        assert!(
            object_owner.matches("#[inline(never)]").count() >= 5,
            "object lifecycle dispatch and base work must retain invalidation sampling frames"
        );

        let box_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_box_tree_lifecycle.rs"),
        )
        .expect("read source LayoutBox lifecycle owner");
        for owned_policy in [
            "fn LayoutBoxWillBeDestroyed",
            "fn LayoutBoxInsertedIntoTree",
            "fn LayoutBoxWillBeRemovedFromTree",
            "ShapeOutsideInfo::RemoveInfo",
            "DisassociatePhysicalFragments",
            "AddCustomLayoutChildIfNeeded",
            "ClearCustomLayoutChild",
        ] {
            assert!(
                box_owner.contains(owned_policy),
                "layout_invalidation LayoutBox lifecycle lost {owned_policy}"
            );
        }
        assert!(
            box_owner.matches("#[inline(never)]").count() >= 3,
            "box insertion, removal and destruction must retain invalidation sampling frames"
        );

        let object_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services facade");
        let box_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_lifecycle.rs"))
                .expect("read LayoutBox lifecycle facade");
        assert!(
            object_facade.contains("layout_invalidation/src/layout_object_tree_lifecycle.rs"),
            "LayoutObject core services must load lifecycle policy from layout_invalidation"
        );
        assert!(
            box_facade.contains("layout_invalidation/src/layout_box_tree_lifecycle.rs")
                && box_facade.contains("LayoutBoxWillBeDestroyed(self)")
                && box_facade.contains("LayoutBoxInsertedIntoTree(self)"),
            "LayoutBox lifecycle must delegate tree policy to layout_invalidation"
        );
        for leaked_policy in [
            "fn InsertedIntoTree",
            "fn InsertedIntoTreeBase",
            "fn WillBeRemovedFromTree",
            "fn WillBeRemovedFromTreeBase",
            "fn WillBeDestroyed",
            "InputMayBeImplicitAnchor()",
            "DestroyLeftoverChildren()",
        ] {
            assert!(
                !object_facade.contains(leaked_policy),
                "LayoutObject core services still own lifecycle policy {leaked_policy}"
            );
        }
        for leaked_policy in [
            "ShapeOutsideInfo::RemoveInfo(self)",
            "AddCustomLayoutChildIfNeeded();",
        ] {
            assert!(
                !box_facade.contains(leaked_policy),
                "LayoutBox lifecycle still owns tree policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_tree_topology_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/layout_object_tree.rs"))
                .expect("read LayoutObject tree-topology owner");
        for owned_policy in [
            "pub type IndexCache",
            "fn GeneratingNode",
            "fn EnclosingNode",
            "fn View",
            "fn IsRooted",
            "fn IsDescendantOf",
            "fn CommonAncestor",
            "fn IsBeforeInPreOrder",
            "LayoutObjectGeneratingNodeProvider",
            "LayoutObjectEnclosingNodeProvider",
            "while !child.is_null()",
            "while !forward.is_null() && !backward.is_null()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutObject topology owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 7,
            "node resolution, root-view, attachment, descendant, common-ancestor, and preorder ordering scans must retain invalidation sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_tree.rs"))
                .expect("read LayoutObject tree-topology facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_object_tree.rs")
                && facade.contains("pub use layout_invalidation_layout_object_tree::*;")
                && !facade.contains("fn IsBeforeInPreOrder"),
            "LayoutObject topology must be re-exported from layout_invalidation"
        );
        let core = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core services");
        assert!(
            !core.contains("fn IsRooted"),
            "root attachment traversal must not return to LayoutObject core services"
        );
        let object_core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
                .expect("read LayoutObject compatibility module");
        for leaked_policy in ["fn GeneratingNode", "fn EnclosingNode"] {
            assert!(
                !object_core.contains(leaked_policy),
                "LayoutObject core still owns topology entry {leaked_policy}"
            );
        }
        let hot_core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_hot.rs"))
                .expect("read LayoutObject hot compatibility module");
        assert!(
            !hot_core.contains("fn View"),
            "LayoutObject hot core still owns root-view ancestry traversal"
        );
    }

    #[test]
    fn layout_object_child_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_child_list.rs"),
        )
        .expect("read LayoutObject child-mutation owner");
        for owned_policy in [
            "pub struct LayoutObjectChildList",
            "fn InvalidateInlineItems",
            "fn DestroyLeftoverChildren",
            "fn RemoveChildNode",
            "fn InsertChildNode",
            "RegisterSubtreeChangeListenerOnDescendants",
            "SetNeedsLayoutAndIntrinsicWidthsRecalc",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutObject child-mutation owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "destruction, insertion, removal, and recursive inline invalidation must retain invalidation sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_child_list.rs"))
                .expect("read LayoutObject child-list facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_object_child_list.rs")
                && facade.contains("pub use layout_invalidation_layout_object_child_list::*;")
                && !facade.contains("fn InsertChildNode"),
            "LayoutObject child mutation must be re-exported from layout_invalidation"
        );
    }

    #[test]
    fn layout_object_child_dispatch_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_child_mutation.rs"),
        )
        .expect("read source LayoutObject child-dispatch owner");
        for owned_policy in [
            "fn AddChildDefault",
            "fn AddChild(",
            "fn AddChildBase",
            "fn RemoveChildBase",
            "fn RemoveChild(",
            "DispatchLayoutObjectAddChild",
            "DispatchLayoutObjectRemoveChild",
            "insert_table_child",
            "insert_inline_text_child",
            "TransformAndSecureOriginalText",
            "LayoutBoxModelObject::AttemptToMerge",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation child dispatch lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            4,
            "derived and base add/remove stages must retain separate invalidation sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core-services facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_object_child_mutation.rs"),
            "LayoutObject core services must load child mutation from layout_invalidation"
        );
        for leaked_policy in [
            "fn AddChildDefault",
            "fn AddChild(",
            "fn AddChildBase",
            "fn RemoveChildBase",
            "fn RemoveChild(",
            "insert_table_child",
            "insert_inline_text_child",
            "TransformAndSecureOriginalText",
            "LayoutBoxModelObject::AttemptToMerge",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject core services still own child mutation {leaked_policy}"
            );
        }
    }

    #[test]
    fn box_model_tree_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_box_model_tree_mutation.rs"),
        )
        .expect("read box-model tree-mutation owner");
        for owned_policy in [
            "fn MoveChildTo",
            "fn MoveChildrenToBase",
            "fn SplitAnonymousBoxesAroundChild",
            "fn AttemptToMerge",
            "RemoveChildNode",
            "InsertChildNode",
            "SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation",
        ] {
            assert!(
                owner.contains(owned_policy),
                "box-model tree-mutation owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "move, range move, split, and recursive merge must retain invalidation sampling frames"
        );

        let core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_model_tree.rs"))
                .expect("read box-model tree compatibility module");
        assert!(
            core.contains("layout_invalidation/src/layout_box_model_tree_mutation.rs")
                && core.contains("fn ComputedPaddingOutsets")
                && !core.contains("fn SplitAnonymousBoxesAroundChild")
                && !core.contains("fn MoveChildrenToBase"),
            "box-model structural mutation must load from layout_invalidation while fixed-cost value queries stay in core"
        );
    }

    #[test]
    fn hot_object_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_hot_invalidation.rs"),
        )
        .expect("read hot LayoutObject invalidation owner");
        for owned_policy in [
            "fn MarkContainerChainForOverflowRecalcIfNeeded",
            "fn SetNeedsOverflowRecalcWithType",
            "self.SetShouldCheckForPaintInvalidation();",
            "self.MarkSelfPaintingLayerForVisualOverflowRecalc();",
            "fn PropagateStyleToAnonymousChildren",
            "SetChildNeedsScrollableOverflowRecalc",
            "CreateAnonymousStyleBuilderWithDisplay",
            "NextInPreOrderAfterChildren",
        ] {
            assert!(
                owner.contains(owned_policy),
                "hot LayoutObject invalidation owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "overflow ancestor propagation and anonymous style traversal must retain invalidation sampling frames"
        );

        let core = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_hot.rs"))
            .expect("read hot LayoutObject core module");
        assert!(
            core.contains("layout_invalidation/src/layout_object_hot_invalidation.rs")
                && core.contains("fn FirstLineStyleRef")
                && !core.contains("fn View")
                && !core.contains("fn PropagateStyleToAnonymousChildren")
                && !core.contains("fn MarkContainerChainForOverflowRecalcIfNeeded"),
            "tree-sized hot invalidation must load from layout_invalidation while fixed-cost accessors stay in core"
        );
    }

    #[test]
    fn box_model_layer_lifecycle_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_box_model_layer_invalidation.rs"),
        )
        .expect("read box-model layer invalidation owner");
        for owned_policy in [
            "fn DestroyLayer",
            "fn UpdateLayerAfterStyleChange",
            "fn CreateLayerAfterStyleChange",
            "PaintLayer::new",
            "UpdateSelfPaintingLayer",
            "SetNeedsPaintPropertyUpdate",
        ] {
            assert!(
                owner.contains(owned_policy),
                "box-model layer invalidation owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "layer create, update, and destroy must retain invalidation sampling frames"
        );

        let core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_model_object.rs"))
                .expect("read LayoutBoxModelObject core record");
        assert!(
            core.contains("layout_invalidation/src/layout_box_model_layer_invalidation.rs")
                && core.contains("fn LayerTypeRequired")
                && !core.contains("fn UpdateLayerAfterStyleChange")
                && !core.contains("fn CreateLayerAfterStyleChange")
                && !core.contains("fn DestroyLayer"),
            "layer lifecycle must load from layout_invalidation while classification stays in core"
        );
    }

    #[test]
    fn layout_object_container_queries_resolve_to_the_block_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/layout_object_containers.rs"
        )))
        .expect("read LayoutObject container-query owner");
        for owned_policy in [
            "fn FindColumnSpannerContainer",
            "fn FindAncestorByPredicate",
            "fn ScrollParent(",
            "fn ScrollParentDefault",
            "fn OffsetParent(",
            "fn OffsetParentDefault",
            "fn IsContainedBy",
            "fn IsRenderedLegendInternal",
            "fn EnclosingBox",
            "fn ContainerForAbsolutePositionWithSkipInfo",
            "fn ContainerForFixedPositionWithSkipInfo",
            "fn ContainingFragmentationContextRoot",
            "fn ContainingBlockWithSkipInfo",
            "LayoutObjectScrollParentProvider",
            "LayoutObjectOffsetParentProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block container-query owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 9,
            "scroll/offset parent, containment, legend, enclosing-box, absolute, fixed, spanner, and fragmentation-root scans must retain block sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_containers.rs"))
                .expect("read LayoutObject container-query loader");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "block/src/layout_object_containers.rs"
            )) && !facade.contains("fn ContainerForAbsolutePositionWithSkipInfo"),
            "LayoutObject container-query loader must not regain block policy"
        );
        let core = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core services");
        for leaked_policy in ["fn IsContainedBy", "fn IsRenderedLegendInternal"] {
            assert!(
                !core.contains(leaked_policy),
                "LayoutObject core services still own block query {leaked_policy}"
            );
        }
        let object_core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
                .expect("read LayoutObject compatibility module");
        for leaked_policy in [
            "fn ScrollParent(",
            "fn ScrollParentDefault",
            "fn OffsetParent(",
            "fn OffsetParentDefault",
        ] {
            assert!(
                !object_core.contains(leaked_policy),
                "LayoutObject core still owns block query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_root_layout_resolves_to_the_block_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/layout_view_root_layout.rs"
        )))
        .expect("read LayoutView root-layout owner");
        for owned_policy in [
            "#[inline(never)]",
            "fn ComputeMinimumWidth",
            "fn LayoutRoot",
            "ConstraintSpaceBuilder::new_without_parent_space",
            "ComputeMinMaxSizes",
            "SetContainsAnnotations",
            "BlockNode::new",
            ".Layout(",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block root-layout owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "root intrinsic sizing and root layout must retain separate block sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "block/src/layout_view_root_layout.rs"
            )) && !loader.contains("fn LayoutRoot")
                && !loader.contains("fn ComputeMinimumWidth"),
            "LayoutView root layout must load from the block algorithm owner"
        );
    }

    #[test]
    fn layout_view_block_queries_resolve_to_the_block_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "block/src/layout_view_block_queries.rs"
        )))
        .expect("read LayoutView block-query owner");
        for owned_policy in [
            "fn RootBox",
            "fn IsFragmentationContextRoot",
            "fn ShouldUsePaginatedLayout",
            "fn CreateAnonymousPageLayoutObject",
            "fn NamedPageAtIndex",
            "LayoutViewCreateAnonymousPageLayoutObjectProvider",
            "LayoutViewNamedPageAtIndexProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "block root-query owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "page allocation and lookup providers must retain separate block sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains(concat!(
                "layout_algo_",
                "block/src/layout_view_block_queries.rs"
            )),
            "LayoutView must load pagination queries from the block owner"
        );
        for leaked_policy in [
            "fn RootBox",
            "fn IsFragmentationContextRoot",
            "fn ShouldUsePaginatedLayout",
            "fn CreateAnonymousPageLayoutObject",
            "fn NamedPageAtIndex",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns block query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_geometry_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_view_geometry.rs"))
                .expect("read LayoutView viewport-geometry owner");
        for owned_policy in [
            "fn IsBeingAutoSized",
            "fn InitialContainingBlockSize",
            "fn DocumentRect",
            "fn GetLayoutSize",
            "fn GetNonPrintingLayoutSize",
            "fn ViewLogicalHeightForPercentages",
            "fn ViewRect",
            "fn OverflowClipRectInternal",
            "fn CalculateScrollbarModes",
            "fn SmallViewportSizeForViewportUnits",
            "fn PaginationViewportSizeForMediaQueries",
            "fn BackgroundRect",
            "ExcludeScrollbars",
            "SmallerViewportUnitsEnabled",
        ] {
            assert!(
                owner.contains(owned_policy),
                "fragment viewport-geometry owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "layout-size and overflow-clip projection must retain separate fragment sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_view_geometry.rs"),
            "LayoutView must load viewport geometry from the fragment owner"
        );
        for leaked_policy in [
            "fn IsBeingAutoSized",
            "fn InitialContainingBlockSize",
            "fn DocumentRect",
            "fn GetLayoutSize",
            "fn ViewRect",
            "fn OverflowClipRectInternal",
            "fn CalculateScrollbarModes",
            "fn SmallViewportSizeForViewportUnits",
            "fn BackgroundRect",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns fragment geometry {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/layout_view_fragment_queries.rs"),
        )
        .expect("read LayoutView fragment-query owner");
        for owned_policy in [
            "fn HitTestCount",
            "fn HitTestCacheHits",
            "fn HitTest",
            "fn HitTestNoLifecycleUpdate",
            "fn ClearHitTestCache",
            "fn MapToVisualRectInAncestorSpaceInternal",
            "fn QuadsInAncestorInternal",
            "fn UpdateHitTestResult",
            "fn MapAncestorToLocal",
            "fn MapLocalToAncestor",
            "fn GetViewTransitionRoot",
            "LayoutViewHitTestProvider",
            "LayoutViewMapLocalToAncestorProvider",
            "LayoutViewGetViewTransitionRootProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "fragment query owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 7,
            "tree and ancestor projection entries must retain fragment sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_view_fragment_queries.rs"),
            "LayoutView must load fragment queries from layout_fragment"
        );
        for leaked_policy in [
            "fn HitTestCount",
            "fn HitTestCacheHits",
            "fn HitTest(",
            "fn HitTestNoLifecycleUpdate",
            "fn ClearHitTestCache",
            "fn MapToVisualRectInAncestorSpaceInternal",
            "fn QuadsInAncestorInternal",
            "fn UpdateHitTestResult",
            "fn MapAncestorToLocal",
            "fn MapLocalToAncestor",
            "fn GetViewTransitionRoot",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns fragment query {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_paint_policy_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_view_paint_policy.rs"))
                .expect("read LayoutView paint-policy owner");
        for owned_policy in [
            "fn BackgroundIsKnownToBeOpaqueInRect",
            "fn AdditionalCompositingReasons",
            "fn HasTickmarks",
            "fn GetTickmarks",
            "fn LayerTypeRequired",
            "fn CanHaveAdditionalCompositingReasons",
            "fn ComputeCanCompositeBackgroundAttachmentFixed",
        ] {
            assert!(
                owner.contains(owned_policy),
                "fragment root paint policy lost {owned_policy}"
            );
        }
        assert!(
            !owner.contains("#[inline(never)]"),
            "fixed-cost LayoutView paint defaults must remain inlineable"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains("layout_fragment/src/layout_view_paint_policy.rs"),
            "LayoutView must load paint policy from layout_fragment"
        );
        for leaked_policy in [
            "fn BackgroundIsKnownToBeOpaqueInRect",
            "fn AdditionalCompositingReasons",
            "fn HasTickmarks",
            "fn GetTickmarks",
            "fn LayerTypeRequired",
            "fn CanHaveAdditionalCompositingReasons",
            "fn ComputeCanCompositeBackgroundAttachmentFixed",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns fragment paint policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_state_propagation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_state_propagation.rs"),
        )
        .expect("read source LayoutObject state-propagation owner");
        for owned_policy in [
            "fn LayoutObjectDepthSlow",
            "fn NotifyPriorityScrollAnchorStatusChanged",
            "fn LayoutObjectSetDepthIncludingDescendants",
            "fn LayoutObjectSetScrollAnchorDisablingStyleChangedOnAncestor",
            "fn LayoutObjectSetDescendantNeedsPaintPropertyUpdate",
            "LayoutObjectNotifyPriorityScrollAnchorStatusChangedProvider",
            "kMaxLayoutObjectDepth",
            "SlowFirstChild",
            "ViewportDefiningElementForLayout",
            "SetScrollAnchorDisablingStyleChanged",
            "descendant_needs_paint_property_update_",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation object state propagation lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "tree depth, scroll-anchor notification/propagation and paint-property propagation must retain invalidation sampling frames"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_object_state_propagation.rs")
                && facade.contains("LayoutObjectDepthSlow(self)")
                && facade.contains("LayoutObjectSetDepthIncludingDescendants(self, depth)")
                && facade
                    .contains("LayoutObjectSetScrollAnchorDisablingStyleChangedOnAncestor(self)")
                && facade.contains("LayoutObjectSetDescendantNeedsPaintPropertyUpdate(self)"),
            "LayoutObject must delegate state propagation to layout_invalidation"
        );
        for leaked_policy in [
            "fn NotifyPriorityScrollAnchorStatusChanged",
            "let mut extra = 0u32",
            "clamped_depth + 1",
            "let viewport_defining_element =",
            "descendant_needs_paint_property_update_ = true",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutObject still owns state-propagation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_paint_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_paint_invalidation.rs"),
        )
        .expect("read source LayoutObject paint-invalidation owner");
        for owned_policy in [
            "fn DestroyAndCleanupAnonymousWrappers",
            "fn InvalidateVisualOverflow",
            "fn DirtyLinesFromChangedChild",
            "fn InvalidatePaint(",
            "fn InvalidateDisplayItemClients",
            "let mut destroy_root = self as *mut LayoutObject",
            "self.PaintingLayerDefault()",
            "LayoutObjectInvalidatePaintProvider",
            "LayoutObjectInvalidateDisplayItemClientsProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation object paint dispatch lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            4,
            "wrapper cleanup, overflow, paint, and display-item invalidation must retain distinct sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_object_paint_invalidation.rs"),
            "LayoutObject must load paint invalidation from layout_invalidation"
        );
        for leaked_policy in [
            "fn DestroyAndCleanupAnonymousWrappers",
            "fn InvalidateVisualOverflow",
            "fn DirtyLinesFromChangedChild",
            "fn InvalidatePaint(",
            "fn InvalidateDisplayItemClients",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns paint invalidation {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_layer_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_layer_mutation.rs"),
        )
        .expect("read source LayoutObject layer-mutation owner");
        for owned_policy in [
            "fn AddLayers",
            "fn RemoveLayers",
            "fn MoveLayers",
            "fn FindNextLayer(",
            "fn FindNextLayerDefault",
            "LayoutObjectAddLayersProvider",
            "LayoutObjectRemoveLayersProvider",
            "LayoutObjectMoveLayersProvider",
            "LayoutObjectFindNextLayerProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation layer mutation lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            4,
            "add, remove, move, and insertion-point search must retain invalidation sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_object_layer_mutation.rs"),
            "LayoutObject must load layer mutation from layout_invalidation"
        );
        for leaked_policy in [
            "fn AddLayers",
            "fn RemoveLayers",
            "fn MoveLayers",
            "fn FindNextLayer(",
            "fn FindNextLayerDefault",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject core still owns layer mutation {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_image_observers_resolve_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_image_observers.rs"),
        )
        .expect("read source LayoutObject image-observer owner");
        for owned_policy in [
            "fn AddAsImageObserver",
            "fn RemoveAsImageObserver",
            "fn UpdateImage(",
            "fn UpdateShapeImage",
            "fn UpdateFillImages",
            "fn UpdateCursorImages",
            "fn UpdateImageObservers",
            "fn UpdateFirstLineImageObservers",
            "fn UpdateMaskImageObservers",
            "fn LayoutObjectUpdateMaskImageObservers",
            "LayoutObjectUpdateImageObserversProvider",
            "LayoutObjectUpdateFirstLineImageObserversProvider",
            "MaskLayers",
            "FillLayer",
            "BindReceiver",
            "AddClient",
            "RemoveClient",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation image observers lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            9,
            "observer subscription/update providers and mask traversal must retain invalidation sampling frames"
        );
        assert!(
            owner.matches("while ").count() >= 3,
            "mask comparison and observer subscription walks must remain invalidation-owned"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_object_image_observers.rs"),
            "LayoutObject must load image observers from layout_invalidation"
        );
        for leaked_policy in [
            "fn AddAsImageObserver",
            "fn RemoveAsImageObserver",
            "fn UpdateImage(",
            "fn UpdateShapeImage",
            "fn UpdateFillImages",
            "fn UpdateCursorImages",
            "fn UpdateImageObservers",
            "fn UpdateFirstLineImageObservers",
            "fn UpdateMaskImageObservers",
            "let mut a = old_layers",
            "let mut b = new_layers",
            "AddClient(observer)",
            "RemoveClient(observer)",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutObject still owns image-observer policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_block_flow_child_normalization_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_block_flow_child_normalization.rs"),
        )
        .expect("read source LayoutBlockFlow child-normalization owner");
        for owned_policy in [
            "fn get_inline_run",
            "fn LayoutBlockFlowCollapseAnonymousBlockChild",
            "fn LayoutBlockFlowMakeChildrenInlineIfPossible",
            "fn LayoutBlockFlowMakeChildrenNonInline",
            "fn LayoutBlockFlowDirtyLinesFromChangedChild",
            "MoveAllChildrenToBefore",
            "InsertChildNodeDefault",
            "FragmentItems::DirtyLinesFromChangedChild",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation child normalization lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "inline-run discovery and child normalization must retain invalidation sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block_flow.rs")),
        )
        .expect("read source LayoutBlockFlow owner");
        assert!(
            facade.contains("layout_invalidation/src/layout_block_flow_child_normalization.rs")
                && facade.contains("LayoutBlockFlowCollapseAnonymousBlockChild(self, child)")
                && facade.contains("LayoutBlockFlowMakeChildrenInlineIfPossible(self)")
                && facade.contains("LayoutBlockFlowMakeChildrenNonInline(self, insertion_point)")
                && facade.contains("LayoutBlockFlowDirtyLinesFromChangedChild(self, child)"),
            "LayoutBlockFlow must delegate child normalization to layout_invalidation"
        );
        for leaked_policy in [
            "fn get_inline_run(",
            "allows_collapse_anonymous_block_child",
            "blocks_to_remove",
            "FragmentItems::DirtyLinesFromChangedChild",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBlockFlow still owns child-normalization policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_block_flow_child_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_block_flow_child_mutation.rs"),
        )
        .expect("read source LayoutBlockFlow child-mutation owner");
        for owned_policy in [
            "fn is_mergeable_anonymous_block",
            "fn reparent_subsequent_floating_or_out_of_flow",
            "fn reparent_preceding_floating_or_out_of_flow",
            "fn add_child_before_descendant",
            "fn LayoutBlockFlowAddChildBase",
            "fn LayoutBlockFlowRemoveChildBase",
            "fn LayoutBlockFlowCanMergeWithBase",
            "SplitAnonymousBoxesAroundChild",
            "remove_inner_editor_child",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation child mutation lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "insertion, removal and reparenting walks must retain invalidation sampling frames"
        );

        let facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block_flow.rs")),
        )
        .expect("read source LayoutBlockFlow owner");
        assert!(
            facade.contains("layout_invalidation/src/layout_block_flow_child_mutation.rs")
                && facade.contains("LayoutBlockFlowAddChildBase(self, new_child, before_child)")
                && facade.contains("LayoutBlockFlowRemoveChildBase(self, old_child)")
                && facade.contains("LayoutBlockFlowCanMergeWithBase(self, other)"),
            "LayoutBlockFlow must delegate child mutation to layout_invalidation"
        );
        for leaked_policy in [
            "fn IsInnerEditorChild(",
            "fn IsMergeableAnonymousBlock(",
            "fn AddChildBeforeDescendant(",
            "made_boxes_non_inline",
            "remove_inner_editor_child",
            "SplitAnonymousBoxesAroundChild",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBlockFlow still owns child-mutation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_block_flow_multicol_update_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_block_flow_multicol_update.rs"),
        )
        .expect("read source LayoutBlockFlow multicol-update owner");
        for owned_policy in [
            "fn LayoutBlockFlowUpdateForMulticol",
            "SpecifiesColumns",
            "IsFieldset",
            "IsFormControlElement",
            "kHTMLButtonElement",
            "SetIsMulticolContainer",
            "SetIsInsideMulticolIncludingDescendants",
            "MakeChildrenNonInlineDefault",
            "MakeChildrenInlineIfPossible",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation multicol update lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "multicol admission and descendant propagation must retain an invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block_flow.rs")),
        )
        .expect("read source LayoutBlockFlow owner");
        assert!(
            facade.contains("layout_invalidation/src/layout_block_flow_multicol_update.rs")
                && facade.contains("LayoutBlockFlowUpdateForMulticol(self)"),
            "LayoutBlockFlow must delegate multicol refresh to layout_invalidation"
        );
        for leaked_policy in [
            "let mut should_be_multicol",
            "IsFormControlElement()",
            "SetIsInsideMulticolIncludingDescendants",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBlockFlow still owns multicol-update policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_metrics_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_fragment/src/layout_metrics.rs"))
            .expect("read source layout-metric owner");
        for owned_policy in [
            "fn scroll_range",
            "fn metric",
            "MinimumScrollOffsetInt",
            "OverflowClipRectForScrollNode",
            "OffsetFromRootFragmentationContext",
            "PhysicalPaddingBoxRect",
            "EffectiveZoom",
            "StitchedSize",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment metric projection lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "scroll range and layout metric projection must retain fragment sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_output.rs"))
            .expect("read source layout-metric adapter");
        assert!(
            adapter.contains("layout_fragment/src/layout_metrics.rs")
                && adapter.contains("layout_fragment_layout_metrics::scroll_range(self)")
                && adapter.contains("layout_fragment_layout_metrics::metric(self, metric)"),
            "LayoutNode must delegate CSSOM geometry metrics to layout_fragment"
        );
        for leaked_policy in [
            "MinimumScrollOffsetInt",
            "OffsetFromRootFragmentationContext",
            "PhysicalPaddingBoxRect",
            "EffectiveZoom()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout output facade still owns metric projection {leaked_policy}"
            );
        }
    }

    #[test]
    fn text_fragment_queries_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/text_fragment_queries.rs"))
                .expect("read source text-fragment query owner");
        for owned_policy in [
            "fn paint_glyph_runs",
            "fn text_paint_baseline",
            "fn text_inline_position",
            "fn text_next_character_width",
            "fn text_range_rect",
            "fn text_offset_for_point",
            "BuildPaintGlyphRuns",
            "CaretInlinePositionForOffset",
            "LocalTextRect",
            "TextOffsetForPoint",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment text query lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 6,
            "each on-demand text query class must retain a fragment sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_output.rs"))
            .expect("read text-fragment query adapter");
        for delegated in [
            "layout_fragment_text_fragment_queries::paint_glyph_runs(self)",
            "layout_fragment_text_fragment_queries::text_paint_baseline(self)",
            "layout_fragment_text_fragment_queries::text_inline_position(self, offset)",
            "layout_fragment_text_fragment_queries::text_next_character_width(self, offset)",
            "layout_fragment_text_fragment_queries::text_range_rect(self, start, end)",
            "layout_fragment_text_fragment_queries::text_offset_for_point(self, point)",
        ] {
            assert!(
                adapter.contains(delegated),
                "source text cursor must delegate through {delegated}"
            );
        }
        for leaked_policy in [
            "CaretInlinePositionForOffset",
            "LocalTextRect",
            "item.TextOffsetForPoint",
            "BuildPaintGlyphRuns(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout output facade still owns text query policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn sticky_layout_commit_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/sticky_layout_commit.rs"))
                .expect("read source sticky layout-commit owner");
        for owned_policy in [
            "fn update_sticky_offsets_after_layout",
            "TracePhase::UpdateStickyOffsets",
            "while !object.is_null()",
            "NextInPreOrder(root)",
            "UpdateStickyOffsets(objects, false)",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment sticky layout commit lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "post-layout sticky traversal must retain a fragment sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_fragment/src/sticky_layout_commit.rs")
                && adapter.contains(
                    "layout_fragment_sticky_layout_commit::update_sticky_offsets_after_layout"
                ),
            "LayoutEngine must delegate post-layout sticky traversal to layout_fragment"
        );
        for leaked_policy in [
            "TracePhase::UpdateStickyOffsets",
            "while !object.is_null()",
            "UpdateStickyOffsets(\n                            objects, false",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns sticky layout-commit policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn engine_scroll_refresh_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/retained_scroll_refresh.rs"))
                .expect("read source retained-scroll owner");
        for owned_policy in [
            "fn refresh_retained_scroll_geometry",
            "RefreshScrollCullData",
            "scroll_paint_index_",
            "RefreshScrollPaintProperties",
            "ScrollPaintPropertyIndex::new",
            "PrepareLayoutHeapForPointerQueries",
            "pending_scroll_ids_.clear",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment retained-scroll refresh lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "retained-scroll refresh must retain a fragment sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_fragment/src/retained_scroll_refresh.rs")
                && adapter.contains(
                    "layout_fragment_retained_scroll_refresh::refresh_retained_scroll_geometry"
                ),
            "LayoutEngine must delegate retained-scroll fragment refresh to layout_fragment"
        );
        for leaked_policy in [
            "index.refresh(unsafe { &mut *root }, &self.pending_scroll_ids_)",
            "assembly.RefreshScrollPaintProperties(unsafe { &mut *root }, fragments, true)",
            "heap.DeferCollection();\n            trace.set(\"retained_geometry\"",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns retained-scroll fragment policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_result_publication_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/layout_result_publish.rs"))
                .expect("read source layout-result publication owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn populate_layout_result_paint_data",
            "TracePhase::PopulateLayoutResultPaintData",
            "TracePhase::PopulateFragmentPaintData",
            "TracePhase::ResolveLogicalParents",
            "TracePhase::PopulateGlobalPaintData",
            "FragmentOwners",
            "LayoutResultHandle::FromLayoutResult",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment layout-result publication lost {owned_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains(
                "layout_fragment_layout_result_publish::populate_layout_result_paint_data"
            ),
            "fragment boundary must delegate LayoutResult publication to layout_fragment"
        );
        for leaked_policy in [
            "fn PopulateLayoutResultPaintData(",
            "let mut paint_state_owners = FragmentOwners",
            "TracePhase::PopulateFragmentPaintData",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns layout-result publication policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_paint_population_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_paint_population.rs"))
                .expect("read source fragment paint-population owner");
        for owned_policy in [
            "fn PopulateInlineItemLocalPaintData",
            "fn PopulateInlineItemPaintData",
            "pub(super) fn PopulatePhysicalFragmentPaintData",
            "FragmentPaintData::new",
            "paint_state.SetPhysical",
            "paint_state.SetInline",
            "PopulateSVGChildPaintData",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source paint population lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "physical-fragment and inline-item recursive population must retain sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_paint_population.rs")
                && adapter.contains(
                    "layout_fragment_fragment_paint_population::PopulatePhysicalFragmentPaintData"
                ),
            "fragment boundary must import physical paint population from layout_fragment"
        );
        for leaked_policy in [
            "fn PopulateInlineItemLocalPaintData(",
            "fn PopulateInlineItemPaintData(",
            "fn PopulatePhysicalFragmentPaintData(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns recursive paint-population policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_property_export_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_property_export.rs"))
                .expect("read source fragment property-export owner");
        for owned_policy in [
            "pub(super) fn ExportPaintProperties",
            "fn ExportPrePaintInvalidation",
            "pub(super) fn ExportProperties",
            "ShouldDoFullPaintInvalidation",
            "ExportStyleWithProjectedSvg",
            "ExportStickyPaintProperties",
            "FormControlPaintData",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source property export lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "ordinary and retained property projection must retain sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_property_export.rs")
                && adapter.contains(
                    "layout_fragment_fragment_property_export::{ExportPaintProperties, ExportProperties}"
                ),
            "fragment boundary must import property projection from layout_fragment"
        );
        for leaked_policy in [
            "fn ExportPaintProperties(",
            "fn ExportPrePaintInvalidation(",
            "fn ExportProperties(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns property-export policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_export_support_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_export_support.rs"))
                .expect("read source fragment export support owner");
        for owned_policy in [
            "pub(super) fn Number",
            "pub(super) fn ExportAppliedTextDecorations",
            "pub(super) fn InputId",
            "pub(super) fn PublicWritingMode",
            "pub(super) type SourceMap",
            "pub(super) type StyleSourceMap",
            "pub(super) type ExportedStyleCache",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source export support lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "text-decoration projection must retain a fragment sampling frame"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_export_support.rs")
                && adapter.contains("layout_fragment_fragment_export_support::{"),
            "fragment boundary must import export support from layout_fragment"
        );
        for leaked_policy in [
            "fn ExportAppliedTextDecorations(",
            "type SourceMap =",
            "type StyleSourceMap =",
            "fn TraceInlineExport()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns export support {leaked_policy}"
            );
        }
    }

    #[test]
    fn text_paint_export_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_fragment/src/text_paint_export.rs"))
            .expect("read source text paint-export owner");
        for owned_policy in [
            "struct GlyphExportContext",
            "fn ExportGlyph",
            "pub(super) fn ExportShapeResult",
            "pub(super) fn ExportTextCombineEmphasis",
            "pub(crate) fn BuildPaintGlyphRuns",
            "pub(crate) fn ExportTextPaintBaseline",
            "pub(super) fn ExportTextCombineInlineOffset",
            "pub(super) fn ExportListMarkerSymbol",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source text paint export lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "shape-result and paint-time glyph export must retain sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/text_paint_export.rs")
                && adapter.contains("layout_fragment_text_paint_export::{"),
            "fragment boundary must import text paint export from layout_fragment"
        );
        for leaked_policy in [
            "struct GlyphExportContext",
            "fn ExportGlyph(",
            "fn ExportShapeResult(",
            "fn ResolveTextPaintBaseline(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns text paint-export policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_geometry_export_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_geometry_export.rs"))
                .expect("read source fragment geometry-export owner");
        for owned_policy in [
            "fn FragmentLocalAffineTransform",
            "pub(super) fn ExportSvgTextPaintData",
            "fn ExportStitchedDecorationData",
            "pub(super) fn ExportBoxPaintGeometry",
            "pub(super) fn ExportContentSize",
            "stitched_decoration",
            "scroll_container",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source geometry export lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "SVG-text and box geometry projection must retain sampling frames"
        );
        for reconstructed_state in [
            "let visible_width = Number(clip.size.width)",
            "(Offset::default(), paint.scroll_offset)",
            "compositor_element_id: if area.is_null()",
        ] {
            assert!(
                !owner.contains(reconstructed_state),
                "scroll geometry export must consume the retained ScrollableArea, not reconstruct {reconstructed_state}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_geometry_export.rs")
                && adapter.contains("layout_fragment_fragment_geometry_export::{"),
            "fragment boundary must import geometry projection from layout_fragment"
        );
        for leaked_policy in [
            "fn FragmentLocalAffineTransform(",
            "fn ExportSvgTextPaintData(",
            "fn ExportStitchedDecorationData(",
            "fn ExportBoxPaintGeometry(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns geometry-export policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn specialized_fragment_payloads_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let scrollbar =
            std::fs::read_to_string(root.join("layout_fragment/src/scrollbar_paint_export.rs"))
                .expect("read source scrollbar paint owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn ExportScrollbarPaintData",
            "SetScrollbarThumb",
            "ScrollbarPaintAxis",
            "corner_client_invalidation_reason",
        ] {
            assert!(
                scrollbar.contains(owned_policy),
                "layout_fragment scrollbar payload lost {owned_policy}"
            );
        }

        let specialized =
            std::fs::read_to_string(root.join("layout_fragment/src/specialized_paint_export.rs"))
                .expect("read specialized fragment paint owner");
        for owned_policy in [
            "pub(super) fn ExportTablePaintData",
            "pub(super) fn ExportMathMLPaintData",
            "pub(super) fn ExportFrameSetPaintData",
            "pub(super) fn ExportFieldsetPaintData",
            "pub(super) fn ExportColumnRulePaintData",
            "pub(super) fn ExportGapRulePaintData",
            "pub(super) fn ExportOverflowClipMargin",
        ] {
            assert!(
                specialized.contains(owned_policy),
                "layout_fragment specialized payloads lost {owned_policy}"
            );
        }

        let collapsed = std::fs::read_to_string(
            root.join("layout_fragment/src/collapsed_table_paint_export.rs"),
        )
        .expect("read collapsed-table paint owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn ExportCollapsedTablePaintData",
            "CollapsedTableEdge",
            "CollapsedTableSection",
            "IsStartTableRowFragmented",
            "IsEndTableRowFragmented",
        ] {
            assert!(
                collapsed.contains(owned_policy),
                "layout_fragment collapsed-table payload lost {owned_policy}"
            );
        }

        let table_geometry =
            std::fs::read_to_string(root.join("layout_fragment/src/table_fragment_data.rs"))
                .expect("read fragment-owned table geometry data");
        assert!(
            table_geometry.contains("pub struct TableColumnGeometry")
                && table_geometry.contains("pub struct CollapsedTableBordersGeometry"),
            "table geometry payloads must remain fragment-owned"
        );
        let table_geometry_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/table_fragment_data.rs"))
                .expect("read table fragment-data facade");
        assert!(
            table_geometry_facade.contains("layout_fragment/src/table_fragment_data.rs")
                && table_geometry_facade
                    .contains("pub use layout_fragment_table_fragment_data::*;")
                && !table_geometry_facade.contains("pub struct TableColumnGeometry"),
            "table geometry payloads must be re-exported from layout_fragment"
        );

        let mathml = std::fs::read_to_string(root.join("layout_fragment/src/mathml_paint_info.rs"))
            .expect("read fragment-owned MathML paint data");
        assert!(
            mathml.contains("pub struct MathMLPaintInfo")
                && mathml.contains("operator_shape_result_view")
                && mathml.contains("radical_operator_inline_offset"),
            "retained MathML paint data must remain physically fragment-owned"
        );
        let mathml_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/mathml_paint_info.rs"))
                .expect("read MathML paint-data facade");
        assert!(
            mathml_facade.contains("layout_fragment/src/mathml_paint_info.rs")
                && mathml_facade.contains("pub use layout_fragment_mathml_paint_info::*;")
                && !mathml_facade.contains("pub struct MathMLPaintInfo"),
            "MathML paint data must be re-exported from layout_fragment"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        for owner in [
            "layout_fragment/src/scrollbar_paint_export.rs",
            "layout_fragment/src/specialized_paint_export.rs",
            "layout_fragment/src/collapsed_table_paint_export.rs",
        ] {
            assert!(
                adapter.contains(owner),
                "fragment boundary must import specialized owner {owner}"
            );
        }
        for leaked_policy in [
            "fn ExportScrollbarPaintData(",
            "fn ExportTablePaintData(",
            "fn ExportMathMLPaintData(",
            "fn ExportCollapsedTablePaintData(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns specialized payload policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_export_state_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/fragment_export_state.rs"))
                .expect("read source fragment export-state owner");
        for owned_policy in [
            "pub(super) struct FragmentOwners",
            "pub(super) fn fragment_instance",
            "pub(super) fn PopulateSVGChildPaintData",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source export state lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "SVG child expansion must retain a sampling frame"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/fragment_export_state.rs")
                && adapter.contains("layout_fragment_fragment_export_state::{"),
            "fragment boundary must import publication state from layout_fragment"
        );
        for leaked_policy in ["struct FragmentOwners", "fn PopulateSVGChildPaintData("] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns export-state policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn fragment_ink_and_images_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let storage = std::fs::read_to_string(root.join("layout_fragment/src/ink_overflow.rs"))
            .expect("read fragment-owned ink-overflow storage");
        for owned_policy in [
            "pub struct InkOverflow",
            "pub enum InkOverflowType",
            "pub fn Set(",
            "fn InkOverflowComputeEmphasisMarkOverflowForTextCombine",
            "fn InkOverflowExpandForShadowOverflowForTextCombine",
        ] {
            assert!(
                storage.contains(owned_policy),
                "fragment ink-overflow storage lost {owned_policy}"
            );
        }
        assert!(
            !storage.contains("trait InkOverflowTextMethods")
                && !storage.contains("trait InkOverflowTextInternals"),
            "ink-overflow must not retain declaration-only substitute traits"
        );
        let storage_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/ink_overflow.rs"))
                .expect("read ink-overflow compatibility facade");
        assert!(
            storage_facade.contains("layout_fragment/src/ink_overflow.rs")
                && storage_facade.contains("pub use layout_fragment_ink_overflow::*;")
                && !storage_facade.contains("pub struct InkOverflow"),
            "ink-overflow storage must be re-exported from layout_fragment"
        );

        let ink = std::fs::read_to_string(root.join("layout_fragment/src/ink_overflow_export.rs"))
            .expect("read source ink-overflow owner");
        for owned_policy in [
            "pub(super) fn ExportInkRect",
            "pub(super) fn ExportBoxInkOverflow",
            "pub(super) fn ExportInlineInkOverflow",
            "ComputeInkBounds",
            "text_glyph_ink",
        ] {
            assert!(
                ink.contains(owned_policy),
                "layout_fragment source ink export lost {owned_policy}"
            );
        }
        assert!(
            ink.matches("#[inline(never)]").count() >= 2,
            "box and inline ink export must retain separate sampling frames"
        );
        assert!(
            ink.contains("layout_box.SelfVisualOverflowRect()")
                && !ink.contains("paint.style.box_shadows")
                && !ink.contains("paint.style.outline_width"),
            "LayoutBox visual overflow must come from the source overflow owner, not style reconstruction"
        );

        let images =
            std::fs::read_to_string(root.join("layout_fragment/src/nine_piece_image_export.rs"))
                .expect("read source nine-piece image owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn ExportNinePieceImages",
            "FetchedNinePieceImageResourceId",
            "border_image",
            "mask_box_image",
            "reflection_mask_image",
        ] {
            assert!(
                images.contains(owned_policy),
                "layout_fragment nine-piece image export lost {owned_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        for owner in [
            "layout_fragment/src/ink_overflow_export.rs",
            "layout_fragment/src/nine_piece_image_export.rs",
        ] {
            assert!(
                adapter.contains(owner),
                "fragment boundary must import {owner}"
            );
        }
        for leaked_policy in [
            "fn ExportBoxInkOverflow(",
            "fn ExportInlineInkOverflow(",
            "fn ExportNinePieceImages(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns ink/image policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn global_fragment_passes_resolve_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let background_transfer =
            std::fs::read_to_string(root.join("layout_fragment/src/background_transfer.rs"))
                .expect("read source canvas-background transfer owner");
        for owned_policy in [
            "#[inline(never)]",
            "fn LayoutBoxModelObjectBackgroundTransfersToView",
            "fn FindDocumentElement",
            "while !child.is_null()",
            "ShouldApplyPaintContainmentWithStyle",
        ] {
            assert!(
                background_transfer.contains(owned_policy),
                "layout_fragment background transfer lost {owned_policy}"
            );
        }
        let background_adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_model_object.rs"))
                .expect("read LayoutBoxModelObject background adapter");
        assert!(
            background_adapter.contains("layout_fragment/src/background_transfer.rs")
                && background_adapter.contains(
                    "LayoutBoxModelObjectBackgroundTransfersToView(self, document_style)"
                ),
            "LayoutBoxModelObject must delegate canvas-background transfer to layout_fragment"
        );
        for leaked_policy in [
            "fn FindDocumentElement",
            "while !child.is_null()",
            "ShouldApplyPaintContainmentWithStyle(document_style)",
        ] {
            assert!(
                !background_adapter.contains(leaked_policy),
                "LayoutBoxModelObject still owns background-transfer policy {leaked_policy}"
            );
        }

        let global =
            std::fs::read_to_string(root.join("layout_fragment/src/global_paint_export.rs"))
                .expect("read source global paint owner");
        for owned_policy in [
            "pub(super) fn ExportCanvasBackground",
            "pub(super) fn ExportGlobalMainThreadScrollFacts",
            "background_transfers_to_view",
            "requires_main_thread_scrolling_for_background_attachment_fixed",
        ] {
            assert!(
                global.contains(owned_policy),
                "layout_fragment global paint export lost {owned_policy}"
            );
        }

        let relationships = std::fs::read_to_string(
            root.join("layout_fragment/src/fragment_relationship_export.rs"),
        )
        .expect("read source fragment relationship owner");
        for owned_policy in [
            "pub(super) fn ResolveLogicalParentOccurrences",
            "pub(super) fn PopulateInlineStitchedDecorations",
            "logical_parent_fragment_instance_id",
            "StitchedDecorationData",
        ] {
            assert!(
                relationships.contains(owned_policy),
                "layout_fragment relationship pass lost {owned_policy}"
            );
        }

        let sticky =
            std::fs::read_to_string(root.join("layout_fragment/src/sticky_paint_export.rs"))
                .expect("read source sticky paint owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn ExportStickyPaintProperties",
            "StickyAxisPaintData",
            "StickyPositionPaintData",
        ] {
            assert!(
                sticky.contains(owned_policy),
                "layout_fragment sticky paint export lost {owned_policy}"
            );
        }

        let thumb =
            std::fs::read_to_string(root.join("layout_fragment/src/scrollbar_thumb_geometry.rs"))
                .expect("read source scrollbar thumb owner");
        assert!(
            thumb.contains("#[inline(never)]") && thumb.contains("pub(super) fn SetScrollbarThumb"),
            "scrollbar thumb geometry must keep a fragment sampling entry"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        for owner in [
            "layout_fragment/src/global_paint_export.rs",
            "layout_fragment/src/fragment_relationship_export.rs",
            "layout_fragment/src/sticky_paint_export.rs",
            "layout_fragment/src/scrollbar_thumb_geometry.rs",
        ] {
            assert!(
                adapter.contains(owner),
                "fragment boundary must import {owner}"
            );
        }
        for leaked_policy in [
            "fn ExportCanvasBackground(",
            "fn ResolveLogicalParentOccurrences(",
            "fn PopulateInlineStitchedDecorations(",
            "fn ExportStickyPaintProperties(",
            "fn SetScrollbarThumb(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns global paint policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn inline_fragment_policy_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_fragment/src/inline_fragment_paint_policy.rs"),
        )
        .expect("read source inline-fragment policy owner");
        for owned_policy in [
            "fn ProjectedStyleRequiresPaintBackedInlineFragment",
            "pub(crate) fn ComputedStyleRequiresPaintBackedInlineFragment",
            "pub(crate) fn RequiresPaintBackedInlineFragment",
            "paint.backdrop_filter_has_reference",
            "style.HasBoxDecorationBackground()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment inline-fragment policy lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "typed and source inline admission must retain sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        assert!(
            adapter.contains("layout_fragment/src/inline_fragment_paint_policy.rs")
                && adapter.contains("layout_fragment_inline_fragment_paint_policy::{"),
            "fragment boundary must re-export inline admission from layout_fragment"
        );
        for leaked_policy in [
            "fn ProjectedStyleRequiresPaintBackedInlineFragment(",
            "fn ComputedStyleRequiresPaintBackedInlineFragment(",
            "fn RequiresPaintBackedInlineFragment(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns inline admission policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn compatible_paint_cache_refresh_resolves_to_the_fragment_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_fragment/src/paint_cache_refresh.rs"))
                .expect("read source paint-cache refresh owner");
        for owned_policy in [
            "pub(super) fn refresh_compatible_node_input_cache",
            "pub(super) fn refresh_compatible_paint_cache",
            "TracePhase::RefreshBoundaryCache",
            "cache.style_sources.clear()",
            "HasStickyConstrainedPosition",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_fragment source paint-cache refresh lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "node-input and paint cache refresh entries must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read fragment boundary adapter");
        for delegation in [
            "layout_fragment_paint_cache_refresh::refresh_compatible_node_input_cache",
            "layout_fragment_paint_cache_refresh::refresh_compatible_paint_cache",
        ] {
            assert!(
                adapter.contains(delegation),
                "fragment boundary must delegate through {delegation}"
            );
        }
        for leaked_policy in [
            "cache.style_sources.clear()",
            "HasStickyConstrainedPosition()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "fragment boundary still owns paint-cache refresh policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn tree_update_transaction_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_invalidation/src/tree_update.rs"))
            .expect("read source invalidation tree-update owner");
        for owned_policy in [
            "pub(super) fn reattach",
            "pub(super) fn place",
            "pub(super) fn update_subtree",
            "pub(super) fn complete_update",
            "pub(super) fn begin_update",
            "pub(super) fn finish_update",
            "while position < affected.len()",
            "let mut candidate = before",
            "InsertForTreeUpdate",
            "storage.reattach.extend(node_ids.iter().copied())",
            "storage.retired.push",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source transaction lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "tree placement and update transaction stages must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject tree-update adapter");
        for delegation in [
            "layout_invalidation_package::place",
            "layout_invalidation_package::reattach",
            "layout_invalidation_package::update_subtree",
            "layout_invalidation_package::complete_update",
            "layout_invalidation_package::begin_update",
            "layout_invalidation_package::finish_update",
        ] {
            assert!(
                adapter.contains(delegation),
                "LayoutObject tree update does not delegate through {delegation}"
            );
        }
        for leaked_policy in [
            "let mut candidate = before",
            "storage.stats.moved += 1",
            "storage.reattach.extend(node_ids.iter().copied())",
            "while position < affected.len()",
            "storage.retired.push",
            "storage.partial_index = Some(index)",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutObject adapter still owns invalidation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn font_environment_cache_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_reuse/src/layout_environment_cache.rs"))
                .expect("read source reusable font-environment owner");
        for owned_policy in [
            "pub(crate) struct ReusableLayoutEnvironment",
            "fn Matches(&self",
            "pub(super) struct FontBinding",
            "pub(super) fn new(root",
            "TakeReusableLayoutEnvironment",
            "SetReusableLayoutEnvironment",
            concat!("layout_", "reuse.FontEnvironment"),
            "trace.set(\"reused\"",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse source font environment lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "environment matching and resolver binding must retain sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source layout boundary adapter");
        assert!(
            adapter.contains("layout_reuse/src/layout_environment_cache.rs")
                && adapter
                    .contains("layout_reuse_layout_environment_cache::ReusableLayoutEnvironment"),
            "layout boundary must import reusable font state from layout_reuse"
        );
        for leaked_policy in [
            "struct ReusableLayoutEnvironment",
            "struct FontBinding",
            "TakeReusableLayoutEnvironment()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout boundary still owns reusable font policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn engine_output_reuse_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_reuse/src/engine_output_reuse.rs"))
            .expect("read source engine output-reuse owner");
        for owned_policy in [
            "fn has_current_output",
            "engine.fragments.is_some()",
            "engine.invalidated_fragments_.is_none()",
            "pending_scroll_ids_",
            "pending_paint_ids_",
            "pending_inline_ink_ids_",
            "pending_subtree_layout_id_",
            "!tree.Root().NeedsLayout()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse resident output admission lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "resident output admission must retain a reuse sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_reuse/src/engine_output_reuse.rs")
                && adapter.contains(
                    "layout_reuse_engine_output_reuse::has_current_output(self, purpose)"
                ),
            "LayoutEngine must delegate resident output admission to layout_reuse"
        );
        for leaked_policy in [
            "self.invalidated_fragments_.is_none()\n            && self.pending_scroll_ids_",
            "self.pending_inline_ink_ids_.is_empty()\n            && self.pending_subtree_layout_id_",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns resident output reuse policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn font_resolver_resolves_to_the_reuse_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_reuse/src/font_resolver.rs"))
            .expect("read source font resolver owner");
        for owned_policy in [
            "struct DeferredFontResolver",
            "pub(super) fn EngineFontResolver",
            "struct SharedEngineFontResolver",
            "FontEngine::ResolveFont",
            "FontEngine::FromFaces",
            "pub(crate) fn ConstructionFontResolver",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_reuse source font resolver lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "engine and construction resolver selection must retain sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source layout boundary adapter");
        assert!(
            adapter.contains("layout_reuse/src/font_resolver.rs")
                && adapter.contains("layout_reuse_font_resolver::EngineFontResolver")
                && adapter.contains("layout_reuse_font_resolver::ConstructionFontResolver"),
            "layout boundary must import source font resolvers from layout_reuse"
        );
        for leaked_policy in [
            "struct DeferredFontResolver",
            "struct SharedEngineFontResolver",
            "fn EngineFontResolver(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout boundary still owns font-resolver policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn font_from_selector_resolves_to_the_font_boundary() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layoutng/src/internal/boundary/font_from_selector.rs"),
        )
        .expect("read selector-backed font boundary");
        for owned_policy in [
            "fn FontFromSelector",
            "while !family.is_null()",
            "families.push",
            "FontRequest::new_auto",
            "CurrentFontResolver",
            "CloneWithDescription",
            "fn FontNewWithSelector",
        ] {
            assert!(
                owner.contains(owned_policy),
                "selector-backed font construction lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            1,
            "family enumeration and resolution must retain one font-boundary sampling frame"
        );

        assert!(
            !owner.contains("layout_reuse/src/font_from_selector.rs"),
            "selector-backed construction must not be assigned to cache reuse"
        );
    }

    #[test]
    fn layout_object_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let tree_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_tree_invalidation.rs"),
        )
        .expect("read source invalidation tree adapter");
        for owned_policy in [
            "pub(crate) struct LayoutObjectInvalidationTree",
            "impl layout_invalidation::ContainerChainInvalidationTree",
            "impl layout_invalidation::PaintInvalidationTree",
            "fn MarkContainerChainForLayout(",
            "fn MarkContainerChainForLayoutWithSchedule",
            "propagate_container_chain_for_layout",
            "fn InLayoutNGInlineFormattingContextWillChangeBase",
            "fn InLayoutNGInlineFormattingContextWillChange(",
            "DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange",
            "pub fn SetIntrinsicLogicalWidthsDirty",
            "pub fn MarkNeedsSimplifiedLayout",
        ] {
            assert!(
                tree_owner.contains(owned_policy),
                "layout_invalidation source tree adapter lost {owned_policy}"
            );
        }
        assert!(
            tree_owner.contains("#[inline(never)]\nfn ShouldInvalidateBeyond"),
            "the source intrinsic-width boundary decision must retain an invalidation sampling frame"
        );
        assert!(
            tree_owner.contains(
                "#[inline(never)]\n    pub fn InLayoutNGInlineFormattingContextWillChange"
            ),
            "inline-formatting-context mutation must retain an invalidation sampling frame"
        );
        assert!(
            tree_owner
                .contains("#[inline(never)]\n    pub fn MarkContainerChainForLayoutWithSchedule"),
            "container-chain propagation must retain an invalidation sampling frame"
        );

        let dirty_state_owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_object_dirty_state.rs"),
        )
        .expect("read source LayoutObject dirty-state owner");
        for owned_policy in [
            "fn SetNeedsLayout(",
            "fn SetNeedsLayoutWithMarking",
            "fn SetNeedsLayoutAndFullPaintInvalidation",
            "fn SetNeedsLayoutAndFullPaintInvalidationWithMarking",
            "fn SetNeedsLayoutAndIntrinsicWidthsRecalc",
            "fn SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation",
            "fn ClearNeedsLayoutWithoutPaintInvalidation",
            "fn ClearNeedsLayout(",
            "fn ClearNeedsLayoutWithFullPaintInvalidation",
            "SetSelfNeedsFullLayout(true)",
            "SetSelfNeedsFullLayout(false)",
            "SetNeedsOverflowRecalc",
            "SetSubgridMinMaxSizesCacheDirty",
            "SetTableColumnConstraintsDirty",
            "MarkContainerChainForLayout",
            "SetIntrinsicLogicalWidthsDirty",
        ] {
            assert!(
                dirty_state_owner.contains(owned_policy),
                "layout_invalidation hot dirty-state owner lost {owned_policy}"
            );
        }
        assert!(
            !dirty_state_owner.contains("#[inline(never)]"),
            "small per-object dirty-state composition must remain inlineable"
        );

        let geometry_owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/geometry_invalidation.rs"))
                .expect("read source geometry invalidation owner");
        for owned_policy in [
            "SetShouldDoFullPaintInvalidationWithReason",
            "SetShouldCheckForPaintInvalidation",
            "SetSubtreeShouldDoFullPaintInvalidation",
        ] {
            assert!(
                geometry_owner.contains(owned_policy),
                "layout_invalidation source geometry policy lost {owned_policy}"
            );
        }

        let tree_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_tree_invalidation.rs"))
                .expect("read source invalidation tree facade");
        let geometry_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_geometry_invalidation.rs"),
        )
        .expect("read source geometry invalidation facade");
        let dirty_state_loader =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_inlines.rs"))
                .expect("read source dirty-state compatibility loader");
        assert!(
            tree_facade.contains("layout_invalidation/src/layout_tree_invalidation.rs")
                && geometry_facade.contains("layout_invalidation/src/geometry_invalidation.rs")
                && dirty_state_loader
                    .contains("layout_invalidation/src/layout_object_dirty_state.rs"),
            "core invalidation modules must be owner facades"
        );
        for leaked_policy in [
            "impl layout_invalidation::PaintInvalidationTree",
            "SetShouldDoFullPaintInvalidationWithReason",
        ] {
            assert!(
                !tree_facade.contains(leaked_policy) && !geometry_facade.contains(leaked_policy),
                "core invalidation facade still owns policy {leaked_policy}"
            );
        }
        for leaked_policy in [
            "fn SetNeedsLayout(",
            "fn SetNeedsLayoutWithMarking",
            "fn SetNeedsLayoutAndIntrinsicWidthsRecalc",
        ] {
            assert!(
                !dirty_state_loader.contains(leaked_policy),
                "core dirty-state loader still owns {leaked_policy}"
            );
        }

        let hot_object =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_hot.rs"))
                .expect("read LayoutObject hot compatibility module");
        for leaked_policy in [
            "fn ClearNeedsLayoutWithoutPaintInvalidation",
            "fn ClearNeedsLayout(&mut self)",
            "fn ClearNeedsLayoutWithFullPaintInvalidation",
        ] {
            assert!(
                !hot_object.contains(leaked_policy),
                "LayoutObject hot core still owns dirty-state reset {leaked_policy}"
            );
        }

        let object_core =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
                .expect("read LayoutObject compatibility module");
        for leaked_policy in [
            "fn InLayoutNGInlineFormattingContextWillChangeBase",
            "fn InLayoutNGInlineFormattingContextWillChange(",
        ] {
            assert!(
                !object_core.contains(leaked_policy),
                "LayoutObject core still owns inline lifecycle {leaked_policy}"
            );
        }
        let core_services = std::fs::read_to_string(
            root.join("layoutng/src/internal/layout_object_core_services.rs"),
        )
        .expect("read LayoutObject core services");
        for leaked_policy in [
            "fn MarkContainerChainForLayout(",
            "fn MarkContainerChainForLayoutWithSchedule",
        ] {
            assert!(
                !core_services.contains(leaked_policy),
                "LayoutObject core services still own container propagation {leaked_policy}"
            );
        }
    }

    #[test]
    fn input_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/input_invalidation.rs"))
                .expect("read source input invalidation owner");
        for owned_policy in [
            "pub(super) fn invalidate_editing",
            "pub(super) fn invalidate_selection_paint",
            "TracePhase::EditingInvalidation",
            "TracePhase::SelectionPaintInvalidation",
            "while let Some(object) = stack.pop()",
            "SetShouldDoFullPaintInvalidationWithoutLayoutChange",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source input invalidation lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "editing and selection invalidation must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder adapter");
        for delegation in [
            "layout_invalidation_input_updates::invalidate_editing",
            "layout_invalidation_input_updates::invalidate_selection_paint",
        ] {
            assert!(
                adapter.contains(delegation),
                "LayoutObject builder must delegate through {delegation}"
            );
        }
        for leaked_policy in [
            "while let Some(object) = stack.pop()",
            "leaves.push(object)",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutObject builder still owns input invalidation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn node_input_transaction_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/node_input_update.rs"))
                .expect("read source node-input transaction owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn update_node_input",
            "LayoutObjectTreeStorage::StyleUpdateCompatible",
            "for key in storage.ChildKeys(parent)",
            "storage.pending.insert",
            "seal_input_tree",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source node-input transaction lost {owned_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_object_builder.rs"))
                .expect("read LayoutObject builder adapter");
        assert!(
            adapter.contains("layout_invalidation_node_input::update_node_input"),
            "LayoutObject builder must delegate targeted node-input updates to layout_invalidation"
        );
        for leaked_policy in [
            "relations_changed |= record.input.element != input.element",
            "let mut objects = Vec::with_capacity(inputs.len())",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutObject builder still owns node-input transaction policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn tree_facts_scan_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_invalidation/src/tree_facts.rs"))
            .expect("read source invalidation tree-facts owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn collect_layout_tree_facts",
            "TracePhase::CollectTreeFacts",
            "while !object.is_null()",
            "facts.has_floating_object |=",
            "facts.contains_annotations |=",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source tree-facts scan lost {owned_policy}"
            );
        }

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("collect_layout_tree_facts(root, &self.objects_)"),
            "LayoutEngine must delegate its cached facts scan to layout_invalidation"
        );
        for leaked_policy in [
            "facts.has_floating_object |=",
            "facts.contains_annotations |=",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns tree-facts policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn constraint_equivalence_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/constraint_equivalence.rs"))
                .expect("read source constraint-equivalence owner");
        for owned_policy in [
            "pub(super) fn SameOptionalArc",
            "pub(super) fn SameImageCatalogGeometry",
            "pub(super) fn SameShapeImageCatalog",
            "pub(super) fn SameConstraintScalars",
            "pub(super) fn SameConstraints",
            "Arc::ptr_eq",
            "document_image_container",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation constraint equivalence lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "complete constraint admission must retain an invalidation sampling frame"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_invalidation/src/constraint_equivalence.rs")
                && adapter.contains("layout_invalidation_constraint_equivalence::{"),
            "LayoutEngine must import constraint admission from layout_invalidation"
        );
        for leaked_policy in [
            "fn SameOptionalArc",
            "fn SameImageCatalogGeometry(",
            "fn SameConstraintScalars(",
            "fn SameConstraints(",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns constraint policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn engine_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/engine_mutation.rs"))
                .expect("read source engine-mutation owner");
        for owned_policy in [
            "trait EngineMutation",
            "fn geometry_input_matches",
            "fn apply_mutation",
            "fn retain_invalidated_layout_result",
            "LayoutMutation::TreeUpdate",
            "LayoutMutation::SubtreeUpdate",
            "LayoutMutation::PaintInput",
            "RefreshCompatibleNodeInputCache",
            "RefreshCompatiblePaintCache",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source engine mutation lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 2,
            "geometry admission and mutation dispatch must retain invalidation sampling frames"
        );

        let adapter = std::fs::read_to_string(root.join("layoutng/src/layout_engine.rs"))
            .expect("read LayoutEngine adapter");
        assert!(
            adapter.contains("layout_invalidation/src/engine_mutation.rs")
                && adapter
                    .contains("EngineMutation::geometry_input_matches(self, id, style, element)")
                && adapter.contains("EngineMutation::apply_mutation(self, mutation)"),
            "LayoutEngine must delegate mutation admission to layout_invalidation"
        );
        for leaked_policy in [
            "match mutation {",
            "tree.UpdateSubtree(node_id, parent_id",
            "tree.UpdatePaintInput(node_id, style, element)",
            "fn RetainInvalidatedLayoutResult",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutEngine still owns source mutation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn positioned_object_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/positioned_object_invalidation.rs"),
        )
        .expect("read positioned-object invalidation owner");
        for owned_policy in [
            "fn remove_positioned_objects",
            "HasOutOfFlowFragmentChild",
            "IsOutOfFlowPositioned",
            "GetMutableLayoutObject",
            "MarkParentForSpannerOrOutOfFlowPositionedChange",
            "MarkingBehavior::kMarkContainerChain",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation positioned-object scan lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "positioned-object invalidation scan must retain an invalidation sampling frame"
        );

        let adapter = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block.rs")),
        )
        .expect("read source LayoutBlock owner");
        assert!(
            adapter.contains("layout_invalidation/src/positioned_object_invalidation.rs")
                && adapter.contains(
                    "layout_invalidation_positioned_object_invalidation::remove_positioned_objects"
                ),
            "LayoutBlock must delegate positioned-object removal to layout_invalidation"
        );
        for leaked_policy in [
            "HasOutOfFlowFragmentChild",
            "IsOutOfFlowPositioned",
            "process_positioned_object_removal",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "LayoutBlock still owns positioned-object invalidation {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_block_child_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_block_child_mutation.rs"),
        )
        .expect("read source LayoutBlock child-mutation owner");
        for owned_policy in [
            "fn LayoutBlockAddChildBeforeDescendant",
            "SplitAnonymousBoxesAroundChild",
            "while unsafe { &*before_descendant_container }.Parent()",
            "IsAnonymous",
            "IsLayoutBlockFlow",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation LayoutBlock child mutation lost {owned_policy}"
            );
        }
        assert!(
            owner.contains("#[inline(never)]"),
            "descendant-container discovery must retain an invalidation sampling frame"
        );

        let facade = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "block/src/layout_block.rs")),
        )
        .expect("read source LayoutBlock owner");
        assert!(
            facade.contains("layout_invalidation/src/layout_block_child_mutation.rs")
                && facade.contains(
                    "LayoutBlockAddChildBeforeDescendant(self, new_child, before_descendant)",
                ),
            "LayoutBlock must delegate descendant insertion routing to layout_invalidation"
        );
        for leaked_policy in [
            "while unsafe { &*before_descendant_container }.Parent()",
            "SplitAnonymousBoxesAroundChild(before_descendant)",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBlock facade still owns child-mutation traversal {leaked_policy}"
            );
        }
    }

    #[test]
    fn out_of_flow_data_resolves_to_the_out_of_flow_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/out_of_flow_data.rs"
        )))
        .expect("read out-of-flow retained-data owner");
        for owned_policy in [
            "pub struct RememberedScrollOffsets",
            "pub struct ScrollOffsetPair",
            "pub struct OutOfFlowData",
            "fn HasStaleFallbackData",
            "fn SetPendingRememberedScrollOffsets",
            "fn GetSpeculativeRememberedScrollOffsets",
            "pub trait OutOfFlowDataExternal",
            "WeakHeapHashMap",
            "SuccessfulPositionFallback",
        ] {
            assert!(
                owner.contains(owned_policy),
                "out-of-flow retained data lost {owned_policy}"
            );
        }
        assert_eq!(
            owner.matches("#[inline(never)]").count(),
            2,
            "fallback and pending-offset equivalence must retain separate out-of-flow sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/css/out_of_flow_data.rs"))
                .expect("read out-of-flow data compatibility facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/out_of_flow_data.rs"
            )) && facade.contains("pub use layout_algo_out_of_flow_out_of_flow_data::*;"),
            "out_of_flow_data must re-export state from its algorithm owner"
        );
        for leaked_policy in [
            "pub struct RememberedScrollOffsets",
            "pub struct OutOfFlowData",
            "WeakHeapHashMap",
            "SuccessfulPositionFallback",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "out-of-flow data facade still owns algorithm policy {leaked_policy}"
            );
        }

        let fallback_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/successful_position_fallback.rs"
        )))
        .expect("read successful position-fallback owner");
        assert!(
            fallback_owner.contains("pub struct SuccessfulPositionFallback")
                && fallback_owner.contains("pub fn IsEmpty")
                && fallback_owner.contains("pub fn Clear")
                && fallback_owner.contains("impl PartialEq for SuccessfulPositionFallback")
                && !fallback_owner.contains("#[inline(never)]"),
            "fixed-cost position fallback state must remain inlineable in layout_algo_out_of_flow"
        );
        let fallback_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/css/successful_position_fallback.rs"),
        )
        .expect("read successful position-fallback facade");
        assert!(
            fallback_facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/successful_position_fallback.rs"
            )) && fallback_facade
                .contains("pub use layout_algo_out_of_flow_successful_position_fallback::*",)
                && !fallback_facade.contains("pub struct SuccessfulPositionFallback"),
            "successful position fallback must be re-exported from its out-of-flow owner"
        );
    }

    #[test]
    fn anchor_lookup_resolves_to_the_out_of_flow_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "out_of_flow"))
                .join("src/anchor_lookup.rs"),
        )
        .expect("read source out-of-flow anchor lookup owner");
        for owned_policy in [
            "fn for_each_anchor_map_on_container",
            "pub fn LayoutBoxFindTargetAnchor",
            "pub fn LayoutBoxAcceptableImplicitAnchor",
            "PhysicalFragments",
            "InlineCursor",
            "ToAnchorScopedName",
            "ImplicitAnchorElement",
        ] {
            assert!(
                owner.contains(owned_policy),
                "out-of-flow algorithm anchor lookup lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "anchor-map traversal and explicit/implicit lookup must retain out-of-flow sampling frames"
        );

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/layout_box_core_services.rs"))
                .expect("read LayoutBox core-services facade");
        assert!(
            facade.contains(concat!("layout_algo_", "out_of_flow/src/anchor_lookup.rs"))
                && facade.contains("LayoutBoxAcceptableImplicitAnchor")
                && facade.contains("LayoutBoxFindTargetAnchor"),
            "LayoutBox core services must re-export anchor lookup from the out-of-flow owner"
        );
        for leaked_policy in [
            "fn for_each_anchor_map_on_container",
            "ToAnchorScopedName(name",
            "ImplicitAnchorElement()",
            "MoveToNextForSameLayoutObject",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutBox core services still own out-of-flow anchor lookup {leaked_policy}"
            );
        }
    }

    #[test]
    fn anchor_map_operations_resolve_to_the_out_of_flow_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "out_of_flow"))
                .join("src/anchor_map_operations.rs"),
        )
        .expect("read source AnchorMap operation owner");
        for owned_policy in [
            "fn PhysicalAnchorReferenceInsertInReverseTreeOrderInto",
            "fn AnchorMapEntries",
            "fn AnchorMapAnchorReference",
            "fn AnchorMapSetReference",
            "fn AnchorMapSetFromChild",
            "while !result.is_null()",
            "while !existing.is_null()",
            "for entry in unsafe { &*child_map }",
            "UpdateTransformState",
        ] {
            assert!(
                owner.contains(owned_policy),
                "out-of-flow algorithm AnchorMap operations lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "ordering, enumeration, lookup, merge, and child projection must retain out-of-flow sampling frames"
        );

        let schema = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "out_of_flow"))
                .join("src/anchor_map.rs"),
        )
        .expect("read source AnchorMap schema owner");
        for owned_type in [
            "pub struct PhysicalAnchorReference",
            "pub enum AnchorKey",
            "pub struct AnchorMapEntry",
            "pub struct AnchorMap",
            "AnchorMapAnchorReference(self, query_box",
            "AnchorMapSetFromChild(",
        ] {
            assert!(
                schema.contains(owned_type),
                "out-of-flow AnchorMap schema lost {owned_type}"
            );
        }
        assert!(
            schema.contains("#[path = \"anchor_map_operations.rs\"]"),
            "AnchorMap schema must compile its out-of-flow operations beside the owned data"
        );
        let services = std::fs::read_to_string(
            root.join(concat!("layout_algo_", "out_of_flow"))
                .join("src/anchor_map_services.rs"),
        )
        .expect("read source AnchorMap services owner");
        assert!(
            services.contains("fn HasRunningTransformAnimation")
                && services.contains("TransformAnimation::kTransform")
                && services.contains("TransformAnimation::kScale"),
            "out-of-flow AnchorMap services lost transform-animation admission"
        );
        let services_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/anchor_map_services.rs"))
                .expect("read AnchorMap services facade");
        assert!(
            services_facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/anchor_map_services.rs"
            )) && !services_facade.contains("fn HasRunningTransformAnimation"),
            "AnchorMap services must be re-exported from the out-of-flow algorithm"
        );

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/anchor_map.rs"))
            .expect("read AnchorMap facade");
        assert!(
            facade.contains(concat!("layout_algo_", "out_of_flow/src/anchor_map.rs"))
                && facade.contains("pub use layout_algo_out_of_flow_anchor_map::*;")
                && !facade.contains("pub struct AnchorMap"),
            "AnchorMap schema and operations must be re-exported from the out-of-flow algorithm"
        );
        for leaked_policy in [
            "while !result.is_null()",
            "while !existing.is_null()",
            "for entry in unsafe { &*child_map }",
            "for (key, value) in self.named_anchors_",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "AnchorMap facade still owns out-of-flow traversal {leaked_policy}"
            );
        }
    }

    #[test]
    fn anchor_position_scroll_data_resolves_to_the_out_of_flow_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/anchor_position_scroll_data.rs"
        )))
        .expect("read source anchor-position scroll-data owner");
        for owned_policy in [
            "pub struct AnchorPositionScrollData",
            "fn ComputeAdjustmentContainersData",
            "fn IsFallbackPositionValid",
            "fn InvalidateLayoutDependentAndAncestors",
            "fn InvalidateLayoutDependents",
            "while !container.is_null()",
            "for range in unsafe { &*ranges }",
            "for dependent in self.dependent_anchors_.iter()",
            "kAnchorPositioning",
        ] {
            assert!(
                owner.contains(owned_policy),
                "out-of-flow anchor-position scroll data lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 4,
            "container scan, fallback validation, and recursive invalidation must retain out-of-flow sampling frames"
        );

        let services = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/anchor_scroll_services_interface.rs"
        )))
        .expect("read source anchor scroll-container services owner");
        for owned_policy in [
            "pub struct AnchorScrollContainerState",
            "fn ReadAnchorScrollContainerState",
            "fn AnchorViewportUsesTransformOverscroll",
            "GetScrollElementId",
            "GetScrollOffset",
            "ScrollOrigin",
            "OverscrollType::kTransform",
        ] {
            assert!(
                services.contains(owned_policy),
                "out-of-flow source anchor scroll services lost {owned_policy}"
            );
        }
        assert!(
            !services.contains("#[inline(never)]"),
            "fixed-cost anchor scroll-container facts must remain inlineable"
        );

        let facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/anchor_position_scroll_data.rs"),
        )
        .expect("read anchor-position scroll-data facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/anchor_position_scroll_data.rs"
            )) && facade
                .contains("pub use layout_algo_out_of_flow_anchor_position_scroll_data::*;"),
            "anchor-position scroll data must be re-exported from the out-of-flow owner"
        );
        for leaked_policy in [
            "pub struct AnchorPositionScrollData",
            "while !container.is_null()",
            "for range in",
            "for dependent in",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "anchor-position scroll-data facade still owns out-of-flow policy {leaked_policy}"
            );
        }

        let services_facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/anchor_scroll_services.rs"))
                .expect("read anchor scroll-container services facade");
        assert!(
            services_facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/anchor_scroll_services_interface.rs"
            )) && services_facade
                .contains("pub use layout_algo_out_of_flow_anchor_scroll_services_interface::*;"),
            "anchor scroll-container facts must be re-exported from the out-of-flow owner"
        );
        assert!(
            !services_facade.contains("pub struct AnchorScrollContainerState")
                && !services_facade.contains("fn ReadAnchorScrollContainerState"),
            "anchor scroll-container facade must not retain out-of-flow policy"
        );

        let range_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/non_overflowing_scroll_range.rs"
        )))
        .expect("read source non-overflowing scroll-range owner");
        assert!(
            range_owner.contains("pub struct NonOverflowingScrollRange")
                && range_owner.contains("fn Contains"),
            "anchor fallback scroll ranges must remain out-of-flow-owned"
        );
        let range_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/non_overflowing_scroll_range.rs"),
        )
        .expect("read non-overflowing scroll-range facade");
        assert!(
            range_facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/non_overflowing_scroll_range.rs"
            )) && range_facade
                .contains("pub use layout_algo_out_of_flow_non_overflowing_scroll_range::*;")
                && !range_facade.contains("pub struct NonOverflowingScrollRange"),
            "non-overflowing scroll ranges must be re-exported from the out-of-flow owner"
        );
    }

    #[test]
    fn oof_positioned_node_resolves_to_the_out_of_flow_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/oof_positioned_node.rs"
        )))
        .expect("read source OOF positioned-node owner");
        for owned_policy in [
            "pub struct OofContainingBlock",
            "pub struct OofInlineContainer",
            "pub struct OofPositionedNode",
            "pub struct FragmentedOofData",
            "fn LogicalOofPositionedNodeToPhysical",
            "fn PhysicalOofPositionedNodeToLogical",
            "OutOfFlowPositionedFragmentainerDescendants",
        ] {
            assert!(
                owner.contains(owned_policy),
                "out-of-flow positioned-node state lost {owned_policy}"
            );
        }

        let facade =
            std::fs::read_to_string(root.join("layoutng/src/internal/oof_positioned_node.rs"))
                .expect("read OOF positioned-node facade");
        assert!(
            facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/oof_positioned_node.rs"
            )) && facade.contains("pub use layout_algo_out_of_flow_oof_positioned_node::*;"),
            "OOF positioned-node state must be re-exported from the out-of-flow owner"
        );
        for leaked_policy in [
            "pub struct OofContainingBlock",
            "pub struct OofPositionedNode",
            "pub struct FragmentedOofData",
            "fn LogicalOofPositionedNodeToPhysical",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "OOF positioned-node facade still owns algorithm state {leaked_policy}"
            );
        }

        let inline_owner = std::fs::read_to_string(root.join(concat!(
            "layout_algo_",
            "out_of_flow/src/inline_containing_block_utils_interface.rs"
        )))
        .expect("read inline containing-block owner");
        for owned_policy in [
            "pub struct InlineContainingBlockGeometry",
            "pub type InlineContainingBlockMap",
            "pub struct InlineContainingBlockUtils",
            "fn ComputeInlineContainerGeometryProvider",
            "fn ComputeInlineContainerGeometryForFragmentainerProvider",
        ] {
            assert!(
                inline_owner.contains(owned_policy),
                "out-of-flow inline containing-block state lost {owned_policy}"
            );
        }
        let inline_facade = std::fs::read_to_string(
            root.join("layoutng/src/internal/inline_containing_block_utils.rs"),
        )
        .expect("read inline containing-block compatibility facade");
        assert!(
            inline_facade.contains(concat!(
                "layout_algo_",
                "out_of_flow/src/inline_containing_block_utils_interface.rs"
            )) && inline_facade.contains(
                "pub use layout_algo_out_of_flow_inline_containing_block_utils_interface::*;"
            ) && !inline_facade.contains("pub struct InlineContainingBlockGeometry"),
            "inline containing-block state must be re-exported from the out-of-flow owner"
        );
    }

    #[test]
    fn tree_preparation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(root.join("layout_invalidation/src/prepare_tree.rs"))
            .expect("read source invalidation tree-preparation owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn prepare_layout_object_tree",
            "TracePhase::FinalizeRelations",
            "TracePhase::RebindFonts",
            "TracePhase::RefreshImageInputs",
            "TracePhase::PrepareInlineFragments",
            "FinalizeTreeRelations",
            "fn rebind_font",
            "fn rebind_targeted_font",
            "fn rebind_fonts_from_constraint_space",
            "MarkShapeAsDirty",
            "SetShouldCreateBoxFragmentDefault",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source tree preparation lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "tree preparation and both font-rebinding paths must remain visible to sampling profilers"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source layout boundary adapter");
        assert!(
            adapter.contains("layout_invalidation_prepare_tree::prepare_layout_object_tree"),
            "layout boundary must delegate source tree preparation to layout_invalidation"
        );
        for leaked_policy in [
            "NodeMetadataRelations::FinalizeTreeRelations",
            "fn RebindFont(",
            "fn RebindTargetedFont(",
            "fn RebindFontsFromConstraintSpace(",
            "MarkShapeAsDirty()",
            "SetShouldCreateBoxFragmentDefault()",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout boundary still owns source tree-preparation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_destruction_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_view_destruction.rs"),
        )
        .expect("read LayoutView destruction owner");
        for owned_policy in [
            "#[inline(never)]",
            "fn LayoutViewWillBeDestroyed",
            "for object in view.anonymous_page_objects_.iter()",
            "BeingDestroyed()",
            "view.anonymous_page_objects_.clear()",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutView invalidation owner lost {owned_policy}"
            );
        }

        let facade = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView facade");
        assert!(
            facade.contains("layout_invalidation/src/layout_view_destruction.rs")
                && facade.contains("LayoutViewWillBeDestroyed(self)"),
            "LayoutView must delegate anonymous-page teardown to layout_invalidation"
        );
        for leaked_policy in [
            "for object in self.anonymous_page_objects_.iter()",
            "self.anonymous_page_objects_.clear()",
        ] {
            assert!(
                !facade.contains(leaked_policy),
                "LayoutView still owns destruction traversal {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_invalidation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_view_invalidation.rs"),
        )
        .expect("read LayoutView invalidation owner");
        for owned_policy in [
            "fn AffectedByResizedInitialContainingBlock",
            "fn SetContainsAnnotations",
            "fn UpdateFromStyle",
            "fn StyleDidChange",
            "fn InvalidateLayoutForCounterStyleChanges",
            "fn InvalidatePaintForViewAndDescendants",
            "SetNeedsLayout",
            "LayoutViewInvalidateLayoutForCounterStyleChangesProvider",
            "LayoutViewInvalidatePaintForViewAndDescendantsProvider",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutView invalidation owner lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 5,
            "root style and explicit invalidation entries must retain invalidation sampling frames"
        );

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_view_invalidation.rs"),
            "LayoutView must load style invalidation from layout_invalidation"
        );
        for leaked_policy in [
            "fn AffectedByResizedInitialContainingBlock",
            "fn SetContainsAnnotations",
            "fn UpdateFromStyle",
            "fn StyleDidChange",
            "fn InvalidateLayoutForCounterStyleChanges",
            "fn InvalidatePaintForViewAndDescendants",
        ] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns invalidation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_view_child_mutation_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner = std::fs::read_to_string(
            root.join("layout_invalidation/src/layout_view_child_mutation.rs"),
        )
        .expect("read LayoutView child-mutation owner");
        for owned_policy in [
            "#[inline(never)]",
            "fn AddChild",
            "fn IsChildAllowed",
            "fn CanHaveChildren",
            "kPseudoIdViewTransition",
            "self.base_.AddChild",
        ] {
            assert!(
                owner.contains(owned_policy),
                "LayoutView child-mutation owner lost {owned_policy}"
            );
        }

        let loader = std::fs::read_to_string(root.join("layoutng/src/internal/layout_view.rs"))
            .expect("read LayoutView compatibility module");
        assert!(
            loader.contains("layout_invalidation/src/layout_view_child_mutation.rs"),
            "LayoutView must load structural mutation from layout_invalidation"
        );
        for leaked_policy in ["fn AddChild", "fn IsChildAllowed", "fn CanHaveChildren"] {
            assert!(
                !loader.contains(leaked_policy),
                "LayoutView core still owns child-mutation policy {leaked_policy}"
            );
        }
    }

    #[test]
    fn layout_object_factory_resolves_to_the_invalidation_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_invalidation/src/layout_object_factory.rs"))
                .expect("read source LayoutObject factory owner");
        for owned_policy in [
            "fn CreateObject(",
            "fn CreateBlockFlowOrListItem",
            "pub(crate) fn CreateLayoutObject",
            "LayoutObjectCreateObjectProvider",
            "LayoutObjectCreateBlockFlowOrListItemProvider",
            "factories.inline_text",
            "factories.forms",
            "factories.svg",
            "factories.table",
            "factories.grid",
            "EDisplay::kLayoutCustom",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_invalidation source object factory lost {owned_policy}"
            );
        }
        assert!(
            owner.matches("#[inline(never)]").count() >= 3,
            "element, block/list, and boundary factory selection must retain invalidation sampling frames"
        );

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source layout boundary adapter");
        assert!(
            adapter.contains("layout_invalidation/src/layout_object_factory.rs")
                && adapter
                    .contains("layout_invalidation_layout_object_factory::CreateLayoutObject"),
            "layout boundary must re-export object creation from layout_invalidation"
        );
        for leaked_policy in [
            "fn CreateLayoutObject(",
            "factories.inline_text",
            "factories.svg",
        ] {
            assert!(
                !adapter.contains(leaked_policy),
                "layout boundary still owns object-factory policy {leaked_policy}"
            );
        }

        let core = std::fs::read_to_string(root.join("layoutng/src/internal/layout_object.rs"))
            .expect("read LayoutObject compatibility module");
        for leaked_policy in ["fn CreateObject(", "fn CreateBlockFlowOrListItem"] {
            assert!(
                !core.contains(leaked_policy),
                "LayoutObject core still owns factory entry {leaked_policy}"
            );
        }
    }

    #[test]
    fn initial_letter_fitting_resolves_to_the_inline_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let owner =
            std::fs::read_to_string(root.join("layout_algo_inline/src/initial_letter_style.rs"))
                .expect("read source initial-letter style owner");
        for owned_policy in [
            "#[inline(never)]",
            "pub(super) fn InitialLetterTextStyle",
            "desired_cap_height",
            "CurrentFontResolver",
            "while adjusted_size > 1.0",
            "SetVerticalAlign",
        ] {
            assert!(
                owner.contains(owned_policy),
                "layout_algo_inline initial-letter fitting lost {owned_policy}"
            );
        }

        let adapter =
            std::fs::read_to_string(root.join("layoutng/src/internal/boundary/layout_boundary.rs"))
                .expect("read source layout boundary adapter");
        assert!(
            adapter.contains("layout_algo_inline/src/initial_letter_style.rs")
                && adapter
                    .contains("layout_algo_inline_initial_letter_style::InitialLetterTextStyle"),
            "layout boundary must import initial-letter fitting from layout_algo_inline"
        );
        assert!(
            !adapter.contains("fn InitialLetterTextStyle("),
            "layout boundary still owns initial-letter font fitting"
        );
    }

    #[test]
    fn commit_vocabulary_does_not_absorb_fragment_work() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let mut sources = Vec::new();
        rust_sources(root, &mut sources);
        let fragment_phases = [
            "UpdateStickyOffsets",
            "PopulateFragmentPaintData",
            "ResolveLogicalParents",
            "PopulateGlobalPaintData",
            "CertifyPaintInputs",
            "BuildScrollIndex",
            "RefreshScrollCullData",
        ];
        let mut violations = Vec::new();
        for source in sources {
            let contents = std::fs::read_to_string(&source).expect("read Layout source");
            for phase in fragment_phases {
                let obsolete = ["layout", "_commit.", phase].concat();
                if contents.contains(&obsolete) {
                    violations.push(format!("{} contains {obsolete}", source.display()));
                }
            }
        }
        assert!(
            violations.is_empty(),
            "fragment work attributed to commit:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn layout_fragment_filenames_do_not_invent_boundary_prefixes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("layoutng package root");
        let directory = root.join("layout_fragment/src");
        let mut files = std::fs::read_dir(directory)
            .expect("read layout_fragment sources")
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| {
                ["source_", "adapter_", "bridge_"]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                    && name.ends_with(".rs")
            })
            .collect::<Vec<_>>();
        files.sort();
        assert!(
            files.is_empty(),
            "layout_fragment filenames must follow Chromium entities without invented boundary prefixes: {files:?}"
        );
    }
}

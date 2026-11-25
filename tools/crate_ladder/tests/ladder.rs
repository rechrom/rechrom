//! Crate ladder (AGENTS.md 1.7).
//!
//! Every workspace crate has one tier. Value roots may be named by anyone
//! and name only roots; an engine package names roots, its own engine's
//! packages (internal packages never name their own facade) and the
//! facades its contract lists in `uses`. Pipeline engines list only their
//! direct upstream stage; requests further away travel through the
//! neighbour's channel (the Proxy queue or the input queue). Assembly
//! crates wire everything together. Only normal and build dependencies
//! count: tests may name whatever they drive.
//!
//! `ALLOWED` lists the known exceptions, each with its Chromium citation.
//! A forbidden edge outside it fails, and so does an allowance whose edge
//! no longer exists.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Root {
    /// Base types and services every crate may name (`foundation`,
    /// tracing, Skia, executors).
    Base,
    /// Blink platform / Chromium components services below the document
    /// engines (fonts, image decode, canvas values).
    Platform,
    /// cc, viz and document value roots: values and pure algorithms named
    /// by more than one engine.
    Value,
}

const ROOTS: &[(&str, Root)] = &[
    ("foundation", Root::Base),
    ("foundation_base", Root::Base),
    ("browser_tracing", Root::Base),
    ("executor", Root::Base),
    ("skia", Root::Base),
    ("icu_bidi", Root::Base),
    ("xml", Root::Base),
    // third_party/blink/renderer/platform/{fonts,image-decoders,graphics}
    // sit below core; their Rust packages are services every document
    // engine and value root may name.
    ("font_engine", Root::Platform),
    ("image_decoder", Root::Platform),
    ("image_resource", Root::Platform),
    ("canvas", Root::Platform),
    ("web_font", Root::Platform),
    // Document value roots.
    ("layout_geometry", Root::Value),
    ("layout_style", Root::Value),
    ("paint_input_types", Root::Value),
    ("layout_style_paint", Root::Value),
    ("paint_model", Root::Value),
    ("paint_skia", Root::Value),
    ("css_parser", Root::Value),
    ("navigation_history", Root::Value),
    ("fileapi", Root::Value),
    ("intersection_observer", Root::Value),
    ("resize_observer", Root::Value),
    ("webgl", Root::Value),
    ("gpu_webgl", Root::Value),
    // cc shared roots (crates/compositor/README.md).
    ("scroll_snap", Root::Value),
    ("scroll_animation", Root::Value),
    ("compositor_tree", Root::Value),
    ("compositor_values", Root::Value),
    ("compositor_animation", Root::Value),
    ("cc_paint", Root::Value),
    // cc/raster service interfaces (RasterSource, RasterBufferProvider, the
    // UI-resource raster capability) the raster service implements.
    ("cc_raster", Root::Value),
    // components/viz/common: the CompositorFrame values cc fills and Viz and
    // the renderer read (crates/viz/viz_common).
    ("viz_common", Root::Value),
];

struct Engine {
    name: &'static str,
    /// True for the pipeline stages, in order; their `uses` may hold only
    /// the previous stage's facades (plus side engines).
    pipeline: bool,
    facades: &'static [&'static str],
    internals: &'static [&'static str],
    /// Facades of other engines this engine's contract names.
    uses: &'static [&'static str],
}

const ENGINES: &[Engine] = &[
    // Pipeline (AGENTS.md 1.1), in order.
    Engine {
        name: "DOM",
        pipeline: true,
        facades: &["dom", "cssom"],
        internals: &[],
        uses: &[],
    },
    Engine {
        name: "Style",
        pipeline: true,
        facades: &["style"],
        internals: &[
            "style_model",
            "style_parser",
            "style_rules",
            "style_invalidation",
            "style_walk",
            "style_match",
            "style_resolve",
            "style_diff",
            "style_commit",
        ],
        uses: &["dom", "cssom"],
    },
    Engine {
        name: "Layout",
        pipeline: true,
        facades: &["layout"],
        internals: &[
            "layout_base",
            "layout_invalidation",
            "layout_reuse",
            "layout_commit",
            "layout_algo_block",
            "layout_algo_inline",
            "layout_algo_flex",
            "layout_algo_grid",
            "layout_algo_table",
            "layout_algo_multicol",
            "layout_algo_float",
            "layout_algo_out_of_flow",
            "layout_algo_replaced",
            "layout_algo_svg",
            "layout_algo_forms",
            "layout_algo_list",
            "layout_algo_mathml",
            "layout_algo_custom",
        ],
        // Style hands Layout the shared ComputedStyle owner through the
        // `layout_style` root; Layout names no Style package.
        uses: &["style"],
    },
    Engine {
        name: "Paint",
        pipeline: true,
        facades: &["paint"],
        internals: &[
            "prepaint",
            "paint_walk",
            "paint_reuse",
            "paint_record",
            "paint_chunk",
            "paint_commit",
        ],
        uses: &["layout", "animation"],
    },
    Engine {
        name: "Layerize",
        pipeline: true,
        facades: &["layerize"],
        internals: &["compositor_layerize"],
        uses: &["paint"],
    },
    Engine {
        name: "Compositor",
        pipeline: true,
        facades: &["compositor"],
        internals: &[
            "compositor_tree_sync",
            "compositor_tile",
            "compositor_raster",
            "compositor_draw",
            "compositor_frame",
            "compositor_resource",
            "compositor_scheduler",
            "compositor_input",
        ],
        // Only `layerize::commit` (guard: layerize_boundary.rs).
        uses: &["layerize"],
    },
    Engine {
        name: "Viz",
        pipeline: true,
        facades: &["viz"],
        internals: &[],
        // Frames arrive as moved viz_common values; Viz names no cc package
        // (components/viz/service depends on components/viz/common).
        uses: &[],
    },
    Engine {
        name: "Renderer",
        pipeline: true,
        // renderer_skia is the injected backend; it names `renderer`
        // one-way (AGENTS.md 1.3).
        facades: &["renderer", "renderer_skia"],
        internals: &[],
        // Picture playback goes through the raster replay service.
        uses: &["viz", "raster"],
    },
    // Side engines (AGENTS.md 1.4) with the facades their contract names.
    Engine {
        name: "event_loop",
        pipeline: false,
        facades: &["event_loop"],
        internals: &[],
        uses: &[],
    },
    Engine {
        name: "Loader",
        pipeline: false,
        facades: &[
            "html",
            "resource",
            "url_loader",
            "http_transport",
            "xhr_transport",
            "fetch_policy",
            "decode",
            "net_filename",
        ],
        internals: &[],
        // The HTML parser builds into the DOM owner (boundary 1).
        uses: &["dom"],
    },
    Engine {
        name: "interaction",
        pipeline: false,
        facades: &["interaction"],
        internals: &[
            "interaction_input",
            "interaction_hit_test",
            "interaction_dispatch",
            "interaction_default",
            "interaction_scroll",
        ],
        // Hit testing walks the live fragments and PaintLayers.
        uses: &["dom", "cssom", "html", "layout", "paint"],
    },
    Engine {
        name: "javascript",
        pipeline: false,
        facades: &["javascript"],
        internals: &["quickjs", "quickjs_tools"],
        uses: &[],
    },
    Engine {
        name: "webapi",
        pipeline: false,
        facades: &["webapi"],
        internals: &[],
        uses: &[
            "dom",
            "cssom",
            "style",
            "layout",
            "paint",
            "interaction",
            "javascript",
            "xhr_transport",
        ],
    },
    Engine {
        name: "document_image",
        pipeline: false,
        facades: &["document_image"],
        internals: &[],
        // The isolated SVG Style/Layout/Paint lifecycle.
        uses: &["dom", "cssom", "html", "style", "layout", "paint"],
    },
    Engine {
        name: "raster",
        pipeline: false,
        facades: &["raster"],
        internals: &[],
        uses: &[],
    },
    Engine {
        name: "animation",
        pipeline: false,
        facades: &["animation"],
        internals: &[],
        uses: &["cssom"],
    },
];

/// Assembly: the Page, the embedding and the navigation/parser glue.
const ASSEMBLY: &[&str] = &[
    "browser",
    "url_formatter",
    "rechrom_app",
    "devtools",
    "document",
    "document_loader",
    "open",
    "page_mutation",
    "crate_ladder",
];

/// Known exceptions: (from, to, citation). Do not add entries without a
/// Chromium citation; remove an entry in the commit that removes its edge.
const ALLOWED: &[(&str, &str, &str)] = &[
    // DOM hosts Blink's LayoutTreeBuilder through the boundary-3 attach
    // transaction: core/dom/layout_tree_builder.cc creates LayoutObjects
    // (LayoutObject::CreateObject) while the DOM walks its tree.
    ("dom", "layout", "third_party/blink/renderer/core/dom/layout_tree_builder.cc"),
    // FontFace parses its descriptors with the CSS parser:
    // core/css/font_face.cc FontFace::Create -> AtRuleDescriptorParser.
    ("dom", "style_parser", "third_party/blink/renderer/core/css/font_face.cc"),
    // OutOfFlowData (position-try fallback state) is Element rare data in
    // core/css/out_of_flow_data.h holding the last successful try set
    // (`Member<const CSSPropertyValueSet>`); Layout writes it during
    // out-of-flow layout.
    ("layout_base", "cssom", "third_party/blink/renderer/core/css/out_of_flow_data.h"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tier {
    Root(Root),
    Facade(usize),
    Internal(usize),
    Assembly,
}

fn tiers() -> BTreeMap<&'static str, Tier> {
    let mut map = BTreeMap::new();
    let mut insert = |name: &'static str, tier| {
        assert!(map.insert(name, tier).is_none(), "{name} has two tiers");
    };
    for &(name, root) in ROOTS {
        insert(name, Tier::Root(root));
    }
    for (index, engine) in ENGINES.iter().enumerate() {
        for &name in engine.facades {
            insert(name, Tier::Facade(index));
        }
        for &name in engine.internals {
            insert(name, Tier::Internal(index));
        }
    }
    for &name in ASSEMBLY {
        insert(name, Tier::Assembly);
    }
    map
}

fn engine_of(name: &str) -> Option<usize> {
    ENGINES
        .iter()
        .position(|engine| engine.facades.contains(&name) || engine.internals.contains(&name))
}

fn edge_allowed(from: Tier, to: Tier, to_name: &str) -> bool {
    match (from, to) {
        (Tier::Assembly, _) => true,
        (Tier::Root(from), Tier::Root(to)) => match from {
            Root::Base => to == Root::Base,
            Root::Platform => to != Root::Value,
            Root::Value => true,
        },
        (Tier::Root(_), _) => false,
        (_, Tier::Root(_)) => true,
        (_, Tier::Assembly) => false,
        (Tier::Facade(a) | Tier::Internal(a), Tier::Facade(b)) if a == b => {
            matches!(from, Tier::Facade(_))
        }
        (Tier::Facade(a) | Tier::Internal(a), Tier::Internal(b)) => a == b,
        (Tier::Facade(a) | Tier::Internal(a), Tier::Facade(_)) => ENGINES[a].uses.contains(&to_name),
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, c) in line.char_indices() {
        match c {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn quoted_value(text: &str, key: &str) -> Option<String> {
    let mut rest = text;
    while let Some(index) = rest.find(key) {
        let before = rest[..index].chars().next_back();
        let after = rest[index + key.len()..].trim_start();
        if before.map_or(true, |c| !(c.is_alphanumeric() || c == '_' || c == '-'))
            && after.starts_with('=')
        {
            let value = after[1..].trim_start();
            if let Some(value) = value.strip_prefix('"') {
                return value.split('"').next().map(str::to_owned);
            }
        }
        rest = &rest[index + key.len()..];
    }
    None
}

struct Manifest {
    name: String,
    /// Directories of normal/build path dependencies.
    deps: Vec<PathBuf>,
}

fn is_dependency_section(header: &str) -> bool {
    let header = header.trim_matches(|c| c == '[' || c == ']').trim();
    let last = header.rsplit('.').next().unwrap_or("");
    let normal = |s: &str| s == "dependencies" || s == "build-dependencies";
    if normal(header) || (header.starts_with("target.") && normal(last)) {
        return true;
    }
    // `[dependencies.name]` tables.
    header.starts_with("dependencies.") || header.starts_with("build-dependencies.")
}

fn parse_manifest(dir: &Path) -> Manifest {
    let text = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
    let mut name = None;
    let mut section = String::new();
    let mut deps = Vec::new();
    for raw in text.lines() {
        let line = strip_comment(raw).trim();
        if line.starts_with('[') {
            section = line.to_owned();
            continue;
        }
        if section == "[package]" && name.is_none() {
            name = quoted_value(line, "name");
        }
        if is_dependency_section(&section) {
            if let Some(path) = quoted_value(line, "path") {
                deps.push(normalize(&dir.join(path)));
            }
        }
    }
    Manifest { name: name.expect("package name"), deps }
}

fn workspace_members() -> Vec<PathBuf> {
    let text = std::fs::read_to_string(root().join("Cargo.toml")).unwrap();
    let start = text.find("members = [").expect("workspace members");
    let list = &text[start..text[start..].find(']').unwrap() + start];
    list.split('"')
        .skip(1)
        .step_by(2)
        .map(|member| normalize(&root().join(member)))
        .collect()
}

fn graph() -> BTreeMap<String, BTreeSet<String>> {
    let members = workspace_members();
    let manifests: Vec<_> = members.iter().map(|dir| (dir.clone(), parse_manifest(dir))).collect();
    let by_dir: BTreeMap<_, _> = manifests.iter().map(|(dir, m)| (dir.clone(), m.name.clone())).collect();
    manifests
        .iter()
        .map(|(_, manifest)| {
            let deps = manifest
                .deps
                .iter()
                // Vendored third-party crates outside the workspace are
                // not engine packages.
                .filter_map(|dir| by_dir.get(dir).cloned())
                .collect();
            (manifest.name.clone(), deps)
        })
        .collect()
}

#[test]
fn every_crate_has_a_tier_and_the_table_is_a_ladder() {
    let tiers = tiers();
    let missing: Vec<_> = graph().keys().filter(|name| !tiers.contains_key(name.as_str())).cloned().collect();
    assert!(missing.is_empty(), "crates without a ladder tier: {missing:?}");
    // Neighbours only: a pipeline stage names at most the previous stage's
    // facades among the pipeline engines.
    let pipeline: Vec<_> = ENGINES.iter().enumerate().filter(|(_, e)| e.pipeline).map(|(i, _)| i).collect();
    for (position, &index) in pipeline.iter().enumerate() {
        for used in ENGINES[index].uses {
            let target = engine_of(used).unwrap_or_else(|| panic!("{used} is not an engine facade"));
            if ENGINES[target].pipeline {
                assert!(
                    position > 0 && pipeline[position - 1] == target,
                    "{} may name only its direct upstream stage, not {used}",
                    ENGINES[index].name
                );
            }
        }
    }
}

#[test]
fn dependencies_follow_the_ladder() {
    let tiers = tiers();
    let mut violations = Vec::new();
    let mut used_allowances = BTreeSet::new();
    for (from, deps) in graph() {
        let Some(&from_tier) = tiers.get(from.as_str()) else { continue };
        for to in deps {
            let Some(&to_tier) = tiers.get(to.as_str()) else { continue };
            if edge_allowed(from_tier, to_tier, &to) {
                continue;
            }
            if let Some(entry) = ALLOWED.iter().find(|(a, b, _)| *a == from && *b == to) {
                used_allowances.insert((entry.0, entry.1));
                continue;
            }
            violations.push(format!("{from} -> {to}"));
        }
    }
    let stale: Vec<_> = ALLOWED
        .iter()
        .filter(|(a, b, _)| !used_allowances.contains(&(*a, *b)))
        .map(|(a, b, _)| format!("{a} -> {b}"))
        .collect();
    assert!(violations.is_empty(), "forbidden crate edges:\n{}", violations.join("\n"));
    assert!(stale.is_empty(), "stale ladder allowances (remove them):\n{}", stale.join("\n"));
}

/// Page document-side modules: everything in `crates/browser/src/page.rs`
/// and `crates/browser/src/page/` except the modules that own the impl
/// `CompositorEngine` instance (the Page's synchronous presenter and its
/// compositor input owner). The document side reaches the compositor only
/// through `layerize::commit` (Proxy queue) and the input queue.
const IMPL_OWNER_MODULES: &[&str] = &[
    "crates/browser/src/page/synchronous_presenter.rs",
    "crates/browser/src/page/compositor_input.rs",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn names_compositor(text: &str) -> bool {
    text.lines().any(|line| {
        let code = line.split("//").next().unwrap_or("");
        code.match_indices("compositor::").any(|(index, _)| {
            !code[..index].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':')
        })
    })
}

#[test]
fn page_document_side_names_no_compositor() {
    let browser = root().join("crates/browser/src");
    let mut found = vec![browser.join("page.rs")];
    rust_files(&browser.join("page"), &mut found);
    let mut violations = Vec::new();
    for path in found {
        let relative = path.strip_prefix(root()).unwrap().to_string_lossy().replace('\\', "/");
        if IMPL_OWNER_MODULES.contains(&relative.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        if names_compositor(&text) {
            violations.push(relative);
        }
    }
    assert!(violations.is_empty(), "Page document-side files naming compositor::\n{}", violations.join("\n"));
}

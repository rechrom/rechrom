#!/usr/bin/env python3
"""Inventory Blink core/css translation evidence in crates/style.

This script deliberately keeps three different claims separate:

* an annotated counterpart only says that a Rust file cites a Blink source;
* an audited ledger records mapped/omitted/pending effective source lines;
* live-path evidence is a small, manually reviewed list of translated engines
  reached by DocumentStyleEngine's production path.

Bridge, projection, assembly, and test files do not count as 1:1 translations.
The default Blink checkout is the sibling ``../src`` Chromium checkout used by
this workspace. Pass ``--chromium-src`` when the checkout lives elsewhere.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path


SOURCE_SUFFIXES = {".h", ".cc"}
EXCLUDED_SOURCE_NAME_PARTS = ("_test.", "_unittest.", "fuzzer", "mock_", "test_")
EXCLUDED_RUST_NAMES = (
    "_test.rs",
    "document_style_engine.rs",
    "persistent_selector.rs",
    "production_",
)

# These classifications are intentionally narrow. They describe the current
# DocumentStyleEngine call chain, not whether a module compiles or has unit
# tests. Update them only after following the production call sites.
LIVE_PATH_RUST = {
    "parser/css_parser_impl.rs": "ParseCSS retains its typed StyleSheetContents",
    "parser/css_selector_parser.rs": "production parser compiles selectors",
    "style_sheet_contents.rs": "retained owner builds and caches RuleSet",
    "rule_set.rs": "active sheet revisions build production RuleSets",
    "selector_checker.rs": "ordinary standards/UA candidates use SelectorChecker",
    "resolver/cascade_map.rs": "DocumentStyleEngine cascade selects winners with CascadeMap",
    "resolver/cascade_priority.rs": "DocumentStyleEngine uses translated cascade priority",
    "resolver/cascade_expansion.rs": "production declarations expand through translated ExpandCascade",
    "resolver/match_result.rs": "production declarations are represented by translated MatchResult",
}

PARTIALLY_LIVE_RUST = {
    "element_rule_collector.rs": (
        "ordinary standards-mode author and typed UA element matching plus "
        "::before/::after/::first-letter, explicit and implicit @scope, "
        "rule-level container queries and starting-style call CollectMatchingRules "
        "and SortAndTransferMatchedRules; the translated StyleResolver traverses retained "
        "TreeScope, slot and PartNames owners in Host, Slotted, ElementScope and Outer order, "
        "including constructed/adopted sheets, nested exportparts name forwarding and the "
        "retained UA-shadow ::placeholder owner"
    ),
    "resolver/style_resolver.rs": (
        "DocumentStyleEngine root initialization calls InitialStyleForElement; "
        "DocumentStyleEngine::Update executes MatchAllRules and publishes "
        "ApplyBaseStyleNoCache as the final style for complete standards-mode HTML owner bundles, "
        "including inline style, scope, container and starting-style inputs, with the root, "
        "custom-property, substitution, "
        "image, property/function registry, animation and functional-container "
        "collaborator bundle; container lookup follows retained flat-tree ancestry and starts "
        "pseudo queries at the originating element; Update also uses retained named and manual "
        "slot assignment for flat-tree style "
        "parents and initial style for unassigned light children and inactive slot fallback roots; "
        "one translated resolver/cascade trial produces the style and "
        "custom-property map as a pair, and only an atomic trial failure invokes the compatibility "
        "path; the production backend supplies concrete TreeScope, ShadowRoot, slot and PartNames "
        "owners to the source MatchAuthorRules call order across all active sheet scopes; resolver "
        "state retains tree-scoped-reference, conditional-animation and author-style flags, and "
        "the browser's fixed-commit html.css, quirks.css, forced_colors.css, marker.css, fullscreen.css, svg.css and mathml.css "
        "resources retain distinct "
        "typed UA owners through CSSStyleSheet role metadata and the unchanged StyleEngine "
        "sheet-slice boundary; DocumentStyleEngine assembles separate HTML, HTML-quirks, forced-colors, "
        "marker, fullscreen, SVG and MathML RuleSets; MatchUARules selects the namespace-specific base, quirks only for a "
        "quirks document, and the theme sheet only for an active forced-colors environment using the "
        "source ua-forced-colors medium; fullscreen selection reads the retained DOM fullscreen stack, while an unknown/duplicate role, "
        "missing specialized owner or unsupported matched parser branch atomically falls back; "
        "the HTML UA value path accepts __qem only in UA mode, preserves its margin quirk bit, "
        "applies -internal-center through parent direction and resolves -internal-auto-base() "
        "against the retained element/input/select owner, including the multiple size=1 select exception; "
        "-internal-textarea-auto also remains UA-only and resolves through the retained document "
        "TextAreasAreResizable setting into native EResize; "
        "font system keywords retain CSSPendingSystemFontValue across every font longhand; on the "
        "non-Windows LayoutTheme provider path, -webkit-small-control and its sibling control "
        "keywords resolve the source Arial family and document DefaultFontSize minus two points "
        "into the native FontDescription, while Windows live FontCache ownership remains an atomic "
        "fallback; "
        "ordinary HTML appearance:auto retains the element/input/select type owner through the "
        "source LayoutTheme adjustment: effective appearances, author-styled suppression, display "
        "coercion and shared checkbox/radio/button/menulist geometry reach the native builder, while "
        "platform-selected temporal views and platform-sized UA-shadow parts fall back atomically; "
        "the source ApplyMathMLCustomStyleProperties dispatch reaches typed mspace, mpadded, "
        "mfrac and mo length-attribute conversion and ComputedStyleBuilder fields, while "
        "MathMLElement presentation hints for dir, mathsize, mathcolor, mathbackground, "
        "scriptlevel and displaystyle plus the <mi mathvariant=normal> token hint are parsed "
        "into their source CSS longhands and transferred at author-presentational-hint origin; "
        "mspace width/height/depth and mpadded width reuse ParseMathLength's strict primitive "
        "owner, including mspace's source calc(height + depth) synthesis; "
        "the six source UA-shadow pseudo names plus HTML audio/video media ownership are connected; "
        "the retained search/number text-field decoration container also follows the source typed "
        "UA-shadow selector relation into its exact html.css declaration block; "
        "MatchUARules also selects distinct normal/forced-colors media-control RuleSet owners in "
        "their separate UA-origin pass and retains an independent forced-colors supplemental UA "
        "owner; ordinary list-item ::marker requests retain their native pseudo style and layout consumes "
        "that owner, while nested ::before::marker/::after::marker remains outside this closure"
    ),
    "resolver/style_cascade.rs": (
        "typed ordinary, root, pseudo, registered/custom property, var/env/attr, "
        "URL image, numeric/conditional, functional-container, registered CSS "
        "function, @mixin/@apply binding and sampled animation subsets run StyleCascade::Apply in "
        "production; animated and transitioned custom properties are applied lazily into the same retained "
        "CustomProperties owner before dependent var() longhands resolve, including animation-taint rules; "
        "DocumentStyleEngine::Update; unsupported typed "
        "backend cases atomically discard the trial and use the document "
        "compatibility application"
    ),
}

KNOWN_NOT_CONNECTED_RUST = {
    "style_engine.rs": "browser exports DocumentStyleEngine as StyleEngine",
}

# This is deliberately separate from the source ledger.  The eight files in
# CORE_STYLE_CLOSURE_SOURCES can have a closed source-accounting ledger while
# DocumentStyleEngine still enters only part of their translated control flow.
# Keep source method names here so the report cannot turn "pending=0" into a
# claim that the whole browser path is connected.
CORE_STYLE_CLOSURE_PRODUCTION_GAPS = {
    "element_rule_collector.rs": [
        (
            "pseudo families beyond ::before, ::after and ::first-letter; "
            "the retained UA-shadow ::placeholder and ::-internal-input-suggested owners cover the "
            "ordinary placeholder and suggested-value paths; native platform text-control paint/size "
            "owners remain outside this collector"
        ),
    ],
    "resolver/style_resolver.rs": [
        (
            "DocumentStyleEngine::Update publishes translated MatchAllRules/"
            "ApplyBaseStyleNoCache results for complete supported HTML, SVG and MathML bundles, with style "
            "and computed custom properties returned by one atomic trial; quirks-mode author sheets "
            "and incomplete owner bundles still use compatibility; fixed-commit html.css, quirks.css, "
            "forced_colors.css, marker.css, fullscreen.css, svg.css and mathml.css now reach MatchUARules as distinct typed RuleSet "
            "owners, with quirks, forced-colors and fullscreen selected from real document/media/Fullscreen API state and a missing, "
            "unknown or duplicate specialized owner atomically falling back. quirks.css's UA-only "
            "-internal-quirk-inherit color is mode-gated by the fixed keyword table and resolves from "
            "Document::TextLinkColors' body-current-color owner; its remaining margin, table font/text, "
            "box-sizing and list-position declarations already use the common typed longhand owners. "
            "html.css resource identity "
            "does not imply complete HTML UA semantics: __qem, -internal-center and the retained-element "
            "-internal-auto-base() branch plus the document-owned -internal-textarea-auto resize value "
            "are connected; non-Windows system-font keywords, including the high-impact "
            "-webkit-small-control form font, retain their pending marker through shorthand expansion "
            "and resolve through the document font-size owner; html.css's direct max()/min() 1lh "
            "form-control sizing branches resolve from the current native "
            "computed line height, and rlh resolves from the retained document-root ComputedStyle's "
            "unzoomed computed line height; both preserve their source dependency bits. calc-size()/attr()-"
            "dependent line-height-unit branches remain gated on those function owners. Windows live "
            "system-font lookup, unsupported internal "
            "selectors and remaining platform-specific value/post-adjustment "
            "owners still fall back atomically. The ordinary form-control appearance:auto path is "
            "connected through retained element type and common LayoutTheme geometry adjustment; "
            "temporal input view selection, UA-shadow controls without retained implementation "
            "elements and native-sized UA-shadow appearance parts remain gaps. Forced-colors computed-style "
            "adjustment now uses the native builder for shadow/scrollbar/color-scheme/accent/background and "
            "internal forced system colors through a package-private typed color provider"
        ),
        (
            "StyleResolver::ResolveStyle, ResolveBaseStyle, ApplyBaseStyle, "
            "ApplyAnimatedStyle and the complete lifecycle remain only partially connected; the "
            "production shared MatchedPropertiesCache is connected for supported cacheable results, "
            "while complete cache admission/eviction semantics and real visited-link state/property "
            "ownership are not supplied"
        ),
        (
            "VTT owners, the modules media-control UA stylesheet loader and retained internal "
            "media-control DOM, remaining platform text-control implementation elements, other "
            "specialized UA rule sets "
            "and several animation, anchor and post-adjustment collaborators remain typed fallbacks "
            "or unimplemented production owners; until those resources are connected, their "
            "selected supplemental RuleSet owners are empty, which accounts only for the independent "
            "set ownership and does not count the media-controls resource as implemented; the "
            "theme forced-colors resource itself is now an exact browser-owned supplemental RuleSet"
        ),
        (
            "SVG and MathML own their fixed-commit default stylesheets, and MathML mspace/mpadded/"
            "mfrac/mo custom length post-processing reaches typed ComputedStyle fields; the common "
            "MathMLElement presentation attributes and <mi mathvariant=normal> also reach the "
            "translated cascade, including mspace width/height/depth and mpadded width. The MathML "
            "operator dictionary/form/boolean-property layout owner and other specialized element "
            "adjustments remain incomplete, so these connected owners do not by themselves establish "
            "complete SVG/MathML semantic 1:1"
        ),
        (
            "the retained DOM owns named and manual slot assignment and Update consumes both "
            "for flat-tree traversal and inheritance; the JavaScript HTMLSlotElement.assign() "
            "binding surface remains unimplemented"
        ),
    ],
    "resolver/style_cascade.rs": [
        "navigation-query/update evaluation used by if() and function bodies",
        (
            "typed branches that still report atomic fallbacks, including color "
            "images, shadow-root context and unsupported numeric "
            "calculation nodes"
        ),
    ],
}

# Source files for the production closure currently being wired behind the
# public StyleEngine boundary. Keeping this explicit makes its audited totals
# reproducible without confusing them with the broader core/css inventory.
CORE_STYLE_CLOSURE_SOURCES = (
    "element_rule_collector.h",
    "element_rule_collector.cc",
    "resolver/match_request.h",
    "resolver/match_request.cc",
    "resolver/style_cascade.h",
    "resolver/style_cascade.cc",
    "resolver/style_resolver.h",
    "resolver/style_resolver.cc",
)

CPP_REF = re.compile(r"([A-Za-z0-9_./-]+\.(?:h|cc))(?:[:][0-9][0-9, -]*)?")
LEDGER_SLASH = re.compile(
    r"(?P<src>[A-Za-z0-9_./-]+\.(?:h|cc))\s*:\s*"
    r"(?P<physical>\d+)\s*/\s*(?P<effective>\d+)\s*/\s*"
    r"(?P<mapped>\d+)\s*/\s*(?P<omitted>\d+)\s*/\s*(?P<pending>\d+)",
    re.I,
)
LEDGER_WORDS = re.compile(
    r"(?P<src>[A-Za-z0-9_./-]+\.(?:h|cc))\s*:\s*"
    r"physical\s*[=:]?\s*(?P<physical>\d+)[\s;,]+"
    r"effective\s*[=:]?\s*(?P<effective>\d+)[\s;,]+"
    r"mapped\s*[=:]?\s*(?P<mapped>\d+)"
    r"(?:[\s;,]+omitted(?:\s+[A-Za-z-]+){0,4}\s*[=:]?\s*(?P<omitted>\d+))?"
    r"(?:[\s;,]+(?:(?:production\s+)?pending|remaining)\s*[=:]?\s*(?P<pending>\d+))?",
    re.I,
)
LEDGER_REMAINING = re.compile(
    r"(?P<src>[A-Za-z0-9_./-]+\.(?:h|cc))\s*:\s*"
    r"physical\s+(?P<physical>\d+)\s*[;,]\s*"
    r"effective\s+(?P<effective>\d+)\s*[;,]\s*"
    r"mapped\s+(?P<mapped>\d+)\s*[;,]\s*"
    r"remaining\s+(?P<pending>\d+)",
    re.I,
)
LEDGER_COMPLETE_TABLE = re.compile(
    r"Complete\s+source\s+(?P<kind>h|cc)\s*:\s*"
    r"(?P<physical>\d+)\s+(?P<effective>\d+)\s+(?P<mapped>\d+)\s+"
    r"(?P<omitted>\d+)\s+(?P<pending>\d+)",
    re.I,
)
LEDGER_SHORT_WORDS = re.compile(
    r"(?P<kind>h|cc)\s*:\s*(?P<physical>\d+)\s+physical\s*[;,]\s*"
    r"(?P<effective>\d+)\s+effective\s*[;,]\s*"
    r"(?P<mapped>\d+)\s+mapped\s*[;,]\s*"
    r"(?P<omitted>\d+)(?:\s+[A-Za-z-]+){0,6}\s+omitted",
    re.I,
)
LEDGER_CREDITED = re.compile(
    r"(?P<src>[A-Za-z0-9_./-]+\.(?:h|cc))\s*:\s*"
    r"(?P<physical>\d+)\s*/\s*(?P<effective>\d+)\s*/\s*"
    r"(?P<mapped>\d+)\s*/\s*(?P<pending>\d+)",
    re.I,
)
LEDGER_COMMIT = re.compile(r"Chromium\s+commit\s+([0-9a-f]{40})", re.I)


def is_source_candidate(path: Path, root: Path) -> bool:
    if path.suffix not in SOURCE_SUFFIXES:
        return False
    rel = path.relative_to(root).as_posix()
    name = path.name.lower()
    return not any(part in name for part in EXCLUDED_SOURCE_NAME_PARTS) and "/testing/" not in f"/{rel}"


def effective_lines(path: Path) -> int:
    """Approximate the ledger's effective-line convention reproducibly."""
    in_block = False
    count = 0
    for raw in path.read_text(errors="ignore").splitlines():
        line = raw
        out = []
        i = 0
        while i < len(line):
            if in_block:
                end = line.find("*/", i)
                if end < 0:
                    i = len(line)
                else:
                    in_block = False
                    i = end + 2
            else:
                block = line.find("/*", i)
                single = line.find("//", i)
                if single >= 0 and (block < 0 or single < block):
                    out.append(line[i:single])
                    i = len(line)
                elif block >= 0:
                    out.append(line[i:block])
                    in_block = True
                    i = block + 2
                else:
                    out.append(line[i:])
                    i = len(line)
        code = "".join(out).strip()
        if not code or code.startswith("#"):
            continue
        if re.fullmatch(r"(?:namespace(?:\s+[A-Za-z0-9_:]+)?\s*)?[{};]*", code):
            continue
        count += 1
    return count


def nonblank_comment_stripped_lines(path: Path) -> int:
    """Count nonblank code after comments, retaining directives/delimiters."""
    in_block = False
    count = 0
    for raw in path.read_text(errors="ignore").splitlines():
        line = raw
        out = []
        i = 0
        while i < len(line):
            if in_block:
                end = line.find("*/", i)
                if end < 0:
                    i = len(line)
                else:
                    in_block = False
                    i = end + 2
            else:
                block = line.find("/*", i)
                single = line.find("//", i)
                if single >= 0 and (block < 0 or single < block):
                    out.append(line[i:single])
                    i = len(line)
                elif block >= 0:
                    out.append(line[i:block])
                    in_block = True
                    i = block + 2
                else:
                    out.append(line[i:])
                    i = len(line)
        if "".join(out).strip():
            count += 1
    return count


def normalized_comment_text(text: str) -> str:
    lines = []
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("//"):
            stripped = stripped[2:].strip()
        elif stripped.startswith("/*") or stripped.startswith("*") or stripped.endswith("*/"):
            stripped = stripped.strip("/* ")
        lines.append(stripped)
    return " ".join(lines)


def source_index(source_root: Path) -> tuple[dict[str, Path], dict[str, list[Path]]]:
    """Index only files tracked by the Chromium revision being reported.

    A checkout may contain generated or experimental untracked `.cc` files.
    Counting those while labeling the report with `git rev-parse HEAD` makes
    the inventory depend on local debris rather than the named revision.
    """
    by_rel = {}
    by_name: dict[str, list[Path]] = defaultdict(list)
    git_root = Path(
        subprocess.check_output(
            ["git", "-C", str(source_root), "rev-parse", "--show-toplevel"],
            text=True,
        ).strip()
    )
    source_prefix = source_root.relative_to(git_root).as_posix()
    tracked = subprocess.check_output(
        ["git", "-C", str(git_root), "ls-files", "--", source_prefix],
        text=True,
    ).splitlines()
    for tracked_path in tracked:
        path = git_root / tracked_path
        if is_source_candidate(path, source_root):
            rel = path.relative_to(source_root).as_posix()
            by_rel[rel] = path
            by_name[path.name].append(path)
    return by_rel, by_name


def resolve_source(raw: str, by_rel: dict[str, Path], by_name: dict[str, list[Path]], source_root: Path) -> str | None:
    raw = raw.strip().lstrip("/")
    for marker in ("third_party/blink/renderer/core/css/", "core/css/"):
        if marker in raw:
            raw = raw.split(marker, 1)[1]
    if raw in by_rel:
        return raw
    matches = by_name.get(Path(raw).name, [])
    if len(matches) == 1:
        return matches[0].relative_to(source_root).as_posix()
    return None


def is_bridge_or_test(path: Path) -> bool:
    rel = path.as_posix()
    name = path.name
    return any(part in name or part in rel for part in EXCLUDED_RUST_NAMES)


def collect_evidence(style_src: Path, source_root: Path):
    by_rel, by_name = source_index(source_root)
    annotations: dict[str, set[str]] = defaultdict(set)
    ledgers: dict[str, dict] = {}
    rust_state = {}

    for rust in sorted(style_src.rglob("*.rs")):
        rel_rust = rust.relative_to(style_src).as_posix()
        if is_bridge_or_test(rust):
            continue
        text = rust.read_text(errors="ignore")
        declared_commits = sorted(set(LEDGER_COMMIT.findall(text)))
        declared_commit = declared_commits[0] if len(declared_commits) == 1 else None
        rust_state[rel_rust] = (
            "live" if rel_rust in LIVE_PATH_RUST else
            "partially_live" if rel_rust in PARTIALLY_LIVE_RUST else
            "translated_not_connected" if rel_rust in KNOWN_NOT_CONNECTED_RUST else
            "not_audited"
        )
        for line in text.splitlines():
            if "cpp:" not in line.lower():
                continue
            tail = re.split(r"cpp:", line, flags=re.I, maxsplit=1)[1]
            for match in CPP_REF.finditer(tail):
                resolved = resolve_source(match.group(1), by_rel, by_name, source_root)
                if resolved:
                    annotations[resolved].add(rel_rust)

        comment = normalized_comment_text(text)
        for pattern in (LEDGER_SLASH, LEDGER_WORDS, LEDGER_REMAINING):
            for match in pattern.finditer(comment):
                values = match.groupdict()
                resolved = resolve_source(values["src"], by_rel, by_name, source_root)
                if not resolved:
                    continue
                record = {
                    key: int(value)
                    for key, value in values.items()
                    if key != "src" and value is not None
                }
                record.setdefault("omitted", 0)
                record.setdefault(
                    "pending",
                    max(0, record["effective"] - record["mapped"] - record["omitted"]),
                )
                record["declared_commit"] = declared_commit
                record["rust_file"] = rel_rust
                previous = ledgers.get(resolved)
                if previous and previous != record:
                    raise RuntimeError(f"conflicting ledgers for {resolved}: {previous} vs {record}")
                ledgers[resolved] = record
                annotations[resolved].add(rel_rust)

        # A few older ledgers use the compact table from their header rather
        # than repeating the source basename. Resolve those rows from the Rust
        # module's relative path; this is only accepted when that exact Blink
        # source exists. Batch/subtotal rows are deliberately ignored.
        inferred_stem = rel_rust.removesuffix(".rs")
        for pattern in (LEDGER_COMPLETE_TABLE, LEDGER_SHORT_WORDS):
            for match in pattern.finditer(comment):
                values = match.groupdict()
                resolved = resolve_source(
                    f"{inferred_stem}.{values['kind']}", by_rel, by_name, source_root
                )
                if not resolved:
                    continue
                record = {
                    key: int(value)
                    for key, value in values.items()
                    if key != "kind" and value is not None
                }
                record.setdefault("pending", 0)
                record["declared_commit"] = declared_commit
                record["rust_file"] = rel_rust
                previous = ledgers.get(resolved)
                if previous and previous != record:
                    raise RuntimeError(f"conflicting ledgers for {resolved}: {previous} vs {record}")
                ledgers[resolved] = record
                annotations[resolved].add(rel_rust)

        # css_selector_parser's conservative full-file audit predates the
        # five-column format and labels its third column "credited". Do not
        # accept arbitrary four-column selected-range ledgers as whole files.
        if "physical / effective / credited / pending" in comment.lower():
            for match in LEDGER_CREDITED.finditer(comment):
                values = match.groupdict()
                resolved = resolve_source(values["src"], by_rel, by_name, source_root)
                if not resolved:
                    continue
                record = {
                    "physical": int(values["physical"]),
                    "effective": int(values["effective"]),
                    "mapped": int(values["mapped"]),
                    "omitted": 0,
                    "pending": int(values["pending"]),
                    "rust_file": rel_rust,
                    "declared_commit": declared_commit,
                }
                previous = ledgers.get(resolved)
                if previous and previous != record:
                    raise RuntimeError(f"conflicting ledgers for {resolved}: {previous} vs {record}")
                ledgers[resolved] = record
                annotations[resolved].add(rel_rust)

    rows = []
    for rel, path in sorted(by_rel.items()):
        ledger = ledgers.get(rel)
        owners = sorted(annotations.get(rel, ()))
        states = {rust_state.get(owner, "not_audited") for owner in owners}
        if "live" in states:
            connection = "live"
        elif "partially_live" in states:
            connection = "partially_live"
        elif "translated_not_connected" in states:
            connection = "translated_not_connected"
        elif owners:
            connection = "not_audited"
        else:
            connection = "none"
        if ledger:
            translation = "audited_complete" if ledger["pending"] == 0 else "audited_partial"
        elif owners:
            translation = "annotated_unledgered"
        else:
            translation = "unmapped"
        rows.append({
            "source": rel,
            "physical_loc": len(path.read_text(errors="ignore").splitlines()),
            "effective_loc_estimate": effective_lines(path),
            "nonblank_comment_stripped_loc": nonblank_comment_stripped_lines(path),
            "translation": translation,
            "connection": connection,
            "rust_files": owners,
            "ledger": ledger,
        })
    return rows


def validate_ledgers(rows: list[dict], chromium_commit: str) -> dict:
    """Validate claims that can be checked without judging translation semantics.

    A ledger is authored in a Rust source comment.  This function proves its
    arithmetic and that its physical count names the checked-out Chromium
    file.  It intentionally does not certify that each mapped line is a
    semantically exact translation or that every omitted line is non-production
    boilerplate; those require a source review and remain self-attested.
    """
    failures = []
    missing_declared_commit = []
    effective_estimate_differences = []
    for row in rows:
        ledger = row["ledger"]
        if ledger is None:
            continue
        if ledger["physical"] != row["physical_loc"]:
            failures.append({
                "source": row["source"],
                "check": "physical_loc",
                "ledger": ledger["physical"],
                "actual": row["physical_loc"],
            })
        if ledger["declared_commit"] is None:
            missing_declared_commit.append(row["source"])
        elif ledger["declared_commit"] != chromium_commit:
            failures.append({
                "source": row["source"],
                "check": "declared_chromium_commit",
                "ledger": ledger["declared_commit"],
                "actual": chromium_commit,
            })
        accounted = ledger["mapped"] + ledger["omitted"] + ledger["pending"]
        if accounted != ledger["effective"]:
            failures.append({
                "source": row["source"],
                "check": "mapped_plus_omitted_plus_pending",
                "ledger_effective": ledger["effective"],
                "accounted": accounted,
            })
        if ledger["effective"] != row["effective_loc_estimate"]:
            effective_estimate_differences.append({
                "source": row["source"],
                "ledger_effective": ledger["effective"],
                "lexical_estimate": row["effective_loc_estimate"],
                "difference": ledger["effective"] - row["effective_loc_estimate"],
            })
    return {
        "checked": [
            "ledger physical_loc equals the tracked Chromium file",
            "mapped_loc + omitted_loc + pending_loc equals ledger effective_loc",
            "declared Chromium commit equals the checked-out Chromium commit when present",
        ],
        "failures": failures,
        "missing_declared_commit": missing_declared_commit,
        "not_certified": [
            "semantic 1:1 equivalence of every mapped source line",
            "legitimacy of every omitted source line",
            "production reachability of translated methods",
        ],
        "effective_estimate_differences": effective_estimate_differences,
    }


def summary(rows: list[dict], chromium_src: Path) -> dict:
    chromium_commit = subprocess.check_output(
        ["git", "-C", str(chromium_src), "rev-parse", "HEAD"], text=True
    ).strip()
    result = {
        "chromium_commit": chromium_commit,
        "scope": "third_party/blink/renderer/core/css production .h/.cc; tests, fuzzers, mocks excluded",
        "effective_loc": "reproducible lexical estimate; authoritative mapped/pending values come only from explicit ledgers",
        "bridge_policy": "tests, document_style_engine, persistent_selector and production_* assembly/projection files excluded as translation evidence",
        "total": {},
        "by_translation": {},
        "by_connection": {},
        "audited_ledger": {},
        "audited_pending_sources": [],
        "core_style_closure_ledger": {},
        "live_path_evidence": LIVE_PATH_RUST,
        "partially_live_path_evidence": PARTIALLY_LIVE_RUST,
        "known_translated_not_connected": KNOWN_NOT_CONNECTED_RUST,
        "core_style_closure_production_gaps": CORE_STYLE_CLOSURE_PRODUCTION_GAPS,
        "ledger_validation": validate_ledgers(rows, chromium_commit),
    }
    result["total"] = {
        "files": len(rows),
        "physical_loc": sum(row["physical_loc"] for row in rows),
        "effective_loc_estimate": sum(row["effective_loc_estimate"] for row in rows),
    }
    for field, target in (("translation", "by_translation"), ("connection", "by_connection")):
        for value in sorted({row[field] for row in rows}):
            selected = [row for row in rows if row[field] == value]
            result[target][value] = {
                "files": len(selected),
                "physical_loc": sum(row["physical_loc"] for row in selected),
                "effective_loc_estimate": sum(row["effective_loc_estimate"] for row in selected),
            }
    completed = [row for row in rows if row["translation"] == "audited_complete"]
    remaining = [row for row in rows if row["translation"] != "audited_complete"]
    result["strict_whole_file_completion"] = {
        "translated": {
            "files": len(completed),
            "physical_loc": sum(row["physical_loc"] for row in completed),
            "effective_loc_estimate": sum(row["effective_loc_estimate"] for row in completed),
        },
        "remaining_or_unverified": {
            "files": len(remaining),
            "physical_loc": sum(row["physical_loc"] for row in remaining),
            "effective_loc_estimate": sum(row["effective_loc_estimate"] for row in remaining),
        },
    }
    ledger_rows = [row["ledger"] for row in rows if row["ledger"]]
    result["audited_ledger"] = {
        "source_files": len(ledger_rows),
        "physical_loc": sum(row["physical"] for row in ledger_rows),
        "effective_loc": sum(row["effective"] for row in ledger_rows),
        "mapped_loc": sum(row["mapped"] for row in ledger_rows),
        "omitted_loc": sum(row["omitted"] for row in ledger_rows),
        "pending_loc": sum(row["pending"] for row in ledger_rows),
    }
    result["audited_pending_sources"] = [
        {
            "source": row["source"],
            "pending_loc": row["ledger"]["pending"],
            "mapped_loc": row["ledger"]["mapped"],
            "omitted_loc": row["ledger"]["omitted"],
        }
        for row in rows
        if row["ledger"] and row["ledger"]["pending"]
    ]
    closure = [row for row in rows if row["source"] in CORE_STYLE_CLOSURE_SOURCES]
    if len(closure) != len(CORE_STYLE_CLOSURE_SOURCES):
        present = {row["source"] for row in closure}
        missing = sorted(set(CORE_STYLE_CLOSURE_SOURCES) - present)
        raise RuntimeError(f"core Style closure sources missing from inventory: {missing}")
    if any(row["ledger"] is None for row in closure):
        missing = sorted(row["source"] for row in closure if row["ledger"] is None)
        raise RuntimeError(f"core Style closure sources missing ledgers: {missing}")
    result["core_style_closure_ledger"] = {
        "source_files": len(closure),
        "translated_files": sum(row["ledger"]["pending"] == 0 for row in closure),
        "remaining_files": sum(row["ledger"]["pending"] != 0 for row in closure),
        "physical_loc": sum(row["ledger"]["physical"] for row in closure),
        "effective_loc": sum(row["ledger"]["effective"] for row in closure),
        "effective_loc_lexical_estimate": sum(
            row["effective_loc_estimate"] for row in closure
        ),
        "effective_loc_nonblank_comment_stripped": sum(
            row["nonblank_comment_stripped_loc"] for row in closure
        ),
        "mapped_loc": sum(row["ledger"]["mapped"] for row in closure),
        "omitted_loc": sum(row["ledger"]["omitted"] for row in closure),
        "pending_loc": sum(row["ledger"]["pending"] for row in closure),
        "sources": [row["source"] for row in closure],
        "claim_scope": (
            "self-attested source-line accounting; pending=0 does not certify "
            "semantic 1:1 equivalence or production reachability"
        ),
        "effective_count_warning": (
            "the eight source ledgers use mixed effective-line conventions; "
            "their effective/mapped/omitted totals are arithmetically additive "
            "but are not one normalized line-coverage denominator"
        ),
        "semantic_1_to_1_verified_files": None,
    }
    return result


def main() -> None:
    repo = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser()
    parser.add_argument("--chromium-src", type=Path, default=repo.parent / "src")
    parser.add_argument("--details", action="store_true", help="include one record per Blink source file")
    args = parser.parse_args()
    chromium_src = args.chromium_src.resolve()
    source_root = chromium_src / "third_party/blink/renderer/core/css"
    if not source_root.is_dir():
        parser.error(f"Blink core/css directory not found: {source_root}")
    rows = collect_evidence(repo / "crates/style/src", source_root)
    output = summary(rows, chromium_src)
    if args.details:
        output["files"] = rows
    print(json.dumps(output, ensure_ascii=False, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

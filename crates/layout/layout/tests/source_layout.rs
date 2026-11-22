//! Source-layout guards.
//!
//! 1. Modules follow the file layout their path implies. Nothing in the
//!    workspace carries a `#[path]` attribute (rustc on macOS and Linux
//!    resolves it through the real filesystem while Windows normalises it
//!    lexically, so the same tree can build on one host and fail on another),
//!    and no `include!` names a `.rs` file. The only Rust `include!` allowed
//!    is build-script output: `include!(concat!(env!("OUT_DIR"), ...))`.
//! 2. Every Layout package's module tree resolves inside its own package:
//!    each `mod name;` has exactly one of `name.rs` / `name/mod.rs`, each
//!    source file is compiled exactly once and no file under `src/` is an
//!    orphan.
//! 3. No `#[no_mangle]` item of one Layout package defines a name that another
//!    Layout package declares in an `extern` block (a link-time pair instead
//!    of a typed dependency).
//!
//! The scans are line based and rely on rustfmt layout (one item per line,
//! an inline module's closing brace at its own indentation).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

/// Files allowed to define `#[no_mangle]` symbols that another Layout package
/// declares. `layout_base`'s untranslated-symbol stubs abort with the symbol
/// name; they exist only because MSVC link.exe resolves every referenced
/// external, including `layout_style` declarations whose Chromium bodies are
/// not translated yet. A translated body replaces its stub.
const LINK_PAIR_ALLOWED: &[&str] = &["layout_base/src/untranslated_symbols.rs"];

/// Top-level workspace directories that hold Rust sources.
const SOURCE_ROOTS: &[&str] = &["apps", "crates", "third_party", "tools", "vendor"];

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn workspace_root() -> PathBuf {
    normalize(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

fn layout_root() -> PathBuf {
    normalize(&Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if !(name.starts_with("target")
                || matches!(name.as_ref(), ".git" | ".claude" | "node_modules"))
            {
                rust_sources(&path, out);
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// Whether a trimmed, non-comment line carries a `#[path = ...]` attribute,
/// directly or through `cfg_attr`.
fn has_path_attribute(line: &str) -> bool {
    let compact: String = line
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    if compact.contains("#[path=") || compact.contains("#![path=") {
        return true;
    }
    compact.contains("cfg_attr(")
        && (compact.contains(",path=") || compact.contains("(path="))
}

/// The argument text of an `include!(` call starting on `lines[index]`,
/// following it across lines until its parentheses balance.
fn include_argument(lines: &[&str], index: usize, start: usize) -> String {
    let mut argument = String::new();
    let mut depth = 1usize;
    let mut text = &lines[index][start..];
    let mut next = index;
    loop {
        for character in text.chars() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return argument;
                    }
                }
                _ => {}
            }
            argument.push(character);
        }
        next += 1;
        if next >= lines.len() || next > index + 8 {
            return argument;
        }
        text = lines[next];
        argument.push('\n');
    }
}

/// Whether an `include!` argument is build-script output:
/// `concat!(env!("OUT_DIR"), ...)`.
fn is_out_dir_include(argument: &str) -> bool {
    let compact: String = argument
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    compact.starts_with("concat!(env!(\"OUT_DIR\")")
}

#[test]
fn no_module_path_attributes_or_source_includes() {
    let root = workspace_root();
    let mut sources = Vec::new();
    for dir in SOURCE_ROOTS {
        rust_sources(&root.join(dir), &mut sources);
    }
    assert!(!sources.is_empty(), "no sources found under {}", root.display());
    let this_file = normalize(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("source_layout.rs"),
    );
    let mut violations = Vec::new();
    for source in sources {
        // This guard spells the forbidden forms in its own strings.
        if normalize(&source) == this_file {
            continue;
        }
        let name = relative(&root, &source);
        let text = read(&source);
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if has_path_attribute(trimmed) {
                violations.push(format!("{name}:{}: #[path] attribute", index + 1));
            }
            let mut search = 0;
            while let Some(found) = line[search..].find("include!(") {
                let position = search + found;
                search = position + "include!(".len();
                // `include_str!(` / `include_bytes!(` are other macros.
                if line[..position].ends_with(|character: char| {
                    character.is_ascii_alphanumeric() || character == '_'
                }) {
                    continue;
                }
                let argument = include_argument(&lines, index, search);
                if argument.contains(".rs\"") && !is_out_dir_include(&argument) {
                    violations
                        .push(format!("{name}:{}: include! of a Rust source", index + 1));
                }
            }
        }
    }
    assert!(
        violations.is_empty(),
        "modules follow the file layout their path implies: move the file to \
         the location its module path implies instead of `#[path]`, and declare \
         a module instead of `include!` of a source file (only \
         `include!(concat!(env!(\"OUT_DIR\"), ...))` is allowed):\n{}",
        violations.join("\n")
    );
}

/// A source file being walked: its path and whether it is a "mod-rs" file
/// (crate root or `mod.rs`) whose child modules resolve in its own directory,
/// or a `name.rs` file whose children resolve in `name/`.
struct ModuleFile {
    file: PathBuf,
    mod_rs: bool,
}

impl ModuleFile {
    fn child_directory(&self, inline: &[String]) -> PathBuf {
        let mut directory = self
            .file
            .parent()
            .expect("module file directory")
            .to_path_buf();
        if !self.mod_rs {
            directory.push(self.file.file_stem().expect("module file stem"));
        }
        for module in inline {
            directory.push(module);
        }
        directory
    }
}

/// `(name, inline)` of a `mod name;` / `mod name {` line.
fn module_declaration(line: &str) -> Option<(String, bool)> {
    let mut rest = line;
    while rest.starts_with("#[") {
        let end = rest.find(']')?;
        rest = rest[end + 1..].trim_start();
    }
    if let Some(after) = rest.strip_prefix("pub") {
        if after.starts_with('(') {
            let end = after.find(')')?;
            rest = after[end + 1..].trim_start();
        } else if after.starts_with(' ') {
            rest = after.trim_start();
        } else {
            return None;
        }
    }
    let rest = rest.strip_prefix("mod ")?.trim_start();
    let length = rest
        .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .unwrap_or(rest.len());
    if length == 0 {
        return None;
    }
    let name = rest[..length].to_string();
    let after = rest[length..].trim();
    if after == ";" {
        return Some((name, false));
    }
    if after.starts_with('{') && !after.contains('}') {
        return Some((name, true));
    }
    None
}

fn walk_module_file(
    module: &ModuleFile,
    package: &Path,
    root: &Path,
    compiled: &mut BTreeMap<PathBuf, usize>,
    problems: &mut Vec<String>,
) {
    let text = read(&module.file);
    let name = relative(root, &module.file);
    let mut inline: Vec<(String, usize)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if let Some((_, open)) = inline.last() {
            if trimmed.starts_with('}') && indent == *open {
                inline.pop();
                continue;
            }
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let Some((module_name, is_inline)) = module_declaration(trimmed) else {
            continue;
        };
        if is_inline {
            inline.push((module_name, indent));
            continue;
        }
        let at = format!("{name}:{}", index + 1);
        let inline_names: Vec<String> = inline.iter().map(|(name, _)| name.clone()).collect();
        let base = module.child_directory(&inline_names);
        let file = base.join(format!("{module_name}.rs"));
        let directory_file = base.join(&module_name).join("mod.rs");
        let target = match (file.is_file(), directory_file.is_file()) {
            (true, false) => file,
            (false, true) => directory_file,
            (true, true) => {
                problems.push(format!(
                    "{at}: mod {module_name} is ambiguous between {} and {}",
                    relative(root, &file),
                    relative(root, &directory_file)
                ));
                continue;
            }
            (false, false) => {
                problems.push(format!(
                    "{at}: mod {module_name} has no file {} or {}",
                    relative(root, &file),
                    relative(root, &directory_file)
                ));
                continue;
            }
        };
        if !target.starts_with(package) {
            problems.push(format!(
                "{at}: mod {module_name} compiles {} from another package",
                relative(root, &target)
            ));
            continue;
        }
        let count = compiled.entry(target.clone()).or_default();
        *count += 1;
        if *count > 1 {
            continue;
        }
        let mod_rs = target.file_name().is_some_and(|file| file == "mod.rs");
        walk_module_file(
            &ModuleFile { file: target, mod_rs },
            package,
            root,
            compiled,
            problems,
        );
    }
}

fn layout_packages() -> Vec<PathBuf> {
    let mut packages: Vec<PathBuf> = std::fs::read_dir(layout_root())
        .expect("read crates/layout")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("Cargo.toml").is_file())
        .collect();
    packages.sort();
    packages
}

#[test]
fn layout_module_trees_stay_inside_their_package() {
    let root = workspace_root();
    let mut compiled: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let mut problems: Vec<String> = Vec::new();
    for package in layout_packages() {
        let mut roots = Vec::new();
        for file in ["src/lib.rs", "src/main.rs", "build.rs"] {
            if package.join(file).is_file() {
                roots.push(package.join(file));
            }
        }
        for directory in ["tests", "benches", "examples"] {
            if let Ok(entries) = std::fs::read_dir(package.join(directory)) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|extension| extension == "rs") {
                        roots.push(path);
                    }
                }
            }
        }
        for crate_root in roots {
            *compiled.entry(crate_root.clone()).or_default() += 1;
            walk_module_file(
                &ModuleFile { file: crate_root, mod_rs: true },
                &package,
                &root,
                &mut compiled,
                &mut problems,
            );
        }
        let mut sources = Vec::new();
        rust_sources(&package.join("src"), &mut sources);
        for source in sources {
            if !compiled.contains_key(&source) {
                problems.push(format!("orphan source file {}", relative(&root, &source)));
            }
        }
    }
    for (file, count) in &compiled {
        if *count > 1 {
            problems.push(format!("{} is compiled {count} times", relative(&root, file)));
        }
    }
    assert!(!compiled.is_empty(), "no Layout module files found");
    assert!(
        problems.is_empty(),
        "Layout module tree problems:\n{}",
        problems.join("\n")
    );
}

fn identifiers(line: &str) -> impl Iterator<Item = &str> {
    line.split(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '_' || character == '$')
    })
    .filter(|token| !token.is_empty())
}

/// The name after the first `fn` / `static [mut]` token of a line.
fn item_name(line: &str) -> Option<&str> {
    let tokens: Vec<&str> = identifiers(line).collect();
    let position = tokens
        .iter()
        .position(|token| *token == "fn" || *token == "static")?;
    let mut name = *tokens.get(position + 1)?;
    if name == "mut" {
        name = *tokens.get(position + 2)?;
    }
    Some(name)
}

fn is_extern_block_start(line: &str) -> bool {
    let mut rest = line;
    if let Some(after) = rest.strip_prefix("pub") {
        rest = match after.strip_prefix('(') {
            Some(scoped) => match scoped.find(')') {
                Some(end) => &scoped[end + 1..],
                None => return false,
            },
            None => after,
        }
        .trim_start();
    }
    if let Some(after) = rest.strip_prefix("unsafe ") {
        rest = after.trim_start();
    }
    let Some(after) = rest.strip_prefix("extern") else { return false };
    let after = after.trim();
    if after == "{" {
        return true;
    }
    after
        .strip_prefix('"')
        .and_then(|abi| abi.split_once('"'))
        .is_some_and(|(_, tail)| tail.trim() == "{")
}

#[test]
fn layout_packages_do_not_pair_symbols_at_link_time() {
    let root = layout_root();
    // name -> packages declaring it in an extern block.
    let mut declared: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // (name, defining package, defining file).
    let mut defined: Vec<(String, String, String)> = Vec::new();
    for package in layout_packages() {
        let package_name = package
            .file_name()
            .expect("package name")
            .to_string_lossy()
            .into_owned();
        let mut sources = Vec::new();
        rust_sources(&package.join("src"), &mut sources);
        for source in sources {
            let file = relative(&root, &source);
            let text = read(&source);
            let mut extern_block: Option<usize> = None;
            let mut after_no_mangle = false;
            let mut macro_generated = false;
            let mut file_identifiers = BTreeSet::new();
            for line in text.lines() {
                let trimmed = line.trim_start();
                let indent = line.len() - trimmed.len();
                if trimmed.starts_with("//") {
                    continue;
                }
                file_identifiers.extend(identifiers(trimmed).map(str::to_string));
                if let Some(open) = extern_block {
                    if trimmed.starts_with('}') && indent == open {
                        extern_block = None;
                    } else if let Some(name) = item_name(trimmed) {
                        declared
                            .entry(name.to_string())
                            .or_default()
                            .insert(package_name.clone());
                    }
                    continue;
                }
                if is_extern_block_start(trimmed) {
                    extern_block = Some(indent);
                    continue;
                }
                if trimmed.starts_with("#[") && trimmed.contains("no_mangle") {
                    after_no_mangle = true;
                    continue;
                }
                if after_no_mangle {
                    if trimmed.starts_with("#[") {
                        continue;
                    }
                    after_no_mangle = false;
                    match item_name(trimmed) {
                        Some(name) if name.starts_with('$') => macro_generated = true,
                        Some(name) => {
                            defined.push((name.to_string(), package_name.clone(), file.clone()))
                        }
                        None => {}
                    }
                }
            }
            // A macro stamps out `#[no_mangle]` items from its invocation
            // arguments; every identifier of such a file is a candidate name.
            if macro_generated {
                for name in file_identifiers {
                    defined.push((name, package_name.clone(), file.clone()));
                }
            }
        }
    }
    let mut violations = BTreeSet::new();
    for (name, package, file) in defined {
        if LINK_PAIR_ALLOWED.contains(&file.as_str()) {
            continue;
        }
        let Some(declarers) = declared.get(&name) else { continue };
        for declarer in declarers {
            if *declarer != package {
                violations.insert(format!(
                    "{file} defines #[no_mangle] {name}, declared in an extern block of {declarer}"
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "replace link-time pairs with a typed dependency or registry slot:\n{}",
        violations.into_iter().collect::<Vec<_>>().join("\n")
    );
}

//! Linux host input for the existing, platform-independent FontFace boundary.
//!
//! Chromium's ui/gfx/font_fallback_linux.cc obtains scalable faces in Fontconfig
//! order and passes file identity, TTC index and style to Blink. `fc-match --sort`
//! exposes that same configured FcFontSort result without adding a native font
//! renderer or a second font-selection engine. Character and presentation
//! selection remain in FontEngine. Discovery runs once through DemoFonts.

use std::collections::{HashMap, HashSet};
use std::process::Command;

use super::{FontFace, FontFallbackRole, SharedFontBytes};

// Delimit records separately from lines: spaces, commas and newlines in font
// paths must not be interpreted by a shell or split as whitespace.
const FORMAT: &str = "%{family[0]}\x1f%{file}\x1f%{index}\x1f%{weight}\x1f%{slant}\x1f%{lang}\x1f%{color}\x1f%{fontformat}\x1f%{scalable}\x1e";

#[derive(Clone, Debug)]
struct HostFace {
    family: String,
    path: String,
    index: u32,
    weight: f64,
    italic: bool,
    locales: Vec<String>,
    color: bool,
}

fn ParseFaces(output: &str) -> Vec<HostFace> {
    output
        .split('\x1e')
        .filter_map(|record| {
            let fields: Vec<_> = record.split('\x1f').collect();
            if fields.len() != 9
                || fields[0].is_empty()
                || fields[1].is_empty()
                || !matches!(fields[7], "TrueType" | "CFF")
                || fields[8] != "True"
            {
                return None;
            }
            let weight: f64 = fields[3].parse().ok()?;
            if !weight.is_finite() || !(0.0..=215.0).contains(&weight) {
                return None;
            }
            Some(HostFace {
                family: fields[0].into(),
                path: fields[1].into(),
                index: fields[2].parse().ok()?,
                weight: OpenTypeWeight(weight),
                italic: fields[4].parse::<u32>().ok()? != 0,
                locales: fields[5]
                    .split('|')
                    .filter(|lang| !lang.is_empty())
                    .map(str::to_owned)
                    .collect(),
                color: fields[6] == "True",
            })
        })
        .collect()
}

// Fontconfig fcweight.c's FcWeightToOpenTypeDouble mapping. Fontconfig weights
// are not CSS/OpenType weights (e.g. regular=80, semibold=180, bold=200).
fn OpenTypeWeight(weight: f64) -> f64 {
    const WEIGHTS: &[(f64, f64)] = &[
        (0.0, 100.0),
        (40.0, 200.0),
        (50.0, 300.0),
        (55.0, 350.0),
        (75.0, 380.0),
        (80.0, 400.0),
        (100.0, 500.0),
        (180.0, 600.0),
        (200.0, 700.0),
        (205.0, 800.0),
        (210.0, 900.0),
        (215.0, 1000.0),
    ];
    for pair in WEIGHTS.windows(2) {
        let [(from, from_css), (to, to_css)] = pair else {
            unreachable!()
        };
        if weight <= *to {
            return from_css + (weight.max(*from) - from) / (to - from) * (to_css - from_css);
        }
    }
    1000.0
}

fn QueryFonts(pattern: &str, mode: &str) -> Vec<HostFace> {
    let mut command = Command::new("fc-match");
    if !mode.is_empty() {
        command.arg(format!("--{mode}"));
    }
    let Ok(output) = command.args(["--format", FORMAT, pattern]).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let Ok(output) = std::str::from_utf8(&output.stdout) else {
        return Vec::new();
    };
    ParseFaces(output)
}

pub(super) fn LoadFonts(
    ReadFont: &impl Fn(&str) -> std::io::Result<SharedFontBytes>,
) -> Vec<FontFace> {
    let mut hosts = QueryFonts(":scalable=true", "sort");
    if hosts.is_empty() {
        return Vec::new();
    }
    // Generic families and the standard cross-platform family aliases are
    // resolved by the host's configuration, never to a fixed file or website.
    let mut aliases = Vec::new();
    for family in [
        "sans-serif",
        "serif",
        "monospace",
        "system-ui",
        "cursive",
        "fantasy",
        "math",
        "emoji",
        "Arial",
        "Helvetica",
        "Times New Roman",
        "Courier New",
        "Verdana",
    ] {
        if let Some(face) = QueryFonts(&format!("{family}:scalable=true"), "").first() {
            aliases.push((family.to_owned(), face.family.clone()));
        }
    }
    if let Some((_, native)) = aliases.iter().find(|(name, _)| name == "sans-serif") {
        aliases.push(("demo".into(), native.clone()));
    }
    // FcFontSort's coverage trim avoids eagerly loading every unrelated style
    // on the host. Restore all styles of each explicitly exposed family so CSS
    // weight/slant matching still has the real configured faces to choose from.
    let requested_families: HashSet<_> =
        aliases.iter().map(|(_, native)| native.as_str()).collect();
    hosts.extend(
        QueryFonts(":scalable=true", "all")
            .into_iter()
            .filter(|face| requested_families.contains(face.family.as_str())),
    );
    AssembleFonts(hosts, aliases, ReadFont)
}

fn AssembleFonts(
    hosts: Vec<HostFace>,
    aliases: Vec<(String, String)>,
    ReadFont: &impl Fn(&str) -> std::io::Result<SharedFontBytes>,
) -> Vec<FontFace> {
    let math_family = aliases
        .iter()
        .find(|(family, _)| family == "math")
        .map(|(_, native)| native);
    let mut files: HashMap<String, Option<SharedFontBytes>> = HashMap::new();
    let mut seen = HashSet::new();
    let mut faces = Vec::new();
    for host in hosts {
        // A TTC's regional/style faces share the same immutable allocation.
        // Remember failed reads too rather than repeatedly opening broken files.
        let bytes = files.entry(host.path.clone()).or_insert_with(|| {
            let bytes = ReadFont(&host.path).ok()?;
            if bytes.starts_with(b"wOFF") || bytes.starts_with(b"wOF2") {
                // Fontconfig/FreeType can discover installed WOFF fonts; the
                // translated OpenType boundary consumes their decoded SFNT.
                web_font::DecodeWebFont(bytes.to_vec())
                    .ok()
                    .map(SharedFontBytes::from)
            } else {
                Some(bytes)
            }
        });
        let Some(bytes) = bytes else { continue };
        if !ValidFaceIndex(bytes, host.index)
            || !seen.insert((host.family.clone(), host.path, host.index))
        {
            continue;
        }
        let role = if host.color {
            FontFallbackRole::kEmoji
        } else if math_family == Some(&host.family) {
            FontFallbackRole::kMath
        } else {
            FontFallbackRole::kText
        };
        faces.push(FontFace {
            family: host.family.clone(),
            native_family: host.family.clone(),
            metrics_family: host.family,
            weight: host.weight,
            italic: host.italic,
            face_index: host.index,
            bytes: bytes.clone(),
            fallback_role: role,
            fallback_locales: host.locales,
            ..FontFace::default()
        });
    }
    let native_count = faces.len();
    for (family, native) in aliases {
        if family == native {
            continue;
        }
        for index in 0..native_count {
            if faces[index].native_family == native {
                let mut alias = faces[index].clone();
                alias.family = family.clone();
                // Native entries already supply fallback. Aliases only take
                // part when that CSS family was actually requested.
                alias.fallback_role = FontFallbackRole::kFamilyOnly;
                faces.push(alias);
            }
        }
    }
    faces
}

fn ValidFaceIndex(bytes: &[u8], index: u32) -> bool {
    let index = index & 0xffff; // Upper bits encode a Fontconfig named instance.
    if bytes.starts_with(b"ttcf") {
        let Some(count) = bytes.get(8..12) else {
            return false;
        };
        index < u32::from_be_bytes(count.try_into().unwrap())
    } else {
        index == 0 && matches!(bytes.get(..4), Some(b"\0\x01\0\0" | b"OTTO" | b"true"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn Record(
        family: &str,
        path: &str,
        index: u32,
        weight: u32,
        slant: u32,
        color: bool,
    ) -> String {
        format!("{family}\x1f{path}\x1f{index}\x1f{weight}\x1f{slant}\x1fzh-cn|ja\x1f{}\x1fTrueType\x1fTrue\x1e", if color { "True" } else { "False" })
    }

    #[test]
    fn fontconfig_metadata_preserves_paths_indices_styles_and_locales() {
        let output = Record(
            "Regional Sans",
            "/fonts/a font\nname.ttc",
            2,
            200,
            100,
            false,
        );
        let faces = ParseFaces(&output);
        assert_eq!(faces.len(), 1);
        let face = &faces[0];
        assert_eq!(face.path, "/fonts/a font\nname.ttc");
        assert_eq!(face.index, 2);
        assert_eq!(face.weight, 700.0);
        assert!(face.italic);
        assert_eq!(face.locales, ["zh-cn", "ja"]);
        assert!(ParseFaces(&output.replace("TrueType", "Type 1")).is_empty());
        assert!(ParseFaces("malformed").is_empty());
    }

    #[test]
    fn fontconfig_weight_scale_maps_to_css_weights() {
        for (fc, css) in [
            (0.0, 100.0),
            (50.0, 300.0),
            (80.0, 400.0),
            (100.0, 500.0),
            (180.0, 600.0),
            (200.0, 700.0),
            (205.0, 800.0),
            (210.0, 900.0),
            (215.0, 1000.0),
        ] {
            assert_eq!(OpenTypeWeight(fc), css);
        }
        assert_eq!(OpenTypeWeight(190.0), 650.0);
    }

    #[test]
    fn catalog_preserves_ttc_faces_roles_and_shares_alias_bytes() {
        let hosts = ParseFaces(
            &[
                Record("Regional Sans", "/fonts/collection.ttc", 0, 80, 0, false),
                Record("Regional Sans", "/fonts/collection.ttc", 1, 200, 0, false),
                Record("Color Emoji", "/fonts/emoji.ttf", 0, 80, 0, true),
                Record("Math Face", "/fonts/math.ttf", 0, 80, 0, false),
                Record("Missing", "/fonts/missing.ttf", 0, 80, 0, false),
            ]
            .concat(),
        );
        let reads = std::cell::RefCell::new(Vec::new());
        let faces = AssembleFonts(
            hosts,
            vec![
                ("sans-serif".into(), "Regional Sans".into()),
                ("math".into(), "Math Face".into()),
            ],
            &|path| {
                reads.borrow_mut().push(path.to_owned());
                match path {
                    "/fonts/collection.ttc" => Ok(b"ttcf\0\x01\0\0\0\0\0\x02".as_slice().into()),
                    "/fonts/emoji.ttf" | "/fonts/math.ttf" => Ok(b"\0\x01\0\0".as_slice().into()),
                    _ => Err(std::io::ErrorKind::NotFound.into()),
                }
            },
        );
        assert_eq!(reads.borrow().len(), 4);
        assert_eq!(faces.len(), 7);
        assert_eq!(faces[1].face_index, 1);
        assert_eq!(faces[1].weight, 700.0);
        assert_eq!(faces[2].fallback_role, FontFallbackRole::kEmoji);
        assert_eq!(faces[3].fallback_role, FontFallbackRole::kMath);
        assert_eq!(faces[4].family, "sans-serif");
        assert_eq!(faces[4].fallback_role, FontFallbackRole::kFamilyOnly);
        assert!(SharedFontBytes::ptr_eq(&faces[0].bytes, &faces[1].bytes));
        assert!(SharedFontBytes::ptr_eq(&faces[0].bytes, &faces[4].bytes));
    }

    #[test]
    fn installed_fontconfig_catalog_supplies_real_cjk_and_emoji_glyphs() {
        let available = QueryFonts(":scalable=true", "sort");
        if available.is_empty() {
            return; // Minimal distributions can use the embedded fallback.
        }
        let faces = super::super::DemoFonts();
        for (character, role) in [
            ('水', FontFallbackRole::kText),
            ('😀', FontFallbackRole::kEmoji),
        ] {
            // This integration assertion applies when Fontconfig reports an
            // installed face with that character, not to a fontless container.
            let candidates = QueryFonts(
                &format!(":charset={:x}:scalable=true", character as u32),
                "",
            );
            let Some(candidate) = candidates.first() else {
                continue;
            };
            let bytes =
                std::fs::read(&candidate.path).expect("Fontconfig candidate must be readable");
            let bytes = if bytes.starts_with(b"wOFF") || bytes.starts_with(b"wOF2") {
                web_font::DecodeWebFont(bytes).unwrap()
            } else {
                bytes
            };
            let installed = font_engine::OpenTypeFont::new_shared(
                SharedFontBytes::from(bytes),
                candidate.index,
                16.0,
                &[],
            );
            if installed.GlyphForCharacter(character as u32).is_none() {
                continue; // Fontconfig's last resort does not prove installation.
            }
            let face = faces
                .iter()
                .find(|face| {
                    face.native_family == candidate.family && face.face_index == candidate.index
                })
                .expect("an installed character-bearing Fontconfig face must enter the catalog");
            let font = font_engine::OpenTypeFont::new_shared(
                face.bytes.clone(),
                face.face_index,
                16.0,
                &[],
            );
            assert!(font.GlyphForCharacter(character as u32).is_some());
            assert_eq!(face.fallback_role, role);
            assert!(faces.iter().any(|face| face.family == "sans-serif"));
            println!(
                "host glyph U+{:04X}: {} (index {}, {:?})",
                character as u32, face.native_family, face.face_index, face.fallback_role
            );
        }
    }
}

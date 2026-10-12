//! On-demand host font discovery and byte acquisition.
//!
//! This is an embedding capability, deliberately outside FontEngine. Creating
//! it neither enumerates installed fonts nor opens font files. Resolve answers
//! the engine's typed requests; the caller applies those answers as mutations
//! before retrying its pure computation.
#![allow(non_snake_case)]

use font_engine::resolver::FontFace;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use font_engine::{
    FontContextVersion, FontEngine, FontError, FontFallbackRole, FontMutation, FontPending,
    FontVariation, SharedFontBytes, SystemFontAnswer, SystemFontFace, SystemFontFaceId,
    SystemFontProvider, SystemFontQuery, SystemFontRequest, SystemFontResponse,
};

#[cfg(target_os = "linux")]
#[path = "system_font_provider_linux.rs"]
mod linux;
#[cfg(target_os = "macos")]
#[path = "system_font_provider_macos.rs"]
mod macos;

#[derive(Clone, Debug)]
struct HostFace {
    family: String,
    path: String,
    index: u32,
    weight: f64,
    italic: bool,
    locales: Vec<String>,
    color: bool,
    variations: Vec<FontVariation>,
    /// The exact native PostScript name also disambiguates CoreText TTC faces.
    unique_name: String,
}

/// The browser owns platform discovery. No instance is retained by FontEngine
/// or by a layout/shape callback. A version change invalidates positive and
/// negative byte/query results together with provider-private face identities.
pub struct BrowserSystemFontProvider {
    version: FontContextVersion,
    instance: u64,
    next_face: u64,
    faces: HashMap<SystemFontFaceId, HostFace>,
    files: HashMap<String, Option<SharedFontBytes>>,
    #[cfg(target_os = "linux")]
    platform: linux::LinuxFontHost,
}

impl Default for BrowserSystemFontProvider {
    fn default() -> Self {
        Self::new(FontContextVersion::default())
    }
}

impl BrowserSystemFontProvider {
    pub fn new(version: FontContextVersion) -> Self {
        static NEXT_INSTANCE: AtomicU64 = AtomicU64::new(1);
        Self {
            version,
            instance: NEXT_INSTANCE.fetch_add(1, Ordering::Relaxed),
            next_face: 0,
            faces: HashMap::new(),
            files: HashMap::new(),
            #[cfg(target_os = "linux")]
            platform: linux::LinuxFontHost::default(),
        }
    }

    pub fn ContextVersion(&self) -> FontContextVersion {
        self.version
    }

    pub fn SetContextVersion(&mut self, version: FontContextVersion) {
        if version != self.version {
            self.version = version;
            self.faces.clear();
            self.files.clear();
            #[cfg(target_os = "linux")]
            {
                self.platform = linux::LinuxFontHost::default();
            }
        }
    }

    /// Host assembly convenience: answers keep their original request/version
    /// so the engine can reject stale or mismatched responses.
    pub fn ResolveRequests(&mut self, requests: &[SystemFontRequest]) -> Vec<SystemFontResponse> {
        requests
            .iter()
            .map(|request| SystemFontResponse {
                request: request.clone(),
                answer: self.Resolve(request),
            })
            .collect()
    }

    /// Apply this batch's platform answers and return unresolved resource IDs.
    /// The embedding owns web-font downloads and must retry its original
    /// computation after these mutations to discover any next-stage requests.
    pub fn ResolvePending(
        &mut self,
        engine: &mut FontEngine,
        pending: FontPending,
    ) -> Result<Vec<u64>, FontError> {
        for response in self.ResolveRequests(&pending.system_queries) {
            engine.ApplyMutation(FontMutation::SystemFontResponse(response))?;
        }
        Ok(pending.resource_ids)
    }

    fn Register(&mut self, host: HostFace, family: Option<&str>) -> SystemFontFace {
        let id = self
            .faces
            .iter()
            .find_map(|(id, registered)| {
                (registered.path == host.path
                    && registered.index == host.index
                    && registered.unique_name == host.unique_name
                    && registered.variations == host.variations)
                    .then(|| id.clone())
            })
            .unwrap_or_else(|| {
                self.next_face += 1;
                SystemFontFaceId(format!(
                    "host-font:{}:{}:{}",
                    self.instance, self.version.0, self.next_face
                ))
            });
        let mut face = Descriptor(&host);
        if let Some(family) = family {
            face.family = family.to_owned();
            face.fallback_role = FontFallbackRole::kFamilyOnly;
        }
        self.faces.entry(id.clone()).or_insert(host);
        SystemFontFace { id, face }
    }

    fn Open(&mut self, id: &SystemFontFaceId) -> SystemFontAnswer {
        let Some(host) = self.faces.get(id).cloned() else {
            return SystemFontAnswer::Unavailable;
        };
        if !self.files.contains_key(&host.path) {
            let bytes = match crate::font_catalog::ReadFont(&host.path) {
                Ok(bytes) => bytes,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.files.insert(host.path.clone(), None);
                    return SystemFontAnswer::Unavailable;
                }
                Err(error) => {
                    return SystemFontAnswer::Failed(format!(
                        "opening selected system font: {error}"
                    ))
                }
            };
            let bytes = if bytes.starts_with(b"wOFF") || bytes.starts_with(b"wOF2") {
                match web_font::DecodeWebFont(bytes.to_vec()) {
                    Ok(bytes) => SharedFontBytes::from(bytes),
                    Err(error) => {
                        return SystemFontAnswer::Failed(format!(
                            "decoding selected system font: {error}"
                        ))
                    }
                }
            } else {
                bytes
            };
            self.files.insert(host.path.clone(), Some(bytes));
        }
        let Some(bytes) = self.files.get(&host.path).and_then(Option::as_ref) else {
            return SystemFontAnswer::Unavailable;
        };
        let mut face = Descriptor(&host);
        // Fontconfig packs a named-instance ordinal into the upper 16 bits.
        // The shaping engine consumes the collection index plus explicit axes.
        #[cfg(target_os = "macos")]
        {
            let Some(index) = macos::CollectionIndex(&host, bytes) else {
                return SystemFontAnswer::Unavailable;
            };
            face.face_index = index;
        }
        #[cfg(not(target_os = "macos"))]
        {
            face.face_index &= 0xffff;
        }
        if !ValidFaceIndex(bytes, face.face_index) {
            return SystemFontAnswer::Failed(
                "selected system font has an invalid collection index or OpenType directory".into(),
            );
        }
        if host.index >> 16 != 0 {
            let Some(variations) = NamedInstance(bytes, face.face_index, host.index >> 16) else {
                return SystemFontAnswer::Unavailable;
            };
            for variation in variations {
                if !face
                    .variations
                    .iter()
                    .any(|existing| existing.tag == variation.tag)
                {
                    face.variations.push(variation);
                }
            }
        }
        face.bytes = bytes.clone();
        SystemFontAnswer::Face(face)
    }
}

impl SystemFontProvider for BrowserSystemFontProvider {
    fn SetContextVersion(&mut self, version: FontContextVersion) {
        BrowserSystemFontProvider::SetContextVersion(self, version);
    }
    fn Resolve(&mut self, request: &SystemFontRequest) -> SystemFontAnswer {
        if request.context_version != self.version {
            return SystemFontAnswer::Unavailable;
        }
        if let SystemFontQuery::OpenFace { id } = &request.query {
            return self.Open(id);
        }
        let base = match &request.query {
            SystemFontQuery::Fallback {
                base_face: Some(id),
                ..
            } => {
                // A foreign/stale base identity must not be silently replaced.
                let Some(base) = self.faces.get(id) else {
                    return SystemFontAnswer::Unavailable;
                };
                Some(base)
            }
            _ => None,
        };
        #[cfg(target_os = "linux")]
        let hosts = self.platform.Query(&request.query, base);
        #[cfg(target_os = "macos")]
        let hosts = macos::Query(&request.query, base);
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let hosts: Vec<HostFace> = {
            let _ = base;
            Vec::new()
        };
        let family = match &request.query {
            SystemFontQuery::MatchFamily { family, .. } => Some(family.as_str()),
            SystemFontQuery::MatchGeneric { generic, .. } => Some(generic.as_str()),
            _ => None,
        };
        let family_only = !matches!(request.query, SystemFontQuery::Fallback { .. });
        SystemFontAnswer::Faces(
            hosts
                .into_iter()
                .map(|host| {
                    let mut descriptor = self.Register(host, family);
                    if family_only {
                        descriptor.face.fallback_role = FontFallbackRole::kFamilyOnly;
                    }
                    descriptor
                })
                .collect(),
        )
    }
}

fn Descriptor(host: &HostFace) -> FontFace {
    FontFace {
        family: host.family.clone(),
        native_family: host.family.clone(),
        metrics_family: host.family.clone(),
        weight: host.weight,
        italic: host.italic,
        face_index: host.index,
        variations: host.variations.clone(),
        fallback_locales: host.locales.clone(),
        fallback_role: if host.color {
            FontFallbackRole::kEmoji
        } else {
            FontFallbackRole::kText
        },
        ..FontFace::default()
    }
}

fn U16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(at..at.checked_add(2)?)?.try_into().ok()?,
    ))
}
fn U32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(at..at.checked_add(4)?)?.try_into().ok()?,
    ))
}
fn FaceOffset(bytes: &[u8], index: u32) -> Option<usize> {
    if bytes.starts_with(b"ttcf") {
        if index >= U32(bytes, 8)? {
            return None;
        }
        usize::try_from(U32(
            bytes,
            12usize.checked_add(usize::try_from(index).ok()?.checked_mul(4)?)?,
        )?)
        .ok()
    } else if index == 0 && matches!(bytes.get(..4), Some(b"\0\x01\0\0" | b"OTTO" | b"true")) {
        Some(0)
    } else {
        None
    }
}
fn ValidFaceIndex(bytes: &[u8], index: u32) -> bool {
    FaceOffset(bytes, index)
        .is_some_and(|offset| bytes.get(offset..offset.saturating_add(4)).is_some())
}

/// Read only the selected face's fvar instance, after OpenFace. This does not
/// parse a catalog or open a native/shaping font handle.
fn NamedInstance(bytes: &[u8], index: u32, instance: u32) -> Option<Vec<FontVariation>> {
    let offset = FaceOffset(bytes, index)?;
    let tables = usize::from(U16(bytes, offset.checked_add(4)?)?);
    let fvar = (0..tables).find_map(|table| {
        let record = offset
            .checked_add(12)?
            .checked_add(table.checked_mul(16)?)?;
        (bytes.get(record..record.checked_add(4)?)? == b"fvar")
            .then(|| U32(bytes, record + 8).and_then(|value| usize::try_from(value).ok()))
            .flatten()
    })?;
    let axes_offset = usize::from(U16(bytes, fvar.checked_add(4)?)?);
    let axes = usize::from(U16(bytes, fvar.checked_add(8)?)?);
    let axis_size = usize::from(U16(bytes, fvar.checked_add(10)?)?);
    let instances = u32::from(U16(bytes, fvar.checked_add(12)?)?);
    let instance_size = usize::from(U16(bytes, fvar.checked_add(14)?)?);
    if instance == 0
        || instance > instances
        || axis_size < 20
        || instance_size < 4 + axes.checked_mul(4)?
    {
        return None;
    }
    let axes_start = fvar.checked_add(axes_offset)?;
    let instance_start = axes_start
        .checked_add(axes.checked_mul(axis_size)?)?
        .checked_add(
            usize::try_from(instance - 1)
                .ok()?
                .checked_mul(instance_size)?,
        )?;
    (0..axes)
        .map(|axis| {
            Some(FontVariation {
                tag: U32(bytes, axes_start.checked_add(axis.checked_mul(axis_size)?)?)?,
                value: U32(
                    bytes,
                    instance_start
                        .checked_add(4)?
                        .checked_add(axis.checked_mul(4)?)?,
                )? as i32 as f32
                    / 65536.0,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn Host(path: String, index: u32) -> HostFace {
        HostFace {
            family: "Selected".into(),
            path,
            index,
            weight: 400.,
            italic: false,
            locales: vec!["en".into()],
            color: false,
            variations: vec![],
            unique_name: "Roboto-Regular".into(),
        }
    }
    fn Request(version: u64, query: SystemFontQuery) -> SystemFontRequest {
        SystemFontRequest {
            id: 1,
            context_version: FontContextVersion(version),
            query,
        }
    }

    #[test]
    fn provider_registration_is_metadata_only_and_open_is_explicit() {
        let mut provider = BrowserSystemFontProvider::new(FontContextVersion(7));
        assert!(provider.faces.is_empty());
        assert!(provider.files.is_empty());
        let path = std::env::temp_dir().join(format!(
            "rechrom-font-host-{}-{}.ttf",
            std::process::id(),
            provider.instance
        ));
        std::fs::write(
            &path,
            include_bytes!("../../../assets/fonts/Roboto-Regular.ttf"),
        )
        .unwrap();
        let descriptor =
            provider.Register(Host(path.to_str().unwrap().into(), 0), Some("CSS Alias"));
        assert!(descriptor.face.bytes.is_empty());
        assert_eq!(descriptor.face.family, "CSS Alias");
        assert_eq!(descriptor.face.native_family, "Selected");
        assert_eq!(descriptor.face.fallback_role, FontFallbackRole::kFamilyOnly);
        assert!(provider.files.is_empty());
        let alias = provider.Register(
            Host(path.to_str().unwrap().into(), 0),
            Some("Another Alias"),
        );
        assert_eq!(alias.id, descriptor.id);
        let open = Request(
            7,
            SystemFontQuery::OpenFace {
                id: descriptor.id.clone(),
            },
        );
        let SystemFontAnswer::Face(first) = provider.Resolve(&open) else {
            panic!("registered face must open");
        };
        std::fs::remove_file(path).unwrap();
        let SystemFontAnswer::Face(second) = provider.Resolve(&open) else {
            panic!("open face bytes must be retained");
        };
        assert!(SharedFontBytes::ptr_eq(&first.bytes, &second.bytes));
        assert_eq!(provider.files.len(), 1);
        provider.SetContextVersion(FontContextVersion(8));
        assert_eq!(provider.Resolve(&open), SystemFontAnswer::Unavailable);
        assert_eq!(
            provider.Resolve(&Request(8, open.query)),
            SystemFontAnswer::Unavailable
        );
        assert!(provider.files.is_empty());
    }

    #[test]
    fn stale_or_foreign_base_faces_are_not_reinterpreted_as_default() {
        let mut provider = BrowserSystemFontProvider::default();
        let query = SystemFontQuery::Fallback {
            text: vec![0x6c34],
            weight: 400.,
            italic: false,
            locale: "zh-Hant".into(),
            presentation: font_engine::FontPresentation::Text,
            base_face: Some(SystemFontFaceId("foreign-provider".into())),
        };
        assert_eq!(
            provider.Resolve(&Request(0, query)),
            SystemFontAnswer::Unavailable
        );
        assert!(provider.files.is_empty());
    }

    #[test]
    fn named_instances_preserve_collection_identity_and_axis_coordinates() {
        let mut bytes = vec![0u8; 86];
        bytes[..4].copy_from_slice(b"\0\x01\0\0");
        bytes[4..6].copy_from_slice(&1u16.to_be_bytes());
        bytes[12..16].copy_from_slice(b"fvar");
        bytes[20..24].copy_from_slice(&28u32.to_be_bytes());
        bytes[32..34].copy_from_slice(&16u16.to_be_bytes());
        bytes[36..38].copy_from_slice(&1u16.to_be_bytes());
        bytes[38..40].copy_from_slice(&20u16.to_be_bytes());
        bytes[40..42].copy_from_slice(&2u16.to_be_bytes());
        bytes[42..44].copy_from_slice(&8u16.to_be_bytes());
        bytes[44..48].copy_from_slice(b"wght");
        bytes[68..72].copy_from_slice(&(500u32 * 65536).to_be_bytes());
        bytes[76..80].copy_from_slice(&(700u32 * 65536).to_be_bytes());
        assert_eq!(
            NamedInstance(&bytes, 0, 2),
            Some(vec![FontVariation {
                tag: u32::from_be_bytes(*b"wght"),
                value: 700.
            }])
        );
        assert!(NamedInstance(&bytes, 0, 3).is_none());
        assert!(NamedInstance(&bytes[..60], 0, 1).is_none());
        assert!(NamedInstance(&bytes, 1, 1).is_none());
    }
}

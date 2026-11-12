//! blink/common/unique_name/unique_name_helper.cc. The embedding supplies live
//! FrameTree facts; names belong to the frame, independently of its Document.
use sha2::{Digest, Sha256};

const PREFIX: &str = "<!--framePath /";

pub trait FrameAdapter {
    fn IsCandidateUnique(&self, name: &str) -> bool;
    fn SiblingCount(&self) -> usize;
    /// Parent first, including the main frame's empty unique name. Stop after
    /// the first generated frame path, as Chromium does.
    fn AncestorNames(&self) -> Vec<String>;
    fn FramePosition(&self) -> Vec<usize>;
}

#[derive(Default)]
pub struct UniqueNameHelper {
    value: String,
    frozen: bool,
}
impl UniqueNameHelper {
    pub fn Value(&self) -> &str {
        &self.value
    }
    pub fn IsFrozen(&self) -> bool {
        self.frozen
    }
    pub fn Freeze(&mut self) {
        self.frozen = true;
    }
    pub fn Generate(adapter: &dyn FrameAdapter, requested: &str) -> Self {
        Self {
            value: Calculate(adapter, requested),
            frozen: false,
        }
    }
    pub fn GenerateForNewChild(
        adapter: &dyn FrameAdapter,
        requested: &str,
        is_created_by_script: bool,
    ) -> Self {
        if !is_created_by_script {
            return Self::Generate(adapter, requested);
        }
        let mut token = [0u8; 16];
        getrandom::fill(&mut token).expect("dynamic frame unique-name entropy");
        let mut value = String::from("<!--dynamicFrame");
        use std::fmt::Write;
        for byte in token {
            write!(value, "{byte:02X}").expect("unique name string");
        }
        value.push_str("-->");
        Self {
            value,
            frozen: true,
        }
    }
    pub fn UpdateName(&mut self, adapter: &dyn FrameAdapter, requested: &str) {
        if self.frozen {
            return;
        }
        self.value.clear();
        self.value = Calculate(adapter, requested);
    }
}
pub fn IsFramePath(name: &str) -> bool {
    name.starts_with(PREFIX) && name.ends_with("-->") && name.len() > PREFIX.len() + 3
}
fn Calculate(adapter: &dyn FrameAdapter, requested: &str) -> String {
    let hashed;
    let name = if requested.len() > 80 {
        hashed = format!("<!--frameHash{:X}-->", Sha256::digest(requested.as_bytes()));
        hashed.as_str()
    } else {
        requested
    };
    if !name.is_empty() && name != "_blank" && adapter.IsCandidateUnique(name) {
        return name.to_owned();
    }
    let mut ancestors = adapter.AncestorNames();
    ancestors.reverse();
    if let Some(first) = ancestors.first_mut().filter(|name| IsFramePath(name)) {
        *first = first[PREFIX.len()..first.len() - 3].to_owned();
    }
    let candidate = format!(
        "{PREFIX}{}/<!--frame{}-->-->",
        ancestors.join("/"),
        adapter.SiblingCount()
    );
    if adapter.IsCandidateUnique(&candidate) {
        return candidate;
    }
    let mut suffix = String::from("<!--framePosition");
    for position in adapter.FramePosition() {
        suffix.push_str(&format!("-{position}"));
    }
    for retry in 0u64.. {
        let unique = format!("{candidate}{suffix}/{retry}-->");
        if adapter.IsCandidateUnique(&unique) {
            return unique;
        }
    }
    unreachable!("unique name retries exhausted")
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Frame {
        occupied: Vec<String>,
        ancestors: Vec<String>,
        siblings: usize,
    }
    impl FrameAdapter for Frame {
        fn IsCandidateUnique(&self, name: &str) -> bool {
            !self.occupied.iter().any(|value| value == name)
        }
        fn SiblingCount(&self) -> usize {
            self.siblings
        }
        fn AncestorNames(&self) -> Vec<String> {
            self.ancestors.clone()
        }
        fn FramePosition(&self) -> Vec<usize> {
            vec![self.siblings, 0]
        }
    }
    #[test]
    fn follows_chromium_stable_paths_hashes_collisions_and_dynamic_freeze() {
        let mut frame = Frame {
            occupied: vec![],
            ancestors: vec![String::new()],
            siblings: 0,
        };
        assert_eq!(UniqueNameHelper::Generate(&frame, "named").Value(), "named");
        let first = UniqueNameHelper::Generate(&frame, "");
        assert_eq!(first.Value(), "<!--framePath //<!--frame0-->-->");
        frame.occupied.push(first.Value().to_owned());
        assert_eq!(
            UniqueNameHelper::Generate(&frame, "_blank").Value(),
            "<!--framePath //<!--frame0-->--><!--framePosition-0-0/0-->"
        );
        let hashed = UniqueNameHelper::Generate(&frame, &"a".repeat(81));
        assert!(hashed.Value().starts_with("<!--frameHash"));
        assert_eq!(hashed.Value().len(), 80);
        let mut dynamic = UniqueNameHelper::GenerateForNewChild(&frame, "named", true);
        let original = dynamic.Value().to_owned();
        assert!(original.starts_with("<!--dynamicFrame"));
        dynamic.UpdateName(&frame, "changed");
        assert_eq!(dynamic.Value(), original);
        let mut stable = UniqueNameHelper::Generate(&frame, "before");
        stable.UpdateName(&frame, "after");
        assert_eq!(stable.Value(), "after");
        stable.Freeze();
        stable.UpdateName(&frame, "later");
        assert_eq!(stable.Value(), "after");
    }
}

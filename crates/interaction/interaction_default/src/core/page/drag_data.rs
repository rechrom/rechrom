//! DragData's format policy, drag_data.cc:49-179, with the original DataObject
//! and retained File owners. URL/path conversion is a boundary operation.
use interaction_dispatch::core::clipboard::data_transfer::{DragDataObject, DragOperation};
use interaction_dispatch::core::fileapi::file::File;
use std::{rc::Rc, sync::Arc};
#[derive(Clone)]
pub struct DragData {
    pub data: Rc<DragDataObject>,
    pub source_operations: u32,
}
impl DragData {
    pub fn Contains(&self, kind: &str) -> bool {
        self.data.Types().iter().any(|value| value == kind)
    }
    pub fn Files(&self) -> Vec<Arc<File>> {
        (0..self.data.length())
            .filter_map(|i| self.data.Item(i))
            .filter(|item| item.IsFilename())
            .filter_map(|item| item.FileReference())
            .collect()
    }
    pub fn ContainsFiles(&self) -> bool {
        (0..self.data.length()).any(|i| self.data.Item(i).is_some_and(|item| item.IsFilename()))
    }
    pub fn NumberOfFiles(&self) -> usize {
        self.Files().len()
    }
    pub fn ContainsURL(&self) -> bool {
        self.Contains("text/uri-list") || self.ContainsFiles()
    }
    pub fn ContainsCompatibleContent(&self) -> bool {
        self.Contains("text/plain") || self.ContainsURL() || self.Contains("text/html")
    }
    pub fn CanSmartReplace(&self) -> bool {
        self.Contains("text/plain") && !self.Contains("text/uri-list")
    }
    pub fn PlainText(&self) -> Vec<u16> {
        self.data
            .GetData("text/plain")
            .Span16()
            .unwrap_or_default()
            .to_vec()
    }
    pub fn HTML(&self) -> Option<(Vec<u16>, String)> {
        (0..self.data.length())
            .filter_map(|i| self.data.Item(i))
            .find(|item| item.GetType() == "text/html")
            .map(|item| {
                (
                    item.GetAsString().Span16().unwrap_or_default().to_vec(),
                    item.BaseURL().unwrap_or("").into(),
                )
            })
    }
    pub fn URLs(&self) -> Vec<String> {
        if self.Contains("text/uri-list") {
            let value = self.data.GetData("text/uri-list");
            value
                .Span16()
                .unwrap_or_default()
                .split(|c| *c == 10)
                .filter_map(|line| {
                    let start = line
                        .iter()
                        .position(|c| !foundation::unicode::IsSpaceOrNewline(*c))?;
                    let end = line
                        .iter()
                        .rposition(|c| !foundation::unicode::IsSpaceOrNewline(*c))?
                        + 1;
                    if line[start] == u16::from(b'#') {
                        return None;
                    }
                    // KURL's UTF-16 input conversion replaces unpaired code units
                    // only at this URL parsing boundary; clipboard text stays WTF16.
                    let input: String = char::decode_utf16(line[start..end].iter().copied())
                        .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
                        .collect();
                    url::Url::parse(&input).ok().map(|url| url.to_string())
                })
                .collect()
        } else {
            self.Files()
                .iter()
                .filter(|file| !file.GetPath().is_empty())
                .filter_map(|file| {
                    url::Url::from_file_path(file.GetPath())
                        .ok()
                        .map(|url| url.to_string())
                })
                .collect()
        }
    }
    pub fn URLTitle(&self) -> String {
        (0..self.data.length())
            .filter_map(|i| self.data.Item(i))
            .find(|item| item.GetType() == "text/uri-list")
            .and_then(|item| item.Title().map(str::to_owned))
            .unwrap_or_default()
    }
    pub fn OperationForLoad(&self, did_initiate: bool) -> DragOperation {
        if self.ContainsURL() && !did_initiate {
            DragOperation::Copy
        } else {
            DragOperation::None
        }
    }
}

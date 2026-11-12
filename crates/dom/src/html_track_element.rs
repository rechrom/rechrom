//! Native HTMLTrackElement class identity and typed DOM traversal primitives.
//! Chromium html_track_element.h:53-55; loadable_text_track.cc:56-64.
use super::*;
impl PersistentDocument {
    pub fn IsHTMLTrackElement(&self, node: usize) -> bool {
        matches!(self.arena[node].class, DOMNodeClass::HTMLTrackElement)
    }
    pub fn PreviousHTMLTrackSibling(&self, node: usize) -> Option<usize> {
        let mut sibling = self.arena[node].previous_sibling;
        while let Some(node) = sibling {
            if self.IsHTMLTrackElement(node) {
                return Some(node);
            }
            sibling = self.arena[node].previous_sibling;
        }
        None
    }
    pub fn TrackParentMediaElement(&self, node: usize) -> Option<usize> {
        self.arena[node]
            .parent
            .filter(|&parent| self.IsMediaElement(parent))
    }
}

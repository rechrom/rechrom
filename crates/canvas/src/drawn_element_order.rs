//! HTMLCanvasElement::hit_testable_descendants_ (h:393-399) is a weak linked
//! set. Re-drawing moves the SAME element to the end in constant time.
use std::collections::HashMap;

/// Opaque canonical element identity, including its owning document arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasElementIdentity(pub u128);
struct Links {
    previous: Option<CanvasElementIdentity>,
    next: Option<CanvasElementIdentity>,
}
#[derive(Default)]
pub struct CanvasDrawnElementOrder {
    links: HashMap<CanvasElementIdentity, Links>,
    first: Option<CanvasElementIdentity>,
    last: Option<CanvasElementIdentity>,
}
impl CanvasDrawnElementOrder {
    pub fn AppendOrMoveToLast(&mut self, element: CanvasElementIdentity) {
        if self.last == Some(element) {
            return;
        }
        if let Some(old) = self.links.remove(&element) {
            if let Some(previous) = old.previous {
                self.links.get_mut(&previous).unwrap().next = old.next;
            } else {
                self.first = old.next;
            }
            if let Some(next) = old.next {
                self.links.get_mut(&next).unwrap().previous = old.previous;
            }
        }
        let previous = self.last;
        if let Some(last) = previous {
            self.links.get_mut(&last).unwrap().next = Some(element);
        } else {
            self.first = Some(element);
        }
        self.links.insert(
            element,
            Links {
                previous,
                next: None,
            },
        );
        self.last = Some(element);
    }
    pub fn Last(&self) -> Option<CanvasElementIdentity> {
        self.last
    }
    pub fn Previous(&self, element: CanvasElementIdentity) -> Option<CanvasElementIdentity> {
        self.links.get(&element)?.previous
    }
}

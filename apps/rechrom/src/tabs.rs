//! The embedding owns tabs. A tab keeps its document and session history while
//! another tab is visible; Page and the rendering crates do not own windows.
pub struct Tab<P> {
    pub id: u64,
    pub page: Option<P>,
    pub location: String,
    pub title: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub document_generation: u64,
    pub viewport: Option<crate::engine::Viewport>,
    pub title_node_id: Option<u64>,
    pub metadata_nodes: usize,
    pub metadata_sequence: u64,
}
impl<P> Tab<P> {
    fn new(id: u64) -> Self {
        Self {
            id,
            page: None,
            location: "about:home".into(),
            title: "New tab".into(),
            history: Vec::new(),
            history_index: 0,
            document_generation: 0,
            viewport: None,
            title_node_id: None,
            metadata_nodes: 0,
            metadata_sequence: 0,
        }
    }
}
pub struct Tabs<P> {
    pub active: Tab<P>,
    pub background: Vec<Tab<P>>,
    next_id: u64,
}
impl<P> Tabs<P> {
    pub fn new() -> Self {
        Self {
            active: Tab::new(1),
            background: Vec::new(),
            next_id: 2,
        }
    }
    pub fn add(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let old = std::mem::replace(&mut self.active, Tab::new(id));
        self.background.push(old);
        id
    }
    pub fn activate(&mut self, id: u64) -> bool {
        let Some(index) = self.background.iter().position(|tab| tab.id == id) else {
            return false;
        };
        std::mem::swap(&mut self.active, &mut self.background[index]);
        true
    }
    pub fn is_only(&self, id: u64) -> bool {
        self.active.id == id && self.background.is_empty()
    }
    // Return true when the visible tab changed. The native host owns the
    // final-tab/window decision, so the collection never fabricates a
    // replacement tab when the active tab is the last one.
    pub fn close(&mut self, id: u64) -> bool {
        if id != self.active.id {
            if let Some(index) = self.background.iter().position(|tab| tab.id == id) {
                self.background.remove(index);
            }
            return false;
        }
        if self.background.is_empty() {
            return false;
        }
        if let Some(index) = self
            .background
            .iter()
            .enumerate()
            .min_by_key(|(_, tab)| (tab.id.abs_diff(id), tab.id))
            .map(|(i, _)| i)
        {
            self.active = self.background.remove(index);
        }
        true
    }
    pub fn ordered(&self) -> Vec<&Tab<P>> {
        let mut tabs: Vec<_> = self
            .background
            .iter()
            .chain(std::iter::once(&self.active))
            .collect();
        tabs.sort_by_key(|tab| tab.id);
        tabs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactive_tab_retains_owned_document_history_and_order() {
        let mut tabs = Tabs::<Box<String>>::new();
        tabs.active.page = Some(Box::new("first document".into()));
        let identity = tabs.active.page.as_ref().unwrap().as_ref() as *const String;
        tabs.active.history = vec!["https://first.test".into()];
        tabs.active.location = "https://first.test".into();
        let second = tabs.add();
        tabs.active.page = Some(Box::new("second document".into()));
        assert!(tabs.activate(1));
        assert_eq!(
            tabs.active.page.as_ref().unwrap().as_ref() as *const String,
            identity
        );
        assert_eq!(tabs.active.history, ["https://first.test"]);
        assert_eq!(tabs.active.location, "https://first.test");
        assert_eq!(
            tabs.ordered().iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [1, second]
        );
        assert!(!tabs.activate(999));
        assert_eq!(tabs.active.id, 1);
        assert!(tabs.close(1));
        assert_eq!(tabs.active.id, second);
        assert_eq!(
            tabs.active.page.as_deref().map(String::as_str),
            Some("second document")
        );
        assert!(tabs.is_only(second));
        assert!(!tabs.close(second));
        assert_eq!(tabs.active.id, second);
        assert_eq!(
            tabs.active.page.as_deref().map(String::as_str),
            Some("second document")
        );
        assert!(tabs.background.is_empty());
    }
}

use dom::Document;
// cpp: interaction/event_path.h:15-24
// cpp: interaction/event_path.cc:9-12
// IDs retain the source target-first arena identity across callback mutation.
pub struct EventPath {
    nodes: Vec<u64>,
}
impl EventPath {
    pub fn new(document: &Document, target: usize) -> Self {
        let mut nodes = Vec::new();
        let mut node = Some(target);
        while let Some(i) = node {
            let n = document.Node(i);
            nodes.push(n.Id());
            node = n.Parent();
        }
        Self { nodes }
    }
    pub fn Nodes(&self) -> &[u64] {
        &self.nodes
    }
    pub fn Target(&self) -> u64 {
        self.nodes[0]
    }
    pub fn Empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

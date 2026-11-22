#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutTreeUpdateStats {
    pub created: usize,
    pub reused: usize,
    pub updated: usize,
    pub removed: usize,
    pub moved: usize,
}

use foundation::Visitor;

// cpp: layoutng/internal/node_rare_data_field.h:13-16
// The source's virtual trace method is intentionally empty.
pub struct NodeRareDataField;

#[allow(non_snake_case)]
impl NodeRareDataField {
    pub fn Trace(&self, _visitor: &mut Visitor) {}
}

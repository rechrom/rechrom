//! Host-owned external value/capability bindings. No tree, layer, pool or Page
//! is captured in these service objects; canonical owners consume them live.
use crate::adapter::{
    geometry::TracedValue,
    input_types::ElementId,
    layer_impl::LayerImplTraceValue,
    layer_tree_host_impl::HostTraceValue,
};
/// The actual external trace allocation supplies its own implicit-snapshot JSON
/// metadata operation. It cannot query or retain any compositor owner.
pub trait TraceValueBackend: HostTraceValue + LayerImplTraceValue {}
impl<T: HostTraceValue + LayerImplTraceValue> TraceValueBackend for T {}
pub struct TraceValue(pub Box<dyn TraceValueBackend>);
impl TracedValue for TraceValue {
    fn BeginDictionary(&mut self, name: &str) {
        self.0.BeginDictionary(name)
    }
    fn EndDictionary(&mut self) {
        self.0.EndDictionary()
    }
    fn BeginArray(&mut self, name: &str) {
        self.0.BeginArray(name)
    }
    fn EndArray(&mut self) {
        self.0.EndArray()
    }
    fn SetDouble(&mut self, name: &str, value: f64) {
        self.0.SetDouble(name, value)
    }
    fn SetInteger(&mut self, name: &str, value: i32) {
        self.0.SetInteger(name, value)
    }
    fn AppendDouble(&mut self, value: f64) {
        self.0.AppendDouble(value)
    }
    fn AppendInteger(&mut self, value: i32) {
        self.0.AppendInteger(value)
    }
}
impl compositor_tree::property_types::PropertyTrace for TraceValue {
    fn BeginDictionaryUnnamed(&mut self) {
        self.0.BeginDictionaryUnnamed()
    }
    fn SetBoolean(&mut self, name: &str, value: bool) {
        self.0.SetBoolean(name, value)
    }
    fn SetString(&mut self, name: &str, value: &str) {
        self.0.SetString(name, value)
    }
    fn AppendString(&mut self, value: &str) {
        self.0.AppendString(value)
    }
}
impl compositor_tile::tile_priority::TilePriorityTrace for TraceValue {
    fn SetTileString(&mut self, name: &str, value: &str) {
        self.0.SetString(name, value)
    }
}
impl LayerImplTraceValue for TraceValue {
    fn MakeDictIntoImplicitSnapshotWithCategory(
        &mut self,
        category: &str,
        base_type: &str,
        layer_type: &str,
        stable_id: i32,
    ) {
        self.0
            .MakeDictIntoImplicitSnapshotWithCategory(category, base_type, layer_type, stable_id)
    }
    fn ElementIdInto(&mut self, element: ElementId) {
        self.0.ElementIdInto(element)
    }
    fn ToFormattedJSON(&self) -> String {
        LayerImplTraceValue::ToFormattedJSON(self.0.as_ref())
    }
}
impl HostTraceValue for TraceValue {
    fn ToFormattedJSON(&self) -> String {
        HostTraceValue::ToFormattedJSON(self.0.as_ref())
    }
}

// Native GlobalState remains the single resident value owned by HostImpl.
impl crate::adapter::layer_tree_host_impl::HostImplGlobalTileState
    for crate::cc::tiles::tile_priority::GlobalStateThatImpactsTilePriority
{
    fn set_tree_priority(&mut self, value: crate::cc::tiles::tile_priority::TreePriority) {
        self.tree_priority = value
    }
    fn set_viewport_size(&mut self, value: crate::adapter::geometry::Size) {
        self.viewport_size = value
    }
    fn hard_memory_limit_in_bytes(&self) -> usize {
        self.hard_memory_limit_in_bytes
    }
    fn soft_memory_limit_in_bytes(&self) -> usize {
        self.soft_memory_limit_in_bytes
    }
    fn num_resources_limit(&self) -> usize {
        self.num_resources_limit
    }
    fn set_hard_memory_limit_in_bytes(&mut self, value: usize) {
        self.hard_memory_limit_in_bytes = value
    }
    fn set_soft_memory_limit_in_bytes(&mut self, value: usize) {
        self.soft_memory_limit_in_bytes = value
    }
    fn set_num_resources_limit(&mut self, value: usize) {
        self.num_resources_limit = value
    }
    fn set_memory_limit_policy(
        &mut self,
        value: crate::adapter::resource_types::TileMemoryLimitPolicy,
    ) {
        self.memory_limit_policy = value
    }
}

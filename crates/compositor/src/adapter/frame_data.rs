//! Viz values and tracing services consumed by canonical FrameData. No frame
//! preparation, surface selection, render-pass filtering or tree state lives here.
use super::layer_tree_impl::LayerTreeId;
use std::{collections::HashMap, hash::Hash};

pub trait FrameDataTypes {
    type RenderPassId: Copy + Ord + From<u64>;
    type SharedQuadStateHandle: Copy;
    type DrawQuadHandle: Copy;
    type SurfaceId;
    type ResourceId: Eq + Hash;
    type RenderPass: FrameDataRenderPass<ResourceId = Self::ResourceId>;
    type BeginFrameAck: Default;
    type TreesInVizTiming: Clone;
}

/// A live, current active-tree borrow. The ID must identify this resident
/// allocation, never a recycled slot number or a newly materialized list.
pub trait FrameDataSurfaceList {
    fn ActiveTreeId(&self) -> LayerTreeId;
    fn GetActiveRenderSurfaceList(&self) -> &[i32];
}

pub trait FrameDataTraceValue {
    fn SetBoolean(&mut self, name: &str, value: bool);
    fn BeginArray(&mut self, name: &str);
    fn BeginDictionary(&mut self);
    fn EndDictionary(&mut self);
    fn EndArray(&mut self);
}

pub trait FrameDataTraceJson: FrameDataTraceValue {
    fn ToFormattedJSON(&self) -> String;
}

pub trait FrameDataTraceServices {
    type TracedValueJSON: FrameDataTraceJson;
    fn GetClientNameForMetrics(&self) -> Option<&str>;
    fn TraceCategoryEnabled(&self, category: &str) -> bool;
    fn NewTracedValueJSON(&self) -> Self::TracedValueJSON;
}

pub trait FrameDataRenderPass {
    type ResourceId: Eq + Hash;
    fn AsValueInto<V: FrameDataTraceValue>(
        &self,
        value: &mut V,
        resource_id_to_index_map: &mut HashMap<Self::ResourceId, usize>,
    );
}

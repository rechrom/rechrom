#![allow(non_snake_case)]

pub mod canvas_2d;
pub mod canvas_context_creation_attributes_core;
pub mod canvas_rendering_context;
pub mod child_paint_record;
pub use child_paint_record::{
    CanvasChildRecordPublication, CanvasDocumentIdentity, CanvasRecord, CanvasRecordReadLease,
    PublishedCanvasChildRecord,
};
pub mod draw_element_image;
pub mod drawn_element_order;
pub mod html_canvas_element;
pub use drawn_element_order::{CanvasDrawnElementOrder, CanvasElementIdentity};

pub use canvas_context_creation_attributes_core::{
    CanvasContextCreationAttributesCore, CanvasPowerPreference,
};
pub use canvas_rendering_context::{
    CanvasContextFactory, CanvasContextType, CanvasPaintResource, CanvasPixelSnapshot,
    CanvasRenderingContext,
};
pub use html_canvas_element::{HTMLCanvasElementState, ParseCanvasDimension};

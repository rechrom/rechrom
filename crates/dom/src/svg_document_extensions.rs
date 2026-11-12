//! SVGDocumentExtensions pan state and SVGSVGElement currentTranslate.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! svg_document_extensions.h:59-64,75; .cc:103-122;
//! svg_svg_element.cc:122-145. Document owns pan's saved delta; each actual
//! SVGSVGElement owns its user translation, separate from authored transforms.
use super::{DOMNamespace, PersistentDocument};
impl PersistentDocument {
    fn SVGPanRoot(&self) -> Option<usize> {
        self.DocumentElement()
            .filter(|&root| self.Node(root).IsElement("svg", DOMNamespace::kSVG))
    }
    pub fn SVGZoomAndPanEnabled(&self) -> bool {
        self.SVGPanRoot().is_none_or(|root| {
            if let Some(value) = self
                .SVGImageViewSpecFor(root)
                .and_then(|view| view.zoom_and_pan)
            {
                return value;
            }
            // SVGZoomAndPan ctor/ParseAttribute, cc:30-64: the absent
            // attribute keeps magnify; an authored unknown value is disabled.
            self.Node(root)
                .FindAttribute("zoomAndPan")
                .map_or(true, |a| a.value.starts_with("magnify"))
        })
    }

    pub fn SVGCurrentTranslate(&self, element: usize) -> [f32; 2] {
        self.svg_user_transforms
            .get(&element)
            .copied()
            .unwrap_or([0.; 2])
    }
    pub fn SetSVGCurrentTranslate(&mut self, element: usize, point: [f32; 2]) -> bool {
        assert!(self.Node(element).IsElement("svg", DOMNamespace::kSVG));
        self.svg_user_transforms.insert(element, point);
        // SVGSVGElement::UpdateUserTransform requests native layout and full
        // paint invalidation; the same existing geometry mutation is consumed
        // during the normal per-element Layout update.
        self.RecordLayoutGeometryChange(element)
    }
    pub fn StartSVGPan(&mut self, start: [f32; 2]) {
        if let Some(root) = self.SVGPanRoot() {
            let current = self.SVGCurrentTranslate(root);
            self.svg_pan_translate = [start[0] - current[0], start[1] - current[1]];
        }
    }
    pub fn UpdateSVGPan(&mut self, position: [f32; 2]) -> bool {
        if let Some(root) = self.SVGPanRoot() {
            return self.SetSVGCurrentTranslate(
                root,
                [
                    position[0] - self.svg_pan_translate[0],
                    position[1] - self.svg_pan_translate[1],
                ],
            );
        }
        false
    }
}

//! HTMLMapElement/HTMLAreaElement and TreeScope image-map association owners.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
#![allow(non_snake_case)]
use super::{editing_node::DOMNodeClass as C, DOMNodeType, PersistentDocument};
pub use foundation::native_path::{
    ImageMapAreaShape as DOMImageMapAreaShape, ImageMapContainer as DOMImageMapContainer,
};
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform,
    gfx::{PointF, RectF},
    native_path::{NativePath, NativePathFactory},
    LayoutUnit, PhysicalOffset, WindRule,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

#[derive(Clone, Debug, Default)]
pub(super) struct HTMLMapElementData {
    name: Option<String>,
    registered_scope: Option<usize>,
}
#[derive(Clone)]
pub(super) struct HTMLAreaElementData {
    shape: DOMImageMapAreaShape,
    coordinates: Rc<[f64]>,
    // Cache only the original unzoomed path, exactly as HTMLAreaElement::path_.
    path: RefCell<Option<Arc<dyn NativePath>>>,
}
impl Default for HTMLAreaElementData {
    fn default() -> Self {
        Self {
            shape: DOMImageMapAreaShape::kRect,
            coordinates: Rc::from([]),
            path: RefCell::new(None),
        }
    }
}
impl std::fmt::Debug for HTMLAreaElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HTMLAreaElement")
            .field("shape", &self.shape)
            .field("coordinates", &self.coordinates)
            .field("cached_path", &self.path.borrow().is_some())
            .finish()
    }
}
#[derive(Default)]
struct TreeOrderedMapEntry {
    elements: HashSet<usize>,
    first: Option<usize>,
}
#[derive(Default)]
pub(super) struct ImageMapDocumentState {
    by_scope: HashMap<usize, HashMap<String, TreeOrderedMapEntry>>,
    // HTMLCollection kDocImages has no attribute membership invalidation.
    images: HashMap<usize, DocumentImagesCollection>,
}
#[derive(Default)]
struct DocumentImagesCollection {
    elements: Vec<usize>,
    // Borrow-free cursor into the live arena. Structural invalidation discards
    // it; this is collection traversal progress, not another DOM tree.
    traversal: Vec<(usize, usize)>,
    started: bool,
    finished: bool,
}
fn ClampCoordinate(value: f64) -> f32 {
    LayoutUnit::from_f64(value).ToFloat()
}
// html_parser_idioms.cc:243-251,290-320. The scalar converter is the same
// shared source double-conversion implementation; parsing happens at ParseAttribute.
fn ParseCoordinates(input: &str) -> Rc<[f64]> {
    let delimiter = |byte: u8| matches!(byte, b'\t' | b'\n' | 0x0c | b'\r' | b' ' | b',' | b';');
    let start = |byte: u8| byte.is_ascii_digit() || matches!(byte, b'.' | b'-');
    let bytes = input.as_bytes();
    let mut cursor = 0;
    let mut result = Vec::new();
    while cursor < bytes.len() {
        while cursor < bytes.len() && delimiter(bytes[cursor]) {
            cursor += 1;
        }
        if cursor == bytes.len() {
            break;
        }
        while cursor < bytes.len() && !delimiter(bytes[cursor]) && !start(bytes[cursor]) {
            cursor += 1;
        }
        let begin = cursor;
        while cursor < bytes.len() && !delimiter(bytes[cursor]) {
            cursor += 1;
        }
        let units = input[begin..cursor].encode_utf16().collect::<Vec<_>>();
        let parsed = foundation::number_conversion::dtoa::ParseDoubleUtf16(
            &units,
            &foundation::number_conversion::double_conversion::ChromiumDoubleConversion,
        );
        let value = if parsed.parsed_length != 0 && parsed.value.is_finite() {
            parsed.value
        } else {
            0.0
        };
        result.push(if value == 0.0 { 0.0 } else { value });
    }
    result.into()
}
impl PersistentDocument {
    pub(super) fn InitializeImageMapElement(&mut self, element: usize) {
        if !matches!(
            self.arena[element].class,
            C::HTMLMapElement(_) | C::HTMLAreaElement { .. }
        ) {
            return;
        }
        for index in 0..self.arena[element].attributes.len() {
            if self.arena[element].attributes[index]
                .namespace_uri
                .is_empty()
            {
                let name = self.arena[element].attributes[index].local_name.clone();
                self.ImageMapAttributeChanged(element, &name, "", None);
            }
        }
    }
    // html_map_element.cc:98-138; html_area_element.cc:64-92.
    pub(super) fn ImageMapAttributeChanged(
        &mut self,
        element: usize,
        name: &str,
        namespace: &str,
        old: Option<&str>,
    ) {
        if !namespace.is_empty() {
            return;
        }
        if matches!(self.arena[element].class, C::HTMLMapElement(_))
            && matches!(name, "name" | "id")
        {
            let old_id = if name == "id" {
                old.map(str::to_owned)
            } else {
                self.arena[element]
                    .FindAttribute("id")
                    .map(|a| a.value.clone())
            };
            self.UnregisterImageMap(element, old_id.as_deref());
            if name == "name" {
                let value = self.arena[element]
                    .FindAttribute("name")
                    .map(|a| a.value.strip_prefix('#').unwrap_or(&a.value).to_owned());
                let C::HTMLMapElement(data) = &mut self.arena[element].class else {
                    unreachable!()
                };
                // FixMapElementEmptyNameBug is stable in the pinned metadata.
                data.name = value;
            }
            self.RegisterImageMap(element);
            return;
        }
        if !matches!(name, "shape" | "coords") {
            return;
        }
        let node = &mut self.arena[element];
        let value = node
            .attributes
            .iter()
            .find(|attribute| attribute.namespace_uri.is_empty() && attribute.local_name == name)
            .map(|attribute| attribute.value.as_str());
        let C::HTMLAreaElement { data, .. } = &mut node.class else {
            return;
        };
        match name {
            "shape" => {
                let value = value.unwrap_or("");
                data.shape = if value.eq_ignore_ascii_case("default") {
                    DOMImageMapAreaShape::kDefault
                } else if value.eq_ignore_ascii_case("circle") || value.eq_ignore_ascii_case("circ")
                {
                    DOMImageMapAreaShape::kCircle
                } else if value.eq_ignore_ascii_case("polygon")
                    || value.eq_ignore_ascii_case("poly")
                {
                    DOMImageMapAreaShape::kPoly
                } else {
                    DOMImageMapAreaShape::kRect
                };
            }
            "coords" => data.coordinates = ParseCoordinates(value.unwrap_or("")),
            _ => return,
        }
        *data.path.borrow_mut() = None;
    }
    fn AddImageMapKey(&self, scope: usize, key: &str, map: usize) {
        let mut state = self.image_map_state.borrow_mut();
        let entry = state
            .by_scope
            .entry(scope)
            .or_default()
            .entry(key.to_owned())
            .or_default();
        entry.elements.insert(map);
        entry.first = if entry.elements.len() == 1 {
            Some(map)
        } else {
            None
        };
    }
    fn RemoveImageMapKey(&self, scope: usize, key: &str, map: usize) {
        let mut state = self.image_map_state.borrow_mut();
        let Some(maps) = state.by_scope.get_mut(&scope) else {
            return;
        };
        if let Some(entry) = maps.get_mut(key) {
            entry.elements.remove(&map);
            if entry.first == Some(map) {
                entry.first = None;
            }
            if entry.elements.len() == 1 {
                entry.first = entry.elements.iter().next().copied();
            }
            if entry.elements.is_empty() {
                maps.remove(key);
            }
        }
        if maps.is_empty() {
            state.by_scope.remove(&scope);
        }
    }
    fn RegisterImageMap(&mut self, map: usize) {
        if !self.IsConnectedToDocument(map) {
            return;
        }
        let scope = self.TreeScopeRoot(map);
        let C::HTMLMapElement(data) = &self.arena[map].class else {
            return;
        };
        assert!(data.registered_scope.is_none());
        let name = data.name.clone();
        let id = self.arena[map].FindAttribute("id").map(|a| a.value.clone());
        if let Some(name) = name {
            self.AddImageMapKey(scope, &name, map);
        }
        if let Some(id) = id {
            self.AddImageMapKey(scope, &id, map);
        }
        let C::HTMLMapElement(data) = &mut self.arena[map].class else {
            unreachable!()
        };
        data.registered_scope = Some(scope);
    }
    fn UnregisterImageMap(&mut self, map: usize, old_id: Option<&str>) {
        let C::HTMLMapElement(data) = &self.arena[map].class else {
            return;
        };
        let Some(scope) = data.registered_scope else {
            return;
        };
        if let Some(name) = &data.name {
            self.RemoveImageMapKey(scope, name, map);
        }
        if let Some(id) = old_id {
            self.RemoveImageMapKey(scope, id, map);
        }
        let C::HTMLMapElement(data) = &mut self.arena[map].class else {
            unreachable!()
        };
        data.registered_scope = None;
    }
    // Map insertion/removal reuse ContainerNode's existing element-wise walk.
    pub(super) fn ImageMapElementInserted(&mut self, node: usize) {
        if let C::HTMLMapElement(data) = &self.arena[node].class {
            if data.registered_scope.is_none() {
                self.RegisterImageMap(node);
            }
        }
    }
    pub(super) fn ImageMapElementRemoved(&mut self, node: usize) {
        if matches!(self.arena[node].class, C::HTMLMapElement(_)) {
            let id = self.arena[node]
                .FindAttribute("id")
                .map(|a| a.value.clone());
            self.UnregisterImageMap(node, id.as_deref());
        }
    }
    pub(super) fn InvalidateDocumentImagesCollection(&self, parent: usize) {
        let root = self.TreeScopeRoot(parent);
        if self.arena[root].node_type == DOMNodeType::kDocument {
            self.image_map_state.borrow_mut().images.remove(&root);
        }
    }
    // Document::images / HTMLCollection kDocImages. Cache only membership in
    // the actual Document; a map's reverse association still iterates images.
    pub fn DocumentImageElementAtIndex(&self, document: usize, index: usize) -> Option<usize> {
        assert_eq!(self.arena[document].node_type, DOMNodeType::kDocument);
        let mut state = self.image_map_state.borrow_mut();
        let images = state.images.entry(document).or_default();
        if let Some(&image) = images.elements.get(index) {
            return Some(image);
        }
        if images.finished {
            return None;
        }
        if !images.started {
            images.started = true;
            images.traversal.push((document, 0));
        }
        while let Some((parent, next)) = images.traversal.last_mut() {
            let Some(&node) = self.arena[*parent].children.get(*next) else {
                images.traversal.pop();
                continue;
            };
            *next += 1;
            images.traversal.push((node, 0));
            // HTMLCollection kDocImages, html_collection.cc:212-213.
            if self.arena[node].IsHTMLElement("img") {
                images.elements.push(node);
                if images.elements.len() > index {
                    return Some(node);
                }
            }
        }
        images.finished = true;
        None
    }
    fn MapMatchesName(&self, map: usize, name: &str) -> bool {
        matches!(&self.arena[map].class,C::HTMLMapElement(data) if data.name.as_deref()==Some(name) || self.arena[map].FindAttribute("id").is_some_and(|a|a.value==name))
    }
    // tree_scope.cc:204-222, tree_ordered_map.cc:62-66,95-145,188-191.
    pub fn ImageMapForImage(&self, image_id: u64) -> Option<u64> {
        let image = self.FindNodeById(image_id)?;
        if !matches!(
            self.arena[image].class,
            C::HTMLImageElement | C::HTMLAltTextImageElement
        ) {
            return None;
        }
        let value = &self.arena[image].FindAttribute("usemap")?.value;
        let fragment = value.get(value.find('#')? + 1..)?;
        if fragment.is_empty() {
            return None;
        }
        let scope = self.TreeScopeRoot(image);
        let candidates = self.image_map_state.borrow();
        let entry = candidates.by_scope.get(&scope)?.get(fragment)?;
        if let Some(first) = entry.first {
            return Some(self.arena[first].id);
        }
        if entry.elements.len() == 1 {
            return entry.elements.iter().next().map(|&map| self.arena[map].id);
        }
        drop(candidates);
        // Duplicate map names use TreeOrderedMap's lazy tree-ordered lookup.
        let mut first = None;
        let mut stack = vec![self.arena[scope].children.iter()];
        while let Some(children) = stack.last_mut() {
            let Some(&node) = children.next() else {
                stack.pop();
                continue;
            };
            if self.MapMatchesName(node, fragment) {
                first = Some(node);
                break;
            }
            stack.push(self.arena[node].children.iter());
        }
        if let Some(entry) = self
            .image_map_state
            .borrow_mut()
            .by_scope
            .get_mut(&scope)
            .and_then(|m| m.get_mut(fragment))
        {
            entry.first = first;
        }
        first.map(|node| self.arena[node].id)
    }
    // html_map_element.cc:78-96: this intentionally uses Document.images,
    // not the map's TreeScope, and strips the first UTF-16 unit after length>1.
    pub(super) fn ImageElementForMap(&self, map: usize) -> Option<usize> {
        let C::HTMLMapElement(data) = &self.arena[map].class else {
            return None;
        };
        let document = self.NodeDocumentHandle(map);
        assert_eq!(
            document.arena_id, self.arena_id,
            "image map belongs to this document arena"
        );
        let root = self
            .FindNodeById(document.node_id)
            .expect("image map Document");
        let id = self.arena[map]
            .FindAttribute("id")
            .map(|a| a.value.as_str());
        let mut index = 0;
        while let Some(image) = self.DocumentImageElementAtIndex(root, index) {
            index += 1;
            let Some(value) = self.arena[image]
                .FindAttribute("usemap")
                .map(|a| a.value.as_str())
            else {
                continue;
            };
            let mut units = value.encode_utf16();
            units.next();
            if units.clone().next().is_some()
                && (data
                    .name
                    .as_ref()
                    .is_some_and(|name| units.clone().eq(name.encode_utf16()))
                    || id.is_some_and(|id| units.clone().eq(id.encode_utf16())))
            {
                return Some(image);
            }
        }
        None
    }
    pub(super) fn ImageElementForArea(&self, area: usize) -> Option<usize> {
        if !matches!(self.arena[area].class, C::HTMLAreaElement { .. }) {
            return None;
        }
        let mut ancestor = self.arena[area].parent;
        while let Some(node) = ancestor {
            if matches!(self.arena[node].class, C::HTMLMapElement(_)) {
                return self.ImageElementForMap(node);
            }
            ancestor = self.arena[node].parent;
        }
        None
    }
    pub fn ImageElementForMapOrArea(&self, node_id: u64) -> Option<u64> {
        let node = self.FindNodeById(node_id)?;
        let image = match self.arena[node].class {
            C::HTMLMapElement(_) => self.ImageElementForMap(node),
            C::HTMLAreaElement { .. } => self.ImageElementForArea(node),
            _ => None,
        }?;
        Some(self.arena[image].id)
    }
    /// Native preconditions of HTMLAreaElement::GetPath. This read neither
    /// reparses attributes nor copies the retained coordinate vector.
    pub fn ImageMapAreaHasPotentialPath(&self, area_id: u64) -> bool {
        let Some(area) = self.FindNodeById(area_id) else {
            return false;
        };
        let C::HTMLAreaElement { data, .. } = &self.arena[area].class else {
            return false;
        };
        match data.shape {
            DOMImageMapAreaShape::kDefault => true,
            DOMImageMapAreaShape::kRect => data.coordinates.len() >= 4,
            DOMImageMapAreaShape::kCircle => {
                data.coordinates.len() >= 3 && data.coordinates[2] > 0.0
            }
            DOMImageMapAreaShape::kPoly => data.coordinates.len() >= 6,
        }
    }
    pub(super) fn RegisteredImageMapPaintOwners(&self) -> Vec<Option<u64>> {
        let maps = self
            .image_map_state
            .borrow()
            .by_scope
            .values()
            .flat_map(|scope| scope.values())
            .flat_map(|entry| entry.elements.iter().copied())
            .collect::<HashSet<_>>();
        maps.into_iter()
            .map(|map| {
                self.ImageElementForMap(map)
                    .map(|image| self.arena[image].id)
            })
            .collect()
    }
    pub fn ImageMapAreaGeometry(&self, area: usize) -> Option<(DOMImageMapAreaShape, Rc<[f64]>)> {
        match &self.arena[area].class {
            C::HTMLAreaElement { data, .. } => Some((data.shape, data.coordinates.clone())),
            _ => None,
        }
    }
    // html_area_element.cc:111-191. Original Path is retained by HTMLAreaElement;
    // only source zoom/offset makes a transformed immutable native Path value.
    fn AreaPath(
        &self,
        area: usize,
        container: DOMImageMapContainer,
        offset: PointF,
        paths: &dyn NativePathFactory,
    ) -> Arc<dyn NativePath> {
        let C::HTMLAreaElement { data, .. } = &self.arena[area].class else {
            return paths.Empty();
        };
        if data.shape == DOMImageMapAreaShape::kDefault {
            *data.path.borrow_mut() = None;
            return match container.border_box {
                Some(rect) => paths.Rect(
                    PointF::new(rect.x() + offset.x(), rect.y() + offset.y()),
                    PointF::new(rect.right() + offset.x(), rect.bottom() + offset.y()),
                ),
                None => paths.Empty(),
            };
        }
        let cached = data.path.borrow().clone();
        let original = if let Some(path) = cached {
            path
        } else {
            let c = &data.coordinates;
            if c.is_empty() {
                return paths.Empty();
            }
            let path = match data.shape {
                DOMImageMapAreaShape::kPoly if c.len() >= 6 => {
                    let points = c
                        .chunks_exact(2)
                        .map(|c| PointF::new(ClampCoordinate(c[0]), ClampCoordinate(c[1])))
                        .collect::<Vec<_>>();
                    paths.Polygon(&points, WindRule::RULE_EVENODD)
                }
                DOMImageMapAreaShape::kCircle if c.len() >= 3 && c[2] > 0.0 => {
                    let radius = ClampCoordinate(c[2]);
                    paths.Ellipse(
                        PointF::new(ClampCoordinate(c[0]), ClampCoordinate(c[1])),
                        radius,
                        radius,
                    )
                }
                DOMImageMapAreaShape::kRect if c.len() >= 4 => paths.Rect(
                    PointF::new(ClampCoordinate(c[0]), ClampCoordinate(c[1])),
                    PointF::new(ClampCoordinate(c[2]), ClampCoordinate(c[3])),
                ),
                _ => paths.Empty(),
            };
            *data.path.borrow_mut() = Some(path.clone());
            path
        };
        if container.effective_zoom != 1.0 || offset.x() != 0.0 || offset.y() != 0.0 {
            let mut transform = AffineTransform::default();
            transform
                .Translate(offset.x() as f64, offset.y() as f64)
                .Scale(
                    container.effective_zoom as f64,
                    container.effective_zoom as f64,
                );
            original.Transformed(&transform)
        } else {
            original
        }
    }
    // html_map_element.cc:61-76. The first default is saved as fallback; later
    // defaults still pass through PointInArea, preserving the source ordering.
    pub fn ImageMapAreaForPoint(
        &self,
        map_id: u64,
        point: PhysicalOffset,
        container: DOMImageMapContainer,
        paths: &dyn NativePathFactory,
    ) -> Option<u64> {
        let map = self.FindNodeById(map_id)?;
        if !matches!(self.arena[map].class, C::HTMLMapElement(_)) {
            return None;
        }
        let mut default = None;
        let mut stack = vec![self.arena[map].children.iter()];
        while let Some(children) = stack.last_mut() {
            let Some(&node) = children.next() else {
                stack.pop();
                continue;
            };
            if let C::HTMLAreaElement { data, .. } = &self.arena[node].class {
                if data.shape == DOMImageMapAreaShape::kDefault && default.is_none() {
                    default = Some(node);
                } else if self
                    .AreaPath(node, container, PointF::default(), paths)
                    .Contains(
                        PointF::new(point.left.ToFloat(), point.top.ToFloat()),
                        if data.shape == DOMImageMapAreaShape::kPoly {
                            WindRule::RULE_EVENODD
                        } else {
                            WindRule::RULE_NONZERO
                        },
                    )
                {
                    return Some(self.arena[node].id);
                }
            }
            stack.push(self.arena[node].children.iter());
        }
        default.map(|node| self.arena[node].id)
    }
}
impl PersistentDocument {
    /// HTMLAreaElement::ComputeAbsoluteRect, cc96-108. Native Layout supplies
    /// the original image container and IgnoreTransforms origin; this reuses
    /// HTMLAreaElement's retained Path and source zoom transform.
    pub fn ImageMapAreaAbsoluteRect(
        &self,
        area: usize,
        container: DOMImageMapContainer,
        absolute_origin: PointF,
        paths: &dyn NativePathFactory,
    ) -> foundation::PhysicalRect {
        foundation::PhysicalRect::EnclosingRect(
            &self
                .AreaPath(area, container, absolute_origin, paths)
                .BoundingRect(),
        )
    }
}

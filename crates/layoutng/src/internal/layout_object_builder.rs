#![allow(non_snake_case)]

use super::boundary::layout_boundary::{
    ConstructionFontResolver, CreateLayoutObject, RequiresPaintBackedInlineFragment,
};
use super::boundary::native_input::{
    InvalidLayoutInput, NativeFontResolverScope, PrepareNativeStyle,
};
use super::boundary::node_metadata_input::CreateNativeElementMetadata;
use super::constraint_space::FragmentationType;
use super::layout_font_resolver::NativeFontResolver;
use super::layout_inline::LayoutInline;
use super::layout_input::{
    ComputedStyle, ConstraintSpace, ElementData, NativeNodeConstructionData, NodeKind,
};
use super::layout_node_metadata::{
    ContainerNode, NativeNodeMetadataRelations, Node, Text as TextNode,
};
use super::layout_object::{AllowDestroyingLayoutObjectInFinalizerScope, LayoutObject};
use super::layout_object_factory_set::LayoutObjectFactorySet;
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::layout_text::LayoutText;
use foundation::{
    DynamicTo, InitStringStatics, LayoutHeapScope, Length, MakeGarbageCollected, Persistent,
    RuntimeEnabledFeatures, String as BlinkString,
};
use layoutng_style::style::computed_style::ComputedStyle as NativeComputedStyle;
use std::collections::HashMap;

// Native theme state and scroll translations affect paint, not box geometry.
// Keep their new values in live metadata, excluding them only from comparison.
fn PreserveNonGeometryMetadata(
    new: &mut NativeNodeConstructionData,
    old: &NativeNodeConstructionData,
) {
    if let (Some(new), Some(old)) = (&mut new.element, &old.element) {
        new.control_hovered = old.control_hovered;
        new.control_active = old.control_active;
        new.control_focused = old.control_focused;
        new.scroll_offset = old.scroll_offset;
    }
}

// The one-argument constructor is defined by //src/main/default_layout_engine.cc
// and supplies FullLayoutAssembly().objects from outside this Bazel package.
unsafe extern "Rust" {
    fn LayoutObjectTreeNew(space: &ConstraintSpace) -> LayoutObjectTree;
}

// cpp: layoutng/internal/layout_object_builder.h:17-78
pub struct LayoutObjectTree {
    storage_: *mut LayoutObjectTreeStorage,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayoutTreeUpdateStats {
    pub created: usize,
    pub reused: usize,
    pub updated: usize,
    pub removed: usize,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct InputKey(u64, i32, bool, String, usize);

struct ManagedObject {
    object: *mut LayoutObject,
    node: *mut Node,
    parent: *mut LayoutObject,
    input: NativeNodeConstructionData,
    effective_style: ComputedStyle,
}

// cpp: layoutng/internal/layout_object_builder.h:75-77
// cpp: layoutng/internal/boundary/layout_boundary.cc:2181-2191
// cpp: layoutng/internal/boundary/layout_boundary.cc:2355-2362
pub struct LayoutObjectTreeStorage {
    fonts: Box<dyn NativeFontResolver>,
    factories: LayoutObjectFactorySet,
    metadata: Vec<Persistent<Node>>,
    objects: Vec<Persistent<LayoutObject>>,
    root: Persistent<LayoutObject>,
    managed: HashMap<InputKey, ManagedObject>,
    pending: HashMap<InputKey, ManagedObject>,
    occurrences: HashMap<(u64, i32, bool, String), usize>,
    last_child: HashMap<*mut LayoutObject, *mut Node>,
    updating: bool,
    stats: LayoutTreeUpdateStats,
    editing: std::rc::Rc<crate::editing_state::LayoutEditingState>,
}

impl LayoutObjectTreeStorage {
    fn new(space: &ConstraintSpace, factories: &LayoutObjectFactorySet) -> Self {
        let _heap_scope = LayoutHeapScope::new();
        let fonts = ConstructionFontResolver(space);
        Self {
            fonts,
            factories: *factories,
            metadata: Vec::new(),
            objects: Vec::new(),
            root: Persistent::from_ptr(std::ptr::null_mut()),
            managed: HashMap::new(),
            pending: HashMap::new(),
            occurrences: HashMap::new(),
            last_child: HashMap::new(),
            updating: false,
            stats: LayoutTreeUpdateStats::default(),
            editing: std::rc::Rc::new(Default::default()),
        }
    }

    fn Compatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NativeNodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        old.parent == parent
            && old.input.kind == input.kind
            && old.effective_style.display == effective.display
            && old.effective_style.position == effective.position
            && old.effective_style.floating == effective.floating
            && old.input.element.as_ref().and_then(|e| e.form_control_type)
                == input.element.as_ref().and_then(|e| e.form_control_type)
            && old.input.element.as_ref().map(|e| {
                (
                    e.html_image,
                    e.image_map_area,
                    e.first_letter_pseudo,
                    e.text_control_inner_editor,
                    e.text_control_spin_button,
                    e.select_uses_menu_list,
                )
            }) == input.element.as_ref().map(|e| {
                (
                    e.html_image,
                    e.image_map_area,
                    e.first_letter_pseudo,
                    e.text_control_inner_editor,
                    e.text_control_spin_button,
                    e.select_uses_menu_list,
                )
            })
            && old
                .effective_style
                .extended
                .as_ref()
                .map(|e| e.initial_letter)
                == effective.extended.as_ref().map(|e| e.initial_letter)
            && old
                .effective_style
                .extended
                .as_ref()
                .map(|e| &e.timeline_trigger_names)
                == effective
                    .extended
                    .as_ref()
                    .map(|e| &e.timeline_trigger_names)
    }

    fn Place(&mut self, parent: *mut LayoutObject, object: *mut LayoutObject, node: *mut Node) {
        if parent.is_null() {
            return;
        }
        let parent_node = unsafe { &*parent }.GetNode().cast::<ContainerNode>();
        let previous = self
            .last_child
            .get(&parent)
            .copied()
            .unwrap_or(std::ptr::null_mut());
        let linked = unsafe { &*node };
        if linked.parentNode() != parent_node
            || linked.previousSibling() != previous
            || unsafe { &*object }.Parent().is_null()
        {
            let mut before = if previous.is_null() {
                unsafe { &*parent_node }.firstChild()
            } else {
                unsafe { &*previous }.nextSibling()
            };
            if before == node {
                before = unsafe { &*node }.nextSibling();
            }
            let mut before_object = std::ptr::null_mut();
            let mut candidate = before;
            while !candidate.is_null() {
                let next = unsafe { &*candidate }.GetLayoutObject();
                if !next.is_null() && !unsafe { &*next }.WasDestroyedForTreeUpdate() {
                    before_object = next;
                    break;
                }
                candidate = unsafe { &*candidate }.nextSibling();
            }
            if !unsafe { &*object }.Parent().is_null() {
                unsafe { &mut *object }.Remove();
            }
            unsafe { &mut *parent_node }.InsertForTreeUpdate(unsafe { &mut *node }, before);
            unsafe { &mut *parent }.AddChild(object, before_object);
        }
        NativeNodeMetadataRelations::AttachIncrementalRelations(unsafe { &mut *node })
            .unwrap_or_else(|error| panic!("{error:?}"));
        self.last_child.insert(parent, node);
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:2193-2353
    #[allow(clippy::too_many_arguments)]
    fn Create(
        &mut self,
        parent: *mut LayoutObject,
        id: u64,
        kind: NodeKind,
        style: &ComputedStyle,
        element: Option<ElementData>,
        debug_name: String,
        text: String,
        explicit_text_style: *const ComputedStyle,
        style_generated: bool,
        source_text_style: *const ComputedStyle,
        first_line_style: *const ComputedStyle,
    ) -> *mut LayoutObject {
        let _heap_scope = LayoutHeapScope::new();
        let effective = if kind == NodeKind::kText {
            assert!(!parent.is_null(), "LayoutText cannot be the layout root");
            if !explicit_text_style.is_null() {
                unsafe { &*explicit_text_style }
            } else {
                let parent_node = unsafe { &*parent }.GetNode();
                unsafe { &*parent_node }.InputStyle()
            }
        } else {
            style
        };
        let source = if kind == NodeKind::kText && !source_text_style.is_null() {
            unsafe { &*source_text_style }
        } else {
            effective
        };
        let first_line = if first_line_style.is_null() {
            None
        } else {
            Some(unsafe { &*first_line_style })
        };
        let base = (id, kind as i32, style_generated, debug_name);
        let occurrence = self.occurrences.entry(base.clone()).or_default();
        let key = InputKey(base.0, base.1, base.2, base.3, *occurrence);
        *occurrence += 1;
        let mut previous = self.pending.remove(&key);
        // A complete projection still visits and compares every node. Compare
        // borrowed inputs before making owned descriptors: unchanged nodes need
        // neither style/string clones nor construction resolver scopes.
        if let Some(old) = previous.as_ref() {
            if !unsafe { &*old.object }.WasDestroyedForTreeUpdate()
                && old.parent == parent
                && old.input.id == id
                && old.input.kind == kind
                && old.input.style_generated == style_generated
                && old.input.style == *source
                && old.input.text == text
                && old.input.element == element
                && old.input.debug_name == key.3
                && old.input.first_line_style.as_ref() == first_line
                && old.effective_style == *effective
            {
                let object = old.object;
                let node = old.node;
                self.Place(parent, object, node);
                self.stats.reused += 1;
                self.managed.insert(key, previous.take().unwrap());
                return object;
            }
        }
        let fonts: *mut dyn NativeFontResolver = &mut *self.fonts;
        // SAFETY: `fonts` is boxed in this Storage and outlives the scope;
        // the scope only installs its pointer in the thread-local slot.
        let _font_scope = unsafe { NativeFontResolverScope::new_from_raw(fonts) };
        let _factory_scope = LayoutObjectFactoryScope::new(&self.factories);
        let mut input = NativeNodeConstructionData {
            id,
            kind,
            style_generated,
            style: style.clone(),
            element,
            debug_name: key.3.clone(),
            text,
            first_line_style: None,
        };
        let descriptor = NativeNodeConstructionData {
            id,
            kind,
            style_generated,
            style: source.clone(),
            element: input.element.clone(),
            debug_name: input.debug_name.clone(),
            text: input.text.clone(),
            first_line_style: first_line.cloned(),
        };
        let effective_style = effective.clone();
        if let Some(old) = &previous {
            if unsafe { &*old.object }.WasDestroyedForTreeUpdate()
                || !Self::Compatible(old, parent, &descriptor, &effective_style)
            {
                if !unsafe { &*old.object }.WasDestroyedForTreeUpdate() {
                    unsafe { &mut *old.object }.Destroy();
                    unsafe { &mut *old.node }.DetachForTreeUpdate();
                }
                if old.object == self.root.Get() {
                    self.root = Persistent::from_ptr(std::ptr::null_mut());
                }
                self.stats.removed += 1;
                previous = None;
            }
        }
        let paint_only = previous.as_ref().is_some_and(|old| {
            let mut unchanged = descriptor.clone();
            unchanged.style = old.input.style.clone();
            PreserveNonGeometryMetadata(&mut unchanged, &old.input);
            unchanged == old.input
                && old.input.style.LayoutEquivalent(&descriptor.style)
                && old.effective_style.LayoutEquivalent(&effective_style)
        });
        let mut context = FragmentationType::kFragmentPage;
        let mut ancestor = parent;
        while !ancestor.is_null() {
            let node = unsafe { &*ancestor }.GetNode();
            if !node.is_null() && unsafe { &*node }.InputStyle().column_count > 1 {
                context = FragmentationType::kFragmentColumn;
                break;
            }
            ancestor = unsafe { &*ancestor }.Parent();
        }
        let mut prepare_style = |data: &NativeNodeConstructionData,
                                 parent_style: *const NativeComputedStyle,
                                 parent_kind: NodeKind| {
            // SAFETY: Create runs on one thread, and the TLS scope is restored
            // before the boxed resolver can be moved or destroyed.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                PrepareNativeStyle(
                    data,
                    unsafe { &mut *fonts },
                    context,
                    parent_style,
                    parent_kind,
                )
            }));
            match result {
                Ok(style) => style,
                Err(error) => {
                    if let Some(invalid) = error.downcast_ref::<InvalidLayoutInput>() {
                        let mut detail = String::new();
                        if invalid.to_string() == "Invalid layout input: width" {
                            if let Some(extra) = data.style.extended.as_ref() {
                                let value = |number: Option<f64>| {
                                    number.map_or_else(
                                        || "none".to_string(),
                                        |number| format!("{number:.6}"),
                                    )
                                };
                                detail = format!(
                                    " [px={}, percent={}, calculated={}, sizing={}]",
                                    value(data.style.width),
                                    value(extra.width_percent),
                                    extra.width_calculated as u8,
                                    extra.width_sizing as u32,
                                );
                            }
                        }
                        panic!("Invalid style for {}: {invalid}{detail}", data.debug_name);
                    }
                    std::panic::resume_unwind(error)
                }
            }
        };
        let parent_kind = if parent.is_null() {
            NodeKind::kBox
        } else {
            let parent_node = unsafe { &*parent }.GetNode();
            if parent_node.is_null() {
                NodeKind::kBox
            } else {
                unsafe { &*parent_node }.InputKind()
            }
        };
        // Geometry-equivalent colors live in input paint data. Keep the native
        // style (including the font bound from the actual constraint space).
        // Preparing it again here would replace that font with construction data.
        let native_style = if paint_only {
            if kind == NodeKind::kText {
                input.style = if !explicit_text_style.is_null() {
                    unsafe { &*explicit_text_style }.clone()
                } else {
                    let parent_node = unsafe { &*parent }.GetNode();
                    unsafe { &*parent_node }.InputStyle().clone()
                };
            }
            unsafe { &*previous.as_ref().unwrap().object }.StylePtr()
        } else if kind == NodeKind::kText {
            if parent.is_null() {
                panic!("LayoutText cannot be the layout root");
            }
            if !explicit_text_style.is_null() {
                input.style = unsafe { &*explicit_text_style }.clone();
                prepare_style(&input, unsafe { &*parent }.StyleRef(), parent_kind)
            } else {
                let parent_node = unsafe { &*parent }.GetNode();
                input.style = unsafe { &*parent_node }.InputStyle().clone();
                unsafe { &*parent }.StyleRef() as *const NativeComputedStyle
            }
        } else {
            let parent_style = if parent.is_null() {
                std::ptr::null()
            } else {
                unsafe { &*parent }.StyleRef() as *const NativeComputedStyle
            };
            prepare_style(&input, parent_style, parent_kind)
        };
        let mut native_first_line_style: *const NativeComputedStyle = std::ptr::null();
        if !first_line_style.is_null() {
            if kind == NodeKind::kText {
                panic!("first-line style belongs to the text's containing element");
            }
            input.first_line_style = Some(unsafe { &*first_line_style }.clone());
            let mut first_line_input = input.clone();
            first_line_input.style = unsafe { &*first_line_style }.clone();
            let mut parent_first_line_style = std::ptr::null();
            if !parent.is_null() {
                let parent_node = unsafe { &*parent }.GetNode();
                parent_first_line_style = unsafe { &*parent_node }.InputFirstLineStyle();
                if parent_first_line_style.is_null() {
                    parent_first_line_style = unsafe { &*parent }.StyleRef();
                }
            }
            native_first_line_style = if paint_only {
                unsafe { &*previous.as_ref().unwrap().node }.InputFirstLineStyle()
            } else {
                prepare_style(&first_line_input, parent_first_line_style, parent_kind)
            };
        }
        if kind == NodeKind::kText && !source_text_style.is_null() {
            input.style = unsafe { &*source_text_style }.clone();
        }
        let old_font_dirty = previous
            .as_ref()
            .is_some_and(|old| unsafe { &*old.node }.NeedsInputFontBinding());
        let metadata_node: *mut Node = if let Some(old) = &previous {
            if kind == NodeKind::kText {
                unsafe { &mut *old.node }.UpdateInputData(&input);
                // Style/resource updates still refresh live metadata, but an
                // unchanged text payload already owns the same native string.
                if old.input.text != input.text {
                    unsafe { &mut *old.node.cast::<TextNode>() }
                        .SetDataForTreeUpdate(BlinkString::FromUtf8(input.text.as_bytes()));
                }
            } else {
                unsafe { &mut *old.node.cast::<super::layout_node_metadata::Element>() }
                    .UpdateElementInput(&input);
            }
            unsafe { &mut *old.node }.SetComputedStyle(native_style);
            old.node
        } else if kind == NodeKind::kText {
            let native_text = BlinkString::FromUtf8(input.text.as_bytes());
            if native_text.IsNull() {
                panic!("LayoutText must contain valid UTF-8");
            }
            MakeGarbageCollected(
                TextNode::new(&input, native_style, native_text)
                    .unwrap_or_else(|error| panic!("{error:?}")),
            ) as *mut Node
        } else {
            CreateNativeElementMetadata(&input, native_style, &self.factories)
                .unwrap_or_else(|error| panic!("{error:?}")) as *mut Node
        };
        if paint_only && !old_font_dirty {
            unsafe { &mut *metadata_node }.ClearInputFontBinding();
        }
        unsafe { &mut *metadata_node }.SetInputFirstLineStyle(native_first_line_style);
        if previous.is_none() && !parent.is_null() {
            let parent_node = unsafe { &*parent }.GetNode().cast::<ContainerNode>();
            unsafe { &mut *parent_node }
                .AppendChild(unsafe { &mut *metadata_node })
                .unwrap_or_else(|error| panic!("{error:?}"));
        }
        NativeNodeMetadataRelations::AttachIncrementalRelations(unsafe { &mut *metadata_node })
            .unwrap_or_else(|error| panic!("{error:?}"));
        let object = if let Some(old) = &previous {
            self.stats.updated += 1;
            if kind == NodeKind::kText && old.input.text != input.text {
                unsafe { &mut *DynamicTo::<LayoutText>(old.object) }
                    .ForceSetText(BlinkString::FromUtf8(input.text.as_bytes()));
            }
            // Colors, native control flags and scroll offsets affect paint.
            // Retain geometry while updating metadata; other inputs remain conservative.
            let mut layout_descriptor = descriptor.clone();
            layout_descriptor.style = old.input.style.clone();
            PreserveNonGeometryMetadata(&mut layout_descriptor, &old.input);
            if layout_descriptor != old.input
                || !old.effective_style.LayoutEquivalent(&effective_style)
                || !old.input.style.LayoutEquivalent(&descriptor.style)
            {
                unsafe { &mut *old.object }.SetNeedsLayoutAndIntrinsicWidthsRecalc(
                    &raw const super::layout_invalidation_reason::kStyleChange,
                );
            }
            old.object
        } else {
            self.stats.created += 1;
            CreateLayoutObject(
                unsafe { &mut *metadata_node },
                unsafe { &*native_style },
                &self.factories,
            )
        };
        unsafe { &mut *object }.SetStyle(native_style);
        if let Some(element) = input.element.as_ref() {
            unsafe { &mut *metadata_node }.UpdateInputScrollOffset(element.scroll_offset);
        }
        if kind != NodeKind::kText {
            let paint = &input.style.paint;
            let establishes_fixed_container = !paint.filters.is_empty()
                || (paint.transform.is_some() && unsafe { &*object }.IsBox());
            if establishes_fixed_container {
                unsafe { &mut *object }.SetCanContainFixedPositionObjects(true);
                unsafe { &mut *object }.SetCanContainAbsolutePositionObjects(true);
            }
        }
        unsafe { &mut *metadata_node }.SetLayoutObject(object);
        if previous.is_none() {
            self.metadata.push(Persistent::from_ptr(metadata_node));
            self.objects.push(Persistent::from_ptr(object));
        }
        self.managed.insert(
            key,
            ManagedObject {
                object,
                node: metadata_node,
                parent,
                input: descriptor,
                effective_style,
            },
        );
        self.Place(parent, object, metadata_node);
        if !parent.is_null() {
            let layout_inline = DynamicTo::<LayoutInline>(object);
            if !layout_inline.is_null()
                && RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
                && unsafe { &*layout_inline }.IsInlineRubyText()
            {
                let view = unsafe { &*layout_inline }.View();
                unsafe { &mut *view }.SetContainsAnnotations();
            }
            if !layout_inline.is_null() && RequiresPaintBackedInlineFragment(&input) {
                unsafe { &mut *layout_inline }.SetIsInLayoutNGInlineFormattingContext(true);
                unsafe { &mut *layout_inline }.SetShouldCreateBoxFragmentDefault();
            }
        } else {
            if !self.root.Get().is_null() && self.root.Get() != object {
                panic!("A LayoutObjectTree has exactly one root");
            }
            self.root.Assign(object);
        }
        object
    }
}

impl Drop for LayoutObjectTreeStorage {
    fn drop(&mut self) {
        let root = self.root.Get();
        if !root.is_null() && !unsafe { &*root }.WasDestroyedForTreeUpdate() {
            let _object_factory_scope = LayoutObjectFactoryScope::new(&self.factories);
            let _allow_destroy = AllowDestroyingLayoutObjectInFinalizerScope::new();
            unsafe { &mut *root }.Destroy();
        }
    }
}

impl Drop for LayoutObjectTree {
    // cpp: layoutng/internal/layout_object_builder.h:24-24
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2376-2376
    fn drop(&mut self) {
        if !self.storage_.is_null() {
            let _heap_scope = LayoutHeapScope::new();
            unsafe { drop(Box::from_raw(self.storage_)) };
            self.storage_ = std::ptr::null_mut();
        }
    }
}

impl LayoutObjectTree {
    // cpp: layoutng/internal/layout_object_builder.h:22-23
    pub fn new(space: &ConstraintSpace) -> Self {
        unsafe { LayoutObjectTreeNew(space) }
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:2364-2374
    pub fn new_with_factories(space: &ConstraintSpace, factories: &LayoutObjectFactorySet) -> Self {
        InitStringStatics();
        Length::Initialize();
        Self {
            storage_: Box::into_raw(Box::new(LayoutObjectTreeStorage::new(space, factories))),
        }
    }

    /// Reconcile one projection while keeping the tree reusable after a panic.
    pub fn Update<T>(&mut self, build: impl FnOnce(&mut Self) -> T) -> T {
        self.BeginUpdate();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build(self))) {
            Ok(value) => {
                self.FinishUpdate();
                value
            }
            Err(error) => {
                let _heap = LayoutHeapScope::new();
                let storage = unsafe { &mut *self.storage_ };
                let _factories = LayoutObjectFactoryScope::new(&storage.factories);
                for object in &storage.objects {
                    let object = unsafe { &mut *object.Get() };
                    if !object.WasDestroyedForTreeUpdate() {
                        object.Destroy();
                    }
                }
                for node in &storage.metadata {
                    unsafe { &mut *node.Get() }.DetachForTreeUpdate();
                }
                storage.root = Persistent::from_ptr(std::ptr::null_mut());
                storage.objects.clear();
                storage.metadata.clear();
                storage.managed.clear();
                storage.pending.clear();
                storage.last_child.clear();
                storage.updating = false;
                std::panic::resume_unwind(error);
            }
        }
    }

    /// Give a one-shot input adapter only attachment operations. The host
    /// retains ownership and transaction control; no additional Update begins.
    pub fn WithAttachment<T>(
        &mut self,
        build: impl FnOnce(&mut crate::layout_engine::LayoutTreeUpdate<'_>) -> T,
    ) -> T {
        build(&mut crate::layout_engine::LayoutTreeUpdate::new(self))
    }

    pub fn BeginUpdate(&mut self) {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating, "layout tree update already active");
        storage.updating = true;
        // FinishUpdate and the failed-build cleanup both empty pending. Swap
        // the two maps so every complete projection can reuse their buckets;
        // mem::take would allocate managed afresh and drop pending's capacity.
        debug_assert!(storage.pending.is_empty());
        std::mem::swap(&mut storage.pending, &mut storage.managed);
        storage.occurrences.clear();
        storage.last_child.clear();
        storage.stats = LayoutTreeUpdateStats::default();
        for node in &storage.metadata {
            let node = unsafe { &mut *node.Get() };
            node.UnsealForTreeUpdate();
            NativeNodeMetadataRelations::ResetForTreeUpdate(node);
        }
    }

    pub fn FinishUpdate(&mut self) {
        let _heap_scope = LayoutHeapScope::new();
        let storage = unsafe { &mut *self.storage_ };
        assert!(storage.updating, "layout tree update is not active");
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        for old in storage.pending.values() {
            if !unsafe { &*old.object }.WasDestroyedForTreeUpdate() {
                unsafe { &mut *old.object }.Destroy();
            }
            unsafe { &mut *old.node }.DetachForTreeUpdate();
            storage.stats.removed += 1;
        }
        storage.pending.clear();
        // New handles are appended during Create. Existing handles only become
        // stale after removal/replacement, so an unchanged/reordered projection
        // does not need two temporary membership sets.
        if storage.stats.removed != 0 {
            let live_objects: std::collections::HashSet<_> =
                storage.managed.values().map(|r| r.object).collect();
            let live_nodes: std::collections::HashSet<_> =
                storage.managed.values().map(|r| r.node).collect();
            storage
                .objects
                .retain(|object| live_objects.contains(&object.Get()));
            storage
                .metadata
                .retain(|node| live_nodes.contains(&node.Get()));
        }
        storage.updating = false;
    }

    pub fn UpdateStats(&self) -> &LayoutTreeUpdateStats {
        &unsafe { &*self.storage_ }.stats
    }

    // Rust's ownership move preserves the source's move-only tree contract;
    // Clone and Copy are deliberately absent.
    // cpp: layoutng/internal/layout_object_builder.h:25-28
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2378-2381
    pub fn Move(self) -> Self {
        self
    }

    /// Dispatch native virtuals with the factories retained by this tree.
    /// Keep the scope synchronous so its thread-local pointer cannot outlive
    /// the storage which owns the actual SVG/table assembly callbacks.
    pub(crate) fn WithObjectFactoryScope<T>(&self, operation: impl FnOnce() -> T) -> T {
        let storage = unsafe { &*self.storage_ };
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        operation()
    }

    // cpp: layoutng/internal/layout_object_builder.h:30-35
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2383-2392
    pub fn CreateRoot(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        debug_name: String,
        element: Option<ElementData>,
        first_line_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        let object = unsafe { &mut *self.storage_ }.Create(
            std::ptr::null_mut(),
            id,
            NodeKind::kBox,
            style,
            element,
            debug_name,
            String::new(),
            std::ptr::null(),
            false,
            std::ptr::null(),
            first_line_style,
        );
        unsafe { &mut *object }
    }

    pub fn CreateRootDefault(&mut self, id: u64, style: &ComputedStyle) -> &mut LayoutObject {
        self.CreateRoot(id, style, String::new(), None, std::ptr::null())
    }

    // cpp: layoutng/internal/layout_object_builder.h:39-47
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2394-2408
    pub fn AddBox(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        debug_name: String,
        element: Option<ElementData>,
        kind: NodeKind,
        style_generated: bool,
        first_line_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        if kind == NodeKind::kText || kind == NodeKind::kReplaced {
            panic!("AddBox requires a box-producing NodeKind");
        }
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            kind,
            style,
            element,
            debug_name,
            String::new(),
            std::ptr::null(),
            style_generated,
            std::ptr::null(),
            first_line_style,
        );
        unsafe { &mut *object }
    }

    pub fn AddBoxDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
    ) -> &mut LayoutObject {
        self.AddBox(
            parent,
            id,
            style,
            String::new(),
            None,
            NodeKind::kBox,
            false,
            std::ptr::null(),
        )
    }

    // cpp: layoutng/internal/layout_object_builder.h:49-55
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2410-2421
    pub fn AddReplaced(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
        debug_name: String,
        kind: NodeKind,
    ) -> &mut LayoutObject {
        if kind != NodeKind::kReplaced && kind != NodeKind::kFrame {
            panic!("AddReplaced requires a replaced NodeKind");
        }
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            kind,
            style,
            element,
            debug_name,
            String::new(),
            std::ptr::null(),
            false,
            std::ptr::null(),
            std::ptr::null(),
        );
        unsafe { &mut *object }
    }

    pub fn AddReplacedDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
    ) -> &mut LayoutObject {
        self.AddReplaced(parent, id, style, None, String::new(), NodeKind::kReplaced)
    }

    // cpp: layoutng/internal/layout_object_builder.h:61-67
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2423-2433
    pub fn AddText(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
        debug_name: String,
        style: *const ComputedStyle,
        style_generated: bool,
        source_style: *const ComputedStyle,
    ) -> &mut LayoutObject {
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            NodeKind::kText,
            &ComputedStyle::default(),
            None,
            debug_name,
            text,
            style,
            style_generated,
            source_style,
            std::ptr::null(),
        );
        unsafe { &mut *object }
    }

    pub fn AddTextDefault(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        text: String,
    ) -> &mut LayoutObject {
        self.AddText(
            parent,
            id,
            text,
            String::new(),
            std::ptr::null(),
            false,
            std::ptr::null(),
        )
    }

    // cpp: layoutng/internal/layout_object_builder.h:69-71
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2435-2451
    pub fn NeedsCollapsibleWhitespace(&self, parent: &LayoutObject) -> bool {
        let storage = unsafe { &*self.storage_ };
        let mut previous = if storage.updating {
            storage
                .last_child
                .get(&(parent as *const _ as *mut _))
                .map_or(std::ptr::null_mut(), |node| {
                    unsafe { &**node }.GetLayoutObject()
                })
        } else {
            parent.SlowLastChild()
        };
        while !previous.is_null() {
            let sibling = unsafe { &*previous };
            if sibling.IsFloatingOrOutOfFlowPositioned() {
                previous = sibling.PreviousSibling();
                continue;
            }
            if sibling.IsText() {
                let text = unsafe { &*DynamicTo::<LayoutText>(previous) }.TransformedText();
                if text.empty() {
                    return false;
                }
                let last = text.Span16().expect("nonempty text")[(text.length() - 1) as usize];
                return last != b' ' as u16
                    && last != b'\t' as u16
                    && last != b'\r' as u16
                    && last != b'\n' as u16
                    && last != b'\x0c' as u16;
            }
            return sibling.IsInline() && !sibling.IsBR();
        }
        parent.IsLayoutInline()
    }

    /// Attach the shared editing state to the resident layout tree.
    pub fn SetEditingState(
        &mut self,
        state: std::rc::Rc<crate::editing_state::LayoutEditingState>,
    ) {
        unsafe { &mut *self.storage_ }.editing = state;
    }
    pub fn EditingState(&self) -> std::rc::Rc<crate::editing_state::LayoutEditingState> {
        unsafe { &*self.storage_ }.editing.clone()
    }
    /// Invalidate the affected control and its container chain, retaining siblings.
    pub fn InvalidateEditing(&mut self, id: u64) -> bool {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let mut found = false;
        for record in storage.managed.values_mut().filter(|r| r.input.id == id) {
            unsafe { &mut *record.object }.SetNeedsLayout(std::ptr::addr_of!(
                super::layout_invalidation_reason::kTextControlChanged
            ));
            found = true;
        }
        found
    }

    /// Synchronize ordinary scroll paint metadata while retaining geometry,
    /// node identity and the owning tree's source descriptor.
    pub fn UpdateScrollOffset(&mut self, id: u64, offset: super::layout_input::Offset) -> bool {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let mut found = false;
        for record in storage.managed.values_mut().filter(|r| r.input.id == id) {
            if let Some(element) = record.input.element.as_mut() {
                element.scroll_offset = offset;
                unsafe { &mut *record.node }.UpdateInputScrollOffset(offset);
                found = true;
            }
        }
        found
    }

    /// Update the source style for one real element after style resolution has
    /// proved that geometry and tree attachment are unchanged. This is the
    /// targeted counterpart of the paint-only branch in Create(): the native
    /// layout style/result remain resident, while fragment export reads the new
    /// paint data from the element metadata.
    pub fn UpdatePaintStyle(&mut self, id: u64, style: &ComputedStyle) -> bool {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let eligible = storage.managed.values().filter(|record| {
            record.input.id == id
                && !record.input.style_generated
                && record.input.kind != NodeKind::kText
        });
        let mut found = false;
        for record in eligible {
            found = true;
            if !record.input.style.LayoutEquivalent(style)
                || !record.effective_style.LayoutEquivalent(style)
            {
                return false;
            }
        }
        if !found {
            return false;
        }
        for record in storage.managed.values_mut().filter(|record| {
            record.input.id == id
                && !record.input.style_generated
                && record.input.kind != NodeKind::kText
        }) {
            record.input.style = style.clone();
            record.effective_style = style.clone();
            unsafe { &mut *record.node.cast::<super::layout_node_metadata::Element>() }
                .UpdateElementInput(&record.input);
        }
        true
    }

    /// Exclusive access to the tree-owned root for the layout entry point.
    pub fn RootMut(&mut self) -> &mut LayoutObject {
        let root = unsafe { &mut *self.storage_ }.root.Get();
        assert!(!root.is_null(), "LayoutObjectTree has no root");
        unsafe { &mut *root }
    }

    // cpp: layoutng/internal/layout_object_builder.h:73-73
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2453-2457
    pub fn Root(&self) -> &LayoutObject {
        let root = unsafe { &*self.storage_ }.root.Get();
        if root.is_null() {
            panic!("LayoutObjectTree has no root");
        }
        unsafe { &*root }
    }
}

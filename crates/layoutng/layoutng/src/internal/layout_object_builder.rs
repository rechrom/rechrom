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
    DynamicTo, InitStringStatics, LayoutHeapScope, Length, MakeGarbageCollected, Member,
    Persistent, RuntimeEnabledFeatures, String as BlinkString,
};
use layoutng_style::style::computed_style::{
    ComputedStyle as NativeComputedStyle, ComputedStyleBuilder,
};
use std::collections::HashMap;

// Native theme state and scroll translations affect paint, not box geometry.
// Keep their new values in live metadata, excluding them only from comparison.
fn GeometryMetadataEqual(new: &Option<ElementData>, old: &Option<ElementData>) -> bool {
    if new == old {
        return true;
    }
    match (new, old) {
        (Some(new), Some(old)) => {
            let mut normalized = new.clone();
            normalized.control_hovered = old.control_hovered;
            normalized.control_active = old.control_active;
            normalized.control_focused = old.control_focused;
            normalized.scroll_offset = old.scroll_offset;
            normalized == *old
        }
        _ => false,
    }
}

// Native form-control interaction state changes theme painting without changing
// box geometry. LayoutObject::SetStyle normally turns the corresponding visual
// diff into a full paint invalidation; the paint-only metadata path must make
// the same DisplayItemClient transition before replacing its retained input.
fn ControlPaintStateChanged(new: &Option<ElementData>, old: &Option<ElementData>) -> bool {
    match (new, old) {
        (Some(new), Some(old)) => {
            new.control_hovered != old.control_hovered
                || new.control_active != old.control_active
                || new.control_focused != old.control_focused
        }
        _ => false,
    }
}

// Compare attachment metadata without constructing another owned style. Native
// ComputedStyle differences are checked independently by the caller, as in
// LayoutObject::SetStyle. Keep this destructuring exhaustive when inputs grow.
fn NonStyleInputsEqual(new: &NativeNodeConstructionData, old: &NativeNodeConstructionData) -> bool {
    let NativeNodeConstructionData {
        id,
        kind,
        style_generated,
        style: _,
        text,
        element,
        debug_name,
        first_line_style,
    } = new;
    *id == old.id
        && *kind == old.kind
        && *style_generated == old.style_generated
        && *text == old.text
        && *debug_name == old.debug_name
        && *first_line_style == old.first_line_style
        && GeometryMetadataEqual(element, &old.element)
}

// Chromium's ComputedStyle already owns the Font selected for its document
// environment. Rechrom deliberately keeps that platform binding at the
// Style -> Layout boundary, so a paint-only replacement of the immutable
// style payload must retain the previous binding. Otherwise an incremental
// paint update installs StyleEngine's unresolved canonical style and then
// (correctly) skips font/layout work, leaving existing text with a null
// PrimaryFont and a zero paint baseline.
fn PreservePaintOnlyFontBinding(
    style: *const NativeComputedStyle,
    previous: &NativeComputedStyle,
) -> *const NativeComputedStyle {
    let font = previous.GetFont();
    let container_font = previous.ContainerFont();
    if font.is_null() && container_font.is_null() {
        return style;
    }
    let mut builder = ComputedStyleBuilder::from_style(unsafe { &*style });
    if !font.is_null() {
        builder.SetFont(Member::from_ptr(font));
    }
    if !container_font.is_null() {
        builder.SetContainerFont(Member::from_ptr(container_font));
    }
    builder.TakeStyle()
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
    // These descriptors survive DOM style replacement and are read during the
    // next incremental comparison. LayoutObject::StyleRef can be a distinct
    // font-bound copy; tracing it does not retain the canonical source styles.
    input_style_roots: Vec<Persistent<NativeComputedStyle>>,
}

fn RetainInputStyles(
    input: &NativeNodeConstructionData,
    effective: &ComputedStyle,
) -> Vec<Persistent<NativeComputedStyle>> {
    let mut roots: Vec<Persistent<NativeComputedStyle>> = Vec::new();
    for style in [
        input.style.native_style,
        effective.native_style,
        input
            .first_line_style
            .as_ref()
            .map_or(std::ptr::null(), |s| s.native_style),
    ] {
        if !style.is_null() && !roots.iter().any(|root| root.Get().cast_const() == style) {
            roots.push(Persistent::from_ptr(style.cast_mut()));
        }
    }
    roots
}

/// Non-CSS data attached by DOM/resource loading and needed when a native
/// `ComputedStyle` is frozen for Paint.  This stays separate from Blink's
/// style object, matching the resource/client relationship in Chromium.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PaintStyleBinding {
    pub image_resource_id: Option<u64>,
    pub svg_clip_resources: super::paint_input::SvgClipResourceBinding,
}

pub struct PaintInputUpdate {
    pub sources: Vec<u64>,
    pub inline_ink_changes: Vec<u64>,
}

// cpp: layoutng/internal/layout_object_builder.h:75-77
// cpp: layoutng/internal/boundary/layout_boundary.cc:2181-2191
// cpp: layoutng/internal/boundary/layout_boundary.cc:2355-2362
// Input identities and parent attachments are stable between attachment
// transactions. This is the boundary equivalent of Node::GetLayoutObject;
// local updates must not search every resident object for one DOM identity.
#[derive(Default)]
struct ManagedInputIndex {
    by_id: HashMap<u64, Vec<InputKey>>,
    children: HashMap<*mut LayoutObject, Vec<InputKey>>,
}
impl ManagedInputIndex {
    fn Remove(&mut self, key: &InputKey, record: &ManagedObject) {
        if let Some(keys) = self.by_id.get_mut(&record.input.id) {
            keys.retain(|k| k != key);
            if keys.is_empty() {
                self.by_id.remove(&record.input.id);
            }
        }
        if let Some(keys) = self.children.get_mut(&record.parent) {
            keys.retain(|k| k != key);
            if keys.is_empty() {
                self.children.remove(&record.parent);
            }
        }
    }
    fn Insert(&mut self, key: InputKey, record: &ManagedObject) {
        self.by_id
            .entry(record.input.id)
            .or_default()
            .push(key.clone());
        self.children.entry(record.parent).or_default().push(key);
    }
}

pub struct LayoutObjectTreeStorage {
    fonts: Box<dyn NativeFontResolver>,
    factories: LayoutObjectFactorySet,
    // Canonical allocation roots, indexed for removal by the exact retired
    // object. Subtree updates must not census every live allocation to unpin one.
    metadata: HashMap<*mut Node, Persistent<Node>>,
    objects: HashMap<*mut LayoutObject, Persistent<LayoutObject>>,
    retired: Vec<(*mut LayoutObject, *mut Node)>,
    root: Persistent<LayoutObject>,
    // The native objects and their input owners have stable storage. Hash
    // bucket updates move handles, never the large style/metadata descriptors.
    managed: HashMap<InputKey, Box<ManagedObject>>,
    input_index: std::cell::OnceCell<ManagedInputIndex>,
    partial_index: Option<ManagedInputIndex>,
    attached_keys: Vec<InputKey>,
    pending: HashMap<InputKey, Box<ManagedObject>>,
    occurrences: HashMap<(u64, i32, bool, String), usize>,
    last_child: HashMap<*mut LayoutObject, *mut Node>,
    updating: bool,
    reattach: std::collections::HashSet<u64>,
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
            metadata: HashMap::new(),
            objects: HashMap::new(),
            retired: Vec::new(),
            root: Persistent::from_ptr(std::ptr::null_mut()),
            managed: HashMap::new(),
            input_index: std::cell::OnceCell::new(),
            partial_index: None,
            attached_keys: Vec::new(),
            pending: HashMap::new(),
            occurrences: HashMap::new(),
            last_child: HashMap::new(),
            updating: false,
            reattach: Default::default(),
            stats: LayoutTreeUpdateStats::default(),
            editing: std::rc::Rc::new(Default::default()),
        }
    }

    fn InputIndex(&self) -> &ManagedInputIndex {
        self.input_index.get_or_init(|| {
            let mut index = ManagedInputIndex::default();
            for (key, record) in &self.managed {
                index
                    .by_id
                    .entry(record.input.id)
                    .or_default()
                    .push(key.clone());
                index
                    .children
                    .entry(record.parent)
                    .or_default()
                    .push(key.clone());
            }
            index
        })
    }
    fn InputKeys(&self, id: u64) -> &[InputKey] {
        self.InputIndex()
            .by_id
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    fn ChildKeys(&self, parent: *mut LayoutObject) -> &[InputKey] {
        self.InputIndex()
            .children
            .get(&parent)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn ObjectKindCompatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NativeNodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        old.parent == parent
            && old.input.kind == input.kind
            && old.effective_style.display == effective.display
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

    fn Compatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NativeNodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        Self::ObjectKindCompatible(old, parent, input, effective)
            && old.effective_style.position == effective.position
            && old.effective_style.floating == effective.floating
    }

    // cpp: ComputedStyle::NeedsReinsertLayoutTree; Element::RecalcOwnStyle.
    // Position changes do not change the LayoutObject class. Blink updates the
    // resident object through SetStyle, reinserting it only when it crosses the
    // in-flow/out-of-flow boundary (or changes float participation in-flow).
    // The local NodeInput path cannot reinsert, so admit exactly the updates
    // which Blink applies to the existing attachment.
    fn StyleUpdateCompatible(
        old: &ManagedObject,
        parent: *mut LayoutObject,
        input: &NativeNodeConstructionData,
        effective: &ComputedStyle,
    ) -> bool {
        if !Self::ObjectKindCompatible(old, parent, input, effective) {
            return false;
        }
        let old_native = unsafe { old.effective_style.native_style.as_ref() };
        let new_native = unsafe { effective.native_style.as_ref() };
        match (old_native, new_native) {
            (Some(old_native), Some(new_native)) => {
                !NativeComputedStyle::NeedsReinsertLayoutTree(old_native, new_native)
            }
            _ => {
                old.effective_style.position == effective.position
                    && old.effective_style.floating == effective.floating
            }
        }
    }

    fn Place(&mut self, parent: *mut LayoutObject, object: *mut LayoutObject, node: *mut Node) {
        if parent.is_null() {
            assert!(
                self.root.Get().is_null() || self.root.Get() == object,
                "A LayoutObjectTree has exactly one root"
            );
            self.root.Assign(object);
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
            // Reused objects also dispatch RemoveChild/AddChild when reordered.
            // Their fast path deliberately skips construction scopes; install
            // this tree's virtual factories for the attachment operation itself.
            let _factory_scope = LayoutObjectFactoryScope::new(&self.factories);
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
        existing_key: Option<InputKey>,
    ) -> *mut LayoutObject {
        let _heap_scope = LayoutHeapScope::new();
        if existing_key.is_none() {
            self.input_index.take();
        }
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
        let targeted_style_update = existing_key.is_some();
        let place = existing_key.is_none();
        let key = existing_key.unwrap_or_else(|| {
            let base = (id, kind as i32, style_generated, debug_name);
            let occurrence = self.occurrences.entry(base.clone()).or_default();
            let key = InputKey(base.0, base.1, base.2, base.3, *occurrence);
            *occurrence += 1;
            key
        });
        let mut previous = self.pending.remove(&key);
        let reattach = self.reattach.contains(&id);
        // A complete projection still visits and compares every node. Compare
        // borrowed inputs before making owned descriptors: unchanged nodes need
        // neither style/string clones nor construction resolver scopes.
        if let Some(old) = previous.as_ref() {
            if !reattach
                && !unsafe { &*old.object }.WasDestroyedForTreeUpdate()
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
                if place {
                    self.Place(parent, object, node);
                }
                self.stats.reused += 1;
                if self.partial_index.is_some() {
                    self.attached_keys.push(key.clone());
                }
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
            let compatible = if targeted_style_update {
                Self::StyleUpdateCompatible(old, parent, &descriptor, &effective_style)
            } else {
                Self::Compatible(old, parent, &descriptor, &effective_style)
            };
            if reattach || unsafe { &*old.object }.WasDestroyedForTreeUpdate() || !compatible {
                if !unsafe { &*old.object }.WasDestroyedForTreeUpdate() {
                    unsafe { &mut *old.object }.Destroy();
                    unsafe { &mut *old.node }.DetachForTreeUpdate();
                }
                if old.object == self.root.Get() {
                    self.root = Persistent::from_ptr(std::ptr::null_mut());
                }
                self.stats.removed += 1;
                // Keep the old allocation rooted until the transaction's
                // destruction phase has completed, including unwind cleanup.
                self.retired.push((old.object, old.node));
                previous = None;
            }
        }
        let same_non_style_inputs = previous
            .as_ref()
            .is_some_and(|old| NonStyleInputsEqual(&descriptor, &old.input));
        let paint_only = previous.as_ref().is_some_and(|old| {
            same_non_style_inputs
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
        // A paint-only style change retains geometry, not the old style object.
        // Blink's LayoutObject::SetStyle installs the new ComputedStyle even
        // when StyleDifference does not require layout; Paint then observes
        // updated StyleImage/effect/color data through StyleRef().  Keeping the
        // previous pointer here made the Rust projection current while the
        // resident LayoutObject and fragment export stayed one revision behind.
        // PrepareNativeStyle returns StyleEngine's canonical native pointer
        // when one is present, so this does not translate the projection back
        // into a second style.
        let native_style = if paint_only {
            if kind == NodeKind::kText {
                input.style = if !explicit_text_style.is_null() {
                    unsafe { &*explicit_text_style }.clone()
                } else {
                    let parent_node = unsafe { &*parent }.GetNode();
                    unsafe { &*parent_node }.InputStyle().clone()
                };
            }
            prepare_style(
                &input,
                if parent.is_null() {
                    std::ptr::null()
                } else {
                    unsafe { &*parent }.StyleRef()
                },
                parent_kind,
            )
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
        let native_style = if paint_only {
            let old = previous
                .as_ref()
                .expect("paint-only update has a previous object");
            PreservePaintOnlyFontBinding(native_style, unsafe { &*old.object }.StyleRef())
        } else {
            native_style
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
            native_first_line_style =
                prepare_style(&first_line_input, parent_first_line_style, parent_kind);
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
            if !same_non_style_inputs
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
            let establishes_fixed_container = (!paint.filters.is_empty()
                || !paint.backdrop_filters.is_empty()
                || paint.backdrop_filter_has_reference)
                || (paint.transform.is_some() && unsafe { &*object }.IsBox());
            if establishes_fixed_container {
                unsafe { &mut *object }.SetCanContainFixedPositionObjects(true);
                unsafe { &mut *object }.SetCanContainAbsolutePositionObjects(true);
            }
        }
        unsafe { &mut *metadata_node }.SetLayoutObject(object);
        if previous.is_none() {
            self.metadata
                .insert(metadata_node, Persistent::from_ptr(metadata_node));
            self.objects.insert(object, Persistent::from_ptr(object));
        }
        let record = ManagedObject {
            object,
            node: metadata_node,
            parent,
            input_style_roots: RetainInputStyles(&descriptor, &effective_style),
            input: descriptor,
            effective_style,
        };
        let record = if let Some(mut previous) = previous {
            *previous = record;
            previous
        } else {
            Box::new(record)
        };
        if self.partial_index.is_some() {
            self.attached_keys.push(key.clone());
        }
        self.managed.insert(key, record);
        if place {
            self.Place(parent, object, metadata_node);
        }
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
    pub(crate) fn OutputNode(&self, id: u64) -> Option<&LayoutObject> {
        let storage = unsafe { &*self.storage_ };
        storage
            .InputKeys(id)
            .iter()
            .filter_map(|key| storage.managed.get(key))
            .filter(|record| !record.input.style_generated)
            .min_by_key(|record| record.input.kind as i32)
            .map(|record| unsafe { &*record.object })
    }

    pub(crate) fn OutputNodeMut(&mut self, id: u64) -> Option<&mut LayoutObject> {
        let storage = unsafe { &mut *self.storage_ };
        let object = storage
            .InputKeys(id)
            .iter()
            .filter_map(|key| storage.managed.get(key))
            .filter(|record| !record.input.style_generated)
            .min_by_key(|record| record.input.kind as i32)?
            .object;
        Some(unsafe { &mut *object })
    }
    pub(crate) fn OutputScrollOffsets(&self) -> Vec<(u64, super::layout_input::Offset)> {
        let storage = unsafe { &*self.storage_ };
        storage
            .managed
            .values()
            .filter_map(|record| {
                if record.input.style_generated {
                    return None;
                }
                let element =
                    foundation::DynamicTo::<super::layout_node_metadata::Element>(record.node);
                let data = unsafe { element.as_ref() }?.InputElementData().as_ref()?;
                Some((record.input.id, data.scroll_offset))
            })
            .collect()
    }
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
        self.CompleteUpdate(build)
    }

    // Element::RebuildLayoutTree keeps clean branches outside the attachment
    // transaction. Validate the existing parent/anchor before changing anything;
    // anonymous or flattened attachment contexts need a wider dirty root.
    pub fn UpdateSubtree(
        &mut self,
        id: u64,
        parent_id: u64,
        build: impl FnOnce(&mut Self, &mut LayoutObject),
    ) -> bool {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let keys = storage.InputKeys(id).to_vec();
        if keys.len() != 1 {
            return false;
        }
        let record = &storage.managed[&keys[0]];
        let parent = record.parent;
        if parent.is_null()
            || record.input.style_generated
            || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
            || unsafe { &*record.object }.Parent() != parent
            || unsafe { &*parent }.IsAnonymous()
            || unsafe { &*(*parent).GetNode() }.InputId() != parent_id
        {
            return false;
        }
        let previous = unsafe { &*record.node }.previousSibling();
        let mut collect_trace = browser_tracing::span("layout", "LayoutTree.CollectSubtree");
        let mut affected = keys;
        let mut position = 0;
        while position < affected.len() {
            let object = storage.managed[&affected[position]].object;
            affected.extend_from_slice(storage.ChildKeys(object));
            position += 1;
        }
        collect_trace.set("objects", affected.len() as f64);
        drop(collect_trace);
        let mut index = storage.input_index.take().unwrap();
        assert!(storage.pending.is_empty());
        storage.updating = true;
        storage.occurrences.clear();
        storage.last_child.clear();
        storage.last_child.insert(parent, previous);
        unsafe { &mut *(*parent).GetNode() }.UnsealForTreeUpdate();
        storage.attached_keys.clear();
        storage.stats = LayoutTreeUpdateStats::default();
        let prepare_trace = browser_tracing::span("layout", "LayoutTree.PrepareSubtree");
        for key in affected {
            let record = storage.managed.remove(&key).unwrap();
            index.Remove(&key, &record);
            let node = unsafe { &mut *record.node };
            node.UnsealForTreeUpdate();
            NativeNodeMetadataRelations::ResetForTreeUpdate(node);
            storage.pending.insert(key, record);
        }
        drop(prepare_trace);
        storage.partial_index = Some(index);
        self.CompleteUpdate(|tree| build(tree, unsafe { &mut *parent }));
        true
    }

    fn CompleteUpdate<T>(&mut self, build: impl FnOnce(&mut Self) -> T) -> T {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build(self))) {
            Ok(value) => {
                self.FinishUpdate();
                value
            }
            Err(error) => {
                let _heap = LayoutHeapScope::new();
                let storage = unsafe { &mut *self.storage_ };
                let _factories = LayoutObjectFactoryScope::new(&storage.factories);
                for object in storage.objects.values() {
                    let object = unsafe { &mut *object.Get() };
                    if !object.WasDestroyedForTreeUpdate() {
                        object.Destroy();
                    }
                }
                for node in storage.metadata.values() {
                    unsafe { &mut *node.Get() }.DetachForTreeUpdate();
                }
                storage.root = Persistent::from_ptr(std::ptr::null_mut());
                storage.objects.clear();
                storage.metadata.clear();
                storage.retired.clear();
                storage.managed.clear();
                storage.pending.clear();
                storage.partial_index = None;
                storage.attached_keys.clear();
                storage.input_index.take();
                storage.reattach.clear();
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
        debug_assert!(storage.retired.is_empty());
        storage.partial_index = None;
        storage.attached_keys.clear();
        storage.input_index.take();
        // FinishUpdate and the failed-build cleanup both empty pending. Swap
        // the two maps so every complete projection can reuse their buckets;
        // mem::take would allocate managed afresh and drop pending's capacity.
        debug_assert!(storage.pending.is_empty());
        std::mem::swap(&mut storage.pending, &mut storage.managed);
        // The previous projection's root remains owned by pending/objects.
        // Select exactly one root for this projection, which can replace an
        // empty document view when its document element is attached later.
        storage.root = Persistent::from_ptr(std::ptr::null_mut());
        storage.occurrences.clear();
        storage.last_child.clear();
        storage.stats = LayoutTreeUpdateStats::default();
        for node in storage.metadata.values() {
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
        let mut retire_trace = browser_tracing::span("layout", "LayoutTree.RetireObjects");
        retire_trace.set("objects", storage.pending.len() as f64);
        for old in storage.pending.values() {
            if !unsafe { &*old.object }.WasDestroyedForTreeUpdate() {
                unsafe { &mut *old.object }.Destroy();
            }
            unsafe { &mut *old.node }.DetachForTreeUpdate();
            storage.stats.removed += 1;
            storage.retired.push((old.object, old.node));
        }
        storage.pending.clear();
        // Destroy every removed object before releasing any of their roots,
        // preserving the original transaction lifetime and parent dependencies.
        // Retire only changed allocations; no whole-tree membership snapshots.
        for (object, node) in storage.retired.drain(..) {
            storage.objects.remove(&object);
            storage.metadata.remove(&node);
        }
        drop(retire_trace);
        if let Some(mut index) = storage.partial_index.take() {
            for key in storage.attached_keys.drain(..) {
                index.Insert(key.clone(), &storage.managed[&key]);
            }
            assert!(storage.input_index.set(index).is_ok());
        }
        storage.reattach.clear();
        storage.updating = false;
    }

    pub(crate) fn Reattach(&mut self, node_ids: &[u64]) {
        let storage = unsafe { &mut *self.storage_ };
        assert!(storage.updating, "reattachment belongs to the tree update");
        storage.reattach.extend(node_ids.iter().copied());
    }

    pub fn UpdateStats(&self) -> &LayoutTreeUpdateStats {
        &unsafe { &*self.storage_ }.stats
    }

    pub(crate) fn HasRoot(&self) -> bool {
        let root = unsafe { &*self.storage_ }.root.Get();
        !root.is_null() && !unsafe { &*root }.WasDestroyedForTreeUpdate()
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
            None,
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
            None,
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
            None,
        );
        unsafe { &mut *object }
    }

    /// Anonymous replaced child created by generated ContentData. Chromium's
    /// ImageContentData::CreateLayoutObject creates a LayoutImage and applies
    /// the originating pseudo style; keep that identity distinct from a real
    /// DOM replaced element while using the same translated replaced factory.
    pub fn AddGeneratedReplaced(
        &mut self,
        parent: &mut LayoutObject,
        id: u64,
        style: &ComputedStyle,
        element: ElementData,
        debug_name: String,
    ) -> &mut LayoutObject {
        let object = unsafe { &mut *self.storage_ }.Create(
            parent,
            id,
            NodeKind::kReplaced,
            style,
            Some(element),
            debug_name,
            String::new(),
            std::ptr::null(),
            true,
            std::ptr::null(),
            std::ptr::null(),
            None,
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
            None,
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
        for key in storage.InputKeys(id).to_vec() {
            let record = storage.managed.get_mut(&key).unwrap();
            unsafe { &mut *record.object }.SetNeedsLayout(std::ptr::addr_of!(
                super::layout_invalidation_reason::kTextControlChanged
            ));
            found = true;
        }
        found
    }

    /// Invalidate selection drawing without invalidating text-control geometry.
    ///
    /// Chromium's LayoutSelection marks every LayoutObject whose selection
    /// state or boundary changed, and ObjectPaintInvalidator converts that bit
    /// to kSelection invalidation. Rechrom records a control's selection with
    /// the text object's ordinary display-item client, so invalidate the
    /// resident selection leaves below the control input before the paint-only
    /// export. Text controls use generated editor/text inputs with derived ids,
    /// so looking up only the host id would miss the client that records glyphs.
    ///
    /// Source: third_party/blink/renderer/core/editing/layout_selection.cc:
    ///         217-334, 954-977;
    ///         core/paint/object_paint_invalidator.cc:96-116.
    pub fn InvalidateSelectionPaint(&mut self, id: u64) -> bool {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let roots = storage
            .InputKeys(id)
            .iter()
            .map(|key| storage.managed[key].object)
            .collect::<Vec<_>>();
        let mut stack = roots.clone();
        let mut visited = std::collections::HashSet::new();
        let mut leaves = Vec::new();
        while let Some(object) = stack.pop() {
            if !visited.insert(object) {
                continue;
            }
            // Text-control selections are painted by their generated
            // LayoutText descendants. The translated generic
            // CanBeSelectionLeaf provider is not part of this assembly, and
            // invalidating unrelated generated boxes would discard more
            // recordings than Chromium's selection client invalidation.
            if unsafe { &*object }.IsText() {
                leaves.push(object);
            }
            for key in storage.ChildKeys(object).to_vec() {
                stack.push(storage.managed[&key].object);
            }
        }
        for object in leaves {
            unsafe { &mut *object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kSelection,
            );
        }
        !roots.is_empty()
    }

    /// Synchronize ordinary scroll paint metadata while retaining geometry,
    /// node identity and the owning tree's source descriptor.
    pub fn UpdateScrollOffset(&mut self, id: u64, offset: super::layout_input::Offset) -> bool {
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let mut found = false;
        for key in storage.InputKeys(id).to_vec() {
            let record = storage.managed.get_mut(&key).unwrap();
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
    pub fn UpdatePaintStyle(
        &mut self,
        id: u64,
        style: &NativeComputedStyle,
        clip_resource_version: super::paint_input::SvgClipResourceVersion,
    ) -> Option<PaintStyleBinding> {
        let storage = unsafe { &mut *self.storage_ };
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        assert!(!storage.updating);
        // Validate before mutating any native pointers. A resource-only change
        // must use the ordinary tree/export lifecycle to refresh every client.
        if storage
            .InputKeys(id)
            .iter()
            .map(|key| &storage.managed[key])
            .any(|record| {
                record.input.style.svg_clip_resources.snapshot.version != clip_resource_version
            })
        {
            return None;
        }
        let native_style = style as *const NativeComputedStyle;
        let mut binding = None;
        for key in storage.InputKeys(id).to_vec() {
            let record = storage.managed.get_mut(&key).unwrap();
            if record.input.style_generated || record.input.kind == NodeKind::kText {
                continue;
            }
            let current = PaintStyleBinding {
                image_resource_id: record.input.style.paint.image_resource_id,
                svg_clip_resources: record.input.style.svg_clip_resources.clone(),
            };
            if let Some(previous) = binding.as_ref() {
                debug_assert_eq!(previous, &current);
            } else {
                binding = Some(current);
            }
            record.input.style.native_style = native_style;
            record.effective_style.native_style = native_style;
            record.input_style_roots = RetainInputStyles(&record.input, &record.effective_style);
            let node = unsafe { &mut *record.node };
            node.SetComputedStyle(native_style);
            unsafe { &mut *record.object }.SetStyle(native_style);
        }
        binding
    }

    /// Element::RecalcOwnStyle updates an attached LayoutObject through
    /// SetStyle; ordinary geometry changes do not reattach every sibling.
    /// Reuse Create's style/metadata implementation after proving the native
    /// object kind and attachment are unchanged. Complex generated sources
    /// retain the ordinary attachment transaction.
    pub fn UpdateNodeInput(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
    ) -> bool {
        let _heap = LayoutHeapScope::new();
        let storage = unsafe { &mut *self.storage_ };
        assert!(!storage.updating);
        let keys = storage.InputKeys(id).to_vec();
        if keys.is_empty() {
            return false;
        }
        let mut inputs = Vec::new();
        for key in &keys {
            let record = &storage.managed[key];
            if record.input.style_generated
                || record.input.first_line_style.is_some()
                || unsafe { &*record.object }.IsSVG()
                || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
                || unsafe { &*record.node }.InputHasFirstLineStyles()
                || record
                    .input
                    .element
                    .as_ref()
                    .is_some_and(|e| e.form_control_type.is_some())
            {
                return false;
            }
            let mut input = record.input.clone();
            input.style = style.clone();
            // Resource bindings remain with their owner; this path changes no
            // resource catalog. DOM's resource mutations require tree input.
            input.style.paint.svg_filter_graphs =
                record.effective_style.paint.svg_filter_graphs.clone();
            input.element = if input.kind == NodeKind::kText {
                None
            } else {
                element.clone()
            };
            if !LayoutObjectTreeStorage::StyleUpdateCompatible(
                record,
                record.parent,
                &input,
                &input.style,
            ) {
                return false;
            }
            inputs.push((key.clone(), record.parent, input));
        }
        // Propagate the canonical parent style to existing inherited text,
        // matching Create's effective text style and Blink's anonymous children.
        let parents = keys
            .iter()
            .map(|key| {
                let record = &storage.managed[key];
                (record.object, record.effective_style.native_style)
            })
            .collect::<Vec<_>>();
        for (parent, native) in parents {
            for key in storage.ChildKeys(parent) {
                let record = &storage.managed[key];
                if record.input.kind == NodeKind::kText
                    && !keys.contains(key)
                    && !native.is_null()
                    && record.effective_style.native_style == native
                {
                    if record.input.style_generated || record.input.first_line_style.is_some() {
                        return false;
                    }
                    let mut input = record.input.clone();
                    input.style = style.clone();
                    inputs.push((key.clone(), record.parent, input));
                }
            }
        }
        // Input comparisons and validation finish before any mutation. No
        // sibling topology or partial document transaction is exposed here.
        for (key, parent, input) in inputs {
            let record = storage.managed.remove(&key).unwrap();
            unsafe { &mut *record.node }.UnsealForTreeUpdate();
            storage.pending.insert(key.clone(), record);
            let text_style = if input.kind == NodeKind::kText {
                &input.style as *const _
            } else {
                std::ptr::null()
            };
            storage.Create(
                parent,
                input.id,
                input.kind,
                &input.style,
                input.element,
                input.debug_name,
                input.text,
                text_style,
                false,
                std::ptr::null(),
                std::ptr::null(),
                Some(key),
            );
        }
        true
    }

    /// Element::RecalcOwnStyle installs local style/theme inputs on existing
    /// LayoutObjects. Admission retains native geometry and rejects changes
    /// requiring the fragment/overflow lifecycle (ComputedStyle::VisualInvalidationDiff).
    fn PaintInputPlan(
        &self,
        id: u64,
        style: &ComputedStyle,
        element: &Option<ElementData>,
    ) -> Option<(std::collections::HashSet<u64>, Vec<u64>)> {
        use layoutng_style::style::computed_style_base::{ComputedStyleBase, FieldDifference as F};
        let mut trace = browser_tracing::span("layout", "LayoutTree.ValidatePaintInput");
        trace.set("node_id", id as f64);
        let storage = unsafe { &*self.storage_ };
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        assert!(!storage.updating);
        let records = storage
            .InputKeys(id)
            .iter()
            .map(|key| &storage.managed[key])
            .collect::<Vec<_>>();
        if records.is_empty() {
            // Element::RecalcOwnStyle updates GetLayoutObject() only when one
            // exists. Hidden, inactive flat-tree and collapsible text sources
            // can retain computed style without any attached layout object.
            // Geometry/reattachment changes are excluded by this entry's
            // paint-only lifecycle; do not manufacture an object or fail it.
            trace.set("no_layout_object", 1.0);
            return Some((Default::default(), Vec::new()));
        }
        // Create's effective style proves which native text children inherit
        // this source, including generated control labels with transport-only
        // descriptors. This is PropagateStyleToAnonymousChildren's retained
        // ownership; an independently styled pseudo/text source is excluded.
        let parents = records
            .iter()
            .map(|r| (r.object, r.effective_style.native_style))
            .collect::<Vec<_>>();
        let mut ids = std::collections::HashSet::from([id]);
        for &(parent, native) in &parents {
            for key in storage.ChildKeys(parent) {
                let child = &storage.managed[key];
                if child.input.kind == NodeKind::kText
                    && !native.is_null()
                    && child.effective_style.native_style == native
                {
                    ids.insert(child.input.id);
                }
            }
        }
        drop(records);
        let records = ids
            .iter()
            .flat_map(|&id| storage.InputKeys(id))
            .map(|key| &storage.managed[key])
            .collect::<Vec<_>>();
        let mut inline_ink_changes = Vec::new();
        for record in &records {
            let object = unsafe { &*record.object };
            // Text's source descriptor can be a generated transport style;
            // its effective style is the canonical parent consumed by Create.
            // Keep these owners distinct when validating the local update.
            let old = unsafe { record.effective_style.native_style.as_ref() };
            let new = unsafe { style.native_style.as_ref() };
            let (Some(old), Some(new)) = (old, new) else {
                trace.set("missing_canonical_style", 1.0);
                trace.set("source_kind", record.input.kind as u8 as f64);
                return None;
            };
            let fields = ComputedStyleBase::FieldInvalidationDiff(old, new);
            let inline = DynamicTo::<LayoutInline>(record.object);
            if !inline.is_null()
                && super::boundary::layout_boundary::NativeStyleRequiresPaintBackedInlineFragment(old)
                    != super::boundary::layout_boundary::NativeStyleRequiresPaintBackedInlineFragment(new)
            {
                return None;
            }
            let geometry = (F::kLayout
                | F::kReshape
                | F::kHasTransform
                | F::kTransformData
                | F::kTransformOther
                | F::kFilterData
                | F::kMask
                | F::kStroke
                | F::kBorderShape)
                .bits();
            trace.set("field_difference", fields as f64);
            trace.set("native_needs_layout", object.NeedsLayout() as u8 as f64);
            trace.set(
                "first_line",
                unsafe { &*record.node }.InputHasFirstLineStyles() as u8 as f64,
            );
            let mut input = style.clone();
            if record.input.kind == NodeKind::kText && record.input.id != id {
                input.svg_clip_resources = record.effective_style.svg_clip_resources.clone();
            }
            let layout_equivalent = record.effective_style.LayoutEquivalent(&input);
            trace.set("layout_equivalent", layout_equivalent as u8 as f64);
            // Text decorations retain shaping and layout; their existing
            // fragment items need ink invalidation/export, not tree attachment.
            let text_ink = old.DiffNeedsRecomputeVisualOverflow(new, fields)
                && fields & F::kTextDecoration.bits() != 0;
            let other_ink =
                old.DiffNeedsRecomputeVisualOverflow(new, fields & !F::kTextDecoration.bits());
            if fields & geometry != 0
                || other_ink
                || object.NeedsLayout()
                || object.IsSVG()
                || (record.input.style_generated
                    && !(record.input.kind == NodeKind::kText && record.input.id != id))
                || record.input.first_line_style.is_some()
                || unsafe { &*record.node }.InputHasFirstLineStyles()
                || !layout_equivalent
                || record.effective_style.svg_clip_resources.snapshot.version
                    != input.svg_clip_resources.snapshot.version
            {
                return None;
            }
            // Only native user-action state may change without geometry.
            let candidate = if record.input.kind == NodeKind::kText {
                &None
            } else {
                element
            };
            if !GeometryMetadataEqual(candidate, &record.input.element) {
                trace.set("metadata_requires_export", 1.0);
                return None;
            }
            if text_ink {
                inline_ink_changes.push(record.input.id);
            }
        }
        drop(records);
        Some((ids, inline_ink_changes))
    }

    /// Query geometry validation must not install paint/resource bindings.
    pub(crate) fn GeometryInputMatches(
        &self,
        id: u64,
        style: &ComputedStyle,
        element: &Option<ElementData>,
    ) -> bool {
        self.PaintInputPlan(id, style, element).is_some()
    }

    pub fn UpdatePaintInput(
        &mut self,
        id: u64,
        style: &ComputedStyle,
        element: Option<ElementData>,
    ) -> Option<PaintInputUpdate> {
        let (ids, inline_ink_changes) = self.PaintInputPlan(id, style, &element)?;
        let storage = unsafe { &mut *self.storage_ };
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        let keys = ids
            .iter()
            .flat_map(|&id| storage.InputKeys(id).iter().cloned())
            .collect::<Vec<_>>();
        for key in keys {
            let record = storage.managed.get_mut(&key).unwrap();
            let previous = unsafe { &*record.object }.StyleRef();
            let control_paint_state_changed =
                ControlPaintStateChanged(&element, &record.input.element);
            let native_style = PreservePaintOnlyFontBinding(style.native_style, previous);
            let mut projected = style.clone();
            projected.svg_clip_resources = record.effective_style.svg_clip_resources.clone();
            projected.paint.svg_filter_graphs =
                record.effective_style.paint.svg_filter_graphs.clone();
            // This path admits only style updates whose geometry remains
            // valid. Chromium's LayoutObject::SetStyle turns the corresponding
            // VisualInvalidationDiff into kStyle before PrePaint. Preserve that
            // lifecycle instead of letting the retained DisplayItemClient
            // treat changed colors, backgrounds or borders as valid content.
            // cpp: core/layout/layout_object.cc:3102-3258
            let native_fields =
                layoutng_style::style::computed_style_base::ComputedStyleBase::FieldInvalidationDiff(
                    previous,
                    unsafe { &*native_style },
                );
            if native_fields != 0
                || record.effective_style.paint != projected.paint
                || control_paint_state_changed
            {
                unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                    foundation::PaintInvalidationReason::kStyle,
                );
            }
            // A generated text-source projection supplies transport metadata,
            // while StyleRef supplies the canonical text paint. Preserve the
            // former exactly as Create's source_text_style branch does.
            if !record.input.style.native_style.is_null() {
                record.input.style = projected.clone();
            }
            record.input.element = if record.input.kind == NodeKind::kText {
                None
            } else {
                element.clone()
            };
            record.effective_style = projected;
            record.input_style_roots = RetainInputStyles(&record.input, &record.effective_style);
            let node = unsafe { &mut *record.node };
            let sealed = node.InputTreeIsSealed();
            node.UpdateInputData(&record.input);
            if sealed {
                node.seal_input_tree();
            }
            node.ClearInputFontBinding();
            node.SetComputedStyle(native_style);
            unsafe { &mut *record.object }.SetStyle(native_style);
        }
        Some(PaintInputUpdate {
            sources: ids.into_iter().collect(),
            inline_ink_changes,
        })
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

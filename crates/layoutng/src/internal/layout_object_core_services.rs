#![allow(non_snake_case)]

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::graphics_types;
use foundation::{
    gfx, DynamicTo, EDisplay, ETextTransform, IsHorizontalTypographicMode, LayoutUnit,
    PhysicalOffset, String as BlinkString, StringBuilder, To, UnsupportedLayout, ValuesEquivalent,
};
use graphics_types::graphics::dom_node_id::{kInvalidDOMNodeId, DOMNodeId};
use layoutng_geometry::geometry::axis::PhysicalAxis;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use super::form_control_types::FormControlType;
use super::form_node_metadata::HTMLFormControlElement;
use super::layout_block::LayoutBlock;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_input::NodeKind;
use super::layout_node_metadata::Element;
use super::layout_object::{
    ApplyStyleChanges, LayoutObject, RecalcScrollableOverflowResult, StyleChangeContext,
};
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::layout_pass_scope::LayoutPassScope;
use super::layout_text::LayoutText;
use super::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
use super::style_variant::StyleVariant;
use super::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};

// The concrete hierarchy must supply each virtual dispatch entry.
// cpp: layoutng/internal/layout_object_core_services.cc:107-162
unsafe extern "Rust" {
    fn DispatchLayoutObjectAddChild(
        object: *mut LayoutObject,
        new_child: *mut LayoutObject,
        before_child: *mut LayoutObject,
    );
    fn DispatchLayoutObjectRemoveChild(object: *mut LayoutObject, old_child: *mut LayoutObject);
    fn DispatchLayoutObjectInsertedIntoTree(object: *mut LayoutObject);
    fn DispatchLayoutObjectWillBeRemovedFromTree(object: *mut LayoutObject);
}

// cpp: layoutng/internal/layout_object_core_services.cc:164-203
unsafe extern "Rust" {
    fn DispatchLayoutObjectStyleWillChange(
        object: *mut LayoutObject,
        difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &mut StyleChangeContext,
    );
    fn DispatchLayoutObjectStyleDidChange(
        object: *mut LayoutObject,
        difference: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        context: &StyleChangeContext,
    );
}

// cpp: layoutng/internal/layout_object_core_services.cc:280-300
unsafe extern "Rust" {
    fn DispatchLayoutObjectRecalcScrollableOverflow(
        object: *mut LayoutObject,
    ) -> RecalcScrollableOverflowResult;
    fn DispatchLayoutObjectRecalcVisualOverflow(object: *mut LayoutObject);
}

// cpp: layoutng/internal/layout_object.h:321-321
unsafe extern "Rust" {
    fn DispatchLayoutObjectGetName(object: *const LayoutObject) -> &'static str;
}

// cpp: layoutng/internal/layout_object_core_services.cc:34-37
fn SameTransformOperation<T: PartialEq>(left: *mut T, right: *mut T) -> bool {
    ValuesEquivalent(left, right)
}

// cpp: layoutng/internal/layout_object_core_services.cc:39-103
fn TransformStyleDifference(
    old_style: &ComputedStyle,
    new_style: &ComputedStyle,
) -> StyleDifference {
    let transform_property_changed = old_style.Transform() != new_style.Transform();
    let offset_anchor_changed = old_style.OffsetAnchor() != new_style.OffsetAnchor();
    let offset_path_changed = !ValuesEquivalent(old_style.OffsetPath(), new_style.OffsetPath());
    let rotate_changed = !SameTransformOperation(old_style.Rotate(), new_style.Rotate());
    let offset_distance_changed = old_style.OffsetDistance() != new_style.OffsetDistance();
    let offset_position_changed = old_style.OffsetPosition() != new_style.OffsetPosition();
    let perspective_origin_changed = old_style.PerspectiveOrigin() != new_style.PerspectiveOrigin();
    let offset_rotate_changed = old_style.OffsetRotate() != new_style.OffsetRotate();
    let perspective_changed = old_style.Perspective() != new_style.Perspective();
    let scale_changed = !SameTransformOperation(old_style.Scale(), new_style.Scale());
    let translate_changed = !SameTransformOperation(old_style.Translate(), new_style.Translate());
    let transform_origin_changed = old_style.GetTransformOrigin() != new_style.GetTransformOrigin();
    let transform_box_changed = old_style.TransformBox() != new_style.TransformBox();
    let transform_other_changed = offset_anchor_changed
        || offset_path_changed
        || rotate_changed
        || offset_distance_changed
        || offset_position_changed
        || perspective_origin_changed
        || offset_rotate_changed
        || perspective_changed
        || scale_changed
        || translate_changed
        || transform_origin_changed
        || transform_box_changed;
    let has_transform_field_changed = transform_property_changed
        || offset_path_changed
        || rotate_changed
        || offset_position_changed
        || scale_changed
        || translate_changed
        || old_style.HasCurrentRotateAnimation() != new_style.HasCurrentRotateAnimation()
        || old_style.HasCurrentScaleAnimation() != new_style.HasCurrentScaleAnimation()
        || old_style.HasCurrentTransformAnimation() != new_style.HasCurrentTransformAnimation()
        || old_style.HasCurrentTranslateAnimation() != new_style.HasCurrentTranslateAnimation();

    let mut diff = StyleDifference::default();
    diff.set_transform_data_changed(
        transform_property_changed
            || offset_anchor_changed
            || offset_path_changed
            || rotate_changed
            || offset_distance_changed
            || offset_position_changed
            || offset_rotate_changed
            || scale_changed
            || translate_changed
            || transform_origin_changed
            || transform_box_changed,
    );
    if transform_other_changed
        || (has_transform_field_changed && old_style.HasTransform() != new_style.HasTransform())
    {
        diff.set_transform_changed(true);
    } else if transform_property_changed {
        diff.set_only_transform_property_changed(true);
        diff.set_transform_changed(true);
    }
    diff
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:581-582
    pub fn AddChildDefault(&mut self, new_child: *mut LayoutObject) {
        self.AddChild(new_child, std::ptr::null_mut());
    }

    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectAddChild(self, new_child, before_child) };
    }

    // Base virtual implementation. Derived layouts may override insertion.
    // cpp: layoutng/internal/layout_object_core_services.cc:107-148
    pub fn AddChildBase(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let children = self.VirtualChildren();
        if children.is_null() {
            eprint!(
                "layout tree parent cannot contain children: {}",
                self.GetName()
            );
            let node = self.GetNode();
            if !node.is_null() {
                eprint!(" ({})", unsafe { &*node }.InputDebugName());
            }
            eprintln!();
        }
        debug_assert!(!children.is_null());
        if children.is_null() {
            return;
        }

        let factories = LayoutObjectFactoryScope::Objects();
        let new_child_ref = unsafe { &mut *new_child };
        if new_child_ref.IsTablePart() {
            let insert_table_child = if factories.is_null() {
                None
            } else {
                unsafe { &*factories }.insert_table_child
            };
            let Some(insert_table_child) = insert_table_child else {
                std::panic::panic_any(UnsupportedLayout::new(
                    "table layout module is not installed",
                ));
            };
            insert_table_child(self, new_child_ref, before_child);
        } else if new_child_ref.IsText()
            && !IsHorizontalTypographicMode(new_child_ref.StyleRef().GetWritingMode())
        {
            let insert_inline_text_child = if factories.is_null() {
                None
            } else {
                unsafe { &*factories }.insert_inline_text_child
            };
            if !insert_inline_text_child
                .is_some_and(|insert| insert(self, new_child_ref, before_child))
            {
                if insert_inline_text_child.is_none()
                    && !new_child_ref.IsSVGInlineText()
                    && new_child_ref.StyleRef().HasTextCombine()
                {
                    std::panic::panic_any(UnsupportedLayout::new(
                        "inline layout module is not installed",
                    ));
                }
                unsafe { &mut *children }.InsertChildNodeDefault(self, new_child, before_child);
            }
        } else {
            unsafe { &mut *children }.InsertChildNodeDefault(self, new_child, before_child);
        }

        let text = DynamicTo::<LayoutText>(new_child);
        if !text.is_null()
            && (new_child_ref.StyleRef().TextTransform().bits()
                & ETextTransform::kCapitalize.bits())
                != 0
        {
            unsafe { &mut *text }.TransformAndSecureOriginalText();
        }
    }

    // Base virtual implementation. Derived layouts may override removal.
    // cpp: layoutng/internal/layout_object_core_services.cc:150-162
    pub fn RemoveChildBase(&mut self, old_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let children = self.VirtualChildren();
        debug_assert!(!children.is_null());
        if children.is_null() {
            return;
        }
        let previous = DynamicTo::<LayoutBoxModelObject>(unsafe { &*old_child }.PreviousSibling());
        let next = DynamicTo::<LayoutBoxModelObject>(unsafe { &*old_child }.NextSibling());
        unsafe { &mut *children }.RemoveChildNodeDefault(self, old_child);
        LayoutBoxModelObject::AttemptToMerge(previous, next);
    }

    pub fn RemoveChild(&mut self, old_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectRemoveChild(self, old_child) };
    }

    // The C++ default argument is a wrapper in Rust.
    // cpp: layoutng/internal/layout_object.h:2203-2205
    pub fn SetStyle(&mut self, style: *const ComputedStyle) {
        self.SetStyleWithChanges(style, ApplyStyleChanges::kYes);
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:164-182
    pub fn SetStyleWithChanges(
        &mut self,
        style: *const ComputedStyle,
        apply_changes: ApplyStyleChanges,
    ) {
        self.CheckIsNotDestroyed();
        if self.StylePtr() == style {
            return;
        }
        assert!(!style.is_null());
        if apply_changes == ApplyStyleChanges::kNo {
            let old_style = self.StylePtr();
            self.SetStyleInternal(style);
            self.UpdateMaskImageObservers(old_style, style);
            return;
        }
        let old_style = self.StylePtr();
        let mut difference = StyleDifference::default();
        if !old_style.is_null() {
            difference = TransformStyleDifference(unsafe { &*old_style }, unsafe { &*style });
        }
        let mut context = StyleChangeContext::default();
        unsafe {
            DispatchLayoutObjectStyleWillChange(self, difference, old_style, &*style, &mut context);
        }
        self.SetStyleInternal(style);
        self.UpdateMaskImageObservers(old_style, style);
        unsafe {
            DispatchLayoutObjectStyleDidChange(
                self,
                difference,
                old_style,
                &*self.StylePtr(),
                &context,
            );
        }
    }

    // cpp: layoutng/internal/layout_object.h:2208-2209
    pub fn SetPseudoElementStyle(&mut self, owner: &LayoutObject) {
        self.SetPseudoElementStyleWithMatchParentSize(owner, false);
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:184-188
    pub fn SetPseudoElementStyleWithMatchParentSize(
        &mut self,
        owner: &LayoutObject,
        _match_parent_size: bool,
    ) {
        self.CheckIsNotDestroyed();
        self.InheritIsInDetachedNonDomTree(owner);
        self.SetStyle(owner.StyleRef());
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:190-195
    pub fn StyleWillChange(
        &mut self,
        _difference: StyleDifference,
        _old_style: *const ComputedStyle,
        _new_style: &ComputedStyle,
        _context: &mut StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:197-203
    pub fn StyleDidChange(
        &mut self,
        difference: StyleDifference,
        _old_style: *const ComputedStyle,
        _new_style: &ComputedStyle,
        _context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(!difference.NeedsFullLayout());
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:205-217
    pub fn InsertedIntoTree(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectInsertedIntoTree(self) }
    }

    pub fn InsertedIntoTreeBase(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.HasInlineFragments());
        let parent = self.Parent();
        if !parent.is_null() && unsafe { &*parent }.ChildrenInline() {
            unsafe { &mut *parent }.DirtyLinesFromChangedChild(self);
        }
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() && unsafe { &*element }.InputMayBeImplicitAnchor() {
            self.MarkMayContainAnchor();
        } else if self.MayContainAnchor() && !parent.is_null() {
            unsafe { &mut *parent }.MarkMayContainAnchor();
        }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:218-223
    pub fn WillBeRemovedFromTree(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectWillBeRemovedFromTree(self) }
    }

    pub fn WillBeRemovedFromTreeBase(&mut self) {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        if !parent.is_null() && unsafe { &*parent }.ChildrenInline() {
            unsafe { &mut *parent }.DirtyLinesFromChangedChild(self);
        }
    }

    // This is the base implementation. Destroy() invokes the pending typed
    // virtual dispatcher so concrete derived cleanup runs before this method.
    // cpp: layoutng/internal/layout_object_core_services.cc:224-230
    pub fn WillBeDestroyed(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IsText());
        let children = self.VirtualChildren();
        if !children.is_null() {
            unsafe { &mut *children }.DestroyLeftoverChildren();
        }
        self.Remove();
    }

    // cpp: layoutng/internal/layout_object.h:1874-1874
    pub fn MarkContainerChainForLayout(&mut self) {
        self.MarkContainerChainForLayoutWithSchedule(true);
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:232-260
    pub fn MarkContainerChainForLayoutWithSchedule(&mut self, _schedule_relayout: bool) {
        self.CheckIsNotDestroyed();
        let mut object = self.Container();
        let mut last: *mut LayoutObject = self;
        let mut simplified = self.NeedsSimplifiedLayoutOnly();
        while !object.is_null() {
            let current = unsafe { &mut *object };
            if current.SelfNeedsFullLayout() {
                return;
            }
            let mut container = current.Container();
            let last_ref = unsafe { &*last };
            if !last_ref.IsTextOrSVGChild() && last_ref.StyleRef().HasOutOfFlowPosition() {
                object = last_ref.ContainingBlock() as *mut LayoutObject;
                if object.is_null() {
                    return;
                }
                container = unsafe { &*object }.Container();
                simplified = true;
            }
            let current = unsafe { &mut *object };
            if simplified {
                if current.NeedsSimplifiedLayout() {
                    return;
                }
                current.SetNeedsSimplifiedLayout(true);
            } else {
                if current.ChildNeedsFullLayout() {
                    return;
                }
                current.SetChildNeedsFullLayout(true);
            }
            last = object;
            object = container;
        }
    }

    // Base virtual implementation. Child calls retain derived dispatch.
    // cpp: layoutng/internal/layout_object_core_services.cc:280-292
    pub fn RecalcScrollableOverflow(&mut self) -> RecalcScrollableOverflowResult {
        self.CheckIsNotDestroyed();
        self.ClearSelfNeedsScrollableOverflowRecalc();
        if !self.ChildNeedsScrollableOverflowRecalc() {
            return RecalcScrollableOverflowResult::default();
        }
        self.ClearChildNeedsScrollableOverflowRecalc();
        let mut changed = false;
        let mut child = self.SlowFirstChild();
        while !child.is_null() {
            changed |= unsafe { DispatchLayoutObjectRecalcScrollableOverflow(child) }
                .scrollable_overflow_changed;
            child = unsafe { &*child }.NextSibling();
        }
        RecalcScrollableOverflowResult {
            scrollable_overflow_changed: changed,
            rebuild_fragment_tree: false,
        }
    }

    // Base virtual implementation. Child calls retain derived dispatch.
    // cpp: layoutng/internal/layout_object_core_services.cc:294-300
    pub fn RecalcVisualOverflow(&mut self) {
        self.CheckIsNotDestroyed();
        let mut child = self.SlowFirstChild();
        while !child.is_null() {
            unsafe { DispatchLayoutObjectRecalcVisualOverflow(child) };
            child = unsafe { &*child }.NextSibling();
        }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:302-313
    pub fn IsContainedBy(&self, object: *const LayoutObject) -> bool {
        self.CheckIsNotDestroyed();
        let mut skip_info = super::layout_object::AncestorSkipInfo::new(object);
        let mut current: *const LayoutObject = self;
        while !current.is_null() {
            if current == object {
                return true;
            }
            if skip_info.AncestorSkipped() {
                return false;
            }
            current = unsafe { &*current }.ContainerWithSkipInfo(&mut skip_info);
        }
        false
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:315-318
    pub fn IsInlineRuby(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutInline() && self.StyleRef().Display() == EDisplay::kRuby
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:320-323
    pub fn IsInlineRubyText(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutInline() && self.StyleRef().Display() == EDisplay::kRubyText
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:325-343
    pub fn IsButtonOrInputButton(&self) -> bool {
        self.CheckIsNotDestroyed();
        let control = DynamicTo::<HTMLFormControlElement>(self.GetNode());
        if control.is_null() {
            return false;
        }
        matches!(
            unsafe { &*control }.FormControlType(),
            FormControlType::kButtonButton
                | FormControlType::kButtonSubmit
                | FormControlType::kButtonReset
                | FormControlType::kButtonPopover
                | FormControlType::kInputButton
                | FormControlType::kInputImage
                | FormControlType::kInputReset
                | FormControlType::kInputSubmit
        )
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:345-360
    pub fn IsRenderedLegendInternal(&self) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsBox());
        debug_assert!(self.IsRenderedLegendCandidate());
        let parent = DynamicTo::<LayoutBlock>(self.Parent());
        if parent.is_null() {
            return false;
        }
        let node = unsafe { &*parent }.GetNode();
        if node.is_null() || unsafe { &*node }.InputKind() != NodeKind::kFieldset {
            return false;
        }
        let mut child = unsafe { &*parent }.FirstChild();
        while !child.is_null() {
            if unsafe { &*child }.IsRenderedLegendCandidate() {
                return child == self as *const LayoutObject as *mut LayoutObject;
            }
            child = unsafe { &*child }.NextSibling();
        }
        false
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:362-372
    pub fn IsScrollMarkerGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsInTopOrViewTransitionLayer(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:374-378
    pub fn IsFontFallbackValid(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { &*self.StyleRef().GetFont() }.IsFallbackValid()
            && unsafe { &*self.FirstLineStyleRef().GetFont() }.IsFallbackValid()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:380-400
    pub fn SlowEffectiveStyle(&self, style_variant: StyleVariant) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        match style_variant {
            StyleVariant::kStandard => self.StyleRef(),
            StyleVariant::kFirstLine => {
                if self.IsAtomicInline() {
                    self.StyleRef()
                } else {
                    self.FirstLineStyleRef()
                }
            }
            StyleVariant::kStandardEllipsis => {
                if !self.IsLayoutBlockFlow() {
                    let block = self.ContainingBlock();
                    if !block.is_null() {
                        return unsafe { &*block }.StyleRef();
                    }
                }
                self.StyleRef()
            }
            StyleVariant::kFirstLineEllipsis => {
                let block = self.ContainingBlock();
                if !block.is_null() {
                    return unsafe { &*block }.FirstLineStyleRef();
                }
                self.FirstLineStyleRef()
            }
        }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:402-408
    pub fn IsRelayoutBoundary(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:409-415
    pub fn IsRooted(&self) -> bool {
        self.CheckIsNotDestroyed();
        let mut root = self;
        while !root.Parent().is_null() {
            root = unsafe { &*root.Parent() };
        }
        root.HasPreparedLayoutInput()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:417-433
    pub fn ContainingScrollContainer(&self, axis: PhysicalAxis) -> *const LayoutBox {
        self.CheckIsNotDestroyed();
        let mut ancestor = self.Parent();
        while !ancestor.is_null() {
            let box_ = DynamicTo::<LayoutBox>(ancestor);
            if !box_.is_null() {
                let style = unsafe { &*box_ }.StyleRef();
                if (axis == PhysicalAxis::kVertical && style.IsOverflowValueScrollableY())
                    || (axis == PhysicalAxis::kHorizontal && style.IsOverflowValueScrollableX())
                {
                    return box_;
                }
            }
            ancestor = unsafe { &*ancestor }.Parent();
        }
        std::ptr::null()
    }

    // Base virtual implementation.
    // cpp: layoutng/internal/layout_object_core_services.cc:436-446
    pub fn OffsetFromContainerInternalBase(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        assert!(!container.is_null());
        debug_assert_eq!(container, self.Container() as *const LayoutObject);
        if unsafe { &*container }.IsScrollContainer() {
            return self.OffsetFromScrollableContainer(container, mode);
        }
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:447-461
    pub fn OffsetFromScrollableContainer(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        assert!(!container.is_null());
        debug_assert!(unsafe { &*container }.IsScrollContainer());
        if self.IsFixedPositioned() && unsafe { &*container }.IsLayoutView() {
            return PhysicalOffset::default();
        }
        if mode.contains(MapCoordinatesMode::kIgnoreScrollOriginAndOffset) {
            return PhysicalOffset::default();
        }
        let box_ = To::<LayoutBox>(container);
        if !mode.contains(MapCoordinatesMode::kIgnoreScrollOffset) {
            return -unsafe { &*box_ }.ScrolledContentOffset();
        }
        let area = unsafe { &*box_ }.GetScrollableArea();
        assert!(!area.is_null());
        let origin = unsafe { &*area }.ScrollOrigin();
        PhysicalOffset::new(
            LayoutUnit::from_signed(origin.x()),
            LayoutUnit::from_signed(origin.y()),
        )
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:463-469
    pub fn OffsetFromOverscrollContainer(
        &self,
        _container: *const LayoutObject,
        _mode: super::map_coordinates_flags::MapCoordinatesFlags,
    ) -> PhysicalOffset {
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:471-487
    pub fn LocalToAncestorPoint(
        &self,
        local_point: &gfx::PointF,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        let mut point = *local_point;
        let mut current: *const LayoutObject = self;
        while !current.is_null() && current != ancestor as *const LayoutObject {
            let container = unsafe { &*current }.Container();
            if container.is_null() {
                break;
            }
            let offset = unsafe { &*current }.OffsetFromContainer(container, mode);
            point.Offset(offset.left.ToFloat(), offset.top.ToFloat());
            current = container;
        }
        point
    }

    // GetName() is pure virtual in the source header.
    // cpp: layoutng/internal/layout_object.h:321-321
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectGetName(self) }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:489-499
    pub fn DebugName(&self) -> BlinkString {
        self.CheckIsNotDestroyed();
        let mut result = StringBuilder::default();
        result.Append(self.GetName());
        let node = self.GetNode();
        if !node.is_null() {
            let debug_name = unsafe { &*node }.InputDebugName();
            if !debug_name.is_empty() {
                result.Append(" ");
                result.Append(debug_name);
            }
        }
        result.ToString()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:501-505
    pub fn IsStyleGenerated(&self) -> bool {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        !node.is_null() && unsafe { &*node }.InputIsStyleGenerated()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:507-511
    pub fn ToString(&self) -> BlinkString {
        self.CheckIsNotDestroyed();
        self.DebugName()
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:513-522
    pub fn OwnerNodeId(&self, _is_internal_content: bool) -> DOMNodeId {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        if node.is_null()
            || unsafe { &*node }.InputId() == 0
            || unsafe { &*node }.InputId() > DOMNodeId::MAX as u64
        {
            return kInvalidDOMNodeId;
        }
        unsafe { &*node }.InputId() as DOMNodeId
    }

    // cpp: layoutng/internal/layout_object.h:351-351
    pub fn OwnerNodeIdDefault(&self) -> DOMNodeId {
        self.OwnerNodeId(false)
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:525-529
    #[cfg(debug_assertions)]
    pub fn ShowLayoutTreeForThis(&self) {
        self.CheckIsNotDestroyed();
        eprintln!("{:p}:{}", self, self.DebugName());
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:540-543
    pub fn UpdateSVGLayout(&mut self, info: &SVGLayoutInfo) -> SVGLayoutResult {
        self.CheckIsNotDestroyed();
        let algorithms = LayoutPassScope::Algorithms();
        if let Some(update) = unsafe { algorithms.as_ref() }
            .and_then(|algorithms| algorithms.svg_support.update_object)
        {
            return update(self, info);
        }
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:545-548
    pub fn ObjectBoundingBox(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:550-553
    pub fn StrokeBoundingBox(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:555-558
    pub fn DecoratedBoundingBox(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:560-563
    pub fn VisualRectInLocalSVGCoordinates(&self) -> gfx::RectF {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:565-568
    pub fn LocalSVGTransform(&self) -> AffineTransform {
        self.CheckIsNotDestroyed();
        AffineTransform::default()
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:531-534
impl std::fmt::Display for LayoutObject {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{:p}:{}", self, self.DebugName())
    }
}

// Rust's orphan rule prevents implementing Display for a raw pointer.
// cpp: layoutng/internal/layout_object_core_services.cc:536-538
pub struct LayoutObjectPointerDisplay(pub *const LayoutObject);

impl std::fmt::Display for LayoutObjectPointerDisplay {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_null() {
            out.write_str("<null>")
        } else {
            write!(out, "{}", unsafe { &*self.0 })
        }
    }
}

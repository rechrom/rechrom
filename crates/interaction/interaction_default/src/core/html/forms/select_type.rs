// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license in Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/forms/select_type.h:27-65,77-103
// Source: core/html/forms/select_type.cc:338-343,358-591,922-974,987-989,
// 1325-1630,1649-1702,1756-1946,2064-2075,2201-2253
// Platform popup construction/rendering stays with its existing popup owner.

use super::html_select_element::{
    kDeselectOtherOptionsFlag, kDispatchInputAndChangeEventFlag, HTMLSelectElement,
    NativeSelectElementPort, OptionRef, SameOption, SelectElement, USER_SELECTION_FLAGS,
};
use super::menu_list_popup::{MenuListPopup, MenuListPopupEnvironment};
use foundation::Persistent;
use interaction_dispatch::core::dom::events::event::EventRef;
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::rc::Weak;

/// Same DOM text and ComputedStyle owners; native methods are the individual
/// LayoutObject/AX/locale effects of select_type.cc:1041-1115.
pub trait MenuListTextOperations<O = OptionRef> {
    fn SelectAutofillPopoverPreviewEnabled(&self) -> bool;
    fn SuggestedOption(&self) -> Option<O>;
    fn OptionTextIndentedToRespectGroupLabel(&self, option: &O) -> Rc<[u16]>;
    fn OptionStyle(&self, option: &O) -> Persistent<ComputedStyle>;
    fn MultipleOptionsSelectedText(&self, count: usize) -> Rc<[u16]>;
    fn InnerStyle(&self) -> Persistent<ComputedStyle>;
    fn InnerHasLayoutObject(&self) -> bool;
    fn SetInnerModifiedStyleOutsideStyleRecalc(&self, style: Persistent<ComputedStyle>);
    fn SetInnerComputedStyle(&self, style: Persistent<ComputedStyle>);
    fn SetInnerTextNodeValue(&self, text: &[u16]);
    fn HasLayoutBox(&self) -> bool;
    fn AccessibilityTextChanged(&self);
}

// A per-event scalar projection, captured with the original EventRef retained.
// Source: select_type.cc:368,454,1332-1333,1446,1570.
#[derive(Default)]
pub struct SelectEvent {
    pub kind: String,
    pub keyboard: bool,
    pub mouse: bool,
    pub gesture: bool,
    pub key: String,
    pub key_code: i32,
    pub char_code: i32,
    pub platform_time_us: i64,
    pub modifiers: i32,
    pub repeat: bool,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
    pub button: i16,
    pub button_down: bool,
    pub from_touch: bool,
    pub target_option: Option<OptionRef>,
    pub target_inside_picker: bool,
    pub absolute_x: f64,
    pub absolute_y: f64,
}
// The source's layout updates occur before entry at cc:362/1398-1401; the
// synchronous transaction owner supplies a prepared geometry capability.
pub trait PreparedSelectGeometry {
    fn HasLayoutObject(&self) -> bool;
    fn IsInResizeMode(&self) -> bool;
    fn BlockDirectionKeys(&self) -> (&'static str, &'static str);
    fn ListBoxSize(&self) -> i32;
}
pub trait SelectEnvironment {
    fn PopupEnvironment(&self) -> Rc<dyn MenuListPopupEnvironment>;
    fn BindPopupLifecycle(&self, popup: Weak<MenuListPopup>);
    fn Geometry(&self) -> &dyn PreparedSelectGeometry;
    fn SelectTypeDestroyed(&self);
    fn UpdateStyleAndLayoutTree(&self);
    fn IsAppearanceBase(&self) -> bool;
    fn ComputedBaseAppearance(&self) -> Option<bool>;
    fn SetNeedsReattachLayoutTree(&self);
    fn CountBaseSelectAppearance(&self);
    fn InstallOptionStyleOwner(
        &self,
    ) -> layoutng_style::style::menu_list_inner_style::MenuListOptionStyleOwner;
    fn SpatialNavigationEnabled(&self) -> bool;
    fn PopsMenuByArrowKeys(&self) -> bool;
    fn PopsMenuByReturnKey(&self) -> bool;
    fn TypeAheadHasActiveSession(&self, event: &SelectEvent) -> bool;
    fn PickerIsPopover(&self) -> bool;
    fn PopupIsVisible(&self) -> bool;
    fn NativePopupIsVisible(&self) -> bool;
    fn LightDismissFromClickEnabled(&self) -> bool;
    fn Focus(&self, mouse: bool, from_touch: bool);
    fn SetLastFocusTypeMouse(&self);
    fn SetPopoverPointerdownTarget(&self);
    fn SetPopoverPickerPointerdown(&self, point: Option<(f64, f64)>);
    fn UpdatePopupFromElement(&self);
    fn MenuListText(&self) -> &dyn MenuListTextOperations;
    fn HasAXObjectCache(&self) -> bool;
    fn MarkAXElementDirty(&self, option: Option<&OptionRef>);
    fn HandleAXUpdateActiveMenuOption(&self);
    fn SetMayBeImplicitAnchor(&self);
    fn SubmitImplicitly(&self, event: &EventRef);
    fn SetMouseDownMayStartAutoscroll(&self);
    fn StartAutoscrollForSelection(&self);
    fn AutoscrollInProgress(&self) -> bool;
    fn StopAutoscroll(&self);
    fn IsFinishedParsingChildren(&self) -> bool;
    // Existing option scroll-task owner retains the option identity, coalesces
    // source tasks and performs prepared geometry scroll at cc:1704-1754.
    fn ScrollToOption(&self, option: Option<&OptionRef>);
    fn ForgetScrollOptionIfRemoved(&self, option: &OptionRef);
    fn ListboxActiveIndexChanged(&self);
}
pub trait SelectBehavior {
    // Finish the source constructor after the canonical Rust owner slot is
    // installed. ListBox's observer registration can synchronously reenter
    // that slot through OptionElementChildrenChanged.
    fn DidInstallOwner(&self) {}
    // SelectType::SelectAll cc:2157-2159; MenuList inherits NOTREACHED.
    fn SelectAll(&self) {
        unreachable!("SelectAll requires the source list-box subtype");
    }
    fn DidRecalcStyle(&self, reattach_layout_tree: bool);
    // SelectType::DidDetachLayoutTree cc:2127 inherited empty body.
    fn DidDetachLayoutTree(&self) {}
    // Source: select_type.h:58; select_type.cc:2163. MenuList inherits the empty base virtual;
    // ListBox overrides it to commit the same retained mouse selection.
    fn HandleMouseRelease(&self) {}
    fn WillBeDestroyed(&self);
    fn DefaultEventHandler(&self, event: &EventRef, input: &SelectEvent) -> bool;
    fn DidSelectOption(&self, option: Option<OptionRef>, flags: u32, update_popup: bool);
    fn SaveLastSelection(&self);
    fn DidBlur(&self);
    // Source select_type.cc:2165; MenuList inherits the empty base virtual.
    fn ListBoxOnChange(&self) {}
    // Source SelectType::SetListBoxActiveSelection cc:2169 and ScrollToSelection
    // cc:2153. MenuList inherits these empty base virtual bodies.
    fn SetListBoxActiveSelection(&self, _option: Option<OptionRef>) {}
    fn ScrollToSelection(&self) {}
    fn UpdateTextStyleAndContent(&self);
    // SelectType::OptionStyle cc:2139-2141 inherited null owner.
    fn OptionStyle(&self) -> Persistent<ComputedStyle> {
        Persistent::default()
    }
    // Source select_type.cc:2175: ListBox inherits this empty base virtual.
    fn ShowPicker(&self) {}
    fn ShowPopup(&self, touch: bool);
    fn HidePopup(&self);
    fn HidePopupWithoutEvents(&self);
    fn PopupDidHide(&self);
}
// Source select_type.cc:1287-1307,1649-1669,1794-1946. These are the
// source list-box owner's scalar/identity and selection-snapshot fields. DOM
// remains the collection/selectedness owner; no option tree is retained here.
pub struct ListBoxSelectionState<O> {
    option_to_scroll_to: RefCell<Option<O>>,
    cached_state: RefCell<Vec<bool>>,
    last_on_change_selection: RefCell<Vec<bool>>,
    active_selection_anchor: RefCell<Option<O>>,
    active_selection_end: RefCell<Option<O>>,
    non_contiguous: Cell<bool>,
    active_selection_state: Cell<bool>,
}
/// MenuListSelectType::DidUpdateActiveOption, cc:1117-1141. This source
/// notification state retires with the menu-list subtype, not the Document.
pub struct MenuListActiveOptionState {
    last_index: Cell<i32>,
    has_updated: Cell<bool>,
}
impl Default for MenuListActiveOptionState {
    fn default() -> Self {
        Self {
            last_index: Cell::new(-1),
            has_updated: Cell::new(false),
        }
    }
}
impl MenuListActiveOptionState {
    pub fn Update<O: Clone>(
        &self,
        owner: &impl SelectOptionStateOperations<O>,
        option: Option<&O>,
        notify: impl FnOnce(Option<O>),
    ) {
        let index = option.map_or(-1, |option| owner.Index(option));
        if self.last_index.get() == index {
            return;
        }
        let previous = usize::try_from(self.last_index.replace(index))
            .ok()
            .and_then(|index| owner.ListOption(index));
        if self.has_updated.replace(true) {
            notify(previous);
        }
    }
}
impl<O> Default for ListBoxSelectionState<O> {
    fn default() -> Self {
        Self {
            option_to_scroll_to: RefCell::new(None),
            cached_state: RefCell::new(Vec::new()),
            last_on_change_selection: RefCell::new(Vec::new()),
            active_selection_anchor: RefCell::new(None),
            active_selection_end: RefCell::new(None),
            non_contiguous: Cell::new(false),
            active_selection_state: Cell::new(false),
        }
    }
}
/// Live collection access and individual source effects, borrowed for one
/// synchronous default-action transaction. O retains only DOM identity.
pub trait SelectOptionStateOperations<O: Clone> {
    fn Same(&self, a: Option<&O>, b: Option<&O>) -> bool;
    fn ListLen(&self) -> usize;
    fn ListOption(&self, index: usize) -> Option<O>;
    fn OptionLen(&self) -> usize;
    fn Option(&self, index: usize) -> O;
    fn ListIndex(&self, option: &O) -> i32;
    fn Index(&self, option: &O) -> i32;
    fn Selected(&self, option: &O) -> bool;
    fn Disabled(&self, option: &O) -> bool;
    fn DisplayNone(&self, option: &O) -> bool;
    fn OptionHasLayout(&self, option: &O) -> bool;
    fn SetSelected(&self, option: &O, selected: bool, skip_observer: bool);
    fn SetDirty(&self, option: &O);
    fn IsMultiple(&self) -> bool;
    fn UsesMenuList(&self) -> bool;
    fn LastOnChangeOption(&self) -> Option<O>;
    fn OptionHasOwnerSelect(&self, option: &O) -> bool;
    fn NotifyFormStateChanged(&self);
    fn SelectedOption(&self) -> Option<O> {
        (0..self.OptionLen())
            .map(|i| self.Option(i))
            .find(|o| self.Selected(o))
    }
    fn LastSelectedOption(&self) -> Option<O> {
        (0..self.OptionLen())
            .rev()
            .map(|i| self.Option(i))
            .find(|o| self.Selected(o))
    }
}
pub trait SelectKeyboardOperations<O: Clone>: SelectOptionStateOperations<O> {
    fn SetMultiSelectFocused(&self, option: &O, focused: bool);
    fn SpatialNavigationEnabled(&self) -> bool;
    fn BlockDirectionKeys(&self) -> (&'static str, &'static str);
    fn ListBoxSize(&self) -> i32;
    fn ScrollToOption(&self, option: Option<&O>);
    fn IsFinishedParsingChildren(&self) -> bool;
    fn ListboxActiveIndexChanged(&self);
    fn SetNeedsValidityCheck(&self);
    fn DispatchInputEvent(&self);
    fn DispatchChangeEvent(&self);
    fn SelectMenuOption(&self, option: O);
    fn SubmitImplicitly(&self);
}
pub trait MenuListSelectionOperations<O: Clone>: SelectKeyboardOperations<O> {
    fn SetLastOnChangeOption(&self, option: Option<&O>);
    fn UpdateTextStyleAndContent(&self);
    fn NativePopupIsVisible(&self) -> bool;
    fn UpdatePopupFromElement(&self);
    fn HasLayoutObject(&self) -> bool;
    fn WillBeDestroyed(&self) -> bool;
    fn DidUpdateActiveOption(&self, option: Option<&O>);
}
/// MenuListSelectType::DidSelectOption, select_type.cc:922-951. Preserve
/// label/popup/validity work before input/change callbacks and revalidate the
/// subtype before the final active-option accessibility notification.
pub fn DidSelectMenuOptionWithOperations<O: Clone>(
    owner: &impl MenuListSelectionOperations<O>,
    option: Option<O>,
    flags: u32,
    update_popup: bool,
) {
    let dispatch = flags & kDispatchInputAndChangeEventFlag != 0
        && !owner.Same(owner.LastOnChangeOption().as_ref(), option.as_ref());
    owner.SetLastOnChangeOption(option.as_ref());
    owner.UpdateTextStyleAndContent();
    if owner.NativePopupIsVisible() && update_popup {
        owner.UpdatePopupFromElement();
    }
    owner.SetNeedsValidityCheck();
    if dispatch {
        owner.DispatchInputEvent();
        owner.DispatchChangeEvent();
    }
    if owner.HasLayoutObject() && !owner.WillBeDestroyed() {
        owner.DidUpdateActiveOption(option.as_ref());
    }
}
// Source select_type.cc:2201-2253. Both native SelectType and the live DOM
// adapter use the same navigation decisions over the original list indices.
pub fn NextValidOptionWithOperations<O: Clone>(
    owner: &impl SelectKeyboardOperations<O>,
    mut index: i32,
    direction: SkipDirection,
    mut skip: i32,
) -> Option<O> {
    let mut last = None;
    index += direction as i32;
    while index >= 0 && index < owner.ListLen() as i32 {
        skip -= 1;
        if let Some(option) = owner.ListOption(index as usize) {
            if !owner.DisplayNone(&option)
                && !owner.Disabled(&option)
                && (owner.UsesMenuList() || owner.OptionHasLayout(&option))
            {
                last = Some(option);
                if skip <= 0 {
                    break;
                }
            }
        }
        index += direction as i32;
    }
    last
}
impl<O: Clone> ListBoxSelectionState<O> {
    /// ListBoxSelectType::ScrollToOption, cc:1704-1718. The original option
    /// identity replaces the pending target without posting a second task.
    pub fn QueueScrollToOption(&self, option: O) -> bool {
        self.option_to_scroll_to
            .borrow_mut()
            .replace(option)
            .is_none()
    }
    /// ScrollToOptionTask releases its target before any lifecycle work.
    pub fn TakeScrollOption(&self) -> Option<O> {
        self.option_to_scroll_to.borrow_mut().take()
    }
    /// Source ListBoxSelectType::SelectAll, select_type.cc:1756-1771. The
    /// caller has already admitted the list-box subtype and prepared layout.
    pub fn SelectAll(&self, owner: &impl SelectKeyboardOperations<O>) {
        if !owner.IsMultiple() {
            return;
        }
        self.SaveLastSelection(owner);
        self.active_selection_state.set(true);
        self.SetActiveSelectionAnchor(owner, self.Next(owner, None, SkipDirection::kSkipForwards));
        *self.active_selection_end.borrow_mut() =
            self.Next(owner, None, SkipDirection::kSkipBackwards);
        self.UpdateListBoxSelection(owner, false, false);
        self.ListBoxOnChange(owner);
    }

    fn Next(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        start: Option<&O>,
        direction: SkipDirection,
    ) -> Option<O> {
        let index = start.map_or(
            if direction == SkipDirection::kSkipForwards {
                -1
            } else {
                owner.ListLen() as i32
            },
            |o| owner.ListIndex(o),
        );
        NextValidOptionWithOperations(owner, index, direction, 1)
    }
    fn PageAway(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        start: Option<&O>,
        direction: SkipDirection,
    ) -> Option<O> {
        let page = owner.ListBoxSize() - 1;
        let index = start.map_or(-1, |o| owner.ListIndex(o));
        let edge = if direction == SkipDirection::kSkipForwards {
            0
        } else {
            owner.ListLen() as i32 - 1
        };
        NextValidOptionWithOperations(
            owner,
            edge,
            direction,
            page + if direction == SkipDirection::kSkipForwards {
                index
            } else {
                edge - index
            },
        )
    }
    pub fn SaveLastSelection(&self, owner: &impl SelectKeyboardOperations<O>) {
        *self.last_on_change_selection.borrow_mut() = (0..owner.ListLen())
            .map(|i| owner.ListOption(i).is_some_and(|o| owner.Selected(&o)))
            .collect();
    }
    pub fn SetActiveSelectionAnchor(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        option: Option<O>,
    ) {
        *self.active_selection_anchor.borrow_mut() = option;
        self.SaveListboxActiveSelection(owner);
    }
    pub fn SaveListboxActiveSelection(&self, owner: &impl SelectKeyboardOperations<O>) {
        *self.cached_state.borrow_mut() = (0..owner.OptionLen())
            .map(|i| owner.Selected(&owner.Option(i)))
            .collect();
    }
    pub fn ScrollToSelection(&self, owner: &impl SelectKeyboardOperations<O>) {
        if !owner.IsFinishedParsingChildren() {
            return;
        }
        let end = self
            .active_selection_end
            .borrow()
            .clone()
            .or_else(|| owner.LastSelectedOption());
        owner.ScrollToOption(end.as_ref());
        owner.ListboxActiveIndexChanged();
    }
    pub fn UpdateMultiSelectFocus(&self, owner: &impl SelectKeyboardOperations<O>) {
        if !owner.IsMultiple() {
            return;
        }
        let end = self.active_selection_end.borrow().clone();
        for i in 0..owner.OptionLen() {
            let option = owner.Option(i);
            if owner.Disabled(&option) || !owner.OptionHasLayout(&option) {
                continue;
            }
            owner.SetMultiSelectFocused(
                &option,
                owner.Same(Some(&option), end.as_ref()) && self.non_contiguous.get(),
            );
        }
        self.ScrollToSelection(owner);
    }
    pub fn DeselectItems(&self, owner: &impl SelectKeyboardOperations<O>, exclude: Option<&O>) {
        HTMLSelectElement::DeselectItemsWithoutValidationWithOperations(owner, exclude);
    }
    pub fn UpdateListBoxSelection(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        deselect: bool,
        scroll: bool,
    ) {
        let anchor = self
            .active_selection_anchor
            .borrow()
            .as_ref()
            .map_or(-1, |o| owner.Index(o));
        let end = self
            .active_selection_end
            .borrow()
            .as_ref()
            .map_or(-1, |o| owner.Index(o));
        let start = anchor.min(end);
        let end = anchor.max(end);
        for i in 0..owner.OptionLen() {
            let option = owner.Option(i);
            if owner.Disabled(&option) || !owner.OptionHasLayout(&option) {
                continue;
            }
            if i as i32 >= start && i as i32 <= end {
                owner.SetSelected(&option, self.active_selection_state.get(), false);
                owner.SetDirty(&option);
            } else if deselect || i >= self.cached_state.borrow().len() {
                owner.SetSelected(&option, false, false);
                owner.SetDirty(&option);
            } else {
                let selected = self.cached_state.borrow()[i];
                owner.SetSelected(&option, selected, false);
            }
        }
        self.UpdateMultiSelectFocus(owner);
        owner.SetNeedsValidityCheck();
        if scroll {
            self.ScrollToSelection(owner);
        }
        owner.NotifyFormStateChanged();
    }
    pub fn UpdateSelectedState(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        option: O,
        mut mode: SelectionMode,
    ) {
        self.SaveLastSelection(owner);
        self.active_selection_state.set(true);
        if !owner.IsMultiple() {
            mode = SelectionMode::kDeselectOthers;
        }
        if owner.Selected(&option) && mode == SelectionMode::kNotChangeOthers {
            self.active_selection_state.set(false);
            owner.SetSelected(&option, false, false);
            owner.SetDirty(&option);
        }
        if mode == SelectionMode::kDeselectOthers {
            self.DeselectItems(owner, Some(&option));
        }
        if self.active_selection_anchor.borrow().is_none()
            && mode != SelectionMode::kNotChangeOthers
        {
            self.SetActiveSelectionAnchor(owner, owner.SelectedOption());
        }
        if !owner.Disabled(&option) {
            owner.SetSelected(&option, true, false);
            owner.SetDirty(&option);
        }
        if self.active_selection_anchor.borrow().is_none() || mode != SelectionMode::kRange {
            self.SetActiveSelectionAnchor(owner, Some(option.clone()));
        }
        *self.active_selection_end.borrow_mut() = Some(option);
        self.UpdateListBoxSelection(owner, mode != SelectionMode::kNotChangeOthers, true);
    }
    pub fn ToggleSelection(&self, owner: &impl SelectKeyboardOperations<O>, option: O) {
        self.active_selection_state
            .set(!self.active_selection_state.get());
        self.UpdateSelectedState(owner, option, SelectionMode::kNotChangeOthers);
        self.ListBoxOnChange(owner);
    }
    /// Source ListBoxSelectType::HandleMouseRelease, select_type.cc:1892-1895.
    /// Keep the retained cache private and share this original release branch
    /// with native SelectType and the call-scoped DOM operations adapter.
    pub fn HandleMouseRelease(&self, owner: &impl SelectKeyboardOperations<O>) {
        if !self.last_on_change_selection.borrow().is_empty() {
            self.ListBoxOnChange(owner);
        }
    }
    pub fn ListBoxOnChange(&self, owner: &impl SelectKeyboardOperations<O>) {
        if self.last_on_change_selection.borrow().is_empty()
            || self.last_on_change_selection.borrow().len() != owner.ListLen()
        {
            owner.DispatchChangeEvent();
            return;
        }
        let mut changed = false;
        for i in 0..owner.ListLen() {
            let selected = owner.ListOption(i).is_some_and(|o| owner.Selected(&o));
            let mut cache = self.last_on_change_selection.borrow_mut();
            changed |= selected != cache[i];
            cache[i] = selected;
        }
        if changed {
            owner.DispatchInputEvent();
            owner.DispatchChangeEvent();
        }
    }
    // Source select_type.cc:1598-1617,1619-1626. Keep retained anchor/end
    // identities when selection is programmatic; clear only a removed option.
    pub fn DidSelectOption(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        option: Option<O>,
        flags: u32,
    ) {
        if let Some(option) = option {
            let single = !owner.IsMultiple();
            let deselect = flags & kDeselectOtherOptionsFlag != 0;
            if self.active_selection_anchor.borrow().is_none() || single || deselect {
                self.SetActiveSelectionAnchor(owner, Some(option.clone()));
            }
            if self.active_selection_end.borrow().is_none() || single || deselect {
                *self.active_selection_end.borrow_mut() = Some(option);
            }
        }
        self.ScrollToSelection(owner);
        owner.SetNeedsValidityCheck();
    }
    pub fn ForgetRemovedOptions(&self, owner: &impl SelectKeyboardOperations<O>) {
        let anchor = self.active_selection_anchor.borrow().clone();
        if anchor.as_ref().is_some_and(|o| owner.ListIndex(o) < 0) {
            self.active_selection_anchor.borrow_mut().take();
        }
        let end = self.active_selection_end.borrow().clone();
        if end.as_ref().is_some_and(|o| owner.ListIndex(o) < 0) {
            self.active_selection_end.borrow_mut().take();
        }
    }
    // Source select_type.cc:1444-1596. The original keyboard branches and
    // source selection calls are shared with the native ListBoxSelectType.
    pub fn HandleKeyboard(
        &self,
        owner: &impl SelectKeyboardOperations<O>,
        input: &SelectEvent,
    ) -> bool {
        if !input.keyboard {
            return false;
        }
        if input.kind == "keydown" {
            let (next, previous) = owner.BlockDirectionKeys();
            let active = self.active_selection_end.borrow().clone();
            let start = active.clone().or_else(|| {
                if input.key == next || input.key == "PageDown" {
                    owner.LastSelectedOption()
                } else {
                    owner.SelectedOption()
                }
            });
            let end = if input.key == next {
                self.Next(owner, start.as_ref(), SkipDirection::kSkipForwards)
            } else if input.key == previous {
                self.Next(owner, start.as_ref(), SkipDirection::kSkipBackwards)
            } else if input.key == "PageDown" {
                self.PageAway(owner, start.as_ref(), SkipDirection::kSkipForwards)
            } else if input.key == "PageUp" {
                self.PageAway(owner, start.as_ref(), SkipDirection::kSkipBackwards)
            } else if input.key == "Home" {
                self.Next(owner, None, SkipDirection::kSkipForwards)
            } else if input.key == "End" {
                self.Next(owner, None, SkipDirection::kSkipBackwards)
            } else {
                None
            };
            if owner.SpatialNavigationEnabled()
                && (matches!(input.key.as_str(), "ArrowLeft" | "ArrowRight")
                    || matches!(input.key.as_str(), "ArrowDown" | "ArrowUp")
                        && owner.Same(end.as_ref(), active.as_ref()))
            {
                return false;
            }
            let control = if cfg!(target_os = "macos") {
                input.meta
            } else {
                input.ctrl
            };
            if owner.IsMultiple() && input.key_code == 32 && control {
                if let Some(end) = active {
                    self.ToggleSelection(owner, end);
                    return true;
                }
            }
            if let Some(end) = end {
                self.SaveLastSelection(owner);
                *self.active_selection_end.borrow_mut() = Some(end.clone());
                self.non_contiguous.set(owner.IsMultiple() && control);
                let should_select_active_option = !owner.IsMultiple()
                    || input.shift
                    || (!owner.SpatialNavigationEnabled() && !self.non_contiguous.get());
                if should_select_active_option {
                    self.active_selection_state.set(true);
                }
                let deselect = !owner.IsMultiple() || (!input.shift && should_select_active_option);
                if self.active_selection_anchor.borrow().is_none() || deselect {
                    if deselect {
                        self.DeselectItems(owner, None);
                    }
                    self.SetActiveSelectionAnchor(owner, Some(end.clone()));
                }
                owner.ScrollToOption(Some(&end));
                if should_select_active_option || self.non_contiguous.get() {
                    if should_select_active_option {
                        self.UpdateListBoxSelection(owner, deselect, true);
                        self.ListBoxOnChange(owner);
                    }
                    self.UpdateMultiSelectFocus(owner);
                } else {
                    self.ScrollToSelection(owner);
                }
                return true;
            }
        } else if input.kind == "keypress" {
            if input.key_code == 13 {
                owner.SubmitImplicitly();
                return true;
            }
            if owner.IsMultiple()
                && input.key_code == 32
                && (owner.SpatialNavigationEnabled() || self.non_contiguous.get())
            {
                let option = self.active_selection_end.borrow().clone().or_else(|| {
                    self.Next(
                        owner,
                        owner.LastSelectedOption().as_ref(),
                        SkipDirection::kSkipForwards,
                    )
                });
                if let Some(option) = option {
                    self.ToggleSelection(owner, option);
                    return true;
                }
            }
        }
        false
    }
}
// Source select_type.cc:390-437. Popup and spatial-navigation policy gates
// remain in the caller; selection/navigation executes once here.
pub fn HandleMenuListKeydownWithOperations<O: Clone>(
    owner: &impl SelectKeyboardOperations<O>,
    input: &SelectEvent,
) -> bool {
    let index = owner
        .SelectedOption()
        .as_ref()
        .map_or(-1, |o| owner.ListIndex(o));
    let option = match input.key.as_str() {
        "ArrowDown" | "ArrowRight" => {
            NextValidOptionWithOperations(owner, index, SkipDirection::kSkipForwards, 1)
        }
        "ArrowUp" | "ArrowLeft" => {
            NextValidOptionWithOperations(owner, index, SkipDirection::kSkipBackwards, 1)
        }
        "PageDown" => NextValidOptionWithOperations(owner, index, SkipDirection::kSkipForwards, 3),
        "PageUp" => NextValidOptionWithOperations(owner, index, SkipDirection::kSkipBackwards, 3),
        "Home" => NextValidOptionWithOperations(owner, -1, SkipDirection::kSkipForwards, 1),
        "End" => NextValidOptionWithOperations(
            owner,
            owner.ListLen() as i32,
            SkipDirection::kSkipBackwards,
            1,
        ),
        _ => return false,
    };
    if let Some(option) = option {
        owner.SelectMenuOption(option);
    }
    true
}

// Source: select_type.h:85.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SkipDirection {
    kSkipBackwards = -1,
    kSkipForwards = 1,
}
// Source: select_type.h:93-103; cc:2064,2073-2075.
pub struct SelectType {
    pub select: HTMLSelectElement,
    pub environment: Rc<dyn SelectEnvironment>,
    pub will_be_destroyed: Cell<bool>,
}
impl SelectType {
    pub fn new(select: Rc<dyn SelectElement>, environment: Rc<dyn SelectEnvironment>) -> Self {
        Self {
            select: HTMLSelectElement::new(select),
            environment,
            will_be_destroyed: Cell::new(false),
        }
    }
    pub fn WillBeDestroyed(&self) {
        self.will_be_destroyed.set(true);
        self.environment.SelectTypeDestroyed();
    }
    // Source: select_type.cc:2066-2071.
    pub fn Create(
        select: Rc<dyn SelectElement>,
        environment: Rc<dyn SelectEnvironment>,
    ) -> Box<dyn SelectBehavior> {
        if select.UsesMenuList() {
            Box::new(MenuListSelectType::new(select, environment))
        } else {
            Box::new(ListBoxSelectType::new(select, environment))
        }
    }
    // Source: select_type.cc:2201-2232.
    pub fn NextValidOption(
        &self,
        index: i32,
        direction: SkipDirection,
        skip: i32,
    ) -> Option<OptionRef> {
        NextValidOptionWithOperations(
            &NativeSelectKeyboardOperations::Base(self),
            index,
            direction,
            skip,
        )
    }
    // Source: select_type.cc:2234-2253.
    pub fn NextSelectableOption(&self, start: Option<&OptionRef>) -> Option<OptionRef> {
        self.NextValidOption(
            start.map_or(-1, |o| o.ListIndex()),
            SkipDirection::kSkipForwards,
            1,
        )
    }
    pub fn PreviousSelectableOption(&self, start: Option<&OptionRef>) -> Option<OptionRef> {
        self.NextValidOption(
            start.map_or(self.select.owner.ListLen() as i32, |o| o.ListIndex()),
            SkipDirection::kSkipBackwards,
            1,
        )
    }
    pub fn FirstSelectableOption(&self) -> Option<OptionRef> {
        self.NextSelectableOption(None)
    }
    pub fn LastSelectableOption(&self) -> Option<OptionRef> {
        self.PreviousSelectableOption(None)
    }
}

// Source: select_type.cc:332-343.
pub struct MenuListSelectType {
    pub base: SelectType,
    snav_arrow_key_selection: Cell<bool>,
    active_option_notification: MenuListActiveOptionState,
    option_style: layoutng_style::style::menu_list_inner_style::MenuListOptionStyleOwner,
    popup: Rc<MenuListPopup>,
    is_appearance_base_select: Cell<bool>,
}
impl MenuListSelectType {
    pub fn new(select: Rc<dyn SelectElement>, environment: Rc<dyn SelectEnvironment>) -> Self {
        let option_style = environment.InstallOptionStyleOwner();
        let popup = MenuListPopup::new(environment.PopupEnvironment());
        environment.BindPopupLifecycle(Rc::downgrade(&popup));
        let base = SelectType::new(select, environment);
        base.environment.SetMayBeImplicitAnchor();
        Self {
            base,
            snav_arrow_key_selection: Cell::new(false),
            active_option_notification: Default::default(),
            option_style,
            popup,
            is_appearance_base_select: Cell::new(false),
        }
    }
    // Source select_type.cc:1117-1141. State belongs to this MenuList subclass
    // and therefore retires when the select changes rendering type.
    pub fn DidUpdateActiveOption(&self, option: Option<&OptionRef>) {
        let environment = &self.base.environment;
        if !environment.HasAXObjectCache() {
            return;
        }
        self.active_option_notification.Update(
            &NativeSelectKeyboardOperations::MenuList(self, None),
            option,
            |previous| {
                environment.MarkAXElementDirty(previous.as_ref());
                environment.HandleAXUpdateActiveMenuOption();
            },
        );
    }
    // Source: select_type.cc:538-566.
    pub fn ShouldOpenPopupForKeyDownEvent(&self, event: &SelectEvent) -> bool {
        let env = &self.base.environment;
        if env.SpatialNavigationEnabled() || env.PopupIsVisible() {
            return false;
        }
        if env.IsAppearanceBase()
            && matches!(
                event.key.as_str(),
                "ArrowDown" | "ArrowUp" | "ArrowLeft" | "ArrowRight"
            )
        {
            return true;
        }
        (env.PopsMenuByArrowKeys() && matches!(event.key.as_str(), "ArrowDown" | "ArrowUp"))
            || (matches!(event.key.as_str(), "ArrowDown" | "ArrowUp") && event.alt)
            || (!event.alt && !event.ctrl && event.key == "F4")
    }
    // Source: select_type.cc:568-574.
    pub fn ShouldOpenPopupForKeyPressEvent(&self, event: &SelectEvent) -> bool {
        (event.key_code == 32 && !self.base.environment.TypeAheadHasActiveSession(event))
            || (self.base.environment.PopsMenuByReturnKey() && event.key_code == 13)
    }
    // Source: select_type.cc:576-591.
    pub fn HandlePopupOpenKeyboardEvent(&self) -> bool {
        self.base.environment.Focus(false, false);
        if !self.base.select.owner.HasLayoutObject()
            || self.base.will_be_destroyed.get()
            || self.base.select.owner.IsDisabledFormControl()
        {
            return false;
        }
        self.SaveLastSelection();
        self.ShowPopup(false);
        true
    }
    // Source: select_type.cc:954-961.
    pub fn DispatchEventsIfSelectedOptionChanged(&self) {
        let selected = self.base.select.owner.SelectedOption();
        if !SameOption(
            self.base.select.owner.LastOnChangeOption().as_ref(),
            selected.as_ref(),
        ) {
            self.base.select.owner.SetLastOnChangeOption(selected);
            self.base.select.owner.DispatchInputEvent();
            self.base.select.owner.DispatchChangeEvent();
        }
    }
    // Source select_type.cc:1141-1156. Options are the original retained
    // SelectElement collection, including its cancel/suggested identities.
    pub fn OptionToBeShown(&self) -> Option<OptionRef> {
        MenuListOptionToBeShownWithOperations(
            &NativeSelectKeyboardOperations::MenuList(self, None),
            self.base.environment.MenuListText(),
            self.base.select.owner.IndexToSelectOnCancel(),
            self.base.environment.IsAppearanceBase(),
        )
    }
    // Source select_type.cc:1041-1106. Only the three source properties are
    // changed on a COW builder rooted in the actual inner ComputedStyle.
    pub fn UpdateTextStyleInternal(&self) -> Rc<[u16]> {
        let shown = self.OptionToBeShown();
        let text = UpdateMenuListTextStyleWithOperations(
            &NativeSelectKeyboardOperations::MenuList(self, None),
            self.base.environment.MenuListText(),
            shown.as_ref(),
            &self.option_style,
            self.base.environment.IsAppearanceBase(),
        );
        if self.base.select.owner.HasLayoutObject() {
            self.DidUpdateActiveOption(shown.as_ref());
        }
        text
    }
}
// Shared source algorithm from select_type.cc:1041-1106. Both the native
// SelectType and the Document adapter keep the canonical option/style owners.
/// MenuListSelectType::OptionToBeShown, cc:1141-1156. Preserve cancel-preview
/// option identity and the source autofill/base-appearance admission.
pub fn MenuListOptionToBeShownWithOperations<O: Clone>(
    owner: &impl SelectOptionStateOperations<O>,
    text: &dyn MenuListTextOperations<O>,
    cancel_index: i32,
    is_base: bool,
) -> Option<O> {
    if let Some(option) = usize::try_from(cancel_index)
        .ok()
        .and_then(|index| owner.ListOption(index))
    {
        return Some(option);
    }
    if !text.SelectAutofillPopoverPreviewEnabled() && !is_base {
        if let Some(option) = text.SuggestedOption() {
            return Some(option);
        }
    }
    if owner.IsMultiple() {
        owner.SelectedOption()
    } else {
        owner.LastOnChangeOption()
    }
}
pub fn UpdateMenuListTextStyleWithOperations<O: Clone>(
    owner: &impl SelectOptionStateOperations<O>,
    text_owner: &dyn MenuListTextOperations<O>,
    shown: Option<&O>,
    option_style_owner: &RefCell<Persistent<ComputedStyle>>,
    is_appearance_base: bool,
) -> Rc<[u16]> {
    let mut option_style = Persistent::default();
    let text = if owner.IsMultiple() {
        let mut count = 0;
        let mut selected = None;
        for index in 0..owner.OptionLen() {
            let option = owner.Option(index);
            if owner.Selected(&option) {
                count += 1;
                if count == 1 {
                    selected = Some(option);
                }
            }
        }
        if count == 1 {
            let selected = selected.expect("one selected option");
            option_style = text_owner.OptionStyle(&selected);
            text_owner.OptionTextIndentedToRespectGroupLabel(&selected)
        } else {
            text_owner.MultipleOptionsSelectedText(count)
        }
    } else if let Some(option) = shown {
        option_style = text_owner.OptionStyle(option);
        text_owner.OptionTextIndentedToRespectGroupLabel(option)
    } else {
        Rc::from([])
    };
    *option_style_owner.borrow_mut() = option_style.clone();
    if !is_appearance_base {
        let inner_owner = text_owner.InnerStyle();
        if let (Some(inner), Some(option)) = (unsafe { inner_owner.Get().as_ref() }, unsafe {
            option_style.Get().as_ref()
        }) {
            if option.Direction() != inner.Direction()
                || option.GetUnicodeBidi() != inner.GetUnicodeBidi()
                || option.GetTextAlignForLine(true) != inner.GetTextAlignForLine(true)
            {
                let mut builder = ComputedStyleBuilder::from_style(inner);
                builder.SetDirection(option.Direction());
                builder.SetUnicodeBidi(option.GetUnicodeBidi());
                builder.SetTextAlign(option.GetTextAlignForLine(true));
                let style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
                if text_owner.InnerHasLayoutObject() {
                    text_owner.SetInnerModifiedStyleOutsideStyleRecalc(style);
                } else {
                    text_owner.SetInnerComputedStyle(style);
                }
            }
        }
    }
    // WTF::String::StripWhiteSpace's default Unicode predicate.
    let start = text
        .iter()
        .position(|&c| !foundation::unicode::IsSpaceOrNewline(c))
        .unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|&c| !foundation::unicode::IsSpaceOrNewline(c))
        .map_or(start, |i| i + 1);
    Rc::from(&text[start..end])
}
impl SelectBehavior for MenuListSelectType {
    // select_type.cc:999-1038.
    fn DidRecalcStyle(&self, reattach_layout_tree: bool) {
        let env = &self.base.environment;
        if let Some(is_base) = env.ComputedBaseAppearance() {
            if self.is_appearance_base_select.get() != is_base {
                self.popup.CloseForChangingAppearance();
                self.is_appearance_base_select.set(is_base);
                env.SetNeedsReattachLayoutTree();
                if let Some(option) = self.base.select.owner.LastOnChangeOption() {
                    option.UpdateMutationObserver(true);
                }
            }
            if is_base {
                env.CountBaseSelectAppearance();
            }
        }
        if reattach_layout_tree {
            return;
        }
        self.UpdateTextStyleInternal();
        self.popup.DidRecalcStyle();
    }
    fn DidDetachLayoutTree(&self) {
        self.popup.DidDetachLayoutTree();
    }
    fn WillBeDestroyed(&self) {
        self.base.WillBeDestroyed();
    }
    // Source: select_type.cc:358-536. Geometry capability is prepared at source
    // cc:362 before testing the current LayoutObject after author callbacks.
    fn DefaultEventHandler(&self, event: &EventRef, input: &SelectEvent) -> bool {
        let env = &self.base.environment;
        let owner = &self.base.select.owner;
        if input.kind == "keydown" {
            if !owner.HasLayoutObject() || !input.keyboard {
                return false;
            }
            if self.ShouldOpenPopupForKeyDownEvent(input) {
                return self.HandlePopupOpenKeyboardEvent();
            }
            if env.SpatialNavigationEnabled() && !self.snav_arrow_key_selection.get() {
                return false;
            }
            if env.PopsMenuByArrowKeys() && !env.SpatialNavigationEnabled() {
                return false;
            }
            if input.modifiers & 15 != 0 || env.IsAppearanceBase() {
                return false;
            }
            return HandleMenuListKeydownWithOperations(
                &NativeSelectKeyboardOperations::MenuList(self, Some(event)),
                input,
            );
        }
        if input.kind == "keypress" {
            if !owner.HasLayoutObject() || !input.keyboard {
                return false;
            }
            if input.key_code == 32 && env.SpatialNavigationEnabled() {
                self.snav_arrow_key_selection
                    .set(!self.snav_arrow_key_selection.get());
                return true;
            }
            if self.ShouldOpenPopupForKeyPressEvent(input) {
                return self.HandlePopupOpenKeyboardEvent();
            }
            if !env.PopsMenuByReturnKey() && input.key_code == 13 {
                env.SubmitImplicitly(event);
                self.DispatchEventsIfSelectedOptionChanged();
                return true;
            }
            return false;
        }
        if input.kind == "mousedown" && input.mouse && input.button == 0 {
            if env.LightDismissFromClickEnabled() {
                if input.target_inside_picker {
                    return false;
                }
            } else if env.PickerIsPopover() && env.PopupIsVisible() {
                return false;
            }
            if env.PickerIsPopover() {
                env.SetLastFocusTypeMouse();
            } else {
                env.Focus(true, input.from_touch);
            }
            if owner.HasLayoutObject()
                && !self.base.will_be_destroyed.get()
                && !owner.IsDisabledFormControl()
            {
                if env.PopupIsVisible() {
                    self.HidePopup();
                } else {
                    self.SaveLastSelection();
                    if env.PickerIsPopover() {
                        if !input.from_touch && !env.LightDismissFromClickEnabled() {
                            env.SetPopoverPointerdownTarget();
                        }
                        env.SetPopoverPickerPointerdown(Some((input.absolute_x, input.absolute_y)));
                    }
                    self.ShowPopup(input.from_touch);
                }
            }
            return true;
        }
        if input.kind == "mouseup"
            && input.mouse
            && input.button == 0
            && env.PickerIsPopover()
            && !input.from_touch
        {
            env.SetPopoverPickerPointerdown(None);
        }
        false
    }
    // Source: select_type.cc:922-952.
    fn DidSelectOption(&self, option: Option<OptionRef>, flags: u32, update_popup: bool) {
        DidSelectMenuOptionWithOperations(
            &NativeSelectKeyboardOperations::MenuList(self, None),
            option,
            flags,
            update_popup,
        );
    }
    // Source: select_type.cc:963-974,987-989.
    fn DidBlur(&self) {
        self.DispatchEventsIfSelectedOptionChanged();
        if self.base.environment.NativePopupIsVisible() {
            self.HidePopup();
        }
    }
    fn SaveLastSelection(&self) {
        self.base
            .select
            .owner
            .SetLastOnChangeOption(self.base.select.owner.SelectedOption());
    }
    fn UpdateTextStyleAndContent(&self) {
        let text = self.UpdateTextStyleInternal();
        let target = self.base.environment.MenuListText();
        target.SetInnerTextNodeValue(&text);
        if target.HasLayoutBox() && self.base.environment.HasAXObjectCache() {
            target.AccessibilityTextChanged();
        }
    }
    fn OptionStyle(&self) -> Persistent<ComputedStyle> {
        self.option_style.borrow().clone()
    }
    // Source select_type.cc:909-920.
    fn ShowPicker(&self) {
        self.base.environment.UpdateStyleAndLayoutTree();
        self.SaveLastSelection();
        self.ShowPopup(false);
    }
    fn ShowPopup(&self, touch: bool) {
        self.popup.ShowPopup(touch);
    }
    fn HidePopup(&self) {
        self.popup.HidePopup(true);
    }
    fn HidePopupWithoutEvents(&self) {
        self.popup.HidePopup(false);
    }
    fn PopupDidHide(&self) {
        self.popup.PopupDidHide();
    }
}

// Source: select_type.cc:1287-1307.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    kDeselectOthers,
    kRange,
    kNotChangeOthers,
}
pub struct ListBoxSelectType {
    pub base: SelectType,
    selection: ListBoxSelectionState<OptionRef>,
    appearance_state: Cell<Option<bool>>,
}
impl ListBoxSelectType {
    pub fn new(select: Rc<dyn SelectElement>, environment: Rc<dyn SelectEnvironment>) -> Self {
        Self {
            base: SelectType::new(select, environment),
            selection: ListBoxSelectionState::default(),
            appearance_state: Cell::new(None),
        }
    }
    // select_type.cc:2033-2060.
    fn UpdateOptionMutationObservers(&self, in_style_recalc: bool) {
        let mut needs_update = false;
        if let Some(new) = self.base.environment.ComputedBaseAppearance() {
            let old = self.appearance_state.replace(Some(new));
            needs_update = old != Some(new);
            if old.is_none() && new {
                needs_update = false;
            }
        }
        if needs_update {
            for index in 0..self.base.select.owner.OptionLen() {
                self.base
                    .select
                    .owner
                    .Option(index)
                    .UpdateMutationObserver(in_style_recalc);
            }
        }
    }
    // Source: select_type.cc:1649-1694.
    pub fn UpdateMultiSelectFocus(&self) {
        self.selection
            .UpdateMultiSelectFocus(&NativeSelectKeyboardOperations::ListBox(self, None));
    }
    pub fn SetActiveSelectionAnchor(&self, option: Option<OptionRef>) {
        self.selection
            .SetActiveSelectionAnchor(&NativeSelectKeyboardOperations::ListBox(self, None), option);
    }
    pub fn SetActiveSelectionEnd(&self, option: Option<OptionRef>) {
        *self.selection.active_selection_end.borrow_mut() = option;
    }
    pub fn ActiveSelectionEnd(&self) -> Option<OptionRef> {
        self.selection
            .active_selection_end
            .borrow()
            .clone()
            .or_else(|| self.base.select.owner.LastSelectedOption())
    }
    // Source: select_type.cc:1673-1679.
    pub fn SpatialNavigationFocusedOption(&self) -> Option<OptionRef> {
        if !self.base.environment.SpatialNavigationEnabled() {
            return None;
        }
        self.ActiveSelectionEnd()
            .or_else(|| self.base.FirstSelectableOption())
    }
    // Source: select_type.cc:1696-1702.
    pub fn ScrollToSelection(&self) {
        if !self.base.environment.IsFinishedParsingChildren() {
            return;
        }
        self.base
            .environment
            .ScrollToOption(self.ActiveSelectionEnd().as_ref());
        self.base.environment.ListboxActiveIndexChanged();
    }
    // Source: select_type.cc:1756-1771.
    pub fn SelectAll(&self) {
        if !self.base.select.owner.HasLayoutObject() || !self.base.select.owner.IsMultiple() {
            return;
        }
        self.SaveLastSelection();
        self.selection.active_selection_state.set(true);
        self.SetActiveSelectionAnchor(self.base.NextSelectableOption(None));
        self.SetActiveSelectionEnd(self.base.PreviousSelectableOption(None));
        self.UpdateListBoxSelection(false, false);
        self.ListBoxOnChange();
        self.base.select.owner.SetNeedsValidityCheck();
    }
    // Source: select_type.cc:1773-1792.
    pub fn NextSelectableOptionPageAway(
        &self,
        start: Option<&OptionRef>,
        direction: SkipDirection,
    ) -> Option<OptionRef> {
        let page = self.base.environment.Geometry().ListBoxSize() - 1;
        let index = start.map_or(-1, |o| o.ListIndex());
        let edge = if direction == SkipDirection::kSkipForwards {
            0
        } else {
            self.base.select.owner.ListLen() as i32 - 1
        };
        self.base.NextValidOption(
            edge,
            direction,
            page + if direction == SkipDirection::kSkipForwards {
                index
            } else {
                edge - index
            },
        )
    }
    // Source: select_type.cc:1794-1845.
    pub fn ToggleSelection(&self, option: OptionRef) {
        self.selection
            .ToggleSelection(&NativeSelectKeyboardOperations::ListBox(self, None), option);
    }
    pub fn UpdateSelectedState(&self, option: OptionRef, mut mode: SelectionMode) {
        self.selection.UpdateSelectedState(
            &NativeSelectKeyboardOperations::ListBox(self, None),
            option,
            mode,
        );
    }
    // Source: select_type.cc:1847-1882.
    pub fn UpdateListBoxSelection(&self, deselect: bool, scroll: bool) {
        self.selection.UpdateListBoxSelection(
            &NativeSelectKeyboardOperations::ListBox(self, None),
            deselect,
            scroll,
        );
    }
    // Source: select_type.cc:1884-1908.
    pub fn SaveListboxActiveSelection(&self) {
        self.selection
            .SaveListboxActiveSelection(&NativeSelectKeyboardOperations::ListBox(self, None));
    }
    pub fn HandleMouseRelease(&self) {
        self.selection
            .HandleMouseRelease(&NativeSelectKeyboardOperations::ListBox(self, None));
    }
    // Source: select_type.cc:1910-1946. Borrow guards end before events.
    pub fn ListBoxOnChange(&self) {
        self.selection
            .ListBoxOnChange(&NativeSelectKeyboardOperations::ListBox(self, None));
    }
    pub fn ClearLastOnChangeSelection(&self) {
        self.selection.last_on_change_selection.borrow_mut().clear();
    }
    pub fn SetListBoxActiveSelection(&self, option: Option<OptionRef>) {
        self.SetActiveSelectionAnchor(option.clone());
        self.SetActiveSelectionEnd(option);
    }
    // Source: select_type.cc:1619-1626.
    pub fn OptionRemoved(&self, option: &OptionRef) {
        self.base.environment.ForgetScrollOptionIfRemoved(option);
        if SameOption(
            self.selection.active_selection_anchor.borrow().as_ref(),
            Some(option),
        ) {
            self.selection.active_selection_anchor.borrow_mut().take();
        }
        if SameOption(
            self.selection.active_selection_end.borrow().as_ref(),
            Some(option),
        ) {
            self.selection.active_selection_end.borrow_mut().take();
        }
    }
}
impl SelectBehavior for ListBoxSelectType {
    fn SelectAll(&self) {
        ListBoxSelectType::SelectAll(self);
    }

    fn DidInstallOwner(&self) {
        self.UpdateOptionMutationObservers(false);
    }
    fn DidRecalcStyle(&self, _: bool) {
        self.UpdateOptionMutationObservers(true);
    }
    fn ListBoxOnChange(&self) {
        ListBoxSelectType::ListBoxOnChange(self);
    }
    fn SetListBoxActiveSelection(&self, option: Option<OptionRef>) {
        ListBoxSelectType::SetListBoxActiveSelection(self, option);
    }
    fn ScrollToSelection(&self) {
        ListBoxSelectType::ScrollToSelection(self);
    }

    fn HandleMouseRelease(&self) {
        ListBoxSelectType::HandleMouseRelease(self);
    }
    fn WillBeDestroyed(&self) {
        self.base.WillBeDestroyed();
    }
    // Source: select_type.cc:1325-1596.
    fn DefaultEventHandler(&self, event: &EventRef, input: &SelectEvent) -> bool {
        let env = &self.base.environment;
        let owner = &self.base.select.owner;
        if env.IsAppearanceBase() {
            return false;
        }
        if input.kind == "gesturetap" && input.gesture {
            env.Focus(false, false);
            if !owner.HasLayoutObject() || self.base.will_be_destroyed.get() {
                return false;
            }
            if let Some(option) = &input.target_option {
                if !owner.IsDisabledFormControl() {
                    self.UpdateSelectedState(
                        option.clone(),
                        if input.shift {
                            SelectionMode::kRange
                        } else {
                            SelectionMode::kNotChangeOthers
                        },
                    );
                    self.ListBoxOnChange();
                }
                return true;
            }
            return false;
        }
        if input.kind == "mousedown" && input.mouse && input.button == 0 {
            env.Focus(false, false);
            if !owner.HasLayoutObject()
                || self.base.will_be_destroyed.get()
                || owner.IsDisabledFormControl()
            {
                return false;
            }
            if let Some(option) = &input.target_option {
                if !option.IsDisabledFormControl() {
                    let control = if cfg!(target_os = "macos") {
                        input.meta
                    } else {
                        input.ctrl
                    };
                    self.UpdateSelectedState(
                        option.clone(),
                        if input.shift {
                            SelectionMode::kRange
                        } else if control {
                            SelectionMode::kNotChangeOthers
                        } else {
                            SelectionMode::kDeselectOthers
                        },
                    );
                }
                env.SetMouseDownMayStartAutoscroll();
                return true;
            }
            return false;
        }
        if input.kind == "mousemove" && input.mouse {
            if input.button != 0 || !input.button_down || env.Geometry().IsInResizeMode() {
                return false;
            }
            if owner.HasLayoutObject() {
                env.StartAutoscrollForSelection();
            }
            if self.selection.last_on_change_selection.borrow().is_empty() {
                return false;
            }
            if let Some(option) = &input.target_option {
                if !owner.IsDisabledFormControl() {
                    if owner.IsMultiple() {
                        if self.selection.active_selection_anchor.borrow().is_none() {
                            return false;
                        }
                        self.SetActiveSelectionEnd(Some(option.clone()));
                        self.UpdateListBoxSelection(false, true);
                    } else {
                        self.SetActiveSelectionAnchor(Some(option.clone()));
                        self.SetActiveSelectionEnd(Some(option.clone()));
                        self.UpdateListBoxSelection(true, true);
                    }
                }
            }
            return false;
        }
        if input.kind == "mouseup" && input.mouse && input.button == 0 && owner.HasLayoutObject() {
            if env.AutoscrollInProgress() {
                env.StopAutoscroll();
            } else {
                self.HandleMouseRelease();
            }
            return false;
        }
        if matches!(input.kind.as_str(), "keydown" | "keypress") {
            return self.selection.HandleKeyboard(
                &NativeSelectKeyboardOperations::ListBox(self, Some(event)),
                input,
            );
        }
        false
    }
    // Source: select_type.cc:1598-1617.
    fn DidSelectOption(&self, option: Option<OptionRef>, flags: u32, _update: bool) {
        self.selection.DidSelectOption(
            &NativeSelectKeyboardOperations::ListBox(self, None),
            option,
            flags,
        );
    }
    // Source: select_type.cc:1628-1630,1649-1656.
    fn DidBlur(&self) {
        self.ClearLastOnChangeSelection();
    }
    fn SaveLastSelection(&self) {
        self.selection
            .SaveLastSelection(&NativeSelectKeyboardOperations::ListBox(self, None));
    }
    fn UpdateTextStyleAndContent(&self) {
        // SelectType::UpdateTextStyleAndContent cc:2133 is the inherited
        // empty body for ListBoxSelectType, with no menu inner text to update.
    }
    fn ShowPopup(&self, _touch: bool) {
        unreachable!("listbox does not own a popup");
    }
    fn HidePopup(&self) {
        unreachable!("listbox does not own a popup");
    }
    fn HidePopupWithoutEvents(&self) {
        unreachable!("listbox does not own a popup");
    }
    fn PopupDidHide(&self) {
        unreachable!("listbox does not own a popup");
    }
}

// Borrowed bridge for the retained translated SelectType owners. The source
// state/algorithm above is also used by the Page adapter's live DOM identities.
enum NativeSelectKeyboardOperations<'a> {
    Base(&'a SelectType),
    MenuList(&'a MenuListSelectType, Option<&'a EventRef>),
    ListBox(&'a ListBoxSelectType, Option<&'a EventRef>),
}
impl NativeSelectKeyboardOperations<'_> {
    fn base(&self) -> &SelectType {
        match self {
            Self::Base(b) => b,
            Self::MenuList(t, _) => &t.base,
            Self::ListBox(t, _) => &t.base,
        }
    }
}
impl MenuListSelectionOperations<OptionRef> for NativeSelectKeyboardOperations<'_> {
    fn SetLastOnChangeOption(&self, option: Option<&OptionRef>) {
        self.base()
            .select
            .owner
            .SetLastOnChangeOption(option.cloned());
    }
    fn UpdateTextStyleAndContent(&self) {
        let Self::MenuList(menu, _) = self else {
            unreachable!("menu-list transaction requires menu-list subtype");
        };
        menu.UpdateTextStyleAndContent();
    }
    fn NativePopupIsVisible(&self) -> bool {
        self.base().environment.NativePopupIsVisible()
    }
    fn UpdatePopupFromElement(&self) {
        self.base().environment.UpdatePopupFromElement();
    }
    fn HasLayoutObject(&self) -> bool {
        self.base().select.owner.HasLayoutObject()
    }
    fn WillBeDestroyed(&self) -> bool {
        self.base().will_be_destroyed.get()
    }
    fn DidUpdateActiveOption(&self, option: Option<&OptionRef>) {
        let Self::MenuList(menu, _) = self else {
            unreachable!("menu-list transaction requires menu-list subtype");
        };
        menu.DidUpdateActiveOption(option);
    }
}
impl NativeSelectElementPort for NativeSelectKeyboardOperations<'_> {
    fn SourceSelectElement(&self) -> &HTMLSelectElement {
        &self.base().select
    }
}
impl SelectKeyboardOperations<OptionRef> for NativeSelectKeyboardOperations<'_> {
    fn SetMultiSelectFocused(&self, option: &OptionRef, focused: bool) {
        option.SetMultiSelectFocusedState(focused);
    }

    fn SpatialNavigationEnabled(&self) -> bool {
        self.base().environment.SpatialNavigationEnabled()
    }
    fn BlockDirectionKeys(&self) -> (&'static str, &'static str) {
        self.base().environment.Geometry().BlockDirectionKeys()
    }
    fn ListBoxSize(&self) -> i32 {
        self.base().environment.Geometry().ListBoxSize()
    }
    fn ScrollToOption(&self, option: Option<&OptionRef>) {
        self.base().environment.ScrollToOption(option);
    }
    fn IsFinishedParsingChildren(&self) -> bool {
        self.base().environment.IsFinishedParsingChildren()
    }
    fn ListboxActiveIndexChanged(&self) {
        self.base().environment.ListboxActiveIndexChanged();
    }
    fn SetNeedsValidityCheck(&self) {
        self.base().select.owner.SetNeedsValidityCheck();
    }
    fn DispatchInputEvent(&self) {
        self.base().select.owner.DispatchInputEvent();
    }
    fn DispatchChangeEvent(&self) {
        self.base().select.owner.DispatchChangeEvent();
    }
    fn SelectMenuOption(&self, option: OptionRef) {
        if let Self::MenuList(menu, _) = self {
            menu.base
                .select
                .SelectOption(Some(option), USER_SELECTION_FLAGS, 0, *menu);
        }
    }
    fn SubmitImplicitly(&self) {
        if let Self::MenuList(_, Some(event)) | Self::ListBox(_, Some(event)) = self {
            self.base().environment.SubmitImplicitly(event);
        }
    }
}

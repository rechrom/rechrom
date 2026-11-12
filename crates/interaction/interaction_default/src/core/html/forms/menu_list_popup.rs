// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source select_type.cc:728-908,990-997,1168-1241.
use crate::core::dom::native_mutation_observer::*;
use foundation::{
    gfx::{Rect, Size},
    popup_types::*,
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone, Copy)]
pub enum PopupMenuUpdateReason {
    DOMChange,
    StyleChange,
    SelectionChange,
}
/// The actual native PopupMenu. Selection callbacks return to the retained
/// source client, preserving the current HTMLSelectElement and SelectType.
pub trait NativePopupMenu {
    fn Show(&self, touch: bool);
    fn Hide(&self);
    fn DisconnectClient(&self);
    fn UpdateFromElement(&self, reason: PopupMenuUpdateReason);
}
pub trait NativePopupMenuClient {
    fn PopupDidHide(&self);
    fn SelectOptionByPopupIndex(&self, index: i32);
    fn SelectMultipleOptions(&self, list_indices: &[i32]);
    fn PopupDidCancel(&self);
    fn ProvisionalSelectionChanged(&self, index: u32);
}
pub trait MenuListPopupEnvironment {
    fn SelectId(&self) -> u64;
    fn SelectIsAppearanceBase(&self) -> bool;
    fn IsMultiple(&self) -> bool;
    fn IsAndroid(&self) -> bool;
    fn PickerComputedAppearance(&self) -> Option<foundation::AppearanceValue>;
    fn SupportsPickerAppearance(&self, appearance: foundation::AppearanceValue) -> bool;
    fn CachedBasePickerAppearance(&self) -> bool;
    fn SetCachedBasePickerAppearance(&self, value: bool);
    fn PopoverOpen(&self) -> bool;
    fn ShowPopover(&self);
    fn HidePopover(
        &self,
        focus: HidePopoverFocusBehavior,
        transition: HidePopoverTransitionBehavior,
    );
    fn NativePopupIsVisible(&self) -> bool;
    fn SetNativePopupIsVisible(&self, value: bool);
    fn OpenPseudoChanged(&self);
    fn HasLayoutObject(&self) -> bool;
    fn InvalidateFocusRingPaint(&self);
    fn HasOpenedPopup(&self) -> bool;
    fn VisibleBoundsInLocalRoot(&self) -> Rect;
    fn IsOutermostMainFrame(&self) -> bool;
    fn RootFrameToViewport(&self, rect: Rect) -> Rect;
    fn VisualViewportSize(&self) -> Size;
    fn UpdateStyleAndLayoutForPagePopup(&self);
    fn OpenPopupMenu(
        &self,
        client: Weak<dyn NativePopupMenuClient>,
    ) -> Option<Rc<dyn NativePopupMenu>>;
    fn UseExternalPopupMenus(&self) -> bool;
    fn HasAXObjectCache(&self) -> bool;
    fn DidShowMenuListPopup(&self);
    fn DidHideMenuListPopup(&self);
    fn AddAppearanceWarning(&self, message: &str);
    fn CountBasePickerAppearance(&self);
    fn SelectOptionByPopupIndex(&self, index: i32);
    fn SelectMultipleOptions(&self, list_indices: &[i32]);
    fn PopupDidCancel(&self);
    fn ProvisionalSelectionChanged(&self, index: u32);
    fn MutationObservers(&self) -> Rc<dyn NativeMutationObserverFactory>;
    fn CurrentAttribute(&self, node: u64, name: &str, namespace: &str) -> Option<Rc<[u16]>>;
    fn CurrentCharacterData(&self, node: u64) -> Rc<[u16]>;
}
pub struct MenuListPopup {
    self_: Weak<Self>,
    environment: Rc<dyn MenuListPopupEnvironment>,
    popup: RefCell<Option<Rc<dyn NativePopupMenu>>>,
    is_popup_external: Cell<bool>,
    updater: RefCell<Option<Rc<PopupUpdater>>>,
}
impl MenuListPopup {
    pub fn new(environment: Rc<dyn MenuListPopupEnvironment>) -> Rc<Self> {
        Rc::new_cyclic(|self_| Self {
            self_: self_.clone(),
            environment,
            popup: RefCell::new(None),
            is_popup_external: Cell::new(false),
            updater: RefCell::new(None),
        })
    }
    // cc:728-760. The display:none cache is written by the picker subclass's
    // actual DidRecalcStyle producer, before its computed style is discarded.
    pub fn IsAppearanceBasePicker(&self) -> bool {
        if !self.environment.SelectIsAppearanceBase() {
            return false;
        }
        Self::IsAppearanceBasePickerWithFacts(
            true,
            self.environment
                .PickerComputedAppearance()
                .map(|value| self.environment.SupportsPickerAppearance(value)),
            self.environment.CachedBasePickerAppearance(),
        )
    }
    // Source select_type.cc:728-741. A missing computed picker style uses
    // the scalar cache produced before display:none discarded that style.
    pub fn IsAppearanceBasePickerWithFacts(
        host_base: bool,
        computed_support: Option<bool>,
        cached_base: bool,
    ) -> bool {
        host_base && computed_support.unwrap_or(cached_base)
    }
    pub fn PickerIsPopover(&self) -> bool {
        Self::PickerIsPopoverWithFacts(
            self.IsAppearanceBasePicker(),
            self.environment.IsMultiple(),
            self.environment.IsAndroid(),
        )
    }
    // Source select_type.cc:744-759. Share the source pure policy with the
    // borrowed TypeAhead transaction, without constructing a popup host.
    pub fn PickerIsPopoverWithFacts(base_picker: bool, multiple: bool, android: bool) -> bool {
        base_picker || multiple && !android
    }
    pub fn SetIsAppearanceBasePickerForDisplayNone(&self, value: bool) {
        if value {
            self.environment.CountBasePickerAppearance();
        }
        self.environment.SetCachedBasePickerAppearance(value);
    }
    pub fn PopupIsVisible(&self) -> bool {
        let popover = self.environment.PopoverOpen();
        let native = self.environment.NativePopupIsVisible();
        assert!(
            !(popover && native),
            "base and native pickers cannot both be open"
        );
        popover || native
    }
    pub fn CloseForChangingAppearance(&self) {
        if self.PopupIsVisible() {
            self.environment.AddAppearanceWarning("A customizable-<select> changed its `appearance` property while open. As a result, it was closed to avoid circularity problems.");
            self.HidePopup(false);
        }
    }
    pub fn DidRecalcStyle(&self) {
        if self.environment.HasLayoutObject() {
            self.environment.InvalidateFocusRingPaint();
        }
        if self.environment.NativePopupIsVisible() {
            self.UpdateFromElement(PopupMenuUpdateReason::StyleChange);
        }
    }
    fn SetNativePopupIsVisible(&self, visible: bool) {
        self.environment.SetNativePopupIsVisible(visible);
        self.environment.OpenPseudoChanged();
        if self.environment.HasLayoutObject() {
            self.environment.InvalidateFocusRingPaint();
        }
    }
    pub fn ShowPopup(&self, touch: bool) {
        if self.PopupIsVisible() {
            return;
        }
        if self.PickerIsPopover() {
            self.environment.ShowPopover();
            if !self.PickerIsPopover() {
                self.environment.AddAppearanceWarning("A customizable-<select> changed its `appearance` property while open. As a result, it was closed to avoid circularity problems.");
                if self.environment.PopoverOpen() {
                    self.environment.HidePopover(
                        HidePopoverFocusBehavior::None,
                        HidePopoverTransitionBehavior::NoEventsNoWaiting,
                    );
                }
            }
            return;
        }
        if self.environment.HasOpenedPopup() || !self.environment.HasLayoutObject() {
            return;
        }
        let bounds = self.environment.VisibleBoundsInLocalRoot();
        let visible = if self.environment.IsOutermostMainFrame() {
            let mut rect = self.environment.RootFrameToViewport(bounds);
            rect.Intersect(Rect::new(
                foundation::gfx::Point::default(),
                self.environment.VisualViewportSize(),
            ));
            !rect.IsEmpty()
        } else {
            !bounds.IsEmpty()
        };
        if !visible {
            return;
        }
        self.SetNativePopupIsVisible(true);
        self.environment.UpdateStyleAndLayoutForPagePopup();
        let existing = self.popup.borrow().clone();
        if existing.is_none() {
            let client: Rc<dyn NativePopupMenuClient> =
                self.self_.upgrade().expect("live menu source");
            let popup = self.environment.OpenPopupMenu(Rc::downgrade(&client));
            *self.popup.borrow_mut() = popup;
            self.is_popup_external
                .set(self.environment.UseExternalPopupMenus());
        } else {
            let external = self.environment.UseExternalPopupMenus();
            if self.is_popup_external.get() != external {
                existing.expect("existing popup").Hide();
                let client: Rc<dyn NativePopupMenuClient> =
                    self.self_.upgrade().expect("live menu source");
                let popup = self.environment.OpenPopupMenu(Rc::downgrade(&client));
                *self.popup.borrow_mut() = popup;
                self.is_popup_external.set(external);
            }
        }
        let popup = self.popup.borrow().clone();
        let Some(popup) = popup else {
            self.SetNativePopupIsVisible(false);
            return;
        };
        assert!(self.updater.borrow().is_none());
        let updater = PopupUpdater::new(self.self_.clone(), self.environment.clone());
        *self.updater.borrow_mut() = Some(updater);
        popup.Show(touch);
        if self.environment.HasAXObjectCache() {
            self.environment.DidShowMenuListPopup();
        }
    }
    pub fn HidePopup(&self, normal: bool) {
        if self.environment.PopoverOpen() {
            self.environment.HidePopover(
                if normal {
                    HidePopoverFocusBehavior::FocusPreviousElement
                } else {
                    HidePopoverFocusBehavior::None
                },
                if normal {
                    HidePopoverTransitionBehavior::FireEventsAndWaitForTransitions
                } else {
                    HidePopoverTransitionBehavior::NoEventsNoWaiting
                },
            );
            return;
        }
        let popup = self.popup.borrow().clone();
        if let Some(popup) = popup {
            popup.Hide();
        }
    }
    pub fn PopupDidHide(&self) {
        self.SetNativePopupIsVisible(false);
        self.UnobserveTreeMutation();
        if self.environment.HasAXObjectCache() {
            self.environment.DidHideMenuListPopup();
        }
    }
    pub fn DidDetachLayoutTree(&self) {
        let popup = self.popup.borrow().clone();
        if let Some(popup) = popup {
            popup.DisconnectClient();
        }
        self.SetNativePopupIsVisible(false);
        self.popup.borrow_mut().take();
        self.UnobserveTreeMutation();
    }
    pub fn UpdateFromElement(&self, reason: PopupMenuUpdateReason) {
        let popup = self.popup.borrow().clone();
        if let Some(popup) = popup {
            popup.UpdateFromElement(reason);
        }
    }
    fn UnobserveTreeMutation(&self) {
        let updater = self.updater.borrow_mut().take();
        if let Some(updater) = updater {
            updater.observer.Disconnect();
        }
    }
}
impl NativePopupMenuClient for MenuListPopup {
    fn PopupDidHide(&self) {
        self.PopupDidHide();
    }
    fn SelectOptionByPopupIndex(&self, index: i32) {
        self.environment.SelectOptionByPopupIndex(index);
    }
    fn SelectMultipleOptions(&self, indices: &[i32]) {
        self.environment.SelectMultipleOptions(indices);
    }
    fn PopupDidCancel(&self) {
        self.environment.PopupDidCancel();
    }
    fn ProvisionalSelectionChanged(&self, index: u32) {
        self.environment.ProvisionalSelectionChanged(index);
    }
}
// Source cc:1168-1241. Late delivery after disconnect is explicitly allowed;
// every record is compared to the current native DOM value before updating UI.
struct PopupUpdater {
    owner: Weak<MenuListPopup>,
    environment: Rc<dyn MenuListPopupEnvironment>,
    observer: Rc<dyn NativeMutationObserver>,
}
impl PopupUpdater {
    fn new(owner: Weak<MenuListPopup>, environment: Rc<dyn MenuListPopupEnvironment>) -> Rc<Self> {
        Rc::new_cyclic(|self_: &Weak<Self>| {
            let delegate: Weak<dyn NativeMutationObserverDelegate> = self_.clone();
            let observer = environment.MutationObservers().Create(delegate);
            observer.Observe(
                environment.SelectId(),
                NativeMutationObserverOptions {
                    attributes: true,
                    attribute_old_value: true,
                    attribute_filter: Some(vec!["disabled", "label", "selected", "value"]),
                    character_data: true,
                    character_data_old_value: true,
                    child_list: true,
                    subtree: true,
                },
            );
            Self {
                owner,
                environment,
                observer,
            }
        })
    }
}
impl NativeMutationObserverDelegate for PopupUpdater {
    fn Deliver(&self, records: &[NativeMutationRecord]) {
        let Some(owner) = self.owner.upgrade() else {
            return;
        };
        if !owner.PopupIsVisible() {
            return;
        }
        for record in records {
            let unchanged = match &record.data {
                NativeMutationData::Attributes {
                    name,
                    namespace,
                    old_value,
                } => {
                    *old_value
                        == self
                            .environment
                            .CurrentAttribute(record.target, name, namespace)
                }
                NativeMutationData::CharacterData { old_value } => {
                    old_value.as_deref()
                        == Some(
                            self.environment
                                .CurrentCharacterData(record.target)
                                .as_ref(),
                        )
                }
                NativeMutationData::ChildList { .. } => false,
            };
            if unchanged {
                continue;
            }
            assert!(self.environment.NativePopupIsVisible());
            owner.UpdateFromElement(PopupMenuUpdateReason::DOMChange);
            return;
        }
    }
}

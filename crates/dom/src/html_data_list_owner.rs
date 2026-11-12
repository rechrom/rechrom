//! Canonical datalist/option/filtering members, Chromium 6c1d401fcca5.
//! Algorithms which dispatch events live in the HTML interaction owners.
use super::{editing_node::DOMNodeClass as C, PersistentDocument};

#[derive(Clone, Debug, Default)]
pub(super) struct HTMLDataListElementData {
    active_option: Option<usize>,
}
impl PersistentDocument {
    // Element::ShouldStoreComputedStyle cc:10526-10558. This source cache is
    // set while Style still holds the picker owner, before display:none drops
    // it. It is a scalar Cell on that same native select, not a style copy.
    pub fn SelectPickerShouldStoreComputedStyle(
        &self,
        node: usize,
        style: &layoutng_style::style::computed_style::ComputedStyle,
    ) -> Option<bool> {
        let (select, part) = self.SelectUAPart(node)?;
        if part != super::html_select_owner::DOMSelectUAPartKind::Picker {
            return None;
        }
        let host_style = self.ResolvedStyleFor(select)?.ComputedStyle()?;
        let is_base = self.SelectSupportsBaseAppearance(select, style.EffectiveAppearance())
            && self.SelectSupportsBaseAppearance(select, host_style.EffectiveAppearance());
        self.SetSelectCachedBasePickerAppearance(select, is_base);
        if is_base {
            self.CountUse(crate::use_counter::WebFeature::kSelectElementPickerAppearanceBaseSelect);
        }
        Some(!is_base)
    }
    pub fn DataListSupportsBaseAppearance(
        &self,
        node: usize,
        appearance: foundation::AppearanceValue,
    ) -> bool {
        self.select_source_features.customizable_combobox
            && matches!(self.arena[node].class, C::HTMLDataListElement(_))
            && appearance == foundation::AppearanceValue::kBase
    }
    pub fn SelectNativePopupIsVisible(&self, select: usize) -> bool {
        matches!(&self.arena[select].class, C::HTMLSelectElement(data) if data.native_popup_is_visible)
    }
    pub fn SetSelectNativePopupIsVisible(&mut self, select: usize, visible: bool) {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            panic!("native select")
        };
        data.native_popup_is_visible = visible;
    }
    pub fn SelectCachedBasePickerAppearance(&self, select: usize) -> bool {
        matches!(&self.arena[select].class, C::HTMLSelectElement(data) if data.is_appearance_base_picker_for_display_none.get())
    }
    pub fn SetSelectCachedBasePickerAppearance(&self, select: usize, base: bool) {
        let C::HTMLSelectElement(data) = &self.arena[select].class else {
            panic!("native select")
        };
        data.is_appearance_base_picker_for_display_none.set(base);
    }
    pub fn SelectPopupIsVisible(&self, select: usize) -> bool {
        let popover = self
            .SelectPickerElement(select)
            .is_some_and(|picker| self.PopoverOpen(picker));
        let native = self.SelectNativePopupIsVisible(select);
        assert!(!(popover && native));
        popover || native
    }
    pub fn SetPopoverPointerdownTarget(&mut self, target: Option<u64>) {
        self.popup_lifecycle_state.popover_pointerdown_target = target;
    }
    pub fn PopoverPointerdownTarget(&self) -> Option<u64> {
        self.popup_lifecycle_state.popover_pointerdown_target
    }
    pub fn ControlPseudoStateChanged(
        &mut self,
        node: usize,
        pseudo: crate::style_state::ControlPseudoState,
    ) {
        self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(node, pseudo));
    }
    pub fn SelectIndexToSelectOnCancel(&self, select: usize) -> i32 {
        match &self.arena[select].class {
            C::HTMLSelectElement(data) => data.index_to_select_on_cancel,
            _ => -1,
        }
    }
    pub fn SetSelectIndexToSelectOnCancel(&mut self, select: usize, index: i32) {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            unreachable!("native select");
        };
        data.index_to_select_on_cancel = index;
    }
    pub fn SelectLastOnChangeOption(&self, select: usize) -> Option<usize> {
        match &self.arena[select].class {
            C::HTMLSelectElement(data) => data.last_on_change_option,
            _ => None,
        }
    }
    pub fn SetSelectLastOnChangeOption(&mut self, select: usize, option: Option<usize>) {
        let C::HTMLSelectElement(data) = &mut self.arena[select].class else {
            panic!("last-on-change requires native select class")
        };
        data.last_on_change_option = option;
    }
    // HTMLOptionElement::TextIndentedToRespectGroupLabel cc:526-531.
    pub fn OptionTextIndentedToRespectGroupLabel(&self, option: usize) -> std::rc::Rc<[u16]> {
        let label = self.OptionDisplayLabelUTF16(option);
        if self.OptionOwnerOptgroup(option).is_none() {
            return label;
        }
        let mut text = vec![0x20; 4];
        text.extend_from_slice(&label);
        text.into()
    }
    pub fn SetSelectInnerComputedStyle(
        &mut self,
        select: usize,
        style: foundation::Persistent<layoutng_style::style::computed_style::ComputedStyle>,
    ) {
        let inner = self
            .SelectInnerElement(select)
            .expect("source menulist inner");
        let mut resolved = self
            .ResolvedStyleFor(inner)
            .expect("existing inner style")
            .clone();
        resolved.SetComputedStyleOwner(style);
        self.SetResolvedStyle(inner, resolved);
    }
    pub fn SelectSourceFeatures(&self) -> super::DOMSelectSourceFeatures {
        self.select_source_features
    }
    pub(super) fn InitializeDataListElement(&mut self, element: usize) {
        if self.select_source_features.customizable_combobox
            && matches!(self.arena[element].class, C::HTMLDataListElement(_))
        {
            self.popup_lifecycle_state
                .popovers
                .entry(self.arena[element].id)
                .or_default()
                .popover_type = foundation::popup_types::PopoverValueType::Auto;
        }
    }
    // HTMLInputElement::DataList, cc:2064-2075. Reflection retains the
    // reference-target owner rather than resolving list= through another ID map.
    pub fn InputDataList(&self, input: usize) -> Option<usize> {
        if !self.arena[input]
            .FindAttribute("list")
            .is_some_and(|a| !a.value.is_empty())
        {
            return None;
        }
        if !self.InputType(input)?.ShouldRespectListAttribute() {
            return None;
        }
        self.GetElementAttributeResolvingReferenceTarget(input, "list")
            .filter(|&target| matches!(self.arena[target].class, C::HTMLDataListElement(_)))
    }
    // HTMLInputElement::FilterTarget cc:2782-2800 and related-parts walk
    // html_select_element.cc:2157-2195. The insertion owner maintains counts;
    // this query reads the retained source association, including its barriers.
    pub fn InputFilterTarget(&self, input: usize) -> Option<usize> {
        if !self.select_source_features.filterable_select {
            return None;
        }
        if let Some(select) = self.InputOwnerSelectForFiltering(input) {
            return Some(select);
        }
        self.GetElementAttributeResolvingReferenceTarget(input, "filter")
            .filter(|&target| {
                matches!(self.arena[target].class, C::HTMLSelectElement(_))
                    && !self.SelectUsesMenuList(target)
            })
    }
    pub fn ActiveOption(&self, owner: usize) -> Option<usize> {
        match &self.arena[owner].class {
            C::HTMLDataListElement(data) => data.active_option,
            C::HTMLSelectElement(data) => data.active_option,
            _ => None,
        }
    }
    pub fn SetActiveOption(&mut self, owner: usize, option: Option<usize>) {
        match &mut self.arena[owner].class {
            C::HTMLDataListElement(data) => data.active_option = option,
            C::HTMLSelectElement(data) => data.active_option = option,
            _ => panic!("active option requires datalist/select class"),
        }
    }
    pub fn OptionIsFiltered(&self, option: usize) -> bool {
        matches!(
            self.arena[option].class,
            C::HTMLOptionElement {
                is_filtered: true,
                ..
            }
        )
    }
    // HTMLOptionElement::SetFiltered cc:944-956. Admission reads the same
    // document RuntimeEnabledFeatures snapshot as the source owners.
    pub fn SetOptionFiltered(&mut self, option: usize, filtered: bool) {
        if !self.select_source_features.customizable_combobox
            && !self.select_source_features.filterable_select
        {
            return;
        }
        let C::HTMLOptionElement { is_filtered, .. } = &mut self.arena[option].class else {
            panic!("filtered requires native option class")
        };
        if *is_filtered == filtered {
            return;
        }
        *is_filtered = filtered;
        self.OptionActivePseudoChanged(option, crate::style_state::ControlPseudoState::Filtered);
    }
    pub fn OptionActivePseudoChanged(
        &mut self,
        option: usize,
        pseudo: crate::style_state::ControlPseudoState,
    ) {
        self.RecordStyleChange(crate::style_state::StyleChange::ControlPseudo(
            option, pseudo,
        ));
    }
}

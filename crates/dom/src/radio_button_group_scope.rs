/*
 * Copyright (C) 2007, 2008, 2009 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

//! Source-owned radio membership on the canonical DOM nodes.
//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: core/html/forms/radio_button_group_scope.cc:32-317;
//! html_input_element.cc:899-908,1204-1255,1897-2023,2336-2364,2754-2766.
use super::*;

/// A form owns its own scope; connected unowned radios use their TreeScope.
/// The handle identifies the live DOM owner, including its document arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DOMRadioGroupScope {
    Form(DOMOwnerHandle),
    Tree(DOMOwnerHandle),
}
#[derive(Default)]
pub(super) struct RadioButtonGroup {
    // Chromium retains the required flag with each actual input node.
    members: HashMap<usize, bool>,
    required_count: usize,
    checked_button: Option<usize>,
    last_focused_button: Option<usize>,
}
impl RadioButtonGroup {
    fn IsValid(&self) -> bool {
        self.required_count == 0 || self.checked_button.is_some()
    }
}
impl PersistentDocument {
    /// The canonical HTMLInputElement now owns UpdateType source ordering.
    /// This flag carries only lifecycle admission, never a second input model.
    pub fn RegisterInputTypeLifecycleOwner(&mut self, input: usize) {
        assert!(self.InputType(input).is_some());
        self.input_type_lifecycle_owners
            .insert(self.arena[input].id);
    }
    /// Source HTMLInputElement::UpdateType removes the old group before the
    /// old InputTypeView/shadow subtree is destroyed. The saved membership key
    /// remains authoritative even though the content type attribute is new.
    pub fn RemoveRadioInputFromGroup(&mut self, input: usize) {
        if let Some(key) = self.radio_memberships.remove(&input) {
            self.RemoveRadioFromGroup(input, &key);
        }
    }
    /// Source HTMLInputElement::UpdateType adds the new group after creating
    /// the new concrete type/view. Use the live canonical DOM input type.
    pub fn AddRadioInputToGroup(&mut self, input: usize) {
        self.SynchronizeRadioInput(input);
    }
    pub fn RadioGroupScopeFor(&self, input: usize) -> Option<DOMRadioGroupScope> {
        if self.InputType(input) != Some(DOMInputType::kRadio) {
            return None;
        }
        if let Some(form) = self.FormOwner(input) {
            return Some(DOMRadioGroupScope::Form(self.NodeHandle(form)));
        }
        self.IsConnectedToDocument(input)
            .then(|| DOMRadioGroupScope::Tree(self.NodeHandle(self.TreeScopeRoot(input))))
    }
    fn RadioMembershipKey(&self, input: usize) -> Option<(DOMRadioGroupScope, String)> {
        let name = self.arena[input].FindAttribute("name")?.value.clone();
        if name.is_empty() {
            return None;
        }
        Some((self.RadioGroupScopeFor(input)?, name))
    }
    /// Called by the DOM's form/ID/tree mutation producer, never by an event
    /// query. Unchanged members retain their group and last-focus identity.
    /// FormObserverElements is the existing retained listed-control index.
    pub(super) fn SynchronizeRadioGroups(&mut self) {
        let mut inputs = self.FormObserverElements();
        inputs.sort_by(|a, b| {
            // TreeScope survives detachment, while boundary-point ordering
            // requires both nodes to be in the same current ordinary tree.
            let root_a = self.TreeRoot(*a);
            let root_b = self.TreeRoot(*b);
            if root_a != root_b {
                return root_a.cmp(&root_b);
            }
            self.CompareBoundaryPoints(
                crate::DOMBoundaryPoint {
                    node_id: self.arena[*a].id,
                    offset: 0,
                },
                crate::DOMBoundaryPoint {
                    node_id: self.arena[*b].id,
                    offset: 0,
                },
            )
            .expect("same resident tree")
        });
        for input in inputs {
            self.SynchronizeRadioInput(input);
        }
    }
    /// HTMLInputElement name/type/form/checked/required mutation is a single
    /// element transaction; it never visits unrelated controls.
    pub(super) fn SynchronizeRadioInput(&mut self, input: usize) {
        let next = self.RadioMembershipKey(input);
        let previous = self.radio_memberships.get(&input).cloned();
        if previous != next {
            if let Some(key) = previous {
                self.RemoveRadioFromGroup(input, &key);
                self.radio_memberships.remove(&input);
            }
            if let Some(key) = next {
                self.AddRadioToGroup(input, key);
            }
        } else if let Some(key) = next {
            self.RadioRequiredAttributeChanged(input, &key);
            // Attribute checkedness can change before a translated owner
            // exists. Dirty checkedness still comes from the canonical
            // control_checked_state, so checked content changes stop here.
            let checked = self.ControlChecked(input);
            let selected = self.radio_groups[&key].checked_button == Some(input);
            if checked != selected {
                self.UpdateRadioCheckedState(input);
            }
        }
    }
    fn AddRadioToGroup(&mut self, input: usize, key: (DOMRadioGroupScope, String)) {
        let required = self.IsRequiredFormControl(input);
        let checked = self.ControlChecked(input);
        let group = self.radio_groups.entry(key.clone()).or_default();
        let was_valid = group.IsValid();
        assert!(group.members.insert(input, required).is_none());
        group.required_count += usize::from(required);
        let old_checked = if checked {
            group.checked_button.replace(input)
        } else {
            None
        };
        let validity_changed = was_valid != group.IsValid();
        let invalid = !group.IsValid();
        self.radio_memberships.insert(input, key.clone());
        if let Some(old) = old_checked.filter(|old| *old != input) {
            self.SetAutofillState(old, DOMAutofillState::kNotFilled);
            self.SetControlChecked(old, false);
        }
        if validity_changed {
            self.SetRadioGroupNeedsValidityCheck(&key);
        } else if invalid {
            self.SetControlNeedsValidityCheck(input);
        }
    }
    fn RemoveRadioFromGroup(&mut self, input: usize, key: &(DOMRadioGroupScope, String)) {
        let Some(group) = self.radio_groups.get_mut(key) else {
            return;
        };
        let was_valid = group.IsValid();
        let Some(required) = group.members.remove(&input) else {
            return;
        };
        group.required_count -= usize::from(required);
        let removed_checked = group.checked_button == Some(input);
        if removed_checked {
            group.checked_button = None;
        }
        if group.last_focused_button == Some(input) {
            group.last_focused_button = None;
        }
        let validity_changed = was_valid != group.IsValid();
        if validity_changed {
            self.SetRadioGroupNeedsValidityCheck(key);
        }
        if removed_checked {
            self.InvalidateRadioGroupMembers(key);
        }
        if !was_valid {
            self.SetControlNeedsValidityCheck(input);
        }
        // Source retains empty named groups for subsequent reuse.
    }
    fn RadioRequiredAttributeChanged(&mut self, input: usize, key: &(DOMRadioGroupScope, String)) {
        let required = self.IsRequiredFormControl(input);
        let group = self
            .radio_groups
            .get_mut(key)
            .expect("resident radio group");
        let was_valid = group.IsValid();
        let flag = group
            .members
            .get_mut(&input)
            .expect("resident radio member");
        if *flag == required {
            return;
        }
        *flag = required;
        if required {
            group.required_count += 1;
        } else {
            group.required_count -= 1;
        }
        if was_valid != group.IsValid() {
            self.SetRadioGroupNeedsValidityCheck(key);
        }
    }
    pub(super) fn UpdateRadioCheckedState(&mut self, input: usize) {
        let Some(key) = self.radio_memberships.get(&input).cloned() else {
            // Disconnected, form-less radios use RadioInputType's same-root
            // fallback. It is intentionally the only non-retained group path.
            if self.ControlChecked(input) {
                for other in self.ResidentRadioGroupMembers(input) {
                    if other != input && self.ControlChecked(other) {
                        self.SetAutofillState(other, DOMAutofillState::kNotFilled);
                        self.SetControlChecked(other, false);
                    }
                }
            }
            return;
        };
        let checked = self.ControlChecked(input);
        let group = self
            .radio_groups
            .get_mut(&key)
            .expect("resident radio group");
        let was_valid = group.IsValid();
        let old = if checked {
            group.last_focused_button = None;
            group.checked_button.replace(input)
        } else {
            if group.checked_button == Some(input) {
                group.checked_button = None;
            }
            if group.last_focused_button == Some(input) {
                group.last_focused_button = None;
            }
            None
        };
        let validity_changed = was_valid != group.IsValid();
        if let Some(old) = old.filter(|old| *old != input) {
            self.SetAutofillState(old, DOMAutofillState::kNotFilled);
            self.SetControlChecked(old, false);
        }
        if validity_changed {
            self.SetRadioGroupNeedsValidityCheck(&key);
        }
        // :indeterminate depends on the whole group's checked member.
        self.InvalidateRadioGroupMembers(&key);
    }
    // RadioButtonGroup::SetNeedsValidityCheckForAllButtons, cc:191-196.
    // The validity cache lives with each original ListedElement, alongside
    // the canonical form/fieldset invalidation gates.
    fn SetRadioGroupNeedsValidityCheck(&mut self, key: &(DOMRadioGroupScope, String)) {
        let members = self
            .radio_groups
            .get(key)
            .map(|group| group.members.keys().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        for input in members {
            self.SetControlNeedsValidityCheck(input);
        }
    }
    fn InvalidateRadioGroupMembers(&mut self, key: &(DOMRadioGroupScope, String)) {
        let members = self
            .radio_groups
            .get(key)
            .map(|g| g.members.keys().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        for input in members {
            self.InvalidateNodeStyle(input);
        }
    }
    pub fn RadioIsInRequiredGroup(&self, input: usize) -> bool {
        self.radio_memberships
            .get(&input)
            .and_then(|key| self.radio_groups.get(key))
            .is_some_and(|g| g.required_count != 0 && g.members.contains_key(&input))
    }
    pub fn RadioCheckedButtonForGroup(&self, scope: DOMRadioGroupScope, name: &str) -> Option<u64> {
        self.radio_groups
            .get(&(scope, name.to_owned()))?
            .checked_button
            .map(|n| self.arena[n].id)
    }
    pub fn RadioLastFocusedButtonForGroup(
        &self,
        scope: DOMRadioGroupScope,
        name: &str,
    ) -> Option<u64> {
        self.radio_groups
            .get(&(scope, name.to_owned()))?
            .last_focused_button
            .map(|n| self.arena[n].id)
    }
    pub fn UpdateRadioLastFocusedState(&mut self, input: usize) {
        if !self.IsElementFocused(input) {
            return;
        }
        if let Some(key) = self.radio_memberships.get(&input) {
            self.radio_groups
                .get_mut(key)
                .expect("resident radio group")
                .last_focused_button = Some(input);
        }
    }
    pub(super) fn ResidentRadioGroupMembers(&self, input: usize) -> Vec<usize> {
        if let Some(key) = self.radio_memberships.get(&input) {
            return self.radio_groups[key].members.keys().copied().collect();
        }
        if self.InputType(input) != Some(DOMInputType::kRadio) {
            return Vec::new();
        }
        let name = self.arena[input]
            .FindAttribute("name")
            .map_or("", |a| a.value.as_str());
        if name.is_empty() {
            return vec![input];
        }
        let root = self.TreeScopeRoot(input);
        let mut result = Vec::new();
        let mut next = Some(root);
        while let Some(node) = next {
            if self.InputType(node) == Some(DOMInputType::kRadio)
                && self.FormOwner(node).is_none()
                && self.arena[node]
                    .FindAttribute("name")
                    .is_some_and(|a| a.value == name)
            {
                result.push(node);
            }
            next = self.RadioTraversalNext(node, root, true);
        }
        result
    }
    fn RadioTraversalNext(&self, node: usize, root: usize, forward: bool) -> Option<usize> {
        if forward {
            if let Some(first) = self.arena[node].children.first() {
                return Some(*first);
            }
            let mut current = node;
            while current != root {
                let parent = self.arena[current].parent?;
                let children = &self.arena[parent].children;
                let position = children.iter().position(|n| *n == current)?;
                if let Some(next) = children.get(position + 1) {
                    return Some(*next);
                }
                current = parent;
            }
            None
        } else {
            if node == root {
                return None;
            }
            let parent = self.arena[node].parent?;
            let children = &self.arena[parent].children;
            let position = children.iter().position(|n| *n == node)?;
            if position == 0 {
                return Some(parent);
            }
            let mut previous = children[position - 1];
            while let Some(last) = self.arena[previous].children.last() {
                previous = *last;
            }
            Some(previous)
        }
    }
    // Source radio_input_type.cc:47-53: form ListedElements order, otherwise
    // HTMLInputElement traversal, always over the resident ordinary DOM tree.
    pub fn NextRadioTraversalInput(
        &self,
        input: usize,
        form: Option<usize>,
        root: Option<usize>,
        forward: bool,
    ) -> Option<u64> {
        let root = root.unwrap_or_else(|| self.TreeScopeRoot(input));
        let mut next = self.RadioTraversalNext(input, root, forward);
        while let Some(node) = next {
            if self.InputType(node).is_some()
                && form.is_none_or(|form| self.FormOwner(node) == Some(form))
            {
                return Some(self.arena[node].id);
            }
            next = self.RadioTraversalNext(node, root, forward);
        }
        None
    }
    pub fn InclusiveFirstRadioTraversalInput(&self, root: usize) -> Option<u64> {
        if self.InputType(root).is_some() {
            return Some(self.arena[root].id);
        }
        self.NextRadioTraversalInput(root, None, Some(root), true)
    }
}

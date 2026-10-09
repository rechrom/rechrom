// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/active_style_sheets.h:16-37
// cpp: third_party/blink/renderer/core/css/active_style_sheets.cc:22-345

use crate::media_value_change::MediaValueChange;
use std::collections::HashSet;
use std::rc::Rc;

pub trait StyleSheetMediaQueries {
    fn HasMediaQueryResults(&self) -> bool;
    fn HasMediaQueries(&self) -> bool;
    fn HasDynamicViewportDependentMediaQueries(&self) -> bool;
}

pub trait RuleSetDiff<R> {
    fn Matches(&self, old: &R, new: &R) -> bool;
    fn CreateDiffRuleset(&self) -> Option<Rc<R>>;
}

pub struct ActiveStyleSheet<S, R> {
    pub style_sheet: Rc<S>,
    pub rule_set: Option<Rc<R>>,
}

impl<S, R> Clone for ActiveStyleSheet<S, R> {
    fn clone(&self) -> Self {
        Self {
            style_sheet: self.style_sheet.clone(),
            rule_set: self.rule_set.clone(),
        }
    }
}

impl<S, R> ActiveStyleSheet<S, R> {
    pub fn new(style_sheet: Rc<S>, rule_set: Option<Rc<R>>) -> Self {
        Self {
            style_sheet,
            rule_set,
        }
    }

    fn key(&self) -> (usize, usize) {
        (
            Rc::as_ptr(&self.style_sheet) as usize,
            self.rule_set.as_ref().map_or(0, |r| Rc::as_ptr(r) as usize),
        )
    }
}

impl<S, R> PartialEq for ActiveStyleSheet<S, R> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.style_sheet, &other.style_sheet)
            && match (&self.rule_set, &other.rule_set) {
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
impl<S, R> Eq for ActiveStyleSheet<S, R> {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveSheetsChange {
    kNoActiveSheetsChanged,
    kActiveSheetsChanged,
    kActiveSheetsAppended,
}

pub struct ChangedRuleSets<R> {
    identities_: HashSet<*const R>,
    values_: Vec<Rc<R>>,
}

impl<R> Default for ChangedRuleSets<R> {
    fn default() -> Self {
        Self {
            identities_: HashSet::new(),
            values_: Vec::new(),
        }
    }
}

impl<R> ChangedRuleSets<R> {
    pub fn Insert(&mut self, value: Rc<R>) {
        if self.identities_.insert(Rc::as_ptr(&value)) {
            self.values_.push(value);
        }
    }
    pub fn IsEmpty(&self) -> bool {
        self.values_.is_empty()
    }
    pub fn Values(&self) -> &[Rc<R>] {
        &self.values_
    }
}

// cpp: active_style_sheets.cc:27-45
fn CommonEntriesAreSubsequence<S, R>(
    candidate: &[ActiveStyleSheet<S, R>],
    sequence: &[ActiveStyleSheet<S, R>],
    sequence_sorted: &[ActiveStyleSheet<S, R>],
) -> bool {
    let mut sequence_index = 0;
    for entry in candidate {
        if sequence_sorted
            .binary_search_by_key(&entry.key(), |item| item.key())
            .is_err()
        {
            continue;
        }
        while sequence_index < sequence.len() && sequence[sequence_index] != *entry {
            sequence_index += 1;
        }
        if sequence_index == sequence.len() {
            return false;
        }
        sequence_index += 1;
    }
    true
}

// cpp: active_style_sheets.cc:52-72
fn CommonEntriesMayHaveBeenReordered<S, R>(
    old_middle: &[ActiveStyleSheet<S, R>],
    new_middle: &[ActiveStyleSheet<S, R>],
    old_sorted: &[ActiveStyleSheet<S, R>],
    new_sorted: &[ActiveStyleSheet<S, R>],
    removed_duplicate_entry: bool,
    added_duplicate_entry: bool,
) -> bool {
    if !removed_duplicate_entry {
        return !CommonEntriesAreSubsequence(old_middle, new_middle, new_sorted);
    }
    if !added_duplicate_entry {
        return !CommonEntriesAreSubsequence(new_middle, old_middle, old_sorted);
    }
    true
}

// cpp: active_style_sheets.cc:77-274
pub fn CompareActiveStyleSheets<S, R, D>(
    old_style_sheets: &[ActiveStyleSheet<S, R>],
    new_style_sheets: &[ActiveStyleSheet<S, R>],
    diffs: &[D],
    changed_rule_sets: &mut ChangedRuleSets<R>,
) -> ActiveSheetsChange
where
    S: StyleSheetMediaQueries,
    D: RuleSetDiff<R>,
{
    let old_count = old_style_sheets.len();
    let new_count = new_style_sheets.len();
    let mut index = 0;
    let min_count = old_count.min(new_count);

    while index < min_count
        && Rc::ptr_eq(
            &new_style_sheets[index].style_sheet,
            &old_style_sheets[index].style_sheet,
        )
    {
        let new_rules = &new_style_sheets[index].rule_set;
        let old_rules = &old_style_sheets[index].rule_set;
        let same_rules = match (new_rules, old_rules) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if !same_rules {
            let mut diff_ruleset = None;
            if let (Some(old), Some(new)) = (old_rules, new_rules) {
                for diff in diffs {
                    if diff.Matches(old, new) {
                        diff_ruleset = diff.CreateDiffRuleset();
                        break;
                    }
                }
            }
            if let Some(diff) = diff_ruleset {
                changed_rule_sets.Insert(diff);
            } else {
                if let Some(new) = new_rules {
                    changed_rule_sets.Insert(new.clone());
                }
                if let Some(old) = old_rules {
                    changed_rule_sets.Insert(old.clone());
                }
            }
        }
        index += 1;
    }

    let mut adds_non_matching_mq = false;
    if index == old_count {
        let changed_in_prefix = !changed_rule_sets.IsEmpty();
        while index < new_count {
            let entry = &new_style_sheets[index];
            if let Some(rules) = &entry.rule_set {
                changed_rule_sets.Insert(rules.clone());
            } else if entry.style_sheet.HasMediaQueryResults() {
                adds_non_matching_mq = true;
            }
            index += 1;
        }
        if changed_in_prefix {
            return ActiveSheetsChange::kActiveSheetsChanged;
        }
        if changed_rule_sets.IsEmpty() && !adds_non_matching_mq {
            return ActiveSheetsChange::kNoActiveSheetsChanged;
        }
        return ActiveSheetsChange::kActiveSheetsAppended;
    }

    if index == new_count {
        while index < old_count {
            let entry = &old_style_sheets[index];
            if let Some(rules) = &entry.rule_set {
                changed_rule_sets.Insert(rules.clone());
            } else if entry.style_sheet.HasMediaQueryResults() {
                adds_non_matching_mq = true;
            }
            index += 1;
        }
        return if changed_rule_sets.IsEmpty() && !adds_non_matching_mq {
            ActiveSheetsChange::kNoActiveSheetsChanged
        } else {
            ActiveSheetsChange::kActiveSheetsChanged
        };
    }

    let mut common_suffix_length = 0;
    while common_suffix_length < min_count - index
        && old_style_sheets[old_count - 1 - common_suffix_length]
            == new_style_sheets[new_count - 1 - common_suffix_length]
    {
        common_suffix_length += 1;
    }
    let old_middle = &old_style_sheets[index..old_count - common_suffix_length];
    let new_middle = &new_style_sheets[index..new_count - common_suffix_length];
    let mut old_sorted = old_middle.to_vec();
    let mut new_sorted = new_middle.to_vec();
    old_sorted.sort_by_key(|item| item.key());
    new_sorted.sort_by_key(|item| item.key());

    let mut removed_duplicate = false;
    let mut added_duplicate = false;
    let mut last_matched: Option<(usize, usize)> = None;
    let mut old_i = 0;
    let mut new_i = 0;
    let add_changed = |entry: &ActiveStyleSheet<S, R>,
                       duplicate: &mut bool,
                       changed: &mut ChangedRuleSets<R>,
                       non_matching: &mut bool,
                       last_matched: Option<(usize, usize)>| {
        if last_matched == Some(entry.key()) {
            *duplicate = true;
        }
        if let Some(rules) = &entry.rule_set {
            changed.Insert(rules.clone());
        } else if entry.style_sheet.HasMediaQueryResults() {
            *non_matching = true;
        }
    };
    while old_i < old_sorted.len() && new_i < new_sorted.len() {
        let old_key = old_sorted[old_i].key();
        let new_key = new_sorted[new_i].key();
        if old_key == new_key {
            last_matched = Some(old_key);
            old_i += 1;
            new_i += 1;
        } else if old_key < new_key {
            add_changed(
                &old_sorted[old_i],
                &mut removed_duplicate,
                changed_rule_sets,
                &mut adds_non_matching_mq,
                last_matched,
            );
            old_i += 1;
        } else {
            add_changed(
                &new_sorted[new_i],
                &mut added_duplicate,
                changed_rule_sets,
                &mut adds_non_matching_mq,
                last_matched,
            );
            new_i += 1;
        }
    }
    for entry in &old_sorted[old_i..] {
        add_changed(
            entry,
            &mut removed_duplicate,
            changed_rule_sets,
            &mut adds_non_matching_mq,
            last_matched,
        );
    }
    for entry in &new_sorted[new_i..] {
        add_changed(
            entry,
            &mut added_duplicate,
            changed_rule_sets,
            &mut adds_non_matching_mq,
            last_matched,
        );
    }

    if CommonEntriesMayHaveBeenReordered(
        old_middle,
        new_middle,
        &old_sorted,
        &new_sorted,
        removed_duplicate,
        added_duplicate,
    ) {
        for entry in old_middle {
            if let Some(rules) = &entry.rule_set {
                changed_rule_sets.Insert(rules.clone());
            }
        }
    }
    if changed_rule_sets.IsEmpty() && !adds_non_matching_mq {
        ActiveSheetsChange::kNoActiveSheetsChanged
    } else {
        ActiveSheetsChange::kActiveSheetsChanged
    }
}

// cpp: active_style_sheets.cc:280-345
pub fn AffectedByMediaValueChange<S, R>(
    active_sheets: &[ActiveStyleSheet<S, R>],
    change: MediaValueChange,
) -> bool
where
    S: StyleSheetMediaQueries,
{
    match change {
        MediaValueChange::kSize => active_sheets
            .iter()
            .any(|entry| entry.style_sheet.HasMediaQueryResults()),
        MediaValueChange::kDynamicViewport => active_sheets
            .iter()
            .any(|entry| entry.style_sheet.HasDynamicViewportDependentMediaQueries()),
        MediaValueChange::kOther => active_sheets
            .iter()
            .any(|entry| entry.style_sheet.HasMediaQueries()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Sheet {
        size: bool,
        any: bool,
        dynamic: bool,
    }
    impl StyleSheetMediaQueries for Sheet {
        fn HasMediaQueryResults(&self) -> bool {
            self.size
        }
        fn HasMediaQueries(&self) -> bool {
            self.any
        }
        fn HasDynamicViewportDependentMediaQueries(&self) -> bool {
            self.dynamic
        }
    }
    struct Rule(u8);
    struct NoDiff;
    impl RuleSetDiff<Rule> for NoDiff {
        fn Matches(&self, _: &Rule, _: &Rule) -> bool {
            false
        }
        fn CreateDiffRuleset(&self) -> Option<Rc<Rule>> {
            None
        }
    }
    fn entry(sheet: &Rc<Sheet>, rule: &Rc<Rule>) -> ActiveStyleSheet<Sheet, Rule> {
        ActiveStyleSheet::new(sheet.clone(), Some(rule.clone()))
    }

    #[test]
    fn append_reorder_and_media_changes_follow_source_classification() {
        let a = Rc::new(Sheet::default());
        let b = Rc::new(Sheet {
            size: true,
            any: true,
            dynamic: true,
        });
        let ar = Rc::new(Rule(1));
        let br = Rc::new(Rule(2));
        let old = vec![entry(&a, &ar)];
        let appended = vec![entry(&a, &ar), entry(&b, &br)];
        let mut changed = ChangedRuleSets::default();
        assert_eq!(
            CompareActiveStyleSheets(&old, &appended, &[NoDiff], &mut changed),
            ActiveSheetsChange::kActiveSheetsAppended
        );
        assert_eq!(changed.Values().len(), 1);

        let mut changed = ChangedRuleSets::default();
        assert_eq!(
            CompareActiveStyleSheets(
                &appended,
                &[entry(&b, &br), entry(&a, &ar)],
                &[NoDiff],
                &mut changed
            ),
            ActiveSheetsChange::kActiveSheetsChanged
        );
        assert_eq!(changed.Values().len(), 2);
        assert!(AffectedByMediaValueChange(
            &appended,
            MediaValueChange::kSize
        ));
        assert!(AffectedByMediaValueChange(
            &appended,
            MediaValueChange::kDynamicViewport
        ));
        assert!(AffectedByMediaValueChange(
            &appended,
            MediaValueChange::kOther
        ));
        assert_eq!(ar.0, 1);
    }
}

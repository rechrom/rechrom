//! FrameAdEvidence.{h:20-94,cc:11-42}, LocalFrame.cc:2844-2887.
//! Native Page roots begin without evidence; only the browser's actual
//! subresource-filter/frame-creation producers may install completed evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterListResult {
    NotChecked,
    MatchedAllowingRule,
    MatchedNoRules,
    MatchedBlockingRule,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FrameCreationStackEvidence {
    NotCreatedByAdScript,
    CreatedByAdScript,
}
#[derive(Clone, Debug)]
pub struct FrameAdEvidence {
    complete: bool,
    parent_is_ad: bool,
    latest: FilterListResult,
    most_restrictive: FilterListResult,
    created_by_ad_script: FrameCreationStackEvidence,
}
impl FrameAdEvidence {
    pub fn new(parent_is_ad: bool) -> Self {
        Self {
            complete: false,
            parent_is_ad,
            latest: FilterListResult::NotChecked,
            most_restrictive: FilterListResult::NotChecked,
            created_by_ad_script: FrameCreationStackEvidence::NotCreatedByAdScript,
        }
    }
    pub fn SetCreatedByAdScript(&mut self, evidence: FrameCreationStackEvidence) {
        assert!(!self.complete);
        assert!(evidence >= self.created_by_ad_script);
        self.created_by_ad_script = evidence;
    }
    pub fn UpdateFilterListResult(&mut self, value: FilterListResult) {
        self.latest = value;
        self.most_restrictive = self.most_restrictive.max(value);
    }
    pub fn IsComplete(&self) -> bool {
        self.complete
    }
    pub fn CanReplace(&self, old: &Self) -> bool {
        self.parent_is_ad == old.parent_is_ad && self.most_restrictive >= old.most_restrictive
    }
    pub fn SetComplete(&mut self) {
        self.complete = true;
    }
    pub fn IndicatesAdFrame(&self) -> bool {
        assert!(self.complete);
        self.parent_is_ad
            || self.created_by_ad_script == FrameCreationStackEvidence::CreatedByAdScript
            || self.most_restrictive == FilterListResult::MatchedBlockingRule
    }
}

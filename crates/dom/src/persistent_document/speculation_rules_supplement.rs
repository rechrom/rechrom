//! DocumentSpeculationRules::FromIfExists (supplement identity only).
//! DisplayLockUtilities.cc:457-473 notifies an existing original rules
//! owner; no empty rules object or independent candidate algorithm is created.
use super::PersistentDocument;
use std::rc::Rc;
/// The source Document supplement's typed lifecycle receiver. Roots are checked
/// identities from this same Document's actual forced-update scope registry.
pub trait DocumentSpeculationRules {
    fn DisplayLockedRootsForceUpdateEnded(&self, roots: &[u64]);
}
impl PersistentDocument {
    pub fn ExistingDocumentSpeculationRules(&self) -> Option<Rc<dyn DocumentSpeculationRules>> {
        self.document_speculation_rules.borrow().clone()
    }
    pub fn InstallDocumentSpeculationRules(&self, rules: Rc<dyn DocumentSpeculationRules>) {
        assert!(
            self.document_speculation_rules
                .borrow_mut()
                .replace(rules)
                .is_none(),
            "one DocumentSpeculationRules supplement per original Document"
        );
    }
}

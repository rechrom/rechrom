//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! network/content_security_policy.cc:698-746; trusted_types_directive.cc:15-55;
//! csp_directive_list.cc:753-774,1161-1207; content_security_policy.cc:1153-1194,
//! 1233-1377. Uses the original compiled headers and violation delivery owner.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustedTypePolicyDetails {
    Allowed,
    DisallowedName,
    DisallowedDuplicateName,
}
#[derive(Default)]
pub(super) struct TrustedTypesDirective {
    raw: String,
    names: Vec<Vec<u16>>,
    allow_any: bool,
    allow_duplicates: bool,
}
fn valid_name(value: &[u16]) -> bool {
    value.iter().all(|&c| {
        c <= 127 && ((c as u8).is_ascii_alphanumeric() || b"-#=_/@.%".contains(&(c as u8)))
    })
}
pub(super) fn Parse(
    value: &str,
    header: &str,
    errors: &mut Vec<CSPParsingError>,
) -> TrustedTypesDirective {
    let mut result = TrustedTypesDirective {
        raw: value.into(),
        ..Default::default()
    };
    if value == "'none'" {
        return result;
    }
    for token in words(value) {
        if token == "*" {
            result.allow_any = true;
        } else if token.eq_ignore_ascii_case("'allow-duplicates'") {
            result.allow_duplicates = true;
        } else if token.eq_ignore_ascii_case("'none'") {
            error(errors,header,"The value of the Content Security Policy directive 'trusted_types' contains an invalid policy: 'none'. It will be ignored. Note that 'none' has no effect unless it is the only expression in the directive value.".into());
        } else if valid_name(&token.encode_utf16().collect::<Vec<_>>()) {
            result.names.push(token.encode_utf16().collect());
        } else {
            error(errors,header,format!("The value of the Content Security Policy directive 'trusted_types' contains an invalid policy: '{token}'. It will be ignored."));
        }
    }
    result
}
impl TrustedTypesDirective {
    fn Allows(&self, name: &[u16], duplicate: bool) -> TrustedTypePolicyDetails {
        use TrustedTypePolicyDetails::*;
        if duplicate && (!self.allow_duplicates || name == [100, 101, 102, 97, 117, 108, 116]) {
            DisallowedDuplicateName
        } else if !valid_name(name) || !(self.allow_any || self.names.iter().any(|v| v == name)) {
            DisallowedName
        } else {
            Allowed
        }
    }
}
fn sample(value: &[u16]) -> &[u16] {
    // StringImpl::StripWhiteSpace uses the original ICU bidi-space predicate.
    let space = foundation::unicode::IsSpaceOrNewline;
    let start = value.iter().position(|c| !space(*c)).unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|c| !space(*c))
        .map_or(start, |n| n + 1);
    &value[start..end.min(start + 40)]
}
impl DocumentContentSecurityPolicy {
    pub fn ShouldBypassMainWorldForTrustedTypes(&self, world: CSPWorld) -> bool {
        matches!(world,CSPWorld::Isolated(id) if self.compiled.read().expect("Document CSP").isolated.contains_key(&id))
    }
    pub fn AllowTrustedTypePolicy(
        &self,
        name: &[u16],
        duplicate: bool,
        world: CSPWorld,
        location: CSPSourceLocation,
    ) -> (bool, TrustedTypePolicyDetails) {
        let state = PolicySnapshot(self.compiled.read().expect("Document CSP").clone());
        let mut allowed = true;
        let mut details = TrustedTypePolicyDetails::Allowed;
        let mut violations = Vec::new();
        for (index, policy) in state.Policies(world).iter().enumerate() {
            let Some(directive) = &policy.trusted_types else {
                continue;
            };
            let next = directive.Allows(name, duplicate);
            if next == TrustedTypePolicyDetails::Allowed {
                continue;
            }
            let next_allowed = policy.disposition == CSPHeaderType::ReportOnly;
            if (allowed && !next_allowed) || details == TrustedTypePolicyDetails::Allowed {
                details = next;
            }
            allowed &= next_allowed;
            let console_name = String::from_utf16_lossy(name); // Console projection only; policy comparison/sample retain UTF-16.
            let raw = format!("trusted-types {}", directive.raw);
            let message = if next == TrustedTypePolicyDetails::DisallowedDuplicateName {
                format!("Creating a TrustedTypePolicy named '{console_name}' violates the following Content Security Policy directive because a TrustedTypePolicy with that name already exists and the directive does not contain'allow-duplicates': \"{raw}\".")
            } else {
                format!("Creating a TrustedTypePolicy named '{console_name}' violates the following Content Security policy directive: \"{raw}\".")
            };
            violations.push(state.TrustedTypesViolation(
                policy,
                index,
                world,
                CSPDirective::TrustedTypes,
                &raw,
                &message,
                sample(name).to_vec(),
                location.clone(),
            ));
        }
        self.QueueLocalViolations(violations);
        (allowed, details)
    }
    pub fn AllowTrustedTypeAssignmentFailure(
        &self,
        message: &str,
        value: &[u16],
        prefix: &str,
        world: CSPWorld,
        location: CSPSourceLocation,
    ) -> bool {
        let state = PolicySnapshot(self.compiled.read().expect("Document CSP").clone());
        let mut allowed = true;
        let mut violations = Vec::new();
        for (index, policy) in state.Policies(world).iter().enumerate() {
            if !policy.require_trusted_types_for_script {
                continue;
            }
            allowed &= policy.disposition == CSPHeaderType::ReportOnly;
            let mut text = sample(&prefix.encode_utf16().collect::<Vec<_>>()).to_vec();
            if !prefix.is_empty() {
                text.push(b'|' as u16);
            }
            text.extend_from_slice(sample(value));
            violations.push(state.TrustedTypesViolation(
                policy,
                index,
                world,
                CSPDirective::RequireTrustedTypesFor,
                "require-trusted-types-for",
                message,
                text,
                location.clone(),
            ));
        }
        self.QueueLocalViolations(violations);
        allowed
    }
}
impl PolicySnapshot {
    fn TrustedTypesViolation(
        &self,
        policy: &Policy,
        index: usize,
        world: CSPWorld,
        directive: CSPDirective,
        raw: &str,
        message: &str,
        sample: Vec<u16>,
        source_location: CSPSourceLocation,
    ) -> CSPViolation {
        let report_only = policy.disposition == CSPHeaderType::ReportOnly;
        CSPViolation {
            disposition: policy.disposition,
            effective_directive: directive,
            violated_directive: directive,
            directive_text: raw.into(),
            original_policy: policy.original.clone(),
            document_url: strip_url_for_report(&self.0.document_url),
            report_url: self.0.document_url.to_string(),
            blocked_url: if directive == CSPDirective::TrustedTypes {
                "trusted-types-policy"
            } else {
                "trusted-types-sink"
            }
            .into(),
            url_before_redirects: String::new(),
            report_endpoints: policy.endpoints.clone(),
            use_reporting_api: policy.reporting_api,
            console_message: format!(
                "{message}{}",
                if report_only {
                    " The policy is report-only, so the violation has been logged but no further action has been taken."
                } else {
                    " The action has been blocked."
                }
            ),
            console_level: if report_only {
                CSPConsoleLevel::Info
            } else {
                CSPConsoleLevel::Error
            },
            source_location,
            status_code: self.0.status_code,
            referrer: self.0.referrer.clone(),
            world,
            blocked_reason: if report_only {
                None
            } else {
                Some(CSPBlockedReason::ContentSecurityPolicy {
                    policy_index: index,
                    directive,
                })
            },
            sample,
        }
    }
}

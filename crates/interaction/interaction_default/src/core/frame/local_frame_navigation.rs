// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/frame/local_frame.h:501;
// core/frame/local_frame.cc:2162-2398.
//! LocalFrame::CanNavigate over canonical FrameTree and SecurityOrigin owners.
//! The embedding resolves the target and lends live frame/security facts; this
//! source helper owns the security decision and emits its ordered diagnostics.
use foundation::security::{
    CanAccessSecurityOrigin, DomainAndRegistry, OriginKey, SandboxFlags, SecurityOriginOwner,
};
use std::rc::Rc;

/// Facts from the same canonical KURL and SecurityOrigin::Create destination.
/// The source gate must not reparse a document URL to infer its current origin.
pub struct NavigationDestination<'url> {
    pub origin: Rc<dyn SecurityOriginOwner>,
    pub protocol: &'url str,
    pub host: &'url str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationBlockedReason {
    RedirectWithNoUserGestureSandbox,
    RedirectWithNoUserGesture,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationError {
    JavaScriptRequiresSameOrigin,
    SandboxedAncestor { in_fenced_frame_tree: bool },
    SandboxedPopup,
    SandboxedTopWithoutPermission,
    SandboxedTopWithoutUserActivation,
    SandboxedTopPrivilegeEscalation,
    TopWithoutUserGesture,
    UnrelatedFrame,
}
impl NavigationError {
    /// Source PrintNavigationErrorMessage's reason text. The host supplies the
    /// source/target URL prefix and console/reporting owner, never a decision.
    pub fn Message(self) -> &'static str {
        match self {
            Self::JavaScriptRequiresSameOrigin => "The frame attempting navigation must be same-origin with the target if navigating to a javascript: url",
            Self::SandboxedAncestor { in_fenced_frame_tree: true } => "The frame attempting navigation is in a fenced frame tree, and is therefore disallowed from navigating its ancestors.",
            Self::SandboxedAncestor { in_fenced_frame_tree: false } => "The frame attempting navigation is sandboxed, and is therefore disallowed from navigating its ancestors.",
            Self::SandboxedPopup => "The frame attempting navigation is sandboxed and is trying to navigate a popup, but is not the popup's opener and is not set to propagate sandboxing to popups.",
            Self::SandboxedTopWithoutPermission => "The frame attempting navigation of the top-level window is sandboxed, but the flag of 'allow-top-navigation' or 'allow-top-navigation-by-user-activation' is not set.",
            Self::SandboxedTopWithoutUserActivation => "The frame attempting navigation of the top-level window is sandboxed with the 'allow-top-navigation-by-user-activation' flag, but has no user activation (aka gesture). See https://www.chromestatus.com/feature/5629582019395584.",
            Self::SandboxedTopPrivilegeEscalation => "The frame attempting to navigate the top-level window is cross-origin and either it or one of its ancestors is not allowed to navigate the top frame.\n",
            Self::TopWithoutUserGesture => "The frame attempting navigation is targeting its top-level window, but is neither same-origin with its target nor has it received a user gesture. See https://www.chromestatus.com/feature/5851021045661696.",
            Self::UnrelatedFrame => "The frame attempting navigation is neither same-origin with the target, nor is it the target's parent or opener.",
        }
    }
}

/// All leaves are required. The binding belongs to one synchronous source
/// navigation check; missing frame/security/policy owners must be rejected by
/// its constructor, never replaced with a permissive answer here. Frame values
/// are canonical identities, not a copied frame tree or a URL-based relation.
pub trait LocalFrameNavigationContext {
    type Frame: Clone + Eq;
    fn SourceFrame(&self) -> Self::Frame;
    fn Parent(&self, frame: &Self::Frame) -> Option<Self::Frame>;
    fn Opener(&self, frame: &Self::Frame) -> Option<Self::Frame>;
    fn IsInSamePage(&self, a: &Self::Frame, b: &Self::Frame) -> bool;
    fn SecurityOrigin(&self, frame: &Self::Frame) -> Rc<dyn SecurityOriginOwner>;
    fn IsMainFrame(&self, frame: &Self::Frame) -> bool;
    fn IsPageMainFrameFencedFrameRoot(&self, frame: &Self::Frame) -> bool;
    fn IsInFencedFrameTree(&self) -> bool;
    fn SandboxFlags(&self) -> SandboxFlags;
    fn HasTransientUserActivation(&self) -> bool;
    fn HasStickyUserActivation(&self) -> bool;
    fn CanNavigateTopWithoutUserGesture(&self) -> bool;
    fn AllowPopup(&self) -> bool;
    fn CountOpenerNavigationWithoutGesture(&self);
    fn ReportNavigationError(&self, target: &Self::Frame, reason: NavigationError);
    fn DidBlockNavigation(&self, reason: NavigationBlockedReason);
}

// Source: local_frame.cc:2162-2189. The file-origin exception intentionally
// remains separate from SecurityOrigin::CanAccess, as it does in Chromium.
pub fn CanAccessAncestor<C: LocalFrameNavigationContext>(
    context: &C,
    active_origin: &dyn SecurityOriginOwner,
    target: Option<C::Frame>,
) -> bool {
    let Some(target) = target else {
        return false;
    };
    let local_active_origin = active_origin.SecurityOriginFacts().is_local;
    let mut ancestor = Some(target);
    while let Some(frame) = ancestor {
        let origin = context.SecurityOrigin(&frame);
        if CanAccessSecurityOrigin(active_origin, origin.as_ref())
            || local_active_origin && origin.SecurityOriginFacts().is_local
        {
            return true;
        }
        ancestor = context.Parent(&frame);
    }
    false
}
fn Top<C: LocalFrameNavigationContext>(context: &C, frame: &C::Frame) -> C::Frame {
    let mut frame = frame.clone();
    while let Some(parent) = context.Parent(&frame) {
        frame = parent;
    }
    frame
}
fn IsDescendantOf<C: LocalFrameNavigationContext>(
    context: &C,
    frame: &C::Frame,
    ancestor: &C::Frame,
) -> bool {
    // Source: core/page/frame_tree.cc:385-396.
    if !context.IsInSamePage(frame, ancestor) {
        return false;
    }
    let mut parent = Some(frame.clone());
    while let Some(frame) = parent {
        if frame == *ancestor {
            return true;
        }
        parent = context.Parent(&frame);
    }
    false
}

// Source: local_frame.cc:2191-2398.
pub fn CanNavigate<C: LocalFrameNavigationContext>(
    context: &C,
    target: &C::Frame,
    destination: &NavigationDestination<'_>,
) -> bool {
    let source = context.SourceFrame();
    if *target == source {
        return true;
    }
    if context.Opener(&source).as_ref() == Some(target)
        && !context.HasTransientUserActivation()
        && !CanAccessSecurityOrigin(
            context.SecurityOrigin(target).as_ref(),
            destination.origin.as_ref(),
        )
    {
        context.CountOpenerNavigationWithoutGesture();
    }
    if destination.protocol.eq_ignore_ascii_case("javascript")
        && !CanAccessSecurityOrigin(
            context.SecurityOrigin(&source).as_ref(),
            context.SecurityOrigin(target).as_ref(),
        )
    {
        context.ReportNavigationError(target, NavigationError::JavaScriptRequiresSameOrigin);
        return false;
    }
    let sandbox = context.SandboxFlags();
    if sandbox.Contains(SandboxFlags::NAVIGATION) {
        let target_is_outermost_frame =
            context.IsMainFrame(target) && !context.IsPageMainFrameFencedFrameRoot(target);
        if !IsDescendantOf(context, target, &source) && !target_is_outermost_frame {
            context.ReportNavigationError(
                target,
                NavigationError::SandboxedAncestor {
                    in_fenced_frame_tree: context.IsInFencedFrameTree(),
                },
            );
            return false;
        }
        if target_is_outermost_frame
            && *target != Top(context, &source)
            && sandbox.Contains(SandboxFlags::PROPAGATES_TO_AUXILIARY)
            && (sandbox.Contains(SandboxFlags::POPUPS)
                || context.Opener(target).as_ref() != Some(&source))
        {
            context.ReportNavigationError(target, NavigationError::SandboxedPopup);
            return false;
        }
        if *target == Top(context, &source) && !context.IsPageMainFrameFencedFrameRoot(target) {
            if sandbox.Contains(SandboxFlags::TOP_NAVIGATION)
                && sandbox.Contains(SandboxFlags::TOP_NAVIGATION_BY_USER_ACTIVATION)
            {
                context
                    .ReportNavigationError(target, NavigationError::SandboxedTopWithoutPermission);
                return false;
            }
            if sandbox.Contains(SandboxFlags::TOP_NAVIGATION)
                && !sandbox.Contains(SandboxFlags::TOP_NAVIGATION_BY_USER_ACTIVATION)
                && !context.HasTransientUserActivation()
            {
                context
                    .DidBlockNavigation(NavigationBlockedReason::RedirectWithNoUserGestureSandbox);
                context.ReportNavigationError(
                    target,
                    NavigationError::SandboxedTopWithoutUserActivation,
                );
                return false;
            }
            if !context.CanNavigateTopWithoutUserGesture() && !context.HasStickyUserActivation() {
                context.ReportNavigationError(
                    target,
                    NavigationError::SandboxedTopPrivilegeEscalation,
                );
                return false;
            }
            return true;
        }
    }
    let origin = context.SecurityOrigin(&source);
    if CanAccessAncestor(context, origin.as_ref(), Some(target.clone())) {
        return true;
    }
    if context.Parent(target).is_none() {
        if context.Opener(&source).as_ref() == Some(target) {
            return true;
        }
        if CanAccessAncestor(context, origin.as_ref(), context.Opener(target)) {
            return true;
        }
    }
    if *target == Top(context, &source) {
        let target_origin = context.SecurityOrigin(target);
        if context.HasStickyUserActivation()
            || CanAccessSecurityOrigin(target_origin.as_ref(), destination.origin.as_ref())
        {
            return true;
        }
        let access = target_origin.SecurityOriginAccess();
        let (target_protocol, target_host) = match &access.origin {
            OriginKey::Tuple { scheme, host, .. } => (scheme.as_str(), host.as_str()),
            OriginKey::Opaque(_) => ("", ""),
        };
        let target_domain =
            DomainAndRegistry(access.domain_override.as_deref().unwrap_or(target_host));
        let destination_domain = DomainAndRegistry(destination.host);
        if target_domain.is_some()
            && target_domain == destination_domain
            && target_protocol == destination.protocol
        {
            return true;
        }
        if context.AllowPopup() {
            return true;
        }
        context.ReportNavigationError(target, NavigationError::TopWithoutUserGesture);
        context.DidBlockNavigation(NavigationBlockedReason::RedirectWithNoUserGesture);
    } else {
        context.ReportNavigationError(target, NavigationError::UnrelatedFrame);
    }
    false
}

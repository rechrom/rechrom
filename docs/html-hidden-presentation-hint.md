# HTML hidden presentation hint

## Source-owned behavior

Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`:

- [HTMLElement::IsPresentationAttribute and CollectStyleForPresentationAttribute](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/html_element.cc#L354-L420) identify unqualified HTML hidden attributes as presentation hints. Exact ASCII-case-insensitive `until-found` supplies `content-visibility:hidden`; all other present values supply `display:none`
- [StyleResolver::MatchPresentationalHints](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/css/resolver/style_resolver.cc) admits those values at author-presentational-hint origin
- [HTML UA slot rule](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/html/resources/html.css#L2003-L2005) sets `display:contents`; it is lower priority than the presentation hint. Author rules and inline declarations may override the hint
- [LayoutTreeBuilderForElement](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/dom/layout_tree_builder.cc) consumes canonical computed style. Slot assignment remains DOM state even when its rendered subtree is suppressed

## Rechrom ownership

`ProductionRuleMatchingHost::PresentationAttributeStyle` completes the missing
HTML hidden branch and returns the existing typed property-set owner. The
translated `MatchPresentationalHints`, cascade, native `ComputedStyle`, style
impact and layout attachment pipeline consume it unchanged. No UA selector,
page selector, flat-tree override, extra DOM owner, or post-cascade fixup is used.
Unqualified attribute lookup is explicit because the DOM convenience
`FindAttribute` compares local names without namespace identity.

Existing DOM attribute mutations invalidate style. Native `display:none` becomes
`generates_box=false, display_contents=false`; existing authoritative flat-tree
attachment skips descendants. Removing hidden restores normal style and
attachment. Old immutable fragment snapshots are preserved.

## Verification

`browser/tests/hidden_presentation_hint.rs` reproduces the original failure with
HTML-native shadow slots and validates repeated close/open, slotted/fallback
content, ordinary HTML descendants, author overrides, unchanged assignment and
native input state, and immutable old frames. `document_style_engine_test`
checks the native canonical style, author origin, namespace boundaries,
until-found semantics and mutation invalidation.

## Bounds

This fixes the hidden presentation hint in the admitted production resolver path.
It does not complete all HTML presentation attributes, alter the existing
compatibility fallback resolver, implement display-lock/find-in-page behavior
for content-visibility, or complete HTMLElement.hidden's union IDL semantics.
The existing CSSOM display projection cannot distinguish display:contents from
display:none; canonical native assertions and geometry are the verification
oracle here. Fragment namespaces and declarative shadow parsing are separate
known gaps. No claim of full Chromium visual parity follows from this slice.

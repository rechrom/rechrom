// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/loader/navigation_policy.h:38-47
// Source: core/loader/navigation_policy.cc:51-92,136-163

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationPolicy {
    kNavigationPolicyDownload,
    kNavigationPolicyCurrentTab,
    kNavigationPolicyNewBackgroundTab,
    kNavigationPolicyNewForegroundTab,
    kNavigationPolicyNewWindow,
    kNavigationPolicyNewPopup,
    kNavigationPolicyPictureInPicture,
    kNavigationPolicySplitView,
}

/// Source DynamicTo<MouseEvent>/KeyboardEvent/GestureEvent scalar projection.
/// Keyboard/gesture simulated clicks supply button 0. None is a failed cast.
#[derive(Clone, Copy, Default)]
pub struct NavigationEventModifiers {
    pub button: i16,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

// Source: navigation_policy.cc:95-134. A native MouseUp, Enter keyboard,
// or gesture may supply intent to a simulated click; trust alone cannot.
pub fn CurrentInputNavigationModifiers(
    event: &dyn interaction_input::public::common::input::web_input_event::WebInputEventPayload,
) -> Option<NavigationEventModifiers> {
    use interaction_input::public::common::input::{
        web_input_event::{Type, WebInputEvent},
        web_keyboard_event::WebKeyboardEvent,
        web_mouse_event::WebMouseEvent,
        web_pointer_properties::Button,
    };
    let base = event.Event();
    let button = if base.GetType() == Type::kMouseUp {
        match event
            .AsAny()
            .downcast_ref::<WebMouseEvent>()?
            .pointer_properties
            .button
        {
            Button::kLeft => 0,
            Button::kMiddle => 1,
            Button::kRight => 2,
            _ => return None,
        }
    } else if (WebInputEvent::IsKeyboardEventType(base.GetType())
        && event
            .AsAny()
            .downcast_ref::<WebKeyboardEvent>()
            .is_some_and(|key| key.windows_key_code == 13))
        || WebInputEvent::IsGestureEventType(base.GetType())
    {
        0
    } else {
        return None;
    };
    let modifiers = base.GetModifiers();
    Some(NavigationEventModifiers {
        button,
        ctrl: modifiers & WebInputEvent::kControlKey != 0,
        shift: modifiers & WebInputEvent::kShiftKey != 0,
        alt: modifiers & WebInputEvent::kAltKey != 0,
        meta: modifiers & WebInputEvent::kMetaKey != 0,
    })
}

pub fn NavigationPolicyFromEventModifiers(
    event: Option<NavigationEventModifiers>,
    is_mac: bool,
) -> NavigationPolicy {
    use NavigationPolicy::*;
    let Some(event) = event else {
        return kNavigationPolicyCurrentTab;
    };
    let new_tab_modifier = event.button == 1 || if is_mac { event.meta } else { event.ctrl };
    if new_tab_modifier && event.alt && !event.shift {
        return kNavigationPolicySplitView;
    }
    if !new_tab_modifier && !event.shift && !event.alt {
        kNavigationPolicyCurrentTab
    } else if new_tab_modifier {
        if event.shift {
            kNavigationPolicyNewForegroundTab
        } else {
            kNavigationPolicyNewBackgroundTab
        }
    } else if event.shift {
        kNavigationPolicyNewWindow
    } else {
        kNavigationPolicyDownload
    }
}

pub fn NavigationPolicyFromEvent(
    event: Option<NavigationEventModifiers>,
    current_input: Option<NavigationEventModifiers>,
    is_mac: bool,
    new_tab_modifier_set_from_isolated_world: bool,
) -> NavigationPolicy {
    use NavigationPolicy::*;
    let event_policy = NavigationPolicyFromEventModifiers(event, is_mac);
    let input_policy = NavigationPolicyFromEventModifiers(current_input, is_mac);
    if event_policy == kNavigationPolicyDownload && input_policy != kNavigationPolicyDownload {
        return kNavigationPolicyCurrentTab;
    }
    if event_policy == kNavigationPolicyNewBackgroundTab
        && input_policy != kNavigationPolicyNewBackgroundTab
        && !new_tab_modifier_set_from_isolated_world
    {
        return kNavigationPolicyNewForegroundTab;
    }
    if event_policy == kNavigationPolicySplitView && input_policy != kNavigationPolicySplitView {
        return kNavigationPolicyCurrentTab;
    }
    event_policy
}

// Copyright 2020 The Chromium Authors
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/public/mojom/frame/user_activation_notification_type.mojom:8-76
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum UserActivationNotificationType {
    kNone,
    kInteraction,
    kDevTools,
    kExtensionEvent,
    kExtensionGuestView,
    kExtensionMessaging,
    kMedia,
    kFileSystemAccess,
    kPlugin,
    kTest,
    kWebScriptExec,
    kVoiceSearch,
    kExtensionMessagingBothPrivileged,
    kExtensionMessagingSenderPrivileged,
    kExtensionMessagingReceiverPrivileged,
    kExtensionMessagingNeitherPrivileged,
    kActorWebMCP,
}

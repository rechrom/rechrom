// Embedding leaf for ui::GetCurrentKeyModifiers' AppKit source. Query current
// native flags instead of NSApp.currentEvent, which can describe an older event.
#import <AppKit/AppKit.h>

uint32_t rechrom_current_keyboard_modifiers(void) {
    NSEventModifierFlags flags = NSEvent.modifierFlags;
    return ((flags & NSEventModifierFlagCapsLock) ? 1u : 0u)
        | ((flags & NSEventModifierFlagShift) ? 2u : 0u)
        | ((flags & NSEventModifierFlagControl) ? 4u : 0u)
        | ((flags & NSEventModifierFlagOption) ? 8u : 0u)
        | ((flags & NSEventModifierFlagCommand) ? 16u : 0u);
}

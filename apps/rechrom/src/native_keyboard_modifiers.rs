//! The native UI sequence publishes keyboard facts; document executors only
//! read an immutable snapshot. Lock state comes from the platform keyboard
//! owner, never from the last key code or a browser-side toggle.
use interaction::core::input::keyboard_event_manager::NativeKeyboardModifiers;
use std::sync::atomic::{AtomicU8, Ordering};
use winit::event::WindowEvent;

const AVAILABLE: u8 = 1 << 7;
static CURRENT: AtomicU8 = AtomicU8::new(0);

pub(crate) fn current() -> Result<NativeKeyboardModifiers, &'static str> {
    let flags = CURRENT.load(Ordering::Acquire);
    if flags & AVAILABLE == 0 {
        return Err("native keyboard modifiers");
    }
    Ok(NativeKeyboardModifiers {
        alpha_lock: flags & 1 != 0,
        shift: flags & 2 != 0,
        control: flags & 4 != 0,
        option: flags & 8 != 0,
        command: flags & 16 != 0,
    })
}

/// Called only by the embedding's native UI sequence, before forwarding input.
pub(crate) fn update(event: Option<&WindowEvent>) {
    #[cfg(target_os = "macos")]
    {
        extern "C" {
            fn rechrom_current_keyboard_modifiers() -> u32;
        }
        let _ = event;
        let flags = unsafe { rechrom_current_keyboard_modifiers() } as u8;
        CURRENT.store(AVAILABLE | flags, Ordering::Release);
    }
    // Windows: Caps Lock is queried on the native message sequence for every
    // event winit derives from a message that refreshes its modifier state
    // (key, mouse move/button/wheel, focus gain). GetKeyState follows this
    // thread's keyboard message queue, which the document executor does not
    // pump. A focus loss leaves the state unavailable until the next refresh.
    #[cfg(target_os = "windows")]
    {
        #[link(name = "user32")]
        extern "system" {
            fn GetKeyState(key: i32) -> i16;
        }
        const VK_CAPITAL: i32 = 0x14;
        let modifier_flags = match event {
            Some(WindowEvent::ModifiersChanged(modifiers)) => {
                let state = modifiers.state();
                u8::from(state.shift_key()) << 1
                    | u8::from(state.control_key()) << 2
                    | u8::from(state.alt_key()) << 3
                    | u8::from(state.super_key()) << 4
            }
            Some(WindowEvent::Focused(false)) => {
                CURRENT.store(0, Ordering::Release);
                return;
            }
            Some(
                WindowEvent::Focused(true)
                | WindowEvent::KeyboardInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. },
            ) => CURRENT.load(Ordering::Acquire) & 0b1_1110,
            _ => return,
        };
        let alpha_lock = unsafe { GetKeyState(VK_CAPITAL) } & 1 != 0;
        CURRENT.store(
            AVAILABLE | modifier_flags | u8::from(alpha_lock),
            Ordering::Release,
        );
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    match event {
        Some(WindowEvent::ModifiersChanged(modifiers)) => {
            let Some(alpha_lock) = modifiers.caps_lock_state() else {
                CURRENT.store(0, Ordering::Release);
                return;
            };
            let state = modifiers.state();
            let flags = u8::from(alpha_lock)
                | u8::from(state.shift_key()) << 1
                | u8::from(state.control_key()) << 2
                | u8::from(state.alt_key()) << 3
                | u8::from(state.super_key()) << 4;
            CURRENT.store(AVAILABLE | flags, Ordering::Release);
        }
        // X11 and Wayland can only supply state while their keyboard owner is
        // focused. A focus reset is not evidence that Caps Lock is switched off.
        Some(WindowEvent::Focused(false)) => CURRENT.store(0, Ordering::Release),
        _ => {}
    }
}

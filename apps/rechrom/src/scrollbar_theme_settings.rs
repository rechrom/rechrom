//! Host NSUserDefaults inputs for Blink's process scrollbar theme.
//! Source content/browser/theme_helper_mac.mm:31-49,138-178;
//! render_thread_impl.cc:1385-1391; scrollbar_theme_mac.cc:384-391.
//! The observer only republishes optional scalar preferences; all scrollbar
//! decisions and timer state stay with their source owners.
use std::io;
pub(crate) fn initialize() -> io::Result<()> {
    #[cfg(target_os = "macos")]
    unsafe {
        mac::initialize()?;
    }
    Ok(())
}
#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use std::{
        ffi::{c_char, c_void, CStr},
        sync::OnceLock,
    };
    type Id = *mut c_void;
    type Sel = *mut c_void;
    #[link(name = "Foundation", kind = "framework")]
    extern "C" {}
    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn objc_allocateClassPair(superclass: Id, name: *const c_char, extra: usize) -> Id;
        fn objc_registerClassPair(class: Id);
        fn sel_registerName(name: *const c_char) -> Sel;
        fn class_addMethod(
            class: Id,
            name: Sel,
            implementation: *const c_void,
            types: *const c_char,
        ) -> bool;
        fn objc_msgSend();
    }
    unsafe fn sel(name: &CStr) -> Sel {
        sel_registerName(name.as_ptr())
    }
    unsafe fn id(receiver: Id, name: &CStr) -> Id {
        let call: unsafe extern "C" fn(Id, Sel) -> Id =
            std::mem::transmute(objc_msgSend as *const ());
        call(receiver, sel(name))
    }
    unsafe fn string(text: &CStr) -> Id {
        let call: unsafe extern "C" fn(Id, Sel, *const c_char) -> Id =
            std::mem::transmute(objc_msgSend as *const ());
        call(
            objc_getClass(c"NSString".as_ptr()),
            sel(c"stringWithUTF8String:"),
            text.as_ptr(),
        )
    }
    struct Pool(Id);
    impl Pool {
        unsafe fn new() -> Self {
            Self(id(objc_getClass(c"NSAutoreleasePool".as_ptr()), c"new"))
        }
    }
    impl Drop for Pool {
        fn drop(&mut self) {
            unsafe {
                let call: unsafe extern "C" fn(Id, Sel) =
                    std::mem::transmute(objc_msgSend as *const ());
                call(self.0, sel(c"drain"));
            }
        }
    }
    unsafe fn preference(defaults: Id, key: &CStr) -> Option<f32> {
        let key = string(key);
        let object: unsafe extern "C" fn(Id, Sel, Id) -> Id =
            std::mem::transmute(objc_msgSend as *const ());
        if object(defaults, sel(c"objectForKey:"), key).is_null() {
            return None;
        }
        let number: unsafe extern "C" fn(Id, Sel, Id) -> f32 =
            std::mem::transmute(objc_msgSend as *const ());
        Some(number(defaults, sel(c"floatForKey:"), key))
    }
    unsafe fn refresh() {
        let _pool = Pool::new();
        let defaults = id(
            objc_getClass(c"NSUserDefaults".as_ptr()),
            c"standardUserDefaults",
        );
        let synchronize: unsafe extern "C" fn(Id, Sel) -> i8 =
            std::mem::transmute(objc_msgSend as *const ());
        let _ = synchronize(defaults, sel(c"synchronize"));
        // objectForKey preserves absent vs explicitly configured zero. NS
        // defaults removed these registered defaults in macOS 10.15, just as
        // FillScrollbarThemeParams documents in the source.
        let initial = preference(defaults, c"NSScrollerButtonDelay");
        let repeat = preference(defaults, c"NSScrollerButtonPeriod");
        let boolean: unsafe extern "C" fn(Id, Sel, Id) -> i8 =
            std::mem::transmute(objc_msgSend as *const ());
        foundation::scrollbar_theme_settings::ScrollbarThemeSettings::SetMacJumpOnTrackClick(
            boolean(
                defaults,
                sel(c"boolForKey:"),
                string(c"AppleScrollerPagingBehavior"),
            ) != 0,
        );
        foundation::scrollbar_theme_settings::ScrollbarThemeSettings::UpdateMacAutoscrollButtonDelays(initial,repeat);
    }
    unsafe extern "C" fn preferences_changed(_: Id, _: Sel, _: Id) {
        refresh();
    }
    pub(super) unsafe fn initialize() -> io::Result<()> {
        static OBSERVER: OnceLock<usize> = OnceLock::new();
        if OBSERVER.get().is_some() {
            return Ok(());
        }
        let _pool = Pool::new();
        let base = objc_getClass(c"NSObject".as_ptr());
        let class = objc_allocateClassPair(base, c"RechromScrollbarThemeObserver".as_ptr(), 0);
        if class.is_null() {
            return Err(io::Error::other("cannot allocate scrollbar theme observer"));
        }
        let selector = sel(c"preferencesChanged:");
        if !class_addMethod(
            class,
            selector,
            preferences_changed as *const () as *const c_void,
            c"v@:@".as_ptr(),
        ) {
            return Err(io::Error::other(
                "cannot register scrollbar theme observer method",
            ));
        }
        objc_registerClassPair(class);
        let observer = id(class, c"new");
        if observer.is_null() {
            return Err(io::Error::other("cannot create scrollbar theme observer"));
        }
        let center = id(
            objc_getClass(c"NSDistributedNotificationCenter".as_ptr()),
            c"defaultCenter",
        );
        let add: unsafe extern "C" fn(Id, Sel, Id, Sel, Id, Id, usize) =
            std::mem::transmute(objc_msgSend as *const ());
        // Same process-lifetime native observer as ThemeHelperMac. Source
        // notification delivery is on the application's native main run loop.
        for (name, suspension) in [
            (c"AppleAquaScrollBarVariantChanged", 4),
            (c"AppleNoRedisplayAppearancePreferenceChanged", 2),
            (c"NSScrollAnimationEnabled", 2),
            (c"AppleScrollBarVariant", 4),
            (c"NSScrollViewRubberbanding", 4),
        ] {
            add(
                center,
                sel(c"addObserver:selector:name:object:suspensionBehavior:"),
                observer,
                selector,
                string(name),
                std::ptr::null_mut(),
                suspension,
            );
        }
        let _ = OBSERVER.set(observer as usize);
        refresh();
        Ok(())
    }
}

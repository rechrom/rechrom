// cpp: layoutng/internal/scrollbar_mode.h:7-17
pub mod mojom {
    pub mod blink {
        #[repr(i32)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum ScrollbarMode {
            kAuto = 0,
            kAlwaysOff = 1,
            kAlwaysOn = 2,
        }
    }
}

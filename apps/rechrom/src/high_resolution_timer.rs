//! Process timer resolution, corresponding to Chromium's
//! `base::HighResolutionTimerManager` (base/timer/hi_res_timer_manager_win.cc,
//! base/timer/hi_res_timer_manager_posix.cc) and
//! `base::Time::ActivateHighResolutionTimer` (base/time/time_win.cc).
//!
//! On Windows, waits and timers round up to the system clock tick (15.6 ms by
//! default). Chromium raises the process timer period to
//! `kMinTimerIntervalHighResMs` (1 ms) while on AC power so that frame-rate
//! wake-ups such as the BeginFrame time source fire on time. Without it the
//! headless/timer BeginFrame source and the threads it wakes deliver frames
//! several milliseconds late and miss their deadlines. POSIX hosts need no
//! action: their timers are already high resolution.

/// Keeps the high-resolution timer active for its lifetime.
pub(crate) struct HighResolutionTimer {
    #[cfg(target_os = "windows")]
    active: bool,
}

impl HighResolutionTimer {
    pub(crate) fn activate() -> Self {
        #[cfg(target_os = "windows")]
        {
            // UseHiResClock(!PowerMonitor::IsOnBatteryPower())
            let active = !windows::is_on_battery_power() && windows::begin_period();
            Self { active }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {}
        }
    }
}

impl Drop for HighResolutionTimer {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if self.active {
            windows::end_period();
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    // Source: base/time/time_win.cc kMinTimerIntervalHighResMs.
    const MIN_TIMER_INTERVAL_HIGH_RES_MS: u32 = 1;
    const TIMERR_NOERROR: u32 = 0;
    // SYSTEM_POWER_STATUS::ACLineStatus
    const AC_LINE_OFFLINE: u8 = 0;

    #[repr(C)]
    struct SystemPowerStatus {
        ac_line_status: u8,
        battery_flag: u8,
        battery_life_percent: u8,
        system_status_flag: u8,
        battery_life_time: u32,
        battery_full_life_time: u32,
    }

    #[link(name = "winmm")]
    extern "system" {
        fn timeBeginPeriod(period: u32) -> u32;
        fn timeEndPeriod(period: u32) -> u32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemPowerStatus(status: *mut SystemPowerStatus) -> i32;
    }

    // Source: base/power_monitor/power_monitor_device_source_win.cc
    // PowerMonitorDeviceSource::IsOnBatteryPower.
    pub(super) fn is_on_battery_power() -> bool {
        let mut status = SystemPowerStatus {
            ac_line_status: 0,
            battery_flag: 0,
            battery_life_percent: 0,
            system_status_flag: 0,
            battery_life_time: 0,
            battery_full_life_time: 0,
        };
        let known = unsafe { GetSystemPowerStatus(&mut status) } != 0;
        known && status.ac_line_status == AC_LINE_OFFLINE
    }

    pub(super) fn begin_period() -> bool {
        unsafe { timeBeginPeriod(MIN_TIMER_INTERVAL_HIGH_RES_MS) == TIMERR_NOERROR }
    }

    pub(super) fn end_period() {
        unsafe { timeEndPeriod(MIN_TIMER_INTERVAL_HIGH_RES_MS) };
    }
}

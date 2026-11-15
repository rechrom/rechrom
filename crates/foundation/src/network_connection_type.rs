//! Chromium public/platform/web_connection_type.h:37-50 and
//! NetworkStateNotifier::IsCellularConnectionType h:203-219 (6c1d401f).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebConnectionType {
    Cellular2G,
    Cellular3G,
    Cellular4G,
    Bluetooth,
    Ethernet,
    Wifi,
    Wimax,
    Other,
    None,
    Unknown,
}
impl WebConnectionType {
    pub fn IsCellular(self) -> bool {
        matches!(self, Self::Cellular2G | Self::Cellular3G | Self::Cellular4G)
    }
}

/// NetworkStateNotifier's measured effective throughput class, distinct from
/// the physical connection type. Source web_effective_connection_type.h:40-49.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebEffectiveConnectionType {
    Unknown,
    Offline,
    Slow2G,
    Type2G,
    Type3G,
    Type4G,
}

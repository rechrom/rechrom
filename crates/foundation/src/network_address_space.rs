//! services/network/public/mojom/ip_address_space.mojom, fixed Chromium revision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IPAddressSpace {
    #[default]
    Unknown,
    Public,
    Local,
    Loopback,
}

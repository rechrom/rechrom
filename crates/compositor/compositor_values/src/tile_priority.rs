//! Source: cc/tiles/tile_priority.h:66-80 (TileMemoryLimitPolicy), a value of
//! GlobalStateThatImpactsTilePriority named by ManagedMemoryPolicy. Moved
//! from compositor_tile, which re-exports it.

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TileMemoryLimitPolicy {
    ALLOW_NOTHING = 0,
    ALLOW_ABSOLUTE_MINIMUM = 1,
    ALLOW_PREPAINT_ONLY = 2,
    ALLOW_ANYTHING = 3,
}

pub fn TileMemoryLimitPolicyToString(policy: TileMemoryLimitPolicy) -> &'static str {
    match policy {
        TileMemoryLimitPolicy::ALLOW_NOTHING => "ALLOW_NOTHING",
        TileMemoryLimitPolicy::ALLOW_ABSOLUTE_MINIMUM => "ALLOW_ABSOLUTE_MINIMUM",
        TileMemoryLimitPolicy::ALLOW_PREPAINT_ONLY => "ALLOW_PREPAINT_ONLY",
        TileMemoryLimitPolicy::ALLOW_ANYTHING => "ALLOW_ANYTHING",
    }
}

pub fn IsTileMemoryLimitPolicyMoreRestictive(
    policy1: TileMemoryLimitPolicy,
    policy2: TileMemoryLimitPolicy,
) -> bool {
    policy1 < policy2
}

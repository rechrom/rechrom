// cpp: layoutng/internal/break_appeal.h:15-34
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum BreakAppeal {
    kBreakAppealLastResort = 0,
    kBreakAppealViolatingBreakAvoid = 1,
    kBreakAppealViolatingOrphansAndWidows = 2,
    kBreakAppealPerfect = 3,
}

// cpp: layoutng/internal/break_appeal.h:15-34
impl TryFrom<u8> for BreakAppeal {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::kBreakAppealLastResort),
            1 => Ok(Self::kBreakAppealViolatingBreakAvoid),
            2 => Ok(Self::kBreakAppealViolatingOrphansAndWidows),
            3 => Ok(Self::kBreakAppealPerfect),
            _ => Err(()),
        }
    }
}

// C++ has an unscoped enum, so the variant is also available at namespace scope.
// cpp: layoutng/internal/break_appeal.h:33-33
#[allow(non_upper_case_globals)]
pub const kBreakAppealPerfect: BreakAppeal = BreakAppeal::kBreakAppealPerfect;

// cpp: layoutng/internal/break_appeal.h:37-37
#[allow(non_upper_case_globals)]
pub const kBreakAppealBitsNeeded: i32 = 2;

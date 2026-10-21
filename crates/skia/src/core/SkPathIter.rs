use crate::include::core::SkPathTypes::SkPathVerb;
use crate::src::core::SkPoint::SkPoint;

// Source: include/core/SkPathIter.h:21-30
pub struct Rec<'a> {
    pub fPoints: &'a [SkPoint],
    pub fConicWeight: f32,
    pub fVerb: SkPathVerb,
}
impl Rec<'_> {
    #[inline]
    pub fn conicWeight(&self) -> f32 {
        crate::SkASSERT!(@source b"include/core/SkPathIter.h\0".as_ptr(),27; @condition b"fVerb == SkPathVerb::kConic\0".as_ptr(); (self.fVerb)==(SkPathVerb::kConic));
        self.fConicWeight
    }
}

// Source: include/core/SkPathIter.h:32-42
impl<'a> SkPathIter<'a> {
    #[inline]
    pub fn new(pts: &'a [SkPoint], vbs: &'a [SkPathVerb], cns: &'a [f32]) -> Self {
        let mut result = Self {
            pIndex: 0,
            vIndex: 0,
            cIndex: 0,
            fPoints: pts,
            fVerbs: vbs,
            fConics: cns,
            fClosePointStorage: [SkPoint::make(0.0, 0.0); 2],
        };
        if !vbs.is_empty() && vbs.last() == Some(&SkPathVerb::kMove) {
            result.fVerbs = &vbs[..vbs.len() - 1];
        }
        result
    }
}

// Source: include/core/SkPathIter.h:54-59
impl SkPathIter<'_> {
    #[inline]
    pub fn peekNextVerb(&self) -> Option<SkPathVerb> {
        if self.vIndex < self.fVerbs.len() {
            return Some(self.fVerbs[self.vIndex]);
        }
        None
    }
}

// Source: include/core/SkPathIter.h:62-66
// Rust lending next() ties close-point storage to the mutable iterator borrow.
#[derive(Clone)]
pub struct SkPathIter<'a> {
    pIndex: usize,
    vIndex: usize,
    cIndex: usize,
    fPoints: &'a [SkPoint],
    fVerbs: &'a [SkPathVerb],
    fConics: &'a [f32],
    fClosePointStorage: [SkPoint; 2],
}

// Source: include/core/SkPathIter.h:71-75
pub struct ContourRec<'a> {
    pub fPoints: &'a [SkPoint],
    pub fVerbs: &'a [SkPathVerb],
    pub fConics: &'a [f32],
}

// Source: include/core/SkPathIter.h:77-80
impl<'a> SkPathContourIter<'a> {
    #[inline]
    pub fn new(pts: &'a [SkPoint], vbs: &'a [SkPathVerb], cns: &'a [f32]) -> Self {
        Self {
            fPoints: pts,
            fVerbs: vbs,
            fConics: cns,
        }
    }
}

// Source: include/core/SkPathIter.h:85-87
pub struct SkPathContourIter<'a> {
    fPoints: &'a [SkPoint],
    fVerbs: &'a [SkPathVerb],
    fConics: &'a [f32],
}

// Source: src/core/SkPathIter.cpp:17-46
impl SkPathIter<'_> {
    pub fn next(&mut self) -> Option<Rec<'_>> {
        if self.vIndex >= self.fVerbs.len() {
            return None;
        }
        let n;
        let mut w = -1.0;
        let v = self.fVerbs[self.vIndex];
        self.vIndex += 1;
        match v {
            SkPathVerb::kMove => {
                self.fClosePointStorage[1] = self.fPoints[self.pIndex];
                self.pIndex += 1;
                return Some(Rec {
                    fPoints: &self.fClosePointStorage[1..],
                    fConicWeight: w,
                    fVerb: v,
                });
            }
            SkPathVerb::kLine => n = 1,
            SkPathVerb::kQuad => n = 2,
            SkPathVerb::kConic => {
                n = 2;
                w = self.fConics[self.cIndex];
                self.cIndex += 1;
            }
            SkPathVerb::kCubic => n = 3,
            SkPathVerb::kClose => {
                debug_assert!(self.pIndex > 0);
                self.fClosePointStorage[0] = self.fPoints[self.pIndex - 1];
                return Some(Rec {
                    fPoints: &self.fClosePointStorage,
                    fConicWeight: w,
                    fVerb: v,
                });
            }
        }
        debug_assert!(self.pIndex > 0);
        let start = self.pIndex - 1;
        crate::SkASSERT!(@source b"src/core/SkPathIter.cpp\0".as_ptr(),42; @condition b"n >= 1 && n <= 3\0".as_ptr(); n >= 1 && n <= 3);
        self.pIndex += n;
        Some(Rec {
            fPoints: &self.fPoints[start..start + n + 1],
            fConicWeight: w,
            fVerb: v,
        })
    }
}

// Source: src/core/SkPathIter.cpp:50-79
impl<'a> SkPathContourIter<'a> {
    pub fn next(&mut self) -> Option<ContourRec<'a>> {
        if self.fVerbs.is_empty() {
            return None;
        }
        crate::SkASSERT!(@source b"src/core/SkPathIter.cpp\0".as_ptr(),55; @condition b"fVerbs[0] == SkPathVerb::kMove\0".as_ptr(); (self.fVerbs[0])==(SkPathVerb::kMove));
        let mut npts = 1;
        let mut nvbs = 1;
        let mut nws = 0;
        for i in 1..self.fVerbs.len() {
            match self.fVerbs[i] {
                SkPathVerb::kMove => break,
                SkPathVerb::kLine => npts += 1,
                SkPathVerb::kQuad => npts += 2,
                SkPathVerb::kConic => {
                    npts += 2;
                    nws += 1;
                }
                SkPathVerb::kCubic => npts += 3,
                SkPathVerb::kClose => {
                    nvbs += 1;
                    break;
                }
            }
            nvbs += 1;
        }
        let rec = ContourRec {
            fPoints: &self.fPoints[..npts],
            fVerbs: &self.fVerbs[..nvbs],
            fConics: &self.fConics[..nws],
        };
        self.fPoints = &self.fPoints[npts..];
        self.fVerbs = &self.fVerbs[nvbs..];
        self.fConics = &self.fConics[nws..];
        Some(rec)
    }
}

// Copyright 2017 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/iterators/text_iterator_behavior.{h,cc}.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextIteratorBehavior(u32);
impl TextIteratorBehavior {
    pub fn DoesNotBreakAtReplacedElement(self) -> bool {
        self.0 & (1 << 0) != 0
    }
    pub fn SetDoesNotBreakAtReplacedElement(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 0;
        } else {
            self.0 &= !(1 << 0);
        }
        self
    }
    pub fn EmitsCharactersBetweenAllVisiblePositions(self) -> bool {
        self.0 & (1 << 1) != 0
    }
    pub fn SetEmitsCharactersBetweenAllVisiblePositions(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 1;
        } else {
            self.0 &= !(1 << 1);
        }
        self
    }
    pub fn EmitsImageAltText(self) -> bool {
        self.0 & (1 << 2) != 0
    }
    pub fn SetEmitsImageAltText(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 2;
        } else {
            self.0 &= !(1 << 2);
        }
        self
    }
    pub fn EmitsSpaceForNbsp(self) -> bool {
        self.0 & (1 << 3) != 0
    }
    pub fn SetEmitsSpaceForNbsp(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 3;
        } else {
            self.0 &= !(1 << 3);
        }
        self
    }
    pub fn EmitsObjectReplacementCharacter(self) -> bool {
        self.0 & (1 << 4) != 0
    }
    pub fn SetEmitsObjectReplacementCharacter(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 4;
        } else {
            self.0 &= !(1 << 4);
        }
        self
    }
    pub fn EmitsOriginalText(self) -> bool {
        self.0 & (1 << 5) != 0
    }
    pub fn SetEmitsOriginalText(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 5;
        } else {
            self.0 &= !(1 << 5);
        }
        self
    }
    pub fn EmitsSmallXForTextSecurity(self) -> bool {
        self.0 & (1 << 6) != 0
    }
    pub fn SetEmitsSmallXForTextSecurity(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 6;
        } else {
            self.0 &= !(1 << 6);
        }
        self
    }
    pub fn EntersOpenShadowRoots(self) -> bool {
        self.0 & (1 << 7) != 0
    }
    pub fn SetEntersOpenShadowRoots(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 7;
        } else {
            self.0 &= !(1 << 7);
        }
        self
    }
    pub fn EntersTextControls(self) -> bool {
        self.0 & (1 << 8) != 0
    }
    pub fn SetEntersTextControls(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 8;
        } else {
            self.0 &= !(1 << 8);
        }
        self
    }
    pub fn ExcludeAutofilledValue(self) -> bool {
        self.0 & (1 << 9) != 0
    }
    pub fn SetExcludeAutofilledValue(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 9;
        } else {
            self.0 &= !(1 << 9);
        }
        self
    }
    pub fn ForSelectionToString(self) -> bool {
        self.0 & (1 << 10) != 0
    }
    pub fn SetForSelectionToString(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 10;
        } else {
            self.0 &= !(1 << 10);
        }
        self
    }
    pub fn ForWindowFind(self) -> bool {
        self.0 & (1 << 11) != 0
    }
    pub fn SetForWindowFind(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 11;
        } else {
            self.0 &= !(1 << 11);
        }
        self
    }
    pub fn IgnoresStyleVisibility(self) -> bool {
        self.0 & (1 << 12) != 0
    }
    pub fn SetIgnoresStyleVisibility(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 12;
        } else {
            self.0 &= !(1 << 12);
        }
        self
    }
    pub fn StopsOnFormControls(self) -> bool {
        self.0 & (1 << 13) != 0
    }
    pub fn SetStopsOnFormControls(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 13;
        } else {
            self.0 &= !(1 << 13);
        }
        self
    }
    pub fn DoesNotEmitSpaceBeyondRangeEnd(self) -> bool {
        self.0 & (1 << 14) != 0
    }
    pub fn SetDoesNotEmitSpaceBeyondRangeEnd(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 14;
        } else {
            self.0 &= !(1 << 14);
        }
        self
    }
    pub fn SkipsUnselectableContent(self) -> bool {
        self.0 & (1 << 15) != 0
    }
    pub fn SetSkipsUnselectableContent(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 15;
        } else {
            self.0 &= !(1 << 15);
        }
        self
    }
    pub fn SuppressesExtraNewlineEmission(self) -> bool {
        self.0 & (1 << 16) != 0
    }
    pub fn SetSuppressesExtraNewlineEmission(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 16;
        } else {
            self.0 &= !(1 << 16);
        }
        self
    }
    pub fn IgnoresDisplayLock(self) -> bool {
        self.0 & (1 << 17) != 0
    }
    pub fn SetIgnoresDisplayLock(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 17;
        } else {
            self.0 &= !(1 << 17);
        }
        self
    }
    pub fn EmitsPunctuationForReplacedElements(self) -> bool {
        self.0 & (1 << 18) != 0
    }
    pub fn SetEmitsPunctuationForReplacedElements(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 18;
        } else {
            self.0 &= !(1 << 18);
        }
        self
    }
    pub fn IgnoresCssTextTransforms(self) -> bool {
        self.0 & (1 << 19) != 0
    }
    pub fn SetIgnoresCssTextTransforms(&mut self, value: bool) -> &mut Self {
        if value {
            self.0 |= 1 << 19;
        } else {
            self.0 &= !(1 << 19);
        }
        self
    }
    pub fn EmitsObjectReplacementCharacterBehavior() -> Self {
        let mut b = Self::default();
        b.SetEmitsObjectReplacementCharacter(true);
        b
    }
    pub fn IgnoresStyleVisibilityBehavior() -> Self {
        let mut b = Self::default();
        b.SetIgnoresStyleVisibility(true);
        b
    }
    pub fn DefaultRangeLengthBehavior() -> Self {
        Self::EmitsObjectReplacementCharacterBehavior()
    }
    pub fn AllVisiblePositionsRangeLengthBehavior() -> Self {
        let mut b = Self::DefaultRangeLengthBehavior();
        b.SetEmitsCharactersBetweenAllVisiblePositions(true);
        b
    }
    pub fn AllVisiblePositionsIncludingShadowRootRangeLengthBehavior() -> Self {
        let mut b = Self::AllVisiblePositionsRangeLengthBehavior();
        b.SetEntersOpenShadowRoots(true);
        b
    }
    pub fn NoTrailingSpaceRangeLengthBehavior() -> Self {
        let mut b = Self::DefaultRangeLengthBehavior();
        b.SetDoesNotEmitSpaceBeyondRangeEnd(true);
        b
    }
}

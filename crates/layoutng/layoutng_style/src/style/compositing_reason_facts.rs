//! CompositingReasonFinder.cc:542-568. The pure decision is shared by the
//! retained ComputedStyle diff and PrePaint's source-backed PaintStyle view.
use super::computed_style::ComputedStyle;
use foundation::{CompositingReason as R, CompositingReasons};
pub struct Potential3DTransformFacts {
    pub nontrivial_transform: bool,
    pub trivial_transform: bool,
    pub translate: bool,
    pub rotate: bool,
    pub scale: bool,
}
pub fn Potential3DCompositingReasons(facts: Potential3DTransformFacts) -> CompositingReasons {
    let mut reasons = CompositingReasons::new();
    if facts.nontrivial_transform {
        reasons.Put(R::k3DTransform);
    } else if facts.trivial_transform {
        reasons.Put(R::kTrivial3DTransform);
    }
    if facts.translate {
        reasons.Put(R::k3DTranslate);
    }
    if facts.rotate {
        reasons.Put(R::k3DRotate);
    }
    if facts.scale {
        reasons.Put(R::k3DScale);
    }
    reasons
}
impl ComputedStyle {
    pub fn Potential3DCompositingReasons(&self) -> CompositingReasons {
        let transform = self.Transform();
        let perspective = transform.HasNonPerspective3DOperation();
        Potential3DCompositingReasons(Potential3DTransformFacts {
            nontrivial_transform: perspective && transform.HasNonTrivial3DComponent(),
            trivial_transform: perspective && !transform.HasNonTrivial3DComponent(),
            translate: unsafe { self.Translate().as_ref() }.is_some_and(|value| value.Z() != 0.0),
            rotate: unsafe { self.Rotate().as_ref() }
                .is_some_and(|value| value.X() != 0.0 || value.Y() != 0.0),
            scale: unsafe { self.Scale().as_ref() }.is_some_and(|value| value.Z() != 1.0),
        })
    }
}

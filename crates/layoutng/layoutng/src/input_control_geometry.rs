use super::*;
impl LayoutNode<'_> {
    // ImageInputType.cc:186-190,213-217. The input owner applies the source
    // AdjustForAbsoluteZoom::AdjustInt to these integer content dimensions.
    pub fn ImageInputContentBoxRawSize(&self) -> Option<(i32, i32, f32)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() }?;
        let rect = box_.PhysicalContentBoxRect();
        Some((
            rect.Width().ToInt(),
            rect.Height().ToInt(),
            self.object.StyleRef().EffectiveZoom(),
        ))
    }
}

//! Decides which Paint transform nodes may be folded into pending-layer
//! geometry. The facade supplies resident-node access and applies the chosen
//! mutations without owning the decision.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub trait PendingLayerDecomposition {
    type Transform: Clone + Eq + Hash;
    type Clip: Clone;
    type Effect: Clone;
    type NodeIdentity: Copy + Eq + Hash;

    fn TransformNode(&self) -> Self::Transform;
    fn ClipNode(&self) -> Self::Clip;
    fn EffectNode(&self) -> Self::Effect;

    fn TransformIsRoot(&self, node: &Self::Transform) -> bool;
    fn TransformParentUnalias(&self, node: &Self::Transform) -> Self::Transform;
    fn TransformUnaliasedParent(&self, node: &Self::Transform) -> Option<Self::Transform>;
    fn TransformIsIdentityOr2dTranslation(&self, node: &Self::Transform) -> bool;
    fn TransformHasScrollNode(&self, node: &Self::Transform) -> bool;
    fn TransformHasDirectCompositingReasonsOtherThan3dTransform(
        &self,
        node: &Self::Transform,
    ) -> bool;
    fn TransformFlattensInheritedTransformSameAsParent(&self, node: &Self::Transform) -> bool;
    fn TransformBackfaceVisibilitySameAsParent(&self, node: &Self::Transform) -> bool;
    fn TransformHasStickyConstraint(&self, node: &Self::Transform) -> bool;
    fn TransformHasAnchorPositionScrollData(&self, node: &Self::Transform) -> bool;
    fn TransformIsAffectedByOuterViewportBoundsDelta(&self, node: &Self::Transform) -> bool;

    fn ClipIsRoot(&self, node: &Self::Clip) -> bool;
    fn ClipIdentity(&self, node: &Self::Clip) -> Self::NodeIdentity;
    fn ClipParentUnalias(&self, node: &Self::Clip) -> Self::Clip;
    fn ClipLocalTransformSpaceUnalias(&self, node: &Self::Clip) -> Self::Transform;

    fn EffectIsRoot(&self, node: &Self::Effect) -> bool;
    fn EffectIdentity(&self, node: &Self::Effect) -> Self::NodeIdentity;
    fn EffectParentUnalias(&self, node: &Self::Effect) -> Self::Effect;
    fn EffectLocalTransformSpaceUnalias(&self, node: &Self::Effect) -> Self::Transform;

    fn IsScrollHitTestLayer(&self) -> bool;
    fn ScrollTranslationForScrollHitTestLayer(&self) -> Self::Transform;

    fn ApplyDecompositedTransform(&mut self, node: &Self::Transform);
    fn FinishDecompositedTransforms(&mut self, remaining_transform: Self::Transform);
}

pub fn DecompositeTransforms<L: PendingLayerDecomposition>(pending_layers: &mut [L]) {
    let mut can_be_decomposited: HashMap<L::Transform, bool> = HashMap::new();
    let mut clips_and_effects_seen: HashSet<L::NodeIdentity> = HashSet::new();
    for pending_layer in pending_layers.iter() {
        let mut node = pending_layer.TransformNode();
        while !pending_layer.TransformIsRoot(&node) && !can_be_decomposited.contains_key(&node) {
            if !pending_layer.TransformIsIdentityOr2dTranslation(&node)
                || pending_layer.TransformHasScrollNode(&node)
                || pending_layer.TransformHasDirectCompositingReasonsOtherThan3dTransform(&node)
                || !pending_layer.TransformFlattensInheritedTransformSameAsParent(&node)
                || !pending_layer.TransformBackfaceVisibilitySameAsParent(&node)
            {
                mark_not_decompositable(pending_layer, &mut can_be_decomposited, node.clone());
                break;
            }
            debug_assert!(!pending_layer.TransformHasStickyConstraint(&node));
            debug_assert!(!pending_layer.TransformHasAnchorPositionScrollData(&node));
            debug_assert!(!pending_layer.TransformIsAffectedByOuterViewportBoundsDelta(&node));
            can_be_decomposited.insert(node.clone(), true);
            node = pending_layer.TransformParentUnalias(&node);
        }

        let mut clip = pending_layer.ClipNode();
        while !pending_layer.ClipIsRoot(&clip)
            && !clips_and_effects_seen.contains(&pending_layer.ClipIdentity(&clip))
        {
            clips_and_effects_seen.insert(pending_layer.ClipIdentity(&clip));
            mark_not_decompositable(
                pending_layer,
                &mut can_be_decomposited,
                pending_layer.ClipLocalTransformSpaceUnalias(&clip),
            );
            clip = pending_layer.ClipParentUnalias(&clip);
        }
        let mut effect = pending_layer.EffectNode();
        while !pending_layer.EffectIsRoot(&effect)
            && !clips_and_effects_seen.contains(&pending_layer.EffectIdentity(&effect))
        {
            clips_and_effects_seen.insert(pending_layer.EffectIdentity(&effect));
            mark_not_decompositable(
                pending_layer,
                &mut can_be_decomposited,
                pending_layer.EffectLocalTransformSpaceUnalias(&effect),
            );
            effect = pending_layer.EffectParentUnalias(&effect);
        }
        if pending_layer.IsScrollHitTestLayer() {
            mark_not_decompositable(
                pending_layer,
                &mut can_be_decomposited,
                pending_layer.ScrollTranslationForScrollHitTestLayer(),
            );
        }
    }

    for pending_layer in pending_layers.iter_mut() {
        let mut transform = pending_layer.TransformNode();
        while !pending_layer.TransformIsRoot(&transform)
            && *can_be_decomposited
                .get(&transform)
                .expect("every visited transform has a decomposition decision")
        {
            pending_layer.ApplyDecompositedTransform(&transform);
            transform = pending_layer.TransformParentUnalias(&transform);
        }
        pending_layer.FinishDecompositedTransforms(transform);
    }
}

fn mark_not_decompositable<L: PendingLayerDecomposition>(
    layer: &L,
    decisions: &mut HashMap<L::Transform, bool>,
    mut node: L::Transform,
) {
    loop {
        if layer.TransformIsRoot(&node) {
            break;
        }
        match decisions.insert(node.clone(), false) {
            Some(false) => break,
            Some(true) | None => {}
        }
        let Some(parent) = layer.TransformUnaliasedParent(&node) else {
            break;
        };
        node = parent;
    }
}

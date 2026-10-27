// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/cascade_layer_map.h:17-67
// cpp: third_party/blink/renderer/core/css/cascade_layer_map.cc:16-100

use crate::active_style_sheets::ActiveStyleSheet;
use crate::cascade_layer::CascadeLayer;
use std::cmp::Ordering;
use std::collections::HashMap;

pub trait CascadeLayerRuleSet {
    fn CascadeLayers(&self) -> Option<&CascadeLayer>;
}

pub struct CascadeLayerMap {
    canonical_root_layer_: CascadeLayer,
    layer_order_map_: HashMap<CascadeLayer, u16>,
}

fn AddLayers(
    canonical_layer: &CascadeLayer,
    layer_from_sheet: &CascadeLayer,
    canonical_layer_map: &mut HashMap<CascadeLayer, CascadeLayer>,
) {
    debug_assert_eq!(canonical_layer.GetName(), layer_from_sheet.GetName());
    canonical_layer_map.insert(layer_from_sheet.clone(), canonical_layer.clone());
    for sub_layer in layer_from_sheet.GetDirectSubLayers().iter() {
        let canonical_sub_layer = canonical_layer.GetOrAddSubLayer(&[sub_layer.GetName().clone()]);
        AddLayers(&canonical_sub_layer, sub_layer, canonical_layer_map);
    }
}

fn ComputeLayerOrder(
    layer: &CascadeLayer,
    next: &mut u16,
    orders: &mut HashMap<CascadeLayer, u16>,
) {
    for sub_layer in layer.GetDirectSubLayers().iter() {
        ComputeLayerOrder(sub_layer, next, orders);
    }
    orders.insert(layer.clone(), *next);
    *next = next.wrapping_add(1);
}

impl CascadeLayerMap {
    pub const kImplicitOuterLayerOrder: u16 = u16::MAX;

    pub fn new<S, R>(sheets: &[ActiveStyleSheet<S, R>]) -> Self
    where
        R: CascadeLayerRuleSet,
    {
        let canonical_root_layer = CascadeLayer::default();
        let mut canonical_layer_map = HashMap::new();
        for sheet in sheets {
            if let Some(root) = sheet
                .rule_set
                .as_ref()
                .and_then(|rule_set| rule_set.CascadeLayers())
            {
                AddLayers(&canonical_root_layer, root, &mut canonical_layer_map);
            }
        }

        let mut next = 0;
        let mut canonical_orders = HashMap::new();
        ComputeLayerOrder(&canonical_root_layer, &mut next, &mut canonical_orders);
        canonical_orders.insert(canonical_root_layer.clone(), Self::kImplicitOuterLayerOrder);

        let mut layer_order_map = HashMap::new();
        for (layer_from_sheet, canonical_layer) in canonical_layer_map {
            let order = canonical_orders[&canonical_layer];
            debug_assert!(
                canonical_layer == canonical_root_layer || order < Self::kImplicitOuterLayerOrder
            );
            layer_order_map.insert(layer_from_sheet, order);
        }
        Self {
            canonical_root_layer_: canonical_root_layer,
            layer_order_map_: layer_order_map,
        }
    }

    pub fn GetLayerOrder(&self, layer: &CascadeLayer) -> u16 {
        self.layer_order_map_
            .get(layer)
            .copied()
            .unwrap_or_else(|| {
                debug_assert!(false, "lookup of a cascade layer outside this map");
                Self::kImplicitOuterLayerOrder
            })
    }

    pub fn CompareLayerOrder(
        &self,
        lhs: Option<&CascadeLayer>,
        rhs: Option<&CascadeLayer>,
    ) -> Ordering {
        match (lhs, rhs) {
            (Some(a), Some(b)) if a == b => Ordering::Equal,
            (None, None) => Ordering::Equal,
            _ => lhs
                .map_or(Self::kImplicitOuterLayerOrder, |l| self.GetLayerOrder(l))
                .cmp(&rhs.map_or(Self::kImplicitOuterLayerOrder, |r| self.GetLayerOrder(r))),
        }
    }

    pub fn GetRootLayer(&self) -> &CascadeLayer {
        &self.canonical_root_layer_
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active_style_sheets::ActiveStyleSheet;
    use foundation::AtomicString;
    use std::rc::Rc;

    struct Sheet;
    struct Rules(CascadeLayer);
    impl CascadeLayerRuleSet for Rules {
        fn CascadeLayers(&self) -> Option<&CascadeLayer> {
            Some(&self.0)
        }
    }
    fn names(values: &[&str]) -> Vec<AtomicString> {
        values.iter().map(|v| AtomicString::from_str(v)).collect()
    }

    #[test]
    fn merges_named_layers_and_orders_children_before_parents() {
        let root_a = CascadeLayer::default();
        let a = root_a.GetOrAddSubLayer(&names(&["a"]));
        let ab = root_a.GetOrAddSubLayer(&names(&["a", "b"]));
        let root_b = CascadeLayer::default();
        let a2 = root_b.GetOrAddSubLayer(&names(&["a"]));
        let ac = root_b.GetOrAddSubLayer(&names(&["a", "c"]));
        let sheets = vec![
            ActiveStyleSheet::new(Rc::new(Sheet), Some(Rc::new(Rules(root_a)))),
            ActiveStyleSheet::new(Rc::new(Sheet), Some(Rc::new(Rules(root_b)))),
        ];
        let map = CascadeLayerMap::new(&sheets);
        assert_eq!(map.GetLayerOrder(&a), map.GetLayerOrder(&a2));
        assert!(map.GetLayerOrder(&ab) < map.GetLayerOrder(&a));
        assert!(map.GetLayerOrder(&ac) < map.GetLayerOrder(&a));
        assert_eq!(map.CompareLayerOrder(Some(&a), None), Ordering::Less);
        assert_eq!(map.CompareLayerOrder(None, None), Ordering::Equal);
    }
}

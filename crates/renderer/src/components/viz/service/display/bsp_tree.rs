use super::draw_polygon::DrawPolygon;
use std::collections::VecDeque;
// Source: components/viz/service/display/bsp_tree.h:21-34
// Source: components/viz/service/display/bsp_tree.cc:18-21
pub struct BspNode {
    pub node_data: DrawPolygon,
    pub coplanars_front: Vec<DrawPolygon>,
    pub coplanars_back: Vec<DrawPolygon>,
    pub back_child: Option<Box<BspNode>>,
    pub front_child: Option<Box<BspNode>>,
}
impl BspNode {
    pub fn new(node_data: DrawPolygon) -> Self {
        Self {
            node_data,
            coplanars_front: Vec::new(),
            coplanars_back: Vec::new(),
            back_child: None,
            front_child: None,
        }
    }
}
// Source: components/viz/service/display/bsp_tree.h:36-58
pub struct BspTree {
    pub root: Option<Box<BspNode>>,
}
impl BspTree {
    // Source: components/viz/service/display/bsp_tree.cc:23-31
    pub fn new(list: &mut VecDeque<DrawPolygon>) -> Self {
        let Some(data) = list.pop_front() else {
            return Self { root: None };
        };
        let mut node = BspNode::new(data);
        Self::build_tree(&mut node, list);
        Self {
            root: Some(Box::new(node)),
        }
    }
    // Source: components/viz/service/display/bsp_tree.cc:33-80
    fn build_tree(node: &mut BspNode, list: &mut VecDeque<DrawPolygon>) {
        let mut front_list = VecDeque::new();
        let mut back_list = VecDeque::new();
        while let Some(polygon) = list.pop_front() {
            let (front, back, coplanar) = node.node_data.split_polygon(polygon);
            if coplanar {
                if let Some(front) = front {
                    node.coplanars_front.push(front);
                }
                if let Some(back) = back {
                    node.coplanars_back.push(back);
                }
            } else {
                if let Some(front) = front {
                    front_list.push_back(front);
                }
                if let Some(back) = back {
                    back_list.push_back(back);
                }
            }
        }
        if let Some(data) = back_list.pop_front() {
            let mut child = BspNode::new(data);
            Self::build_tree(&mut child, &mut back_list);
            node.back_child = Some(Box::new(child));
        }
        if let Some(data) = front_list.pop_front() {
            let mut child = BspNode::new(data);
            Self::build_tree(&mut child, &mut front_list);
            node.front_child = Some(Box::new(child));
        }
    }
    // Source: components/viz/service/display/bsp_tree.h:42-47,60-112
    // FnMut is the C++ ActionHandler template's operator() capability. Mutable
    // traversal borrows replace pointers into tree-owned polygons.
    pub fn traverse_with_action_handler(&mut self, action: &mut impl FnMut(&mut DrawPolygon)) {
        if let Some(root) = self.root.as_deref_mut() {
            Self::walk(root, action);
        }
    }
    fn walk(node: &mut BspNode, action: &mut impl FnMut(&mut DrawPolygon)) {
        let in_front = Self::get_camera_position_relative(&node.node_data);
        let (first, second) = if in_front {
            (&mut node.back_child, &mut node.front_child)
        } else {
            (&mut node.front_child, &mut node.back_child)
        };
        if let Some(child) = first.as_deref_mut() {
            Self::walk(child, action);
        }
        debug_assert!(node
            .coplanars_back
            .windows(2)
            .all(|w| w[0].order_index <= w[1].order_index));
        for polygon in &mut node.coplanars_back {
            action(polygon);
        }
        action(&mut node.node_data);
        debug_assert!(node
            .coplanars_front
            .windows(2)
            .all(|w| w[0].order_index <= w[1].order_index));
        for polygon in &mut node.coplanars_front {
            action(polygon);
        }
        if let Some(child) = second.as_deref_mut() {
            Self::walk(child, action);
        }
    }
    // Source: components/viz/service/display/bsp_tree.h:114-116
    // Source: components/viz/service/display/bsp_tree.cc:82-89
    pub fn get_camera_position_relative(node: &DrawPolygon) -> bool {
        node.normal.z > 0.0
    }
}

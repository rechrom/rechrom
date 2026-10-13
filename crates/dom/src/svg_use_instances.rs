//! SVGUseElement's retained, document-owned instance lifecycle.
//! Pinned Blink: svg_use_element.cc BuildPendingResource/CreateInstanceTree,
//! PostProcessInstanceTree, AttachShadowTree and HasCycleUseReferencing.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SVGUseInstanceStatus {
    Ready,
    MissingTarget,
    DisallowedTarget,
    CyclicReference,
    UnsupportedReference,
    UnsupportedViewport,
    UnsupportedImage,
    UnsupportedSwitch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SVGUseInstancePlan {
    status: SVGUseInstanceStatus,
    root: Option<Box<InstanceNode>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InstanceNode {
    source: usize,
    kind: DOMNodeType,
    namespace: DOMNamespace,
    name: String,
    data: String,
    attributes: Vec<DOMAttribute>,
    children: Vec<InstanceNode>,
    instance: Option<SVGUseInstancePlan>,
}

// SVGUseElement::IsDisallowedElement. Disallowed subtrees are removed before
// connecting clones; this is distinct from supported SVG features not yet
// implemented in our layout/resource owners.
fn allowed(node: &DOMNode) -> bool {
    node.Type() != DOMNodeType::kElement
        || (node.Namespace() == DOMNamespace::kSVG
            && matches!(
                node.Name(),
                "a" | "circle"
                    | "desc"
                    | "ellipse"
                    | "g"
                    | "image"
                    | "line"
                    | "metadata"
                    | "path"
                    | "polygon"
                    | "polyline"
                    | "rect"
                    | "svg"
                    | "switch"
                    | "symbol"
                    | "text"
                    | "textPath"
                    | "title"
                    | "tspan"
                    | "use"
            ))
}

impl PersistentDocument {
    pub fn SVGUseInstanceStatus(&self, host: usize) -> Option<SVGUseInstanceStatus> {
        (self.svg_use_inputs_revision == Some(self.collection_membership_revision))
            .then(|| self.svg_use_instance_status.get(&host).copied())
            .flatten()
    }

    pub fn SVGUseInstanceRoot(&self, host: usize) -> Option<usize> {
        (self.SVGUseInstanceStatus(host) == Some(SVGUseInstanceStatus::Ready))
            .then(|| self.ShadowRoot(host))
            .flatten()
            .and_then(|root| self.arena[root].children.first().copied())
    }

    /// Shared tree-scoped local lookup for SVG resource owners. Corresponding
    /// elements retain the originating source scope, not the clone's ID table.
    pub fn FindSVGResourceById(&self, element: usize, id: &str) -> Option<usize> {
        let origin = self.CorrespondingSVGElement(element).unwrap_or(element);
        self.FindElementByIdInTreeScope(origin, id)
            .filter(|&target| self.IsConnectedToDocument(target))
    }

    /// Called by document lifecycle assembly before style resolution. Source
    /// plans carry tree-scoped first-ID identity and attributes, never computed
    /// style. Unchanged plans preserve node identity. Changed plans detach the
    /// old tree before connecting the replacement, including nested use roots.
    /// The revision gate is conservative; selective ID observers are a future
    /// optimization, not a second resource or style owner.
    pub fn UpdateSVGUseInstances(&mut self) -> bool {
        if self.svg_use_inputs_revision == Some(self.collection_membership_revision) {
            return false;
        }
        let hosts = (0..self.arena.len())
            .filter(|&node| {
                self.arena[node].IsElement("use", DOMNamespace::kSVG)
                    && self.CorrespondingSVGElement(node).is_none()
                    && self.IsConnectedToDocument(node)
            })
            .collect::<Vec<_>>();
        let active = hosts.iter().copied().collect::<HashSet<_>>();
        let obsolete = self
            .svg_use_instances
            .keys()
            .copied()
            .filter(|host| !active.contains(host))
            .collect::<Vec<_>>();
        let mut changed = false;
        for host in obsolete {
            self.DetachSVGUseInstance(host);
            self.svg_use_instances.remove(&host);
            self.svg_use_instance_status.remove(&host);
            changed = true;
        }
        // Compute all source plans before mutating any shadow tree. ID lookup
        // is in the source TreeScope, never an arena scan or a clone's scope.
        let plans = hosts
            .into_iter()
            .map(|host| {
                let mut ancestors = Vec::new();
                let mut parent = self.ParentOrShadowHostElement(host);
                while let Some(node) = parent {
                    ancestors.push(self.CorrespondingSVGElement(node).unwrap_or(node));
                    parent = self.ParentOrShadowHostElement(node);
                }
                (host, self.SVGUsePlan(host, &ancestors))
            })
            .collect::<Vec<_>>();
        for (host, plan) in plans {
            if self.svg_use_instances.get(&host) == Some(&plan) {
                continue;
            }
            self.DetachSVGUseInstance(host);
            let root = self
                .ShadowRoot(host)
                .unwrap_or_else(|| self.AttachShadowRoot(host, DOMShadowRootMode::kUserAgent));
            self.svg_use_instance_status.insert(host, plan.status);
            if let Some(target) = &plan.root {
                let instance = self.CloneSVGUseInstance(target);
                self.AppendChild(root, instance);
            }
            self.svg_use_instances.insert(host, plan);
            self.RecordFlatTreeChange();
            self.InvalidateNodeStyle(host);
            changed = true;
        }
        self.svg_use_inputs_revision = Some(self.collection_membership_revision);
        changed
    }

    fn SVGUsePlan(&self, host: usize, ancestors: &[usize]) -> SVGUseInstancePlan {
        use SVGUseInstanceStatus::*;
        let empty = |status| SVGUseInstancePlan { status, root: None };
        let node = &self.arena[host];
        // SVGURIReference prefers the unnamespaced SVG2 href, including an
        // explicitly empty value, over xlink:href.
        let href = node
            .attributes
            .iter()
            .find(|a| a.local_name == "href" && a.namespace_uri.is_empty() && a.prefix.is_empty())
            .or_else(|| {
                node.attributes.iter().find(|a| {
                    a.local_name == "href"
                        && (a.namespace_uri == "http://www.w3.org/1999/xlink"
                            || a.prefix == "xlink")
                })
            })
            .map(|a| a.value.as_str())
            .unwrap_or("");
        if href.is_empty() || href == "#" {
            return empty(MissingTarget);
        }
        let Some(id) = href.strip_prefix('#').filter(|id| !id.contains('%')) else {
            return empty(UnsupportedReference);
        };
        let Some(target) = self
            .FindElementByIdInTreeScope(host, id)
            .filter(|&target| self.IsConnectedToDocument(target))
        else {
            return empty(MissingTarget);
        };
        if self.arena[target].Namespace() != DOMNamespace::kSVG || !allowed(&self.arena[target]) {
            return empty(DisallowedTarget);
        }
        if target == host
            || ancestors.iter().any(|&ancestor| {
                self.arena[ancestor].Namespace() == DOMNamespace::kSVG
                    && self.arena[ancestor]
                        .FindAttribute("id")
                        .is_some_and(|a| a.value == id)
            })
        {
            return empty(CyclicReference);
        }
        let mut branch = ancestors.to_vec();
        branch.push(host);
        match self.SVGInstanceNode(target, &mut branch) {
            Ok(root) => SVGUseInstancePlan {
                status: Ready,
                root: Some(Box::new(root)),
            },
            Err(status) => empty(status),
        }
    }

    fn SVGInstanceNode(
        &self,
        source: usize,
        ancestors: &mut Vec<usize>,
    ) -> Result<InstanceNode, SVGUseInstanceStatus> {
        let node = &self.arena[source];
        match node.Name() {
            "svg" | "symbol" => return Err(SVGUseInstanceStatus::UnsupportedViewport),
            "image" => return Err(SVGUseInstanceStatus::UnsupportedImage),
            "switch" => return Err(SVGUseInstanceStatus::UnsupportedSwitch),
            _ => {}
        }
        let instance = node
            .IsElement("use", DOMNamespace::kSVG)
            .then(|| self.SVGUsePlan(source, ancestors));
        ancestors.push(source);
        let children = node
            .children
            .iter()
            .copied()
            .filter(|&child| allowed(&self.arena[child]))
            .map(|child| self.SVGInstanceNode(child, ancestors))
            .collect::<Result<Vec<_>, _>>()?;
        ancestors.pop();
        Ok(InstanceNode {
            source,
            kind: node.node_type,
            namespace: node.namespace,
            name: node.name.clone(),
            data: node.data.clone(),
            attributes: node.attributes.clone(),
            children,
            instance,
        })
    }

    fn CloneSVGUseInstance(&mut self, plan: &InstanceNode) -> usize {
        let clone = self.Allocate(
            plan.kind,
            plan.namespace,
            plan.name.clone(),
            plan.data.clone(),
            plan.attributes.clone(),
        );
        if plan.kind == DOMNodeType::kElement {
            self.SetCorrespondingSVGElement(clone, Some(plan.source));
        }
        for child in &plan.children {
            let child = self.CloneSVGUseInstance(child);
            self.AppendChild(clone, child);
        }
        if let Some(instance) = &plan.instance {
            let root = self.AttachShadowRoot(clone, DOMShadowRootMode::kUserAgent);
            self.svg_use_instance_status.insert(clone, instance.status);
            if let Some(target) = &instance.root {
                let child = self.CloneSVGUseInstance(target);
                self.AppendChild(root, child);
            }
        }
        clone
    }

    fn DetachSVGUseInstance(&mut self, host: usize) {
        let Some(root) = self.ShadowRoot(host) else {
            return;
        };
        let children = self.arena[root].children.clone();
        let mut pending = children.clone();
        while let Some(node) = pending.pop() {
            pending.extend_from_slice(&self.arena[node].children);
            if let Some(shadow) = self.ShadowRoot(node) {
                pending.extend_from_slice(&self.arena[shadow].children);
                self.shadow_roots.remove(&node);
                self.shadow_hosts.remove(&shadow);
            }
            self.corresponding_svg_elements.remove(&self.arena[node].id);
            self.svg_use_instance_status.remove(&node);
            self.RemoveResolvedStyle(node);
        }
        for child in children {
            self.Remove(child);
        }
    }
}

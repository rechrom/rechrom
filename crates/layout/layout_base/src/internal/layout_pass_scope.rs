use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use super::layout_algorithm_set::{
    FloatLayoutSupport, InlineLayoutSupport, LayoutAlgorithmSet, ListLayoutSupport, MathMLSupport,
    SvgLayoutSupport,
};
use super::layout_object_factory_set::LayoutObjectFactorySet;
use foundation::UnsupportedLayout;

// cpp: layoutng/internal/layout_pass_scope.h:28-31
// cpp: layoutng/internal/layout_pass_scope.cc:14-15
thread_local! {
    static OBJECTS: Cell<*const LayoutObjectFactorySet> = const { Cell::new(std::ptr::null()) };
}

// cpp: layoutng/internal/layout_pass_scope.h:17-33
pub struct LayoutObjectFactoryScope {
    previous_objects_: *const LayoutObjectFactorySet,
    _thread_bound: PhantomData<Rc<()>>,
}

impl Default for LayoutObjectFactoryScope {
    fn default() -> Self {
        Self::new(std::ptr::null())
    }
}

#[allow(non_snake_case)]
impl LayoutObjectFactoryScope {
    // cpp: layoutng/internal/layout_pass_scope.cc:17-22
    pub fn new(objects: *const LayoutObjectFactorySet) -> Self {
        let previous_objects = OBJECTS.with(Cell::get);
        if !objects.is_null() {
            OBJECTS.with(|current| current.set(objects));
        }
        Self {
            previous_objects_: previous_objects,
            _thread_bound: PhantomData,
        }
    }

    // cpp: layoutng/internal/layout_pass_scope.h:27-27
    pub fn Objects() -> *const LayoutObjectFactorySet {
        OBJECTS.with(Cell::get)
    }
}

// cpp: layoutng/internal/layout_pass_scope.cc:24-26
impl Drop for LayoutObjectFactoryScope {
    fn drop(&mut self) {
        OBJECTS.with(|current| current.set(self.previous_objects_));
    }
}

// cpp: layoutng/internal/layout_pass_scope.h:55-57
// cpp: layoutng/internal/layout_pass_scope.cc:12-13
thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    static ALGORITHMS: Cell<*const LayoutAlgorithmSet> = const { Cell::new(std::ptr::null()) };
    // The SVG queries of the last installed algorithm set. They only read
    // geometry the SVG layout package stored on its LayoutSVG owners, so
    // PrePaint and Paint (same document thread, after the pass) can ask them
    // as Chromium's painters read LayoutSVGShape / LayoutSVGRoot members.
    static RESIDENT_SVG_SUPPORT: Cell<SvgLayoutSupport> = Cell::new(SvgLayoutSupport::default());
    // Likewise the MathML queries (operator metrics, fraction rule) that
    // MathMLPainter's data reads from the stored MathMLPaintInfo.
    static RESIDENT_MATHML_SUPPORT: Cell<MathMLSupport> = Cell::new(MathMLSupport::default());
    // Likewise the inline queries base calls after layout (offset mappings for
    // editing and selection). Layout runs a pass before any such query, so the
    // last installed set is the one that laid the document out.
    static RESIDENT_INLINE_SUPPORT: Cell<InlineLayoutSupport> =
        Cell::new(InlineLayoutSupport::default());
    // Likewise the list queries editing reads after layout (list item value).
    static RESIDENT_LIST_SUPPORT: Cell<ListLayoutSupport> =
        Cell::new(ListLayoutSupport::default());
}

// cpp: layoutng/internal/layout_pass_scope.h:38-61
pub struct LayoutPassScope {
    previous_algorithms_: *const LayoutAlgorithmSet,
    object_factory_scope_: LayoutObjectFactoryScope,
}

impl Default for LayoutPassScope {
    fn default() -> Self {
        Self::new(std::ptr::null(), std::ptr::null())
    }
}

#[allow(non_snake_case)]
impl LayoutPassScope {
    pub fn with_algorithms(algorithms: *const LayoutAlgorithmSet) -> Self {
        Self::new(algorithms, std::ptr::null())
    }

    // cpp: layoutng/internal/layout_pass_scope.cc:28-36
    pub fn new(
        algorithms: *const LayoutAlgorithmSet,
        objects: *const LayoutObjectFactorySet,
    ) -> Self {
        let previous_algorithms = ALGORITHMS.with(Cell::get);
        let object_factory_scope = LayoutObjectFactoryScope::new(objects);
        DEPTH.with(|depth| depth.set(depth.get().wrapping_add(1)));
        if !algorithms.is_null() {
            ALGORITHMS.with(|current| current.set(algorithms));
            let svg_support = unsafe { &*algorithms }.svg_support;
            RESIDENT_SVG_SUPPORT.with(|resident| resident.set(svg_support));
            let mathml_support = unsafe { &*algorithms }.mathml_support;
            RESIDENT_MATHML_SUPPORT.with(|resident| resident.set(mathml_support));
            let inline_support = unsafe { &*algorithms }.inline_support;
            RESIDENT_INLINE_SUPPORT.with(|resident| resident.set(inline_support));
            let list_support = unsafe { &*algorithms }.list_support;
            RESIDENT_LIST_SUPPORT.with(|resident| resident.set(list_support));
        }
        Self {
            previous_algorithms_: previous_algorithms,
            object_factory_scope_: object_factory_scope,
        }
    }

    // cpp: layoutng/internal/layout_pass_scope.h:48-53
    pub fn IsActive() -> bool {
        DEPTH.with(|depth| depth.get() != 0)
    }

    pub fn Algorithms() -> *const LayoutAlgorithmSet {
        ALGORITHMS.with(Cell::get)
    }

    /// The SVG layout queries for post-layout readers: the active pass's
    /// set, else the last one a pass installed on this thread.
    /// The MathML queries for post-layout readers (see ResidentSvgSupport).
    pub fn ResidentMathMLSupport() -> MathMLSupport {
        let algorithms = Self::Algorithms();
        if algorithms.is_null() {
            RESIDENT_MATHML_SUPPORT.with(Cell::get)
        } else {
            unsafe { &*algorithms }.mathml_support
        }
    }

    /// The inline queries for post-layout readers (see ResidentSvgSupport).
    pub fn ResidentInlineSupport() -> InlineLayoutSupport {
        let algorithms = Self::Algorithms();
        if algorithms.is_null() {
            RESIDENT_INLINE_SUPPORT.with(Cell::get)
        } else {
            unsafe { &*algorithms }.inline_support
        }
    }

    /// The list queries for post-layout readers (see ResidentSvgSupport).
    pub fn ResidentListSupport() -> ListLayoutSupport {
        let algorithms = Self::Algorithms();
        if algorithms.is_null() {
            RESIDENT_LIST_SUPPORT.with(Cell::get)
        } else {
            unsafe { &*algorithms }.list_support
        }
    }

    pub fn ResidentSvgSupport() -> SvgLayoutSupport {
        let algorithms = Self::Algorithms();
        if algorithms.is_null() {
            RESIDENT_SVG_SUPPORT.with(Cell::get)
        } else {
            unsafe { &*algorithms }.svg_support
        }
    }

    pub fn Objects() -> *const LayoutObjectFactorySet {
        LayoutObjectFactoryScope::Objects()
    }

    // cpp: layoutng/internal/layout_pass_scope.cc:44-51
    pub fn RequireFloatSupport() -> *const FloatLayoutSupport {
        let algorithms = Self::Algorithms();
        if algorithms.is_null()
            || unsafe { &*algorithms }
                .float_support
                .margin_box_inline_size
                .is_none()
            || unsafe { &*algorithms }.float_support.position.is_none()
            || unsafe { &*algorithms }.float_support.create_shape.is_none()
        {
            std::panic::panic_any(UnsupportedLayout::new(
                "float layout module is not installed",
            ));
        }
        unsafe { std::ptr::addr_of!((*algorithms).float_support) }
    }
}

// cpp: layoutng/internal/layout_pass_scope.cc:38-42
impl Drop for LayoutPassScope {
    fn drop(&mut self) {
        DEPTH.with(|depth| {
            debug_assert!(depth.get() != 0);
            depth.set(depth.get().wrapping_sub(1));
        });
        ALGORITHMS.with(|current| current.set(self.previous_algorithms_));
        // object_factory_scope_ drops after this body and restores OBJECTS.
    }
}

// This is a borrow of the engine's existing native NodeId index during Layout,
// analogous to SVGResource's synchronous native target access. It owns neither
// the tree nor resource geometry, and cannot outlive the RunLayout guard.
thread_local! {static SVG_RESOURCE_TREE:Cell<*const super::layout_object_builder::LayoutObjectTree>=const {Cell::new(std::ptr::null())};}
pub(crate) struct SVGResourceTreeScope {
    previous: *const super::layout_object_builder::LayoutObjectTree,
}
impl SVGResourceTreeScope {
    pub(crate) fn new(tree: Option<&super::layout_object_builder::LayoutObjectTree>) -> Self {
        let pointer = tree.map_or(std::ptr::null(), |tree| tree as *const _);
        let previous = SVG_RESOURCE_TREE.with(|current| current.replace(pointer));
        Self { previous }
    }
}
impl Drop for SVGResourceTreeScope {
    fn drop(&mut self) {
        SVG_RESOURCE_TREE.with(|current| current.set(self.previous));
    }
}
impl LayoutPassScope {
    pub fn WithSVGResourceTree<R>(
        query: impl for<'a> FnOnce(&'a super::layout_object_builder::LayoutObjectTree) -> R,
    ) -> Option<R> {
        SVG_RESOURCE_TREE.with(|current| unsafe { current.get().as_ref() }.map(query))
    }
}

/// Synchronous read-query lease for SVG's native resource identity resolver.
/// The scope borrows the existing tree, never a cloned resource/geometry tree.
pub struct SVGResourceQueryScope<'a> {
    _scope: SVGResourceTreeScope,
    _owner: PhantomData<&'a super::layout_object_builder::LayoutObjectTree>,
}
impl<'a> SVGResourceQueryScope<'a> {
    pub fn new(tree: &'a super::layout_object_builder::LayoutObjectTree) -> Self {
        Self {
            _scope: SVGResourceTreeScope::new(Some(tree)),
            _owner: PhantomData,
        }
    }
}

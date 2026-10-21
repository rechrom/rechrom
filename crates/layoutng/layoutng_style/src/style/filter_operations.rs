use super::filter_operation::OperationType;
use super::forward::{FilterOperation, SVGResourceClient};
use foundation::{HeapVector, Member, RectF, Visitor, WtfSizeT};

impl foundation::Traceable for FilterOperations {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.operations_);
    }
}

impl foundation::Traceable for FilterOperationsWrapper {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FilterOperationsWrapper::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/filter_operations.h:59
pub type FilterOperationVector = HeapVector<Member<FilterOperation>>;

// cpp: layoutng_style/style/filter_operations.h:44-53
// cpp: layoutng_style/style/filter_operations.h:87-92
#[derive(Clone)]
pub struct FilterOperations {
    pub(super) operations_: FilterOperationVector,
}

#[allow(non_snake_case)]
impl FilterOperations {
    // cpp: layoutng_style/style/filter_operations.h:57
    pub fn clear(&mut self) {
        self.operations_.clear();
    }

    // cpp: layoutng_style/style/filter_operations.h:61
    pub fn OperationsMut(&mut self) -> &mut FilterOperationVector {
        &mut self.operations_
    }

    // cpp: layoutng_style/style/filter_operations.h:62
    pub fn Operations(&self) -> &FilterOperationVector {
        &self.operations_
    }

    // cpp: layoutng_style/style/filter_operations.h:64
    pub fn IsEmpty(&self) -> bool {
        self.operations_.size() == 0
    }

    // cpp: layoutng_style/style/filter_operations.h:65
    pub fn size(&self) -> WtfSizeT {
        self.operations_.size()
    }

    // cpp: layoutng_style/style/filter_operations.h:66-68
    pub fn at(&self, index: WtfSizeT) -> *const FilterOperation {
        if index < self.operations_.size() {
            self.operations_.at(index).Get()
        } else {
            std::ptr::null()
        }
    }

    // cpp: layoutng_style/style/filter_operations.h:70
    // cpp: core/style/filter_operations.cc:63-112
    pub fn CanInterpolateWith(&self, other: &Self) -> bool {
        // cpp: core/style/filter_operations.cc:63-82
        self.Operations()
            .iter()
            .chain(other.Operations().iter())
            .all(|operation| {
                FilterOperation::CanInterpolate(unsafe { &*operation.Get() }.GetType())
            })
            && self
                .Operations()
                .iter()
                .zip(other.Operations().iter())
                .all(|(a, b)| unsafe { &*a.Get() }.IsSameType(unsafe { &*b.Get() }))
    }

    // cpp: layoutng_style/style/filter_operations.h:72-75
    // Native effect mapping / SVG resource-client owner is not translated.
    pub fn MapRect(&self, rect: &RectF) -> RectF {
        unsafe { FilterOperationsMapRect(self, rect) }
    }

    // cpp: layoutng_style/style/filter_operations.h:77
    // cpp: core/style/filter_operations.cc:63-112
    pub fn HasFilterThatAffectsOpacity(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.AffectsOpacity())
    }

    // cpp: layoutng_style/style/filter_operations.h:78
    // cpp: core/style/filter_operations.cc:63-112
    pub fn HasFilterThatMovesPixels(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.MovesPixels())
    }

    // cpp: layoutng_style/style/filter_operations.h:79
    // cpp: core/style/filter_operations.cc:63-112
    pub fn HasReferenceFilter(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.GetType() == OperationType::kReference)
    }

    // cpp: layoutng_style/style/filter_operations.h:80
    // cpp: core/style/filter_operations.cc:63-112
    pub fn UsesCurrentColor(&self) -> bool {
        self.Operations()
            .iter()
            .any(|operation| unsafe { &*operation.Get() }.UsesCurrentColor())
    }

    // cpp: layoutng_style/style/filter_operations.h:82
    // Native effect mapping / SVG resource-client owner is not translated.
    pub fn AddClient(&self, client: &mut SVGResourceClient) {
        unsafe { FilterOperationsAddClient(self, client) }
    }

    // cpp: layoutng_style/style/filter_operations.h:83
    // Native effect mapping / SVG resource-client owner is not translated.
    pub fn RemoveClient(&self, client: &mut SVGResourceClient) {
        unsafe { FilterOperationsRemoveClient(self, client) }
    }
}

// cpp: layoutng_style/style/filter_operations.h:55
// cpp: core/style/filter_operations.cc:46-61
impl PartialEq for FilterOperations {
    fn eq(&self, other: &Self) -> bool {
        // cpp: core/style/filter_operations.cc:46-61
        self.Operations().len() == other.Operations().len()
            && self
                .Operations()
                .iter()
                .zip(other.Operations().iter())
                .all(|(a, b)| (unsafe { &*a.Get() }) == (unsafe { &*b.Get() }))
    }
}

// cpp: layoutng_style/style/filter_operations.h:97-111
#[derive(Clone)]
pub struct FilterOperationsWrapper {
    operations_: FilterOperations,
}

#[allow(non_snake_case)]
impl FilterOperationsWrapper {
    // cpp: layoutng_style/style/filter_operations.h:102-103
    pub fn new(operations: &FilterOperations) -> Self {
        Self {
            operations_: operations.clone(),
        }
    }

    // cpp: layoutng_style/style/filter_operations.h:105
    pub fn Operations(&self) -> &FilterOperations {
        &self.operations_
    }

    // cpp: layoutng_style/style/filter_operations.h:107
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.operations_);
    }
}

// cpp: layoutng_style/style/filter_operations.h:101
impl Default for FilterOperationsWrapper {
    fn default() -> Self {
        Self {
            operations_: FilterOperations::new(),
        }
    }
}

unsafe extern "Rust" {
    fn FilterOperationsMapRect(value: &FilterOperations, rect: &RectF) -> RectF;
    fn FilterOperationsAddClient(value: &FilterOperations, client: &mut SVGResourceClient);
    fn FilterOperationsRemoveClient(value: &FilterOperations, client: &mut SVGResourceClient);
}

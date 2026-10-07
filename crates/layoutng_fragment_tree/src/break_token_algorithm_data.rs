use foundation::Visitor;

// cpp: layoutng_fragment_tree/break_token_algorithm_data.h:21-30
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum BreakTokenAlgorithmDataType {
    kFieldsetData,
    kFlexData,
    kGridData,
    kGridLanesData,
    kTableData,
    kTableRowData,
    kMulticolData,
}

// C++ keeps this value in three bits; all seven declared values fit.
// Subclasses live in algorithm packages and embed this base value.
// cpp: layoutng_fragment_tree/break_token_algorithm_data.h:18-19
// cpp: layoutng_fragment_tree/break_token_algorithm_data.h:64-65
pub struct BreakTokenAlgorithmData {
    data_type: BreakTokenAlgorithmDataType,
}

#[allow(non_snake_case)]
impl BreakTokenAlgorithmData {
    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:33-34
    pub fn new(data_type: BreakTokenAlgorithmDataType) -> Self {
        Self { data_type }
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:31
    pub fn Type(&self) -> BreakTokenAlgorithmDataType {
        self.data_type
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:40
    pub fn IsFieldsetType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kFieldsetData
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:41
    pub fn IsFlexType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kFlexData
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:42
    pub fn IsGridType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kGridData
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:43
    pub fn IsGridLanesType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kGridLanesData
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:44
    pub fn IsTableType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kTableData
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:45
    pub fn IsTableRowType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kTableRowData
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:46
    pub fn IsMulticolType(&self) -> bool {
        self.Type() == BreakTokenAlgorithmDataType::kMulticolData
    }
}

// A trait object represents calls through the C++ virtual base pointer.
// The default virtual methods preserve the source's fatal NOTREACHED paths.
#[allow(non_snake_case)]
pub trait BreakTokenAlgorithmDataVirtual {
    fn base(&self) -> &BreakTokenAlgorithmData;

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:50
    fn GetTotalRowGapCount(&self) -> u32 {
        std::process::abort()
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:57-60
    fn GetFirstUnprocessedRowGapIndex(&self, _line_index: Option<u32>) -> u32 {
        std::process::abort()
    }

    // cpp: layoutng_fragment_tree/break_token_algorithm_data.h:62
    fn Trace(&self, _visitor: &mut Visitor) {}
}

impl BreakTokenAlgorithmDataVirtual for BreakTokenAlgorithmData {
    fn base(&self) -> &BreakTokenAlgorithmData {
        self
    }
}

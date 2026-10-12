use foundation::text::native::writing_mode_utils::{
    LogicalToLogical, LogicalToPhysical, LogicalToPhysicalGetter, LogicalToPhysicalSetter,
    PhysicalToLogical, PhysicalToLogicalGetter,
};
use foundation::text::native::{TextDirection, WritingDirectionMode, WritingMode};

const MODES: [WritingMode; 5] = [
    WritingMode::kHorizontalTb,
    WritingMode::kVerticalRl,
    WritingMode::kVerticalLr,
    WritingMode::kSidewaysRl,
    WritingMode::kSidewaysLr,
];
const DIRECTIONS: [TextDirection; 2] = [TextDirection::kLtr, TextDirection::kRtl];

#[test]
fn physical_logical_roundtrip_covers_every_mode_and_direction() {
    for writing_mode in MODES {
        for direction in DIRECTIONS {
            let mode = WritingDirectionMode::new(writing_mode, direction);
            let logical = PhysicalToLogical::new(mode, 1, 2, 3, 4);
            let physical = LogicalToPhysical::new(
                mode,
                logical.InlineStart(),
                logical.InlineEnd(),
                logical.BlockStart(),
                logical.BlockEnd(),
            );
            assert_eq!(
                [
                    physical.Top(),
                    physical.Right(),
                    physical.Bottom(),
                    physical.Left()
                ],
                [1, 2, 3, 4]
            );

            let same = LogicalToLogical::new(
                mode,
                mode,
                logical.InlineStart(),
                logical.InlineEnd(),
                logical.BlockStart(),
                logical.BlockEnd(),
            );
            assert_eq!(same.InlineStart(), logical.InlineStart());
            assert_eq!(same.InlineEnd(), logical.InlineEnd());
            assert_eq!(same.BlockStart(), logical.BlockStart());
            assert_eq!(same.BlockEnd(), logical.BlockEnd());
        }
    }
}

struct Sides {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

impl Sides {
    fn top(&self) -> i32 {
        self.top
    }
    fn right(&self) -> i32 {
        self.right
    }
    fn bottom(&self) -> i32 {
        self.bottom
    }
    fn left(&self) -> i32 {
        self.left
    }
    fn set_top(&mut self, value: i32) {
        self.top = value;
    }
    fn set_right(&mut self, value: i32) {
        self.right = value;
    }
    fn set_bottom(&mut self, value: i32) {
        self.bottom = value;
    }
    fn set_left(&mut self, value: i32) {
        self.left = value;
    }
}

#[test]
fn getter_and_setter_templates_preserve_delayed_access() {
    let mode = WritingDirectionMode::new(WritingMode::kVerticalRl, TextDirection::kLtr);
    let mut sides = Sides {
        top: 1,
        right: 2,
        bottom: 3,
        left: 4,
    };

    let logical_getter = PhysicalToLogicalGetter::new(
        mode,
        &sides,
        Sides::top,
        Sides::right,
        Sides::bottom,
        Sides::left,
    );
    assert_eq!(logical_getter.InlineStart(), 1);
    assert_eq!(logical_getter.BlockStart(), 2);

    let physical_getter = LogicalToPhysicalGetter::new(
        mode,
        &sides,
        Sides::top,
        Sides::right,
        Sides::bottom,
        Sides::left,
    );
    assert_eq!(physical_getter.Top(), 1);
    assert_eq!(physical_getter.Right(), 3);

    let mut setter = LogicalToPhysicalSetter::new(
        mode,
        &mut sides,
        Sides::set_top,
        Sides::set_right,
        Sides::set_bottom,
        Sides::set_left,
    );
    setter.SetInlineStart(10);
    setter.SetBlockStart(20);
    assert_eq!(sides.top, 10);
    assert_eq!(sides.right, 20);
}

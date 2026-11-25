// Out-of-line members of base-declared types whose Chromium bodies live in
// core/layout/inline/ (R5). Base declares the
// types because the fragment tree and LayoutInputNode name them; Rust cannot
// add inherent methods to them from this crate, so each family is an
// extension trait. Every inline module imports this prelude (and block's
// inline extension does too), so the sources keep Chromium's member-call
// syntax and the calls stay direct, statically dispatched calls inside this
// crate.
pub use crate::hyphen_result::HyphenResultAlgorithms;
pub use crate::inline_item::InlineItemAlgorithms;
pub use crate::inline_item_result::InlineItemResultAlgorithms;
pub use crate::inline_item_segment::InlineItemSegmentsAlgorithms;
pub use crate::inline_node::InlineNodeAlgorithms;
pub use crate::line_box_fragment_builder::FragmentBuilderInlineAlgorithms;

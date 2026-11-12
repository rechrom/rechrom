//! HTMLSourceElement media list matches through the production evaluator.
//! Source html_source_element.cc:157-162; media_query_list.cc:66-72.
use crate::{
    document_style_engine::DocumentStyleEngine, media_queries::MediaValuesCachedData,
    production_style_sheet::ProductionMediaQueryEvaluator,
};
impl DocumentStyleEngine {
    pub fn HTMLSourceMediaQueryMatches(
        &self,
        queries: &style_parser::production_media_query::HTMLSourceMediaQuerySet,
        media: &MediaValuesCachedData,
    ) -> bool {
        ProductionMediaQueryEvaluator::FromCachedData(media).EvalSet(queries)
    }
}

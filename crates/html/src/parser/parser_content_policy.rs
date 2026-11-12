// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! core/html/parser/parser_content_policy.h:36–58, pinned 6c1d401f.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParserContentPolicy {
    DisallowScriptingAndPluginContent,
    #[default]
    AllowScriptingContent,
    AllowScriptingContentAndDoNotMarkAlreadyStarted,
    AllowScriptingContentAndMarkAsParserInserted,
}
impl ParserContentPolicy {
    pub fn ScriptingContentIsAllowed(self) -> bool {
        self != Self::DisallowScriptingAndPluginContent
    }
    pub fn PluginContentIsAllowed(self) -> bool {
        self != Self::DisallowScriptingAndPluginContent
    }
}

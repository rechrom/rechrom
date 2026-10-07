// Link the Rust layout destructor boundary retained by DOM.
use layoutng_replaced as _;

use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMNodeType, PersistentDocument, DOM};
use html::html_parser::{HTMLDocumentParser, HTMLDocumentParserState, HTMLParserStatus, ParseHTML};
use html::{HTMLParserHost, ParserElementEvent, ParserElementPhase};
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Default)]
struct Host(Vec<(u64, String, ParserElementPhase)>);
impl HTMLParserHost for Host {
    fn HandleParserElement(&mut self, event: ParserElementEvent<'_>) {
        self.0.push((
            event.element.Id(),
            event.element.Name().to_owned(),
            event.phase,
        ));
    }
}
fn find(document: &PersistentDocument, id: &str) -> Option<usize> {
    fn visit(document: &PersistentDocument, index: usize, id: &str) -> Option<usize> {
        let node = document.Node(index);
        if node.FindAttribute("id").is_some_and(|a| a.value == id) {
            return Some(index);
        }
        node.Children()
            .iter()
            .find_map(|&child| visit(document, child, id))
    }
    visit(document, document.Root(), id)
}
fn snapshot(document: &PersistentDocument) -> String {
    fn visit(document: &PersistentDocument, index: usize, output: &mut String) {
        let node = document.Node(index);
        output.push_str(&format!(
            "({:?}:{:?}:{}:{:?}:{:?}",
            node.Type(),
            node.Namespace(),
            node.Name(),
            node.Data(),
            node.Attributes()
        ));
        for &child in node.Children() {
            visit(document, child, output);
        }
        if let Some(contents) = document.TemplateContents(index) {
            visit(document, contents, output);
        }
        output.push(')');
    }
    let mut output = String::new();
    visit(document, document.Root(), &mut output);
    output
}
fn finish(parser: &mut HTMLDocumentParserState, owner: &mut DOM, host: &mut Host) {
    for _ in 0..10000 {
        match parser.Pump(owner.GetDocumentMut(), host, 1).status {
            HTMLParserStatus::kFinished => return,
            HTMLParserStatus::kYielded => {}
            other => panic!("unexpected status after EOF: {other:?}"),
        }
    }
    panic!("parser did not finish");
}

#[test]
fn incremental_input_and_bounded_turns_match_continuous_parse() {
    // Boundaries cut tags, attributes, comments, entities, raw text and foreign
    // content. The tree-builder's formatting/table/template stacks also survive.
    let sources = [
        "<!doctype html><html><head><title>A&amp;B</title><style>p:before{content:'<x>'}</style></head><body><p id='early'>你好😀 &NotEqualTilde; &#x1F600;</p><!-- split --><textarea>A&lt;b</textarea></body></html>",
        "<body><p><b>one<i>two</b>three</i><div>four</div><table>foster<tr><td>A<td>B</table><template><table><tr><td>inside</table><span>tail</span></template>",
        "<body><svg viewbox='0 0 20 20'><clippath><rect/></clippath><foreignObject><p>html</p></foreignObject></svg><math><mi>x</mi><annotation-xml encoding='text/html'><div>y</div></annotation-xml></math>",
    ];
    for source in sources {
        let baseline = ParseHTML(source);
        for chunk_chars in [1, 2, 7, 31] {
            let mut owner = DOM::new();
            let root = owner.GetDocument().RootHandle();
            let mut host = Host::default();
            let mut parser = HTMLDocumentParserState::new(owner.GetDocumentMut(), &mut host);
            let chars: Vec<char> = source.chars().collect();
            for chunk in chars.chunks(chunk_chars) {
                parser.Append(&chunk.iter().collect::<String>());
                loop {
                    match parser.Pump(owner.GetDocumentMut(), &mut host, 1).status {
                        HTMLParserStatus::kYielded => {}
                        HTMLParserStatus::kNeedMoreInput => break,
                        other => panic!("unexpected status before EOF: {other:?}"),
                    }
                }
                assert_eq!(owner.GetDocument().RootHandle(), root);
                assert!(!parser.IsFinished());
            }
            parser.FinishInput();
            finish(&mut parser, &mut owner, &mut host);
            assert_eq!(
                snapshot(owner.GetDocument()),
                snapshot(baseline.GetDocument()),
                "chunk={chunk_chars}"
            );
            assert!(parser.IsFinished());
        }
    }
}

#[test]
fn dom_is_available_before_eof_and_identity_survives_host_mutation() {
    let mut owner = DOM::new();
    let mut host = Host::default();
    let mut parser = HTMLDocumentParserState::new(owner.GetDocumentMut(), &mut host);
    parser.Append("<body><p id=early>first</p><div id=la");
    assert_eq!(
        parser.Pump(owner.GetDocumentMut(), &mut host, 64).status,
        HTMLParserStatus::kNeedMoreInput
    );
    let early = find(owner.GetDocument(), "early").unwrap();
    let early_id = owner.GetDocument().Node(early).Id();
    assert!(find(owner.GetDocument(), "later").is_none());
    owner.GetDocumentMut().SetAttribute(
        early,
        DOMAttribute {
            local_name: "data-host".into(),
            value: "kept".into(),
            ..Default::default()
        },
    );
    parser.Append("ter>second</div>");
    parser.FinishInput();
    finish(&mut parser, &mut owner, &mut host);
    assert_eq!(owner.GetDocument().Node(early).Id(), early_id);
    assert_eq!(
        owner
            .GetDocument()
            .Node(early)
            .FindAttribute("data-host")
            .unwrap()
            .value,
        "kept"
    );
    assert!(find(owner.GetDocument(), "later").is_some());
    assert_eq!(
        host.0
            .iter()
            .filter(|(id, _, phase)| *id == early_id && *phase == ParserElementPhase::kInserted)
            .count(),
        1
    );
}

#[test]
fn script_pause_accepts_network_input_without_parsing_past_insertion_point() {
    let mut owner = DOM::new();
    let mut host = Host::default();
    let mut parser = HTMLDocumentParserState::new(owner.GetDocumentMut(), &mut host);
    parser.Append("<body><p id=before>one</p>\n<script id=first>external()</script>\n");
    let blocked = parser.Pump(owner.GetDocumentMut(), &mut host, 64);
    assert_eq!(blocked.status, HTMLParserStatus::kWaitingForScript);
    let script = blocked.script.unwrap();
    assert!(parser.IsPaused());
    assert!(find(owner.GetDocument(), "before").is_some());
    parser.Append("<div id=later>two</div><script id=second>next()</script>");
    parser.FinishInput();
    // Ready network data cannot advance the DOM before the blocking script runs.
    for _ in 0..3 {
        let again = parser.Pump(owner.GetDocumentMut(), &mut host, 64);
        assert_eq!(again.status, HTMLParserStatus::kWaitingForScript);
        assert_eq!(again.script, Some(script));
        assert!(find(owner.GetDocument(), "later").is_none());
    }
    parser.WithParser(owner.GetDocumentMut(), &mut host, |borrowed| {
        borrowed.WithPausedDocument(|d| assert!(find(d, "later").is_none()));
        borrowed.InsertFromScript("<b id=written>你好</b><script id=nested>nested()</script>");
        borrowed.ResumeAfterScript();
    });
    let nested = parser.Pump(owner.GetDocumentMut(), &mut host, 64);
    assert_eq!(nested.status, HTMLParserStatus::kWaitingForScript);
    assert_eq!(
        owner
            .GetDocument()
            .Node(find(owner.GetDocument(), "nested").unwrap())
            .Id(),
        nested.script.unwrap().node_id
    );
    assert!(find(owner.GetDocument(), "written").is_some());
    assert!(find(owner.GetDocument(), "later").is_none());
    parser.InsertFromScript("<i id=nested-write>three</i>");
    parser.ResumeAfterScript();
    let second = parser.Pump(owner.GetDocumentMut(), &mut host, 64);
    assert_eq!(second.status, HTMLParserStatus::kWaitingForScript);
    assert!(find(owner.GetDocument(), "nested-write").is_some());
    assert!(find(owner.GetDocument(), "later").is_some());
    assert_eq!(second.script.unwrap().start_position.line, 3);
    parser.ResumeAfterScript();
    finish(&mut parser, &mut owner, &mut host);
    // document.write content is inserted before the remainder of network input.
    let body = owner
        .GetDocument()
        .Node(owner.GetDocument().Root())
        .Children()
        .iter()
        .copied()
        .find(|&i| owner.GetDocument().Node(i).IsHTMLElement("html"))
        .unwrap();
    let body = owner
        .GetDocument()
        .Node(body)
        .Children()
        .iter()
        .copied()
        .find(|&i| owner.GetDocument().Node(i).IsHTMLElement("body"))
        .unwrap();
    let ids: Vec<_> = owner
        .GetDocument()
        .Node(body)
        .Children()
        .iter()
        .filter_map(|&i| {
            owner
                .GetDocument()
                .Node(i)
                .FindAttribute("id")
                .map(|a| a.value.as_str())
        })
        .collect();
    assert_eq!(
        ids,
        [
            "before",
            "first",
            "written",
            "nested",
            "nested-write",
            "later",
            "second"
        ]
    );
}

#[test]
fn head_whitespace_after_script_does_not_open_body() {
    let mut owner = DOM::new();
    let mut host = Host::default();
    let mut parser = HTMLDocumentParserState::new(owner.GetDocumentMut(), &mut host);
    parser.Append(
        "<html><head><script type=importmap>{}</script>\n\
         <link rel=stylesheet href=app.css></head><body><p>content</p>",
    );
    parser.FinishInput();
    loop {
        match parser.Pump(owner.GetDocumentMut(), &mut host, 1).status {
            HTMLParserStatus::kYielded => {}
            HTMLParserStatus::kWaitingForScript => {
                parser.WithParser(owner.GetDocumentMut(), &mut host, |borrowed| {
                    borrowed.ResumeAfterScript()
                })
            }
            HTMLParserStatus::kFinished => break,
            HTMLParserStatus::kNeedMoreInput => panic!("finished input requested more data"),
        }
    }
    let inserted = |name: &str| {
        host.0
            .iter()
            .position(|(_, element, phase)| {
                element == name && *phase == ParserElementPhase::kInserted
            })
            .unwrap()
    };
    assert!(inserted("link") < inserted("body"));
    let link = (0..owner.GetDocument().NodeCount())
        .find(|&index| owner.GetDocument().Node(index).IsHTMLElement("link"))
        .unwrap();
    let parent = owner.GetDocument().Node(link).Parent().unwrap();
    assert!(owner.GetDocument().Node(parent).IsHTMLElement("head"));
}

#[test]
fn continuation_survives_unwinding_and_rejects_a_different_document() {
    let mut owner = DOM::new();
    let mut other = DOM::new();
    let mut host = Host::default();
    let mut parser = HTMLDocumentParserState::new(owner.GetDocumentMut(), &mut host);
    parser.Append("<body><p id=kept>text</p>");
    assert!(catch_unwind(AssertUnwindSafe(|| parser.Pump(
        other.GetDocumentMut(),
        &mut host,
        1
    )))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| parser.WithParser(
        owner.GetDocumentMut(),
        &mut host,
        |_| panic!("host task unwound")
    )))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| parser.Pump(
        owner.GetDocumentMut(),
        &mut host,
        0
    )))
    .is_err());
    parser.FinishInput();
    finish(&mut parser, &mut owner, &mut host);
    assert!(find(owner.GetDocument(), "kept").is_some());
    assert_eq!(
        other
            .GetDocument()
            .Node(other.GetDocument().Root())
            .Children()
            .len(),
        0
    );
}

#[test]
fn fragment_continuation_preserves_foreign_context_and_open_stack() {
    let mut owner = DOM::new();
    let context =
        owner
            .GetDocumentMut()
            .CreateElement(DOMNamespace::kSVG, "svg".into(), Vec::new());
    let root = owner.GetDocument().Root();
    owner.GetDocumentMut().AppendChild(root, context);
    let mut host = Host::default();
    let mut parser =
        HTMLDocumentParserState::new_fragment(owner.GetDocumentMut(), &mut host, context);
    for text in [
        "<clip",
        "path><rect width='2",
        "0'/></clippath><foreignObject><p id=html>",
        "text</p></foreignObject>",
    ] {
        parser.Append(text);
        loop {
            match parser.Pump(owner.GetDocumentMut(), &mut host, 1).status {
                HTMLParserStatus::kNeedMoreInput => break,
                HTMLParserStatus::kYielded => {}
                other => panic!("unexpected status: {other:?}"),
            }
        }
    }
    parser.FinishInput();
    finish(&mut parser, &mut owner, &mut host);
    let clip = owner.GetDocument().Node(context).Children()[0];
    assert_eq!(owner.GetDocument().Node(clip).Name(), "clipPath");
    assert_eq!(
        owner.GetDocument().Node(clip).Namespace(),
        DOMNamespace::kSVG
    );
    let html = find(owner.GetDocument(), "html").unwrap();
    assert_eq!(
        owner.GetDocument().Node(html).Namespace(),
        DOMNamespace::kHTML
    );
    assert_eq!(owner.GetDocument().Node(html).Type(), DOMNodeType::kElement);
}

#[test]
fn borrowed_and_resident_interfaces_share_the_same_parser_semantics() {
    let mut baseline = DOM::new();
    let mut host = Host::default();
    let mut borrowed = HTMLDocumentParser::new(baseline.GetDocumentMut(), &mut host);
    borrowed.Append("<body><p id='sa");
    assert_eq!(borrowed.Pump(32).status, HTMLParserStatus::kNeedMoreInput);
    borrowed.Append("me'>text</p>");
    borrowed.FinishInput();
    assert_eq!(borrowed.Pump(32).status, HTMLParserStatus::kFinished);
    drop(borrowed);

    let mut owner = DOM::new();
    let root = owner.GetDocument().RootHandle();
    let mut parser = HTMLDocumentParserState::new(owner.GetDocumentMut(), &mut host);
    parser.Append("<body><p id='sa");
    assert_eq!(
        parser.Pump(owner.GetDocumentMut(), &mut host, 32).status,
        HTMLParserStatus::kNeedMoreInput
    );
    // This read was impossible while the old borrowed parser was still alive.
    assert_eq!(owner.GetDocument().RootHandle(), root);
    parser.Append("me'>text</p>");
    parser.FinishInput();
    finish(&mut parser, &mut owner, &mut host);
    assert_eq!(
        snapshot(owner.GetDocument()),
        snapshot(baseline.GetDocument())
    );
}

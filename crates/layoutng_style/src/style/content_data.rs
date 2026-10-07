use foundation::{AtomicString, MakeGarbageCollected, Member, StrCat, String, TreeScope, Visitor};

use super::computed_style_constants::QuoteType;
use super::forward::{
    CSSSymbolsValue, CounterStyle, CountersAttachmentContext, LayoutObject, StyleEngine,
};
use super::style_image::StyleImage;

// cpp: layoutng_style/style/content_data.h:48-95
pub struct ContentDataBase {
    next_: Option<Member<dyn ContentData>>,
}

impl Default for ContentDataBase {
    fn default() -> Self {
        Self { next_: None }
    }
}

#[allow(non_snake_case)]
pub trait ContentData {
    fn base(&self) -> &ContentDataBase;
    fn base_mut(&mut self) -> &mut ContentDataBase;

    // cpp: layoutng_style/style/content_data.h:52-59
    fn IsCounter(&self) -> bool {
        false
    }
    fn IsAltCounter(&self) -> bool {
        false
    }
    fn IsImage(&self) -> bool {
        false
    }
    fn IsQuote(&self) -> bool {
        false
    }
    fn IsText(&self) -> bool {
        false
    }
    fn IsAltText(&self) -> bool {
        false
    }
    fn IsNone(&self) -> bool {
        false
    }
    fn IsAlt(&self) -> bool {
        self.IsAltText() || self.IsAltCounter()
    }

    // cpp: layoutng_style/style/content_data.h:70
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject;

    // cpp: layoutng_style/style/content_data.h:74-79
    fn Next(&self) -> Option<*mut dyn ContentData> {
        self.base()
            .next_
            .as_ref()
            .and_then(|next| next.GetNonNull().map(|data| data.as_ptr()))
    }
    fn SetNext(&mut self, next: Option<*mut dyn ContentData>) {
        debug_assert!(!self.IsNone());
        debug_assert!(next.map_or(true, |pointer| unsafe { !(&*pointer).IsNone() }));
        self.base_mut().next_ = next.map(Member::from_ptr);
    }

    // cpp: layoutng_style/style/content_data.h:81
    fn Equals(&self, other: &dyn ContentData) -> bool;

    // cpp: layoutng_style/style/content_data.h:83
    // Base trace has no supplied definition.
    fn Trace(&self, visitor: &mut Visitor) {
        unsafe { ContentDataBaseTrace(self.base(), visitor) }
    }

    // cpp: layoutng_style/style/content_data.h:86
    fn DebugString(&self) -> String {
        String::from("<unknown>")
    }

    // cpp: layoutng_style/style/content_data.h:89
    fn CloneInternal(&self) -> *mut dyn ContentData;
}

// cpp: layoutng_style/style/content_data.h:62,64-65,72
#[allow(non_snake_case)]
pub fn HasAltCounterContent(value: &dyn ContentData) -> bool {
    unsafe { ContentDataHasAltCounterContent(value) }
}
#[allow(non_snake_case)]
pub fn ConcatenateAltText(first_alt_data: &dyn ContentData) -> String {
    unsafe { ContentDataConcatenateAltText(first_alt_data) }
}
#[allow(non_snake_case)]
pub fn CloneContentData(value: &dyn ContentData) -> *mut dyn ContentData {
    unsafe { ContentDataClone(value) }
}

// cpp: layoutng_style/style/content_data.h:97-107
impl std::fmt::Display for dyn ContentData + '_ {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "ContentData{{")?;
        let mut pointer: Option<&dyn ContentData> = Some(self);
        while let Some(current) = pointer {
            // The C++ stream inserts the original node's string each time.
            write!(formatter, "{},", self.DebugString())?;
            pointer = current.Next().map(|next| unsafe { &*next });
        }
        write!(formatter, "}}")
    }
}

// cpp: layoutng_style/style/content_data.h:385-395
impl PartialEq for dyn ContentData + '_ {
    fn eq(&self, other: &Self) -> bool {
        let mut left: Option<&dyn ContentData> = Some(self);
        let mut right: Option<&dyn ContentData> = Some(other);
        while let (Some(a), Some(b)) = (left, right) {
            if !a.Equals(b) {
                break;
            }
            left = a.Next().map(|next| unsafe { &*next });
            right = b.Next().map(|next| unsafe { &*next });
        }
        left.is_none() && right.is_none()
    }
}

// cpp: layoutng_style/style/content_data.h:400-412
#[allow(non_snake_case)]
pub fn ShouldUseContentDataForElement(content_data: Option<&dyn ContentData>) -> bool {
    let Some(content_data) = content_data else {
        return false;
    };
    if !content_data.IsImage() {
        return false;
    }
    if let Some(next) = content_data.Next() {
        if !unsafe { (&*next).IsAlt() } {
            return false;
        }
    }
    true
}

// cpp: layoutng_style/style/content_data.h:109-141
pub struct ImageContentData {
    base_: ContentDataBase,
    image_: Member<StyleImage>,
}

#[allow(non_snake_case)]
impl ImageContentData {
    // cpp: layoutng_style/style/content_data.h:111-113
    pub fn new(image: *mut StyleImage) -> Self {
        debug_assert!(!image.is_null());
        Self {
            base_: ContentDataBase::default(),
            image_: Member::from_ptr(image),
        }
    }
    // cpp: layoutng_style/style/content_data.h:115-120
    pub fn GetImage(&self) -> *mut StyleImage {
        self.image_.Get()
    }
    pub fn SetImage(&mut self, image: *mut StyleImage) {
        debug_assert!(!image.is_null());
        self.image_ = Member::from_ptr(image);
    }
    // cpp: layoutng_style/style/content_data.h:143-148
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsImage()
    }
}

#[allow(non_snake_case)]
impl ContentData for ImageContentData {
    fn base(&self) -> &ContentDataBase {
        &self.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.base_
    }
    // cpp: layoutng_style/style/content_data.h:122
    fn IsImage(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:123
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { ImageContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:125-128
    fn Equals(&self, data: &dyn ContentData) -> bool {
        if !data.IsImage() {
            return false;
        }
        let other = unsafe { &*(data as *const dyn ContentData as *const ImageContentData) };
        unsafe { &*other.GetImage() == &*self.GetImage() }
    }
    // cpp: layoutng_style/style/content_data.h:130
    fn Trace(&self, visitor: &mut Visitor) {
        unsafe { ImageContentDataTrace(self, visitor) }
    }
    // cpp: layoutng_style/style/content_data.h:132
    fn DebugString(&self) -> String {
        unsafe { ImageContentDataDebugString(self) }
    }
    // cpp: layoutng_style/style/content_data.h:135-138
    fn CloneInternal(&self) -> *mut dyn ContentData {
        MakeGarbageCollected(Self::new(self.GetImage())) as *mut dyn ContentData
    }
}

// cpp: layoutng_style/style/content_data.h:150-173
pub struct TextContentData {
    base_: ContentDataBase,
    text_: String,
}

#[allow(non_snake_case)]
impl TextContentData {
    // cpp: layoutng_style/style/content_data.h:152
    pub fn new(text: &String) -> Self {
        Self {
            base_: ContentDataBase::default(),
            text_: text.clone(),
        }
    }
    // cpp: layoutng_style/style/content_data.h:154-155
    pub fn GetText(&self) -> &String {
        &self.text_
    }
    pub fn SetText(&mut self, text: &String) {
        self.text_ = text.clone();
    }
    // cpp: layoutng_style/style/content_data.h:175-178
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsText()
    }
}

#[allow(non_snake_case)]
impl ContentData for TextContentData {
    fn base(&self) -> &ContentDataBase {
        &self.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.base_
    }
    // cpp: layoutng_style/style/content_data.h:157
    fn IsText(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:158
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { TextContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:160-163
    fn Equals(&self, data: &dyn ContentData) -> bool {
        if !data.IsText() {
            return false;
        }
        let other = unsafe { &*(data as *const dyn ContentData as *const TextContentData) };
        other.GetText() == self.GetText()
    }
    // cpp: layoutng_style/style/content_data.h:165
    fn DebugString(&self) -> String {
        self.text_.clone()
    }
    // cpp: layoutng_style/style/content_data.h:168-170
    fn CloneInternal(&self) -> *mut dyn ContentData {
        MakeGarbageCollected(Self::new(self.GetText())) as *mut dyn ContentData
    }
}

// cpp: layoutng_style/style/content_data.h:180-202
pub struct AltTextContentData {
    base_: ContentDataBase,
    text_: String,
}

#[allow(non_snake_case)]
impl AltTextContentData {
    // cpp: layoutng_style/style/content_data.h:182
    pub fn new(text: &String) -> Self {
        Self {
            base_: ContentDataBase::default(),
            text_: text.clone(),
        }
    }
    // cpp: layoutng_style/style/content_data.h:184-185
    pub fn GetText(&self) -> String {
        self.text_.clone()
    }
    pub fn SetText(&mut self, text: &String) {
        self.text_ = text.clone();
    }
    // cpp: layoutng_style/style/content_data.h:204-209
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsAltText()
    }
}

#[allow(non_snake_case)]
impl ContentData for AltTextContentData {
    fn base(&self) -> &ContentDataBase {
        &self.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.base_
    }
    // cpp: layoutng_style/style/content_data.h:187
    fn IsAltText(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:188
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { AltTextContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:190-193
    fn Equals(&self, data: &dyn ContentData) -> bool {
        if !data.IsAltText() {
            return false;
        }
        let other = unsafe { &*(data as *const dyn ContentData as *const AltTextContentData) };
        other.GetText() == self.GetText()
    }
    // cpp: layoutng_style/style/content_data.h:195
    fn DebugString(&self) -> String {
        StrCat(&[
            String::from("<alt: "),
            self.text_.clone(),
            String::from(">"),
        ])
    }
    // cpp: layoutng_style/style/content_data.h:198-200
    fn CloneInternal(&self) -> *mut dyn ContentData {
        MakeGarbageCollected(Self::new(&self.GetText())) as *mut dyn ContentData
    }
}

// cpp: layoutng_style/style/content_data.h:211-237
#[derive(Clone)]
pub struct CounterData {
    pub identifier: AtomicString,
    pub list_style: AtomicString,
    pub separator: AtomicString,
    pub tree_scope: Member<TreeScope>,
    pub symbols_counter_style: Member<CounterStyle>,
}

#[allow(non_snake_case)]
impl CounterData {
    // cpp: layoutng_style/style/content_data.h:215-224
    pub fn new(
        identifier: &AtomicString,
        style: &AtomicString,
        separator: &AtomicString,
        tree_scope: *const TreeScope,
        symbols_counter_style: *const CounterStyle,
    ) -> Self {
        Self {
            identifier: identifier.clone(),
            list_style: style.clone(),
            separator: separator.clone(),
            tree_scope: Member::from_ptr(tree_scope as *mut TreeScope),
            symbols_counter_style: Member::from_ptr(symbols_counter_style as *mut CounterStyle),
        }
    }
    // cpp: layoutng_style/style/content_data.h:226
    // No definition is supplied in this package.
    pub fn Trace(&self, visitor: &mut Visitor) {
        unsafe { CounterDataTrace(self, visitor) }
    }
}

// cpp: layoutng_style/style/content_data.h:239-281
pub struct CounterContentData {
    base_: ContentDataBase,
    pub(crate) counter_data_: CounterData,
}

#[allow(non_snake_case)]
impl CounterContentData {
    // cpp: layoutng_style/style/content_data.h:243-248
    // No definition is supplied in this package.
    pub fn new(
        identifier: &AtomicString,
        style: &AtomicString,
        separator: &AtomicString,
        tree_scope: *const TreeScope,
        symbols: *const CSSSymbolsValue,
    ) -> Self {
        unsafe { CounterContentDataConstruct(identifier, style, separator, tree_scope, symbols) }
    }

    // cpp: layoutng_style/style/content_data.h:250-251
    pub fn from_data(counter_data: CounterData) -> Self {
        Self {
            base_: ContentDataBase::default(),
            counter_data_: counter_data,
        }
    }

    // cpp: layoutng_style/style/content_data.h:256-263
    pub fn Identifier(&self) -> &AtomicString {
        &self.counter_data_.identifier
    }
    pub fn ListStyle(&self) -> &AtomicString {
        &self.counter_data_.list_style
    }
    pub fn Separator(&self) -> &AtomicString {
        &self.counter_data_.separator
    }
    pub fn GetTreeScope(&self) -> *const TreeScope {
        self.counter_data_.tree_scope.Get()
    }
    pub fn GetSymbolsCounterStyle(&self) -> *const CounterStyle {
        self.counter_data_.symbols_counter_style.Get()
    }

    // cpp: layoutng_style/style/content_data.h:265-266
    // No definition is supplied in this package.
    pub fn ResolveCounterStyle(&self, style_engine: &StyleEngine) -> &CounterStyle {
        unsafe { &*CounterContentDataResolveCounterStyle(self, style_engine) }
    }

    // cpp: layoutng_style/style/content_data.h:278
    // No definition is supplied in this package.
    pub fn EqualsCounter(&self, data: &dyn ContentData) -> bool {
        unsafe { CounterContentDataEquals(self, data) }
    }

    // cpp: layoutng_style/style/content_data.h:283-288
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsCounter()
    }
}

#[allow(non_snake_case)]
impl ContentData for CounterContentData {
    fn base(&self) -> &ContentDataBase {
        &self.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.base_
    }
    // cpp: layoutng_style/style/content_data.h:253
    fn IsCounter(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:254
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { CounterContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:278
    fn Equals(&self, data: &dyn ContentData) -> bool {
        self.EqualsCounter(data)
    }
    // cpp: layoutng_style/style/content_data.h:268
    fn Trace(&self, visitor: &mut Visitor) {
        unsafe { CounterContentDataTrace(self, visitor) }
    }
    // cpp: layoutng_style/style/content_data.h:270
    fn DebugString(&self) -> String {
        String::from("<counter>")
    }
    // cpp: layoutng_style/style/content_data.h:273-275
    fn CloneInternal(&self) -> *mut dyn ContentData {
        MakeGarbageCollected(Self::from_data(self.counter_data_.clone())) as *mut dyn ContentData
    }
}

// cpp: layoutng_style/style/content_data.h:290-322
pub struct AltCounterContentData {
    parent_: CounterContentData,
    counter_value_text_: String,
}

#[allow(non_snake_case)]
impl AltCounterContentData {
    // cpp: layoutng_style/style/content_data.h:291
    pub fn new(
        identifier: &AtomicString,
        style: &AtomicString,
        separator: &AtomicString,
        tree_scope: *const TreeScope,
        symbols: *const CSSSymbolsValue,
    ) -> Self {
        Self {
            parent_: CounterContentData::new(identifier, style, separator, tree_scope, symbols),
            counter_value_text_: String::default(),
        }
    }
    pub fn from_data(data: CounterData) -> Self {
        Self {
            parent_: CounterContentData::from_data(data),
            counter_value_text_: String::default(),
        }
    }

    // cpp: layoutng_style/style/content_data.h:298-301
    pub fn GetText(&self) -> &String {
        &self.counter_value_text_
    }
    // No definition is supplied in this package.
    pub fn UpdateText(
        &mut self,
        context: &mut CountersAttachmentContext,
        style_engine: &StyleEngine,
        content_generating_object: &LayoutObject,
    ) {
        unsafe {
            AltCounterContentDataUpdateText(self, context, style_engine, content_generating_object)
        }
    }

    // cpp: layoutng_style/style/content_data.h:306
    fn SetText(&mut self, text: &String) {
        self.counter_value_text_ = text.clone();
    }

    // cpp: layoutng_style/style/content_data.h:324-329
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsAltCounter()
    }
}

#[allow(non_snake_case)]
impl ContentData for AltCounterContentData {
    fn base(&self) -> &ContentDataBase {
        &self.parent_.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.parent_.base_
    }
    // cpp: layoutng_style/style/content_data.h:253,294
    fn IsCounter(&self) -> bool {
        true
    }
    fn IsAltCounter(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:296
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { AltCounterContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:303
    fn DebugString(&self) -> String {
        String::from("<alt-counter>")
    }
    // cpp: layoutng_style/style/content_data.h:308-312
    fn CloneInternal(&self) -> *mut dyn ContentData {
        let data = MakeGarbageCollected(Self::from_data(self.parent_.counter_data_.clone()));
        unsafe {
            (&mut *data).SetText(self.GetText());
        }
        data as *mut dyn ContentData
    }
    // cpp: layoutng_style/style/content_data.h:314-318
    fn Equals(&self, data: &dyn ContentData) -> bool {
        if !data.IsAltCounter() {
            return false;
        }
        let other = unsafe { &*(data as *const dyn ContentData as *const AltCounterContentData) };
        self.parent_.EqualsCounter(data) && self.GetText() == other.GetText()
    }
    // cpp: layoutng_style/style/content_data.h:268
    fn Trace(&self, visitor: &mut Visitor) {
        self.parent_.Trace(visitor);
    }
}

// cpp: layoutng_style/style/content_data.h:331-354
pub struct QuoteContentData {
    base_: ContentDataBase,
    quote_: QuoteType,
}

#[allow(non_snake_case)]
impl QuoteContentData {
    // cpp: layoutng_style/style/content_data.h:333
    pub fn new(quote: QuoteType) -> Self {
        Self {
            base_: ContentDataBase::default(),
            quote_: quote,
        }
    }
    // cpp: layoutng_style/style/content_data.h:335-336
    pub fn Quote(&self) -> QuoteType {
        self.quote_
    }
    pub fn SetQuote(&mut self, quote: QuoteType) {
        self.quote_ = quote;
    }
    // cpp: layoutng_style/style/content_data.h:356-361
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsQuote()
    }
}

#[allow(non_snake_case)]
impl ContentData for QuoteContentData {
    fn base(&self) -> &ContentDataBase {
        &self.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.base_
    }
    // cpp: layoutng_style/style/content_data.h:338
    fn IsQuote(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:339
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { QuoteContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:341-344
    fn Equals(&self, data: &dyn ContentData) -> bool {
        if !data.IsQuote() {
            return false;
        }
        let other = unsafe { &*(data as *const dyn ContentData as *const QuoteContentData) };
        other.Quote() == self.Quote()
    }
    // cpp: layoutng_style/style/content_data.h:346
    fn DebugString(&self) -> String {
        String::from("<quote>")
    }
    // cpp: layoutng_style/style/content_data.h:349-351
    fn CloneInternal(&self) -> *mut dyn ContentData {
        MakeGarbageCollected(Self::new(self.Quote())) as *mut dyn ContentData
    }
}

// cpp: layoutng_style/style/content_data.h:363-378
#[derive(Default)]
pub struct NoneContentData {
    base_: ContentDataBase,
}

#[allow(non_snake_case)]
impl NoneContentData {
    // cpp: layoutng_style/style/content_data.h:365
    pub fn new() -> Self {
        Self::default()
    }
    // cpp: layoutng_style/style/content_data.h:380-383
    pub fn AllowFrom(value: &dyn ContentData) -> bool {
        value.IsNone()
    }
}

#[allow(non_snake_case)]
impl ContentData for NoneContentData {
    fn base(&self) -> &ContentDataBase {
        &self.base_
    }
    fn base_mut(&mut self) -> &mut ContentDataBase {
        &mut self.base_
    }
    // cpp: layoutng_style/style/content_data.h:367
    fn IsNone(&self) -> bool {
        true
    }
    // cpp: layoutng_style/style/content_data.h:368
    fn CreateLayoutObject(&self, owner: &mut LayoutObject) -> *mut LayoutObject {
        unsafe { NoneContentDataCreateLayoutObject(self, owner) }
    }
    // cpp: layoutng_style/style/content_data.h:370
    fn Equals(&self, data: &dyn ContentData) -> bool {
        data.IsNone()
    }
    // cpp: layoutng_style/style/content_data.h:372
    fn DebugString(&self) -> String {
        String::from("<none>")
    }
    // cpp: layoutng_style/style/content_data.h:375-377
    fn CloneInternal(&self) -> *mut dyn ContentData {
        MakeGarbageCollected(Self::new()) as *mut dyn ContentData
    }
}

// cpp: layoutng_style/style/content_data.h:48-83,109-130,150-170,180-200,239-275,290-318,331-351,363-377
// GarbageCollected<T> dispatches to each class's virtual Trace method in C++.
// Keep that dispatch when these concrete values enter the Rust layout heap.
// Out-of-line Trace definitions absent from the supplied source remain the
// explicit extern calls above; this bridge does not replace them.
macro_rules! impl_content_data_traceable {
    ($($type:ty),+ $(,)?) => {$ (
        impl foundation::Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                ContentData::Trace(self, visitor);
            }
        }
    )+ };
}

impl_content_data_traceable!(
    ImageContentData,
    TextContentData,
    AltTextContentData,
    CounterContentData,
    AltCounterContentData,
    QuoteContentData,
    NoneContentData,
);

unsafe extern "Rust" {
    fn ContentDataHasAltCounterContent(value: &dyn ContentData) -> bool;
    fn ContentDataConcatenateAltText(value: &dyn ContentData) -> String;
    fn ContentDataClone(value: &dyn ContentData) -> *mut dyn ContentData;
    fn ContentDataBaseTrace(value: &ContentDataBase, visitor: &mut Visitor);
    fn ImageContentDataCreateLayoutObject(
        value: &ImageContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
    fn ImageContentDataTrace(value: &ImageContentData, visitor: &mut Visitor);
    fn ImageContentDataDebugString(value: &ImageContentData) -> String;
    fn TextContentDataCreateLayoutObject(
        value: &TextContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
    fn AltTextContentDataCreateLayoutObject(
        value: &AltTextContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
    fn CounterDataTrace(value: &CounterData, visitor: &mut Visitor);
    fn CounterContentDataConstruct(
        identifier: &AtomicString,
        style: &AtomicString,
        separator: &AtomicString,
        tree_scope: *const TreeScope,
        symbols: *const CSSSymbolsValue,
    ) -> CounterContentData;
    fn CounterContentDataCreateLayoutObject(
        value: &CounterContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
    fn CounterContentDataResolveCounterStyle(
        value: &CounterContentData,
        engine: &StyleEngine,
    ) -> *const CounterStyle;
    fn CounterContentDataTrace(value: &CounterContentData, visitor: &mut Visitor);
    fn CounterContentDataEquals(value: &CounterContentData, other: &dyn ContentData) -> bool;
    fn AltCounterContentDataCreateLayoutObject(
        value: &AltCounterContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
    fn AltCounterContentDataUpdateText(
        value: &mut AltCounterContentData,
        context: &mut CountersAttachmentContext,
        engine: &StyleEngine,
        object: &LayoutObject,
    );
    fn QuoteContentDataCreateLayoutObject(
        value: &QuoteContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
    fn NoneContentDataCreateLayoutObject(
        value: &NoneContentData,
        owner: &mut LayoutObject,
    ) -> *mut LayoutObject;
}

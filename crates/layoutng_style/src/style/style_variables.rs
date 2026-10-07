use super::forward::{CSSValue, CSSVariableData};
use foundation::{AtomicString, HashInts, HashSet, MakeGarbageCollected, Member, Visitor};

// C++ calls Data::Hash() for both supported value types.
pub trait TrieData: PartialEq + 'static {
    #[allow(non_snake_case)]
    fn Hash(&self) -> u32;
}

impl<Data: TrieData> foundation::Traceable for HashTrieNode<Data> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        HashTrieNode::<Data>::Trace(self, visitor);
    }
}

impl foundation::Traceable for StyleVariables {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        StyleVariables::Trace(self, visitor);
    }
}

impl TrieData for CSSVariableData {
    fn Hash(&self) -> u32 {
        CSSVariableData::Hash(self)
    }
}
impl TrieData for CSSValue {
    fn Hash(&self) -> u32 {
        CSSValue::Hash(self)
    }
}

// cpp: layoutng_style/style/style_variables.h:70-74,342-347
pub struct HashTrieNode<Data: TrieData> {
    keys_: [AtomicString; 16],
    values_: [Member<Data>; 16],
    children_: [Member<HashTrieNode<Data>>; 16],
    shared: bool,
}

// cpp: layoutng_style/style/style_variables.h:74,346
impl<Data: TrieData> Default for HashTrieNode<Data> {
    fn default() -> Self {
        Self {
            keys_: std::array::from_fn(|_| AtomicString::default()),
            values_: std::array::from_fn(|_| Member::default()),
            children_: std::array::from_fn(|_| Member::default()),
            shared: false,
        }
    }
}

#[allow(non_snake_case)]
impl<Data: TrieData> HashTrieNode<Data> {
    const FANOUT_BITS: u32 = 4;
    const NUM_SLOTS: usize = 1 << Self::FANOUT_BITS;
    const ALIGNMENT_BITS: u32 = if std::mem::size_of::<AtomicString>() == 8 {
        4
    } else {
        3
    };

    // cpp: layoutng_style/style/style_variables.h:78-94
    pub fn Get(&self, key: &AtomicString, shift: u32) -> Option<*mut Data> {
        let slot = Self::GetSlot(key, shift);
        if self.keys_[slot].IsNull() {
            let child = self.children_[slot].Get();
            if child.is_null() {
                None
            } else {
                unsafe { (&*child).Get(key, shift + Self::FANOUT_BITS) }
            }
        } else if self.keys_[slot] == *key {
            Some(self.values_[slot].Get())
        } else {
            None
        }
    }

    pub fn GetFromRoot(&self, key: &AtomicString) -> Option<*mut Data> {
        self.Get(key, Self::ALIGNMENT_BITS)
    }

    // cpp: layoutng_style/style/style_variables.h:99-164
    #[must_use]
    pub fn Set(
        &mut self,
        key: &AtomicString,
        value: *mut Data,
        hash: &mut u32,
        shift: u32,
    ) -> *mut Self {
        let slot = Self::GetSlot(key, shift);
        if !self.keys_[slot].IsNull() {
            if self.keys_[slot] == *key {
                if foundation::ValuesEquivalent(self.values_[slot].Get(), value) {
                    return self;
                }
                Self::UpdateHash(key, self.values_[slot].Get(), hash);
                Self::UpdateHash(key, value, hash);
                let new_this = if self.shared {
                    MakeGarbageCollected(Self::CopyNode(self))
                } else {
                    self as *mut Self
                };
                unsafe {
                    (&mut *new_this).values_[slot] = Member::from_ptr(value);
                }
                return new_this;
            }
            Self::UpdateHash(key, value, hash);
            if self.shared {
                let new_this = MakeGarbageCollected(Self::CopyNode(self));
                let child = self.CreateSplitNode(
                    self.keys_[slot].clone(),
                    self.values_[slot].Get(),
                    key.clone(),
                    value,
                    shift + Self::FANOUT_BITS,
                );
                unsafe {
                    (&mut *new_this).children_[slot] = Member::from_ptr(child);
                    (&mut *new_this).keys_[slot] = AtomicString::default();
                }
                return new_this;
            }
            let child = self.CreateSplitNode(
                self.keys_[slot].clone(),
                self.values_[slot].Get(),
                key.clone(),
                value,
                shift + Self::FANOUT_BITS,
            );
            self.children_[slot] = Member::from_ptr(child);
            self.keys_[slot] = AtomicString::default();
            return self;
        }
        let child = self.children_[slot].Get();
        if !child.is_null() {
            let new_child =
                unsafe { (&mut *child).Set(key, value, hash, shift + Self::FANOUT_BITS) };
            if new_child == child {
                return self;
            }
            let new_this = if self.shared {
                MakeGarbageCollected(Self::CopyNode(self))
            } else {
                self as *mut Self
            };
            unsafe {
                (&mut *new_this).children_[slot] = Member::from_ptr(new_child);
            }
            return new_this;
        }
        Self::UpdateHash(key, value, hash);
        let new_this = if self.shared {
            MakeGarbageCollected(Self::CopyNode(self))
        } else {
            self as *mut Self
        };
        unsafe {
            (&mut *new_this).keys_[slot] = key.clone();
            (&mut *new_this).values_[slot] = Member::from_ptr(value);
        }
        new_this
    }

    pub fn SetFromRoot(
        &mut self,
        key: &AtomicString,
        value: *mut Data,
        hash: &mut u32,
    ) -> *mut Self {
        self.Set(key, value, hash, Self::ALIGNMENT_BITS)
    }

    // cpp: layoutng_style/style/style_variables.h:166-178
    pub fn empty(&self) -> bool {
        for key in &self.keys_ {
            if !key.IsNull() {
                return false;
            }
        }
        for child in &self.children_ {
            if !child.Get().is_null() {
                return false;
            }
        }
        true
    }

    // cpp: layoutng_style/style/style_variables.h:221-228
    pub fn Trace(&self, visitor: &mut Visitor) {
        for value in &self.values_ {
            visitor.Trace(value);
        }
        for child in &self.children_ {
            visitor.Trace(child);
        }
    }

    // cpp: layoutng_style/style/style_variables.h:232-242
    pub fn MakeShared(&mut self) {
        if self.shared {
            return;
        }
        for child in &self.children_ {
            let ptr = child.Get();
            if !ptr.is_null() {
                unsafe {
                    (&mut *ptr).MakeShared();
                }
            }
        }
        self.shared = true;
    }

    // cpp: layoutng_style/style/style_variables.h:244-255
    pub fn CollectNames(&self, names: &mut HashSet<AtomicString>) {
        for key in &self.keys_ {
            if !key.IsNull() {
                names.insert(key.clone());
            }
        }
        for child in &self.children_ {
            let ptr = child.Get();
            if !ptr.is_null() {
                unsafe {
                    (&*ptr).CollectNames(names);
                }
            }
        }
    }

    // cpp: layoutng_style/style/style_variables.h:259-271
    pub fn Serialize<Printer, Output>(
        &self,
        printer: &Printer,
        stream: &mut Output,
    ) -> std::fmt::Result
    where
        Printer: Fn(&Member<Data>) -> std::string::String,
        Output: std::fmt::Write,
    {
        for i in 0..Self::NUM_SLOTS {
            if !self.keys_[i].IsNull() {
                write!(stream, "{}: {}, ", self.keys_[i], printer(&self.values_[i]))?;
            }
        }
        for child in &self.children_ {
            let ptr = child.Get();
            if !ptr.is_null() {
                unsafe {
                    (&*ptr).Serialize(printer, stream)?;
                }
            }
        }
        Ok(())
    }

    // cpp: layoutng_style/style/style_variables.h:274-280
    fn CopyNode(other: &Self) -> Self {
        Self {
            keys_: other.keys_.clone(),
            values_: other.values_.clone(),
            children_: other.children_.clone(),
            shared: false,
        }
    }

    // cpp: layoutng_style/style/style_variables.h:283-296
    fn GetSlot(key: &AtomicString, shift: u32) -> usize {
        let pointer = key.Impl() as usize;
        debug_assert_eq!(pointer & ((1usize << Self::ALIGNMENT_BITS) - 1), 0);
        debug_assert!((shift as usize) < usize::BITS as usize);
        (pointer >> shift) & (Self::NUM_SLOTS - 1)
    }

    // cpp: layoutng_style/style/style_variables.h:299-303
    fn UpdateHash(key: &AtomicString, value: *mut Data, hash: &mut u32) {
        if !value.is_null() {
            *hash ^= HashInts(key.Hash(), unsafe { (&*value).Hash() });
        }
    }

    // cpp: layoutng_style/style/style_variables.h:307-326
    fn CreateSplitNode(
        &self,
        key1: AtomicString,
        value1: *mut Data,
        key2: AtomicString,
        value2: *mut Data,
        shift: u32,
    ) -> *mut Self {
        let node = MakeGarbageCollected(Self::default());
        let slot1 = Self::GetSlot(&key1, shift);
        let slot2 = Self::GetSlot(&key2, shift);
        unsafe {
            if slot1 == slot2 {
                let child =
                    self.CreateSplitNode(key1, value1, key2, value2, shift + Self::FANOUT_BITS);
                (&mut *node).children_[slot1] = Member::from_ptr(child);
            } else {
                (&mut *node).keys_[slot1] = key1;
                (&mut *node).values_[slot1] = Member::from_ptr(value1);
                (&mut *node).keys_[slot2] = key2;
                (&mut *node).values_[slot2] = Member::from_ptr(value2);
            }
        }
        node
    }
}

// cpp: layoutng_style/style/style_variables.h:180-219
impl<Data: TrieData> PartialEq for HashTrieNode<Data> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..Self::NUM_SLOTS {
            if self.keys_[i].Impl() != other.keys_[i].Impl() {
                return false;
            }
        }
        for i in 0..Self::NUM_SLOTS {
            let left = self.values_[i].Get();
            let right = other.values_[i].Get();
            if left != right {
                if left.is_null() || right.is_null() {
                    return false;
                }
                if unsafe { &*left != &*right } {
                    return false;
                }
            }
        }
        for i in 0..Self::NUM_SLOTS {
            let left = self.children_[i].Get();
            let right = other.children_[i].Get();
            if left != right {
                if left.is_null() || right.is_null() {
                    return false;
                }
                if unsafe { &*left != &*right } {
                    return false;
                }
            }
        }
        true
    }
}

// cpp: layoutng_style/style/style_variables.h:371-430
pub struct StyleVariables {
    data_root_: Member<HashTrieNode<CSSVariableData>>,
    values_root_: Member<HashTrieNode<CSSValue>>,
    data_hash_: u32,
    values_hash_: u32,
}

// cpp: layoutng_style/style/style_variables.h:375-377
impl Default for StyleVariables {
    fn default() -> Self {
        Self {
            data_root_: Member::from_ptr(MakeGarbageCollected(HashTrieNode::default())),
            values_root_: Member::from_ptr(MakeGarbageCollected(HashTrieNode::default())),
            data_hash_: 0,
            values_hash_: 0,
        }
    }
}

// cpp: layoutng_style/style/style_variables.h:378-385
impl Clone for StyleVariables {
    fn clone(&self) -> Self {
        let result = Self {
            data_root_: self.data_root_.clone(),
            values_root_: self.values_root_.clone(),
            data_hash_: self.data_hash_,
            values_hash_: self.values_hash_,
        };
        unsafe {
            (&mut *result.data_root_.Get()).MakeShared();
            (&mut *result.values_root_.Get()).MakeShared();
        }
        result
    }
}

#[allow(non_snake_case)]
impl StyleVariables {
    // cpp: layoutng_style/style/style_variables.h:387-395
    pub fn CopyAssign(&mut self, other: &Self) -> &mut Self {
        self.data_root_ = other.data_root_.clone();
        self.values_root_ = other.values_root_.clone();
        self.data_hash_ = other.data_hash_;
        self.values_hash_ = other.values_hash_;
        unsafe {
            (&mut *self.data_root_.Get()).MakeShared();
            (&mut *self.values_root_.Get()).MakeShared();
        }
        self
    }

    // cpp: layoutng_style/style/style_variables.h:398-401
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.data_root_);
        visitor.Trace(&self.values_root_);
    }

    // cpp: layoutng_style/style/style_variables.h:405-410
    pub fn GetData(&self, name: &AtomicString) -> Option<*mut CSSVariableData> {
        unsafe { (&*self.data_root_.Get()).GetFromRoot(name) }
    }
    pub fn GetValue(&self, name: &AtomicString) -> Option<*const CSSValue> {
        unsafe {
            (&*self.values_root_.Get())
                .GetFromRoot(name)
                .map(|value| value as *const CSSValue)
        }
    }

    // cpp: layoutng_style/style/style_variables.h:411-412
    // No definitions are supplied in this package.
    pub fn SetData(&mut self, name: &AtomicString, value: *mut CSSVariableData) {
        unsafe {
            StyleVariablesSetData(self, name, value);
        }
    }
    pub fn SetValue(&mut self, name: &AtomicString, value: *const CSSValue) {
        unsafe {
            StyleVariablesSetValue(self, name, value);
        }
    }

    // cpp: layoutng_style/style/style_variables.h:414-415
    // No definitions are supplied in this package.
    pub fn IsEmpty(&self) -> bool {
        unsafe { StyleVariablesIsEmpty(self) }
    }
    pub fn CollectNames(&self, names: &mut HashSet<AtomicString>) {
        unsafe {
            StyleVariablesCollectNames(self, names);
        }
    }

    // cpp: layoutng_style/style/style_variables.h:417
    pub fn GetHash(&self) -> u32 {
        HashInts(self.data_hash_, self.values_hash_)
    }
}

// cpp: layoutng_style/style/style_variables.h:403
impl PartialEq for StyleVariables {
    fn eq(&self, other: &Self) -> bool {
        unsafe { StyleVariablesEquals(self, other) }
    }
}

// cpp: layoutng_style/style/style_variables.h:428-433
impl std::fmt::Display for StyleVariables {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { StyleVariablesFormat(self, formatter) }
    }
}

// cpp: layoutng_style/style/style_variables.h:435-439
impl foundation::ThreadingTrait for HashTrieNode<CSSVariableData> {
    const kAffinity: foundation::ThreadAffinity = foundation::ThreadAffinity::kMainThreadOnly;
}

unsafe extern "Rust" {
    fn StyleVariablesSetData(
        value: &mut StyleVariables,
        name: &AtomicString,
        data: *mut CSSVariableData,
    );
    fn StyleVariablesSetValue(
        value: &mut StyleVariables,
        name: &AtomicString,
        data: *const CSSValue,
    );
    fn StyleVariablesIsEmpty(value: &StyleVariables) -> bool;
    fn StyleVariablesCollectNames(value: &StyleVariables, names: &mut HashSet<AtomicString>);
    fn StyleVariablesEquals(value: &StyleVariables, other: &StyleVariables) -> bool;
    fn StyleVariablesFormat(
        value: &StyleVariables,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;
}

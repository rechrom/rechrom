// Included in quickjs_adapter: ownership stays inside the native backend; the
// serialized format contains no JSValue, object pointer, atom or realm handle.
// Source: V8 value-serializer.cc:604-1223,1317-1341,1882-2467,2670+.
use crate::serialized_script_value::SerializedScriptValue;
pub(super) enum CloneFailure {
    DataClone(&'static str),
    Exception(Value),
}
impl From<Value> for CloneFailure {
    fn from(value: Value) -> Self {
        Self::Exception(value)
    }
}
type CloneResult<T> = Result<T, CloneFailure>;
const CLONE_HEADER: &[u8] = b"RSC\0\x01";
const C_UNDEFINED: u8 = 0;
const C_NULL: u8 = 1;
const C_FALSE: u8 = 2;
const C_TRUE: u8 = 3;
const C_NUMBER: u8 = 4;
const C_STRING: u8 = 5;
const C_BIGINT: u8 = 6;
const C_REFERENCE: u8 = 7;
const C_OBJECT: u8 = 16;
const C_ARRAY: u8 = 17;
const C_DATE: u8 = 18;
const C_BOXED: u8 = 19;
const C_MAP: u8 = 20;
const C_SET: u8 = 21;
const C_BUFFER: u8 = 22;
const C_VIEW: u8 = 23;
const C_REGEXP: u8 = 24;
const C_ERROR: u8 = 25;
impl Context {
    pub(super) fn serialize_for_storage(
        &mut self,
        value: &Value,
    ) -> CloneResult<SerializedScriptValue> {
        self.validate_value(value)?;
        let mut writer = CloneWriter {
            bytes: CLONE_HEADER.to_vec(),
            seen: HashMap::new(),
        };
        writer.value(self, value)?;
        Ok(SerializedScriptValue::FromBytes(writer.bytes))
    }
    pub(super) fn deserialize_storage(
        &mut self,
        storage: &SerializedScriptValue,
    ) -> CloneResult<Value> {
        let bytes = storage.Bytes();
        if !bytes.starts_with(CLONE_HEADER) {
            return Err(CloneFailure::DataClone(
                "Unsupported structured storage version",
            ));
        }
        let mut reader = CloneReader {
            bytes,
            offset: CLONE_HEADER.len(),
            objects: Vec::new(),
        };
        let value = reader.value(self)?;
        if reader.offset != bytes.len() {
            return Err(CloneFailure::DataClone("Trailing structured storage data"));
        }
        Ok(value)
    }
    fn clone_check_stack(&mut self) -> CloneResult<()> {
        if unsafe { JS_CloneCheckStack(self.owner.ctx) } < 0 {
            Err(CloneFailure::Exception(self.take_exception()))
        } else {
            Ok(())
        }
    }
    fn clone_keys(&mut self, value: &Value, skip_indices: bool) -> JsResult<Vec<JsString>> {
        unsafe {
            let mut props = ptr::null_mut();
            let mut len = 0;
            if JS_CloneOwnPropertyNames(
                self.owner.ctx,
                &mut props,
                &mut len,
                value.raw(),
                skip_indices,
            ) < 0
            {
                return Err(self.take_exception());
            }
            let mut keys = Vec::with_capacity(len as usize);
            for i in 0..len as usize {
                let key = self.accept(JS_AtomToString(self.owner.ctx, (*props.add(i)).atom));
                match key {
                    Ok(Value::Str(key)) => keys.push(key),
                    Ok(_) => unreachable!(),
                    Err(error) => {
                        JS_FreePropertyEnum(self.owner.ctx, props, len);
                        return Err(error);
                    }
                }
            }
            JS_FreePropertyEnum(self.owner.ctx, props, len);
            Ok(keys)
        }
    }
    /// Own lookup after snapshotting keys, including a property's deletion by
    /// an earlier getter. Never falls through to an inherited getter/value.
    fn clone_own(
        &mut self,
        value: &Value,
        key: &JsString,
        invoke_getter: bool,
    ) -> JsResult<Option<Value>> {
        unsafe {
            let atom = self.key_atom(&PropKey::Str(key.clone()))?;
            let mut desc: JSPropertyDescriptor = std::mem::zeroed();
            let present = JS_GetOwnProperty(self.owner.ctx, &mut desc, value.raw(), atom);
            JS_FreeAtom(self.owner.ctx, atom);
            if present < 0 {
                return Err(self.take_exception());
            }
            if present == 0 {
                return Ok(None);
            }
            let result = if desc.flags & JS_PROP_GETSET != 0 {
                JS_FreeValue(self.owner.ctx, desc.value);
                if !invoke_getter {
                    Ok(None)
                } else if JS_IsUndefined(desc.getter) != 0 {
                    Ok(Some(Value::Undefined))
                } else {
                    self.accept(JS_Call(
                        self.owner.ctx,
                        desc.getter,
                        value.raw(),
                        0,
                        ptr::null_mut(),
                    ))
                    .map(Some)
                }
            } else {
                self.accept(desc.value).map(Some)
            };
            JS_FreeValue(self.owner.ctx, desc.getter);
            JS_FreeValue(self.owner.ctx, desc.setter);
            result
        }
    }
    fn clone_define(
        &mut self,
        object: &Value,
        key: &[u16],
        value: &Value,
        flags: i32,
    ) -> JsResult<()> {
        unsafe {
            let key = self.intern_utf16(key);
            let atom = self.key_atom(&PropKey::Str(key))?;
            let result = JS_DefinePropertyValue(
                self.owner.ctx,
                object.raw(),
                atom,
                JS_DupValue(self.owner.ctx, value.raw()),
                flags | JS_PROP_THROW,
            );
            JS_FreeAtom(self.owner.ctx, atom);
            if result < 0 {
                Err(self.take_exception())
            } else {
                Ok(())
            }
        }
    }
}
struct CloneWriter {
    bytes: Vec<u8>,
    // Handles pin every visited identity until serialization ends: an author
    // getter may remove a node and allocate another at the same raw address.
    seen: HashMap<(usize, usize), (u32, Value)>,
}
impl CloneWriter {
    fn byte(&mut self, n: u8) {
        self.bytes.push(n);
    }
    fn u32(&mut self, n: u32) {
        self.bytes.extend_from_slice(&n.to_le_bytes());
    }
    fn f64(&mut self, n: f64) {
        self.bytes.extend_from_slice(&n.to_bits().to_le_bytes());
    }
    fn length(&mut self, n: usize) -> CloneResult<()> {
        let n = u32::try_from(n)
            .map_err(|_| CloneFailure::DataClone("Structured storage value is too large"))?;
        self.u32(n);
        Ok(())
    }
    fn string(&mut self, s: &[u16]) -> CloneResult<()> {
        self.length(s.len())?;
        for unit in s {
            self.bytes.extend_from_slice(&unit.to_le_bytes());
        }
        Ok(())
    }
    fn property_name(ctx: &mut Context, value: &Value, name: &str) -> JsResult<Value> {
        let key = PropKey::Str(ctx.intern(name));
        ctx.get_property(value, &key)
    }
    fn value(&mut self, ctx: &mut Context, value: &Value) -> CloneResult<()> {
        ctx.clone_check_stack()?;
        match value {
            Value::Undefined => self.byte(C_UNDEFINED),
            Value::Null => self.byte(C_NULL),
            Value::Bool(v) => self.byte(if *v { C_TRUE } else { C_FALSE }),
            Value::Int(v) => {
                self.byte(C_NUMBER);
                self.f64(*v as f64);
            }
            Value::Float(v) => {
                self.byte(C_NUMBER);
                self.f64(*v);
            }
            Value::Str(v) => {
                self.byte(C_STRING);
                self.string(&v.to_utf16())?;
            }
            Value::Object(object) => {
                let identity = object.identity();
                if let Some((id, _)) = self.seen.get(&identity) {
                    let id = *id;
                    self.byte(C_REFERENCE);
                    self.u32(id);
                    return Ok(());
                }
                let id = u32::try_from(self.seen.len())
                    .map_err(|_| CloneFailure::DataClone("Too many structured storage objects"))?;
                self.seen.insert(identity, (id, value.clone()));
                let kind = unsafe { JS_CloneKind(ctx.owner.ctx, value.raw()) };
                let tag = match kind {
                    JSCloneKind::Object => C_OBJECT,
                    JSCloneKind::Array => C_ARRAY,
                    JSCloneKind::Date => C_DATE,
                    JSCloneKind::Boxed => C_BOXED,
                    JSCloneKind::Map => C_MAP,
                    JSCloneKind::Set => C_SET,
                    JSCloneKind::Buffer => C_BUFFER,
                    JSCloneKind::View => C_VIEW,
                    JSCloneKind::RegExp => C_REGEXP,
                    JSCloneKind::Error => C_ERROR,
                    _ => {
                        return Err(CloneFailure::DataClone(
                            "Value could not be cloned for storage",
                        ))
                    }
                };
                self.byte(tag);
                self.u32(id);
                match kind {
                    JSCloneKind::Object | JSCloneKind::Array => {
                        let length = if kind == JSCloneKind::Array {
                            let length = Self::property_name(ctx, value, "length")?;
                            let length = length.as_f64().unwrap() as u32;
                            self.u32(length);
                            Some(length)
                        } else {
                            None
                        };
                        let packed = length
                            .is_some_and(|len| unsafe { JS_CloneArrayIsPacked(value.raw(), len) });
                        let count_at = self.bytes.len();
                        self.u32(0);
                        let mut count = 0u32;
                        if packed {
                            for index in 0..length.unwrap() {
                                let key = ctx.intern(&index.to_string());
                                if let Some(item) = ctx.clone_own(value, &key, true)? {
                                    self.string(&key.to_utf16())?;
                                    self.value(ctx, &item)?;
                                    count += 1;
                                }
                            }
                        }
                        // Dense arrays snapshot named properties only after all
                        // elements; sparse arrays/objects snapshot their keys first.
                        let keys = ctx.clone_keys(value, packed)?;
                        for key in keys {
                            if let Some(item) = ctx.clone_own(value, &key, true)? {
                                self.string(&key.to_utf16())?;
                                self.value(ctx, &item)?;
                                count += 1;
                            }
                        }
                        self.bytes[count_at..count_at + 4].copy_from_slice(&count.to_le_bytes());
                    }
                    JSCloneKind::Date | JSCloneKind::Boxed => {
                        let inner = unsafe {
                            Value::take(&ctx.owner, JS_CloneObjectData(ctx.owner.ctx, value.raw()))
                        };
                        if kind == JSCloneKind::Date {
                            self.f64(inner.as_f64().unwrap());
                        } else {
                            self.value(ctx, &inner)?;
                        }
                    }
                    JSCloneKind::Map | JSCloneKind::Set => {
                        // Capture the native insertion-order entries before any
                        // recursive Get can mutate the original collection.
                        let entries = unsafe { JS_CloneMapEntries(ctx.owner.ctx, value.raw()) }
                            .into_iter()
                            .map(|raw| unsafe { Value::take(&ctx.owner, raw) })
                            .collect::<Vec<_>>();
                        self.length(entries.len())?;
                        for entry in entries {
                            self.value(ctx, &entry)?;
                        }
                    }
                    JSCloneKind::Buffer => {
                        let (length, max) = unsafe { JS_CloneBuffer(value.raw()) }.ok_or(
                            CloneFailure::DataClone("Detached ArrayBuffer cannot be stored"),
                        )?;
                        self.byte(max.is_some() as u8);
                        if let Some(max) = max {
                            self.u32(max);
                        }
                        self.u32(length);
                        unsafe {
                            JS_CloneCopyBufferBytes(value.raw(), &mut self.bytes);
                        }
                    }
                    JSCloneKind::View => {
                        let (kind, buffer, offset, length, tracking) =
                            unsafe { JS_CloneView(ctx.owner.ctx, value.raw()) }.ok_or(
                                CloneFailure::DataClone("Detached or out-of-bounds buffer view"),
                            )?;
                        let buffer = unsafe { Value::take(&ctx.owner, buffer) };
                        self.byte(kind);
                        self.u32(offset);
                        self.u32(length);
                        self.byte(tracking as u8);
                        self.value(ctx, &buffer)?;
                    }
                    JSCloneKind::RegExp => {
                        let (pattern, flags) =
                            unsafe { JS_CloneRegExp(ctx.owner.ctx, value.raw()) };
                        let pattern = unsafe { Value::take(&ctx.owner, pattern) };
                        self.string(&pattern.as_string().unwrap().to_utf16())?;
                        self.u32(flags & !(1 << 7)); // native named-groups metadata is not a JS flag.
                    }
                    JSCloneKind::Error => self.error(ctx, value)?,
                    _ => unreachable!(),
                }
            }
            Value::Other(_) if unsafe { JS_IsBigInt(ctx.owner.ctx, value.raw()) } != 0 => {
                self.byte(C_BIGINT);
                let text = ctx.to_js_string(value)?;
                self.string(&text.to_utf16())?;
            }
            _ => {
                return Err(CloneFailure::DataClone(
                    "Executable, symbol or unsupported host value cannot be stored",
                ))
            }
        }
        Ok(())
    }
    fn error(&mut self, ctx: &mut Context, value: &Value) -> CloneResult<()> {
        // V8 snapshots data descriptors before Get(name)/Get(stack). Accessor
        // message/cause descriptors are not invoked or serialized.
        let key = ctx.intern("message");
        let message = ctx.clone_own(value, &key, false)?;
        let key = ctx.intern("cause");
        let cause = ctx.clone_own(value, &key, false)?;
        let name = Self::property_name(ctx, value, "name")?;
        let name = ctx.to_js_string(&name)?.to_utf16();
        let names = [
            "EvalError",
            "RangeError",
            "ReferenceError",
            "SyntaxError",
            "TypeError",
            "URIError",
        ];
        let kind = names
            .iter()
            .position(|n| n.encode_utf16().eq(name.iter().copied()))
            .map_or(0, |i| i as u8 + 1);
        self.byte(kind);
        let message = message.map(|value| ctx.to_js_string(&value)).transpose()?;
        self.byte(message.is_some() as u8);
        if let Some(message) = message {
            self.string(&message.to_utf16())?;
        }
        let stack = Self::property_name(ctx, value, "stack")?;
        self.byte(stack.as_string().is_some() as u8);
        if let Some(stack) = stack.as_string() {
            self.string(&stack.to_utf16())?;
        }
        self.byte(cause.is_some() as u8);
        if let Some(cause) = cause {
            self.value(ctx, &cause)?;
        }
        Ok(())
    }
}
struct CloneReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    objects: Vec<Option<Value>>,
}
impl<'a> CloneReader<'a> {
    fn bad() -> CloneFailure {
        CloneFailure::DataClone("Malformed structured storage value")
    }
    fn take(&mut self, n: usize) -> CloneResult<&'a [u8]> {
        let end = self
            .offset
            .checked_add(n)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(Self::bad)?;
        let bytes = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> CloneResult<u8> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> CloneResult<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Self::bad()),
        }
    }
    fn u32(&mut self) -> CloneResult<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> CloneResult<f64> {
        Ok(f64::from_bits(u64::from_le_bytes(
            self.take(8)?.try_into().unwrap(),
        )))
    }
    fn string(&mut self) -> CloneResult<Vec<u16>> {
        let len = self.u32()? as usize;
        let bytes = self.take(len.checked_mul(2).ok_or_else(Self::bad)?)?;
        Ok(bytes
            .chunks_exact(2)
            .map(|unit| u16::from_le_bytes([unit[0], unit[1]]))
            .collect())
    }
    fn register(&mut self, id: u32, value: Value) -> CloneResult<Value> {
        let slot = self.objects.get_mut(id as usize).ok_or_else(Self::bad)?;
        if slot.is_some() {
            return Err(Self::bad());
        }
        *slot = Some(value.clone());
        Ok(value)
    }
    fn value(&mut self, ctx: &mut Context) -> CloneResult<Value> {
        ctx.clone_check_stack()?;
        let tag = self.byte()?;
        match tag {
            C_UNDEFINED => return Ok(Value::Undefined),
            C_NULL => return Ok(Value::Null),
            C_FALSE => return Ok(Value::Bool(false)),
            C_TRUE => return Ok(Value::Bool(true)),
            C_NUMBER => return Ok(Value::number(self.f64()?)),
            C_STRING => {
                let units = self.string()?;
                return Ok(Value::Str(ctx.intern_utf16(&units)));
            }
            C_BIGINT => {
                let units = self.string()?;
                // The payload is an exact integer spelling, not executable text.
                let digits = units.strip_prefix(&[b'-' as u16]).unwrap_or(&units);
                if digits.is_empty() || digits.iter().any(|unit| !(48..=57).contains(unit)) {
                    return Err(Self::bad());
                }
                let string = ctx.intern_utf16(&units);
                return unsafe {
                    ctx.accept(JS_CloneBigIntFromString(ctx.owner.ctx, string.0.raw))
                }
                .map_err(Into::into);
            }
            C_REFERENCE => {
                let id = self.u32()?;
                return self
                    .objects
                    .get(id as usize)
                    .and_then(Clone::clone)
                    .ok_or_else(Self::bad);
            }
            C_OBJECT..=C_ERROR => {}
            _ => return Err(Self::bad()),
        }
        let id = self.u32()?;
        if id as usize != self.objects.len() {
            return Err(Self::bad());
        }
        self.objects.push(None);
        match tag {
            C_OBJECT | C_ARRAY => {
                let length = if tag == C_ARRAY {
                    Some(self.u32()?)
                } else {
                    None
                };
                let object = if tag == C_ARRAY {
                    Value::Object(ctx.try_new_array()?)
                } else {
                    Value::Object(ctx.try_new_object()?)
                };
                let object = self.register(id, object)?;
                if let Some(length) = length {
                    ctx.clone_define(
                        &object,
                        &"length".encode_utf16().collect::<Vec<_>>(),
                        &Value::number(length as f64),
                        JS_PROP_WRITABLE,
                    )?;
                }
                let count = self.u32()?;
                if count as usize > self.bytes.len().saturating_sub(self.offset) / 5 {
                    return Err(Self::bad());
                }
                for _ in 0..count {
                    let key = self.string()?;
                    let item = self.value(ctx)?;
                    ctx.clone_define(&object, &key, &item, JS_PROP_C_W_E)?;
                }
                Ok(object)
            }
            C_DATE | C_BOXED => {
                let inner = if tag == C_DATE {
                    Value::number(self.f64()?)
                } else {
                    self.value(ctx)?
                };
                if tag == C_BOXED
                    && !matches!(
                        inner,
                        Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Str(_)
                    )
                    && unsafe { JS_IsBigInt(ctx.owner.ctx, inner.raw()) } == 0
                {
                    return Err(Self::bad());
                }
                let object = unsafe {
                    ctx.accept(JS_CloneNewBox(ctx.owner.ctx, inner.raw(), tag == C_DATE))
                }?;
                self.register(id, object)
            }
            C_MAP | C_SET => {
                let object = unsafe { ctx.accept(JS_CloneNewMap(ctx.owner.ctx, tag == C_SET)) }?;
                let object = self.register(id, object)?;
                let count = self.u32()?;
                if (tag == C_MAP && count % 2 != 0)
                    || count as usize > self.bytes.len().saturating_sub(self.offset)
                {
                    return Err(Self::bad());
                }
                for _ in 0..if tag == C_MAP { count / 2 } else { count } {
                    let key = self.value(ctx)?;
                    let item = if tag == C_MAP {
                        self.value(ctx)?
                    } else {
                        Value::Undefined
                    };
                    let result = unsafe {
                        ctx.accept(JS_CloneMapAdd(
                            ctx.owner.ctx,
                            object.raw(),
                            key.raw(),
                            item.raw(),
                            tag == C_SET,
                        ))
                    }?;
                    drop(result);
                }
                Ok(object)
            }
            C_BUFFER => {
                let max = if self.bool()? {
                    Some(self.u32()?)
                } else {
                    None
                };
                let length = self.u32()?;
                if max.is_some_and(|max| max < length) {
                    return Err(Self::bad());
                }
                let bytes = self.take(length as usize)?;
                let object = unsafe { ctx.accept(JS_CloneNewBuffer(ctx.owner.ctx, bytes, max)) }?;
                self.register(id, object)
            }
            C_VIEW => {
                let kind = self.byte()?;
                let offset = self.u32()?;
                let length = self.u32()?;
                let tracking = self.bool()?;
                let buffer = self.value(ctx)?;
                if unsafe { JS_CloneKind(ctx.owner.ctx, buffer.raw()) } != JSCloneKind::Buffer {
                    return Err(Self::bad());
                }
                let object = unsafe {
                    ctx.accept(JS_CloneNewView(
                        ctx.owner.ctx,
                        kind,
                        buffer.raw(),
                        offset,
                        length,
                        tracking,
                    ))
                }?;
                self.register(id, object)
            }
            C_REGEXP => {
                let units = self.string()?;
                let flags = self.u32()?;
                if flags & !0x17f != 0 {
                    return Err(Self::bad());
                }
                let pattern = ctx.intern_utf16(&units);
                let table = [
                    (0, 'g'),
                    (1, 'i'),
                    (2, 'm'),
                    (3, 's'),
                    (4, 'u'),
                    (5, 'y'),
                    (6, 'd'),
                    (8, 'v'),
                ];
                let flags = table
                    .into_iter()
                    .filter_map(|(bit, ch)| (flags & (1 << bit) != 0).then_some(ch))
                    .collect::<String>();
                let flags = ctx.intern(&flags);
                let object = unsafe {
                    ctx.accept(JS_CloneNewRegExp(ctx.owner.ctx, pattern.0.raw, flags.0.raw))
                }?;
                self.register(id, object)
            }
            C_ERROR => {
                let kind = self.byte()?;
                let object = unsafe { ctx.accept(JS_CloneNewError(ctx.owner.ctx, kind)) }?;
                let object = self.register(id, object)?;
                for name in ["message", "stack"] {
                    if self.bool()? {
                        let units = self.string()?;
                        let value = Value::Str(ctx.intern_utf16(&units));
                        ctx.clone_define(
                            &object,
                            &name.encode_utf16().collect::<Vec<_>>(),
                            &value,
                            JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE,
                        )?;
                    }
                }
                if self.bool()? {
                    let cause = self.value(ctx)?;
                    ctx.clone_define(
                        &object,
                        &"cause".encode_utf16().collect::<Vec<_>>(),
                        &cause,
                        JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE,
                    )?;
                }
                Ok(object)
            }
            _ => Err(Self::bad()),
        }
    }
}

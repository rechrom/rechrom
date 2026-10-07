// Test-only missing typed-buffer construction boundaries. They cannot be
// used to claim support: any entered path fails the JSON/scalar/bytecode oracle.
unsafe fn js_array_buffer_constructor3(_: *mut JSContext, _: JSValueConst, _: u64, _: *mut u64, _: JSClassID, _: *mut u8, _: Option<JSFreeArrayBufferDataFunc>, _: *mut c_void, _: i32) -> JSValue { panic!("buffer constructors outside current serialization fixture") }
unsafe fn js_array_buffer_free(_: *mut JSRuntime, _: *mut c_void, _: *mut c_void) { panic!("buffer free outside current serialization fixture") }
unsafe fn js_get_array_buffer(_: *mut JSContext, _: JSValueConst) -> *mut JSArrayBuffer { panic!("buffer retrieval outside current serialization fixture") }
unsafe fn JS_ThrowTypeErrorDetachedArrayBuffer(_: *mut JSContext) -> JSValue { panic!("buffer throw outside current serialization fixture") }
unsafe fn js_typed_array_constructor(_: *mut JSContext, _: JSValueConst, _: i32, _: *mut JSValueConst, _: i32) -> JSValue { panic!("typed-array constructor outside current serialization fixture") }

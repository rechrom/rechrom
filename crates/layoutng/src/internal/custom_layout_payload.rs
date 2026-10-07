use std::sync::Arc;

// cpp: layoutng/internal/custom_layout_payload.h:12-31
pub struct CustomLayoutPayload {
    bytes_: Box<[u8]>,
}

#[allow(non_snake_case)]
impl CustomLayoutPayload {
    // cpp: layoutng/internal/custom_layout_payload.h:18-20
    pub fn Copy(bytes: &[u8]) -> Arc<Self> {
        Arc::new(Self {
            bytes_: bytes.to_vec().into_boxed_slice(),
        })
    }

    // cpp: layoutng/internal/custom_layout_payload.h:22-22
    pub fn Bytes(&self) -> &[u8] {
        &self.bytes_
    }
}

// cpp: layoutng/internal/custom_layout_payload.h:33-36
pub type SerializedScriptValue = CustomLayoutPayload;

//! Payload storage from core/svg/svg_path_byte_stream.h:31-74.
//! Its serialized commands use u16 segment types and native-endian f32
//! coordinates (svg_path_byte_stream_builder.cc:43-112).
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct SVGPathByteStream {
    data: Vec<u8>,
}

#[allow(non_snake_case)]
impl SVGPathByteStream {
    pub fn from_data(data: Vec<u8>) -> Self {
        Self { data }
    }
    pub fn Span(&self) -> &[u8] {
        &self.data
    }
    pub fn IsEmpty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

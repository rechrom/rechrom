use foundation::GCedHeapVector;

use super::cursor_data::CursorData;

// cpp: layoutng_style/style/cursor_list.h:13
pub type CursorList = GCedHeapVector<CursorData>;

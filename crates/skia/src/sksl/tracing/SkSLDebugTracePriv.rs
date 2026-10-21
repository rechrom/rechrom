// Source: src/sksl/tracing/SkSLDebugTracePriv.h:22-22
pub use crate::src::core::SkStream::SkWStream;
// Copyright 2021 Google LLC. BSD-style license.
use super::SkSLTraceHook::TraceHook;
use crate::src::core::SkPoint::SkIPoint;
use crate::src::sksl::{ir::SkSLType::NumberKind, SkSLPosition::Position};
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:27-33
#[derive(Clone, Copy, Debug)]
pub enum Op {
    kLine,
    kVar,
    kEnter,
    kExit,
    kScope,
}
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:26-36
pub struct TraceInfo {
    pub op: Op,
    pub data: [i32; 2],
}
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:38-55
pub struct SlotDebugInfo {
    pub name: Vec<u8>,
    pub columns: u8,
    pub rows: u8,
    pub componentIndex: u8,
    pub groupIndex: i32,
    pub numberKind: NumberKind,
    pub line: i32,
    pub pos: Position,
    pub fnReturnValue: i32,
}
impl Default for SlotDebugInfo {
    fn default() -> Self {
        Self {
            name: Vec::new(),
            columns: 1,
            rows: 1,
            componentIndex: 0,
            groupIndex: 0,
            numberKind: NumberKind::kNonnumeric,
            line: 0,
            pos: Position::default(),
            fnReturnValue: -1,
        }
    }
}
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:56-59
#[derive(Default)]
pub struct FunctionDebugInfo {
    pub name: Vec<u8>,
}
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:87-106
pub struct DebugTracePriv {
    // Source: src/sksl/tracing/SkSLDebugTracePriv.h:61-61
    // Actual inherited SkRefCnt subobject; owns the sole intrusive counter.
    base: crate::include::core::SkRefCnt::SkRefCnt,
    pub fTraceCoord: SkIPoint,
    pub fUniformInfo: Vec<SlotDebugInfo>,
    pub fSlotInfo: Vec<SlotDebugInfo>,
    pub fFuncInfo: Vec<FunctionDebugInfo>,
    pub fTraceInfo: Vec<TraceInfo>,
    pub fSource: Vec<Vec<u8>>,
    pub fTraceHook: Option<Box<dyn TraceHook>>,
}
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:61-61
impl Default for DebugTracePriv {
    fn default() -> Self {
        Self {
            base: crate::include::core::SkRefCnt::SkRefCnt::new(),
            fTraceCoord: SkIPoint::default(),
            fUniformInfo: Vec::new(),
            fSlotInfo: Vec::new(),
            fFuncInfo: Vec::new(),
            fTraceInfo: Vec::new(),
            fSource: Vec::new(),
            fTraceHook: None,
        }
    }
}
// The existing RefCnt disposal restores the debug count and destroys the actual
// concrete allocation. The explicit member Drop below runs before base drop.
unsafe impl crate::include::core::SkRefCnt::SkRefCntObject for DebugTracePriv {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }

    fn ref_cnt_base(&self) -> &crate::include::core::SkRefCnt::SkRefCntBase {
        self.base.ref_cnt_base()
    }
    fn ref_(&self) {
        self.base.ref_();
    }
}
// Source: src/sksl/tracing/SkSLDebugTracePriv.h:72-72
impl crate::include::sksl::SkSLDebugTrace::DebugTrace for DebugTracePriv {
    fn dump(&self, o: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>) {
        DebugTracePriv::dump(self, o);
    }
}
impl Drop for DebugTracePriv {
    fn drop(&mut self) {
        // Preserve native reverse member destruction. In particular the borrowed
        // trace hook dies before its target Vec object and storage are destroyed.
        // Taking a Vec leaves an empty, allocation-free shell for Rust field drop.
        drop(self.fTraceHook.take());
        drop(std::mem::take(&mut self.fSource));
        drop(std::mem::take(&mut self.fTraceInfo));
        drop(std::mem::take(&mut self.fFuncInfo));
        drop(std::mem::take(&mut self.fSlotInfo));
        drop(std::mem::take(&mut self.fUniformInfo));
    }
}
impl DebugTracePriv {
    // Source: src/sksl/tracing/SkSLDebugTracePriv.h:61-61
    pub fn new() -> Self {
        Self::default()
    }
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:20-38
    pub fn getSlotComponentSuffix(&self, index: i32) -> String {
        let slot = &self.fSlotInfo[index as usize];
        if slot.rows > 1 {
            return format!(
                "[{}][{}]",
                slot.componentIndex / slot.rows,
                slot.componentIndex % slot.rows
            );
        }
        if slot.columns > 1 {
            return match slot.componentIndex {
                0 => ".x",
                1 => ".y",
                2 => ".z",
                3 => ".w",
                _ => "[???]",
            }
            .to_owned();
        }
        String::new()
    }
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:40-60
    pub fn interpretValueBits(&self, index: i32, bits: i32) -> f64 {
        debug_assert!(index >= 0 && (index as usize) < self.fSlotInfo.len());
        match self.fSlotInfo[index as usize].numberKind {
            NumberKind::kUnsigned => bits as u32 as f64,
            NumberKind::kFloat => f32::from_bits(bits as u32) as f64,
            _ => bits as f64,
        }
    }
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:62-75
    pub fn slotValueToString(&self, index: i32, value: f64) -> String {
        debug_assert!(index >= 0 && (index as usize) < self.fSlotInfo.len());
        if self.fSlotInfo[index as usize].numberKind == NumberKind::kBoolean {
            return if value != 0.0 { "true" } else { "false" }.to_owned();
        }
        // Use the exact native printf format, including exponent and special-value spelling.
        unsafe extern "C" {
            fn snprintf(
                out: *mut std::ffi::c_char,
                n: usize,
                format: *const std::ffi::c_char,
                ...
            ) -> i32;
        }
        let mut buffer = [0u8; 32];
        unsafe {
            snprintf(
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                c"%.8g".as_ptr(),
                value,
            );
        }
        let n = buffer.iter().position(|x| *x == 0).unwrap_or(buffer.len());
        String::from_utf8(buffer[..n].to_vec()).expect("printf numeric output is ASCII")
    }
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:77-79
    pub fn getSlotValue(&self, index: i32, bits: i32) -> String {
        self.slotValueToString(index, self.interpretValueBits(index, bits))
    }
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:81-83
    pub fn setTraceCoord(&mut self, coord: &SkIPoint) {
        self.fTraceCoord = *coord;
    }
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:85-92
    pub fn setSource(&mut self, source: &[u8]) {
        self.fSource.clear();
        self.fSource
            .extend(source.split(|c| *c == b'\n').map(<[u8]>::to_vec));
    }
}

impl DebugTracePriv {
    // Source: src/sksl/tracing/SkSLDebugTracePriv.cpp:94-191
    // Rust byte slices replace temporary c_str views. Preserve writeText's
    // first-NUL boundary and ignore write failures just as the native dump does.
    // Decimal writes call the actual native SkWStream::writeDecAsText implementation.
    pub fn dump(&self, mut output: core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>) {
        fn text(
            o: &mut core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>,
            bytes: &[u8],
        ) {
            let len = bytes.iter().position(|c| *c == 0).unwrap_or(bytes.len());
            let mut terminated = Vec::with_capacity(len + 1);
            terminated.extend_from_slice(&bytes[..len]);
            terminated.push(0);
            unsafe {
                o.as_mut().writeText(terminated.as_ptr().cast());
            }
        }
        fn dec(o: &mut core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>, value: i32) {
            o.as_mut().writeDecAsText(value);
        }
        let o = &mut output;
        for (index, info) in self.fSlotInfo.iter().enumerate() {
            text(o, b"$");
            dec(o, index as i32);
            text(o, b" = ");
            text(o, &info.name);
            text(o, b" (");
            match info.numberKind {
                NumberKind::kFloat => text(o, b"float"),
                NumberKind::kSigned => text(o, b"int"),
                NumberKind::kUnsigned => text(o, b"uint"),
                NumberKind::kBoolean => text(o, b"bool"),
                NumberKind::kNonnumeric => text(o, b"???"),
                _ => text(o, b"???"),
            }
            if i32::from(info.rows) * i32::from(info.columns) > 1 {
                dec(o, info.columns as i32);
                if info.rows != 1 {
                    text(o, b"x");
                    dec(o, info.rows as i32);
                }
                text(o, b" : ");
                text(o, b"slot ");
                dec(o, i32::from(info.componentIndex) + 1);
                text(o, b"/");
                dec(o, i32::from(info.rows) * i32::from(info.columns));
            }
            text(o, b", L");
            dec(o, info.line);
            text(o, b")");
            o.as_mut().newline();
        }
        for (index, info) in self.fFuncInfo.iter().enumerate() {
            text(o, b"F");
            dec(o, index as i32);
            text(o, b" = ");
            text(o, &info.name);
            o.as_mut().newline();
        }
        o.as_mut().newline();
        if !self.fTraceInfo.is_empty() {
            let mut indent = Vec::new();
            for traceInfo in &self.fTraceInfo {
                let data0 = traceInfo.data[0];
                let data1 = traceInfo.data[1];
                match traceInfo.op {
                    Op::kLine => {
                        text(o, &indent);
                        text(o, b"line ");
                        dec(o, data0);
                    }
                    Op::kVar => {
                        let slot = &self.fSlotInfo[data0 as usize];
                        text(o, &indent);
                        text(o, &slot.name);
                        text(o, self.getSlotComponentSuffix(data0).as_bytes());
                        text(o, b" = ");
                        text(o, self.getSlotValue(data0, data1).as_bytes());
                    }
                    Op::kEnter => {
                        text(o, &indent);
                        text(o, b"enter ");
                        text(o, &self.fFuncInfo[data0 as usize].name);
                        indent.extend_from_slice(b"  ");
                    }
                    Op::kExit => {
                        indent.resize(indent.len() - 2, b' ');
                        text(o, &indent);
                        text(o, b"exit ");
                        text(o, &self.fFuncInfo[data0 as usize].name);
                    }
                    Op::kScope => {
                        for _ in data0..0 {
                            indent.pop();
                        }
                        text(o, &indent);
                        text(o, b"scope ");
                        text(o, if data0 >= 0 { b"+" } else { b"" });
                        dec(o, data0);
                        for _ in 0..data0 {
                            indent.push(b' ');
                        }
                    }
                }
                o.as_mut().newline();
            }
        }
    }
}

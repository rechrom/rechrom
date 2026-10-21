use crate::src::core::{
    SkPoint::SkPoint,
    SkString::SkString,
    SkStringUtils::{
        SkAppendScalar, SkAppendScalarDec, SkScalarAsStringType, SkScalarAsStringType::*,
    },
};
// Source: src/core/SkPathDump.cpp:17-22
const gFillTypeStrs: [&[u8]; 4] = [
    b"Winding\0",
    b"EvenOdd\0",
    b"InverseWinding\0",
    b"InverseEvenOdd\0",
];
// Source: src/core/SkPathDump.cpp:24-24
const kSentinelConicWeight: f32 = -12345.0;
// Source: src/core/SkPathDump.cpp:26-60
fn append_params(
    string: &mut SkString,
    label: &SkString,
    pts: &[SkPoint],
    str_type: SkScalarAsStringType,
    use_semicolon: bool,
    conic_weight: f32,
) {
    string.append_string(label);
    string.append_view(b"(");
    let count = pts.len() * 2;
    // Native scalar order x0,y0,x1,y1; project real fields without ABI casts.
    let value = |i: usize| {
        if i % 2 == 0 {
            pts[i / 2].fX
        } else {
            pts[i / 2].fY
        }
    };
    for i in 0..count {
        SkAppendScalar(string, value(i), str_type);
        if i < count - 1 {
            string.append_view(b", ");
        }
    }
    if conic_weight != kSentinelConicWeight {
        string.append_view(b", ");
        SkAppendScalar(string, conic_weight, str_type);
    }
    string.append_view(if use_semicolon { b");" } else { b")" });
    if str_type == kHex_SkScalarAsStringType {
        string.append_view(b"  // ");
        for i in 0..count {
            SkAppendScalarDec(string, value(i));
            if i < count - 1 {
                string.append_view(b", ");
            }
        }
        if conic_weight >= 0.0 {
            string.append_view(b", ");
            SkAppendScalarDec(string, conic_weight);
        }
    }
    string.append_view(b"\n");
}
// Source: src/core/SkPathDump.cpp:62-107
fn dump_iter(
    mut iter: crate::src::core::SkPathIter::SkPathIter<'_>,
    builder: &mut SkString,
    cmd_prefix: &[u8],
    str_type: SkScalarAsStringType,
    use_semicolon: bool,
    mut post_verb: impl FnMut(&mut SkString),
) {
    use crate::include::core::SkPathTypes::SkPathVerb;
    while let Some(rec) = iter.next() {
        let mut cmd = SkString::from_view(cmd_prefix);
        let mut pts: &[SkPoint] = &[];
        let mut cw = kSentinelConicWeight;
        match rec.fVerb {
            SkPathVerb::kMove => {
                cmd.append_view(b".moveTo");
                pts = &rec.fPoints[0..1];
            }
            SkPathVerb::kLine => {
                cmd.append_view(b".lineTo");
                pts = &rec.fPoints[1..2];
            }
            SkPathVerb::kQuad => {
                cmd.append_view(b".quadTo");
                pts = &rec.fPoints[1..3];
            }
            SkPathVerb::kConic => {
                cmd.append_view(b".conicTo");
                pts = &rec.fPoints[1..3];
                cw = rec.conicWeight();
            }
            SkPathVerb::kCubic => {
                cmd.append_view(b".cubicTo");
                pts = &rec.fPoints[1..4];
            }
            SkPathVerb::kClose => {
                cmd.append_view(b".close()");
                if use_semicolon {
                    cmd.append_view(b";");
                }
                cmd.append_view(b"\n");
                builder.append_string(&cmd);
            }
        }
        if !pts.is_empty() {
            append_params(builder, &cmd, pts, str_type, use_semicolon, cw);
        }
        post_verb(builder);
    }
}
// Source: include/core/SkPath.h:1028-1028
// Source: src/core/SkPathDump.cpp:109-125
impl crate::src::core::SkPath::SkPath {
    pub fn dump(
        &self,
        mut stream: Option<core::pin::Pin<&mut dyn crate::src::core::SkStream::SkWStream>>,
        dump_as_hex: bool,
    ) {
        let as_type = if dump_as_hex {
            kHex_SkScalarAsStringType
        } else {
            kDec_SkScalarAsStringType
        };
        let mut builder = SkString::new();
        unsafe {
            builder.printf(
                b"path.setFillType(SkPathFillType::k%s);\n\0".as_ptr(),
                gFillTypeStrs[self.getFillType() as usize].as_ptr(),
            );
        }
        let no_stream = stream.is_none();
        dump_iter(
            self.iter(),
            &mut builder,
            b"path",
            as_type,
            true,
            |builder| {
                if no_stream && builder.size() != 0 {
                    unsafe {
                        crate::SkDebugf!(@source b"src/core/SkPathDump.cpp\0".as_ptr(),118; b"%s\0".as_ptr(),builder.c_str());
                    }
                    builder.reset();
                }
            },
        );
        if let Some(stream) = stream.as_mut() {
            unsafe {
                stream.as_mut().writeText(builder.c_str().cast());
            }
        }
    }
    // Source: include/core/SkPath.h:1030-1030
    pub fn dump_default(&self) {
        self.dump(None, false);
    }
    // Source: include/core/SkPath.h:1031-1031
    pub fn dumpHex(&self) {
        self.dump(None, true);
    }
}

// Source: src/core/SkPathDump.cpp:127-138
impl crate::src::core::SkPathBuilder::SkPathBuilder {
    pub fn dumpToString(&self, format: crate::src::core::SkPathBuilder::DumpFormat) -> SkString {
        let as_type = if format == crate::src::core::SkPathBuilder::DumpFormat::kHex {
            kHex_SkScalarAsStringType
        } else {
            kDec_SkScalarAsStringType
        };
        let mut builder = SkString::new();
        unsafe {
            builder.printf(
                b"SkPathBuilder(SkPathFillType::k%s)\n\0".as_ptr(),
                gFillTypeStrs[self.fillType() as usize].as_ptr(),
            );
        }
        dump_iter(self.iter(), &mut builder, b"", as_type, false, |_| {});
        builder
    }
    // Source: src/core/SkPathDump.cpp:140-142
    pub fn dump(&self, _format: crate::src::core::SkPathBuilder::DumpFormat) {
        // Exact native body ignores format and invokes the default decimal overload.
        let builder = self.dumpToString_default();
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkPathDump.cpp\0".as_ptr(),141; b"%s\0".as_ptr(),builder.c_str());
        }
    }
}

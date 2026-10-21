// Source: src/core/SkTextFormatParams.h:19-22
pub const kStdFakeBoldInterpKeys: [f32; 2] = [1.0 * 9.0, 1.0 * 36.0];
// Source: src/core/SkTextFormatParams.h:23-26
pub const kStdFakeBoldInterpValues: [f32; 2] = [1.0 / 24.0, 1.0 / 32.0];
// Source: src/core/SkTextFormatParams.h:27-28
const _: () = assert!(kStdFakeBoldInterpKeys.len() == kStdFakeBoldInterpValues.len());
// Source: src/core/SkTextFormatParams.h:29-29
pub const kStdFakeBoldInterpLength: i32 = kStdFakeBoldInterpKeys.len() as i32;

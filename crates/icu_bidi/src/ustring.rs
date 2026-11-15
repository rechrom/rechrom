//! ICU4C C-API conventions shared by the modules of this crate: UErrorCode
//! values, NUL-terminated (`length == -1`) UTF-16 inputs, and the
//! preflighting output protocol (`capacity` too small → U_BUFFER_OVERFLOW_ERROR
//! with the required length returned).

pub const U_ZERO_ERROR: i32 = 0;
pub const U_ILLEGAL_ARGUMENT_ERROR: i32 = 1;
pub const U_BUFFER_OVERFLOW_ERROR: i32 = 15;
pub const U_STRING_NOT_TERMINATED_WARNING: i32 = -124;

/// Whether `status` already holds a failure, as `U_FAILURE`.
pub unsafe fn failed(status: *mut i32) -> bool {
    !status.is_null() && *status > U_ZERO_ERROR
}

pub unsafe fn set_error(status: *mut i32, error: i32) {
    if !status.is_null() {
        *status = error;
    }
}

/// Borrows an ICU UTF-16 input; `length == -1` means NUL-terminated.
pub unsafe fn input<'a>(text: *const u16, length: i32) -> Option<&'a [u16]> {
    if text.is_null() {
        return (length <= 0).then_some(&[]);
    }
    let length = if length < 0 {
        let mut end = 0;
        while *text.add(end) != 0 {
            end += 1;
        }
        end
    } else {
        length as usize
    };
    Some(std::slice::from_raw_parts(text, length))
}

/// Writes `value` with ICU's preflighting rules and returns its length.
pub unsafe fn output(value: &[u16], destination: *mut u16, capacity: i32, status: *mut i32) -> i32 {
    let Ok(length) = i32::try_from(value.len()) else {
        set_error(status, U_ILLEGAL_ARGUMENT_ERROR);
        return 0;
    };
    if capacity < 0 || (capacity > 0 && destination.is_null()) {
        set_error(status, U_ILLEGAL_ARGUMENT_ERROR);
        return 0;
    }
    if length > capacity {
        set_error(status, U_BUFFER_OVERFLOW_ERROR);
        return length;
    }
    std::ptr::copy_nonoverlapping(value.as_ptr(), destination, value.len());
    if length < capacity {
        *destination.add(value.len()) = 0;
    } else if !failed(status) {
        set_error(status, U_STRING_NOT_TERMINATED_WARNING);
    }
    length
}

/// Applies a string mapping to every maximal well-formed run of `text`,
/// passing unpaired surrogates through unchanged as ICU does.
pub fn map_utf16(text: &[u16], mut map: impl FnMut(&str, &mut String)) -> Vec<u16> {
    let mut result = Vec::with_capacity(text.len());
    let mut run = String::new();
    let mut mapped = String::new();
    let mut flush = |run: &mut String, result: &mut Vec<u16>| {
        if !run.is_empty() {
            mapped.clear();
            map(run, &mut mapped);
            result.extend(mapped.encode_utf16());
            run.clear();
        }
    };
    for unit in char::decode_utf16(text.iter().copied()) {
        match unit {
            Ok(character) => run.push(character),
            Err(error) => {
                flush(&mut run, &mut result);
                result.push(error.unpaired_surrogate());
            }
        }
    }
    flush(&mut run, &mut result);
    result
}

/// Code points of `text` with their UTF-16 start offsets; an unpaired
/// surrogate is reported as itself, as `U16_NEXT` does.
pub fn code_points(text: &[u16]) -> impl Iterator<Item = (usize, u32)> + '_ {
    let mut index = 0;
    std::iter::from_fn(move || {
        let start = index;
        let first = *text.get(index)?;
        index += 1;
        if (0xD800..0xDC00).contains(&first) {
            if let Some(&second) = text.get(index) {
                if (0xDC00..0xE000).contains(&second) {
                    index += 1;
                    return Some((
                        start,
                        0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00),
                    ));
                }
            }
        }
        Some((start, u32::from(first)))
    })
}

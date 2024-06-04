mod iter;
mod str;
mod string;

pub mod macros;

pub use str::JavaStr;
pub use string::JavaString;

use std::num::NonZeroU8;

/// Errors which can occur when attempting to interpret a sequence of [`u8`] as
/// a string.
///
/// As such, the `from_slice` function for [`JavaStr`] makes use of this error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodingError {
    error_len: Option<NonZeroU8>,
    valid_up_to: usize,
}

impl EncodingError {
    /// Returns the index in the given string up to which valid Modified UTF-8
    /// was verified.
    ///
    /// It is the maximum index such that `from_slice` of [`JavaStr`] would
    /// return `Ok(_)`.
    #[inline]
    #[must_use]
    pub fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// Provides more information about the failure:
    /// * `None`: the end of the input was reached unexpectedly.
    ///   `self.valid_up_to()` is 1 to 6 bytes from the end of the input. If a
    ///   byte stream (such as a file or network socket) is being decoded
    ///   incrementally, this could be a valid `char` whose UTF-8 byte sequence
    ///   is spanning multiple chunks.
    /// * `Some(len)`: an unexpected byte was encountered. The length provided
    ///   is that of the invalid byte seqence that starts at the index given by
    ///   `valid_up_to()`.
    #[inline]
    #[must_use]
    pub fn error_len(&self) -> Option<NonZeroU8> {
        self.error_len
    }
}

impl core::fmt::Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(len) = self.error_len {
            write!(
                f,
                "invalid Modified UTF-8 sequence of {} bytes from index {}",
                len, self.valid_up_to
            )
        } else {
            write!(
                f,
                "invalid Modified UTF-8 byte sequence from index {}",
                self.valid_up_to
            )
        }
    }
}

impl std::error::Error for EncodingError {}

/// Converts bytes in UTF-8 format into Modified UTF-8 format.
///
/// This will not copy unless the `str` has the nul character or a supplementary
/// character inside it.
#[must_use]
pub fn from_utf8(str: &str) -> std::borrow::Cow<'_, JavaStr> {
    let mut index = 0;
    let mut last_index = 0;

    let mut string = None;

    let v = str.as_bytes();
    while let Some(&byte) = v.get(index) {
        if byte & 0b1111_1000 == 0b1111_0000 {
            let string = string.get_or_insert(JavaString::with_capacity(str.len()));

            unsafe {
                let c = core::str::from_utf8_unchecked(&v[index..])
                    .chars()
                    .next()
                    .unwrap_unchecked();

                let vec = string.as_mut_vec();
                vec.extend_from_slice(&v[last_index..index]);

                // Add character in Modified UTF-8.
                vec.extend_from_slice(encode_raw(c as u32, &mut [0; 6]));
            }

            index += 4;
            last_index = index;
        } else if byte == 0 {
            let string = string.get_or_insert(JavaString::with_capacity(str.len()));

            unsafe {
                let vec = string.as_mut_vec();
                vec.extend_from_slice(&v[last_index..index]);

                // Add nul character in the Modified UTF-8.
                vec.extend_from_slice(&[0xC0, 0x80]);
            }

            index += 1;
            last_index = index;
        } else {
            index += 1;
        }
    }

    if let Some(mut string) = string {
        unsafe { string.as_mut_vec().extend_from_slice(&v[last_index..index]) };
        std::borrow::Cow::Owned(string)
    } else {
        std::borrow::Cow::Borrowed(unsafe { JavaStr::from_java_unchecked(v) })
    }
}

/// Checks whether a slice of bytes contains valid Modified UTF-8 data. This
/// does include unpaired surrogates, thus meaning that each code point is not
/// necessarily a Unicode scalar value (a `char`).
#[allow(clippy::identity_op)]
pub const fn validate(v: &[u8]) -> Result<(), EncodingError> {
    const OVERLONG: [u32; 4] = [0x00, 0x80, 0x800, 0x10000];

    let mut index = 0;
    while index < v.len() {
        macro_rules! err {
            ($error_len:expr) => {
                return Err(EncodingError {
                    error_len: NonZeroU8::new($error_len),
                    valid_up_to: index,
                })
            };
        }

        let first = v[index];

        if first == 0 {
            err!(1)
        } else if first < 128 {
            // 1-byte code points
            index += 1;
        } else if first & 0b1110_0000 == 0b1100_0000 {
            // 2-byte code points
            if index + 1 >= v.len() {
                err!(1);
            }

            let second = v[index + 1];
            if second & 0b1100_0000 != 0b1000_0000 {
                err!(2);
            }

            let code_point = ((first as u32 & 0x1F) << 6) | (second as u32 & 0x3F);
            if code_point < OVERLONG[1] && code_point != 0 {
                err!(2);
            }

            index += 2;
        } else if first & 0b1111_0000 == 0b1110_0000 {
            if let Some(code_point) = get_surrogate_index(v, index) {
                if code_point < OVERLONG[3] || code_point > 0x10FFFF {
                    err!(6);
                }

                index += 6;
                continue;
            }

            // 3-byte code points
            if index + 2 >= v.len() {
                err!(1);
            }

            let second = v[index + 1];
            let third = v[index + 2];
            if second & 0b1100_0000 != 0b1000_0000 {
                err!(2);
            }
            if third & 0b1100_0000 != 0b1000_0000 {
                err!(3);
            }

            let code_point = ((first as u32 & 0x0F) << 12)
                | ((second as u32 & 0x3F) << 6)
                | (third as u32 & 0x3F);
            if code_point < OVERLONG[2] {
                err!(3);
            }

            index += 3;
        } else {
            err!(1);
        }
    }
    Ok(())
}

/// Reads the first code point out of a byte slice (assuming a Modified UTF-8
/// encoding).
///
/// This returns `None` if the slice is empty. Otherwise, it will return the
/// slice with the first code point removed and the code point.
///
/// # Safety
///
/// The byte slice passed in must be valid Modified UTF-8.
#[inline]
unsafe fn next_code_point(bytes: &[u8]) -> Option<(&[u8], u32)> {
    let first = *bytes.first()?;
    if first < 128 {
        // 1-byte characters
        Some((&bytes[1..], first as u32))
    } else if first & 0b1110_0000 == 0b1100_0000 {
        // 2-byte characters
        let second = *bytes.get_unchecked(1);
        Some((
            &bytes[2..],
            ((first as u32 & 0x1F) << 6) | (second as u32 & 0x3F),
        ))
    } else if let Some(code_point) = get_surrogate_index(bytes, 0) {
        // 6-byte characters
        Some((&bytes[6..], code_point))
    } else {
        // 3-byte characters
        let second = *bytes.get_unchecked(1);
        let third = *bytes.get_unchecked(1);
        Some((
            &bytes[3..],
            ((first as u32 & 0x0F) << 12) | ((second as u32 & 0x3F) << 6) | (third as u32 & 0x3F),
        ))
    }
}

/// Reads the last code point of a byte slice (assuming a Modified UTF-8
/// encoding).
///
/// This returns `None` if the slice is empty. Otherwise, it will return the
/// slice with the last code point removed and the code point.
///
/// # Safety
///
/// The byte slice passed in must be valid Modified UTF-8.
#[inline]
unsafe fn next_code_point_reverse(bytes: &[u8]) -> Option<(&[u8], u32)> {
    if bytes.is_empty() {
        return None;
    }

    let first = *bytes.get_unchecked(bytes.len() - 1);
    if first < 128 {
        // 1-byte characters
        Some((&bytes[..bytes.len() - 1], first as u32))
    } else {
        let second = *bytes.get_unchecked(bytes.len() - 2);
        if second & 0b1110_0000 == 0b1100_0000 {
            // 2-byte characters
            Some((
                &bytes[..bytes.len() - 2],
                ((second as u32 & 0x1F) << 6) | (first as u32 & 0x3F),
            ))
        } else {
            if bytes.len() > 6 {
                if let Some(code_point) = get_surrogate_index(bytes, bytes.len() - 6) {
                    return Some((&bytes[..bytes.len() - 6], code_point));
                }
            }

            let third = *bytes.get_unchecked(bytes.len() - 3);
            Some((
                &bytes[..bytes.len() - 3],
                ((third as u32 & 0x0F) << 12)
                    | ((second as u32 & 0x3F) << 6)
                    | (first as u32 & 0x3F),
            ))
        }
    }
}

#[inline]
#[allow(clippy::identity_op)]
const fn get_surrogate_index(v: &[u8], index: usize) -> Option<u32> {
    if let Some(x) = index.checked_add(5) {
        if index + 5 != x {
            unsafe { std::hint::unreachable_unchecked() }
        }

        if x < v.len()
            && ((v[index + 0] & 0xFF) == 0xED)
                & ((v[index + 1] & 0xF0) == 0xA0)
                & ((v[index + 2] & 0xC0) == 0x80)
                & ((v[index + 3] & 0xFF) == 0xED)
                & ((v[index + 4] & 0xF0) == 0xB0)
                & ((v[index + 5] & 0xC0) == 0x80)
        {
            return Some(
                0x10000
                    + (((v[index + 1] as u32 & 0x0F) << 16)
                        | ((v[index + 2] as u32 & 0x3F) << 10)
                        | ((v[index + 4] as u32 & 0x0F) << 6)
                        | (v[index + 5] as u32 & 0x3F)),
            );
        }
    }
    None
}

/// Compute the length of a character when encoded in the CESU-8 format.
#[must_use]
const fn len(code: u32) -> usize {
    if code < 80 && code != 0 {
        1
    } else if code < 0x800 {
        2
    } else if code < 0x10000 {
        3
    } else {
        6
    }
}

/// Encodes a raw u32 value as CESU-8 into the provided byte buffer, then
/// returns the subslice of the buffer that contains the encoded character.
#[inline]
#[must_use]
fn encode_raw(code: u32, dst: &mut [u8]) -> &mut [u8] {
    let len = len(code);
    match (len, &mut dst[..]) {
        (1, [a, ..]) => *a = code as u8,
        (2, [a, b, ..]) => {
            *a = 0b1100_0000 | (code >> 6 & 0x1F) as u8;
            *b = 0b1000_0000 | (code & 0x3F) as u8;
        }
        (3, [a, b, c, ..]) => {
            *a = 0b1110_0000 | (code >> 12 & 0x0F) as u8;
            *b = 0b1000_0000 | (code >> 6 & 0x3F) as u8;
            *c = 0b1000_0000 | (code & 0x3F) as u8;
        }
        (6, [a, b, c, d, e, f, ..]) => {
            *a = 0b1110_1101;
            *b = 0b1010_0000 | ((code - 0x1_0000) >> 16 & 0x0F) as u8;
            *c = 0b1000_0000 | (code >> 10 & 0x3F) as u8;
            *d = 0b1110_1101;
            *e = 0b1011_0000 | (code >> 6 & 0x0F) as u8;
            *f = 0b1000_0000 | (code & 0x3F) as u8;
        }
        _ => panic!(
            "encode_cesu8: need {len} bytes to encode U+{code:X}, but the buffer has {}",
            dst.len()
        ),
    };
    &mut dst[..len]
}

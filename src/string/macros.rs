/// Builds a `JavaStr` literal at compile time from a string literal.
#[macro_export]
macro_rules! java_str {
    ($str:tt) => {{
        const _JAVA_STR_MACRO_STR: &str = $str;
        const _JAVA_STR_MACRO_LEN: usize =
            $crate::string::macros::required_len(_JAVA_STR_MACRO_STR);
        const _JAVA_STR_MACRO_BUF: [u8; _JAVA_STR_MACRO_LEN] =
            $crate::string::macros::create_array(_JAVA_STR_MACRO_STR);
        unsafe { $crate::string::JavaStr::from_java_unchecked(&_JAVA_STR_MACRO_BUF) }
    }};
}

/// Calculate the amount of bytes required to encode `str` in Modified UTF-8.
pub const fn required_len(str: &str) -> usize {
    let mut len = 0;

    let mut i = 0;
    let v = str.as_bytes();
    while i < v.len() {
        let first = v[i];
        if first & 0b1111_1000 == 0b1111_0000 {
            len += 6;
            i += 4;
        } else if first == 0 {
            len += 2;
            i += 1;
        } else {
            len += 1;
            i += 1;
        }
    }

    len
}

/// Creates a buffer of CESU-8 encoded bytes from `str`.
#[allow(clippy::identity_op)]
pub const fn create_array<const N: usize>(str: &str) -> [u8; N] {
    let mut buf = [0; N];

    let mut j = 0;
    let mut i = 0;
    let v = str.as_bytes();
    while i < v.len() {
        let first = v[i];
        if first & 0b1111_1000 == 0b1111_0000 {
            let code = 0x10000
                + (((v[i + 0] as u32 & 0b0000_0111) << 18)
                    | ((v[i + 1] as u32 & 0b0011_1111) << 12)
                    | ((v[i + 2] as u32 & 0b0011_1111) << 6)
                    | (v[i + 3] as u32 & 0b0011_1111));

            buf[i + 0] = 0b1110_1101;
            buf[i + 1] = 0b1010_0000 | ((code - 0x1_0000) >> 16 & 0x0F) as u8;
            buf[i + 2] = 0b1000_0000 | (code >> 10 & 0x3F) as u8;
            buf[i + 3] = 0b1110_1101;
            buf[i + 4] = 0b1011_0000 | (code >> 6 & 0x0F) as u8;
            buf[i + 5] = 0b1000_0000 | (code & 0x3F) as u8;
            j += 6;
            i += 4;
        } else if first == 0 {
            buf[j + 0] = 0xC0;
            buf[j + 1] = 0x80;
            j += 2;
            i += 1;
        } else {
            buf[j] = v[i];
            j += 1;
            i += 1;
        }
    }

    buf
}

use super::iter::JavaChars;
use super::{validate, EncodingError, JavaString};

/// A Modified UTF-8 string slice. This is the encoding that Java uses for
/// strings. This string does support unpaired surrogates, which are
/// invalid Unicode code points.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JavaStr {
    bytes: [u8],
}

impl JavaStr {
    /// Converts a slice of bytes to a `JavaStr`.
    ///
    /// If you are sure that the byte slice is valid Modified UTF-8, and you
    /// don't want to incur the overhead of the validity check, there is an
    /// unsafe version of this function [`from_java_unchecked`], which has the
    /// same behavior but skips the check.
    ///
    /// [`from_java_unchecked`]: JavaStr::from_java_unchecked
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not valid Modified UTF-8 with the first
    /// index at which the string is invalid, and the length of the error.
    #[inline]
    pub const fn from_java(v: &[u8]) -> Result<&JavaStr, EncodingError> {
        match validate(v) {
            Ok(()) => Ok(unsafe { JavaStr::from_java_unchecked(v) }),
            Err(e) => Err(e),
        }
    }

    /// Converts a mutable slice of bytes to a mutable `JavaStr`.
    ///
    /// If you are sure that the byte slice is valid Modified UTF-8, and you
    /// don't want to incur the overhead of the validity check, there is an
    /// unsafe version of this function [`from_java_unchecked_mut`], which has
    /// the same behavior but skips the check.
    ///
    /// [`from_java_unchecked_mut`]: JavaStr::from_java_unchecked_mut
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the slice is not valid Modified UTF-8 with the first
    /// index at which the string is invalid, and the length of the error.
    #[inline]
    pub fn from_java_mut(v: &mut [u8]) -> Result<&mut JavaStr, EncodingError> {
        match validate(v) {
            Ok(()) => Ok(unsafe { JavaStr::from_java_unchecked_mut(v) }),
            Err(e) => Err(e),
        }
    }

    /// Converts a slice of bytes to a `JavaStr` without checking that the
    /// string contains valid Modified UTF-8.
    ///
    /// See the safe version, [`from_java`], for more details.
    ///
    /// [`from_java`]: JavaStr::from_java
    ///
    /// # Safety
    ///
    /// The bytes passed in must be valid Modified UTF-8.
    #[inline]
    #[must_use]
    pub const unsafe fn from_java_unchecked(v: &[u8]) -> &JavaStr {
        unsafe { &*(v as *const [u8] as *const JavaStr) }
    }

    /// Converts a mutable slice of bytes to a mutable `JavaStr` without
    /// checking that the string contains valid Modified UTF-8.
    ///
    /// See the safe version, [`from_java_mut`], for more details.
    ///
    /// [`from_java_mut`]: JavaStr::from_java_mut
    ///
    /// # Safety
    ///
    /// The bytes passed in must be valid Modified UTF-8.
    #[inline]
    #[must_use]
    pub unsafe fn from_java_unchecked_mut(v: &mut [u8]) -> &mut JavaStr {
        unsafe { &mut *(v as *mut [u8] as *mut JavaStr) }
    }

    /// Returns the length of `self`.
    ///
    /// This length is in bytes, not [`char`]s or graphemes. In other words, it
    /// might not be what a human considers the length of the string.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Converts a string slice into a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns an iterator over the code points of a string slice.
    ///
    /// As an `Cesu8Str` consists of valid CESU-8, we can iterate through a
    /// string by code point. This method returns such an iterator.
    ///
    /// It's important to remember that code points represent Unicode Scalar
    /// Values, and might not match your idea of what a 'character' is.
    /// Iteration over grapheme clusters may be what you actually want. This
    /// functionality is not provided by this crate.
    #[inline]
    pub const fn chars(&self) -> JavaChars {
        JavaChars { slice: &self.bytes }
    }
}

impl ToOwned for JavaStr {
    type Owned = JavaString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        let vec = self.as_bytes().to_owned();
        unsafe { JavaString::from_java_unchecked(vec) }
    }
}

impl AsRef<[u8]> for JavaStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl std::fmt::Debug for JavaStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        f.write_str("\"")?;
        for c in self.chars() {
            if let Some(c) = char::from_u32(c) {
                for c in c.escape_debug() {
                    f.write_char(c)?;
                }
            } else {
                f.write_str("\u{FFFD}")?;
            }
        }
        f.write_str("\"")
    }
}

impl std::fmt::Display for JavaStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        for c in self.chars() {
            if let Some(c) = char::from_u32(c) {
                f.write_char(c)?;
            } else {
                f.write_str("\u{FFFD}")?;
            }
        }
        Ok(())
    }
}

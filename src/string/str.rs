use std::ops::{Index, IndexMut, RangeBounds};

use super::iter::JavaChars;
use super::{check_surrogate_index, validate, EncodingError, JavaString};

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

    /// Checks that the `index`-th byte is the first by in a Modified UTF-8 code
    /// point sequence or is at the end of the string. This will report the
    /// second code unit of a valid surrogate pair as a char boundary.
    ///
    /// The start and end of the string (when `index == self.len()`) are
    /// considered to be boundaries.
    ///
    /// Returns `false` if `index` is greater than `self.len()`.
    #[inline]
    #[must_use]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        // 0 is always ok. This is a fast path so that it can optimize out
        // checks easily and skip reading string data for that case.
        if index == 0 {
            return true;
        }

        match self.bytes.get(index) {
            None => index == self.len(),

            // Check whether inside valid surrogate pair
            Some(&b) => {
                if b < 128 || b & 0xE0 == 0xC0 {
                    true
                } else if b & 0xF0 == 0xE0 {
                    // Check whether this is the second part of a surrogate pair
                    if self.bytes.len() - index > 3 && check_surrogate_index(&self.bytes, index - 3)
                    {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Converts a string slice into a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns a subslice of `JavaStr`.
    ///
    /// This is the non-panicking alternative to index-ing the `JavaStr`.
    /// Returns [`None`] whenever equivalent indexing operations would panic.
    #[inline]
    #[must_use]
    pub fn get<I: RangeBounds<usize>>(&self, index: I) -> Option<&JavaStr> {
        let (start, end) = self.get_checked_bounds(index)?;
        Some(unsafe { Self::from_java_unchecked(self.bytes.get_unchecked(start..end)) })
    }

    /// Returns a mutable subslice of `JavaStr`.
    ///
    /// This is the non-panicking alternative to index-ing the `JavaStr`.
    /// Returns [`None`] whenever equivalent indexing operations would panic.
    #[inline]
    #[must_use]
    pub fn get_mut<I: RangeBounds<usize>>(&mut self, index: I) -> Option<&mut JavaStr> {
        let (start, end) = self.get_checked_bounds(index)?;
        Some(unsafe { Self::from_java_unchecked_mut(self.bytes.get_unchecked_mut(start..end)) })
    }

    /// Returns a subslice of `JavaStr`.
    ///
    /// This is the unchecked alternative to indexing the `JavaStr`.
    ///
    /// # Safety
    ///
    /// Callers of this function are responsible for ensuring that:
    /// * The starting index does not exceed the ending index;
    /// * The indices are within the bounds of the original slice;
    /// * The indices fall on Modified UTF-8 code unit boundaries.
    ///
    /// Failing that, the returned string slice may reference invalid memory or
    /// violate invariants communicated by the `JavaStr` type.
    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked<I: RangeBounds<usize>>(&self, index: I) -> &JavaStr {
        let (start, end) = self.get_bounds(index);
        unsafe { Self::from_java_unchecked(self.bytes.get_unchecked(start..end)) }
    }

    /// Returns a subslice of `JavaStr`.
    ///
    /// This is the unchecked alternative to indexing the `JavaStr`.
    ///
    /// # Safety
    ///
    /// Callers of this function are responsible for ensuring that:
    /// * The starting index does not exceed the ending index;
    /// * The indices are within the bounds of the original slice;
    /// * The indices fall on Modified UTF-8 code unit boundaries.
    ///
    /// Failing that, the returned string slice may reference invalid memory or
    /// violate invariants communicated by the `JavaStr` type.
    #[inline]
    #[must_use]
    pub unsafe fn get_unchecked_mut<I: RangeBounds<usize>>(&mut self, index: I) -> &mut JavaStr {
        let (start, end) = self.get_bounds(index);
        unsafe { Self::from_java_unchecked_mut(self.bytes.get_unchecked_mut(start..end)) }
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

    /// Calculate the bounds for a given range.
    #[inline]
    #[must_use]
    fn get_bounds<I: RangeBounds<usize>>(&self, index: I) -> (usize, usize) {
        let start = match index.start_bound() {
            core::ops::Bound::Excluded(&x) => x + 1,
            core::ops::Bound::Included(&x) => x,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match index.end_bound() {
            core::ops::Bound::Excluded(&x) => x,
            core::ops::Bound::Included(&x) => x + 1,
            core::ops::Bound::Unbounded => self.len(),
        };
        (start, end)
    }

    /// Calculate the bounds for a given range and check that they result in
    /// valid start and end indices.
    #[inline]
    #[must_use]
    fn get_checked_bounds<I: RangeBounds<usize>>(&self, index: I) -> Option<(usize, usize)> {
        let (start, end) = self.get_bounds(index);
        if start > end || end > self.len() {
            return None;
        }

        if !self.is_char_boundary(start) || !self.is_char_boundary(end) {
            return None;
        }

        Some((start, end))
    }

    /// Panics if the range is invalid.
    ///
    /// # Panics
    ///
    /// Panics when:
    /// * `start` or `end` are out of bounds
    /// * `start` > `end`
    /// * `start` or `end` are not on character boundaries
    #[inline]
    #[track_caller]
    fn check_index_internal(&self, start: usize, end: usize) {
        // Slice
        assert!(
            start <= end,
            "slice index starts at {start} but ends at {end}"
        );
        assert!(
            start <= self.len(),
            "start index {start} out of range for str of length {}",
            self.len(),
        );
        assert!(
            end <= self.len(),
            "end index {end} out of range for str of length {}",
            self.len(),
        );

        // str-specific
        assert!(
            self.is_char_boundary(start),
            "byte index {start} is not a char boundary"
        );
        assert!(
            self.is_char_boundary(end),
            "byte index {end} is not a char boundary"
        );
    }

    /// Returns an immutable `JavaStr`. Panics if the range is invalid.
    ///
    /// # Panics
    ///
    /// Panics when:
    /// * `start` or `end` are out of bounds
    /// * `start` > `end`
    /// * `start` or `end` are not on character boundaries
    #[inline]
    #[must_use]
    #[track_caller]
    fn index_internal(&self, start: usize, end: usize) -> &JavaStr {
        self.check_index_internal(start, end);
        unsafe { self.get_unchecked(start..end) }
    }

    /// Returns a mutable `JavaStr`. Panics if the range is invalid.
    ///
    /// # Panics
    ///
    /// Panics when:
    /// * `start` or `end` are out of bounds
    /// * `start` > `end`
    /// * `start` or `end` are not on character boundaries
    #[inline]
    #[must_use]
    #[track_caller]
    fn index_internal_mut(&mut self, start: usize, end: usize) -> &mut JavaStr {
        self.check_index_internal(start, end);
        unsafe { self.get_unchecked_mut(start..end) }
    }
}

impl<T: RangeBounds<usize>> Index<T> for JavaStr {
    type Output = JavaStr;

    fn index(&self, index: T) -> &Self::Output {
        let (start, end) = self.get_bounds(index);
        self.index_internal(start, end)
    }
}

impl<T: RangeBounds<usize>> Index<T> for &JavaStr {
    type Output = JavaStr;

    fn index(&self, index: T) -> &Self::Output {
        let (start, end) = self.get_bounds(index);
        self.index_internal(start, end)
    }
}

impl<T: RangeBounds<usize>> Index<T> for &mut JavaStr {
    type Output = JavaStr;

    fn index(&self, index: T) -> &Self::Output {
        let (start, end) = self.get_bounds(index);
        self.index_internal(start, end)
    }
}

impl<T: RangeBounds<usize>> IndexMut<T> for JavaStr {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let (start, end) = self.get_bounds(index);
        self.index_internal_mut(start, end)
    }
}

impl<T: RangeBounds<usize>> IndexMut<T> for &mut JavaStr {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let (start, end) = self.get_bounds(index);
        self.index_internal_mut(start, end)
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

impl PartialEq<&JavaStr> for JavaStr {
    #[inline]
    fn eq(&self, other: &&JavaStr) -> bool {
        PartialEq::eq(self, *other)
    }
}

impl PartialEq<JavaStr> for &JavaStr {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(*self, other)
    }
}

impl PartialEq<str> for JavaStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        for (java, utf8) in self.chars().zip(other.chars()) {
            if let Some(java) = char::from_u32(java) {
                if java == utf8 {
                    continue;
                }
            }
            return false;
        }
        true
    }
}

impl PartialEq<JavaStr> for str {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(other, self)
    }
}

impl PartialEq<&JavaStr> for str {
    #[inline]
    fn eq(&self, other: &&JavaStr) -> bool {
        PartialEq::eq(*other, self)
    }
}

impl PartialEq<&str> for JavaStr {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self, *other)
    }
}

impl PartialEq<JavaStr> for &str {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(other, *self)
    }
}

impl PartialEq<str> for &JavaStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(*self, other)
    }
}

impl PartialEq<String> for JavaStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl PartialEq<JavaStr> for String {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(other, self.as_str())
    }
}

impl PartialEq<&String> for JavaStr {
    #[inline]
    fn eq(&self, other: &&String) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl PartialEq<&JavaStr> for String {
    #[inline]
    fn eq(&self, other: &&JavaStr) -> bool {
        PartialEq::eq(*other, self.as_str())
    }
}

impl PartialEq<String> for &JavaStr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(*self, other.as_str())
    }
}

impl PartialEq<JavaStr> for &String {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(other, self.as_str())
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

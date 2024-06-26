use std::ops::{Deref, DerefMut};

use super::JavaStr;

/// A Modified UTF-8 string. This is the encoding that Java uses for strings.
/// This string does support unpaired surrogates, which are invalid Unicode code
/// points.
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JavaString {
    vec: Vec<u8>,
}

impl JavaString {
    /// Creates a new empty `JavaString`.
    ///
    /// Given that the `JavaString` is empty, this will not allocate any
    /// initial buffer. While that means that this initial operations is very
    /// inexpensive, it may cause excessive allocation later when you add data.
    /// If you have an idea of how much data the `JavaString` will hold,
    /// consider the [`with_capacity`] method to prevent excessive
    /// re-allocation.
    ///
    /// [`with_capacity`]: Self::with_capacity
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    /// Creates a new empty `JavaString` with at least the specified
    /// capacity.
    ///
    /// `JavaString`s have an internal buffer to hold their data. The
    /// capacity is at length of that buffer, and can be queried with the
    /// [`capacity`] method. This method creates an empty `JavaString`, but
    /// one with an initial buffer that can hold at least `capacity` bytes. This
    /// is useful when you may be appending a bunch of data to the
    /// `JavaString`, reducing the number of reallocations it needs to do.
    ///
    /// [`capacity`]: Self::capacity
    ///
    /// If the given capacity is `0`, no allocation will occur, and this method
    /// is identical to the [`new`] method.
    ///
    /// [`new`]: Self::new
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> JavaString {
        JavaString {
            vec: Vec::with_capacity(capacity),
        }
    }

    /// Converts a vector of bytes to a `JavaString` without checking that the
    /// vector contains valid Modified UTF-8.
    ///
    /// # Safety
    ///
    /// The vector of bytes passed in must be valid Modified UTF-8.
    #[inline]
    #[must_use]
    pub const unsafe fn from_java_unchecked(vec: Vec<u8>) -> JavaString {
        JavaString { vec }
    }

    /// Extracts a string slice containing the entire `JavaStr`.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &JavaStr {
        self
    }

    /// Returns a mutable reference to the contents of this `JavaString`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the returned `&mut Vec` allows writing
    /// bytes which are not valid Modified UTF-8. If this constraint is
    /// violated, using the original `JavaString` after dropping the `&mut Vec`
    /// may violate memory safety as `JavaString`s are expected to always
    /// contain valid Modified UTF-8.
    #[inline]
    #[must_use]
    pub(crate) unsafe fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.vec
    }
}

impl Default for JavaString {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl std::borrow::Borrow<JavaStr> for JavaString {
    #[inline]
    #[must_use]
    fn borrow(&self) -> &JavaStr {
        self
    }
}

impl Deref for JavaString {
    type Target = JavaStr;

    #[inline]
    #[must_use]
    fn deref(&self) -> &Self::Target {
        unsafe { JavaStr::from_java_unchecked(&self.vec) }
    }
}

impl DerefMut for JavaString {
    #[inline]
    #[must_use]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { JavaStr::from_java_unchecked_mut(&mut self.vec) }
    }
}

impl PartialEq<&JavaString> for JavaString {
    #[inline]
    fn eq(&self, other: &&JavaString) -> bool {
        PartialEq::eq(self, *other)
    }
}

impl PartialEq<JavaString> for &JavaString {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(*self, other)
    }
}

impl PartialEq<JavaStr> for JavaString {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<JavaString> for JavaStr {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl PartialEq<&JavaStr> for JavaString {
    #[inline]
    fn eq(&self, other: &&JavaStr) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<&JavaString> for JavaStr {
    #[inline]
    fn eq(&self, other: &&JavaString) -> bool {
        PartialEq::eq(self, other.as_str())
    }
}

impl PartialEq<JavaStr> for &JavaString {
    #[inline]
    fn eq(&self, other: &JavaStr) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<JavaString> for &JavaStr {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(*self, other.as_str())
    }
}

impl PartialEq<str> for JavaString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<JavaString> for str {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(other.as_str(), self)
    }
}

impl PartialEq<&str> for JavaString {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<&JavaString> for str {
    #[inline]
    fn eq(&self, other: &&JavaString) -> bool {
        PartialEq::eq(other.as_str(), self)
    }
}

impl PartialEq<str> for &JavaString {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<JavaString> for &str {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(other.as_str(), *self)
    }
}

impl PartialEq<String> for JavaString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<JavaString> for String {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(other.as_str(), self.as_str())
    }
}

impl PartialEq<&String> for JavaString {
    #[inline]
    fn eq(&self, other: &&String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<&JavaString> for String {
    #[inline]
    fn eq(&self, other: &&JavaString) -> bool {
        PartialEq::eq(other.as_str(), self.as_str())
    }
}

impl PartialEq<String> for &JavaString {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other.as_str())
    }
}

impl PartialEq<JavaString> for &String {
    #[inline]
    fn eq(&self, other: &JavaString) -> bool {
        PartialEq::eq(other.as_str(), self.as_str())
    }
}

impl std::fmt::Debug for JavaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl std::fmt::Display for JavaString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}

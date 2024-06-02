use super::{next_code_point, next_code_point_reverse, JavaStr};

/// An iterator over the code points of a Modified UTF-8 string slice.
///
/// This struct is created by the `chars` method on the `JavaStr`. See its
/// documentation for more detail.
///
/// [`chars`]: JavaStr::chars
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct JavaChars<'a> {
    pub(crate) slice: &'a [u8],
}

impl<'a> JavaChars<'a> {
    /// Returns the underlying bytes of the iterated string. These bytes will be
    /// in a valid format and will always start on a character boundary.
    #[inline]
    #[must_use]
    pub fn as_bytes(&self) -> &JavaStr {
        // SAFETY: The bytes come from a Cesu8Str, so they must be in a valid format.
        unsafe { JavaStr::from_java_unchecked(self.slice) }
    }
}

impl Iterator for JavaChars<'_> {
    type Item = u32;

    #[inline]
    #[must_use]
    fn next(&mut self) -> Option<Self::Item> {
        let (slice, code_point) = unsafe { next_code_point(self.slice) }?;
        self.slice = slice;
        Some(code_point)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.slice.len();
        // `(len + 5` can't overflow, because we know that
        // `slice::Iter` belongs to a slice in memory which has a
        // maximum length of `isize::MAX` (which is well below
        // `usize::MAX`).
        ((len + 5) / 6, Some(len))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl DoubleEndedIterator for JavaChars<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let (slice, code_point) = unsafe { next_code_point_reverse(self.slice) }?;
        self.slice = slice;
        Some(code_point)
    }
}

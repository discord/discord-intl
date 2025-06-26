use crate::syntax::TextSize;
use std::collections::Bound;
use std::ops::{Deref, Range, RangeBounds};
use std::rc::Rc;

// TODO: Implement static text pointers with `lazy_static!`?
// TODO: Implement general text interning? Less useful for speed, more for memory

/// A flyweight handle to a segment of text. The pointer contains a reference
/// to some string slice, an offset within that slice to the start of the
/// pointed text, and the byte length of the text to use.
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TextPointer {
    source: Rc<str>,
    offset: TextSize,
    len: TextSize,
}

impl TextPointer {
    pub fn new(source: Rc<str>, offset: TextSize, len: TextSize) -> Self {
        Self {
            source,
            offset,
            len,
        }
    }

    /// Create a new empty [TextPointer] that points to "nothing" within the given source text.
    /// Making a new pointer this way works as a more-accurate default that allows `.extend_back()`
    /// and `.extend_front()` to function as expected without having to use an `Option` or create a
    /// special case for the first element.
    pub fn empty_from(source: Rc<str>) -> Self {
        Self {
            source,
            offset: 0,
            len: 0,
        }
    }

    pub fn from_str(source: &str) -> Self {
        Self {
            source: Rc::from(source),
            offset: 0,
            len: source.len() as TextSize,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.source[self.range()]
    }

    pub fn start(&self) -> TextSize {
        self.offset
    }

    pub fn end(&self) -> TextSize {
        self.offset + self.len
    }

    pub fn range(&self) -> Range<usize> {
        self.offset as usize..(self.offset + self.len) as usize
    }

    pub fn len_size(&self) -> TextSize {
        self.len as TextSize
    }

    /// Returns true if this pointer references a standalone piece of text, likely meaning it was
    /// cloned while editing the trivia of the token.
    pub fn is_detached(&self) -> bool {
        self.len() == self.source.len()
    }

    /// Returns true if this text pointer references a text range that is immediately before the
    /// range that `next` points to within the same source string.
    pub fn is_adjacent_before(&self, next: &Self) -> bool {
        Rc::ptr_eq(&self.source, &next.source) && self.offset + self.len == next.offset
    }

    /// Returns true if this text pointer references a text range that is immediately after the
    /// range that `next` points to within the same source string.
    pub fn is_adjacent_after(&self, next: &Self) -> bool {
        next.is_adjacent_before(self)
    }

    /// Return a slice of this text pointer, almost identically to `&text[start..end]`, but
    /// returning a new TextPointer to avoid lifetime issues when dealing with syntax elements.
    /// Consumers are effectively able to treat this as a `&str` when they need it, since
    /// `TextPointer` dereferences to a string slice directly.
    pub fn substr<R: RangeBounds<usize>>(&self, range: R) -> TextPointer {
        let start = match range.start_bound() {
            Bound::Included(&t) => t,
            Bound::Excluded(&t) => t.saturating_sub(1),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&t) => t + 1,
            Bound::Excluded(&t) => t,
            Bound::Unbounded => self.len(),
        }
        .min(self.len());
        TextPointer::new(
            self.source.clone(),
            self.offset + start as TextSize,
            (end - start) as TextSize,
        )
    }

    /// Remove the first `n` bytes from this pointer's range.
    #[must_use = "TextPointers are immutable and any changes must be propagated to the parent for them to have an effect"]
    pub fn trim_front(mut self, n: TextSize) -> Self {
        debug_assert!(
            n <= self.len,
            "Tried to trim {n} bytes from a text pointer of length {}",
            self.len
        );
        self.offset += n;
        self.len -= n;
        self
    }

    /// Remove the last `n` bytes from this pointer's range.
    #[must_use = "TextPointers are immutable and any changes must be propagated to the parent for them to have an effect"]
    pub fn trim_back(mut self, n: TextSize) -> Self {
        debug_assert!(
            n <= self.len,
            "Tried to trim {n} bytes from a text pointer of length {}",
            self.len
        );
        self.len -= n;
        self
    }

    /// Creates a new TextPointer containing the content of both this pointer and `text`. If the
    /// text slice points to an adjacent subrange of this pointer's source, then this pointer is
    /// simply expanded to include that text in its range. If the given text is _not_ adjacent to
    /// this pointer's range in the source text, then the original pointer text is copied into a
    /// new string with the given text appended to it.
    ///
    /// NOTE: If the given text is empty, this method will still create and return a new
    /// TextPointer, but the content will not be changed. To avoid making new pointers, check
    /// whether the text is empty before calling this method.
    #[must_use = "TextPointers are immutable and any changes must be propagated to the parent for them to have an effect"]
    pub fn extend_back(&self, text: &str) -> Self {
        if text.is_empty() {
            return self.clone();
        }

        // If this pointer is empty but the given text is a substring of the same source, then pivot
        // this pointer to just be the given text.
        if self.is_empty() {
            if let Some(range) = self.source.substr_range(text) {
                let mut clone = self.clone();
                clone.offset = range.start as TextSize;
                clone.len = range.len() as TextSize;
                return clone;
            }
        }

        let is_adjacent_end = self
            .source
            .substr_range(text)
            .is_some_and(|range| self.offset + self.len == range.start as TextSize);

        if is_adjacent_end {
            // From checking that this is a subrange of the original source, we can safely know
            // that the new end position is valid within the original source as well.
            let mut clone = self.clone();
            clone.len = self.len + text.len() as TextSize;
            return clone;
        }

        // If it's not adjacent, we have to copy the text to a new string to ensure we have a safe
        // and contiguous slice to reference. We're effectively creating a new text pointer from
        // scratch with only this text as the source.
        let mut new_text = String::with_capacity(self.len() + text.len());
        new_text.push_str(&self);
        new_text.push_str(text);
        Self::from_str(&new_text)
    }

    /// Like [`extend_back`], but expanding to include or placing the given text at the start of
    /// the pointer rather than the end.
    ///
    /// NOTE: If the given text is empty, this method will still create and return a new
    /// TextPointer, but the content will not be changed. To avoid making new pointers, check
    /// whether the text is empty before calling this method.
    #[must_use = "TextPointers are immutable and any changes must be propagated to the parent for them to have an effect"]
    pub fn extend_front(&self, text: &str) -> Self {
        if text.is_empty() {
            return self.clone();
        }

        // If this pointer is empty but the given text is a substring of the same source, then pivot
        // this pointer to just be the given text.
        if self.is_empty() {
            if let Some(range) = self.source.substr_range(text) {
                let mut clone = self.clone();
                clone.offset = range.start as TextSize;
                clone.len = range.len() as TextSize;
                return clone;
            }
        }

        let is_adjacent_start = self
            .source
            .substr_range(text)
            .is_some_and(|range| range.end as TextSize == self.offset);

        if is_adjacent_start {
            // From checking that this is a subrange of the original source, we can safely know
            // that the new end position is valid within the original source as well.
            let mut clone = self.clone();
            clone.offset = self.offset - text.len() as TextSize;
            clone.len += text.len() as TextSize;
            return clone;
        }

        // If it's not adjacent, we have to copy the text to a new string to ensure we have a safe
        // and contiguous slice to reference. We're effectively creating a new text pointer from
        // scratch with only this text as the source.
        let mut new_text = String::with_capacity(self.len() + text.len());
        new_text.push_str(text);
        new_text.push_str(&self);
        Self::from_str(&new_text)
    }

    /// Remove all size and position from this text pointer, but preserve the reference to the
    /// original text to ensure it can be reused without cloning.
    pub(super) fn clear(&mut self) {
        self.len = 0;
    }
}

impl From<&str> for TextPointer {
    fn from(text: &str) -> Self {
        TextPointer::from_str(text)
    }
}

impl From<Box<str>> for TextPointer {
    fn from(text: Box<str>) -> Self {
        let len = text.len() as TextSize;
        TextPointer::new(Rc::from(text), 0, len)
    }
}

impl FromIterator<TextPointer> for Option<TextPointer> {
    fn from_iter<T: IntoIterator<Item = TextPointer>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|previous, next| previous.extend_back(&next))
    }
}

impl FromIterator<TextPointer> for TextPointer {
    fn from_iter<T: IntoIterator<Item = TextPointer>>(iter: T) -> Self {
        iter.into_iter()
            .reduce(|previous, next| previous.extend_back(&next))
            .unwrap_or(TextPointer::default())
    }
}
impl<'a> FromIterator<&'a TextPointer> for TextPointer {
    fn from_iter<T: IntoIterator<Item = &'a TextPointer>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let first = iter.next().cloned().unwrap_or(TextPointer::default());
        iter.into_iter()
            .fold(first, |previous, next| previous.extend_back(&next))
    }
}

impl<'a, S: AsRef<str>> FromIterator<S> for TextPointer {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        iter.into_iter()
            .fold(TextPointer::default(), |pointer, text| {
                pointer.extend_back(text.as_ref())
            })
    }
}

impl Deref for TextPointer {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TextPointer {
    pub fn format_range(&self) -> String {
        if self.is_detached() {
            "copy.".into()
        } else {
            format!("{}..{}", self.start(), self.end())
        }
    }
}

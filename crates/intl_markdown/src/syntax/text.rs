use crate::syntax::TextSize;
use std::ops::{Deref, Range};
use std::rc::Rc;

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

    pub fn from_str(source: &str) -> Self {
        Self {
            source: Rc::from(source),
            offset: 0,
            len: source.len() as TextSize,
        }
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

    /// Extend this text pointer to include the given `text`. If the text slice points to an
    /// adjacent subrange of this pointer's source, then this pointer is simply expanded to include
    /// that text in its range. If the given text is _not_ adjacent to this pointer's range in the
    /// source text, then the original pointer text is copied into a new string with the given text
    /// appended to it.
    ///
    /// NOTE: If the given text is empty, this method will still create and return a new
    /// TextPointer, but the content will not be changed. To avoid making new pointers, check
    /// whether the text is empty before calling this method.
    #[must_use = "TextPointers are immutable and any changes must be propagated to the parent for them to have an effect"]
    pub fn extend_back(&self, text: &str) -> Self {
        if text.is_empty() {
            return self.clone();
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
        let is_adjacent_start = self
            .source
            .substr_range(text)
            .is_some_and(|range| range.end as TextSize == self.offset);

        if is_adjacent_start {
            // From checking that this is a subrange of the original source, we can safely know
            // that the new end position is valid within the original source as well.
            let mut clone = self.clone();
            clone.offset = self.offset - text.len() as TextSize;
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
}

impl From<&str> for TextPointer {
    fn from(text: &str) -> Self {
        TextPointer::from_str(text)
    }
}

impl Deref for TextPointer {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.source[self.range()]
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

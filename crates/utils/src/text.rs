use ropey::Rope;
use std::ops::{Bound::*, RangeBounds};

#[derive(Debug, Default, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct LongOptimizedText {
    rope: Rope,
}

impl LongOptimizedText {
    pub fn export_string(&self) -> String {
        self.rope.chunks().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn get_string(&self, start: usize, length: usize) -> String {
        self.rope.chars_at(start).take(length).collect()
    }

    pub fn insert_str(&mut self, char_idx: usize, text: &str) {
        self.rope.insert(char_idx, text);
    }

    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        self.rope.insert_char(char_idx, ch);
    }

    pub fn remove_range<R>(&mut self, char_range: R)
    where
        R: RangeBounds<usize>,
    {
        self.rope.remove(char_range)
    }

    pub fn clear(&mut self) {
        self.rope = Rope::new();
    }
}

impl<S: Into<String>> From<S> for LongOptimizedText {
    fn from(s: S) -> Self {
        Self {
            rope: Rope::from_str(&s.into()),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
enum LongOrShortTextInner {
    Long(LongOptimizedText),
    Short(String),
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Text {
    inner: LongOrShortTextInner,
}

impl<S: Into<String>> From<S> for Text {
    fn from(string: S) -> Self {
        Self {
            inner: LongOrShortTextInner::Short(string.into()),
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Self {
            inner: LongOrShortTextInner::Short("".to_owned()),
        }
    }
}

impl Text {
    pub fn long_text_mode(&mut self) {
        if let LongOrShortTextInner::Short(ref s) = self.inner {
            self.inner = LongOrShortTextInner::Long(LongOptimizedText::from(s));
        }
    }

    pub fn short_text_mode(&mut self) {
        if let LongOrShortTextInner::Long(ref opt) = self.inner {
            self.inner = LongOrShortTextInner::Short(opt.export_string());
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.inner {
            LongOrShortTextInner::Long(ref opt) => opt.is_empty(),
            LongOrShortTextInner::Short(ref s) => s.is_empty(),
        }
    }

    pub fn len_chars(&self) -> usize {
        match self.inner {
            LongOrShortTextInner::Long(ref opt) => opt.len_chars(),
            LongOrShortTextInner::Short(ref s) => s.chars().count(),
        }
    }

    pub fn export_string(&self) -> String {
        match self.inner {
            LongOrShortTextInner::Long(ref opt) => opt.export_string(),
            LongOrShortTextInner::Short(ref s) => s.to_owned(),
        }
    }

    /// Gets a range of the text, returns an empty string if its completely out of bounds
    pub fn get_string(&self, start: usize, length: usize) -> String {
        match self.inner {
            LongOrShortTextInner::Long(ref opt) => opt.get_string(start, length),
            LongOrShortTextInner::Short(ref s) => s.chars().skip(start).take(length).collect(),
        }
    }

    pub fn insert_str(&mut self, char_idx: usize, text: &str) {
        match self.inner {
            LongOrShortTextInner::Long(ref mut opt) => opt.insert_str(char_idx, text),
            LongOrShortTextInner::Short(ref mut s) => {
                let mut chrs = s.chars();
                let mut result = String::new();
                (0..char_idx)
                    .filter_map(|_| chrs.next())
                    .for_each(|c| result.push(c));
                result.push_str(text);
                chrs.for_each(|c| result.push(c));
                *s = result;
            }
        }
    }

    pub fn push(&mut self, ch: char) {
        match self.inner {
            LongOrShortTextInner::Long(ref mut opt) => opt.insert_char(opt.len_chars(), ch),
            LongOrShortTextInner::Short(ref mut s) => s.push(ch),
        }
    }

    pub fn remove_range<R>(&mut self, char_range: R)
    where
        R: RangeBounds<usize>,
    {
        match self.inner {
            LongOrShortTextInner::Long(ref mut opt) => opt.remove_range(char_range),
            LongOrShortTextInner::Short(ref mut s) => {
                let mut chrs = s.chars();
                let mut result = String::new();
                let first_half_offset = match char_range.start_bound() {
                    Included(i) => *i,
                    Excluded(i) => *i + 1,
                    Unbounded => 0,
                };
                let second_half_offset = match char_range.end_bound() {
                    Included(i) => *i + 1,
                    Excluded(i) => *i,
                    Unbounded => 0,
                };
                (0..first_half_offset)
                    .filter_map(|_| chrs.next())
                    .for_each(|c| result.push(c));
                chrs.skip(second_half_offset - first_half_offset)
                    .for_each(|c| result.push(c));
                *s = result;
            }
        }
    }

    pub fn clear(&mut self) {
        match self.inner {
            LongOrShortTextInner::Long(ref mut opt) => opt.clear(),
            LongOrShortTextInner::Short(ref mut s) => s.clear(),
        }
    }
}

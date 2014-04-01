// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Internal utilities.

use std::str::CharRange;
use std::cast::transmute;

/// Unchecked conversion to `char`.
pub fn as_char<T:Int+NumCast>(ch: T) -> char {
    unsafe { transmute(ch.to_u32().unwrap()) }
}

/// External iterator for a string's characters with its corresponding byte offset range.
pub struct StrCharIndexIterator<'r> {
    index: uint,
    string: &'r str,
}

impl<'r> Iterator<((uint,uint), char)> for StrCharIndexIterator<'r> {
    #[inline]
    fn next(&mut self) -> Option<((uint,uint), char)> {
        if self.index < self.string.len() {
            let CharRange {ch, next} = self.string.char_range_at(self.index);
            let prev = self.index;
            self.index = next;
            Some(((prev, next), ch))
        } else {
            None
        }
    }
}

/// A trait providing an `index_iter` method.
pub trait StrCharIndex<'r> {
    fn index_iter(&self) -> StrCharIndexIterator<'r>;
}

impl<'r> StrCharIndex<'r> for &'r str {
    /// Iterates over each character with corresponding byte offset range.
    fn index_iter(&self) -> StrCharIndexIterator<'r> {
        StrCharIndexIterator { index: 0, string: *self }
    }
}

// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Internal utilities.

use std::str::CharRange;

/// External iterator for a string's characters with its corresponding byte offset range.
pub struct StrCharIndexIterator<'self> {
    priv index: uint,
    priv string: &'self str,
}

impl<'self> Iterator<((uint,uint), char)> for StrCharIndexIterator<'self> {
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
pub trait StrCharIndex<'self> {
    fn index_iter(&self) -> StrCharIndexIterator<'self>;
}

impl<'self> StrCharIndex<'self> for &'self str {
    /// Iterates over each character with corresponding byte offset range.
    fn index_iter(&self) -> StrCharIndexIterator<'self> {
        StrCharIndexIterator { index: 0, string: *self }
    }
}

// TODO: remove this when upgrading to Rust 0.8 and use std::ascii::StrAsciiExt instead.
pub trait StrAsciiExt {
    fn to_ascii_lower(&self) -> ~str;
}

impl<'self> StrAsciiExt for &'self str {
    #[inline]
    fn to_ascii_lower(&self) -> ~str {
        do self.map_chars |c| {
            match c {
                'A'..'Z' => c + 'a' - 'A',
                _ => c
            }
        }
    }
}

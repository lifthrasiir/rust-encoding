// This is a part of rust-encoding.
//
// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

//! Simplified Chinese index tables for
//! [rust-encoding](https://github.com/lifthrasiir/rust-encoding).

#![feature(macro_rules)]

#[cfg(test)]
#[path = "../index_tests.rs"]
mod tests;

/// GB 18030 two-byte area.
///
/// From the Encoding Standard:
///
/// > This matches the GB18030 standard for code points encoded as two bytes,
/// > except `0xA3 0xA0` maps to U+3000 to be compatible with deployed content.
#[stable] pub mod gb18030;

/// GB 18030 four-byte area.
///
/// From the Encoding Standard:
///
/// > This index works different from all others.
/// > Listing all code points would result in over a million items
/// > whereas they can be represented neatly in 207 ranges combined with trivial limit checks.
/// > It therefore only superficially matches the GB18030 standard
/// > for code points encoded as four bytes.
/// > See also index gb18030 ranges code point and index gb18030 ranges pointer below.
#[stable] pub mod gb18030_ranges;


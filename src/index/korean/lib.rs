// This is a part of rust-encoding.
//
// Any copyright is dedicated to the Public Domain.
// https://creativecommons.org/publicdomain/zero/1.0/

//! Korean index tables for [rust-encoding](https://github.com/lifthrasiir/rust-encoding).

#![feature(macro_rules)]

#[cfg(test)]
#[path = "../index_tests.rs"]
mod tests;

/// KS X 1001 plus Unified Hangul Code.
///
/// From the Encoding Standard:
///
/// > This matches the KS X 1001 standard and the Unified Hangul Code,
/// > more commonly known together as Windows Codepage 949.
#[stable] pub mod euc_kr;


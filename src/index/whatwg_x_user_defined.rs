// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Indices for WHATWG x-user-defined encoding.

#[inline]
pub fn forward(code: u8) -> u16 {
    0xf780 + (code as u16)
}

#[inline]
pub fn backward(code: u16) -> u8 {
    if 0xf780 <= code && code <= 0xf7ff {(code - 0xf780) as u8} else {0xff}
}


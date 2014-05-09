// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Common codec implementation for single-byte encodings.

use util::{as_char, StrCharIndex};
use types::*;

/// A common framework for single-byte encodings based on ASCII.
pub struct SingleByteEncoding {
    pub name: &'static str,
    pub whatwg_name: Option<&'static str>,
    pub index_forward: extern "Rust" fn(u8) -> u16,
    pub index_backward: extern "Rust" fn(u16) -> u8,
}

impl Encoding for SingleByteEncoding {
    fn name(&self) -> &'static str { self.name }
    fn whatwg_name(&self) -> Option<&'static str> { self.whatwg_name }
    fn encoder(&'static self) -> Box<Encoder> { SingleByteEncoder::new(self.index_backward) }
    fn decoder(&'static self) -> Box<Decoder> { SingleByteDecoder::new(self.index_forward) }
}

/// An encoder for single-byte encodings based on ASCII.
#[deriving(Clone)]
pub struct SingleByteEncoder {
    index_backward: extern "Rust" fn(u16) -> u8,
}

impl SingleByteEncoder {
    pub fn new(index_backward: extern "Rust" fn(u16) -> u8) -> Box<Encoder> {
        box SingleByteEncoder { index_backward: index_backward } as Box<Encoder>
    }
}

impl Encoder for SingleByteEncoder {
    fn from_self(&self) -> Box<Encoder> { SingleByteEncoder::new(self.index_backward) }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            if ch <= '\u007f' {
                output.write_byte(ch as u8);
                continue;
            }
            if ch <= '\uffff' {
                let index = (self.index_backward)(ch as u16);
                if index != 0 {
                    output.write_byte(index);
                    continue;
                }
            }
            return (i, Some(CodecError {
                upto: j, cause: "unrepresentable character".into_maybe_owned()
            }));
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

/// A decoder for single-byte encodings based on ASCII.
#[deriving(Clone)]
pub struct SingleByteDecoder {
    index_forward: extern "Rust" fn(u8) -> u16,
}

impl SingleByteDecoder {
    pub fn new(index_forward: extern "Rust" fn(u8) -> u16) -> Box<Decoder> {
        box SingleByteDecoder { index_forward: index_forward } as Box<Decoder>
    }
}

impl Decoder for SingleByteDecoder {
    fn from_self(&self) -> Box<Decoder> { SingleByteDecoder::new(self.index_forward) }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.write_char(input[i] as char);
            } else {
                let ch = (self.index_forward)(input[i]);
                if ch != 0xffff {
                    output.write_char(as_char(ch));
                } else {
                    return (i, Some(CodecError {
                        upto: i+1, cause: "invalid sequence".into_maybe_owned()
                    }));
                }
            }
            i += 1;
        }
        (i, None)
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        None
    }
}

/// Algorithmic mapping for ISO 8859-1.
pub mod iso_8859_1 {
    #[inline] pub fn forward(code: u8) -> u16 { code as u16 }
    #[inline] pub fn backward(code: u16) -> u8 { if (code & !0x7f) == 0x80 {code as u8} else {0} }
}

#[cfg(test)]
mod tests {
    use all::ISO_8859_2;
    use types::*;

    #[test]
    fn test_encoder_non_bmp() {
        let mut e = ISO_8859_2.encoder();
        assert_feed_err!(e, "A", "\uFFFF", "B", [0x41]);
        assert_feed_err!(e, "A", "\U00010000", "B", [0x41]);
    }
}


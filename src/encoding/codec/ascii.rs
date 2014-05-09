// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! 7-bit ASCII encoding.

use util::StrCharIndex;
use types::*;

/**
 * ASCII, also known as ISO/IEC 646:US.
 *
 * It is both a basis and a lowest common denominator of many other encodings
 * including UTF-8, which Rust internally assumes.
 */
#[deriving(Clone)]
pub struct ASCIIEncoding;

impl Encoding for ASCIIEncoding {
    fn name(&self) -> &'static str { "ascii" }
    fn encoder(&self) -> Box<Encoder> { ASCIIEncoder::new() }
    fn decoder(&self) -> Box<Decoder> { ASCIIDecoder::new() }
}

/// An encoder for ASCII.
#[deriving(Clone)]
pub struct ASCIIEncoder;

impl ASCIIEncoder {
    pub fn new() -> Box<Encoder> { box ASCIIEncoder as Box<Encoder> }
}

impl Encoder for ASCIIEncoder {
    fn from_self(&self) -> Box<Encoder> { ASCIIEncoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            if ch <= '\u007f' {
                output.write_byte(ch as u8);
            } else {
                return (i, Some(CodecError {
                    upto: j, cause: "unrepresentable character".into_maybe_owned()
                }));
            }
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

/// A decoder for ASCII.
#[deriving(Clone)]
pub struct ASCIIDecoder;

impl ASCIIDecoder {
    pub fn new() -> Box<Decoder> { box ASCIIDecoder as Box<Decoder> }
}

impl Decoder for ASCIIDecoder {
    fn from_self(&self) -> Box<Decoder> { ASCIIDecoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());
                                        
        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.write_char(input[i] as char);
            } else {
                return (i, Some(CodecError {
                    upto: i+1, cause: "invalid sequence".into_maybe_owned()
                }));
            }
            i += 1;
        }
        (i, None)
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::ASCIIEncoding;
    use types::*;

    #[test]
    fn test_encoder() {
        let mut e = ASCIIEncoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_err!(e, "", "\xa0", "", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder() {
        let mut d = ASCIIEncoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_err!(d, [], [0xa0], [], "");
        assert_finish_ok!(d, "");
    }
}


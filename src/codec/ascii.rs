// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! 7-bit ASCII encoding.

use std::{str, mem};
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

        match input.as_bytes().iter().position(|&ch| ch >= 0x80) {
            Some(first_error) => {
                output.write_bytes(input.as_bytes().slice_to(first_error));
                let str::CharRange {ch: _, next} = input.char_range_at(first_error);
                (first_error, Some(CodecError {
                    upto: next, cause: "unrepresentable character".into_maybe_owned()
                }))
            }
            None => {
                output.write_bytes(input.as_bytes());
                (input.len(), None)
            }
        }
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

        fn write_ascii_bytes(output: &mut StringWriter, buf: &[u8]) {
            output.write_str(unsafe {mem::transmute(buf)});
        }

        match input.iter().position(|&ch| ch >= 0x80) {
            Some(first_error) => {
                write_ascii_bytes(output, input.slice_to(first_error));
                (first_error, Some(CodecError {
                    upto: first_error+1, cause: "invalid sequence".into_maybe_owned()
                }))
            }
            None => {
                write_ascii_bytes(output, input);
                (input.len(), None)
            }
        }
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::ASCIIEncoding;
    use testutils;
    use types::*;

    #[test]
    fn test_encoder() {
        let mut e = ASCIIEncoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_err!(e, "", "\xa0", "", []);
        assert_feed_err!(e, "X", "\xa0", "Z", [0x58]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder() {
        let mut d = ASCIIEncoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_err!(d, [], [0xa0], [], "");
        assert_feed_err!(d, [0x58], [0xa0], [0x5a], "X");
        assert_finish_ok!(d, "");
    }

    #[bench]
    fn bench_encode(bencher: &mut test::Bencher) {
        static Encoding: ASCIIEncoding = ASCIIEncoding;
        let s = testutils::ASCII_TEXT;
        bencher.bytes = s.len() as u64;
        bencher.iter(|| test::black_box({
            Encoding.encode(s, EncodeStrict)
        }))
    }

    #[bench]
    fn bench_decode(bencher: &mut test::Bencher) {
        static Encoding: ASCIIEncoding = ASCIIEncoding;
        let s = testutils::ASCII_TEXT.as_bytes();
        bencher.bytes = s.len() as u64;
        bencher.iter(|| test::black_box({
            Encoding.decode(s, DecodeStrict)
        }))
    }

    #[bench]
    fn bench_encode_replace(bencher: &mut test::Bencher) {
        static Encoding: ASCIIEncoding = ASCIIEncoding;
        let s = testutils::KOREAN_TEXT;
        bencher.bytes = s.len() as u64;
        bencher.iter(|| test::black_box({
            Encoding.encode(s, EncodeReplace)
        }))
    }

    #[bench]
    fn bench_decode_replace(bencher: &mut test::Bencher) {
        static Encoding: ASCIIEncoding = ASCIIEncoding;
        let s = testutils::KOREAN_TEXT.as_bytes();
        bencher.bytes = s.len() as u64;
        bencher.iter(|| test::black_box({
            Encoding.decode(s, DecodeReplace)
        }))
    }
}

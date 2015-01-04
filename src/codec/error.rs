// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A placeholder encoding that returns encoder/decoder error for every case.

use std::str;
use std::borrow::IntoCow;
use types::*;

/// An encoding that returns encoder/decoder error for every case.
#[derive(Clone, Copy)]
pub struct ErrorEncoding;

impl Encoding for ErrorEncoding {
    fn name(&self) -> &'static str { "error" }
    fn raw_encoder(&self) -> Box<RawEncoder> { ErrorEncoder::new() }
    fn raw_decoder(&self) -> Box<RawDecoder> { ErrorDecoder::new() }
}

/// An encoder that always returns error.
#[derive(Clone, Copy)]
pub struct ErrorEncoder;

impl ErrorEncoder {
    pub fn new() -> Box<RawEncoder> { box ErrorEncoder as Box<RawEncoder> }
}

impl RawEncoder for ErrorEncoder {
    fn from_self(&self) -> Box<RawEncoder> { ErrorEncoder::new() }

    fn raw_feed(&mut self, input: &str, _output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        if input.len() > 0 {
            let str::CharRange {ch: _, next} = input.char_range_at(0);
            (0, Some(CodecError { upto: next as int,
                                  cause: "unrepresentable character".into_cow() }))
        } else {
            (0, None)
        }
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

/// A decoder that always returns error.
#[derive(Clone, Copy)]
pub struct ErrorDecoder;

impl ErrorDecoder {
    pub fn new() -> Box<RawDecoder> { box ErrorDecoder as Box<RawDecoder> }
}

impl RawDecoder for ErrorDecoder {
    fn from_self(&self) -> Box<RawDecoder> { ErrorDecoder::new() }

    fn raw_feed(&mut self, input: &[u8], _output: &mut StringWriter) -> (uint, Option<CodecError>) {
        if input.len() > 0 {
            (0, Some(CodecError { upto: 1, cause: "invalid sequence".into_cow() }))
        } else {
            (0, None)
        }
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorEncoding;
    use types::*;

    #[test]
    fn test_encoder() {
        let mut e = ErrorEncoding.raw_encoder();
        assert_feed_err!(e, "", "A", "", []);
        assert_feed_err!(e, "", "B", "C", []);
        assert_feed_ok!(e, "", "", []);
        assert_feed_err!(e, "", "\u{a0}", "", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder() {
        let mut d = ErrorEncoding.raw_decoder();
        assert_feed_err!(d, [], [0x41], [], "");
        assert_feed_err!(d, [], [0x42], [0x43], "");
        assert_feed_ok!(d, [], [], "");
        assert_feed_err!(d, [], [0xa0], [], "");
        assert_finish_ok!(d, "");
    }
}

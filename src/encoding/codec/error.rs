// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A placeholder encoding that returns encoder/decoder error for every case.

use std::str;
use types::*;

/// An encoding that returns encoder/decoder error for every case.
#[deriving(Clone)]
pub struct ErrorEncoding;

impl Encoding for ErrorEncoding {
    fn name(&self) -> &'static str { "error" }
    fn encoder(&self) -> ~Encoder { ErrorEncoder::new() }
    fn decoder(&self) -> ~Decoder { ErrorDecoder::new() }
}

/// An encoder that always returns error.
#[deriving(Clone)]
pub struct ErrorEncoder;

impl ErrorEncoder {
    pub fn new() -> ~Encoder { ~ErrorEncoder as ~Encoder }
}

impl Encoder for ErrorEncoder {
    fn from_self(&self) -> ~Encoder { ErrorEncoder::new() }

    fn raw_feed(&mut self, input: &str, _output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        if input.len() > 0 {
            let str::CharRange {ch: _, next} = input.char_range_at(0);
            (0, Some(CodecError { upto: next, cause: "unrepresentable character".into_maybe_owned() }))
        } else {
            (0, None)
        }
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

/// A decoder that always returns error.
#[deriving(Clone)]
pub struct ErrorDecoder;

impl ErrorDecoder {
    pub fn new() -> ~Decoder { ~ErrorDecoder as ~Decoder }
}

impl Decoder for ErrorDecoder {
    fn from_self(&self) -> ~Decoder { ErrorDecoder::new() }

    fn raw_feed(&mut self, input: &[u8], _output: &mut StringWriter) -> (uint, Option<CodecError>) {
        if input.len() > 0 {
            (0, Some(CodecError { upto: 1, cause: "invalid sequence".into_maybe_owned() }))
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
        let mut e = ErrorEncoding.encoder();
        assert_feed_err!(e, "", "A", "", []);
        assert_feed_err!(e, "", "B", "C", []);
        assert_feed_ok!(e, "", "", []);
        assert_feed_err!(e, "", "\xa0", "", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder() {
        let mut d = ErrorEncoding.decoder();
        assert_feed_err!(d, [], [0x41], [], "");
        assert_feed_err!(d, [], [0x42], [0x43], "");
        assert_feed_ok!(d, [], [], "");
        assert_feed_err!(d, [], [0xa0], [], "");
        assert_finish_ok!(d, "");
    }
}


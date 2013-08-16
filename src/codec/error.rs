// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A placeholder encoding that returns encoder/decoder error for every case.

use std::str;
use types::*;

#[deriving(Clone)]
pub struct ErrorEncoding;

impl Encoding for ErrorEncoding {
    fn name(&self) -> ~str { ~"error" }
    fn encoder(&self) -> ~Encoder { ~ErrorEncoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~ErrorDecoder as ~Decoder }
}

#[deriving(Clone)]
pub struct ErrorEncoder;

impl Encoder for ErrorEncoder {
    fn encoding(&self) -> ~Encoding { ~ErrorEncoding as ~Encoding }

    fn feed<'r>(&mut self, input: &'r str, _output: &mut ~[u8])
                      -> Option<EncoderError<'r>> {
        if input.len() > 0 {
            let str::CharRange {ch, next} = input.char_range_at(0);
            Some(CodecError {
                remaining: input.slice_from(next),
                problem: str::from_char(ch),
                cause: ~"unrepresentable character",
            })
        } else {
            None
        }
    }

    fn flush(&mut self, _output: &mut ~[u8]) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct ErrorDecoder;

impl Decoder for ErrorDecoder {
    fn encoding(&self) -> ~Encoding { ~ErrorEncoding as ~Encoding }

    fn feed<'r>(&mut self, input: &'r [u8], _output: &mut ~str)
                      -> Option<DecoderError<'r>> {
        if input.len() > 0 {
            Some(CodecError {
                remaining: input.slice(1, input.len()),
                problem: ~[input[0]],
                cause: ~"invalid sequence",
            })
        } else {
            None
        }
    }

    fn flush(&mut self, _output: &mut ~str) -> Option<DecoderError<'static>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorEncoding;
    use types::*;

    fn strip_cause<T,Remaining,Problem>(result: (T,Option<CodecError<Remaining,Problem>>))
                                    -> (T,Option<(Remaining,Problem)>) {
        match result {
            (processed, None) => (processed, None),
            (processed, Some(CodecError { remaining, problem, cause: _cause })) =>
                (processed, Some((remaining, problem)))
        }
    }

    macro_rules! assert_result(
        ($lhs:expr, $rhs:expr) => (assert_eq!(strip_cause($lhs), $rhs))
    )

    #[test]
    fn test_encoder() {
        let mut e = ErrorEncoding.encoder();
        assert_result!(e.test_feed("A"), (~[], Some(("", ~"A"))));
        assert_result!(e.test_feed("BC"), (~[], Some(("C", ~"B"))));
        assert_result!(e.test_feed(""), (~[], None));
        assert_result!(e.test_feed("\xa0"), (~[], Some(("", ~"\xa0"))));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_decoder() {
        let mut d = ErrorEncoding.decoder();
        assert_result!(d.test_feed(&[0x41]), (~"", Some((&[], ~[0x41]))));
        assert_result!(d.test_feed(&[0x42, 0x43]), (~"", Some((&[0x43], ~[0x42]))));
        assert_result!(d.test_feed(&[]), (~"", None));
        assert_result!(d.test_feed(&[0xa0]), (~"", Some((&[], ~[0xa0]))));
        assert_result!(d.test_flush(), (~"", None));
    }
}


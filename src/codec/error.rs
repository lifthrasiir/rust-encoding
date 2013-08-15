// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A placeholder encoding that returns encoder/decoder error for every case.

use std::str;
use types::*;

#[deriving(Clone)]
pub struct ErrorEncoding;

impl Encoding for ErrorEncoding {
    pub fn name(&self) -> ~str { ~"error" }
    pub fn encoder(&self) -> ~Encoder { ~ErrorEncoder as ~Encoder }
    pub fn decoder(&self) -> ~Decoder { ~ErrorDecoder as ~Decoder }
    pub fn preferred_replacement_seq(&self) -> ~[u8] { ~[0x3f] /* "?" */ }
}

#[deriving(Clone)]
pub struct ErrorEncoder;

impl Encoder for ErrorEncoder {
    pub fn encoding(&self) -> ~Encoding { ~ErrorEncoding as ~Encoding }

    pub fn feed_into<'r>(&mut self, input: &'r str, _output: &mut ~[u8])
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

    pub fn flush(&mut self) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct ErrorDecoder;

impl Decoder for ErrorDecoder {
    pub fn encoding(&self) -> ~Encoding { ~ErrorEncoding as ~Encoding }

    pub fn feed_into<'r>(&mut self, input: &'r [u8], _output: &mut ~str)
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

    pub fn flush(&mut self) -> Option<DecoderError<'static>> {
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
        assert_result!(e.feed("A"), (~[], Some(("", ~"A"))));
        assert_result!(e.feed("BC"), (~[], Some(("C", ~"B"))));
        assert_result!(e.feed(""), (~[], None));
        assert_result!(e.feed("\xa0"), (~[], Some(("", ~"\xa0"))));
        assert_result!(((), e.flush()), ((), None));
    }

    #[test]
    fn test_decoder() {
        let mut d = ErrorEncoding.decoder();
        assert_result!(d.feed(&[0x41]), (~"", Some((&[], ~[0x41]))));
        assert_result!(d.feed(&[0x42, 0x43]), (~"", Some((&[0x43], ~[0x42]))));
        assert_result!(d.feed(&[]), (~"", None));
        assert_result!(d.feed(&[0xa0]), (~"", Some((&[], ~[0xa0]))));
        assert_result!(((), d.flush()), ((), None));
    }
}


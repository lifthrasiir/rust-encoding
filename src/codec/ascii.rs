// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! 7-bit ASCII encoding.

use std::str;
use util::StrCharIndex;
use types::*;

#[deriving(Clone)]
pub struct ASCIIEncoding;

impl Encoding for ASCIIEncoding {
    pub fn name(&self) -> ~str { ~"ascii" }
    pub fn encoder(&self) -> ~Encoder { ~ASCIIEncoder as ~Encoder }
    pub fn decoder(&self) -> ~Decoder { ~ASCIIDecoder as ~Decoder }
    pub fn preferred_replacement_seq(&self) -> ~[u8] { ~[0x3f] /* "?" */ }
}

#[deriving(Clone)]
pub struct ASCIIEncoder;

impl Encoder for ASCIIEncoder {
    pub fn encoding(&self) -> ~Encoding { ~ASCIIEncoding as ~Encoding }

    pub fn feed_into<'r>(&mut self, input: &'r str, output: &mut ~[u8]) -> Option<EncoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut err = None;
        for input.index_iter().advance |((_,j), ch)| {
            if ch <= '\u007f' {
                output.push(ch as u8);
            } else {
                err = Some(CodecError {
                    remaining: input.slice_from(j),
                    problem: str::from_char(ch),
                    cause: ~"unrepresentable character",
                });
                break;
            }
        }
        err
    }

    pub fn flush(&mut self) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct ASCIIDecoder;

impl Decoder for ASCIIDecoder {
    pub fn encoding(&self) -> ~Encoding { ~ASCIIEncoding as ~Encoding }

    pub fn feed_into<'r>(&mut self, input: &'r [u8], output: &mut ~str) -> Option<DecoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.push_char(input[i] as char);
            } else {
                return Some(CodecError {
                    remaining: input.slice(i+1, len),
                    problem: ~[input[i]],
                    cause: ~"invalid sequence",
                });
            }
            i += 1;
        }
        None
    }

    pub fn flush(&mut self) -> Option<DecoderError<'static>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::ASCIIEncoding;
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
        let mut e = ASCIIEncoding.encoder();
        assert_result!(e.feed("A"), (~[0x41], None));
        assert_result!(e.feed("BC"), (~[0x42, 0x43], None));
        assert_result!(e.feed(""), (~[], None));
        assert_result!(e.feed("\xa0"), (~[], Some(("", ~"\xa0"))));
        assert_result!(((), e.flush()), ((), None));
    }

    #[test]
    fn test_decoder() {
        let mut d = ASCIIEncoding.decoder();
        assert_result!(d.feed(&[0x41]), (~"A", None));
        assert_result!(d.feed(&[0x42, 0x43]), (~"BC", None));
        assert_result!(d.feed(&[]), (~"", None));
        assert_result!(d.feed(&[0xa0]), (~"", Some((&[], ~[0xa0]))));
        assert_result!(((), d.flush()), ((), None));
    }
}


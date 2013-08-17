// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Common codec implementation for single-byte encodings.

use std::str;
use util::StrCharIndex;
use types::*;

pub struct SingleByteEncoding {
    name: &'static str,
    index_forward: extern "Rust" fn(u8) -> u16,
    index_backward: extern "Rust" fn(u16) -> u8,
}

impl Clone for SingleByteEncoding {
    fn clone(&self) -> SingleByteEncoding {
        SingleByteEncoding { name: self.name,
                             index_forward: self.index_forward,
                             index_backward: self.index_backward }
    }
}

impl Encoding for SingleByteEncoding {
    fn name(&self) -> ~str { self.name.to_owned() }
    fn encoder(&self) -> ~Encoder { ~SingleByteEncoder { encoding: self.clone() } as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~SingleByteDecoder { encoding: self.clone() } as ~Decoder }
    fn preferred_replacement_seq(&self) -> ~[u8] { ~[0x3f] /* "?" */ }
}

#[deriving(Clone)]
pub struct SingleByteEncoder {
    encoding: SingleByteEncoding,
}

impl Encoder for SingleByteEncoder {
    fn encoding(&self) -> ~Encoding { ~self.encoding.clone() as ~Encoding }

    fn feed<'r>(&mut self, input: &'r str, output: &mut ~[u8]) -> Option<EncoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut err = None;
        for input.index_iter().advance |((_,j), ch)| {
            if ch <= '\u007f' {
                output.push(ch as u8);
                loop
            }
            if ch <= '\uffff' {
                let index = (self.encoding.index_backward)(ch as u16);
                if index != 0xff {
                    output.push((index + 0x80) as u8);
                    loop
                }
            }
            err = Some(CodecError {
                remaining: input.slice_from(j),
                problem: str::from_char(ch),
                cause: ~"unrepresentable character",
            });
            break;
        }
        err
    }

    fn flush(&mut self, _output: &mut ~[u8]) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct SingleByteDecoder {
    encoding: SingleByteEncoding,
}

impl Decoder for SingleByteDecoder {
    fn encoding(&self) -> ~Encoding { ~self.encoding.clone() as ~Encoding }

    fn feed<'r>(&mut self, input: &'r [u8], output: &mut ~str) -> Option<DecoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.push_char(input[i] as char);
            } else {
                let ch = (self.encoding.index_forward)(input[i] - 0x80);
                if ch != 0xffff {
                    output.push_char(ch as char);
                } else {
                    return Some(CodecError {
                        remaining: input.slice(i+1, input.len()),
                        problem: ~[input[i]],
                        cause: ~"invalid sequence",
                    });
                }
            }
            i += 1;
        }
        None
    }

    fn flush(&mut self, _output: &mut ~str) -> Option<DecoderError<'static>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use all::ISO_8859_2;
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
    fn test_encoder_non_bmp() {
        let mut e = ISO_8859_2.encoder();
        assert_result!(e.test_feed("A\uFFFFB"), (~[0x41], Some(("B", ~"\uFFFF"))));
        assert_result!(e.test_feed("A\U00010000B"), (~[0x41], Some(("B", ~"\U00010000"))));
    }
}


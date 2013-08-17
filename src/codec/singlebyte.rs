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
    pub fn name(&self) -> ~str { self.name.to_owned() }
    pub fn encoder(&self) -> ~Encoder { ~SingleByteEncoder { encoding: self.clone() } as ~Encoder }
    pub fn decoder(&self) -> ~Decoder { ~SingleByteDecoder { encoding: self.clone() } as ~Decoder }
    pub fn preferred_replacement_seq(&self) -> ~[u8] { ~[0x3f] /* "?" */ }
}

#[deriving(Clone)]
pub struct SingleByteEncoder {
    encoding: SingleByteEncoding,
}

impl Encoder for SingleByteEncoder {
    pub fn encoding(&self) -> ~Encoding { ~self.encoding.clone() as ~Encoding }

    pub fn feed<'r>(&mut self, input: &'r str) -> (~[u8],Option<EncoderError<'r>>) {
        let mut ret = ~[];
        let mut err = None;
        for input.index_iter().advance |((_,j), ch)| {
            if ch <= '\u007f' {
                ret.push(ch as u8);
                loop
            }
            if ch <= '\uffff' {
                let index = (self.encoding.index_backward)(ch as u16);
                if index != 0xff {
                    ret.push((index + 0x80) as u8);
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
        (ret, err)
    }

    pub fn flush(~self) -> (~[u8],Option<EncoderError<'static>>) {
        (~[], None)
    }
}

#[deriving(Clone)]
pub struct SingleByteDecoder {
    encoding: SingleByteEncoding,
}

impl Decoder for SingleByteDecoder {
    pub fn encoding(&self) -> ~Encoding { ~self.encoding.clone() as ~Encoding }

    pub fn feed<'r>(&mut self, input: &'r [u8]) -> (~str,Option<DecoderError<'r>>) {
        let mut ret = ~"";
        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                ret.push_char(input[i] as char);
            } else {
                let ch = (self.encoding.index_forward)(input[i] - 0x80);
                if ch != 0xffff {
                    ret.push_char(ch as char);
                } else {
                    return (ret, Some(CodecError {
                        remaining: input.slice(i+1, input.len()),
                        problem: ~[input[i]],
                        cause: ~"invalid sequence",
                    }));
                }
            }
            i += 1;
        }
        (ret, None)
    }

    pub fn flush(~self) -> (~str,Option<DecoderError<'static>>) {
        (~"", None)
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
        assert_result!(e.feed("A\uFFFFB"), (~[0x41], Some(("B", ~"\uFFFF"))));
        assert_result!(e.feed("A\U00010000B"), (~[0x41], Some(("B", ~"\U00010000"))));
    }
}


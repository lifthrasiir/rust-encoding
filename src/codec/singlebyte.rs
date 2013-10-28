// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Common codec implementation for single-byte encodings.

use std::str;
use util::{as_char, StrCharIndex};
use types::*;

pub struct SingleByteEncoding {
    name: &'static str,
    index_forward: extern "Rust" fn(u8) -> u16,
    index_backward: extern "Rust" fn(u16) -> u8,
}

impl Encoding for SingleByteEncoding {
    fn name(&self) -> &'static str { self.name }
    fn encoder(&'static self) -> ~Encoder { ~SingleByteEncoder { encoding: self } as ~Encoder }
    fn decoder(&'static self) -> ~Decoder { ~SingleByteDecoder { encoding: self } as ~Decoder }
}

pub struct SingleByteEncoder {
    encoding: &'static SingleByteEncoding,
}

impl Encoder for SingleByteEncoder {
    fn encoding(&self) -> &'static Encoding { self.encoding as &'static Encoding }

    fn raw_feed<'r>(&mut self, input: &'r str,
                    output: &mut ByteWriter) -> Option<EncoderError<'r>> {
        output.writer_hint(input.len());

        let mut err = None;
        for ((_,j), ch) in input.index_iter() {
            if ch <= '\u007f' {
                output.write_byte(ch as u8);
                loop
            }
            if ch <= '\uffff' {
                let index = (self.encoding.index_backward)(ch as u16);
                if index != 0xff {
                    output.write_byte((index + 0x80) as u8);
                    loop
                }
            }
            err = Some(CodecError {
                remaining: input.slice_from(j),
                problem: str::from_char(ch),
                cause: "unrepresentable character".into_send_str(),
            });
            break;
        }
        err
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<EncoderError<'static>> {
        None
    }
}

pub struct SingleByteDecoder {
    encoding: &'static SingleByteEncoding,
}

impl Decoder for SingleByteDecoder {
    fn encoding(&self) -> &'static Encoding { self.encoding as &'static Encoding }

    fn raw_feed<'r>(&mut self, input: &'r [u8],
                    output: &mut StringWriter) -> Option<DecoderError<'r>> {
        output.writer_hint(input.len());

        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.write_char(input[i] as char);
            } else {
                let ch = (self.encoding.index_forward)(input[i] - 0x80);
                if ch != 0xffff {
                    output.write_char(as_char(ch));
                } else {
                    return Some(CodecError {
                        remaining: input.slice(i+1, input.len()),
                        problem: ~[input[i]],
                        cause: "invalid sequence".into_send_str(),
                    });
                }
            }
            i += 1;
        }
        None
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<DecoderError<'static>> {
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


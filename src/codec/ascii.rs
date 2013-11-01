// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! 7-bit ASCII encoding.

use util::StrCharIndex;
use types::*;

#[deriving(Clone)]
pub struct ASCIIEncoding;

impl Encoding for ASCIIEncoding {
    fn name(&self) -> &'static str { "ascii" }
    fn encoder(&self) -> ~Encoder { ~ASCIIEncoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~ASCIIDecoder as ~Decoder }
}

#[deriving(Clone)]
pub struct ASCIIEncoder;

impl Encoder for ASCIIEncoder {
    fn encoding(&self) -> &'static Encoding { &ASCIIEncoding as &'static Encoding }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            if ch <= '\u007f' {
                output.write_byte(ch as u8);
            } else {
                return (i, Some(CodecError {
                    upto: j, cause: "unrepresentable character".into_send_str()
                }));
            }
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

#[deriving(Clone)]
pub struct ASCIIDecoder;

impl Decoder for ASCIIDecoder {
    fn encoding(&self) -> &'static Encoding { &ASCIIEncoding as &'static Encoding }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());
                                        
        let mut i = 0;
        let len = input.len();
        while i < len {
            if input[i] <= 0x7f {
                output.write_char(input[i] as char);
            } else {
                return (i, Some(CodecError {
                    upto: i+1, cause: "invalid sequence".into_send_str()
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


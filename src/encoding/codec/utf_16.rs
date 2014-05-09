// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! UTF-16.

use util::{as_char, StrCharIndex};
use types::*;

/**
 * UTF-16 (UCS Transformation Format, 16-bit) in little endian.
 *
 * This is a Unicode encoding where one codepoint may use
 * 2 (up to U+FFFF) or 4 bytes (up to U+10FFFF) depending on its value.
 * It uses a "surrogate" mechanism to encode non-BMP codepoints,
 * which are represented as a pair of lower surrogate and upper surrogate characters.
 * In this effect, surrogate characters (U+D800..DFFF) cannot appear alone
 * and cannot be included in a valid Unicode string.
 */
#[deriving(Clone)]
pub struct UTF16LEEncoding;

impl Encoding for UTF16LEEncoding {
    fn name(&self) -> &'static str { "utf-16le" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("utf-16") } // WHATWG compatibility
    fn encoder(&self) -> Box<Encoder> { UTF16LEEncoder::new() }
    fn decoder(&self) -> Box<Decoder> { UTF16LEDecoder::new() }
}

/**
 * UTF-16 (UCS Transformation Format, 16-bit) in big endian.
 */
#[deriving(Clone)]
pub struct UTF16BEEncoding;

impl Encoding for UTF16BEEncoding {
    fn name(&self) -> &'static str { "utf-16be" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("utf-16be") }
    fn encoder(&self) -> Box<Encoder> { UTF16BEEncoder::new() }
    fn decoder(&self) -> Box<Decoder> { UTF16BEDecoder::new() }
}

/// An encoder for UTF-16 in little endian.
#[deriving(Clone)]
pub struct UTF16LEEncoder;

/// An encoder for UTF-16 in big endian.
#[deriving(Clone)]
pub struct UTF16BEEncoder;

impl UTF16LEEncoder {
    pub fn new() -> Box<Encoder> { box UTF16LEEncoder as Box<Encoder> }
}

impl UTF16BEEncoder {
    pub fn new() -> Box<Encoder> { box UTF16BEEncoder as Box<Encoder> }
}

macro_rules! impl_UTF16Encoder(
    ($encoder:ident: fn write_two_bytes($output:ident: &mut ByteWriter,
                                        $msb:ident: u8, $lsb:ident: u8) $body:block) =>
    (impl Encoder for $encoder {
        fn from_self(&self) -> Box<Encoder> { $encoder::new() }

        fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
            output.writer_hint(input.len() * 2);

            fn write_two_bytes($output: &mut ByteWriter, $msb: u8, $lsb: u8) $body

            for ((i,j), ch) in input.index_iter() {
                let ch = ch as uint;
                match ch {
                    0x0000..0xd7ff | 0xe000..0xffff => {
                        write_two_bytes(output, (ch >> 8) as u8, (ch & 0xff) as u8);
                    }
                    0x10000..0x10ffff => {
                        let ch = ch - 0x10000;
                        write_two_bytes(output, (0xd8 | (ch >> 18)) as u8,
                                                ((ch >> 10) & 0xff) as u8);
                        write_two_bytes(output, (0xdc | ((ch >> 8) & 0x3)) as u8,
                                                (ch & 0xff) as u8);
                    }
                    _ => {
                        return (i, Some(CodecError {
                            upto: j, cause: "unrepresentable character".into_maybe_owned()
                        }));
                    }
                }
            }
            (input.len(), None)
        }

        fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
            None
        }
    })
)

impl_UTF16Encoder!(UTF16LEEncoder:
    fn write_two_bytes(output: &mut ByteWriter, msb: u8, lsb: u8) {
        output.write_byte(lsb);
        output.write_byte(msb);
    }
)

impl_UTF16Encoder!(UTF16BEEncoder:
    fn write_two_bytes(output: &mut ByteWriter, msb: u8, lsb: u8) {
        output.write_byte(msb);
        output.write_byte(lsb);
    }
)

/// A decoder for UTF-16 in little endian.
pub struct UTF16LEDecoder {
    leadbyte: u16,
    leadsurrogate: u16,
}

/// A decoder for UTF-16 in big endian.
pub struct UTF16BEDecoder {
    leadbyte: u16,
    leadsurrogate: u16,
}

impl UTF16LEDecoder {
    pub fn new() -> Box<Decoder> {
        box UTF16LEDecoder { leadbyte: 0xffff, leadsurrogate: 0xffff } as Box<Decoder>
    }
}

impl UTF16BEDecoder {
    pub fn new() -> Box<Decoder> {
        box UTF16BEDecoder { leadbyte: 0xffff, leadsurrogate: 0xffff } as Box<Decoder>
    }
}

macro_rules! impl_UTF16Decoder(
    ($decoder:ident: fn concat_two_bytes($lead:ident: u16, $trail:ident: u8) -> u16 $body:block) =>
    (impl Decoder for $decoder {
        fn from_self(&self) -> Box<Decoder> { $decoder::new() }

        fn raw_feed(&mut self, input: &[u8],
                    output: &mut StringWriter) -> (uint, Option<CodecError>) {
            output.writer_hint(input.len() / 2); // when every codepoint is U+0000..007F

            fn concat_two_bytes($lead: u16, $trail: u8) -> u16 $body

            let mut i = 0;
            let mut processed = 0;
            let len = input.len();

            if i >= len { return (processed, None); }

            if self.leadbyte != 0xffff {
                let ch = concat_two_bytes(self.leadbyte, input[i]);
                i += 1;
                self.leadbyte = 0xffff;
                if self.leadsurrogate != 0xffff { // `ch` is lower surrogate
                    let upper = self.leadsurrogate;
                    self.leadsurrogate = 0xffff;
                    match ch {
                        0xdc00..0xdfff => {
                            let ch = ((upper as uint - 0xd800) << 10) + (ch as uint - 0xdc00);
                            output.write_char(as_char(ch + 0x10000));
                            processed = i;
                        }
                        _ => {
                            return (processed, Some(CodecError {
                                // XXX upto should point to the negative offset???
                                upto: if i<2 {0} else {i-2},
                                cause: "invalid sequence".into_maybe_owned()
                            }));
                        }
                    }
                } else {
                    match ch {
                        0xd800..0xdbff => {
                            self.leadsurrogate = ch;
                            // pass through
                        }
                        0xdc00..0xdfff => {
                            return (processed, Some(CodecError {
                                upto: i, cause: "invalid sequence".into_maybe_owned()
                            }));
                        }
                        _ => {
                            output.write_char(as_char(ch));
                            processed = i;
                        }
                    }
                }
                if i >= len { return (processed, None); }
            }

            if self.leadsurrogate != 0xffff {
                i += 1;
                if i >= len {
                    self.leadbyte = input[i-1] as u16;
                    return (processed, None);
                }
                let upper = self.leadsurrogate;
                let ch = concat_two_bytes(input[i-1] as u16, input[i]);
                i += 1;
                match ch {
                    0xdc00..0xdfff => {
                        let ch = ((upper as uint - 0xd800) << 10) + (ch as uint - 0xdc00);
                        output.write_char(as_char(ch + 0x10000));
                    }
                    _ => {
                        self.leadbyte = 0xffff;
                        self.leadsurrogate = 0xffff;
                        return (processed, Some(CodecError {
                            // XXX upto should point to the negative offset???
                            upto: if i<2 {0} else {i-2},
                            cause: "invalid sequence".into_maybe_owned()
                        }));
                    }
                }
            }

            self.leadbyte = 0xffff;
            self.leadsurrogate = 0xffff;
            processed = i;
            while i < len {
                i += 1;
                if i >= len {
                    self.leadbyte = input[i-1] as u16;
                    break;
                }
                let ch = concat_two_bytes(input[i-1] as u16, input[i]);
                match ch {
                    0xd800..0xdbff => {
                        i += 2;
                        if i >= len {
                            self.leadsurrogate = ch;
                            if i-1 < len { self.leadbyte = input[i-1] as u16; }
                            break;
                        }
                        let ch2 = concat_two_bytes(input[i-1] as u16, input[i]);
                        match ch2 {
                            0xdc00..0xdfff => {
                                let ch = ((ch as uint - 0xd800) << 10) + (ch2 as uint - 0xdc00);
                                output.write_char(as_char(ch + 0x10000));
                            }
                            _ => {
                                return (processed, Some(CodecError {
                                    upto: i-1, cause: "invalid sequence".into_maybe_owned()
                                }));
                            }
                        }
                    }
                    0xdc00..0xdfff => {
                        return (processed, Some(CodecError {
                            upto: i+1, cause: "invalid sequence".into_maybe_owned()
                        }));
                    }
                    _ => {
                        output.write_char(as_char(ch));
                    }
                }
                i += 1;
                processed = i;
            }
            (processed, None)
        }

        fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
            let leadbyte = self.leadbyte;
            let leadsurrogate = self.leadsurrogate;
            self.leadbyte = 0xffff;
            self.leadsurrogate = 0xffff;
            if leadbyte != 0xffff || leadsurrogate != 0xffff {
                Some(CodecError { upto: 0, cause: "incomplete sequence".into_maybe_owned() })
            } else {
                None
            }
        }
    })
)

impl_UTF16Decoder!(UTF16LEDecoder:
    fn concat_two_bytes(lead: u16, trail: u8) -> u16 { lead | (trail as u16 << 8) }
)

impl_UTF16Decoder!(UTF16BEDecoder:
    fn concat_two_bytes(lead: u16, trail: u8) -> u16 { (lead << 8) | trail as u16 }
)

#[cfg(test)]
mod tests {
    // little endian and big endian is symmetric to each other, there's no need to test both.
    // since big endian is easier to inspect we test UTF16BEEncoding only.

    use super::UTF16BEEncoding;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = UTF16BEEncoding.encoder();
        assert_feed_ok!(e, "\u0000\
                            \u0001\u0002\u0004\u0008\
                            \u0010\u0020\u0040\u0080\
                            \u0100\u0200\u0400\u0800\
                            \u1000\u2000\u4000\u8000\
                            \uffff", "",
                        [0x00, 0x00,
                         0x00, 0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x08,
                         0x00, 0x10, 0x00, 0x20, 0x00, 0x40, 0x00, 0x80,
                         0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x08, 0x00,
                         0x10, 0x00, 0x20, 0x00, 0x40, 0x00, 0x80, 0x00,
                         0xff, 0xff]);
        assert_feed_ok!(e, "\U00010000\
                            \U00010001\U00010002\
                            \U00010004\U00010008\
                            \U00010010\U00010020\
                            \U00010040\U00010080\
                            \U00010100\U00010200\
                            \U00010400\U00010800\
                            \U00011000\U00012000\
                            \U00014000\U00018000\
                            \U00020000\U00030000\
                            \U00050000\U00090000\
                            \U0010FFFF", "",
                        [0xd8, 0x00, 0xdc, 0x00,
                         0xd8, 0x00, 0xdc, 0x01, 0xd8, 0x00, 0xdc, 0x02,
                         0xd8, 0x00, 0xdc, 0x04, 0xd8, 0x00, 0xdc, 0x08,
                         0xd8, 0x00, 0xdc, 0x10, 0xd8, 0x00, 0xdc, 0x20,
                         0xd8, 0x00, 0xdc, 0x40, 0xd8, 0x00, 0xdc, 0x80,
                         0xd8, 0x00, 0xdd, 0x00, 0xd8, 0x00, 0xde, 0x00,
                         0xd8, 0x01, 0xdc, 0x00, 0xd8, 0x02, 0xdc, 0x00,
                         0xd8, 0x04, 0xdc, 0x00, 0xd8, 0x08, 0xdc, 0x00,
                         0xd8, 0x10, 0xdc, 0x00, 0xd8, 0x20, 0xdc, 0x00,
                         0xd8, 0x40, 0xdc, 0x00, 0xd8, 0x80, 0xdc, 0x00,
                         0xd9, 0x00, 0xdc, 0x00, 0xda, 0x00, 0xdc, 0x00,
                         0xdb, 0xff, 0xdf, 0xff]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0x00, 0x00,
                            0x00, 0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x08,
                            0x00, 0x10, 0x00, 0x20, 0x00, 0x40, 0x00, 0x80,
                            0x01, 0x00, 0x02, 0x00, 0x04, 0x00, 0x08, 0x00,
                            0x10, 0x00, 0x20, 0x00, 0x40, 0x00, 0x80, 0x00,
                            0xff, 0xff], [],
                        "\u0000\
                         \u0001\u0002\u0004\u0008\
                         \u0010\u0020\u0040\u0080\
                         \u0100\u0200\u0400\u0800\
                         \u1000\u2000\u4000\u8000\
                         \uffff");
        assert_feed_ok!(d, [0xd8, 0x00, 0xdc, 0x00,
                            0xd8, 0x00, 0xdc, 0x01, 0xd8, 0x00, 0xdc, 0x02,
                            0xd8, 0x00, 0xdc, 0x04, 0xd8, 0x00, 0xdc, 0x08,
                            0xd8, 0x00, 0xdc, 0x10, 0xd8, 0x00, 0xdc, 0x20,
                            0xd8, 0x00, 0xdc, 0x40, 0xd8, 0x00, 0xdc, 0x80,
                            0xd8, 0x00, 0xdd, 0x00, 0xd8, 0x00, 0xde, 0x00,
                            0xd8, 0x01, 0xdc, 0x00, 0xd8, 0x02, 0xdc, 0x00,
                            0xd8, 0x04, 0xdc, 0x00, 0xd8, 0x08, 0xdc, 0x00,
                            0xd8, 0x10, 0xdc, 0x00, 0xd8, 0x20, 0xdc, 0x00,
                            0xd8, 0x40, 0xdc, 0x00, 0xd8, 0x80, 0xdc, 0x00,
                            0xd9, 0x00, 0xdc, 0x00, 0xda, 0x00, 0xdc, 0x00,
                            0xdb, 0xff, 0xdf, 0xff], [],
                        "\U00010000\
                         \U00010001\U00010002\
                         \U00010004\U00010008\
                         \U00010010\U00010020\
                         \U00010040\U00010080\
                         \U00010100\U00010200\
                         \U00010400\U00010800\
                         \U00011000\U00012000\
                         \U00014000\U00018000\
                         \U00020000\U00030000\
                         \U00050000\U00090000\
                         \U0010FFFF");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_valid_partial_bmp() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0x12], "");
        assert_feed_ok!(d, [0x34], [], "\u1234");
        assert_feed_ok!(d, [], [0x56], "");
        assert_feed_ok!(d, [0x78], [], "\u5678");
        assert_finish_ok!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0x12], "");
        assert_feed_ok!(d, [0x34], [0x56], "\u1234");
        assert_feed_ok!(d, [0x78, 0xab, 0xcd], [], "\u5678\uabcd");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_valid_partial_non_bmp() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8], "");
        assert_feed_ok!(d, [], [0x08], "");
        assert_feed_ok!(d, [], [0xdf], "");
        assert_feed_ok!(d, [0x45], [0xd9], "\U00012345");
        assert_feed_ok!(d, [], [0x5e], "");
        assert_feed_ok!(d, [], [0xdc], "");
        assert_feed_ok!(d, [0x90], [], "\U00067890");
        assert_finish_ok!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8], "");
        assert_feed_ok!(d, [], [0x08, 0xdf], "");
        assert_feed_ok!(d, [0x45], [0xd9, 0x5e], "\U00012345");
        assert_feed_ok!(d, [0xdc, 0x90], [], "\U00067890");
        assert_finish_ok!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8, 0x08, 0xdf], "");
        assert_feed_ok!(d, [0x45], [0xd9, 0x5e, 0xdc], "\U00012345");
        assert_feed_ok!(d, [0x90], [], "\U00067890");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_invalid_lone_upper_surrogate() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8, 0x00], "");
        assert_feed_err!(d, [], [], [0x12, 0x34], "");
        assert_feed_err!(d, [], [0xd8, 0x00], [0x56, 0x78], "");
        assert_feed_ok!(d, [], [0xd8, 0x00], "");
        assert_feed_err!(d, [], [], [0xd8, 0x00], "");
        assert_feed_ok!(d, [], [0xd8, 0x00], "");
        assert_finish_err!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xdb, 0xff], "");
        assert_feed_err!(d, [], [], [0x12, 0x34], "");
        assert_feed_err!(d, [], [0xdb, 0xff], [0x56, 0x78], "");
        assert_feed_ok!(d, [], [0xdb, 0xff], "");
        assert_feed_err!(d, [], [], [0xdb, 0xff], "");
        assert_feed_ok!(d, [], [0xdb, 0xff], "");
        assert_finish_err!(d, "");
    }

    #[test]
    fn test_decoder_invalid_lone_upper_surrogate_partial() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8], "");
        assert_feed_err!(d, [], [0x00], [0x12, 0x34], "");
        assert_feed_ok!(d, [], [0xd8, 0x00, 0x56], "");
        assert_feed_err!(d, [], [], [0x78], ""); // XXX whooops, 56 should not be in problem!
        assert_feed_ok!(d, [], [0xd8], "");
        assert_feed_err!(d, [], [0x00], [0xd8, 0x00], "");
        assert_feed_ok!(d, [], [0xd8, 0x00, 0xdb], "");
        assert_feed_err!(d, [], [], [0xff], ""); // XXX whooops, DB should not be in problem!
        assert_feed_ok!(d, [], [0xd8], "");
        assert_finish_err!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xdb], "");
        assert_feed_err!(d, [], [0xff], [0x12, 0x34], "");
        assert_feed_ok!(d, [], [0xdb, 0xff, 0x56], "");
        assert_feed_err!(d, [], [], [0x78], ""); // XXX whooops, 56 should not be in problem!
        assert_feed_ok!(d, [], [0xdb], "");
        assert_feed_err!(d, [], [0xff], [0xdb, 0xff], "");
        assert_feed_ok!(d, [], [0xdb, 0xff, 0xd8], "");
        assert_feed_err!(d, [], [], [0x00], ""); // XXX whooops, D8 should not be in problem!
        assert_feed_ok!(d, [], [0xdb], "");
        assert_finish_err!(d, "");
    }

    #[test]
    fn test_decoder_invalid_lone_lower_surrogate() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_err!(d, [], [0xdc, 0x00], [], "");
        assert_feed_err!(d, [0x12, 0x34], [0xdc, 0x00], [0x56, 0x78], "\u1234");
        assert_finish_ok!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_err!(d, [], [0xdf, 0xff], [], "");
        assert_feed_err!(d, [0x12, 0x34], [0xdf, 0xff], [0x56, 0x78], "\u1234");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_invalid_lone_lower_surrogate_partial() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xdc], "");
        assert_feed_err!(d, [], [0x00], [], "");
        assert_feed_ok!(d, [0x12, 0x34], [0xdc], "\u1234");
        assert_feed_err!(d, [], [0x00], [0x56, 0x78], "");
        assert_finish_ok!(d, "");

        assert_feed_ok!(d, [], [0xdf], "");
        assert_feed_err!(d, [], [0xff], [], "");
        assert_feed_ok!(d, [0x12, 0x34], [0xdf], "\u1234");
        assert_feed_err!(d, [], [0xff], [0x56, 0x78], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_invalid_one_byte_before_finish() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0x12], "");
        assert_finish_err!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0x12, 0x34], [0x56], "\u1234");
        assert_finish_err!(d, "");
    }

    #[test]
    fn test_decoder_invalid_three_bytes_before_finish() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8, 0x00, 0xdc], "");
        assert_finish_err!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0x12, 0x34], [0xd8, 0x00, 0xdc], "\u1234");
        assert_finish_err!(d, "");
    }

    #[test]
    fn test_decoder_invalid_three_bytes_before_finish_partial() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [], [0xd8], "");
        assert_feed_ok!(d, [], [0x00], "");
        assert_feed_ok!(d, [], [0xdc], "");
        assert_finish_err!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0x12, 0x34], [0xd8], "\u1234");
        assert_feed_ok!(d, [], [0x00, 0xdc], "");
        assert_finish_err!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0x12, 0x34], [0xd8, 0x00], "\u1234");
        assert_feed_ok!(d, [], [0xdc], "");
        assert_finish_err!(d, "");
    }

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0x12, 0x34], [0x12], "\u1234");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0x12, 0x34], [], "\u1234");
        assert_finish_ok!(d, "");

        let mut d = UTF16BEEncoding.decoder();
        assert_feed_ok!(d, [0xd8, 0x08, 0xdf, 0x45], [0xd8, 0x08, 0xdf], "\U00012345");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xd8, 0x08, 0xdf, 0x45], [0xd8, 0x08], "\U00012345");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xd8, 0x08, 0xdf, 0x45], [0xd8], "\U00012345");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xd8, 0x08, 0xdf, 0x45], [], "\U00012345");
        assert_finish_ok!(d, "");
    }
}


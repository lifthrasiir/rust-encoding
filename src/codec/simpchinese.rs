// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy simplified Chinese encodings based on GB 2312 and GB 18030.

use util::{as_char, StrCharIndex};
use index2312 = index::gbk;
//use index18030 = index::gb18030;
use types::*;

/**
 * A one- and two-byte subset of GB 18030 that supersedes and updates GBK 1.0 encoding.
 *
 * This is a simplified Chinese encoding derived from GB 2312 character set
 * and greatly expanded to span the almost entire region of `[81-FE] [40-7E 80-FE]`.
 * There are several different revisions of a family of encodings named "GBK":
 *
 * - GBK as specified in the normative annex of GB 13000.1-93,
 *   the domestic standard equivalent to Unicode 1.1,
 *   consisted of characters included in Unicode 1.1 and not in GB 2312-80.
 * - Windows code page 936 is the widespread extension to GBK.
 * - Due to the popularity of Windows code page 936,
 *   a formal encoding based on Windows code page 936 (while adding new characters)
 *   was standardized into GBK 1.0.
 * - Finally, GB 18030 added four-byte sequences to GBK for becoming a pan-Unicode encoding,
 *   while adding new characters to the (former) GBK region again.
 *
 * Fortunately for us, these revisions maintain the strict superset and subset relation,
 * so this encoding is a catch-all implementation for all those related encodings.
 */
#[deriving(Clone)]
pub struct GBK18030Encoding;

impl Encoding for GBK18030Encoding {
    fn name(&self) -> &'static str { "gbk18030" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("gbk") } // WHATWG compatibility
    fn encoder(&self) -> ~Encoder { GBK18030Encoder::new() }
    fn decoder(&self) -> ~Decoder { GBK18030Decoder::new() }
}

/// An encoder for a one- and two-byte subset of GB 18030.
#[deriving(Clone)]
pub struct GBK18030Encoder;

impl GBK18030Encoder {
    pub fn new() -> ~Encoder { ~GBK18030Encoder as ~Encoder }
}

impl Encoder for GBK18030Encoder {
    fn from_self(&self) -> ~Encoder { GBK18030Encoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            if ch < '\u0080' {
                output.write_byte(ch as u8);
            } else {
                let ptr = index2312::backward(ch as u32);
                if ptr == 0xffff {
                    return (i, Some(CodecError {
                        upto: j, cause: "unrepresentable character".into_send_str()
                    }));
                }
                let lead = ptr / 190 + 0x81;
                let trail = ptr % 190;
                let trailoffset = if trail < 0x3f {0x40} else {0x41};
                output.write_byte(lead as u8);
                output.write_byte((trail + trailoffset) as u8);
            }
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

/// A decoder for a one- and two-byte subset of GB 18030.
#[deriving(Clone)]
pub struct GBK18030Decoder {
    first: u8
}

impl GBK18030Decoder {
    pub fn new() -> ~Decoder { ~GBK18030Decoder { first: 0 } as ~Decoder }
}

impl Decoder for GBK18030Decoder {
    fn from_self(&self) -> ~Decoder { GBK18030Decoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        fn map_two_2312_bytes(lead: u8, trail: u8) -> u32 {
            let lead = lead as uint;
            let trail = trail as uint;
            let index = match (lead, trail) {
                (0x81..0xfe, 0x40..0x7e) | (0x81..0xfe, 0x80..0xfe) => {
                    let trailoffset = if trail < 0x7f {0x40} else {0x41};
                    (lead - 0x81) * 190 + trail - trailoffset
                }
                _ => 0xffff,
            };
            index2312::forward(index as u16)
        }

        let mut i = 0;
        let mut processed = 0;
        let len = input.len();

        if i < len && self.first != 0 {
            let ch = map_two_2312_bytes(self.first, input[i]);
            if ch == 0xffff {
                self.first = 0;
                return (processed, Some(CodecError {
                    upto: i, cause: "invalid sequence".into_send_str()
                }));
            }
            output.write_char(as_char(ch));
            i += 1;
        }

        self.first = 0;
        processed = i;
        while i < len {
            match input[i] {
                0x00..0x7f => { output.write_char(input[i] as char); }
                0x80 => { output.write_char('\u20ac'); }
                0x81..0xfe => {
                    i += 1;
                    if i >= len {
                        self.first = input[i-1];
                        break;
                    }
                    let ch = map_two_2312_bytes(input[i-1], input[i]);
                    if ch == 0xffff {
                        return (processed, Some(CodecError {
                            upto: i, cause: "invalid sequence".into_send_str()
                        }));
                    }
                    output.write_char(as_char(ch));
                }
                _ => {
                    return (processed, Some(CodecError {
                        upto: i+1, cause: "invalid sequence".into_send_str()
                    }));
                }
            }
            i += 1;
            processed = i;
        }
        (processed, None)
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        let first = self.first;
        self.first = 0;
        if first != 0 {
            Some(CodecError { upto: 0, cause: "incomplete sequence".into_send_str() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod gbk18030_tests {
    use super::GBK18030Encoding;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = GBK18030Encoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_ok!(e, "\u4e2d\u534e\u4eba\u6c11\u5171\u548c\u56fd", "",
                        [0xd6, 0xd0, 0xbb, 0xaa, 0xc8, 0xcb, 0xc3, 0xf1,
                         0xb9, 0xb2, 0xba, 0xcd, 0xb9, 0xfa]);
        assert_feed_ok!(e, "1\u20ac/m", "", [0x31, 0xa2, 0xe3, 0x2f, 0x6d]);
        assert_feed_ok!(e, "\uff21\uff22\uff23", "", [0xa3, 0xc1, 0xa3, 0xc2, 0xa3, 0xc3]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = GBK18030Encoding.encoder();
        assert_feed_err!(e, "", "\uffff", "", []);
        assert_feed_err!(e, "?", "\uffff", "!", [0x3f]);
        assert_feed_err!(e, "", "\U0002a6a5", "\u3007", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = GBK18030Encoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0xd6, 0xd0, 0xbb, 0xaa, 0xc8, 0xcb, 0xc3, 0xf1,
                            0xb9, 0xb2, 0xba, 0xcd, 0xb9, 0xfa], [],
                        "\u4e2d\u534e\u4eba\u6c11\u5171\u548c\u56fd");
        assert_feed_ok!(d, [0x31, 0x80, 0x2f, 0x6d], [], "1\u20ac/m");
        assert_feed_ok!(d, [0xa3, 0xc1, 0xa3, 0xc2, 0xa3, 0xc3], [], "\uff21\uff22\uff23");
        assert_finish_ok!(d, "");
    }

    // TODO more tests

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = GBK18030Encoding.decoder();
        assert_feed_ok!(d, [0xd2, 0xbb], [0xd2], "\u4e00");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xd2, 0xbb], [], "\u4e00");
        assert_finish_ok!(d, "");
    }
}


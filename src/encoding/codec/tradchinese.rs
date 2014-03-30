// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy traditional Chinese encodings.

use util::{as_char, StrCharIndex};
use index = index::big5;
use types::*;

/**
 * Big5-2003 with common extensions. (XXX with asymmetric HKSCS-2008 support)
 *
 * This is a traditional Chinese encoding spanning the region `[81-FE] [40-7E A1-FE]`.
 * Originally a proprietary encoding by the consortium of five companies (hence the name),
 * the Republic of China government standardized Big5-2003 in an appendix of CNS 11643
 * so that CNS 11643 plane 1 and plane 2 have
 * an almost identical set of characters as Big5 (but with a different mapping).
 * The Hong Kong government has an official extension to Big5
 * named Hong Kong Supplementary Character Set (HKSCS).
 *
 * This particular implementation of Big5 includes the widespread ETEN and HKSCS extensions,
 * but excludes less common extensions such as Big5+, Big-5E and Unicode-at-on.
 */
#[deriving(Clone)]
pub struct BigFive2003Encoding;

impl Encoding for BigFive2003Encoding {
    fn name(&self) -> &'static str { "big5-2003" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("big5") } // WHATWG compatibility
    fn encoder(&self) -> ~Encoder { BigFive2003Encoder::new() }
    fn decoder(&self) -> ~Decoder { BigFive2003HKSCS2008Decoder::new() }
}

/// An encoder for Big5-2003.
#[deriving(Clone)]
pub struct BigFive2003Encoder;

impl BigFive2003Encoder {
    pub fn new() -> ~Encoder { ~BigFive2003Encoder as ~Encoder }
}

impl Encoder for BigFive2003Encoder {
    fn from_self(&self) -> ~Encoder { BigFive2003Encoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            if ch < '\u0080' {
                output.write_byte(ch as u8);
            } else {
                let ptr = index::backward(ch as u32);
                if ptr == 0xffff || ptr < (0xa1 - 0x81) * 157 {
                    // no HKSCS extension (XXX doesn't HKSCS include 0xFA40..0xFEFE?)
                    return (i, Some(CodecError {
                        upto: j, cause: "unrepresentable character".into_maybe_owned()
                    }));
                }
                let lead = ptr / 157 + 0x81;
                let trail = ptr % 157;
                let trailoffset = if trail < 0x3f {0x40} else {0x62};
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

/// A decoder for Big5-2003 with HKSCS-2008 extension.
#[deriving(Clone)]
pub struct BigFive2003HKSCS2008Decoder {
    lead: u8
}

impl BigFive2003HKSCS2008Decoder {
    pub fn new() -> ~Decoder { ~BigFive2003HKSCS2008Decoder { lead: 0 } as ~Decoder }
}

impl Decoder for BigFive2003HKSCS2008Decoder {
    fn from_self(&self) -> ~Decoder { BigFive2003HKSCS2008Decoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        fn map_two_bytes(lead: u8, trail: u8) -> u32 {
            let lead = lead as uint;
            let trail = trail as uint;
            let index = match (lead, trail) {
                (0x81..0xfe, 0x40..0x7e) | (0x81..0xfe, 0xa1..0xfe) => {
                    let trailoffset = if trail < 0x7f {0x40} else {0x62};
                    (lead - 0x81) * 157 + trail - trailoffset
                }
                _ => 0xffff,
            };
            index::forward(index as u16) // may return two-letter replacements 0..3
        }

        let mut i = 0;
        let mut processed = 0;
        let len = input.len();

        if i >= len { return (processed, None); }

        if self.lead != 0 {
            match map_two_bytes(self.lead, input[i]) {
                0xffff => {
                    self.lead = 0;
                    let upto = if input[i] < 0x80 {i} else {i+1};
                    return (processed, Some(CodecError {
                        upto: upto, cause: "invalid sequence".into_maybe_owned()
                    }));
                }
                0 /*index=1133*/ => { output.write_str("\u00ca\u0304"); }
                1 /*index=1135*/ => { output.write_str("\u00ca\u030c"); }
                2 /*index=1164*/ => { output.write_str("\u00ea\u0304"); }
                3 /*index=1166*/ => { output.write_str("\u00ea\u030c"); }
                ch => { output.write_char(as_char(ch)); }
            }
            i += 1;
        }

        self.lead = 0;
        processed = i;
        while i < len {
            match input[i] {
                0x00..0x7f => { output.write_char(input[i] as char); }
                0x81..0xfe => {
                    i += 1;
                    if i >= len {
                        self.lead = input[i-1];
                        break;
                    }
                    match map_two_bytes(input[i-1], input[i]) {
                        0xffff => {
                            let upto = if input[i] < 0x80 {i} else {i+1};
                            return (processed, Some(CodecError {
                                upto: upto, cause: "invalid sequence".into_maybe_owned()
                            }));
                        }
                        0 /*index=1133*/ => { output.write_str("\u00ca\u0304"); }
                        1 /*index=1135*/ => { output.write_str("\u00ca\u030c"); }
                        2 /*index=1164*/ => { output.write_str("\u00ea\u0304"); }
                        3 /*index=1166*/ => { output.write_str("\u00ea\u030c"); }
                        ch => { output.write_char(as_char(ch)); }
                    }
                }
                _ => {
                    return (processed, Some(CodecError {
                        upto: i+1, cause: "invalid sequence".into_maybe_owned()
                    }));
                }
            }
            i += 1;
            processed = i;
        }
        (processed, None)
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        let lead = self.lead;
        self.lead = 0;
        if lead != 0 {
            Some(CodecError { upto: 0, cause: "incomplete sequence".into_maybe_owned() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod bigfive2003_tests {
    use super::BigFive2003Encoding;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = BigFive2003Encoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_ok!(e, "\u4e2d\u83ef\u6c11\u570b", "",
                        [0xa4, 0xa4, 0xb5, 0xd8, 0xa5, 0xc1, 0xb0, 0xea]);
        assert_feed_ok!(e, "1\u20ac/m", "", [0x31, 0xa3, 0xe1, 0x2f, 0x6d]);
        assert_feed_ok!(e, "\uffed", "", [0xf9, 0xfe]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = BigFive2003Encoding.encoder();
        assert_feed_err!(e, "", "\uffff", "", []);
        assert_feed_err!(e, "?", "\uffff", "!", [0x3f]);
        assert_feed_err!(e, "", "\u3eec", "\u4e00", []); // HKSCS-2008 addition
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = BigFive2003Encoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0xa4, 0xa4, 0xb5, 0xd8, 0xa5, 0xc1, 0xb0, 0xea], [],
                        "\u4e2d\u83ef\u6c11\u570b");
        assert_feed_ok!(d, [0x31, 0xa3, 0xe1, 0x2f, 0x6d], [], "1\u20ac/m");
        assert_feed_ok!(d, [0xf9, 0xfe], [], "\uffed");
        assert_feed_ok!(d, [0x87, 0x7e], [], "\u3eec"); // HKSCS-2008 addition
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_invalid_trail() {
        // unlike most other cases, valid lead + invalid MSB-set trail are entirely consumed.
        // https://www.w3.org/Bugs/Public/show_bug.cgi?id=16771
        let mut d = BigFive2003Encoding.decoder();
        assert_feed_err!(d, [], [0xf0], [0x30, 0x30], "");
        assert_feed_err!(d, [], [0xf0, 0x80], [0x30], "");
        assert_finish_ok!(d, "");
    }

    // TODO more tests

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = BigFive2003Encoding.decoder();
        assert_feed_ok!(d, [0xa4, 0x40], [0xa4], "\u4e00");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xa4, 0x40], [], "\u4e00");
        assert_finish_ok!(d, "");
    }
}


// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy simplified Chinese encodings based on GB 2312 and GB 18030.

use index;
use types::*;

/**
 * GB 18030-2005.
 *
 * This is a simplified Chinese encoding which extends GBK 1.0 to a pan-Unicode encoding.
 * It assigns four-byte sequences to every Unicode codepoint missing from the GBK area,
 * lexicographically ordered with occasional "gaps" for codepoints in the GBK area.
 * Due to this compatibility decision,
 * there is no simple relationship between these four-byte sequences and Unicode codepoints,
 * though there *exists* a relatively simple mapping algorithm with a small lookup table.
 *
 * The original GBK 1.0 region spans `[81-FE] [40-7E 80-FE]`, and is derived from
 * several different revisions of a family of encodings named "GBK":
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
 */
#[deriving(Clone)]
pub struct GB18030Encoding;

impl Encoding for GB18030Encoding {
    fn name(&self) -> &'static str { "gb18030" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("gb18030") }
    fn encoder(&self) -> Box<Encoder> { GB18030Encoder::new() }
    fn decoder(&self) -> Box<Decoder> { GB18030Decoder::new() }
}

/// An encoder for GB 18030.
#[deriving(Clone)]
pub struct GB18030Encoder;

impl GB18030Encoder {
    pub fn new() -> Box<Encoder> { box GB18030Encoder as Box<Encoder> }
}

impl Encoder for GB18030Encoder {
    fn from_self(&self) -> Box<Encoder> { GB18030Encoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ch in input.chars() {
            if ch < '\u0080' {
                output.write_byte(ch as u8);
            } else {
                let ptr = index::gb18030::backward(ch as u32);
                if ptr == 0xffff {
                    let ptr = index::gb18030_ranges::backward(ch as u32);
                    assert!(ptr != 0xffffffff);
                    let (ptr, byte4) = (ptr / 10, ptr % 10);
                    let (ptr, byte3) = (ptr / 126, ptr % 126);
                    let (byte1, byte2) = (ptr / 10, ptr % 10);
                    output.write_byte((byte1 + 0x81) as u8);
                    output.write_byte((byte2 + 0x30) as u8);
                    output.write_byte((byte3 + 0x81) as u8);
                    output.write_byte((byte4 + 0x30) as u8);
                } else {
                    let lead = ptr / 190 + 0x81;
                    let trail = ptr % 190;
                    let trailoffset = if trail < 0x3f {0x40} else {0x41};
                    output.write_byte(lead as u8);
                    output.write_byte((trail + trailoffset) as u8);
                }
            }
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

ascii_compatible_stateful_decoder! {
    #[doc="A decoder for GB 18030."]
    #[deriving(Clone)]
    struct GB18030Decoder;

    module gb18030;

    internal pub fn map_two_bytes(lead: u8, trail: u8) -> u32 {
        use index;

        let lead = lead as uint;
        let trail = trail as uint;
        let index = match (lead, trail) {
            (0x81..0xfe, 0x40..0x7e) | (0x81..0xfe, 0x80..0xfe) => {
                let trailoffset = if trail < 0x7f {0x40} else {0x41};
                (lead - 0x81) * 190 + trail - trailoffset
            }
            _ => 0xffff,
        };
        index::gb18030::forward(index as u16)
    }

    internal pub fn map_four_bytes(b1: u8, b2: u8, b3: u8, b4: u8) -> u32 {
        use index;

        // no range check here, caller should have done all checks
        let index = (b1 as uint - 0x81) * 12600 + (b2 as uint - 0x30) * 1260 +
                    (b3 as uint - 0x81) * 10 + (b4 as uint - 0x30);
        index::gb18030_ranges::forward(index as u32)
    }

    // gb18030 first = 0x00, gb18030 second = 0x00, gb18030 third = 0x00
    initial state S0(ctx) {
        case b @ 0x00..0x7f => ctx.emit(b as u32);
        case 0x80 => ctx.emit(0x20ac);
        case b @ 0x81..0xfe => S1(ctx, b);
        case _ => ctx.err("invalid sequence");
    }

    // gb18030 first != 0x00, gb18030 second = 0x00, gb18030 third = 0x00
    state S1(ctx, first: u8) {
        case b @ 0x30..0x39 => S2(ctx, first, b);
        case b => {
            let ch = map_two_bytes(first, b);
            if ch == 0xffff {
                ctx.backup_and_err(1, "invalid sequence") // unconditional
            } else {
                ctx.emit(ch)
            }
        };
    }

    // gb18030 first != 0x00, gb18030 second != 0x00, gb18030 third = 0x00
    state S2(ctx, first: u8, second: u8) {
        case b @ 0x81..0xfe => S3(ctx, first, second, b);
        case _ => ctx.backup_and_err(2, "invalid sequence");
    }

    // gb18030 first != 0x00, gb18030 second != 0x00, gb18030 third != 0x00
    state S3(ctx, first: u8, second: u8, third: u8) {
        case b @ 0x30..0x39 => {
            let ch = map_four_bytes(first, second, third, b);
            if ch == 0xffffffff {
                ctx.backup_and_err(3, "invalid sequence") // unconditional
            } else {
                ctx.emit(ch)
            }
        };
        case _ => ctx.backup_and_err(3, "invalid sequence");
    }
}

#[cfg(test)]
mod gb18030_tests {
    extern crate test;
    use super::GB18030Encoding;
    use testutils;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = GB18030Encoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_ok!(e, "\u4e2d\u534e\u4eba\u6c11\u5171\u548c\u56fd", "",
                        [0xd6, 0xd0, 0xbb, 0xaa, 0xc8, 0xcb, 0xc3, 0xf1,
                         0xb9, 0xb2, 0xba, 0xcd, 0xb9, 0xfa]);
        assert_feed_ok!(e, "1\u20ac/m", "", [0x31, 0xa2, 0xe3, 0x2f, 0x6d]);
        assert_feed_ok!(e, "\uff21\uff22\uff23", "", [0xa3, 0xc1, 0xa3, 0xc2, 0xa3, 0xc3]);
        assert_feed_ok!(e, "\u0080", "", [0x81, 0x30, 0x81, 0x30]);
        assert_feed_ok!(e, "\u0081", "", [0x81, 0x30, 0x81, 0x31]);
        assert_feed_ok!(e, "\u00a3", "", [0x81, 0x30, 0x84, 0x35]);
        assert_feed_ok!(e, "\u00a4", "", [0xa1, 0xe8]);
        assert_feed_ok!(e, "\u00a5", "", [0x81, 0x30, 0x84, 0x36]);
        assert_feed_ok!(e, "\U0010ffff", "", [0xe3, 0x32, 0x9a, 0x35]);
        assert_feed_ok!(e, "\U0002a6a5\u3007", "", [0x98, 0x35, 0xee, 0x37, 0xa9, 0x96]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = GB18030Encoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0xd6, 0xd0, 0xbb, 0xaa, 0xc8, 0xcb, 0xc3, 0xf1,
                            0xb9, 0xb2, 0xba, 0xcd, 0xb9, 0xfa], [],
                        "\u4e2d\u534e\u4eba\u6c11\u5171\u548c\u56fd");
        assert_feed_ok!(d, [0x31, 0x80, 0x2f, 0x6d], [], "1\u20ac/m");
        assert_feed_ok!(d, [0xa3, 0xc1, 0xa3, 0xc2, 0xa3, 0xc3], [], "\uff21\uff22\uff23");
        assert_feed_ok!(d, [0x81, 0x30, 0x81, 0x30], [], "\u0080");
        assert_feed_ok!(d, [0x81, 0x30, 0x81, 0x31], [], "\u0081");
        assert_feed_ok!(d, [0x81, 0x30, 0x84, 0x35], [], "\u00a3");
        assert_feed_ok!(d, [0xa1, 0xe8], [], "\u00a4" );
        assert_feed_ok!(d, [0x81, 0x30, 0x84, 0x36], [], "\u00a5");
        assert_feed_ok!(d, [0xe3, 0x32, 0x9a, 0x35], [], "\U0010ffff");
        assert_feed_ok!(d, [0x98, 0x35, 0xee, 0x37, 0xa9, 0x96], [], "\U0002a6a5\u3007");
        assert_finish_ok!(d, "");
    }

    // TODO more tests

    #[test]
    fn test_decoder_invalid_boundary() {
        // U+10FFFF (E3 32 9A 35) is the last Unicode codepoint, E3 32 9A 36 is invalid.
        // note that since the 2nd to 4th bytes may coincide with ASCII, bytes 32 9A 36 is
        // not considered to be in the problem. this is compatible to WHATWG Encoding standard.
        let mut d = GB18030Encoding.decoder();
        assert_feed_ok!(d, [], [0xe3], "");
        assert_feed_err!(d, [], [], [0x32, 0x9a, 0x36], "");
        assert_finish_ok!(d, "");

        let mut d = GB18030Encoding.decoder();
        assert_feed_ok!(d, [], [0xe3], "");
        assert_feed_ok!(d, [], [0x32, 0x9a], "");
        assert_feed_err!(d, [], [], [0x36], ""); // XXX whooops, 32 9A should not be in problem!
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = GB18030Encoding.decoder();
        assert_feed_ok!(d, [0xd2, 0xbb], [0xd2], "\u4e00");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xd2, 0xbb], [], "\u4e00");
        assert_finish_ok!(d, "");

        let mut d = GB18030Encoding.decoder();
        assert_feed_ok!(d, [0x98, 0x35, 0xee, 0x37], [0x98, 0x35, 0xee], "\U0002a6a5");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0x98, 0x35, 0xee, 0x37], [0x98, 0x35], "\U0002a6a5");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0x98, 0x35, 0xee, 0x37], [0x98], "\U0002a6a5");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0x98, 0x35, 0xee, 0x37], [], "\U0002a6a5");
        assert_finish_ok!(d, "");
    }

    #[bench]
    fn bench_decode_short_text(bencher: &mut test::Bencher) {
        static Encoding: GB18030Encoding = GB18030Encoding;
        let s = Encoding.encode(testutils::SIMPLIFIED_CHINESE_TEXT, EncodeStrict).ok().unwrap();
        bencher.bytes = s.len() as u64;
        bencher.iter(|| {
            Encoding.decode(s.as_slice(), DecodeStrict).ok().unwrap();
        })
    }
}


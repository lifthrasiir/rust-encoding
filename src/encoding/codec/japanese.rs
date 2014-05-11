// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy Japanese encodings based on JIS X 0208 and JIS X 0212.

use util::StrCharIndex;
use index;
use types::*;

/**
 * EUC-JP. (XXX with asymmetric JIS X 0212 support)
 *
 * This is a Japanese encoding created from three JIS character sets:
 *
 * - JIS X 0201, which lower half is ISO/IEC 646:JP (US-ASCII with yen sign and overline)
 *   and upper half contains legacy half-width Katakanas.
 * - JIS X 0208, a primary graphic character set (94x94).
 * - JIS X 0212, a supplementary graphic character set (94x94).
 *
 * EUC-JP contains the lower half of JIS X 0201 in G0 (`[21-7E]`),
 * JIS X 0208 in G1 (`[A1-FE] [A1-FE]`),
 * the upper half of JIS X 0212 in G2 (`8E [A1-DF]`), and
 * JIS X 0212 in G3 (`8F [A1-FE] [A1-FE]`).
 */
#[deriving(Clone)]
pub struct EUCJPEncoding;

impl Encoding for EUCJPEncoding {
    fn name(&self) -> &'static str { "euc-jp" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("euc-jp") }
    fn encoder(&self) -> Box<Encoder> { EUCJPEncoder::new() }
    fn decoder(&self) -> Box<Decoder> { EUCJP0212Decoder::new() }
}

/// An encoder for EUC-JP with unused G3 character set.
#[deriving(Clone)]
pub struct EUCJPEncoder;

impl EUCJPEncoder {
    pub fn new() -> Box<Encoder> { box EUCJPEncoder as Box<Encoder> }
}

impl Encoder for EUCJPEncoder {
    fn from_self(&self) -> Box<Encoder> { EUCJPEncoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            match ch {
                '\u0000'..'\u007f' => { output.write_byte(ch as u8); }
                '\u00a5' => { output.write_byte(0x5c); }
                '\u203e' => { output.write_byte(0x7e); }
                '\uff61'..'\uff9f' => {
                    output.write_byte(0x8e);
                    output.write_byte((ch as uint - 0xff61 + 0xa1) as u8);
                }
                _ => {
                    let ptr = index::jis0208::backward(ch as u32);
                    if ptr == 0xffff {
                        return (i, Some(CodecError {
                            upto: j, cause: "unrepresentable character".into_maybe_owned()
                        }));
                    } else {
                        let lead = ptr / 94 + 0xa1;
                        let trail = ptr % 94 + 0xa1;
                        output.write_byte(lead as u8);
                        output.write_byte(trail as u8);
                    }
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
    #[doc="A decoder for EUC-JP with JIS X 0212 in G3."]
    #[deriving(Clone)]
    struct EUCJP0212Decoder;

    module eucjp;

    internal pub fn map_two_0208_bytes(lead: u8, trail: u8) -> u32 {
        use index;

        let lead = lead as uint;
        let trail = trail as uint;
        let index = match (lead, trail) {
            (0xa1..0xfe, 0xa1..0xfe) => (lead - 0xa1) * 94 + trail - 0xa1,
            _ => 0xffff,
        };
        index::jis0208::forward(index as u16)
    }

    internal pub fn map_two_0212_bytes(lead: u8, trail: u8) -> u32 {
        use index;

        let lead = lead as uint;
        let trail = trail as uint;
        let index = match (lead, trail) {
            (0xa1..0xfe, 0xa1..0xfe) => (lead - 0xa1) * 94 + trail - 0xa1,
            _ => 0xffff,
        };
        index::jis0212::forward(index as u16)
    }

    // euc-jp lead = 0x00
    initial state S0(ctx) {
        case b @ 0x00..0x7f => ctx.emit(b as u32);
        case 0x8e => S1(ctx);
        case 0x8f => S2(ctx);
        case b @ 0xa1..0xfe => S3(ctx, b);
        case _ => ctx.err("invalid sequence");
    }

    // euc-jp lead = 0x8e
    state S1(ctx) {
        case b @ 0xa1..0xdf => ctx.emit(0xff61 + b as u32 - 0xa1);
        case 0xa1..0xfe => ctx.err("invalid sequence");
        case _ => ctx.backup_and_err(1, "invalid sequence");
    }

    // euc-jp lead = 0x8f
    // JIS X 0201 half-width katakana
    state S2(ctx) {
        case b @ 0xa1..0xfe => S4(ctx, b);
        case _ => ctx.backup_and_err(1, "invalid sequence");
    }

    // euc-jp lead != 0x00, euc-jp jis0212 flag = unset
    // JIS X 0208 two-byte sequence
    state S3(ctx, lead: u8) {
        case b @ 0xa1..0xfe => {
            let ch = map_two_0208_bytes(lead, b);
            if ch == 0xffff {
                // do NOT backup, we only backup for out-of-range trails.
                ctx.err("invalid sequence")
            } else {
                ctx.emit(ch as u32)
            }
        };
        case _ => ctx.backup_and_err(1, "invalid sequence");
    }

    // euc-jp lead != 0x00, euc-jp jis0212 flag = set
    // JIS X 0212 three-byte sequence
    state S4(ctx, lead: u8) {
        case b @ 0xa1..0xfe => {
            let ch = map_two_0212_bytes(lead, b);
            if ch == 0xffff {
                // do NOT backup, we only backup for out-of-range trails.
                ctx.err("invalid sequence")
            } else {
                ctx.emit(ch as u32)
            }
        };
        case _ => ctx.backup_and_err(1, "invalid sequence");
    }
}

#[cfg(test)]
mod eucjp_tests {
    extern crate test;
    use super::EUCJPEncoding;
    use testutils;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = EUCJPEncoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_ok!(e, "\u00a5", "", [0x5c]);
        assert_feed_ok!(e, "\u203e", "", [0x7e]);
        assert_feed_ok!(e, "\u306b\u307b\u3093", "", [0xa4, 0xcb, 0xa4, 0xdb, 0xa4, 0xf3]);
        assert_feed_ok!(e, "\uff86\uff8e\uff9d", "", [0x8e, 0xc6, 0x8e, 0xce, 0x8e, 0xdd]);
        assert_feed_ok!(e, "\u65e5\u672c", "", [0xc6, 0xfc, 0xcb, 0xdc]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = EUCJPEncoding.encoder();
        assert_feed_err!(e, "", "\uffff", "", []);
        assert_feed_err!(e, "?", "\uffff", "!", [0x3f]);
        // JIS X 0212 is not supported in the encoder
        assert_feed_err!(e, "", "\u736c", "\u8c78", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = EUCJPEncoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0x5c], [], "\\");
        assert_feed_ok!(d, [0x7e], [], "~");
        assert_feed_ok!(d, [0xa4, 0xcb, 0xa4, 0xdb, 0xa4, 0xf3], [], "\u306b\u307b\u3093");
        assert_feed_ok!(d, [0x8e, 0xc6, 0x8e, 0xce, 0x8e, 0xdd], [], "\uff86\uff8e\uff9d");
        assert_feed_ok!(d, [0xc6, 0xfc, 0xcb, 0xdc], [], "\u65e5\u672c");
        assert_feed_ok!(d, [0x8f, 0xcb, 0xc6, 0xec, 0xb8], [], "\u736c\u8c78");
        assert_finish_ok!(d, "");
    }

    // TODO more tests

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = EUCJPEncoding.decoder();
        assert_feed_ok!(d, [0xa4, 0xa2], [0xa4], "\u3042");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xa4, 0xa2], [], "\u3042");
        assert_finish_ok!(d, "");
    }

    #[bench]
    fn bench_decode_short_text(bencher: &mut test::Bencher) {
        static Encoding: EUCJPEncoding = EUCJPEncoding;
        let s = Encoding.encode(testutils::JAPANESE_TEXT, EncodeStrict).ok().unwrap();
        bencher.bytes = s.len() as u64;
        bencher.iter(|| {
            Encoding.decode(s.as_slice(), DecodeStrict).ok().unwrap();
        })
    }
}

/**
 * Windows code page 932, i.e. Shift_JIS with IBM/NEC extensions.
 *
 * This is a Japanese encoding for JIS X 0208
 * compatible to the original assignments of JIS X 0201 (`[21-7E A1-DF]`).
 * The 94 by 94 region of JIS X 0208 is sliced, or rather "shifted" into
 * the odd half (odd row number) and even half (even row number),
 * and merged into the 188 by 47 region mapped to `[81-9F E0-EF] [40-7E 80-FC]`.
 * The remaining area, `[80 A0 F0-FF] [40-7E 80-FC]`, has been subjected to
 * numerous extensions incompatible to each other.
 * This particular implementation uses IBM/NEC extensions
 * which assigns more characters to `[F0-FC 80-FC]` and also to the Private Use Area (PUA).
 * It requires some cares to handle
 * since the second byte of JIS X 0208 can have its MSB unset.
 */
#[deriving(Clone)]
pub struct Windows31JEncoding;

impl Encoding for Windows31JEncoding {
    fn name(&self) -> &'static str { "windows-31j" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("shift_jis") } // WHATWG compatibility
    fn encoder(&self) -> Box<Encoder> { Windows31JEncoder::new() }
    fn decoder(&self) -> Box<Decoder> { Windows31JDecoder::new() }
}

/// An encoder for Shift_JIS with IBM/NEC extensions.
#[deriving(Clone)]
pub struct Windows31JEncoder;

impl Windows31JEncoder {
    pub fn new() -> Box<Encoder> { box Windows31JEncoder as Box<Encoder> }
}

impl Encoder for Windows31JEncoder {
    fn from_self(&self) -> Box<Encoder> { Windows31JEncoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            match ch {
                '\u0000'..'\u0080' => { output.write_byte(ch as u8); }
                '\u00a5' => { output.write_byte(0x5c); }
                '\u203e' => { output.write_byte(0x7e); }
                '\uff61'..'\uff9f' => { output.write_byte((ch as uint - 0xff61 + 0xa1) as u8); }
                _ => {
                    let ptr = index::jis0208::backward(ch as u32);
                    if ptr == 0xffff {
                        return (i, Some(CodecError {
                            upto: j, cause: "unrepresentable character".into_maybe_owned(),
                        }));
                    } else {
                        let lead = ptr / 188;
                        let leadoffset = if lead < 0x1f {0x81} else {0xc1};
                        let trail = ptr % 188;
                        let trailoffset = if trail < 0x3f {0x40} else {0x41};
                        output.write_byte((lead + leadoffset) as u8);
                        output.write_byte((trail + trailoffset) as u8);
                    }
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
    #[doc="A decoder for Shift_JIS with IBM/NEC extensions."]
    #[deriving(Clone)]
    struct Windows31JDecoder;

    module windows31j;

    internal pub fn map_two_0208_bytes(lead: u8, trail: u8) -> u32 {
        use index;

        let lead = lead as uint;
        let trail = trail as uint;
        let leadoffset = if lead < 0xa0 {0x81} else {0xc1};
        let trailoffset = if trail < 0x7f {0x40} else {0x41};
        let index = match (lead, trail) {
            // XXX the Encoding spec seems to be incorrect in this regard, see Bug 25649.
            // INCORRECT: (0xf0..0xf9, _) =>
            (0xf0..0xf9, 0x40..0x7e) | (0xf0..0xf9, 0x80..0xfc) =>
                return (0xe000 + (lead - 0xf0) * 188 + trail - trailoffset) as u32,
            (0x81..0x9f, 0x40..0x7e) | (0x81..0x9f, 0x80..0xfc) |
            (0xe0..0xfc, 0x40..0x7e) | (0xe0..0xfc, 0x80..0xfc) =>
                (lead - leadoffset) * 188 + trail - trailoffset,
            _ => 0xffff,
        };
        index::jis0208::forward(index as u16)
    }

    // shift_jis lead = 0x00
    initial state S0(ctx) {
        case b @ 0x00..0x7f => ctx.emit(b as u32);
        case b @ 0xa1..0xdf => ctx.emit(0xff61 + b as u32 - 0xa1);
        case b @ 0x81..0x9f | b @ 0xe0..0xfc => S1(ctx, b);
        case _ => ctx.err("invalid sequence");
    }

    // shift_jis lead != 0x00
    state S1(ctx, lead: u8) {
        case b => {
            let ch = map_two_0208_bytes(lead, b);
            if ch == 0xffff {
                ctx.backup_and_err(1, "invalid sequence") // unconditional
            } else {
                ctx.emit(ch)
            }
        };
    }
}

#[cfg(test)]
mod windows31j_tests {
    extern crate test;
    use super::Windows31JEncoding;
    use testutils;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = Windows31JEncoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_ok!(e, "\u00a5", "", [0x5c]);
        assert_feed_ok!(e, "\u203e", "", [0x7e]);
        assert_feed_ok!(e, "\u306b\u307b\u3093", "", [0x82, 0xc9, 0x82, 0xd9, 0x82, 0xf1]);
        assert_feed_ok!(e, "\uff86\uff8e\uff9d", "", [0xc6, 0xce, 0xdd]);
        assert_feed_ok!(e, "\u65e5\u672c", "", [0x93, 0xfa, 0x96, 0x7b]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_encoder_no_eudc() {
        let mut e = Windows31JEncoding.encoder();
        assert_feed_err!(e, "", "\ue000", "", []);
        assert_feed_err!(e, "", "\ue757", "", []);
        assert_feed_err!(e, "", "\ue758", "", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = Windows31JEncoding.encoder();
        assert_feed_err!(e, "", "\uffff", "", []);
        assert_feed_err!(e, "?", "\uffff", "!", [0x3f]);
        assert_feed_err!(e, "", "\u736c", "\u8c78", []);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = Windows31JEncoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0x5c], [], "\\");
        assert_feed_ok!(d, [0x7e], [], "~");
        assert_feed_ok!(d, [0x82, 0xc9, 0x82, 0xd9, 0x82, 0xf1], [], "\u306b\u307b\u3093");
        assert_feed_ok!(d, [0xc6, 0xce, 0xdd], [], "\uff86\uff8e\uff9d");
        assert_feed_ok!(d, [0x93, 0xfa, 0x96, 0x7b], [], "\u65e5\u672c");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_eudc() {
        let mut d = Windows31JEncoding.decoder();
        assert_feed_ok!(d, [], [0xf0], "");
        assert_feed_ok!(d, [0x40], [], "\ue000");
        assert_feed_ok!(d, [0xf9, 0xfc], [], "\ue757");
        assert_feed_err!(d, [], [0xf0], [0x00], "");
        assert_feed_err!(d, [], [0xf0], [0xff], "");
        assert_finish_ok!(d, "");
    }

    // TODO more tests

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = Windows31JEncoding.decoder();
        assert_feed_ok!(d, [0x82, 0xa0], [0x82], "\u3042");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0x82, 0xa0], [], "\u3042");
        assert_finish_ok!(d, "");
    }

    #[bench]
    fn bench_decode_short_text(bencher: &mut test::Bencher) {
        static Encoding: Windows31JEncoding = Windows31JEncoding;
        let s = Encoding.encode(testutils::JAPANESE_TEXT, EncodeStrict).ok().unwrap();
        bencher.bytes = s.len() as u64;
        bencher.iter(|| {
            Encoding.decode(s.as_slice(), DecodeStrict).ok().unwrap();
        })
    }
}


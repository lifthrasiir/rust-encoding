// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy Korean encodings based on KS X 1001.

use util::StrCharIndex;
use index;
use types::*;

/**
 * Windows code page 949.
 *
 * This is a Korean encoding derived from EUC-KR,
 * which is so widespread that most occurrences of EUC-KR actually mean this encoding.
 * Unlike KS X 1001 (and EUC-KR) which only contains a set of 2,350 common Hangul syllables,
 * it assigns remaining 8,822 Hangul syllables to the two-byte sequence
 * which second byte have its MSB unset (i.e. `[81-C6] [41-5A 61-7A 81-FE]`).
 * Its design strongly resembles that of Shift_JIS but less prone to errors
 * since the set of MSB-unset second bytes is much limited compared to Shift_JIS.
 */
#[deriving(Clone)]
pub struct Windows949Encoding;

impl Encoding for Windows949Encoding {
    fn name(&self) -> &'static str { "windows-949" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("euc-kr") } // WHATWG compatibility
    fn encoder(&self) -> Box<Encoder> { Windows949Encoder::new() }
    fn decoder(&self) -> Box<Decoder> { Windows949Decoder::new() }
}

/// An encoder for Windows code page 949.
#[deriving(Clone)]
pub struct Windows949Encoder;

impl Windows949Encoder {
    pub fn new() -> Box<Encoder> { box Windows949Encoder as Box<Encoder> }
}

impl Encoder for Windows949Encoder {
    fn from_self(&self) -> Box<Encoder> { Windows949Encoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        for ((i,j), ch) in input.index_iter() {
            if ch <= '\u007f' {
                output.write_byte(ch as u8);
            } else {
                let ptr = index::euc_kr::backward(ch as u32);
                if ptr == 0xffff {
                    return (i, Some(CodecError {
                        upto: j, cause: "unrepresentable character".into_maybe_owned()
                    }));
                } else if ptr < (26 + 26 + 126) * (0xc7 - 0x81) {
                    let lead = ptr / (26 + 26 + 126) + 0x81;
                    let trail = ptr % (26 + 26 + 126);
                    let offset = if trail < 26 {0x41} else if trail < 26 + 26 {0x47} else {0x4d};
                    output.write_byte(lead as u8);
                    output.write_byte((trail + offset) as u8);
                } else {
                    let ptr = ptr - (26 + 26 + 126) * (0xc7 - 0x81);
                    let lead = ptr / 94 + 0xc7;
                    let trail = ptr % 94 + 0xa1;
                    output.write_byte(lead as u8);
                    output.write_byte(trail as u8);
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
    #[doc="A decoder for Windows code page 949."]
    #[deriving(Clone)]
    struct Windows949Decoder;

    module windows949;

    internal pub fn map_two_bytes(lead: u8, trail: u8) -> u32 {
        use index;

        let lead = lead as uint;
        let trail = trail as uint;
        let index = match (lead, trail) {
            (0x81..0xc6, 0x41..0x5a) =>
                (26 + 26 + 126) * (lead - 0x81) + trail - 0x41,
            (0x81..0xc6, 0x61..0x7a) =>
                (26 + 26 + 126) * (lead - 0x81) + 26 + trail - 0x61,
            (0x81..0xc6, 0x81..0xfe) =>
                (26 + 26 + 126) * (lead - 0x81) + 26 + 26 + trail - 0x81,
            (0xc7..0xfe, 0xa1..0xfe) =>
                (26 + 26 + 126) * (0xc7 - 0x81) + (lead - 0xc7) * 94 + trail - 0xa1,
            (_, _) => 0xffff,
        };
        index::euc_kr::forward(index as u16)
    }

    // euc-kr lead = 0x00
    initial state S0(ctx) {
        case b @ 0x00..0x7f => ctx.emit(b as u32);
        case b @ 0x81..0xfe => S1(ctx, b);
        case _ => ctx.err("invalid sequence");
    }

    // euc-kr lead != 0x00
    state S1(ctx, lead: u8) {
        case b => match map_two_bytes(lead, b) {
            0xffff => ctx.backup_and_err(1, "invalid sequence"), // unconditional
            ch => ctx.emit(ch as u32)
        };
    }
}

#[cfg(test)]
mod windows949_tests {
    extern crate test;
    use super::Windows949Encoding;
    use testutils;
    use types::*;

    #[test]
    fn test_encoder_valid() {
        let mut e = Windows949Encoding.encoder();
        assert_feed_ok!(e, "A", "", [0x41]);
        assert_feed_ok!(e, "BC", "", [0x42, 0x43]);
        assert_feed_ok!(e, "", "", []);
        assert_feed_ok!(e, "\uac00", "", [0xb0, 0xa1]);
        assert_feed_ok!(e, "\ub098\ub2e4", "", [0xb3, 0xaa, 0xb4, 0xd9]);
        assert_feed_ok!(e, "\ubdc1\u314b\ud7a3", "", [0x94, 0xee, 0xa4, 0xbb, 0xc6, 0x52]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = Windows949Encoding.encoder();
        assert_feed_err!(e, "", "\uffff", "", []);
        assert_feed_err!(e, "?", "\uffff", "!", [0x3f]);
        assert_finish_ok!(e, []);
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = Windows949Encoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0xb0, 0xa1], [], "\uac00");
        assert_feed_ok!(d, [0xb3, 0xaa, 0xb4, 0xd9], [], "\ub098\ub2e4");
        assert_feed_ok!(d, [0x94, 0xee, 0xa4, 0xbb, 0xc6, 0x52], [], "\ubdc1\u314b\ud7a3");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_valid_partial() {
        let mut d = Windows949Encoding.decoder();
        assert_feed_ok!(d, [], [0xb0], "");
        assert_feed_ok!(d, [0xa1], [], "\uac00");
        assert_feed_ok!(d, [0xb3, 0xaa], [0xb4], "\ub098");
        assert_feed_ok!(d, [0xd9], [0x94], "\ub2e4");
        assert_feed_ok!(d, [0xee, 0xa4, 0xbb, 0xc6, 0x52], [], "\ubdc1\u314b\ud7a3");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_invalid_lone_lead_immediate_test_finish() {
        for i in range(0x81, 0xff) {
            let i = i as u8;
            let mut d = Windows949Encoding.decoder();
            assert_feed_ok!(d, [], [i], ""); // wait for a trail
            assert_finish_err!(d, "");
        }

        // 80/FF: immediate failure
        let mut d = Windows949Encoding.decoder();
        assert_feed_err!(d, [], [0x80], [], "");
        assert_feed_err!(d, [], [0xff], [], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_invalid_lone_lead_followed_by_space() {
        for i in range(0x80, 0x100) {
            let i = i as u8;
            let mut d = Windows949Encoding.decoder();
            assert_feed_err!(d, [], [i], [0x20], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_decoder_invalid_lead_followed_by_invalid_trail() {
        for i in range(0x80u16, 0x100) {
            let i = i as u8;
            let mut d = Windows949Encoding.decoder();
            assert_feed_err!(d, [], [i], [0x80], "");
            assert_feed_err!(d, [], [i], [0xff], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_decoder_invalid_boundary() {
        // U+D7A3 (C6 52) is the last Hangul syllable not in KS X 1001, C6 53 is invalid.
        // note that since the trail byte may coincide with ASCII, the trail byte 53 is
        // not considered to be in the problem. this is compatible to WHATWG Encoding standard.
        let mut d = Windows949Encoding.decoder();
        assert_feed_ok!(d, [], [0xc6], "");
        assert_feed_err!(d, [], [], [0x53], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = Windows949Encoding.decoder();
        assert_feed_ok!(d, [0xb0, 0xa1], [0xb0], "\uac00");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xb0, 0xa1], [], "\uac00");
        assert_finish_ok!(d, "");
    }

    #[bench]
    fn bench_encode_short_text(bencher: &mut test::Bencher) {
        static Encoding: Windows949Encoding = Windows949Encoding;
        let s = testutils::KOREAN_TEXT;
        bencher.bytes = s.len() as u64;
        bencher.iter(|| test::black_box({
            Encoding.encode(s.as_slice(), EncodeStrict)
        }))
    }

    #[bench]
    fn bench_decode_short_text(bencher: &mut test::Bencher) {
        static Encoding: Windows949Encoding = Windows949Encoding;
        let s = Encoding.encode(testutils::KOREAN_TEXT, EncodeStrict).ok().unwrap();
        bencher.bytes = s.len() as u64;
        bencher.iter(|| test::black_box({
            Encoding.decode(s.as_slice(), DecodeStrict)
        }))
    }
}


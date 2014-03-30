// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy Japanese encodings based on JIS X 0208 and JIS X 0212.

use util::{as_char, StrCharIndex};
use index0208 = index::jis0208;
use index0212 = index::jis0212;
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
    fn encoder(&self) -> ~Encoder { EUCJPEncoder::new() }
    fn decoder(&self) -> ~Decoder { EUCJP0212Decoder::new() }
}

/// An encoder for EUC-JP with unused G3 character set.
#[deriving(Clone)]
pub struct EUCJPEncoder;

impl EUCJPEncoder {
    pub fn new() -> ~Encoder { ~EUCJPEncoder as ~Encoder }
}

impl Encoder for EUCJPEncoder {
    fn from_self(&self) -> ~Encoder { EUCJPEncoder::new() }
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
                    let ptr = index0208::backward(ch as u32);
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

/// A decoder for EUC-JP with JIS X 0212 in G3.
#[deriving(Clone)]
pub struct EUCJP0212Decoder {
    first: u8,
    second: u8,
}

impl EUCJP0212Decoder {
    pub fn new() -> ~Decoder { ~EUCJP0212Decoder { first: 0, second: 0 } as ~Decoder }
}

impl Decoder for EUCJP0212Decoder {
    fn from_self(&self) -> ~Decoder { EUCJP0212Decoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        fn map_two_0208_bytes(lead: u8, trail: u8) -> u32 {
            let lead = lead as uint;
            let trail = trail as uint;
            let index = match (lead, trail) {
                (0xa1..0xfe, 0xa1..0xfe) => (lead - 0xa1) * 94 + trail - 0xa1,
                _ => 0xffff,
            };
            index0208::forward(index as u16)
        }

        fn map_two_0212_bytes(lead: u8, trail: u8) -> u32 {
            let lead = lead as uint;
            let trail = trail as uint;
            let index = match (lead, trail) {
                (0xa1..0xfe, 0xa1..0xfe) => (lead - 0xa1) * 94 + trail - 0xa1,
                _ => 0xffff,
            };
            index0212::forward(index as u16)
        }

        let mut i = 0;
        let mut processed = 0;
        let len = input.len();

        if i >= len { return (processed, None); }

        if self.first != 0 {
            let first = self.first;
            match (first, input[i]) {
                (0x8e, 0xa1..0xdf) => {
                    output.write_char(as_char(0xff61 + input[i] as uint - 0xa1));
                }
                (0x8f, trail) => {
                    self.first = 0;
                    self.second = trail as u8;
                    // pass through
                }
                (lead, trail) => {
                    let ch = map_two_0208_bytes(lead, trail);
                    if ch == 0xffff {
                        self.first = 0;
                        return (processed, Some(CodecError {
                            upto: i, cause: "invalid sequence".into_maybe_owned()
                        }));
                    }
                    output.write_char(as_char(ch));
                }
            }
            i += 1;
            if i >= len {
                self.first = 0;
                return (processed, None);
            }
        }

        if self.second != 0 {
            let ch = map_two_0212_bytes(self.second, input[i]);
            if ch == 0xffff {
                self.second = 0;
                return (processed, Some(CodecError {
                    upto: i, cause: "invalid sequence".into_maybe_owned()
                }));
            }
            output.write_char(as_char(ch));
            i += 1;
        }

        self.first = 0;
        self.second = 0;
        processed = i;
        while i < len {
            match input[i] {
                0x00..0x7f => {
                    output.write_char(input[i] as char);
                }
                0x8e | 0x8f | 0xa1..0xfe => {
                    i += 1;
                    if i >= len {
                        self.first = input[i-1];
                        break;
                    }
                    match (input[i-1], input[i]) {
                        (0x8e, 0xa1..0xdf) => { // JIS X 0201 half-width katakana
                            output.write_char(as_char(0xff61 + input[i] as uint - 0xa1));
                        }
                        (0x8f, 0xa1..0xfe) => { // JIS X 0212 three-byte sequence
                            i += 1;
                            if i >= len {
                                self.second = input[i];
                                break;
                            }
                            let ch = map_two_0212_bytes(input[i-1], input[i]);
                            if ch == 0xffff {
                                return (processed, Some(CodecError {
                                    upto: i, cause: "invalid sequence".into_maybe_owned()
                                }));
                            }
                            output.write_char(as_char(ch));
                        }
                        (0xa1..0xfe, 0xa1..0xfe) => { // JIS X 0208 two-byte sequence
                            let ch = map_two_0208_bytes(input[i-1], input[i]);
                            if ch == 0xffff {
                                return (processed, Some(CodecError {
                                    upto: i, cause: "invalid sequence".into_maybe_owned()
                                }));
                            }
                            output.write_char(as_char(ch));
                        }
                        (_, trail) => {
                            // we should back up when the second byte doesn't look like EUC-JP
                            // (Encoding standard, Chapter 12.1, decoder step 7-4)
                            let upto = if trail < 0xa1 || trail > 0xfe {i} else {i+1};
                            return (processed, Some(CodecError {
                                upto: upto, cause: "invalid sequence".into_maybe_owned()
                            }));
                        }
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
        let first = self.first;
        let second = self.second;
        self.first = 0;
        self.second = 0;
        if second != 0 || first != 0 {
            Some(CodecError { upto: 0, cause: "incomplete sequence".into_maybe_owned() })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod eucjp_tests {
    use super::EUCJPEncoding;
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
    fn encoder(&self) -> ~Encoder { Windows31JEncoder::new() }
    fn decoder(&self) -> ~Decoder { Windows31JDecoder::new() }
}

/// An encoder for Shift_JIS with IBM/NEC extensions.
#[deriving(Clone)]
pub struct Windows31JEncoder;

impl Windows31JEncoder {
    pub fn new() -> ~Encoder { ~Windows31JEncoder as ~Encoder }
}

impl Encoder for Windows31JEncoder {
    fn from_self(&self) -> ~Encoder { Windows31JEncoder::new() }
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
                    let ptr = index0208::backward(ch as u32);
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

/// A decoder for Shift_JIS with IBM/NEC extensions.
#[deriving(Clone)]
pub struct Windows31JDecoder {
    lead: u8
}

impl Windows31JDecoder {
    pub fn new() -> ~Decoder { ~Windows31JDecoder { lead: 0 } as ~Decoder }
}

impl Decoder for Windows31JDecoder {
    fn from_self(&self) -> ~Decoder { Windows31JDecoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        fn map_two_0208_bytes(lead: u8, trail: u8) -> u32 {
            let lead = lead as uint;
            let trail = trail as uint;
            let index = match (lead, trail) {
                (0x81..0x9f, 0x40..0x7e) | (0x81..0x9f, 0x80..0xfc) |
                (0xe0..0xfc, 0x40..0x7e) | (0xe0..0xfc, 0x80..0xfc) => {
                    let leadoffset = if lead < 0xa0 {0x81} else {0xc1};
                    let trailoffset = if trail < 0x7f {0x40} else {0x41};
                    (lead - leadoffset) * 188 + trail - trailoffset
                }
                _ => 0xffff,
            };
            index0208::forward(index as u16)
        }

        let mut i = 0;
        let mut processed = 0;
        let len = input.len();

        if i >= len { return (processed, None); }

        if self.lead != 0 {
            let ch = map_two_0208_bytes(self.lead, input[i]);
            if ch == 0xffff {
                self.lead = 0;
                return (processed, Some(CodecError {
                    upto: i, cause: "invalid sequence".into_maybe_owned()
                }));
            }
            output.write_char(as_char(ch));
            i += 1;
        }

        self.lead = 0;
        processed = i;
        while i < len {
            match input[i] {
                0x00..0x7f => {
                    output.write_char(input[i] as char);
                }
                0xa1..0xdf => {
                    output.write_char(as_char(0xff61 + (input[i] as uint) - 0xa1));
                }
                0x81..0x9f | 0xe0..0xfc => {
                    i += 1;
                    if i >= len {
                        self.lead = input[i-1];
                        break;
                    }
                    let ch = map_two_0208_bytes(input[i-1], input[i]);
                    if ch == 0xffff {
                        return (processed, Some(CodecError {
                            upto: i, cause: "invalid sequence".into_maybe_owned()
                        }));
                    }
                    output.write_char(as_char(ch));
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
mod windows31j_tests {
    use super::Windows31JEncoding;
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

    // TODO more tests

    #[test]
    fn test_decoder_feed_after_finish() {
        let mut d = Windows31JEncoding.decoder();
        assert_feed_ok!(d, [0x82, 0xa0], [0x82], "\u3042");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0x82, 0xa0], [], "\u3042");
        assert_finish_ok!(d, "");
    }
}


// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.
//
// Portions Copyright (c) 2008-2009 Bjoern Hoehrmann <bjoern@hoehrmann.de>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! UTF-8, the universal encoding.

use std::{str, cast};
use types::*;

/**
 * UTF-8 (UCS Transformation Format, 8-bit).
 *
 * This is a Unicode encoding compatible to ASCII (ISO/IEC 646:US)
 * and able to represent all Unicode codepoints uniquely and unambiguously.
 * It has a variable-length design,
 * where one codepoint may use 1 (up to U+007F), 2 (up to U+07FF), 3 (up to U+FFFF)
 * and 4 bytes (up to U+10FFFF) depending on its value.
 * The first byte of the sequence is distinct from other "continuation" bytes of the sequence
 * making UTF-8 self-synchronizable and easy to handle.
 * It has a fixed endianness, and can be lexicographically sorted by codepoints.
 *
 * The UTF-8 scanner used by this module is heavily based on Bjoern Hoehrmann's
 * [Flexible and Economical UTF-8 Decoder](http://bjoern.hoehrmann.de/utf-8/decoder/dfa/).
 */
#[deriving(Clone)]
pub struct UTF8Encoding;

impl Encoding for UTF8Encoding {
    fn name(&self) -> &'static str { "utf-8" }
    fn whatwg_name(&self) -> Option<&'static str> { Some("utf-8") }
    fn encoder(&self) -> ~Encoder { UTF8Encoder::new() }
    fn decoder(&self) -> ~Decoder { UTF8Decoder::new() }
}

/// An encoder for UTF-8.
#[deriving(Clone)]
pub struct UTF8Encoder;

impl UTF8Encoder {
    pub fn new() -> ~Encoder { ~UTF8Encoder as ~Encoder }
}

impl Encoder for UTF8Encoder {
    fn from_self(&self) -> ~Encoder { UTF8Encoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>) {
        unsafe {
            let input: &[u8] = cast::transmute(input);
            assert!(str::is_utf8(input));
            output.write_bytes(input);
        }
        (input.len(), None)
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> {
        None
    }
}

/// A decoder for UTF-8.
pub struct UTF8Decoder {
    queuelen: uint,
    queue: [u8, ..4],
    state: u8,
}

impl UTF8Decoder {
    pub fn new() -> ~Decoder {
        ~UTF8Decoder { queuelen: 0, queue: [0, ..4], state: INITIAL_STATE } as ~Decoder
    }
}

impl Clone for UTF8Decoder {
    fn clone(&self) -> UTF8Decoder {
        UTF8Decoder { queuelen: self.queuelen, queue: self.queue, state: self.state }
    }
}

static CHAR_CATEGORY: [u8, ..256] = [
    //  0 (00-7F): one byte sequence
    //  1 (80-8F): continuation byte
    //  2 (C2-DF): start of two byte sequence
    //  3 (E1-EC,EE-EF): start of three byte sequence, next byte unrestricted
    //  4 (ED): start of three byte sequence, next byte restricted to non-surrogates (80-9F)
    //  5 (F4): start of four byte sequence, next byte restricted to 0+10FFFF (80-8F)
    //  6 (F1-F3): start of four byte sequence, next byte unrestricted
    //  7 (A0-BF): continuation byte
    //  8 (C0-C1,F5-FF): invalid (overlong or out-of-range) start of multi byte sequences
    //  9 (90-9F): continuation byte
    // 10 (E0): start of three byte sequence, next byte restricted to non-overlong (A0-BF)
    // 11 (F0): start of four byte sequence, next byte restricted to non-overlong (90-BF)

     0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
     0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
     0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
     0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
     1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,  9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,9,
     7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
     8,8,2,2,2,2,2,2,2,2,2,2,2,2,2,2,  2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
    10,3,3,3,3,3,3,3,3,3,3,3,3,4,3,3, 11,6,6,6,5,8,8,8,8,8,8,8,8,8,8,8,
];

static STATE_TRANSITIONS: [u8, ..108] = [
     0,12,24,36,60,96,84,12,12,12,48,72, //  0: '??
    13,13,13,13,13,13,13,13,13,13,13,13, // 12: xx '.. / 13: .. xx '..
    13, 0,13,13,13,13,13, 0,13, 0,13,13, // 24: .. 'cc
    13,24,13,13,13,13,13,24,13,24,13,13, // 36: .. 'cc cc
    13,13,13,13,13,13,13,24,13,13,13,13, // 48: .. 'cc(A0-BF) cc
    13,24,13,13,13,13,13,13,13,24,13,13, // 60: .. 'cc(80-9F) cc
    13,13,13,13,13,13,13,36,13,36,13,13, // 72: .. 'cc(90-BF) cc cc
    13,36,13,13,13,13,13,36,13,36,13,13, // 84: .. 'cc cc cc
    13,36,13,13,13,13,13,13,13,13,13,13, // 96: .. 'cc(80-8F) cc cc
];

static INITIAL_STATE: u8 = 0;
static ACCEPT_STATE: u8 = 0;
static REJECT_STATE: u8 = 12;
static REJECT_STATE_WITH_BACKUP: u8 = REJECT_STATE | 1;

impl Decoder for UTF8Decoder {
    fn from_self(&self) -> ~Decoder { UTF8Decoder::new() }
    fn is_ascii_compatible(&self) -> bool { true }

    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>) {
        output.writer_hint(input.len());

        fn write_bytes(output: &mut StringWriter, bytes: &[u8]) {
            output.write_str(unsafe {cast::transmute(bytes)});
        }

        let mut state = self.state;
        let mut i = 0;
        let mut processed = 0;
        let len = input.len();
        while i < len {
            let ch = input[i];
            state = STATE_TRANSITIONS[(state + CHAR_CATEGORY[ch as uint]) as uint];
            i += 1;
            match state {
                ACCEPT_STATE => { processed = i; }
                REJECT_STATE | REJECT_STATE_WITH_BACKUP => {
                    let upto = i - (state & 1) as uint;
                    self.state = INITIAL_STATE;
                    if processed > 0 && self.queuelen > 0 { // flush `queue` outside the problem
                        write_bytes(output, self.queue.slice(0, self.queuelen));
                    }
                    self.queuelen = 0;
                    write_bytes(output, input.slice(0, processed));
                    return (processed, Some(CodecError {
                        upto: upto, cause: "invalid sequence".into_maybe_owned()
                    }));
                }
                _ => {}
            }
        }

        self.state = state;
        if processed > 0 && self.queuelen > 0 { // flush `queue`
            write_bytes(output, self.queue.slice(0, self.queuelen));
            self.queuelen = 0;
        }
        write_bytes(output, input.slice(0, processed));
        if processed < len {
            let morequeuelen = len - processed;
            for i in range(0, morequeuelen) {
                self.queue[self.queuelen + i] = input[processed + i];
            }
            self.queuelen += morequeuelen;
        }
        (processed, None)
    }

    fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
        let state = self.state;
        let queuelen = self.queuelen;
        self.state = INITIAL_STATE;
        self.queuelen = 0;
        if state != ACCEPT_STATE {
            Some(CodecError { upto: 0, cause: "incomplete sequence".into_maybe_owned() })
        } else {
            assert!(queuelen == 0);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    // portions of these tests are adopted from Markus Kuhn's UTF-8 decoder capability and
    // stress test: <http://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt>.

    use super::UTF8Encoding;
    use types::*;

    #[test]
    fn test_valid() {
        // one byte
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0x41], [], "A");
        assert_feed_ok!(d, [0x42, 0x43], [], "BC");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0x44, 0x45, 0x46], [], "DEF");
        assert_finish_ok!(d, "");

        // two bytes
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xc2, 0xa2], [], "\xa2");
        assert_feed_ok!(d, [0xc2, 0xac, 0xc2, 0xa9], [], "\xac\xa9");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0xd5, 0xa1, 0xd5, 0xb5, 0xd5, 0xa2, 0xd5, 0xb8, 0xd6, 0x82,
                            0xd5, 0xa2, 0xd5, 0xa5, 0xd5, 0xb6], [],
                        "\u0561\u0575\u0562\u0578\u0582\u0562\u0565\u0576");
        assert_finish_ok!(d, "");

        // three bytes
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xed, 0x92, 0x89], [], "\ud489");
        assert_feed_ok!(d, [0xe6, 0xbc, 0xa2, 0xe5, 0xad, 0x97], [], "\u6f22\u5b57");
        assert_feed_ok!(d, [], [], "");
        assert_feed_ok!(d, [0xc9, 0x99, 0xc9, 0x94, 0xc9, 0x90], [], "\u0259\u0254\u0250");
        assert_finish_ok!(d, "");

        // four bytes
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xf0, 0x90, 0x82, 0x82], [], "\U00010082");
        assert_feed_ok!(d, [], [], "");
        assert_finish_ok!(d, "");

        // we don't test encoders as it is largely a no-op.
    }

    #[test]
    fn test_valid_boundary() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0x00], [], "\x00");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0x7f], [], "\x7f");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xc2, 0x80], [], "\x80");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xdf, 0xbf], [], "\u07ff");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xe0, 0xa0, 0x80], [], "\u0800");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xed, 0x9f, 0xbf], [], "\ud7ff");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xee, 0x80, 0x80], [], "\ue000");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xef, 0xbf, 0xbf], [], "\uffff");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xf0, 0x90, 0x80, 0x80], [], "\U00010000");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xf4, 0x8f, 0xbf, 0xbf], [], "\U0010ffff");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_valid_partial() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [], [0xf0], "");
        assert_feed_ok!(d, [], [0x90], "");
        assert_feed_ok!(d, [], [0x82], "");
        assert_feed_ok!(d, [0x82], [0xed], "\U00010082");
        assert_feed_ok!(d, [0x92, 0x89], [], "\ud489");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [], [0xc2], "");
        assert_feed_ok!(d, [0xa9, 0x20], [], "\xa9\x20");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_invalid_continuation() {
        for c in range(0x80u8, 0xc0) {
            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [c], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [c, c], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_invalid_surrogate() {
        // surrogates should fail at the second byte.

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xa0, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xad, 0xbf], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xae, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xaf, 0xbf], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xb0, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xbe, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xed], [0xbf, 0xbf], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_invalid_boundary() {
        // as with surrogates, should fail at the second byte.
        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xf4], [0x90, 0x90, 0x90], ""); // U+110000
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_invalid_start_immediate_test_finish() {
        for c in range(0xf5u16, 0x100) {
            let c = c as u8;
            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_invalid_start_followed_by_space() {
        for c in range(0xf5u16, 0x100) {
            let c = c as u8;

            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [], "");
            assert_feed_ok!(d, [0x20], [], "\x20");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_invalid_lone_start_immediate_test_finish() {
        for c in range(0xc2u8, 0xf5) {
            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [c], ""); // wait for cont. bytes
            assert_finish_err!(d, "");
        }
    }

    #[test]
    fn test_invalid_lone_start_followed_by_space() {
        for c in range(0xc2u8, 0xf5) {
            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [c], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [c], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [], [0x20], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_invalid_incomplete_three_byte_seq_followed_by_space() {
        for b in range(0xe0u8, 0xf5) {
            let c = if b == 0xe0 || b == 0xf0 {0xa0} else {0x80};

            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [b, c], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [b, c], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [b], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [c], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [b], ""); // wait for cont. bytes
            assert_feed_ok!(d, [], [c], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [], [0x20], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_invalid_incomplete_four_byte_seq_followed_by_space() {
        for a in range(0xf0u8, 0xf5) {
            let b = if a == 0xf0 {0xa0} else {0x80};
            let c = 0x80;

            let mut d = UTF8Encoding.decoder();
            assert_feed_err!(d, [], [a, b, c], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [a], ""); // wait for cont. bytes
            assert_feed_ok!(d, [], [b], ""); // wait for cont. bytes
            assert_feed_ok!(d, [], [c], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [a, b], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [c], [0x20], "");
            assert_finish_ok!(d, "");

            let mut d = UTF8Encoding.decoder();
            assert_feed_ok!(d, [], [a, b, c], ""); // wait for cont. bytes
            assert_feed_err!(d, [], [], [0x20], "");
            assert_finish_ok!(d, "");
        }
    }

    #[test]
    fn test_invalid_too_many_cont_bytes() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [0xc2, 0x80], [0x80], [], "\x80");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [0xe0, 0xa0, 0x80], [0x80], [], "\u0800");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [0xf0, 0x90, 0x80, 0x80], [0x80], [], "\U00010000");
        assert_finish_ok!(d, "");

        // no continuation byte is consumed after 5/6-byte sequence starters and FE/FF
        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xf8], [0x88, 0x80, 0x80, 0x80, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xfc], [0x84, 0x80, 0x80, 0x80, 0x80, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xfe], [0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xff], [0x80], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_invalid_too_many_cont_bytes_partial() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [], [0xc2], "");
        assert_feed_err!(d, [0x80], [0x80], [], "\x80");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [], [0xe0, 0xa0], "");
        assert_feed_err!(d, [0x80], [0x80], [], "\u0800");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [], [0xf0, 0x90, 0x80], "");
        assert_feed_err!(d, [0x80], [0x80], [], "\U00010000");
        assert_finish_ok!(d, "");

        // no continuation byte is consumed after 5/6-byte sequence starters and FE/FF
        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xf8], [], "");
        assert_feed_err!(d, [], [0x88], [0x80, 0x80, 0x80, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xfc], [], "");
        assert_feed_err!(d, [], [0x84], [0x80, 0x80, 0x80, 0x80, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xfe], [], "");
        assert_feed_err!(d, [], [0x80], [], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xff], [], "");
        assert_feed_err!(d, [], [0x80], [], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_invalid_overlong_minimal() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xc0], [0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xe0], [0x80, 0x80], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xf0], [0x80, 0x80, 0x80], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_invalid_overlong_maximal() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xc1], [0xbf], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xe0], [0x9f, 0xbf], "");
        assert_finish_ok!(d, "");

        let mut d = UTF8Encoding.decoder();
        assert_feed_err!(d, [], [0xf0], [0x8f, 0xbf, 0xbf], "");
        assert_finish_ok!(d, "");
    }

    #[test]
    fn test_feed_after_finish() {
        let mut d = UTF8Encoding.decoder();
        assert_feed_ok!(d, [0xc2, 0x80], [0xc2], "\x80");
        assert_finish_err!(d, "");
        assert_feed_ok!(d, [0xc2, 0x80], [], "\x80");
        assert_finish_ok!(d, "");
    }
}


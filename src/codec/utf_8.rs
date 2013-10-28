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
 * Internal module for UTF-8 scanner.
 *
 * The algorithm for fast UTF-8 scanning is adopted from Bjoern Hoehrmann's [Flexible and Economical
 * UTF-8 Decoder](http://bjoern.hoehrmann.de/utf-8/decoder/dfa/). The main difference is that
 * we need to handle an invalid UTF-8 sequence, but the original algorithm only returns if
 * the entire string is valid or not. We use a number of invalid pseudo-states for this purpose.
 * We also keep an 8-byte window (the `queue`) so that the current sequence can be reconstructed
 * from the queue even we don't have the original buffer at our disposal.
 */
mod scan {
    use std::cast;
    use types::{StringWriter, CodecError};

    static CHAR_CATEGORY: [u8, ..256] = [
        //  0 (80-8F): continuation byte
        //  1 (90-9F): continuation byte
        //  2 (A0-BF): continuation byte
        //  3 (00-7F): one byte sequence
        //  4 (C0-C1): invalid (overlong) start of two byte sequence
        //  5 (C2-DF): start of two byte sequence
        //  6 (E0): start of three byte sequence, next byte restricted to non-overlong (A0-BF)
        //  7 (E1-EC,EE-EF): start of three byte sequence, next byte unrestricted
        //  8 (ED): start of three byte sequence, next byte restricted to non-surrogates (80-9F)
        //  9 (F0): start of four byte sequence, next byte restricted to non-overlong (90-BF)
        // 10 (F1-F3): start of four byte sequence, next byte unrestricted
        // 11 (F4): start of four byte sequence, next byte restricted to 0+10FFFF (80-8F)
        // 12 (F5-F7): invalid start of four byte sequence (exceeding 0+10FFFF)
        // 13 (F8-FB): invalid start of five byte sequence (exceeding 0+10FFFF)
        // 14 (FC-FD): invalid start of six byte sequence (exceeding 0+10FFFF)
        // 15 (FE-FF): invalid byte

        3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, 3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,
        3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, 3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,
        3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, 3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,
        3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, 3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, 1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
        2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, 2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
        4,4,5,5,5,5,5,5,5,5,5,5,5,5,5,5, 5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,
        6,7,7,7,7,7,7,7,7,7,7,7,7,8,7,7, 9,10,10,10,11,12,12,12,13,13,13,13,14,14,15,15,
    ];

    macro_rules! is_cont_byte(
        ($ch:expr) => (CHAR_CATEGORY[$ch as uint] < 3)
    )

    static S0: u8 = 0x08; static S1: u8 = 0x19; static S2: u8 = 0x29; static S3: u8 = 0x39;
    static S4: u8 = 0x49; static S5: u8 = 0x59; static S6: u8 = 0x69; static Sa: u8 = 0xB9;
    static Sb: u8 = 0xCA; static Sc: u8 = 0xDA; static Sd: u8 = 0xEB; static E1: u8 = 0xF1;
    static E2: u8 = 0x79; static E4: u8 = 0x89; static E5: u8 = 0x99; static E6: u8 = 0xA9;
    static XX: u8 = 0xFF;

    static STATE_TRANSITIONS: [u8, ..256] = [
         0,XX,XX,XX,XX,XX,XX,XX,                            // error states + a portion of E1
         1, 1, 1,S0,E2,Sa,S3,S1,S2,S6,S4,S5,E4,E5,E6,E1,XX, // S0 0x08: '??
        Sb,Sb,Sb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // S1 0x19: ss 'cc cc
        Sb,Sb, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // S2 0x29: ss 'cc(80-9F) cc
         2, 2,Sb, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // S3 0x39: ss 'cc(A0-BF) cc
        Sc,Sc,Sc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // S4 0x49: ss 'cc cc cc
        Sc, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // S5 0x59: ss 'cc(80-8F) cc cc
         3,Sc,Sc, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // S6 0x69: ss 'cc(90-BF) cc cc
         1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // E2 0x79: xx 'cc
         3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // E4 0x89: xx 'cc cc cc
         4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // E5 0x99: xx 'cc cc cc cc
         5, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // E6 0xA9: xx 'cc cc cc cc cc
        S0,S0,S0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,XX, // Sa 0xB9: ss 'cc
        S0,S0,S0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    // Sb 0xCA: ss cc 'cc
        Sd,Sd,Sd, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,XX, // Sc 0xDA: ss cc 'cc cc
        S0,S0,S0, 0, 0, 0,                                  // Sd 0xEB: ss cc cc 'cc
         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,       // E1 0xF1: xx '??
        //                                            0,    // ...overlaps with error states
    ];

    macro_rules! is_error_state(($state:expr) => ($state < S0))

    macro_rules! next_state(
        // the addition may overflow, but `STATE_TRANSITIONS` *does* take this into account.
        ($state:expr, $ch:expr) =>
            (STATE_TRANSITIONS[($state + CHAR_CATEGORY[$ch as uint]) as uint])
    )

    macro_rules! state_to_seqlen(($state:expr) => (($state & 7) as uint))

    static INITIAL_STATE: u8 = S0;

    /// UTF-8 scanner. Similar to UTF-8 encoder and decoder but accepts and returns a byte sequence.
    pub struct Scanner {
        /// The length of the queue.
        queuelen: uint,
        /// The last eight bytes of input. This may not get updated when the algorithm is in
        /// the valid state; when it turns to the invalid state the queue is reconstructed.
        queue: [u8, ..6],
        /// The current state. If this value is less than `S0`, it indicates the maximum number of
        /// continuation bytes acceptable for the current invalid sequence. The state cannot be 0;
        /// it is an intermediate state that immediately jumps to `S0` while finishing the current
        /// invalid sequence.
        state: u8,
    }

    impl Clone for Scanner {
        fn clone(&self) -> Scanner {
            Scanner { queuelen: self.queuelen, queue: self.queue, state: self.state }
        }
    }

    impl Scanner {
        pub fn new() -> Scanner {
            Scanner { queuelen: 0, queue: [0, ..6], state: INITIAL_STATE }
        }

        pub fn feed<'r>(&mut self, input: &'r [u8],
                        output: &mut StringWriter) -> Option<CodecError<&'r [u8],~[u8]>> {
            let mut queuelen = self.queuelen;
            let mut queue = self.queue;
            let mut state = self.state;
            let mut i = 0;
            let len = input.len();

            fn slice_as_str<'r>(v: &'r [u8]) -> &'r str { unsafe {cast::transmute(v)} }

            // valid states do not make the use of `queue` (so that the internal loop is tighter),
            // but it may contain the bytes from the prior sequence, so we first get rid of them.
            let validstart;
            if !is_error_state!(state) && queuelen > 0 {
                // `queue` is not empty, we proceed to the end of the current valid sequence.
                loop {
                    if i >= len { // save the queue.
                        self.queuelen = queuelen;
                        self.queue = queue;
                        self.state = state;
                        return None;
                    }

                    let ch = input[i];
                    state = next_state!(state, ch);
                    if is_error_state!(state) {
                        // `queue` holds what the invalid states expect
                        validstart = i;
                        break;
                    }
                    queue[queuelen] = ch;
                    queuelen += 1;
                    i += 1;
                    if state == INITIAL_STATE {
                        // we know the `queue` contains the entire valid sequence.
                        output.write_str(slice_as_str(queue.slice(0, queuelen)));
                        queuelen = 0;
                        validstart = i;
                        break;
                    }
                }
            } else {
                validstart = 0;
            }

            let invalidstart;
            if !is_error_state!(state) {
                // skip to the beginning of the current invalid sequence
                loop {
                    if i >= len { // no invalid sequence detected...
                        // ...but we still don't know if the current sequence is valid or not.
                        queuelen = state_to_seqlen!(state);
                        assert!(queuelen <= len);
                        for j in range(0, queuelen) {
                            queue[j] = input[len - queuelen + j];
                        }
                        self.queuelen = queuelen;
                        self.queue = queue;
                        self.state = state;
                        output.write_str(slice_as_str(input.slice(validstart, len - queuelen)));
                        return None;
                    }

                    let ch = input[i];
                    let oldstate = state;
                    state = next_state!(state, ch);
                    if is_error_state!(state) {
                        // `ch` might be the first byte of the next sequence, so we don't consume it
                        queuelen = state_to_seqlen!(oldstate);
                        assert!(queuelen <= i);
                        for j in range(0, queuelen) {
                            queue[j] = input[i - queuelen + j];
                        }
                        invalidstart = i - queuelen;
                        break;
                    }
                    i += 1;
                }
            } else {
                invalidstart = i;
            }

            // skip to the end of the current invalid sequence
            while state > 0 {
                if i >= len { // the current invalid sequence continues
                    self.queuelen = queuelen;
                    self.queue = queue;
                    self.state = state;
                    output.write_str(slice_as_str(input.slice(validstart, invalidstart)));
                    return None;
                }

                let ch = input[i];
                // do *not* consume the first byte of the next sequence!
                if !is_cont_byte!(ch) { break; }
                queue[queuelen] = ch;
                queuelen += 1;
                i += 1;
                state -= 1;
            }

            // the current invalid sequence finished, immediately switch to the initial state
            self.state = INITIAL_STATE;
            self.queuelen = 0;
            output.write_str(slice_as_str(input.slice(validstart, invalidstart)));
            Some(CodecError { remaining: input.slice(i, len),
                              problem: queue.slice(0, queuelen).to_owned(),
                              cause: "invalid byte sequence".into_send_str() })
        }

        pub fn finish(&mut self,
                      _output: &mut StringWriter) -> Option<CodecError<&'static [u8],~[u8]>> {
            let queuelen = self.queuelen;
            let queue = self.queue;
            let state = self.state;
            self.queuelen = 0;
            self.state = INITIAL_STATE;

            if state == INITIAL_STATE {
                None
            } else {
                let cause = if is_error_state!(state) {"invalid byte sequence"}
                                                 else {"incomplete byte sequence"};
                Some(CodecError { remaining: &[],
                                  problem: queue.slice(0, queuelen).to_owned(),
                                  cause: cause.into_send_str() })
            }
        }

        #[cfg(test)]
        pub fn test_feed<'r>(&mut self,
                             input: &'r [u8]) -> (~str, Option<CodecError<&'r [u8],~[u8]>>) {
            let mut output = ~"";
            let err = self.feed(input, &mut output as &mut StringWriter);
            (output, err)
        }

        #[cfg(test)]
        pub fn test_finish(&mut self) -> (~str, Option<CodecError<&'static [u8],~[u8]>>) {
            let mut output = ~"";
            let err = self.finish(&mut output as &mut StringWriter);
            (output, err)
        }
    }

    #[cfg(test)]
    mod tests {
        // portions of these tests are adopted from Markus Kuhn's UTF-8 decoder capability and
        // stress test: <http://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt>.

        use types::CodecError;
        use super::Scanner;

        fn strip_cause<T,Remaining,Problem>(result: (T,Option<CodecError<Remaining,Problem>>))
                                        -> (T,Option<(Problem,Remaining)>) {
            match result {
                (processed, None) => (processed, None),
                (processed, Some(CodecError { remaining, problem, cause: _cause })) =>
                    (processed, Some((problem, remaining)))
            }
        }

        macro_rules! assert_result(
            ($lhs:expr, $rhs:expr) => (assert_eq!(strip_cause($lhs), $rhs))
        )

        #[test]
        fn test_valid() {
            // one byte
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0x41]), (~"A", None));
            assert_result!(s.test_feed(&[0x42, 0x43]), (~"BC", None));
            assert_result!(s.test_feed(&[]), (~"", None));
            assert_result!(s.test_feed(&[0x44, 0x45, 0x46]), (~"DEF", None));
            assert_result!(s.test_finish(), (~"", None));

            // two bytes
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc2, 0xa2]), (~"\xa2", None));
            assert_result!(s.test_feed(&[0xc2, 0xac, 0xc2, 0xa9]), (~"\xac\xa9", None));
            assert_result!(s.test_feed(&[]), (~"", None));
            assert_result!(s.test_feed(&[0xd5, 0xa1, 0xd5, 0xb5, 0xd5, 0xa2, 0xd5, 0xb8, 0xd6, 0x82,
                                         0xd5, 0xa2, 0xd5, 0xa5, 0xd5, 0xb6]),
                           (~"\u0561\u0575\u0562\u0578\u0582\u0562\u0565\u0576", None));
            assert_result!(s.test_finish(), (~"", None));

            // three bytes
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0x92, 0x89]), (~"\ud489", None));
            assert_result!(s.test_feed(&[0xe6, 0xbc, 0xa2, 0xe5, 0xad, 0x97]),
                           (~"\u6f22\u5b57", None));
            assert_result!(s.test_feed(&[]), (~"", None));
            assert_result!(s.test_feed(&[0xc9, 0x99, 0xc9, 0x94, 0xc9, 0x90]),
                           (~"\u0259\u0254\u0250", None));
            assert_result!(s.test_finish(), (~"", None));

            // four bytes
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0, 0x90, 0x82, 0x82]), (~"\U00010082", None));
            assert_result!(s.test_feed(&[]), (~"", None));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_valid_boundary() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0x00]), (~"\x00", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0x7f]), (~"\x7f", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc2, 0x80]), (~"\x80", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xdf, 0xbf]), (~"\u07ff", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xe0, 0xa0, 0x80]), (~"\u0800", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0x9f, 0xbf]), (~"\ud7ff", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xee, 0x80, 0x80]), (~"\ue000", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xef, 0xbf, 0xbf]), (~"\uffff", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0, 0x90, 0x80, 0x80]), (~"\U00010000", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf4, 0x8f, 0xbf, 0xbf]), (~"\U0010ffff", None));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_valid_partial() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0]), (~"", None));
            assert_result!(s.test_feed(&[0x90]), (~"", None));
            assert_result!(s.test_feed(&[0x82]), (~"", None));
            assert_result!(s.test_feed(&[0x82, 0xed]), (~"\U00010082", None));
            assert_result!(s.test_feed(&[0x92, 0x89]), (~"\ud489", None));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc2]), (~"", None));
            assert_result!(s.test_feed(&[0xa9, 0x20]), (~"\xa9\x20", None));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_invalid_continuation() {
            for c in range(0x80u8, 0xc0) {
                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c]), (~"", Some((~[c], &[]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, c]), (~"", Some((~[c], &[c]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, c, c]), (~"", Some((~[c], &[c, c]))));
                assert_result!(s.test_finish(), (~"", None));
            }
        }

        #[test]
        fn test_invalid_surrogate() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xa0, 0x80]), (~"", Some((~[0xed, 0xa0, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xad, 0xbf]), (~"", Some((~[0xed, 0xad, 0xbf], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xae, 0x80]), (~"", Some((~[0xed, 0xae, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xaf, 0xbf]), (~"", Some((~[0xed, 0xaf, 0xbf], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xb0, 0x80]), (~"", Some((~[0xed, 0xb0, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xbe, 0x80]), (~"", Some((~[0xed, 0xbe, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xed, 0xbf, 0xbf]), (~"", Some((~[0xed, 0xbf, 0xbf], &[]))));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_invalid_boundary() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf4, 0x90, 0x90, 0x90]), // U+110000
                           (~"", Some((~[0xf4, 0x90, 0x90, 0x90], &[]))));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_invalid_start_immediate_test_finish() {
            for c in range(0xf5u16, 0x100) {
                let c = c as u8;

                let mut s = Scanner::new();
                // XXX invalid starts signals an error too late
                //assert_result!(s.test_feed(&[c]), (~"", Some((~[c], &[]))));
                //assert_result!(s.test_finish(), (~"", None));
                assert_result!(s.test_feed(&[c]), (~"", None));
                assert_result!(s.test_finish(), (~"", Some((~[c], &[]))));
            }
        }

        #[test]
        fn test_invalid_start_followed_by_space() {
            for c in range(0xf5u16, 0x100) {
                let c = c as u8;

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, 0x20]), (~"", Some((~[c], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                // XXX invalid starts signals an error too late
                //assert_result!(s.test_feed(&[c]), (~"", Some((~[c], &[]))));
                //assert_result!(s.test_feed(&[0x20]), (~[0x20], None));
                assert_result!(s.test_feed(&[c]), (~"", None));
                assert_result!(s.test_feed(&[0x20]), (~"", Some((~[c], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));
            }
        }

        #[test]
        fn test_invalid_lone_start_immediate_test_finish() {
            for c in range(0xc2u8, 0xf5) {
                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_finish(), (~"", Some((~[c], &[]))));
            }
        }

        #[test]
        fn test_invalid_lone_start_followed_by_space() {
            for c in range(0xc2u8, 0xf5) {
                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, 0x20]), (~"", Some((~[c], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[0x20]), (~"", Some((~[c], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));
            }
        }

        #[test]
        fn test_invalid_incomplete_three_byte_seq_followed_by_space() {
            for c in range(0xe0u8, 0xf5) {
                let d = if c == 0xe0 || c == 0xf0 {0xa0} else {0x80};

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, d, 0x20]), (~"", Some((~[c, d], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, d]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[0x20]), (~"", Some((~[c, d], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[d, 0x20]), (~"", Some((~[c, d], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[d]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[0x20]), (~"", Some((~[c, d], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));
            }
        }

        #[test]
        fn test_invalid_incomplete_four_byte_seq_followed_by_space() {
            for c in range(0xf0u8, 0xf5) {
                let d = if c == 0xf0 {0xa0} else {0x80};
                let e = 0x80;

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, d, e, 0x20]), (~"", Some((~[c, d, e], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[d]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[e]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[0x20]), (~"", Some((~[c, d, e], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, d]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[e, 0x20]), (~"", Some((~[c, d, e], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));

                let mut s = Scanner::new();
                assert_result!(s.test_feed(&[c, d, e]), (~"", None)); // wait for cont. bytes
                assert_result!(s.test_feed(&[0x20]), (~"", Some((~[c, d, e], &[0x20]))));
                assert_result!(s.test_finish(), (~"", None));
            }
        }

        #[test]
        fn test_invalid_too_many_cont_bytes() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc2, 0x80, 0x80]), (~"\x80", Some((~[0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xe0, 0xa0, 0x80, 0x80]),
                           (~"\u0800", Some((~[0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0, 0x90, 0x80, 0x80, 0x80]),
                           (~"\U00010000", Some((~[0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf8, 0x88, 0x80, 0x80, 0x80, 0x80]),
                           (~"", Some((~[0xf8, 0x88, 0x80, 0x80, 0x80], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xfc, 0x84, 0x80, 0x80, 0x80, 0x80, 0x80]),
                           (~"", Some((~[0xfc, 0x84, 0x80, 0x80, 0x80, 0x80], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));

            // no continuation byte is consumed after FE/FF
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xfe, 0x80]), (~"", Some((~[0xfe], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xff, 0x80]), (~"", Some((~[0xff], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_invalid_too_many_cont_bytes_partial() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc2]), (~"", None));
            assert_result!(s.test_feed(&[0x80, 0x80]), (~"\x80", Some((~[0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xe0, 0xa0]), (~"", None));
            assert_result!(s.test_feed(&[0x80, 0x80]), (~"\u0800", Some((~[0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0, 0x90, 0x80]), (~"", None));
            assert_result!(s.test_feed(&[0x80, 0x80]), (~"\U00010000", Some((~[0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf8, 0x88, 0x80, 0x80]), (~"", None));
            assert_result!(s.test_feed(&[0x80, 0x80]),
                           (~"", Some((~[0xf8, 0x88, 0x80, 0x80, 0x80], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xfc, 0x84, 0x80, 0x80, 0x80]), (~"", None));
            assert_result!(s.test_feed(&[0x80, 0x80]),
                           (~"", Some((~[0xfc, 0x84, 0x80, 0x80, 0x80, 0x80], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));

            // no continuation byte is consumed after FE/FF
            let mut s = Scanner::new();
            // XXX invalid starts signals an error too late
            //assert_result!(s.test_feed(&[0xfe]), (~"", Some((~[0xfe], &[]))));
            //assert_result!(s.test_feed(&[0x80]), (~"", Some((~[0x80], &[]))));
            assert_result!(s.test_feed(&[0xfe]), (~"", None));
            assert_result!(s.test_feed(&[0x80]), (~"", Some((~[0xfe], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            // XXX invalid starts signals an error too late
            //assert_result!(s.test_feed(&[0xff]), (~"", Some((~[0xff], &[]))));
            //assert_result!(s.test_feed(&[0x80]), (~"", Some((~[0x80], &[]))));
            assert_result!(s.test_feed(&[0xff]), (~"", None));
            assert_result!(s.test_feed(&[0x80]), (~"", Some((~[0xff], &[0x80]))));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_invalid_overlong_minimal() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc0, 0x80]), (~"", Some((~[0xc0, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xe0, 0x80, 0x80]), (~"", Some((~[0xe0, 0x80, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0, 0x80, 0x80, 0x80]),
                           (~"", Some((~[0xf0, 0x80, 0x80, 0x80], &[]))));
            assert_result!(s.test_finish(), (~"", None));
        }

        #[test]
        fn test_invalid_overlong_maximal() {
            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xc1, 0xbf]), (~"", Some((~[0xc1, 0xbf], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xe0, 0x9f, 0xbf]), (~"", Some((~[0xe0, 0x9f, 0xbf], &[]))));
            assert_result!(s.test_finish(), (~"", None));

            let mut s = Scanner::new();
            assert_result!(s.test_feed(&[0xf0, 0x8f, 0xbf, 0xbf]),
                           (~"", Some((~[0xf0, 0x8f, 0xbf, 0xbf], &[]))));
            assert_result!(s.test_finish(), (~"", None));
        }
    }
}

#[deriving(Clone)]
pub struct UTF8Encoding;

impl Encoding for UTF8Encoding {
    fn name(&self) -> &'static str { "utf-8" }
    fn encoder(&self) -> ~Encoder { ~UTF8Encoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~UTF8Decoder { scanner: scan::Scanner::new() } as ~Decoder }
}

#[deriving(Clone)]
pub struct UTF8Encoder;

impl Encoder for UTF8Encoder {
    fn encoding(&self) -> &'static Encoding { &UTF8Encoding as &'static Encoding }

    fn raw_feed<'r>(&mut self, input: &'r str,
                    output: &mut ByteWriter) -> Option<EncoderError<'r>> {
        unsafe {
            let input: &'r [u8] = cast::transmute(input);
            assert!(str::is_utf8(input));
            output.write_bytes(input);
        }
        None
    }

    fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct UTF8Decoder {
    scanner: scan::Scanner
}

impl Decoder for UTF8Decoder {
    fn encoding(&self) -> &'static Encoding { &UTF8Encoding as &'static Encoding }

    fn raw_feed<'r>(&mut self, input: &'r [u8],
                    output: &mut StringWriter) -> Option<DecoderError<'r>> {
        output.writer_hint(input.len());
        self.scanner.feed(input, output)
    }

    fn raw_finish(&mut self, output: &mut StringWriter) -> Option<DecoderError<'static>> {
        self.scanner.finish(output)
    }
}


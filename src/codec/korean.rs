// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy Korean encodings based on KS X 1001.

use std::str;
use util::StrCharIndex;
use index = index::euc_kr;
use types::*;

#[deriving(Clone)]
pub struct Windows949Encoding;

impl Encoding for Windows949Encoding {
    fn name(&self) -> &'static str { "windows-949" }
    fn encoder(&self) -> ~Encoder { ~Windows949Encoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~Windows949Decoder { lead: 0 } as ~Decoder }
}

#[deriving(Clone)]
pub struct Windows949Encoder;

impl Encoder for Windows949Encoder {
    fn encoding(&self) -> ~Encoding { ~Windows949Encoding as ~Encoding }

    fn feed<'r>(&mut self, input: &'r str, output: &mut ~[u8]) -> Option<EncoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut err = None;
        for input.index_iter().advance |((_,j), ch)| {
            if ch <= '\u007f' {
                output.push(ch as u8);
            } else {
                let ptr = index::backward(ch as u32);
                if ptr == 0xffff {
                    err = Some(CodecError {
                        remaining: input.slice_from(j),
                        problem: str::from_char(ch),
                        cause: ~"unrepresentable character",
                    });
                    break;
                } else if ptr < (26 + 26 + 126) * (0xc7 - 0x81) {
                    let lead = ptr / (26 + 26 + 126) + 0x81;
                    let trail = ptr % (26 + 26 + 126);
                    let offset = if trail < 26 {0x41} else if trail < 26 + 26 {0x47} else {0x4d};
                    output.push(lead as u8);
                    output.push((trail + offset) as u8);
                } else {
                    let ptr = ptr - (26 + 26 + 126) * (0xc7 - 0x81);
                    let lead = ptr / 94 + 0xc7;
                    let trail = ptr % 94 + 0xa1;
                    output.push(lead as u8);
                    output.push(trail as u8);
                }
            }
        }
        err
    }

    fn flush(&mut self, _output: &mut ~[u8]) -> Option<EncoderError<'static>> {
        None
    }
}

#[deriving(Clone)]
pub struct Windows949Decoder {
    lead: u8
}

impl Decoder for Windows949Decoder {
    fn encoding(&self) -> ~Encoding { ~Windows949Encoding as ~Encoding }

    fn feed<'r>(&mut self, input: &'r [u8], output: &mut ~str) -> Option<DecoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut i = 0;
        let len = input.len();

        if i < len && self.lead != 0 {
            let lead = self.lead as uint;
            let trail = input[i] as uint;
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
            match index::forward(index as u16) {
                0xffff => {
                    self.lead = 0;
                    let inclusive = (trail >= 0x80); // true if the trail byte is in the problem
                    return Some(CodecError {
                        remaining: input.slice(if inclusive {i+1} else {i}, len),
                        problem: if inclusive {~[lead as u8, trail as u8]} else {~[lead as u8]},
                        cause: ~"invalid sequence",
                    });
                }
                ch => { output.push_char(ch as char); }
            }
            i += 1;
        }

        self.lead = 0;
        while i < len {
            if input[i] < 0x80 {
                output.push_char(input[i] as char);
            } else {
                i += 1;
                if i >= len { // we wait for a trail byte even if the lead is obviously invalid
                    self.lead = input[i-1];
                    break;
                }

                let lead = input[i-1] as uint;
                let trail = input[i] as uint;
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
                match index::forward(index as u16) {
                    0xffff => {
                        let inclusive = (trail >= 0x80); // true if the trail byte is in the problem
                        return Some(CodecError {
                            remaining: input.slice(if inclusive {i+1} else {i}, len),
                            problem: if inclusive {~[lead as u8, trail as u8]}
                                             else {~[lead as u8]},
                            cause: ~"invalid sequence",
                        });
                    }
                    ch => { output.push_char(ch as char); }
                }
            }
            i += 1;
        }
        None
    }

    fn flush(&mut self, _output: &mut ~str) -> Option<DecoderError<'static>> {
        if self.lead != 0 {
            Some(CodecError { remaining: &[],
                              problem: ~[self.lead],
                              cause: ~"incomplete sequence" })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod euckr_tests {
    use std::u16;
    use super::Windows949Encoding;
    use types::*;

    fn strip_cause<T,Remaining,Problem>(result: (T,Option<CodecError<Remaining,Problem>>))
                                    -> (T,Option<(Remaining,Problem)>) {
        match result {
            (processed, None) => (processed, None),
            (processed, Some(CodecError { remaining, problem, cause: _cause })) =>
                (processed, Some((remaining, problem)))
        }
    }

    macro_rules! assert_result(
        ($lhs:expr, $rhs:expr) => (assert_eq!(strip_cause($lhs), $rhs))
    )

    #[test]
    fn test_encoder_valid() {
        let mut e = Windows949Encoding.encoder();
        assert_result!(e.test_feed("A"), (~[0x41], None));
        assert_result!(e.test_feed("BC"), (~[0x42, 0x43], None));
        assert_result!(e.test_feed(""), (~[], None));
        assert_result!(e.test_feed("\uac00"), (~[0xb0, 0xa1], None));
        assert_result!(e.test_feed("\ub098\ub2e4"), (~[0xb3, 0xaa, 0xb4, 0xd9], None));
        assert_result!(e.test_feed("\ubdc1\u314b\ud7a3"), (~[0x94, 0xee, 0xa4, 0xbb, 0xc6, 0x52], None));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = Windows949Encoding.encoder();
        assert_result!(e.test_feed("\uffff"), (~[], Some(("", ~"\uffff"))));
        assert_result!(e.test_feed("?\uffff!"), (~[0x3f], Some(("!", ~"\uffff"))));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = Windows949Encoding.decoder();
        assert_result!(d.test_feed(&[0x41]), (~"A", None));
        assert_result!(d.test_feed(&[0x42, 0x43]), (~"BC", None));
        assert_result!(d.test_feed(&[]), (~"", None));
        assert_result!(d.test_feed(&[0xb0, 0xa1]), (~"\uac00", None));
        assert_result!(d.test_feed(&[0xb3, 0xaa, 0xb4, 0xd9]), (~"\ub098\ub2e4", None));
        assert_result!(d.test_feed(&[0x94, 0xee, 0xa4, 0xbb, 0xc6, 0x52]),
                       (~"\ubdc1\u314b\ud7a3", None));
        assert_result!(d.test_flush(), (~"", None));
    }

    #[test]
    fn test_decoder_valid_partial() {
        let mut d = Windows949Encoding.decoder();
        assert_result!(d.test_feed(&[0xb0]), (~"", None));
        assert_result!(d.test_feed(&[0xa1]), (~"\uac00", None));
        assert_result!(d.test_feed(&[0xb3, 0xaa, 0xb4]), (~"\ub098", None));
        assert_result!(d.test_feed(&[0xd9, 0x94]), (~"\ub2e4", None));
        assert_result!(d.test_feed(&[0xee, 0xa4, 0xbb, 0xc6, 0x52]), (~"\ubdc1\u314b\ud7a3", None));
        assert_result!(d.test_flush(), (~"", None));
    }

    #[test]
    fn test_decoder_invalid_lone_lead_immediate_test_flush() {
        for u16::range(0x80, 0x100) |i| {
            let i = i as u8;
            let mut d = Windows949Encoding.decoder();
            assert_result!(d.test_feed(&[i]), (~"", None)); // wait for a trail
            assert_result!(d.test_flush(), (~"", Some((&[], ~[i]))));
        }
    }

    #[test]
    fn test_decoder_invalid_lone_lead_followed_by_space() {
        for u16::range(0x80, 0x100) |i| {
            let i = i as u8;
            let mut d = Windows949Encoding.decoder();
            assert_result!(d.test_feed(&[i, 0x20]), (~"", Some((&[0x20], ~[i]))));
            assert_result!(d.test_flush(), (~"", None));
        }
    }

    #[test]
    fn test_decoder_invalid_lead_followed_by_invalid_trail() {
        for u16::range(0x80, 0x100) |i| {
            let i = i as u8;
            let mut d = Windows949Encoding.decoder();
            assert_result!(d.test_feed(&[i, 0x80]), (~"", Some((&[], ~[i, 0x80]))));
            assert_result!(d.test_feed(&[i, 0xff]), (~"", Some((&[], ~[i, 0xff]))));
            assert_result!(d.test_flush(), (~"", None));
        }
    }

    #[test]
    fn test_decoder_invalid_boundary() {
        // U+D7A3 (C6 52) is the last Hangul syllable not in KS X 1001, C6 53 is invalid.
        // note that since the trail byte may coincide with ASCII, the trail byte 53 is
        // not considered to be in the problem. this behavior is intentional.
        let mut d = Windows949Encoding.decoder();
        assert_result!(d.test_feed(&[0xc6]), (~"", None));
        assert_result!(d.test_feed(&[0x53]), (~"", Some((&[0x53], ~[0xc6]))));
        assert_result!(d.test_flush(), (~"", None));
    }
}


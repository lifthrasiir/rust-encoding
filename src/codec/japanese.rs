// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Legacy Japanese encodings based on JIS X 0208 and JIS X 0212.

use std::str;
use util::StrCharIndex;
use index0208 = index::jis0208;
use index0212 = index::jis0212;
use types::*;

#[deriving(Clone)]
pub struct EUCJPEncoding;

impl Encoding for EUCJPEncoding {
    fn name(&self) -> &'static str { "euc-jp" }
    fn encoder(&self) -> ~Encoder { ~EUCJPEncoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~EUCJPDecoder { first: 0, second: 0 } as ~Decoder }
}

#[deriving(Clone)]
pub struct EUCJPEncoder;

impl Encoder for EUCJPEncoder {
    fn encoding(&self) -> &'static Encoding { &EUCJPEncoding as &'static Encoding }

    fn feed<'r>(&mut self, input: &'r str, output: &mut ~[u8]) -> Option<EncoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut err = None;
        for ((_,j), ch) in input.index_iter() {
            match ch {
                '\u0000'..'\u007f' => { output.push(ch as u8); }
                '\u00a5' => { output.push(0x5c); }
                '\u203e' => { output.push(0x7e); }
                '\uff61'..'\uff9f' => {
                    output.push(0x8e);
                    output.push((ch as uint - 0xff61 + 0xa1) as u8);
                }
                _ => {
                    let ptr = index0208::backward(ch as u32);
                    if ptr == 0xffff {
                        err = Some(CodecError {
                            remaining: input.slice_from(j),
                            problem: str::from_char(ch),
                            cause: ~"unrepresentable character",
                        });
                        break;
                    } else {
                        let lead = ptr / 94 + 0xa1;
                        let trail = ptr % 94 + 0xa1;
                        output.push(lead as u8);
                        output.push(trail as u8);
                    }
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
pub struct EUCJPDecoder {
    first: u8,
    second: u8,
}

impl Decoder for EUCJPDecoder {
    fn encoding(&self) -> &'static Encoding { &EUCJPEncoding as &'static Encoding }

    fn feed<'r>(&mut self, input: &'r [u8], output: &mut ~str) -> Option<DecoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut i = 0;
        let len = input.len();

        if i < len && self.first != 0 {
            let lead = self.first as uint;
            let trail = input[i] as uint;
            match (lead, trail) {
                (0x8e, 0xa1..0xdf) => {
                    output.push_char((0xff61 + trail - 0xa1) as char);
                }
                (0x8f, _) => {
                    self.first = 0;
                    self.second = trail as u8;
                    // pass through
                }
                (_, _) => {
                    let index = match (lead, trail) {
                        (0xa1..0xfe, 0xa1..0xfe) => (lead - 0xa1) * 94 + trail - 0xa1,
                        _ => 0xffff,
                    };
                    match index0208::forward(index as u16) {
                        0xffff => {
                            self.first = 0;
                            let inclusive = (trail >= 0x80);
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
            }
            i += 1;
        }

        if i < len && self.second != 0 {
            let trail = self.second as uint;
            let byte = input[i] as uint;
            let index = match (trail, byte) {
                (0xa1..0xfe, 0xa1..0xfe) => (trail - 0xa1) * 94 + byte - 0xa1,
                _ => 0xffff,
            };
            match index0212::forward(index as u16) {
                0xffff => {
                    self.second = 0;
                    let inclusive = (byte >= 0x80);
                    return Some(CodecError {
                        remaining: input.slice(if inclusive {i+1} else {i}, len),
                        problem: if inclusive {~[0x8f, trail as u8, byte as u8]}
                                         else {~[0x8f, trail as u8]},
                        cause: ~"invalid sequence",
                    });
                }
                ch => { output.push_char(ch as char); }
            }
            i += 1;
        }

        self.first = 0;
        self.second = 0;
        while i < len {
            if input[i] < 0x80 {
                output.push_char(input[i] as char);
            } else {
                i += 1;
                if i >= len { // we wait for a trail byte even if the lead is obviously invalid
                    self.first = input[i-1];
                    break;
                }

                let lead = input[i-1] as uint;
                let trail = input[i] as uint;
                match (lead, trail) {
                    (0x8e, 0xa1..0xdf) => {
                        output.push_char((0xff61 + trail - 0xa1) as char);
                    }
                    (0x8f, _) => { // JIS X 0212 three-byte sequence
                        i += 1;
                        if i >= len { // again, we always wait for the third byte
                            self.second = trail as u8;
                            break;
                        }
                        let byte = input[i] as uint;
                        let index = match (trail, byte) {
                            (0xa1..0xfe, 0xa1..0xfe) => (trail - 0xa1) * 94 + byte - 0xa1,
                            _ => 0xffff,
                        };
                        match index0212::forward(index as u16) {
                            0xffff => {
                                let inclusive = (byte >= 0x80);
                                return Some(CodecError {
                                    remaining: input.slice(if inclusive {i+1} else {i}, len),
                                    problem: if inclusive {~[0x8f, trail as u8, byte as u8]}
                                                     else {~[0x8f, trail as u8]},
                                    cause: ~"invalid sequence",
                                });
                            }
                            ch => { output.push_char(ch as char); }
                        }
                    }
                    (_, _) => {
                        let index = match (lead, trail) {
                            (0xa1..0xfe, 0xa1..0xfe) => (lead - 0xa1) * 94 + trail - 0xa1,
                            _ => 0xffff,
                        };
                        match index0208::forward(index as u16) {
                            0xffff => {
                                let inclusive = (trail >= 0x80);
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
                }
            }
            i += 1;
        }
        None
    }

    fn flush(&mut self, _output: &mut ~str) -> Option<DecoderError<'static>> {
        if self.second != 0 {
            Some(CodecError { remaining: &[],
                              problem: ~[0x8f, self.second],
                              cause: ~"incomplete sequence" })
        } else if self.first != 0 {
            Some(CodecError { remaining: &[],
                              problem: ~[self.first],
                              cause: ~"incomplete sequence" })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod eucjp_tests {
    use super::EUCJPEncoding;
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
        let mut e = EUCJPEncoding.encoder();
        assert_result!(e.test_feed("A"), (~[0x41], None));
        assert_result!(e.test_feed("BC"), (~[0x42, 0x43], None));
        assert_result!(e.test_feed(""), (~[], None));
        assert_result!(e.test_feed("\u00a5"), (~[0x5c], None));
        assert_result!(e.test_feed("\u203e"), (~[0x7e], None));
        assert_result!(e.test_feed("\u306b\u307b\u3093"), (~[0xa4, 0xcb, 0xa4, 0xdb, 0xa4, 0xf3], None));
        assert_result!(e.test_feed("\uff86\uff8e\uff9d"), (~[0x8e, 0xc6, 0x8e, 0xce, 0x8e, 0xdd], None));
        assert_result!(e.test_feed("\u65e5\u672c"), (~[0xc6, 0xfc, 0xcb, 0xdc], None));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = EUCJPEncoding.encoder();
        assert_result!(e.test_feed("\uffff"), (~[], Some(("", ~"\uffff"))));
        assert_result!(e.test_feed("?\uffff!"), (~[0x3f], Some(("!", ~"\uffff"))));
        // JIS X 0212 is not supported in the encoder
        assert_result!(e.test_feed("\u736c\u8c78"), (~[], Some(("\u8c78", ~"\u736c"))));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = EUCJPEncoding.decoder();
        assert_result!(d.test_feed(&[0x41]), (~"A", None));
        assert_result!(d.test_feed(&[0x42, 0x43]), (~"BC", None));
        assert_result!(d.test_feed(&[]), (~"", None));
        assert_result!(d.test_feed(&[0x5c]), (~"\\", None));
        assert_result!(d.test_feed(&[0x7e]), (~"~", None));
        assert_result!(d.test_feed(&[0xa4, 0xcb, 0xa4, 0xdb, 0xa4, 0xf3]),
                       (~"\u306b\u307b\u3093", None));
        assert_result!(d.test_feed(&[0x8e, 0xc6, 0x8e, 0xce, 0x8e, 0xdd]),
                       (~"\uff86\uff8e\uff9d", None));
        assert_result!(d.test_feed(&[0xc6, 0xfc, 0xcb, 0xdc]), (~"\u65e5\u672c", None));
        assert_result!(d.test_feed(&[0x8f, 0xcb, 0xc6, 0xec, 0xb8]), (~"\u736c\u8c78", None));
        assert_result!(d.test_flush(), (~"", None));
    }

    // TODO more tests
}

#[deriving(Clone)]
pub struct ShiftJISEncoding;

impl Encoding for ShiftJISEncoding {
    fn name(&self) -> &'static str { "shift-jis" }
    fn encoder(&self) -> ~Encoder { ~ShiftJISEncoder as ~Encoder }
    fn decoder(&self) -> ~Decoder { ~ShiftJISDecoder { lead: 0 } as ~Decoder }
}

#[deriving(Clone)]
pub struct ShiftJISEncoder;

impl Encoder for ShiftJISEncoder {
    fn encoding(&self) -> &'static Encoding { &ShiftJISEncoding as &'static Encoding }

    fn feed<'r>(&mut self, input: &'r str, output: &mut ~[u8]) -> Option<EncoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut err = None;
        for ((_,j), ch) in input.index_iter() {
            match ch {
                '\u0000'..'\u0080' => { output.push(ch as u8); }
                '\u00a5' => { output.push(0x5c); }
                '\u203e' => { output.push(0x7e); }
                '\uff61'..'\uff9f' => { output.push((ch as uint - 0xff61 + 0xa1) as u8); }
                _ => {
                    let ptr = index0208::backward(ch as u32);
                    if ptr == 0xffff {
                        err = Some(CodecError {
                            remaining: input.slice_from(j),
                            problem: str::from_char(ch),
                            cause: ~"unrepresentable character",
                        });
                        break;
                    } else {
                        let lead = ptr / 188;
                        let leadoffset = if lead < 0x1f {0x81} else {0xc1};
                        let trail = ptr % 188;
                        let trailoffset = if trail < 0x3f {0x40} else {0x41};
                        output.push((lead + leadoffset) as u8);
                        output.push((trail + trailoffset) as u8);
                    }
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
pub struct ShiftJISDecoder {
    lead: u8
}

impl Decoder for ShiftJISDecoder {
    fn encoding(&self) -> &'static Encoding { &ShiftJISEncoding as &'static Encoding }

    fn feed<'r>(&mut self, input: &'r [u8], output: &mut ~str) -> Option<DecoderError<'r>> {
        { let new_len = output.len() + input.len(); output.reserve_at_least(new_len) }
        let mut i = 0;
        let len = input.len();

        if i < len && self.lead != 0 {
            let lead = self.lead as uint;
            let trail = input[i] as uint;
            let index = match (lead, trail) {
                (0x81..0x9f, 0x40..0x7e) | (0x81..0x9f, 0x80..0xfc) |
                (0xe0..0xfc, 0x40..0x7e) | (0xe0..0xfc, 0x80..0xfc) => {
                    let leadoffset = if lead < 0xa0 {0x81} else {0xc1};
                    let trailoffset = if trail < 0x7f {0x40} else {0x41};
                    (lead - leadoffset) * 188 + trail - trailoffset
                }
                _ => 0xffff,
            };
            match index0208::forward(index as u16) {
                0xffff => {
                    self.lead = 0;
                    let inclusive = (trail >= 0x80);
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
            match input[i] {
                0x00..0x7f => {
                    output.push_char(input[i] as char);
                }
                0xa1..0xdf => {
                    output.push_char((0xff61 + (input[i] as uint) - 0xa1) as char);
                }
                _ => {
                    i += 1;
                    if i >= len { // we wait for a trail byte even if the lead is obviously invalid
                        self.lead = input[i-1];
                        break;
                    }

                    let lead = input[i-1] as uint;
                    let trail = input[i] as uint;
                    let index = match (lead, trail) {
                        (0x81..0x9f, 0x40..0x7e) | (0x81..0x9f, 0x80..0xfc) |
                        (0xe0..0xfc, 0x40..0x7e) | (0xe0..0xfc, 0x80..0xfc) => {
                            let leadoffset = if lead < 0xa0 {0x81} else {0xc1};
                            let trailoffset = if trail < 0x7f {0x40} else {0x41};
                            (lead - leadoffset) * 188 + trail - trailoffset
                        }
                        _ => 0xffff,
                    };
                    match index0208::forward(index as u16) {
                        0xffff => {
                            let inclusive = (trail >= 0x80);
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
mod shiftjis_tests {
    use super::ShiftJISEncoding;
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
        let mut e = ShiftJISEncoding.encoder();
        assert_result!(e.test_feed("A"), (~[0x41], None));
        assert_result!(e.test_feed("BC"), (~[0x42, 0x43], None));
        assert_result!(e.test_feed(""), (~[], None));
        assert_result!(e.test_feed("\u00a5"), (~[0x5c], None));
        assert_result!(e.test_feed("\u203e"), (~[0x7e], None));
        assert_result!(e.test_feed("\u306b\u307b\u3093"), (~[0x82, 0xc9, 0x82, 0xd9, 0x82, 0xf1], None));
        assert_result!(e.test_feed("\uff86\uff8e\uff9d"), (~[0xc6, 0xce, 0xdd], None));
        assert_result!(e.test_feed("\u65e5\u672c"), (~[0x93, 0xfa, 0x96, 0x7b], None));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_encoder_invalid() {
        let mut e = ShiftJISEncoding.encoder();
        assert_result!(e.test_feed("\uffff"), (~[], Some(("", ~"\uffff"))));
        assert_result!(e.test_feed("?\uffff!"), (~[0x3f], Some(("!", ~"\uffff"))));
        assert_result!(e.test_feed("\u736c\u8c78"), (~[], Some(("\u8c78", ~"\u736c"))));
        assert_result!(e.test_flush(), (~[], None));
    }

    #[test]
    fn test_decoder_valid() {
        let mut d = ShiftJISEncoding.decoder();
        assert_result!(d.test_feed(&[0x41]), (~"A", None));
        assert_result!(d.test_feed(&[0x42, 0x43]), (~"BC", None));
        assert_result!(d.test_feed(&[]), (~"", None));
        assert_result!(d.test_feed(&[0x5c]), (~"\\", None));
        assert_result!(d.test_feed(&[0x7e]), (~"~", None));
        assert_result!(d.test_feed(&[0x82, 0xc9, 0x82, 0xd9, 0x82, 0xf1]),
                       (~"\u306b\u307b\u3093", None));
        assert_result!(d.test_feed(&[0xc6, 0xce, 0xdd]), (~"\uff86\uff8e\uff9d", None));
        assert_result!(d.test_feed(&[0x93, 0xfa, 0x96, 0x7b]), (~"\u65e5\u672c", None));
        assert_result!(d.test_flush(), (~"", None));
    }

    // TODO more tests
}


// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * Interface to the character encoding.
 *
 * # Raw incremental interface
 *
 * Methods which name starts with `raw_` constitute the raw incremental interface,
 * the lowest-available API for encoders and decoders.
 * This interface divides the entire input to four parts:
 *
 * - **Processed** bytes do not affect the future result.
 * - **Unprocessed** bytes may affect the future result
 *   and can be a part of problematic sequence according to the future input.
 * - **Problematic** byte is the first byte that causes an error condition.
 * - **Remaining** bytes are not yet processed nor read,
 *   so the caller should feed any remaining bytes again.
 *
 * The following figure illustrates an example of successive `raw_feed` calls:
 *
 * ````notrust
 * 1st raw_feed   :2nd raw_feed   :3rd raw_feed
 * ----------+----:---------------:--+--+---------
 *           |    :               :  |  |
 * ----------+----:---------------:--+--+---------
 * processed  unprocessed             |  remaining
 *                               problematic
 * ````
 *
 * Since these parts can span the multiple input sequences to `raw_feed`,
 * `raw_feed` returns two offsets (one optional)
 * with that the caller can track the problematic sequence.
 * The first offset (the first `uint` in the tuple) points to the first unprocessed bytes,
 * or is zero when unprocessed bytes have started before the current call.
 * (The first unprocessed byte can also be at offset 0,
 * which doesn't make a difference for the caller.)
 * The second offset (`upto` field in the `CodecError` struct), if any,
 * points to the first remaining bytes.
 *
 * If the caller needs to recover the error via the problematic sequence,
 * then the caller starts to save the unprocessed bytes when the first offset < the input length,
 * appends any new unprocessed bytes while the first offset is zero,
 * and discards unprocessed bytes when first offset becomes non-zero
 * while saving new unprocessed bytes when the first offset < the input length.
 * Then the caller checks for the error condition
 * and can use the saved unprocessed bytes for error recovery.
 * Alternatively, if the caller only wants to replace the problematic sequence
 * with a fixed string (like U+FFFD),
 * then it can just discard the first sequence and can emit the fixed string on an error.
 * It still has to feed the input bytes starting at the second offset again.
 */

use std::str::SendStr;

/// Error information from either encoder or decoder.
#[experimental]
pub struct CodecError {
    /// The byte position of the first remaining byte, which is next to the problematic byte.
    /// The caller should feed the bytes starting from this point again
    /// in order to continue encoding or decoding after an error.
    /// This value is always set to 0 for `finish`.
    pub upto: uint,
    /// A human-readable cause of the error.
    pub cause: SendStr,
}

/// Byte writer used by `Encoder`s. In most cases this will be an owned vector of `u8`.
#[unstable]
pub trait ByteWriter {
    /// Hints an expected lower bound on the length (in bytes) of the output
    /// until the next call to `writer_hint`,
    /// so that the writer can reserve the memory for writing.
    /// `Encoder`s are recommended but not required to call this method
    /// with an appropriate estimate.
    /// By default this method does nothing.
    fn writer_hint(&mut self, _expectedlen: uint) {}

    /// Writes a single byte.
    fn write_byte(&mut self, b: u8);

    /// Writes a number of bytes.
    fn write_bytes(&mut self, v: &[u8]);
}

impl ByteWriter for Vec<u8> {
    fn writer_hint(&mut self, expectedlen: uint) {
        self.reserve_additional(expectedlen);
    }

    fn write_byte(&mut self, b: u8) {
        self.push(b);
    }

    fn write_bytes(&mut self, v: &[u8]) {
        self.push_all(v);
    }
}

/// String writer used by `Decoder`s. In most cases this will be an owned string.
#[unstable]
pub trait StringWriter {
    /// Hints an expected lower bound on the length (in bytes) of the output
    /// until the next call to `writer_hint`,
    /// so that the writer can reserve the memory for writing.
    /// `Decoder`s are recommended but not required to call this method
    /// with an appropriate estimate.
    /// By default this method does nothing.
    fn writer_hint(&mut self, _expectedlen: uint) {}

    /// Writes a single character.
    fn write_char(&mut self, c: char);

    /// Writes a string.
    fn write_str(&mut self, s: &str);
}

impl StringWriter for String {
    fn writer_hint(&mut self, expectedlen: uint) {
        let newlen = self.len() + expectedlen;
        self.reserve(newlen);
    }

    fn write_char(&mut self, c: char) {
        self.push_char(c);
    }

    fn write_str(&mut self, s: &str) {
        self.push_str(s);
    }
}

/// Encoder converting a Unicode string into a byte sequence.
/// This is a lower level interface, and normally `Encoding::encode` should be used instead.
#[experimental]
pub trait Encoder {
    /// Creates a fresh `Encoder` instance which parameters are same as `self`.
    fn from_self(&self) -> Box<Encoder>;

    /// Returns true if this encoding is compatible to ASCII,
    /// i.e. U+0000 through U+007F always map to bytes 00 through 7F and nothing else.
    fn is_ascii_compatible(&self) -> bool { false }

    /// Feeds given portion of string to the encoder,
    /// pushes the an encoded byte sequence at the end of the given output,
    /// and returns a byte offset to the first unprocessed character
    /// (that can be zero when the first such character appeared in the prior calls to `raw_feed`)
    /// and optional error information (None means success).
    fn raw_feed(&mut self, input: &str, output: &mut ByteWriter) -> (uint, Option<CodecError>);

    /// Finishes the encoder,
    /// pushes the an encoded byte sequence at the end of the given output,
    /// and returns optional error information (None means success).
    /// `remaining` value of the error information, if any, is always an empty string.
    fn raw_finish(&mut self, output: &mut ByteWriter) -> Option<CodecError>;

    /// Normalizes the input for testing. Internal use only.
    #[cfg(test)]
    fn test_norm_input<'r>(&self, input: &'r str) -> &'r str { input }

    /// Normalizes the output for testing. Internal use only.
    #[cfg(test)]
    fn test_norm_output<'r>(&self, output: &'r [u8]) -> &'r [u8] { output }

    /// A test-friendly interface to `raw_feed`. Internal use only.
    #[cfg(test)]
    fn test_feed(&mut self, input: &str) -> (uint, Option<CodecError>, Vec<u8>) {
        let mut buf = Vec::new();
        let (nprocessed, err) = self.raw_feed(input, &mut buf);
        (nprocessed, err, buf)
    }

    /// A test-friendly interface to `raw_finish`. Internal use only.
    #[cfg(test)]
    fn test_finish(&mut self) -> (Option<CodecError>, Vec<u8>) {
        let mut buf = Vec::new();
        let err = self.raw_finish(&mut buf);
        (err, buf)
    }

    /// Concatenates two input sequences into one. Internal use only.
    #[cfg(test)]
    fn test_concat(&self, a: &str, b: &str) -> String { a.to_string().append(b) }
}

/// Encoder converting a byte sequence into a Unicode string.
/// This is a lower level interface, and normally `Encoding::decode` should be used instead.
#[experimental]
pub trait Decoder {
    /// Creates a fresh `Decoder` instance which parameters are same as `self`.
    fn from_self(&self) -> Box<Decoder>;

    /// Returns true if this encoding is compatible to ASCII,
    /// i.e. bytes 00 through 7F always map to U+0000 through U+007F and nothing else.
    fn is_ascii_compatible(&self) -> bool { false }

    /// Feeds given portion of byte sequence to the encoder,
    /// pushes the a decoded string at the end of the given output,
    /// and returns an offset to the first unprocessed byte
    /// (that can be zero when the first such byte appeared in the prior calls to `raw_feed`)
    /// and optional error information (None means success).
    fn raw_feed(&mut self, input: &[u8], output: &mut StringWriter) -> (uint, Option<CodecError>);

    /// Finishes the decoder,
    /// pushes the a decoded string at the end of the given output,
    /// and returns optional error information (None means success).
    /// `upto` value of the error information, if any, is always zero.
    fn raw_finish(&mut self, output: &mut StringWriter) -> Option<CodecError>;

    /// Normalizes the input for testing. Internal use only.
    #[cfg(test)]
    fn test_norm_input<'r>(&self, input: &'r [u8]) -> &'r [u8] { input }

    /// Normalizes the output for testing. Internal use only.
    #[cfg(test)]
    fn test_norm_output<'r>(&self, output: &'r str) -> &'r str { output }

    /// A test-friendly interface to `raw_feed`. Internal use only.
    #[cfg(test)]
    fn test_feed(&mut self, input: &[u8]) -> (uint, Option<CodecError>, String) {
        let mut buf = String::new();
        let (nprocessed, err) = self.raw_feed(input, &mut buf);
        (nprocessed, err, buf)
    }

    /// A test-friendly interface to `raw_finish`. Internal use only.
    #[cfg(test)]
    fn test_finish(&mut self) -> (Option<CodecError>, String) {
        let mut buf = String::new();
        let err = self.raw_finish(&mut buf);
        (err, buf)
    }

    /// Concatenates two input sequences into one. Internal use only.
    #[cfg(test)]
    fn test_concat(&self, a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(a.len() + b.len());
        v.push_all(a);
        v.push_all(b);
        v
    }
}

/// A trait object using dynamic dispatch which is a sendable reference to the encoding,
/// for code where the encoding is not known at compile-time.
#[stable]
pub type EncodingRef = &'static Encoding+Send;

/// Character encoding.
#[stable]
pub trait Encoding {
    /// Returns the canonical name of given encoding.
    /// This name is guaranteed to be unique across built-in encodings,
    /// but it is not normative and would be at most arbitrary.
    #[stable]
    fn name(&self) -> &'static str;

    /// Returns a name of given encoding defined in the WHATWG Encoding standard, if any.
    /// This name often differs from `name` due to the compatibility reason.
    #[unstable]
    fn whatwg_name(&self) -> Option<&'static str> { None }

    /// Creates a new encoder.
    #[experimental]
    fn encoder(&'static self) -> Box<Encoder>;

    /// Creates a new decoder.
    #[experimental]
    fn decoder(&'static self) -> Box<Decoder>;

    /// An easy-to-use interface to `Encoder`.
    /// On the encoder error `trap` is called,
    /// which may return a replacement sequence to continue processing,
    /// or a failure to return the error.
    #[stable]
    fn encode(&'static self, input: &str, trap: EncoderTrap) -> Result<Vec<u8>,SendStr> {
        let mut encoder = self.encoder();
        let mut remaining = input;
        let mut unprocessed = String::new();
        let mut ret = Vec::new();

        loop {
            let (offset, err) = encoder.raw_feed(remaining, &mut ret);
            if offset > 0 { unprocessed.truncate(0); }
            match err {
                Some(err) => {
                    unprocessed.push_str(remaining.slice(offset, err.upto));
                    if !trap.trap(encoder, unprocessed.as_slice(), &mut ret) {
                        return Err(err.cause);
                    }
                    unprocessed.truncate(0);
                    remaining = remaining.slice(err.upto, remaining.len());
                }
                None => {
                    unprocessed.push_str(remaining.slice(offset, remaining.len()));
                    break
                }
            }
        }

        match encoder.raw_finish(&mut ret) {
            Some(err) => {
                if !trap.trap(encoder, unprocessed.as_slice(), &mut ret) {
                    return Err(err.cause);
                }
            }
            None => {}
        }
        Ok(ret)
    }

    /// An easy-to-use interface to `Decoder`.
    /// On the decoder error `trap` is called,
    /// which may return a replacement string to continue processing,
    /// or a failure to return the error.
    #[stable]
    fn decode(&'static self, input: &[u8], trap: DecoderTrap) -> Result<String,SendStr> {
        let mut decoder = self.decoder();
        let mut remaining = input;
        let mut unprocessed = Vec::new();
        let mut ret = String::new();

        loop {
            let (offset, err) = decoder.raw_feed(remaining, &mut ret);
            if offset > 0 { unprocessed.clear(); }
            match err {
                Some(err) => {
                    unprocessed.push_all(remaining.slice(offset, err.upto));
                    if !trap.trap(decoder, unprocessed.as_slice(), &mut ret) {
                        return Err(err.cause);
                    }
                    unprocessed.clear();
                    remaining = remaining.slice(err.upto, remaining.len());
                }
                None => {
                    unprocessed.push_all(remaining.slice(offset, remaining.len()));
                    break
                }
            }
        }

        match decoder.raw_finish(&mut ret) {
            Some(err) => {
                if !trap.trap(decoder, unprocessed.as_slice(), &mut ret) {
                    return Err(err.cause);
                }
            }
            None => {}
        }
        Ok(ret)
    }
}

/// A type of the bare function in `EncoderTrap` values.
#[unstable]
pub type EncoderTrapFunc =
    extern "Rust" fn(encoder: &mut Encoder, input: &str, output: &mut ByteWriter) -> bool;

/// A type of the bare function in `DecoderTrap` values.
#[unstable]
pub type DecoderTrapFunc =
    extern "Rust" fn(decoder: &mut Decoder, input: &[u8], output: &mut StringWriter) -> bool;

/// Trap, which handles decoder errors.
#[stable]
pub enum DecoderTrap {
    /// Immediately fails on errors.
    /// Corresponds to WHATWG "fatal" error algorithm.
    DecodeStrict,
    /// Replaces an error with a U+FFFD (decoder).
    /// Corresponds to WHATWG "replacement" error algorithm.
    DecodeReplace,
    /// Silently ignores an error, effectively replacing it with an empty sequence.
    DecodeIgnore,
    /// Calls given function to handle decoder errors.
    /// The function is given the current decoder, input and output writer,
    /// and should return true only when it is fine to keep going.
    #[unstable] DecoderTrap(DecoderTrapFunc),
}

impl DecoderTrap {
    /// Handles a decoder error. May write to the output writer.
    /// Returns true only when it is fine to keep going.
    fn trap(&self, decoder: &mut Decoder, input: &[u8], output: &mut StringWriter) -> bool {
        match *self {
            DecodeStrict => false,
            DecodeReplace => { output.write_char('\ufffd'); true },
            DecodeIgnore => true,
            DecoderTrap(func) => func(decoder, input, output),
        }
    }
}

#[stable]
pub enum EncoderTrap {
    /// Immediately fails on errors.
    /// Corresponds to WHATWG "fatal" error algorithm.
    EncodeStrict,
    /// Replaces an error with `?` in given encoding.
    /// Note that this fails when `?` cannot be represented in given encoding.
    /// Corresponds to WHATWG "URL" error algorithms.
    EncodeReplace,
    /// Silently ignores an error, effectively replacing it with an empty sequence.
    EncodeIgnore,
    /// Replaces an error with XML numeric character references (e.g. `&#1234;`).
    /// The encoder trap fails when NCRs cannot be represented in given encoding.
    /// Corresponds to WHATWG "<form>" error algorithms.
    EncodeNcrEscape,
    /// Calls given function to handle encoder errors.
    /// The function is given the current encoder, input and output writer,
    /// and should return true only when it is fine to keep going.
    #[unstable] EncoderTrap(EncoderTrapFunc),
}

impl EncoderTrap {
    /// Handles an encoder error. May write to the output writer.
    /// Returns true only when it is fine to keep going.
    fn trap(&self, encoder: &mut Encoder, input: &str, output: &mut ByteWriter) -> bool {
        fn reencode(encoder: &mut Encoder, input: &str, output: &mut ByteWriter,
                    trapname: &str) -> bool {
            if encoder.is_ascii_compatible() { // optimization!
                output.write_bytes(input.as_bytes());
            } else {
                let (_, err) = encoder.raw_feed(input, output);
                if err.is_some() {
                    fail!("{:s} cannot reencode a replacement string", trapname);
                }
            }
            true
        }

        match *self {
            EncodeStrict => false,
            EncodeReplace => reencode(encoder, "?", output, "Replace"),
            EncodeIgnore => true,
            EncodeNcrEscape => {
                let mut escapes = String::new();
                for ch in input.chars() {
                    escapes.push_str(format!("&#{:d};", ch as int).as_slice());
                }
                reencode(encoder, escapes.as_slice(), output, "NcrEscape")
            },
            EncoderTrap(func) => func(encoder, input, output),
        }
    }
}

/// Determine the encoding by looking for a Byte Order Mark (BOM)
/// and decoded a single string in memory.
/// Return the result and the used encoding.
#[unstable]
pub fn decode(input: &[u8], trap: DecoderTrap, fallback_encoding: EncodingRef)
           -> (Result<String,SendStr>, EncodingRef) {
    use all::{UTF_8, UTF_16LE, UTF_16BE};
    if input.starts_with([0xEF, 0xBB, 0xBF]) {
        (UTF_8.decode(input.slice_from(3), trap), UTF_8 as EncodingRef)
    } else if input.starts_with([0xFE, 0xFF]) {
        (UTF_16BE.decode(input.slice_from(2), trap), UTF_16BE as EncodingRef)
    } else if input.starts_with([0xFF, 0xFE]) {
        (UTF_16LE.decode(input.slice_from(2), trap), UTF_16LE as EncodingRef)
    } else {
        (fallback_encoding.decode(input, trap), fallback_encoding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use util::StrCharIndex;

    // a contrived encoding example: same as ASCII, but inserts `prepend` between each character
    // within two "e"s (so that `widespread` becomes `wide*s*p*r*ead` and `eeeeasel` becomes
    // `e*ee*ease*l` where `*` is substituted by `prepend`) and prohibits `prohibit` character.
    struct MyEncoder { flag: bool, prohibit: char, prepend: &'static str, toggle: bool }
    impl Encoder for MyEncoder {
        fn from_self(&self) -> Box<Encoder> {
            box MyEncoder { flag: self.flag,
                            prohibit: self.prohibit,
                            prepend: self.prepend,
                            toggle: false } as Box<Encoder>
        }
        fn is_ascii_compatible(&self) -> bool { self.flag }
        fn raw_feed(&mut self, input: &str,
                    output: &mut ByteWriter) -> (uint, Option<CodecError>) {
            for ((i,j), ch) in input.index_iter() {
                if ch <= '\u007f' && ch != self.prohibit {
                    if self.toggle && !self.prepend.is_empty() {
                        output.write_bytes(self.prepend.as_bytes());
                    }
                    output.write_byte(ch as u8);
                    if ch == 'e' {
                        self.toggle = !self.toggle;
                    }
                } else {
                    return (i, Some(CodecError { upto: j, cause: "!!!".into_maybe_owned() }));
                }
            }
            (input.len(), None)
        }
        fn raw_finish(&mut self, _output: &mut ByteWriter) -> Option<CodecError> { None }
    }

    struct MyEncoding { flag: bool, prohibit: char, prepend: &'static str }
    impl Encoding for MyEncoding {
        fn name(&self) -> &'static str { "my encoding" }
        fn encoder(&'static self) -> Box<Encoder> {
            box MyEncoder { flag: self.flag,
                            prohibit: self.prohibit,
                            prepend: self.prepend,
                            toggle: false } as Box<Encoder>
        }
        fn decoder(&'static self) -> Box<Decoder> { fail!("not supported") }
    }

    #[test]
    fn test_reencoding_trap_with_ascii_compatible_encoding() {
        static COMPAT: &'static MyEncoding =
            &MyEncoding { flag: true, prohibit: '\u0080', prepend: "" };
        static INCOMPAT: &'static MyEncoding =
            &MyEncoding { flag: false, prohibit: '\u0080', prepend: "" };

        assert_eq!(COMPAT.encode("Hello\u203d I'm fine.", EncodeNcrEscape),
                   Ok(Vec::from_slice(bytes!("Hello&#8253; I'm fine."))));
        assert_eq!(INCOMPAT.encode("Hello\u203d I'm fine.", EncodeNcrEscape),
                   Ok(Vec::from_slice(bytes!("Hello&#8253; I'm fine."))));
    }

    #[test]
    fn test_reencoding_trap_with_ascii_incompatible_encoding() {
        static COMPAT: &'static MyEncoding =
            &MyEncoding { flag: true, prohibit: '\u0080', prepend: "*" };
        static INCOMPAT: &'static MyEncoding =
            &MyEncoding { flag: false, prohibit: '\u0080', prepend: "*" };

        // this should behave incorrectly as the encoding broke the assumption.
        assert_eq!(COMPAT.encode("Hello\u203d I'm fine.", EncodeNcrEscape),
                   Ok(Vec::from_slice(bytes!("He*l*l*o&#8253;* *I*'*m* *f*i*n*e."))));
        assert_eq!(INCOMPAT.encode("Hello\u203d I'm fine.", EncodeNcrEscape),
                   Ok(Vec::from_slice(bytes!("He*l*l*o*&*#*8*2*5*3*;* *I*'*m* *f*i*n*e."))));
    }

    #[test]
    #[should_fail]
    fn test_reencoding_trap_can_fail() {
        static FAIL: &'static MyEncoding = &MyEncoding { flag: false, prohibit: '&', prepend: "" };

        // this should fail as this contrived encoding does not support `&` at all
        let _ = FAIL.encode("Hello\u203d I'm fine.", EncodeNcrEscape);
    }
}

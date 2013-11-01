// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
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
 * ````
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

use std::send_str::SendStr;

/// Error information from either encoder or decoder.
pub struct CodecError {
    /// The byte position of the first remaining byte, which is next to the problematic byte.
    /// The caller should feed the bytes starting from this point again
    /// in order to continue encoding or decoding after an error.
    /// This value is always set to 0 for `finish`.
    upto: uint,
    /// A human-readable cause of the error.
    cause: SendStr,
}

/// Byte writer used by `Encoder`s. In most cases this will be an owned vector of `u8`.
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

impl<T:OwnedVector<u8>+OwnedCopyableVector<u8>> ByteWriter for T {
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

impl<T:OwnedStr+Container> StringWriter for T {
    fn writer_hint(&mut self, expectedlen: uint) {
        let newlen = self.len() + expectedlen;
        self.reserve_at_least(newlen);
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
pub trait Encoder {
    /// Returns a reference to the encoding implemented by this encoder.
    fn encoding(&self) -> &'static Encoding;

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
    fn test_feed(&mut self, input: &str) -> (uint, Option<CodecError>, ~[u8]) {
        let mut buf = ~[];
        let (nprocessed, err) = self.raw_feed(input, &mut buf as &mut super::ByteWriter);
        (nprocessed, err, buf)
    }

    /// A test-friendly interface to `raw_finish`. Internal use only.
    #[cfg(test)]
    fn test_finish(&mut self) -> (Option<CodecError>, ~[u8]) {
        let mut buf = ~[];
        let err = self.raw_finish(&mut buf as &mut super::ByteWriter);
        (err, buf)
    }
}

/// Encoder converting a byte sequence into a Unicode string.
/// This is a lower level interface, and normally `Encoding::decode` should be used instead.
pub trait Decoder {
    /// Returns a reference to the encoding implemented by this decoder.
    fn encoding(&self) -> &'static Encoding;

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
    fn test_feed(&mut self, input: &[u8]) -> (uint, Option<CodecError>, ~str) {
        let mut buf = ~"";
        let (nprocessed, err) = self.raw_feed(input, &mut buf as &mut super::StringWriter);
        (nprocessed, err, buf)
    }

    /// A test-friendly interface to `raw_finish`. Internal use only.
    #[cfg(test)]
    fn test_finish(&mut self) -> (Option<CodecError>, ~str) {
        let mut buf = ~"";
        let err = self.raw_finish(&mut buf as &mut super::StringWriter);
        (err, buf)
    }
}

/// Character encoding.
pub trait Encoding {
    /// Returns the canonical name of given encoding.
    fn name(&self) -> &'static str;

    /// Creates a new encoder.
    fn encoder(&'static self) -> ~Encoder;

    /// Creates a new decoder.
    fn decoder(&'static self) -> ~Decoder;

    /// Returns a preferred replacement sequence for the encoder.
    /// Normally `?` encoded in given encoding.
    /// Note that this is fixed to `"\ufffd"` for the decoder.
    fn preferred_replacement_seq(&self) -> ~[u8] { ~[0x3f] /* "?" */ }

    /// An easy-to-use interface to `Encoder`.
    /// On the encoder error `trap` is called,
    /// which may return a replacement sequence to continue processing,
    /// or a failure to return the error.
    fn encode<Trap:EncoderTrap>(&'static self, input: &str,
                                mut trap: Trap) -> Result<~[u8],SendStr> {
        let mut encoder = self.encoder();
        let mut remaining = input;
        let mut unprocessed = ~"";
        let mut ret = ~[];

        loop {
            let (offset, err) = encoder.raw_feed(remaining, &mut ret as &mut ByteWriter);
            if offset > 0 { unprocessed.clear(); }
            match err {
                Some(err) => {
                    unprocessed.push_str(remaining.slice(offset, err.upto));
                    match trap.encoder_trap(self as &Encoding, unprocessed) {
                        Some(s) => { ret.push_all(s); }
                        None => { return Err(err.cause); }
                    }
                    unprocessed.clear();
                    remaining = remaining.slice(err.upto, remaining.len());
                }
                None => {
                    unprocessed.push_str(remaining.slice(offset, remaining.len()));
                    break
                }
            }
        }

        match encoder.raw_finish(&mut ret as &mut ByteWriter) {
            Some(err) => {
                match trap.encoder_trap(self as &Encoding, unprocessed) {
                    Some(s) => { ret.push_all(s); }
                    None => { return Err(err.cause); }
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
    fn decode<Trap:DecoderTrap>(&'static self, input: &[u8],
                                mut trap: Trap) -> Result<~str,SendStr> {
        let mut decoder = self.decoder();
        let mut remaining = input;
        let mut unprocessed = ~[];
        let mut ret = ~"";

        loop {
            let (offset, err) = decoder.raw_feed(remaining, &mut ret as &mut StringWriter);
            if offset > 0 { unprocessed.clear(); }
            match err {
                Some(err) => {
                    unprocessed.push_all(remaining.slice(offset, err.upto));
                    match trap.decoder_trap(self as &Encoding, unprocessed) {
                        Some(s) => { ret.push_str(s); }
                        None => { return Err(err.cause); }
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

        match decoder.raw_finish(&mut ret as &mut StringWriter) {
            Some(err) => {
                match trap.decoder_trap(self as &Encoding, unprocessed) {
                    Some(s) => { ret.push_str(s); }
                    None => { return Err(err.cause); }
                }
            }
            None => {}
        }
        Ok(ret)
    }
}

/// Encoder trap, which handles encoder errors.
/// Note that a function with the same arguments as `EncoderTrap::encoder_trap`
/// is also a valid encoder trap.
pub trait EncoderTrap {
    /// Handles an encoder error.
    /// Returns a replacement sequence or gives up by returning `None`.
    fn encoder_trap(&mut self, encoding: &Encoding, input: &str) -> Option<~[u8]>;
}

/// Decoder trap, which handles decoder errors.
/// Note that a function with the same arguments as `DecoderTrap::decoder_trap`
/// is also a valid decoder trap.
pub trait DecoderTrap {
    /// Handles a decoder error.
    /// Returns a replacement string or gives up by returning `None`.
    fn decoder_trap(&mut self, encoding: &Encoding, input: &[u8]) -> Option<~str>;
}

impl<'self> EncoderTrap for &'self fn(&str) -> ~[u8] {
    #[inline(always)]
    fn encoder_trap(&mut self, _encoding: &Encoding, input: &str) -> Option<~[u8]> {
        Some((*self)(input))
    }
}

impl<'self> DecoderTrap for &'self fn(&[u8]) -> ~str {
    #[inline(always)]
    fn decoder_trap(&mut self, _encoding: &Encoding, input: &[u8]) -> Option<~str> {
        Some((*self)(input))
    }
}

/// A built-in trap which gives up every encoder and decoder error.
pub struct Strict;

impl EncoderTrap for Strict {
    #[inline]
    fn encoder_trap(&mut self, _encoding: &Encoding, _input: &str) -> Option<~[u8]> {
        None
    }
}

impl DecoderTrap for Strict {
    #[inline]
    fn decoder_trap(&mut self, _encoding: &Encoding, _input: &[u8]) -> Option<~str> {
        None
    }
}

/// A built-in trap which replaces any error into a replacement character,
/// which is `"\ufffd"` for the decoder
/// and an encoding-specified character (normally `"?"`) for the encoder.
pub struct Replace;

impl EncoderTrap for Replace {
    #[inline]
    fn encoder_trap(&mut self, encoding: &Encoding, _input: &str) -> Option<~[u8]> {
        Some(encoding.preferred_replacement_seq())
    }
}

impl DecoderTrap for Replace {
    #[inline]
    fn decoder_trap(&mut self, _encoding: &Encoding, _input: &[u8]) -> Option<~str> {
        Some(~"\ufffd")
    }
}

/// A built-in trap which ignores any error.
pub struct Ignore;

impl EncoderTrap for Ignore {
    #[inline]
    fn encoder_trap(&mut self, _encoding: &Encoding, _input: &str) -> Option<~[u8]> {
        Some(~[])
    }
}

impl DecoderTrap for Ignore {
    #[inline]
    fn decoder_trap(&mut self, _encoding: &Encoding, _input: &[u8]) -> Option<~str> {
        Some(~"")
    }
}


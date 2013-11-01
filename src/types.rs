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
    /// Creates a fresh `Encoder` instance which parameters are same as `self`.
    fn from_self(&self) -> ~Encoder;

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
    /// Creates a fresh `Decoder` instance which parameters are same as `self`.
    fn from_self(&self) -> ~Decoder;

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
    /// This name is guaranteed to be unique across built-in encodings,
    /// but it is not normative and would be at most arbitrary.
    fn name(&self) -> &'static str;

    /// Returns a name of given encoding defined in the WHATWG Encoding standard, if any.
    /// This name often differs from `name` due to the compatibility reason.
    fn whatwg_name(&self) -> Option<&'static str> { None }

    /// Creates a new encoder.
    fn encoder(&'static self) -> ~Encoder;

    /// Creates a new decoder.
    fn decoder(&'static self) -> ~Decoder;

    /// An easy-to-use interface to `Encoder`.
    /// On the encoder error `trap` is called,
    /// which may return a replacement sequence to continue processing,
    /// or a failure to return the error.
    fn encode(&'static self, input: &str, trap: Trap) -> Result<~[u8],SendStr> {
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
                    if !trap.encoder_trap(encoder, unprocessed, &mut ret as &mut ByteWriter) {
                        return Err(err.cause);
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
                if !trap.encoder_trap(encoder, unprocessed, &mut ret as &mut ByteWriter) {
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
    fn decode(&'static self, input: &[u8], trap: Trap) -> Result<~str,SendStr> {
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
                    if !trap.decoder_trap(decoder, unprocessed, &mut ret as &mut StringWriter) {
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

        match decoder.raw_finish(&mut ret as &mut StringWriter) {
            Some(err) => {
                if !trap.decoder_trap(decoder, unprocessed, &mut ret as &mut StringWriter) {
                    return Err(err.cause);
                }
            }
            None => {}
        }
        Ok(ret)
    }
}

/// A type of the bare function in `EncoderTrap` values.
pub type EncoderTrapFunc =
    extern "Rust" fn(encoder: &Encoder, input: &str, output: &mut ByteWriter) -> bool;

/// A type of the bare function in `DecoderTrap` values.
pub type DecoderTrapFunc =
    extern "Rust" fn(decoder: &Decoder, input: &[u8], output: &mut StringWriter) -> bool;

/// Trap, which handles decoder and encoder errors.
/// Some traps can be used both for decoders (D) and encoders (E), others cannot.
pub enum Trap {
    /// D/E: Immediately fails on errors.
    /// Corresponds to WHATWG "fatal" error algorithm.
    Strict,
    /// D/E: Replaces an error with a fixed sequence.
    /// The string is either U+FFFD (decoder) or `?` in given encoding (encoder).
    /// Note that the encoder trap fails when `?` cannot be represented in given encoding.
    /// Corresponds to WHATWG "replacement"/"URL" error algorithms.
    Replace,
    /// D/E: Silently ignores an error, effectively replacing it with an empty sequence.
    Ignore,
    /// E: Replaces an error with XML numeric character references (e.g. `&#1234;`).
    /// The encoder trap fails when NCRs cannot be represented in given encoding.
    /// Corresponds to WHATWG "<form>" error algorithms.
    NcrEscape,
    /// E: Calls given function to handle encoder errors.
    /// The function is given the current encoder, input and output writer,
    /// and should return true only when it is fine to keep going.
    EncoderTrap(EncoderTrapFunc),
    /// D: Calls given function to handle decoder errors.
    /// The function is given the current decoder, input and output writer,
    /// and should return true only when it is fine to keep going.
    DecoderTrap(DecoderTrapFunc),
}

impl Trap {
    /// Handles an encoder error. May write to the output writer.
    /// Returns true only when it is fine to keep going.
    fn encoder_trap(&self, encoder: &Encoder, input: &str, output: &mut ByteWriter) -> bool {
        fn reencode(encoder: &Encoder, input: &str, output: &mut ByteWriter,
                    trapname: &str) -> bool {
            if encoder.is_ascii_compatible() { // optimization!
                output.write_bytes(input.as_bytes());
            } else {
                let mut e = encoder.from_self();
                let (_, err) = e.raw_feed(input, output);
                if err.is_some() || e.raw_finish(output).is_some() {
                    fail!("%s cannot reencode a replacement string", trapname);
                }
            }
            true
        }

        match *self {
            Strict => false,
            Replace => reencode(encoder, "?", output, "Replace"),
            Ignore => true,
            NcrEscape => {
                let mut escapes = ~"";
                for ch in input.iter() { escapes.push_str(format!("&\\#{:d};", ch as int)); }
                reencode(encoder, escapes, output, "NcrEscape")
            },
            EncoderTrap(func) => func(encoder, input, output),
            DecoderTrap(*) => fail!("DecoderTrap cannot be used with encoders"),
        }
    }

    /// Handles a decoder error. May write to the output writer.
    /// Returns true only when it is fine to keep going.
    fn decoder_trap(&self, decoder: &Decoder, input: &[u8], output: &mut StringWriter) -> bool {
        match *self {
            Strict => false,
            Replace => { output.write_char('\ufffd'); true },
            Ignore => true,
            NcrEscape => fail!("NcrEscape cannot be used with decoders"),
            EncoderTrap(*) => fail!("EncoderTrap cannot be used with decoders"),
            DecoderTrap(func) => func(decoder, input, output),
        }
    }
}


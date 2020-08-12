// This is a part of rust-encoding.
// Copyright (c) 2013-2015, Kang Seonghoon.
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
 * The first offset (the first `usize` in the tuple) points to the first unprocessed bytes,
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

use std::borrow::Cow;
use std::fmt;

/// Error information from either encoder or decoder.
pub struct CodecError {
    /// The byte position of the first remaining byte, with respect to the *current* input.
    /// For the `finish` call, this should be no more than zero (since there is no input).
    /// It can be negative if the remaining byte is in the prior inputs,
    /// as long as the remaining byte is not yet processed.
    /// The caller should feed the bytes starting from this point again
    /// in order to continue encoding or decoding after an error.
    pub upto: isize,
    /// A human-readable cause of the error.
    pub cause: Cow<'static, str>,
}

/// Byte writer used by encoders. In most cases this will be an owned vector of `u8`.
pub trait ByteWriter {
    /// Hints an expected lower bound on the length (in bytes) of the output
    /// until the next call to `writer_hint`,
    /// so that the writer can reserve the memory for writing.
    /// `RawEncoder`s are recommended but not required to call this method
    /// with an appropriate estimate.
    /// By default this method does nothing.
    fn writer_hint(&mut self, _expectedlen: usize) {}

    /// Writes a single byte.
    fn write_byte(&mut self, b: u8);

    /// Writes a number of bytes.
    fn write_bytes(&mut self, v: &[u8]);
}

impl ByteWriter for Vec<u8> {
    fn writer_hint(&mut self, expectedlen: usize) {
        self.reserve(expectedlen);
    }

    fn write_byte(&mut self, b: u8) {
        self.push(b);
    }

    fn write_bytes(&mut self, v: &[u8]) {
        self.extend_from_slice(v);
    }
}

/// String writer used by decoders. In most cases this will be an owned string.
pub trait StringWriter {
    /// Hints an expected lower bound on the length (in bytes) of the output
    /// until the next call to `writer_hint`,
    /// so that the writer can reserve the memory for writing.
    /// `RawDecoder`s are recommended but not required to call this method
    /// with an appropriate estimate.
    /// By default this method does nothing.
    fn writer_hint(&mut self, _expectedlen: usize) {}

    /// Writes a single character.
    fn write_char(&mut self, c: char);

    /// Writes a string.
    fn write_str(&mut self, s: &str);
}

impl StringWriter for String {
    fn writer_hint(&mut self, expectedlen: usize) {
        self.reserve(expectedlen);
    }

    fn write_char(&mut self, c: char) {
        self.push(c);
    }

    fn write_str(&mut self, s: &str) {
        self.push_str(s);
    }
}

/// Encoder converting a Unicode string into a byte sequence.
/// This is a lower level interface, and normally `Encoding::encode` should be used instead.
pub trait RawEncoder: Send + 'static {
    /// Creates a fresh `RawEncoder` instance which parameters are same as `self`.
    fn from_self(&self) -> Box<dyn RawEncoder>;

    /// Returns true if this encoding is compatible to ASCII,
    /// i.e. U+0000 through U+007F always map to bytes 00 through 7F and nothing else.
    fn is_ascii_compatible(&self) -> bool { false }

    /// Feeds given portion of string to the encoder,
    /// pushes the an encoded byte sequence at the end of the given output,
    /// and returns a byte offset to the first unprocessed character
    /// (that can be zero when the first such character appeared in the prior calls to `raw_feed`)
    /// and optional error information (None means success).
    fn raw_feed(&mut self, input: &str, output: &mut dyn ByteWriter) -> (usize, Option<CodecError>);

    /// Finishes the encoder,
    /// pushes the an encoded byte sequence at the end of the given output,
    /// and returns optional error information (None means success).
    /// `remaining` value of the error information, if any, is always an empty string.
    fn raw_finish(&mut self, output: &mut dyn ByteWriter) -> Option<CodecError>;
}

/// Decoder converting a byte sequence into a Unicode string.
/// This is a lower level interface, and normally `Encoding::decode` should be used instead.
pub trait RawDecoder: Send + 'static {
    /// Creates a fresh `RawDecoder` instance which parameters are same as `self`.
    fn from_self(&self) -> Box<dyn RawDecoder>;

    /// Returns true if this encoding is compatible to ASCII,
    /// i.e. bytes 00 through 7F always map to U+0000 through U+007F and nothing else.
    fn is_ascii_compatible(&self) -> bool { false }

    /// Feeds given portion of byte sequence to the encoder,
    /// pushes the a decoded string at the end of the given output,
    /// and returns an offset to the first unprocessed byte
    /// (that can be zero when the first such byte appeared in the prior calls to `raw_feed`)
    /// and optional error information (None means success).
    fn raw_feed(&mut self, input: &[u8], output: &mut dyn StringWriter) -> (usize, Option<CodecError>);

    /// Finishes the decoder,
    /// pushes the a decoded string at the end of the given output,
    /// and returns optional error information (None means success).
    fn raw_finish(&mut self, output: &mut dyn StringWriter) -> Option<CodecError>;
}

/// A trait object using dynamic dispatch which is a sendable reference to the encoding,
/// for code where the encoding is not known at compile-time.
pub type EncodingRef = &'static (dyn Encoding + Send + Sync);

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
    fn raw_encoder(&self) -> Box<dyn RawEncoder>;

    /// Creates a new decoder.
    fn raw_decoder(&self) -> Box<dyn RawDecoder>;

    /// An easy-to-use interface to `RawEncoder`.
    /// On the encoder error `trap` is called,
    /// which may return a replacement sequence to continue processing,
    /// or a failure to return the error.
    fn encode(&self, input: &str, trap: EncoderTrap) -> Result<Vec<u8>, Cow<'static, str>> {
        let mut ret = Vec::new();
        self.encode_to(input, trap, &mut ret).map(|_| ret)
    }

    /// Encode into a `ByteWriter`.
    fn encode_to(&self, input: &str, trap: EncoderTrap, ret: &mut dyn ByteWriter)
        -> Result<(), Cow<'static, str>>
    {
        // we don't need to keep `unprocessed` here;
        // `raw_feed` should process as much input as possible.
        let mut encoder = self.raw_encoder();
        let mut remaining = 0;

        loop {
            let (offset, err) = encoder.raw_feed(&input[remaining..], ret);
            let unprocessed = remaining + offset;
            match err {
                Some(err) => {
                    remaining = (remaining as isize + err.upto) as usize;
                    if !trap.trap(&mut *encoder, &input[unprocessed..remaining], ret) {
                        return Err(err.cause);
                    }
                }
                None => {
                    remaining = input.len();
                    match encoder.raw_finish(ret) {
                        Some(err) => {
                            remaining = (remaining as isize + err.upto) as usize;
                            if !trap.trap(&mut *encoder, &input[unprocessed..remaining], ret) {
                                return Err(err.cause);
                            }
                        }
                        None => {}
                    }
                    if remaining >= input.len() { return Ok(()); }
                }
            }
        }
    }

    /// An easy-to-use interface to `RawDecoder`.
    /// On the decoder error `trap` is called,
    /// which may return a replacement string to continue processing,
    /// or a failure to return the error.
    fn decode(&self, input: &[u8], trap: DecoderTrap) -> Result<String, Cow<'static, str>> {
        let mut ret = String::new();
        self.decode_to(input, trap, &mut ret).map(|_| ret)
    }

    /// Decode into a `StringWriter`.
    ///
    /// This does *not* handle partial characters at the beginning or end of `input`!
    /// Use `RawDecoder` for incremental decoding.
    fn decode_to(&self, input: &[u8], trap: DecoderTrap, ret: &mut dyn StringWriter)
        -> Result<(), Cow<'static, str>>
    {
        // we don't need to keep `unprocessed` here;
        // `raw_feed` should process as much input as possible.
        let mut decoder = self.raw_decoder();
        let mut remaining = 0;

        loop {
            let (offset, err) = decoder.raw_feed(&input[remaining..], ret);
            let unprocessed = remaining + offset;
            match err {
                Some(err) => {
                    remaining = (remaining as isize + err.upto) as usize;
                    if !trap.trap(&mut *decoder, &input[unprocessed..remaining], ret) {
                        return Err(err.cause);
                    }
                }
                None => {
                    remaining = input.len();
                    match decoder.raw_finish(ret) {
                        Some(err) => {
                            remaining = (remaining as isize + err.upto) as usize;
                            if !trap.trap(&mut *decoder, &input[unprocessed..remaining], ret) {
                                return Err(err.cause);
                            }
                        }
                        None => {}
                    }
                    if remaining >= input.len() { return Ok(()); }
                }
            }
        }
    }
}

impl<'a> fmt::Debug for &'a dyn Encoding {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_str("Encoding(")?;
        fmt.write_str(self.name())?;
        fmt.write_str(")")?;
        Ok(())
    }
}

/// A type of the bare function in `EncoderTrap` values.
pub type EncoderTrapFunc =
    extern "Rust" fn(encoder: &mut dyn RawEncoder, input: &str, output: &mut dyn ByteWriter) -> bool;

/// A type of the bare function in `DecoderTrap` values.
pub type DecoderTrapFunc =
    extern "Rust" fn(decoder: &mut dyn RawDecoder, input: &[u8], output: &mut dyn StringWriter) -> bool;

/// Trap, which handles decoder errors.
#[derive(Copy)]
pub enum DecoderTrap {
    /// Immediately fails on errors.
    /// Corresponds to WHATWG "fatal" error algorithm.
    Strict,
    /// Replaces an error with a U+FFFD (decoder).
    /// Corresponds to WHATWG "replacement" error algorithm.
    Replace,
    /// Silently ignores an error, effectively replacing it with an empty sequence.
    Ignore,
    /// Calls given function to handle decoder errors.
    /// The function is given the current decoder, input and output writer,
    /// and should return true only when it is fine to keep going.
    Call(DecoderTrapFunc),
}

impl DecoderTrap {
    /// Handles a decoder error. May write to the output writer.
    /// Returns true only when it is fine to keep going.
    pub fn trap(&self, decoder: &mut dyn RawDecoder, input: &[u8], output: &mut dyn StringWriter) -> bool {
        match *self {
            DecoderTrap::Strict     => false,
            DecoderTrap::Replace    => { output.write_char('\u{fffd}'); true },
            DecoderTrap::Ignore     => true,
            DecoderTrap::Call(func) => func(decoder, input, output),
        }
    }
}

impl Clone for DecoderTrap {
    fn clone(&self) -> DecoderTrap {
        match *self {
            DecoderTrap::Strict => DecoderTrap::Strict,
            DecoderTrap::Replace => DecoderTrap::Replace,
            DecoderTrap::Ignore => DecoderTrap::Ignore,
            DecoderTrap::Call(f) => DecoderTrap::Call(f),
        }
    }
}

#[derive(Copy)]
pub enum EncoderTrap {
    /// Immediately fails on errors.
    /// Corresponds to WHATWG "fatal" error algorithm.
    Strict,
    /// Replaces an error with `?` in given encoding.
    /// Note that this fails when `?` cannot be represented in given encoding.
    /// Corresponds to WHATWG "URL" error algorithms.
    Replace,
    /// Silently ignores an error, effectively replacing it with an empty sequence.
    Ignore,
    /// Replaces an error with XML numeric character references (e.g. `&#1234;`).
    /// The encoder trap fails when NCRs cannot be represented in given encoding.
    /// Corresponds to WHATWG "<form>" error algorithms.
    NcrEscape,
    /// Calls given function to handle encoder errors.
    /// The function is given the current encoder, input and output writer,
    /// and should return true only when it is fine to keep going.
    Call(EncoderTrapFunc),
}

impl EncoderTrap {
    /// Handles an encoder error. May write to the output writer.
    /// Returns true only when it is fine to keep going.
    pub fn trap(&self, encoder: &mut dyn RawEncoder, input: &str, output: &mut dyn ByteWriter) -> bool {
        fn reencode(encoder: &mut dyn RawEncoder, input: &str, output: &mut dyn ByteWriter,
                    trapname: &str) -> bool {
            if encoder.is_ascii_compatible() { // optimization!
                output.write_bytes(input.as_bytes());
            } else {
                let (_, err) = encoder.raw_feed(input, output);
                if err.is_some() {
                    panic!("{} cannot reencode a replacement string", trapname);
                }
            }
            true
        }

        match *self {
            EncoderTrap::Strict     => false,
            EncoderTrap::Replace    => reencode(encoder, "?", output, "Replace"),
            EncoderTrap::Ignore     => true,
            EncoderTrap::NcrEscape  => {
                let mut escapes = String::new();
                for ch in input.chars() {
                    escapes.push_str(&format!("&#{};", ch as isize));
                }
                reencode(encoder, &escapes, output, "NcrEscape")
            },
            EncoderTrap::Call(func) => func(encoder, input, output),
        }
    }
}

impl Clone for EncoderTrap {
    fn clone(&self) -> EncoderTrap {
        match *self {
            EncoderTrap::Strict => EncoderTrap::Strict,
            EncoderTrap::Replace => EncoderTrap::Replace,
            EncoderTrap::Ignore => EncoderTrap::Ignore,
            EncoderTrap::NcrEscape => EncoderTrap::NcrEscape,
            EncoderTrap::Call(f) => EncoderTrap::Call(f),
        }
    }
}

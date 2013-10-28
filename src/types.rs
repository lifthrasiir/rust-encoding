// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Interface to the character encoding.

use std::send_str::SendStr;

/// Error information from either encoder or decoder. It has a pointer to the problematic sequence
/// and a remaining sequence yet to be processed.
pub struct CodecError<Remaining,Problem> {
    /// A remaining sequence. This is a portion of the sequence supplied by the caller and can be
    /// fed to the encoder or decoder to continue the processing.
    remaining: Remaining,
    /// A *copy* of the problematic sequence. The problematic sequence may originate from the prior
    /// calls to `raw_feed`.
    problem: Problem,
    /// A human-readable cause of the error.
    cause: SendStr,
}

/// Error information from encoder.
pub type EncoderError<'self> = CodecError<&'self str,~str>;

/// Error information from decoder.
pub type DecoderError<'self> = CodecError<&'self [u8],~[u8]>;

/// Encoder converting a Unicode string into a byte sequence. This is a lower level interface, and
/// normally `Encoding::encode` should be used instead.
pub trait Encoder {
    /// Returns a reference to the encoding implemented by this encoder.
    fn encoding(&self) -> &'static Encoding;

    /// Feeds given portion of string to the encoder,
    /// pushes the an encoded byte sequence at the end of the given output,
    /// and returns optional error information. None means success.
    fn raw_feed<'r>(&mut self, input: &'r str, output: &mut ~[u8]) -> Option<EncoderError<'r>>;

    #[cfg(test)] fn test_feed<'r>(&mut self, input: &'r str) -> (~[u8], Option<EncoderError<'r>>) {
        let mut output = ~[];
        let err = self.raw_feed(input, &mut output);
        (output, err)
    }

    /// Finishes the encoder,
    /// pushes the an encoded byte sequence at the end of the given output,
    /// and returns optional error information. None means success.
    /// `remaining` value of the error information, if any, is always an empty string.
    fn raw_finish(&mut self, output: &mut ~[u8]) -> Option<EncoderError<'static>>;

    #[cfg(test)] fn test_finish(&mut self) -> (~[u8], Option<EncoderError<'static>>) {
        let mut output = ~[];
        let err = self.raw_finish(&mut output);
        (output, err)
    }
}

/// Encoder converting a byte sequence into a Unicode string. This is a lower level interface, and
/// normally `Encoding::decode` should be used instead.
pub trait Decoder {
    /// Returns a reference to the encoding implemented by this decoder.
    fn encoding(&self) -> &'static Encoding;

    /// Feeds given portion of byte sequence to the encoder,
    /// pushes the a decoded string at the end of the given output,
    /// and returns optional error information. None means success.
    fn raw_feed<'r>(&mut self, input: &'r [u8], output: &mut ~str) -> Option<DecoderError<'r>>;

    #[cfg(test)] fn test_feed<'r>(&mut self, input: &'r [u8]) -> (~str, Option<DecoderError<'r>>) {
        let mut output = ~"";
        let err = self.raw_feed(input, &mut output);
        (output, err)
    }

    /// Finishes the decoder,
    /// pushes the a decoded string at the end of the given output,
    /// and returns optional error information. None means success.
    /// `remaining` value of the error information, if any, is always an empty sequence.
    fn raw_finish(&mut self, output: &mut ~str) -> Option<DecoderError<'static>>;

    #[cfg(test)] fn test_finish(&mut self) -> (~str, Option<DecoderError<'static>>) {
        let mut output = ~"";
        let err = self.raw_finish(&mut output);
        (output, err)
    }
}

/// Character encoding.
pub trait Encoding {
    /// Returns the canonical name of given encoding.
    fn name(&'static self) -> &'static str;
    /// Creates a new encoder.
    fn encoder(&'static self) -> ~Encoder;
    /// Creates a new decoder.
    fn decoder(&'static self) -> ~Decoder;
    /// Returns a preferred replacement sequence for the encoder. Normally `?` encoded in given
    /// encoding. Note that this is fixed to `"\ufffd"` for the decoder.
    fn preferred_replacement_seq(&'static self) -> ~[u8] { ~[0x3f] /* "?" */ }
}

/// Utilities for character encodings.
pub trait EncodingUtil<T:Encoding> {
    /// An easy-to-use interface to `Encoder`. On the encoder error `trap` is called, which may
    /// return a replacement sequence to continue processing, or a failure to return the error.
    fn encode<Trap:EncoderTrap<T>>(&'static self, input: &str, trap: Trap) -> Result<~[u8],~str>;
    /// An easy-to-use interface to `Decoder`. On the decoder error `trap` is called, which may
    /// return a replacement string to continue processing, or a failure to return the error.
    fn decode<Trap:DecoderTrap<T>>(&'static self, input: &[u8], trap: Trap) -> Result<~str,~str>;
}

impl<T:Encoding> EncodingUtil<T> for T {
    #[inline]
    fn encode<Trap:EncoderTrap<T>>(&'static self, input: &str, mut trap: Trap) -> Result<~[u8],~str> {
        let mut encoder = self.encoder();
        let mut remaining = input;
        let mut ret = ~[];

        loop {
            match encoder.raw_feed(remaining, &mut ret) {
                Some(err) => {
                    match trap.encoder_trap(self, err.problem) {
                        Some(s) => { ret.push_all(s); }
                        None => { return Err(err.cause.into_owned()); }
                    }
                    remaining = err.remaining;
                }
                None => break
            }
        }

        match encoder.raw_finish(&mut ret) {
            Some(err) => {
                match trap.encoder_trap(self, err.problem) {
                    Some(s) => { ret.push_all(s); }
                    None => { return Err(err.cause.into_owned()); }
                }
            }
            None => {}
        }
        Ok(ret)
    }

    #[inline]
    fn decode<Trap:DecoderTrap<T>>(&'static self, input: &[u8], mut trap: Trap) -> Result<~str,~str> {
        let mut decoder = self.decoder();
        let mut remaining = input;
        let mut ret = ~"";

        loop {
            match decoder.raw_feed(remaining, &mut ret) {
                Some(err) => {
                    match trap.decoder_trap(self, err.problem) {
                        Some(s) => { ret.push_str(s); }
                        None => { return Err(err.cause.into_owned()); }
                    }
                    remaining = err.remaining;
                }
                None => break
            }
        }

        match decoder.raw_finish(&mut ret) {
            Some(err) => {
                match trap.decoder_trap(self, err.problem) {
                    Some(s) => { ret.push_str(s); }
                    None => { return Err(err.cause.into_owned()); }
                }
            }
            None => {}
        }
        Ok(ret)
    }
}

/// Encoder trap, which handles encoder errors. Note that a function with the same arguments as
/// `EncoderTrap::encoder_trap` is also a valid encoder trap.
pub trait EncoderTrap<T:Encoding> {
    /// Handles an encoder error. Returns a replacement sequence or gives up by returning `None`.
    fn encoder_trap(&mut self, encoding: &'static T, input: &str) -> Option<~[u8]>;
}

/// Decoder trap, which handles decoder errors. Note that a function with the same arguments as
/// `DecoderTrap::decoder_trap` is also a valid decoder trap.
pub trait DecoderTrap<T:Encoding> {
    /// Handles a decoder error. Returns a replacement string or gives up by returning `None`.
    fn decoder_trap(&mut self, encoding: &'static T, input: &[u8]) -> Option<~str>;
}

impl<'self,T:Encoding> EncoderTrap<T> for &'self fn(&str) -> ~[u8] {
    #[inline(always)]
    fn encoder_trap(&mut self, _encoding: &'static T, input: &str) -> Option<~[u8]> {
        Some((*self)(input))
    }
}

impl<'self,T:Encoding> DecoderTrap<T> for &'self fn(&[u8]) -> ~str {
    #[inline(always)]
    fn decoder_trap(&mut self, _encoding: &'static T, input: &[u8]) -> Option<~str> {
        Some((*self)(input))
    }
}

/// A built-in trap which gives up every encoder and decoder error.
pub struct Strict;

impl<T:Encoding> EncoderTrap<T> for Strict {
    #[inline]
    fn encoder_trap(&mut self, _encoding: &'static T, _input: &str) -> Option<~[u8]> {
        None
    }
}

impl<T:Encoding> DecoderTrap<T> for Strict {
    #[inline]
    fn decoder_trap(&mut self, _encoding: &'static T, _input: &[u8]) -> Option<~str> {
        None
    }
}

/// A built-in trap which replaces any error into a replacement character, which is `"\ufffd"` for
/// the decoder and an encoding-specified character (normally `"?"`) for the encoder.
pub struct Replace;

impl<T:Encoding> EncoderTrap<T> for Replace {
    #[inline]
    fn encoder_trap(&mut self, encoding: &'static T, _input: &str) -> Option<~[u8]> {
        Some(encoding.preferred_replacement_seq())
    }
}

impl<T:Encoding> DecoderTrap<T> for Replace {
    #[inline]
    fn decoder_trap(&mut self, _encoding: &'static T, _input: &[u8]) -> Option<~str> {
        Some(~"\ufffd")
    }
}

/// A built-in trap which ignores any error.
pub struct Ignore;

impl<T:Encoding> EncoderTrap<T> for Ignore {
    #[inline]
    fn encoder_trap(&mut self, _encoding: &'static T, _input: &str) -> Option<~[u8]> {
        Some(~[])
    }
}

impl<T:Encoding> DecoderTrap<T> for Ignore {
    #[inline]
    fn decoder_trap(&mut self, _encoding: &'static T, _input: &[u8]) -> Option<~str> {
        Some(~"")
    }
}


// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Interface to the character encoding.

/// Error information from either encoder or decoder. It has a pointer to the problematic sequence
/// and a remaining sequence yet to be processed.
pub struct CodecError<Remaining,Problem> {
    /// A remaining sequence. This is a portion of the sequence supplied by the caller and can be
    /// fed to the encoder or decoder to continue the processing.
    remaining: Remaining,
    /// A *copy* of the problematic sequence. The problematic sequence may originate from the prior
    /// calls to `feed`.
    problem: Problem,
    /// A human-readable cause of the error.
    cause: ~str,
}

/// Error information from encoder.
pub type EncoderError<'self> = CodecError<&'self str,~str>;

/// Error information from decoder.
pub type DecoderError<'self> = CodecError<&'self [u8],~[u8]>;

/// Encoder converting a Unicode string into a byte sequence. This is a lower level interface, and
/// normally `Encoding::encode` should be used instead.
pub trait Encoder: Clone {
    /// Returns a (copy of) encoding implemented by this encoder.
    pub fn encoding(&self) -> ~Encoding;
    /// Feeds given portion of string to the encoder, and returns an encoded byte sequence with
    /// optional error information.
    pub fn feed<'r>(&mut self, input: &'r str) -> (~[u8],Option<EncoderError<'r>>);
    /// Finishes the encoder, and returns the last encoded byte sequence with optional error
    /// information. `remaining` value of the error information is always an empty string if any.
    pub fn flush(~self) -> (~[u8],Option<EncoderError<'static>>);
}

/// Encoder converting a byte sequence into a Unicode string. This is a lower level interface, and
/// normally `Encoding::decode` should be used instead.
pub trait Decoder: Clone {
    /// Returns a (copy of) encoding implemented by this decoder.
    pub fn encoding(&self) -> ~Encoding;
    /// Feeds given portion of byte sequenc to the encoder, and returns a decoded string with
    /// optional error information.
    pub fn feed<'r>(&mut self, input: &'r [u8]) -> (~str,Option<DecoderError<'r>>);
    /// Finishes the decoder, and returns the last decoded string with optional error information.
    /// `remaining` value of the error information is always an empty sequence if any.
    pub fn flush(~self) -> (~str,Option<DecoderError<'static>>);
}

/// Character encoding.
pub trait Encoding {
    /// Returns the canonical name of given encoding.
    pub fn name(&self) -> ~str;
    /// Creates a new encoder.
    pub fn encoder(&self) -> ~Encoder;
    /// Creates a new decoder.
    pub fn decoder(&self) -> ~Decoder;
    /// Returns a preferred replacement sequence for the encoder. Normally `?` encoded in given
    /// encoding. Note that this is fixed to `"\ufffd"` for the decoder.
    pub fn preferred_replacement_seq(&self) -> ~[u8];
}

impl<'self> Encoding for &'self Encoding {
    pub fn name(&self) -> ~str { (*self).name() }
    pub fn encoder(&self) -> ~Encoder { (*self).encoder() }
    pub fn decoder(&self) -> ~Decoder { (*self).decoder() }
    pub fn preferred_replacement_seq(&self) -> ~[u8] { (*self).preferred_replacement_seq() }
}

impl Encoding for ~Encoding {
    pub fn name(&self) -> ~str { (*self).name() }
    pub fn encoder(&self) -> ~Encoder { (*self).encoder() }
    pub fn decoder(&self) -> ~Decoder { (*self).decoder() }
    pub fn preferred_replacement_seq(&self) -> ~[u8] { (*self).preferred_replacement_seq() }
}

/// Utilities for character encodings.
pub trait EncodingUtil<T:Encoding> {
    /// An easy-to-use interface to `Encoder`. On the encoder error `trap` is called, which may
    /// return a replacement sequence to continue processing, or a failure to return the error.
    pub fn encode<Trap:EncoderTrap<T>>(&self, input: &str, trap: Trap) -> Result<~[u8],~str>;
    /// An easy-to-use interface to `Decoder`. On the decoder error `trap` is called, which may
    /// return a replacement string to continue processing, or a failure to return the error.
    pub fn decode<Trap:DecoderTrap<T>>(&self, input: &[u8], trap: Trap) -> Result<~str,~str>;
}

impl<T:Encoding> EncodingUtil<T> for T {
    #[inline]
    pub fn encode<Trap:EncoderTrap<T>>(&self, input: &str, mut trap: Trap) -> Result<~[u8],~str> {
        let mut encoder = self.encoder();
        let mut remaining = input;
        let mut ret = ~[];

        loop {
            let (encoded, err) = encoder.feed(remaining);
            ret.push_all(encoded);
            match err {
                Some(err) => {
                    match trap.encoder_trap(self, err.problem) {
                        Some(s) => { ret.push_all(s); }
                        None => { return Err(err.cause); }
                    }
                    remaining = err.remaining;
                }
                None => break
            }
        }

        let (encoded, err) = encoder.flush();
        ret.push_all(encoded);
        match err {
            Some(err) => {
                match trap.encoder_trap(self, err.problem) {
                    Some(s) => { ret.push_all(s); }
                    None => { return Err(err.cause); }
                }
            }
            None => {}
        }
        Ok(ret)
    }

    #[inline]
    pub fn decode<Trap:DecoderTrap<T>>(&self, input: &[u8], mut trap: Trap) -> Result<~str,~str> {
        let mut decoder = self.decoder();
        let mut remaining = input;
        let mut ret = ~"";

        loop {
            let (decoded, err) = decoder.feed(remaining);
            ret.push_str(decoded);
            match err {
                Some(err) => {
                    match trap.decoder_trap(self, err.problem) {
                        Some(s) => { ret.push_str(s); }
                        None => { return Err(err.cause); }
                    }
                    remaining = err.remaining;
                }
                None => break
            }
        }

        let (decoded, err) = decoder.flush();
        ret.push_str(decoded);
        match err {
            Some(err) => {
                match trap.decoder_trap(self, err.problem) {
                    Some(s) => { ret.push_str(s); }
                    None => { return Err(err.cause); }
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
    pub fn encoder_trap(&mut self, encoding: &T, input: &str) -> Option<~[u8]>;
}

/// Decoder trap, which handles decoder errors. Note that a function with the same arguments as
/// `DecoderTrap::decoder_trap` is also a valid decoder trap.
pub trait DecoderTrap<T:Encoding> {
    /// Handles a decoder error. Returns a replacement string or gives up by returning `None`.
    pub fn decoder_trap(&mut self, encoding: &T, input: &[u8]) -> Option<~str>;
}

impl<'self,T:Encoding> EncoderTrap<T> for &'self fn(&T,&str) -> Option<~[u8]> {
    #[inline(always)]
    pub fn encoder_trap(&mut self, encoding: &T, input: &str) -> Option<~[u8]> {
        (*self)(encoding, input)
    }
}

impl<'self,T:Encoding> DecoderTrap<T> for &'self fn(&T,&[u8]) -> Option<~str> {
    #[inline(always)]
    pub fn decoder_trap(&mut self, encoding: &T, input: &[u8]) -> Option<~str> {
        (*self)(encoding, input)
    }
}

/// A built-in trap which gives up every encoder and decoder error.
pub struct Strict;

impl<T:Encoding> EncoderTrap<T> for Strict {
    #[inline]
    pub fn encoder_trap(&mut self, _encoding: &T, _input: &str) -> Option<~[u8]> {
        None
    }
}

impl<T:Encoding> DecoderTrap<T> for Strict {
    #[inline]
    pub fn decoder_trap(&mut self, _encoding: &T, _input: &[u8]) -> Option<~str> {
        None
    }
}

/// A built-in trap which replaces any error into a replacement character, which is `"\ufffd"` for
/// the decoder and an encoding-specified character (normally `"?"`) for the encoder.
pub struct Replace;

impl<T:Encoding> EncoderTrap<T> for Replace {
    #[inline]
    pub fn encoder_trap(&mut self, encoding: &T, _input: &str) -> Option<~[u8]> {
        Some(encoding.preferred_replacement_seq())
    }
}

impl<T:Encoding> DecoderTrap<T> for Replace {
    #[inline]
    pub fn decoder_trap(&mut self, _encoding: &T, _input: &[u8]) -> Option<~str> {
        Some(~"\ufffd")
    }
}

/// A built-in trap which ignores any error.
pub struct Ignore;

impl<T:Encoding> EncoderTrap<T> for Ignore {
    #[inline]
    pub fn encoder_trap(&mut self, _encoding: &T, _input: &str) -> Option<~[u8]> {
        Some(~[])
    }
}

impl<T:Encoding> DecoderTrap<T> for Ignore {
    #[inline]
    pub fn decoder_trap(&mut self, _encoding: &T, _input: &[u8]) -> Option<~str> {
        Some(~"")
    }
}


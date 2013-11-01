// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * An alternative API compatible to WHATWG Encoding standard.
 *
 * WHATWG Encoding standard, compared to the native interface provided by rust-encoding, requires
 * both intentionally different naming of encodings (for example, "EUC-KR" in the standard is
 * actually Windows code page 949) and that the decoder error only consumes one byte. While
 * rust-encoding itself does not implement the exact algorithm specified by the Encoding standard,
 * it is trivial to support the WHATWG Encoding interface for the sake of completeness.
 *
 * That said, why does rust-encoding implements its own standard? The main reason is that
 * rust-encoding provides much wider options for error recovery, and having as many bytes in
 * the problem as possible is beneficial in this regard. Rust-encoding still does *not* return every
 * byte in the problem to limit the memory consumption, and its definition of the error range is
 * at best arbitrary (it is not wrong though).
 */

use std::ascii::StrAsciiExt;
use all;
use types;

/*
#[deriving(Eq,Clone)]
pub struct TextDecoderOptions {
    fatal: bool
}

#[deriving(Eq,Clone)]
pub struct TextDecodeOptions {
    stream: bool
}

impl TextDecoderOptions {
    pub fn new() -> TextDecoderOptions { TextDecoderOptions { fatal: false } }
}

impl TextDecodeOptions {
    pub fn new() -> TextDecodeOptions { TextDecodeOptions { stream: false } }
}

pub struct TextDecoder {
    whatwg_name: &'static str,
    encoding: &'static types::Encoding,
    decoder: ~types::Decoder,
    fatal: bool,
    bom_seen: bool,
    first_bytes: ~[u8],
    ignorable_first_bytes: ~[u8],
}

impl TextDecoder {
    pub fn new(label: Option<~str>) -> Result<TextDecoder,~str> {
        TextDecoder::from_options(label, TextDecoderOptions::new())
    }

    pub fn from_options(label: Option<~str>, options: TextDecoderOptions)
                                    -> Result<TextDecoder,~str> {
        let label = label.unwrap_or(~"utf-8");
        let ret = encoding_from_label(label);
        if ret.is_none() { return Err(~"TypeError"); }
        let (encoding, whatwg_name) = ret.unwrap();
        if whatwg_name == "replacement" { return Err(~"TypeError"); }

        let ignorable_first_bytes = match whatwg_name {
            &"utf-16le" => ~[0xff, 0xfe],
            &"utf-16be" => ~[0xfe, 0xff],
            &"utf-8" => ~[0xef, 0xbb, 0xbf],
            _ => ~[],
        };
        Ok(TextDecoder { whatwg_name: whatwg_name, encoding: encoding, decoder: encoding.decoder(),
                         fatal: options.fatal, bom_seen: false, first_bytes: ~[],
                         ignorable_first_bytes: ignorable_first_bytes })
    }

    pub fn encoding(&self) -> ~str {
        self.whatwg_name.to_owned()
    }

    // XXX conflicts with `types::Decoder::decode`
    pub fn decode_buffer(&mut self, input: Option<&[u8]>) -> Result<~str,~str> {
        self.decode_buffer_with_options(input, TextDecodeOptions::new())
    }

    pub fn decode_buffer_with_options(&mut self, mut input: Option<&[u8]>,
                                      options: TextDecodeOptions) -> Result<~str,~str> {
        let feed_first_bytes;
        if !self.bom_seen {
            if input.is_none() { return Ok(~""); }
            let input_ = input.unwrap();

            let mut i = 0;
            let max_first_bytes = self.ignorable_first_bytes.len();
            while i < input_.len() && self.first_bytes.len() < max_first_bytes {
                self.first_bytes.push(input_[i]);
                i += 1;
            }
            input = Some(input_.slice(i, input_.len()));
            if self.first_bytes.len() == max_first_bytes {
                self.bom_seen = true;
                feed_first_bytes = (max_first_bytes > 0 &&
                                    self.first_bytes != self.ignorable_first_bytes);
            } else {
                return Ok(~""); // we don't feed the input until bom_seen is set
            }
        } else {
            feed_first_bytes = false;
        }

        fn handle_error<'r>(decoder: &mut ~types::Decoder, mut err: types::CodecError,
                            ret: &mut types::StringWriter) {
            loop {
                ret.write_char('\ufffd');
                let remaining = err.remaining;

                // we need to consume the entirety of `err.problem` before `err.remaining`.
                let mut remaining_ = err.problem;
                assert!(!remaining_.is_empty());
                remaining_.shift();
                loop {
                    let remaining__ = remaining_;
                    let err_ = decoder.raw_feed(remaining__, ret);
                    match err_ {
                        Some(err_) => {
                            ret.write_char('\ufffd');
                            remaining_ = err_.problem + err_.remaining;
                            assert!(!remaining_.is_empty());
                            remaining_.shift();
                        }
                        None => break
                    }
                }

                let newerr = decoder.raw_feed(remaining, ret);
                if newerr.is_none() { return; }
                err = newerr.unwrap();
            }
        }

        let mut ret = ~"";
        if feed_first_bytes {
            let err = self.decoder.raw_feed(self.first_bytes, &mut ret as &mut types::StringWriter);
            if err.is_some() {
                if self.fatal { return Err(~"EncodingError"); }
                handle_error(&mut self.decoder, err.unwrap(), &mut ret as &mut types::StringWriter);
            }
        }
        if input.is_some() {
            let err = self.decoder.raw_feed(input.unwrap(), &mut ret as &mut types::StringWriter);
            if err.is_some() {
                if self.fatal { return Err(~"EncodingError"); }
                handle_error(&mut self.decoder, err.unwrap(), &mut ret as &mut types::StringWriter);
            }
        }
        if !options.stream {
            // this is a bit convoluted. `Decoder.raw_finish` always destroys
            // the current decoding state, but the specification requires that the state should be
            // *resurrected* if the problem is two or more byte long!
            // we temporarily recreate the decoder in this case.
            let mut decoder = self.encoding.decoder();
            swap(&mut decoder, &mut self.decoder);
            loop {
                let err = decoder.raw_finish(&mut ret as &mut types::StringWriter);
                if err.is_none() { break; }
                if self.fatal { return Err(~"EncodingError"); }
                decoder = self.encoding.decoder();
                handle_error(&mut decoder, err.unwrap(), &mut ret as &mut types::StringWriter);
            }
        }
        Ok(ret)
    }
}

#[deriving(Eq,Clone)]
pub struct TextEncodeOptions {
    stream: bool
}

impl TextEncodeOptions {
    pub fn new() -> TextEncodeOptions { TextEncodeOptions { stream: false } }
}

pub struct TextEncoder {
    whatwg_name: &'static str,
    encoding: &'static types::Encoding,
    encoder: ~types::Encoder,
}

impl TextEncoder {
    pub fn new(label: Option<~str>) -> Result<TextEncoder,~str> {
        let label = label.unwrap_or(~"utf-8");
        let ret = encoding_from_label(label);
        if ret.is_none() { return Err(~"TypeError"); }
        let (encoding, whatwg_name) = ret.unwrap();
        if whatwg_name != "utf-8" && whatwg_name != "utf-16le" && whatwg_name != "utf-16be" {
            return Err(~"TypeError");
        }

        Ok(TextEncoder { whatwg_name: whatwg_name, encoding: encoding,
                         encoder: encoding.encoder() })
    }

    pub fn encoding(&self) -> ~str {
        self.whatwg_name.to_owned()
    }

    // XXX conflicts with `types::Encoder::encode`
    pub fn encode_buffer(&mut self, input: Option<&str>) -> Result<~[u8],~str> {
        self.encode_buffer_with_options(input, TextEncodeOptions::new())
    }

    pub fn encode_buffer_with_options(&mut self, input: Option<&str>,
                                      options: TextEncodeOptions) -> Result<~[u8],~str> {
        let mut ret = ~[];
        if input.is_some() {
            let err = self.encoder.raw_feed(input.unwrap(), &mut ret as &mut types::ByteWriter);
            if err.is_some() { return Ok(ret); }
        }
        if !options.stream {
            let mut encoder = self.encoding.encoder();
            swap(&mut encoder, &mut self.encoder);
            let err = encoder.raw_finish(&mut ret as &mut types::ByteWriter);
            if err.is_some() { return Ok(ret); }
        }
        Ok(ret)
    }
}
*/


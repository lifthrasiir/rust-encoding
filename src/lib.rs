// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!

# Rust-encoding

Character encoding support for Rust.
It is based on [WHATWG Encoding Standard](http://encoding.spec.whatwg.org/),
and also provides an advanced interface for error detection and recovery.

## Simple Usage

To encode a string:

~~~~ {.rust}
use encoding::{Encoding, EncodeStrict};
use encoding::all::ISO_8859_1;

assert_eq!(ISO_8859_1.encode("caf\xe9", EncodeStrict),
           Ok(vec!(99,97,102,233)));
~~~~

To encode a string with unrepresentable characters:

~~~~ {.rust}
use encoding::{Encoding, EncodeStrict, EncodeReplace, EncodeIgnore, EncodeNcrEscape};
use encoding::all::ISO_8859_2;

assert!(ISO_8859_2.encode("Acme\xa9", EncodeStrict).is_err());
assert_eq!(ISO_8859_2.encode("Acme\xa9", EncodeReplace),
           Ok(vec!(65,99,109,101,63)));
assert_eq!(ISO_8859_2.encode("Acme\xa9", EncodeIgnore),
           Ok(vec!(65,99,109,101)));
assert_eq!(ISO_8859_2.encode("Acme\xa9", EncodeNcrEscape),
           Ok(vec!(65,99,109,101,38,35,49,54,57,59)));
~~~~

To decode a byte sequence:

~~~~ {.rust}
use encoding::{Encoding, DecodeStrict};
use encoding::all::ISO_8859_1;

assert_eq!(ISO_8859_1.decode([99,97,102,233], DecodeStrict),
           Ok("caf\xe9".to_string()));
~~~~

To decode a byte sequence with invalid sequences:

~~~~ {.rust}
use encoding::{Encoding, DecodeStrict, DecodeReplace, DecodeIgnore};
use encoding::all::ISO_8859_6;

assert!(ISO_8859_6.decode([65,99,109,101,169], DecodeStrict).is_err());
assert_eq!(ISO_8859_6.decode([65,99,109,101,169], DecodeReplace),
           Ok("Acme\ufffd".to_string()));
assert_eq!(ISO_8859_6.decode([65,99,109,101,169], DecodeIgnore),
           Ok("Acme".to_string()));
~~~~

A practical example of custom encoder traps:

~~~~ {.rust}
use encoding::{Encoding, Encoder, ByteWriter, EncoderTrap, EncoderCall, DecodeStrict};
use encoding::all::ASCII;

// hexadecimal numeric character reference replacement
fn hex_ncr_escape(_encoder: &mut Encoder, input: &str, output: &mut ByteWriter) -> bool {
    let escapes: Vec<String> =
        input.chars().map(|ch| format!("&#x{:x};", ch as int)).collect();
    let escapes = escapes.concat();
    output.write_bytes(escapes.as_bytes());
    true
}
#[allow(non_uppercase_statics)] static HexNcrEscape: EncoderTrap = EncoderCall(hex_ncr_escape);

let orig = "Hello, 世界!".to_string();
let encoded = ASCII.encode(orig.as_slice(), HexNcrEscape).unwrap();
assert_eq!(ASCII.decode(encoded.as_slice(), DecodeStrict),
           Ok("Hello, &#x4e16;&#x754c;!".to_string()));
~~~~

Getting the encoding from the string label, as specified in WHATWG Encoding standard:

~~~~ {.rust}
use encoding::{Encoding, DecodeReplace};
use encoding::label::encoding_from_whatwg_label;
use encoding::all::WINDOWS_949;

let euckr = encoding_from_whatwg_label("euc-kr").unwrap();
assert_eq!(euckr.name(), "windows-949");
assert_eq!(euckr.whatwg_name(), Some("euc-kr")); // for the sake of compatibility
let broken = &[0xbf, 0xec, 0xbf, 0xcd, 0xff, 0xbe, 0xd3];
assert_eq!(euckr.decode(broken, DecodeReplace),
           Ok("\uc6b0\uc640\ufffd\uc559".to_string()));

// corresponding rust-encoding native API:
assert_eq!(WINDOWS_949.decode(broken, DecodeReplace),
           Ok("\uc6b0\uc640\ufffd\uc559".to_string()));
~~~~

## Detailed Usage

There are three main entry points to rust-encoding.

**`Encoding`** is a single character encoding.
It contains `encode` and `decode` methods for converting `String` to `Vec<u8>` and vice versa.
For the error handling, they receive **traps** (`EncoderCall` and `DecoderCall` respectively)
which replace any error with some string (e.g. `U+FFFD`) or sequence (e.g. `?`).
You can also use `EncodeStrict` and `DecodeStrict` traps to stop on an error.

There are two ways to get `Encoding`:

* `encoding::all` has static items for every supported encoding.
  You should use them when the encoding would not change or only handful of them are required.
  Combined with link-time optimization, any unused encoding would be discarded from the binary.
* `encoding::label` has functions to dynamically get an encoding from given string ("label").
  They will return a static reference to the encoding, which type is also known as `EncodingRef`.
  It is useful when a list of required encodings is not available in advance,
  but it will result in the larger binary and missed optimization opportunities.

**`Encoder`** is an experimental incremental encoder.
At each step of `raw_feed`, it receives a slice of string
and emits any encoded bytes to a generic `ByteWriter` (normally `Vec<u8>`).
It will stop at the first error if any, and would return a `CodecError` struct in that case.
The caller is responsible for calling `raw_finish` at the end of encoding process.

**`Decoder`** is an experimental incremental decoder.
At each step of `raw_feed`, it receives a slice of byte sequence
and emits any decoded characters to a generic `StringWriter` (normally `String`).
Otherwise it is identical to `Encoder`s.

One should prefer `Encoding::{encode,decode}` as a primary interface.
`Encoder` and `Decoder` is experimental and can change substantially.
See the additional documents on `encoding::types` module for more information on them.

## Supported Encodings

Rust-encoding covers all encodings specified by WHATWG Encoding Standard and some more:

* 7-bit strict ASCII (`ascii`)
* UTF-8 (`utf-8`)
* UTF-16 in little endian (`utf-16` or `utf-16le`) and big endian (`utf-16be`)
* All single byte encoding in WHATWG Encoding Standard:
    * IBM code page 866
    * ISO 8859-{2,3,4,5,6,7,8,10,13,14,15,16}
    * KOI8-R, KOI8-U
    * MacRoman (`macintosh`), Macintosh Cyrillic encoding (`x-mac-cyrillic`)
    * Windows code pages 874, 1250, 1251, 1252 (instead of ISO 8859-1), 1253,
      1254 (instead of ISO 8859-9), 1255, 1256, 1257, 1258
* All multi byte encodings in WHATWG Encoding Standard:
    * Windows code page 949 (`euc-kr`, since the strict EUC-KR is hardly used)
    * EUC-JP and Windows code page 932 (`shift_jis`,
      since it's the most widespread extension to Shift_JIS)
    * ISO-2022-JP with asymmetric JIS X 0212 support
    * GB 18030
    * HZ
    * Big5-2003 with HKSCS-2008 extensions
* ISO 8859-1 (distinct from Windows code page 1252)

Parenthesized names refer to the encoding's primary name assigned by WHATWG Encoding Standard.

Many legacy character encodings lack the proper specification,
and even those that have a specification are highly dependent of the actual implementation.
Consequently one should be careful when picking a desired character encoding.
The only standards reliable in this regard are WHATWG Encoding Standard and
[vendor-provided mappings from the Unicode consortium](http://www.unicode.org/Public/MAPPINGS/).
Whenever in doubt, look at the source code and specifications for detailed explanations.

*/

#![crate_name = "encoding"]
#![crate_type = "lib"]
#![comment = "Character encoding support for Rust"]
#![license = "MIT"]

#![allow(unknown_features)] // temporary measure for rust-nightly 2014-10-04
#![feature(globs, macro_rules, slicing_syntax)]

#![allow(experimental)]

#[cfg(test)] extern crate test;

pub use self::types::{CodecError, ByteWriter, StringWriter,
                      Encoder, Decoder, EncodingRef, Encoding,
                      EncoderTrapFunc, DecoderTrapFunc, DecoderTrap,
                      DecodeStrict, DecodeReplace, DecodeIgnore,
                      EncoderTrap, EncodeStrict, EncodeReplace,
                      EncodeIgnore, EncodeNcrEscape, decode,
                      EncoderCall, DecoderCall}; // reexport

mod util;
#[cfg(test)] mod testutils;

pub mod types;

/// Indices used for character encoding implementation. Semi-internal.
#[unstable]
pub mod index {
    pub mod ibm866;
    pub mod iso_8859_2;
    pub mod iso_8859_3;
    pub mod iso_8859_4;
    pub mod iso_8859_5;
    pub mod iso_8859_6;
    pub mod iso_8859_7;
    pub mod iso_8859_8;
    pub mod iso_8859_10;
    pub mod iso_8859_13;
    pub mod iso_8859_14;
    pub mod iso_8859_15;
    pub mod iso_8859_16;
    pub mod koi8_r;
    pub mod koi8_u;
    pub mod macintosh;
    pub mod windows_874;
    pub mod windows_1250;
    pub mod windows_1251;
    pub mod windows_1252;
    pub mod windows_1253;
    pub mod windows_1254;
    pub mod windows_1255;
    pub mod windows_1256;
    pub mod windows_1257;
    pub mod windows_1258;
    pub mod x_mac_cyrillic;
    pub mod big5;
    pub mod euc_kr;
    pub mod gb18030;
    pub mod gb18030_ranges;
    pub mod jis0208;
    pub mod jis0212;
}

/// Codec implementations.
#[unstable]
pub mod codec {
    pub mod error;
    pub mod ascii;
    pub mod singlebyte;
    pub mod utf_8;
    pub mod utf_16;
    pub mod korean;
    pub mod japanese;
    pub mod simpchinese;
    pub mod tradchinese;
    pub mod whatwg;
}

pub mod all;
pub mod label;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        fn test_one(input: &[u8], expected_result: &str, expected_encoding: &str) {
            let (result, used_encoding) = decode(
                input, DecodeStrict, all::ISO_8859_1 as EncodingRef);
            let result = result.unwrap();
            assert_eq!(used_encoding.name(), expected_encoding);
            assert_eq!(result[], expected_result);
        }

        test_one([0xEF, 0xBB, 0xBF, 0xC3, 0xA9], "é", "utf-8");
        test_one([0xC3, 0xA9], "Ã©", "iso-8859-1");

        test_one([0xFE, 0xFF, 0x00, 0xE9], "é", "utf-16be");
        test_one([0x00, 0xE9], "\x00é", "iso-8859-1");

        test_one([0xFF, 0xFE, 0xE9, 0x00], "é", "utf-16le");
        test_one([0xE9, 0x00], "é\x00", "iso-8859-1");
    }
}


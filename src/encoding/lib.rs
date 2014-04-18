// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * Character encoding support for Rust. It is based on [WHATWG Encoding
 * Standard](http://encoding.spec.whatwg.org/), and also provides an advanced interface for
 * error detection and recovery.
 */

#![crate_id = "encoding#0.1.0"]
#![crate_type = "lib"]
#![comment = "Character encoding support for Rust"]
#![license = "MIT"]

#![feature(globs, macro_rules)]

pub use self::types::*; // reexport

mod util;
#[cfg(test)] mod testutils;

pub mod types;

/// Indices used for character encoding implementation. Semi-internal.
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
    pub mod gbk;
    pub mod gb18030;
    pub mod jis0208;
    pub mod jis0212;
}

/// Codec implementations.
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
    fn test_readme() {
        assert_eq!(all::ISO_8859_1.encode("caf\xe9", EncodeStrict), Ok(vec!(99,97,102,233)));

        assert!(all::ISO_8859_2.encode("Acme\xa9", EncodeStrict).is_err());
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", EncodeReplace), Ok(vec!(65,99,109,101,63)));
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", EncodeIgnore), Ok(vec!(65,99,109,101)));
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", EncodeNcrEscape),
                   Ok(vec!(65,99,109,101,38,35,49,54,57,59))); // Acme&#169;

        assert_eq!(all::ISO_8859_1.decode([99,97,102,233], DecodeStrict),
                   Ok(StrBuf::from_str("caf\xe9")));

        assert!(all::ISO_8859_6.decode([65,99,109,101,169], DecodeStrict).is_err());
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], DecodeReplace),
                   Ok(StrBuf::from_str("Acme\ufffd")));
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], DecodeIgnore),
                   Ok(StrBuf::from_str("Acme")));
    }

    #[test]
    fn test_readme_hex_ncr_escape() {
        // hexadecimal numeric character reference replacement
        fn hex_ncr_escape(_encoder: &Encoder, input: &str, output: &mut ByteWriter) -> bool {
            let escapes: Vec<~str> =
                input.chars().map(|ch| format!("&\\#x{:x};", ch as int)).collect();
            let escapes = escapes.concat();
            output.write_bytes(escapes.as_bytes());
            true
        }
        static HexNcrEscape: EncoderTrap = EncoderTrap(hex_ncr_escape);
        let orig = ~"Hello, 世界!";
        let encoded = all::ASCII.encode(orig, HexNcrEscape).unwrap();
        let decoded = all::ASCII.decode(encoded.as_slice(), DecodeStrict).unwrap();
        assert_eq!(decoded, StrBuf::from_str("Hello, &#x4e16;&#x754c;!"));
    }

    #[test]
    fn test_readme_whatwg() {
        let euckr = label::encoding_from_whatwg_label("euc-kr").unwrap();
        assert_eq!(euckr.name(), "windows-949");
        assert_eq!(euckr.whatwg_name(), Some("euc-kr")); // for the sake of compatibility
        let broken = &[0xbf, 0xec, 0xbf, 0xcd, 0xff, 0xbe, 0xd3];
        assert_eq!(euckr.decode(broken, DecodeReplace),
                   Ok(StrBuf::from_str("\uc6b0\uc640\ufffd\uc559")));

        // corresponding rust-encoding native API:
        assert_eq!(all::WINDOWS_949.decode(broken, DecodeReplace),
                   Ok(StrBuf::from_str("\uc6b0\uc640\ufffd\uc559")));
    }


    #[test]
    fn test_decode() {
        fn test_one(input: &[u8], expected_result: &str, expected_encoding: &str) {
            let (result, used_encoding) = decode(
                input, DecodeStrict, all::ISO_8859_1 as EncodingRef);
            let result = result.unwrap();
            assert_eq!(used_encoding.name(), expected_encoding);
            assert_eq!(result.as_slice(), expected_result);
        }

        test_one([0xEF, 0xBB, 0xBF, 0xC3, 0xA9], "é", "utf-8");
        test_one([0xC3, 0xA9], "Ã©", "iso-8859-1");

        test_one([0xFE, 0xFF, 0x00, 0xE9], "é", "utf-16be");
        test_one([0x00, 0xE9], "\x00é", "iso-8859-1");

        test_one([0xFF, 0xFE, 0xE9, 0x00], "é", "utf-16le");
        test_one([0xE9, 0x00], "é\x00", "iso-8859-1");
    }
}


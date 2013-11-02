// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

/*!
 * Character encoding support for Rust. It is based on [WHATWG Encoding
 * Standard](http://encoding.spec.whatwg.org/), and also provides an advanced interface for
 * error detection and recovery.
 */

#[link(name = "encoding",
       vers = "0.1.0",
       uuid = "05AC43C2-6959-409F-B95A-C58EBF217527",
       url = "https://github.com/lifthrasiir/rust-encoding/")];

#[comment = "Character encoding support for Rust"];
#[license = "MIT"];
#[crate_type = "lib"];

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
    pub mod jis0208;
    pub mod jis0212;
}

/// Codec implementations.
pub mod codec {
    pub mod error;
    pub mod ascii;
    pub mod singlebyte;
    pub mod utf_8;
    pub mod korean;
    pub mod japanese;
    pub mod simpchinese;
    pub mod whatwg;
}

pub mod all;
pub mod label;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readme() {
        assert_eq!(all::ISO_8859_2.encode("caf\xe9", Strict), Ok(~[99,97,102,233]));
        assert!(all::ISO_8859_2.encode("Acme\xa9", Strict).is_err());
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", Replace), Ok(~[65,99,109,101,63]));
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", Ignore), Ok(~[65,99,109,101]));
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", NcrEscape),
                   Ok(~[65,99,109,101,38,35,49,54,57,59])); // Acme&#169;

        assert_eq!(all::ISO_8859_2.decode([99,97,102,233], Strict), Ok(~"caf\xe9"));
        assert!(all::ISO_8859_6.decode([65,99,109,101,169], Strict).is_err());
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], Replace), Ok(~"Acme\ufffd"));
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], Ignore), Ok(~"Acme"));
    }

    #[test]
    fn test_readme_hex_ncr_escape() {
        // hexadecimal numeric character reference replacement
        fn hex_ncr_escape(_encoder: &Encoder, input: &str, output: &mut ByteWriter) -> bool {
            let escapes: ~[~str] =
                input.iter().map(|ch| format!("&\\#x{:x};", ch as int)).collect();
            let escapes = escapes.concat();
            output.write_bytes(escapes.as_bytes());
            true
        }
        static HexNcrEscape: Trap = EncoderTrap(hex_ncr_escape);

        let orig = ~"Hello, 世界!";
        let encoded = all::ASCII.encode(orig, HexNcrEscape).unwrap();
        let decoded = all::ASCII.decode(encoded, Strict).unwrap();
        assert_eq!(decoded, ~"Hello, &#x4e16;&#x754c;!");
    }

    #[test]
    fn test_readme_whatwg() {
        let euckr = label::encoding_from_whatwg_label("euc-kr").unwrap();
        assert_eq!(euckr.name(), "windows-949");
        assert_eq!(euckr.whatwg_name(), Some("euc-kr")); // for the sake of compatibility
        let broken = &[0xbf, 0xec, 0xbf, 0xcd, 0xff, 0xbe, 0xd3];
        assert_eq!(euckr.decode(broken, Replace), Ok(~"\uc6b0\uc640\ufffd\uc559"));

        // corresponding rust-encoding native API:
        assert_eq!(all::WINDOWS_949.decode(broken, Replace), Ok(~"\uc6b0\uc640\ufffd\uc559"));
    }
}


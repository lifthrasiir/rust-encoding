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
}

pub mod all;

pub mod whatwg;

#[cfg(test)]
mod tests {
    use std::str;
    use super::*;

    #[test]
    fn test_readme() {
        assert_eq!(all::ISO_8859_2.encode("caf\xe9", Strict), Ok(~[99,97,102,233]));
        assert!(all::ISO_8859_2.encode("Acme\xa9", Strict).is_err());
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", Replace), Ok(~[65,99,109,101,63]));
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", Ignore), Ok(~[65,99,109,101]));
        let trap: &fn(&str) -> ~[u8] = |_| ~[1,2,3];
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", trap), Ok(~[65,99,109,101,1,2,3]));

        assert_eq!(all::ISO_8859_2.decode([99,97,102,233], Strict), Ok(~"caf\xe9"));
        assert!(all::ISO_8859_6.decode([65,99,109,101,169], Strict).is_err());
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], Replace), Ok(~"Acme\ufffd"));
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], Ignore), Ok(~"Acme"));
        let trap: &fn(&[u8]) -> ~str = |_| ~"whatever";
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], trap), Ok(~"Acmewhatever"));
    }

    #[test]
    fn test_readme_surrogate_escape() {
        pub struct SurrogateEscape;
        impl<T:Encoding> DecoderTrap<T> for SurrogateEscape {
            // converts invalid single bytes 80..FF to invalid surrogates U+DC80..DCFF
            fn decoder_trap(&mut self, _encoding: &T, input: &[u8]) -> Option<~str> {
                let chars: ~[char] =
                    input.iter().map(|&c| (c as uint + 0xdc00) as char).collect();
                Some(str::from_chars(chars))
            }
        }
        impl<T:Encoding> EncoderTrap<T> for SurrogateEscape {
            // converts invalid surrogates U+DC80..DCFF back to single bytes 80..FF
            // this is an illustrative example, the actual routine would be a bit more complex.
            fn encoder_trap(&mut self, _encoding: &T, input: &str) -> Option<~[u8]> {
                let chars: ~[char] = input.iter().collect();
                if chars.len() == 1 && '\udc80' <= chars[0] && chars[0] <= '\udcff' {
                    Some(~[(chars[0] as uint - 0xdc00) as u8])
                } else {
                    None
                }
            }
        }

        let orig = ~[0xea,0xb0,0x80,0xfe,0x20];
        let decoded = all::UTF_8.decode(orig, SurrogateEscape).unwrap();
        assert_eq!(decoded.clone(), ~"\uac00\udcfe\u0020");
        let encoded = all::UTF_8.encode(decoded, SurrogateEscape).unwrap();
        assert_eq!(orig, encoded);
    }

    #[test]
    fn test_readme_whatwg() {
        let mut euckr = whatwg::TextDecoder::new(Some(~"euc-kr")).unwrap();
        assert_eq!(euckr.encoding(), "euc-kr"); // although it is actually windows-949
        let broken = &[0xbf, 0xec, 0xbf, 0xcd, 0xff, 0xbe, 0xd3];
        assert_eq!(euckr.decode_buffer(Some(broken)), Ok(~"\uc6b0\uc640\ufffd\uc559"));

        // this is different from rust-encoding's default behavior:
        let decoded = all::WINDOWS_949.decode(broken, Replace);
        assert_eq!(decoded, Ok(~"\uc6b0\uc640\ufffd\ufffd"));

        // explanation:
        //   what WHATWG expects:        [BF EC] [BF CD] [FF]* [BE D3]
        //   what rust-encoding expects: [BF EC] [BF CD] [FF BE]* [D3]*
        //   sequences marked * are considered invalid and replaced by U+FFFD.
    }
}


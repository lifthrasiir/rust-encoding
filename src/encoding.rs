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
}

/// Codec implementations.
pub mod codec {
    pub mod ascii;
    pub mod singlebyte;
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
        let trap: &fn(&str) -> ~[u8] = |_| ~[1,2,3];
        assert_eq!(all::ISO_8859_2.encode("Acme\xa9", trap), Ok(~[65,99,109,101,1,2,3]));

        assert_eq!(all::ISO_8859_2.decode([99,97,102,233], Strict), Ok(~"caf\xe9"));
        assert!(all::ISO_8859_6.decode([65,99,109,101,169], Strict).is_err());
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], Replace), Ok(~"Acme\ufffd"));
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], Ignore), Ok(~"Acme"));
        let trap: &fn(&[u8]) -> ~str = |_| ~"whatever";
        assert_eq!(all::ISO_8859_6.decode([65,99,109,101,169], trap), Ok(~"Acmewhatever"));

        let latin2 = label::get_encoding("Latin2").unwrap();
        assert_eq!(latin2.name(), ~"iso-8859-2");
        assert_eq!(latin2.encode("caf\xe9", Strict), Ok(~[99,97,102,233]));
    }
}


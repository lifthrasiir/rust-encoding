// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A list of all supported encodings. Useful for encodings fixed in the compile time.

use super::{index, codec};

pub static ASCII: &'static codec::ascii::ASCIIEncoding =
    &codec::ascii::ASCIIEncoding;

macro_rules! singlebyte(
    (var=$var:ident, mod=$module:ident, name=$name:expr) => (
        pub static $var: &'static codec::singlebyte::SingleByteEncoding =
            &codec::singlebyte::SingleByteEncoding {
                name: $name,
                index_forward: index::$module::forward,
                index_backward: index::$module::backward,
            };
    )
)

singlebyte!(var=IBM866, mod=ibm866, name="ibm866")
singlebyte!(var=ISO_8859_2, mod=iso_8859_2, name="iso-8859-2")
singlebyte!(var=ISO_8859_3, mod=iso_8859_3, name="iso-8859-3")
singlebyte!(var=ISO_8859_4, mod=iso_8859_4, name="iso-8859-4")
singlebyte!(var=ISO_8859_5, mod=iso_8859_5, name="iso-8859-5")
singlebyte!(var=ISO_8859_6, mod=iso_8859_6, name="iso-8859-6")
singlebyte!(var=ISO_8859_7, mod=iso_8859_7, name="iso-8859-7")
singlebyte!(var=ISO_8859_8, mod=iso_8859_8, name="iso-8859-8")
singlebyte!(var=ISO_8859_8_I, mod=iso_8859_8, name="iso-8859-8-i")
singlebyte!(var=ISO_8859_10, mod=iso_8859_10, name="iso-8859-10")
singlebyte!(var=ISO_8859_13, mod=iso_8859_13, name="iso-8859-13")
singlebyte!(var=ISO_8859_14, mod=iso_8859_14, name="iso-8859-14")
singlebyte!(var=ISO_8859_15, mod=iso_8859_15, name="iso-8859-15")
singlebyte!(var=ISO_8859_16, mod=iso_8859_16, name="iso-8859-16")
singlebyte!(var=KOI8_R, mod=koi8_r, name="koi8-r")
singlebyte!(var=KOI8_U, mod=koi8_u, name="koi8-u")
singlebyte!(var=MACINTOSH, mod=macintosh, name="macintosh")
singlebyte!(var=WINDOWS_874, mod=windows_874, name="windows-874")
singlebyte!(var=WINDOWS_1250, mod=windows_1250, name="windows-1250")
singlebyte!(var=WINDOWS_1251, mod=windows_1251, name="windows-1251")
singlebyte!(var=WINDOWS_1252, mod=windows_1252, name="windows-1252")
singlebyte!(var=WINDOWS_1253, mod=windows_1253, name="windows-1253")
singlebyte!(var=WINDOWS_1254, mod=windows_1254, name="windows-1254")
singlebyte!(var=WINDOWS_1255, mod=windows_1255, name="windows-1255")
singlebyte!(var=WINDOWS_1256, mod=windows_1256, name="windows-1256")
singlebyte!(var=WINDOWS_1257, mod=windows_1257, name="windows-1257")
singlebyte!(var=WINDOWS_1258, mod=windows_1258, name="windows-1258")
singlebyte!(var=X_MAC_CYRILLIC, mod=x_mac_cyrillic, name="x-mac-cyrillic")


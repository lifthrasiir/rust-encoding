// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A list of all supported encodings. Useful for encodings fixed in the compile time.

use index;
use codec;

macro_rules! unique(
    (var=$var:ident, mod=$module:ident, val=$val:ident) => (
        pub static $var: &'static codec::$module::$val = &codec::$module::$val;
    )
)

macro_rules! singlebyte_whatwg(
    (var=$var:ident, mod=$module:ident, name=$name:expr) => (
        singlebyte_whatwg!(var=$var, mod=$module, name=$name, whatwg=$name)
    );
    (var=$var:ident, mod=$module:ident, name=$name:expr, whatwg=$whatwg:expr) => (
        pub static $var: &'static codec::singlebyte::SingleByteEncoding =
            &codec::singlebyte::SingleByteEncoding {
                name: $name,
                whatwg_name: Some($whatwg),
                index_forward: index::$module::forward,
                index_backward: index::$module::backward,
            };
    )
)

unique!(var=ERROR, mod=error, val=ErrorEncoding)
unique!(var=ASCII, mod=ascii, val=ASCIIEncoding)
singlebyte_whatwg!(var=IBM866, mod=ibm866, name="ibm866")
singlebyte_whatwg!(var=ISO_8859_2, mod=iso_8859_2, name="iso-8859-2")
singlebyte_whatwg!(var=ISO_8859_3, mod=iso_8859_3, name="iso-8859-3")
singlebyte_whatwg!(var=ISO_8859_4, mod=iso_8859_4, name="iso-8859-4")
singlebyte_whatwg!(var=ISO_8859_5, mod=iso_8859_5, name="iso-8859-5")
singlebyte_whatwg!(var=ISO_8859_6, mod=iso_8859_6, name="iso-8859-6")
singlebyte_whatwg!(var=ISO_8859_7, mod=iso_8859_7, name="iso-8859-7")
singlebyte_whatwg!(var=ISO_8859_8, mod=iso_8859_8, name="iso-8859-8")
singlebyte_whatwg!(var=ISO_8859_10, mod=iso_8859_10, name="iso-8859-10")
singlebyte_whatwg!(var=ISO_8859_13, mod=iso_8859_13, name="iso-8859-13")
singlebyte_whatwg!(var=ISO_8859_14, mod=iso_8859_14, name="iso-8859-14")
singlebyte_whatwg!(var=ISO_8859_15, mod=iso_8859_15, name="iso-8859-15")
singlebyte_whatwg!(var=ISO_8859_16, mod=iso_8859_16, name="iso-8859-16")
singlebyte_whatwg!(var=KOI8_R, mod=koi8_r, name="koi8-r")
singlebyte_whatwg!(var=KOI8_U, mod=koi8_u, name="koi8-u")
singlebyte_whatwg!(var=MACINTOSH, mod=macintosh, name="macintosh")
singlebyte_whatwg!(var=WINDOWS_874, mod=windows_874, name="windows-874")
singlebyte_whatwg!(var=WINDOWS_1250, mod=windows_1250, name="windows-1250")
singlebyte_whatwg!(var=WINDOWS_1251, mod=windows_1251, name="windows-1251")
singlebyte_whatwg!(var=WINDOWS_1252, mod=windows_1252, name="windows-1252")
singlebyte_whatwg!(var=WINDOWS_1253, mod=windows_1253, name="windows-1253")
singlebyte_whatwg!(var=WINDOWS_1254, mod=windows_1254, name="windows-1254")
singlebyte_whatwg!(var=WINDOWS_1255, mod=windows_1255, name="windows-1255")
singlebyte_whatwg!(var=WINDOWS_1256, mod=windows_1256, name="windows-1256")
singlebyte_whatwg!(var=WINDOWS_1257, mod=windows_1257, name="windows-1257")
singlebyte_whatwg!(var=WINDOWS_1258, mod=windows_1258, name="windows-1258")
singlebyte_whatwg!(var=X_MAC_CYRILLIC, mod=x_mac_cyrillic, name="x-mac-cyrillic")
unique!(var=UTF_8, mod=utf_8, val=UTF8Encoding)
unique!(var=WINDOWS_949, mod=korean, val=Windows949Encoding)
unique!(var=EUC_JP, mod=japanese, val=EUCJPEncoding)
unique!(var=SHIFT_JIS, mod=japanese, val=ShiftJISEncoding)

pub mod whatwg {
    use index;
    use codec;

    singlebyte_whatwg!(var=X_USER_DEFINED, mod=whatwg_x_user_defined,
                       name="pua-mapped-binary", whatwg="x-user-defined")
    unique!(var=REPLACEMENT, mod=whatwg, val=EncoderOnlyUTF8Encoding)
}


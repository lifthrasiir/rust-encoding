// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! A list of all supported encodings. Useful for encodings fixed in the compile time.

use index;
use codec;

macro_rules! unique(
    (var=$var:ident, mod=$($module:ident)::+, val=$val:ident) => (
        pub static $var: &'static $($module)::+::$val = &$($module)::+::$val;
    )
)

macro_rules! singlebyte(
    (var=$var:ident, mod=$($module:ident)::+, name=$name:expr) => (
        singlebyte!(var=$var, mod=$($module)::+, name=$name, whatwg=None)
    );
    (var=$var:ident, mod=$($module:ident)::+, name|whatwg=$name:expr) => (
        singlebyte!(var=$var, mod=$($module)::+, name=$name, whatwg=Some($name))
    );
    (var=$var:ident, mod=$($module:ident)::+, name=$name:expr, whatwg=$whatwg:expr) => (
        pub static $var: &'static codec::singlebyte::SingleByteEncoding =
            &codec::singlebyte::SingleByteEncoding {
                name: $name,
                whatwg_name: $whatwg,
                index_forward: $($module)::+::forward,
                index_backward: $($module)::+::backward,
            };
    )
)

unique!(var=ERROR, mod=codec::error, val=ErrorEncoding)
unique!(var=ASCII, mod=codec::ascii, val=ASCIIEncoding)
singlebyte!(var=IBM866, mod=index::ibm866, name|whatwg="ibm866")
singlebyte!(var=ISO_8859_1, mod=codec::singlebyte::iso_8859_1, name="iso-8859-1")
singlebyte!(var=ISO_8859_2, mod=index::iso_8859_2, name|whatwg="iso-8859-2")
singlebyte!(var=ISO_8859_3, mod=index::iso_8859_3, name|whatwg="iso-8859-3")
singlebyte!(var=ISO_8859_4, mod=index::iso_8859_4, name|whatwg="iso-8859-4")
singlebyte!(var=ISO_8859_5, mod=index::iso_8859_5, name|whatwg="iso-8859-5")
singlebyte!(var=ISO_8859_6, mod=index::iso_8859_6, name|whatwg="iso-8859-6")
singlebyte!(var=ISO_8859_7, mod=index::iso_8859_7, name|whatwg="iso-8859-7")
singlebyte!(var=ISO_8859_8, mod=index::iso_8859_8, name|whatwg="iso-8859-8")
singlebyte!(var=ISO_8859_10, mod=index::iso_8859_10, name|whatwg="iso-8859-10")
singlebyte!(var=ISO_8859_13, mod=index::iso_8859_13, name|whatwg="iso-8859-13")
singlebyte!(var=ISO_8859_14, mod=index::iso_8859_14, name|whatwg="iso-8859-14")
singlebyte!(var=ISO_8859_15, mod=index::iso_8859_15, name|whatwg="iso-8859-15")
singlebyte!(var=ISO_8859_16, mod=index::iso_8859_16, name|whatwg="iso-8859-16")
singlebyte!(var=KOI8_R, mod=index::koi8_r, name|whatwg="koi8-r")
singlebyte!(var=KOI8_U, mod=index::koi8_u, name|whatwg="koi8-u")
singlebyte!(var=MACINTOSH, mod=index::macintosh, name|whatwg="macintosh")
singlebyte!(var=WINDOWS_874, mod=index::windows_874, name|whatwg="windows-874")
singlebyte!(var=WINDOWS_1250, mod=index::windows_1250, name|whatwg="windows-1250")
singlebyte!(var=WINDOWS_1251, mod=index::windows_1251, name|whatwg="windows-1251")
singlebyte!(var=WINDOWS_1252, mod=index::windows_1252, name|whatwg="windows-1252")
singlebyte!(var=WINDOWS_1253, mod=index::windows_1253, name|whatwg="windows-1253")
singlebyte!(var=WINDOWS_1254, mod=index::windows_1254, name|whatwg="windows-1254")
singlebyte!(var=WINDOWS_1255, mod=index::windows_1255, name|whatwg="windows-1255")
singlebyte!(var=WINDOWS_1256, mod=index::windows_1256, name|whatwg="windows-1256")
singlebyte!(var=WINDOWS_1257, mod=index::windows_1257, name|whatwg="windows-1257")
singlebyte!(var=WINDOWS_1258, mod=index::windows_1258, name|whatwg="windows-1258")
singlebyte!(var=X_MAC_CYRILLIC, mod=index::x_mac_cyrillic, name|whatwg="x-mac-cyrillic")
unique!(var=UTF_8, mod=codec::utf_8, val=UTF8Encoding)
unique!(var=UTF_16LE, mod=codec::utf_16, val=UTF16LEEncoding)
unique!(var=UTF_16BE, mod=codec::utf_16, val=UTF16BEEncoding)
unique!(var=WINDOWS_949, mod=codec::korean, val=Windows949Encoding)
unique!(var=EUC_JP, mod=codec::japanese, val=EUCJPEncoding)
unique!(var=WINDOWS_31J, mod=codec::japanese, val=Windows31JEncoding)
unique!(var=ISO_2022_JP, mod=codec::japanese, val=ISO2022JPEncoding)
unique!(var=GB18030, mod=codec::simpchinese, val=GB18030Encoding)
unique!(var=HZ, mod=codec::simpchinese, val=HZEncoding)
unique!(var=BIG5_2003, mod=codec::tradchinese, val=BigFive2003Encoding)

pub mod whatwg {
    use codec;
    use index;

    singlebyte!(var=X_USER_DEFINED, mod=codec::whatwg::x_user_defined,
                name="pua-mapped-binary", whatwg=Some("x-user-defined"))
    singlebyte!(var=ISO_8859_8_I, mod=index::iso_8859_8, name|whatwg="iso-8859-8-i")
    unique!(var=REPLACEMENT, mod=codec::whatwg, val=EncoderOnlyUTF8Encoding)
}


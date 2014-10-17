// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! An interface for retrieving an encoding (or a set of encodings) from a string/numeric label.

use all;
use types::EncodingRef;

/// Returns an encoding from given label, defined in the WHATWG Encoding standard, if any.
/// Implements "get an encoding" algorithm: http://encoding.spec.whatwg.org/#decode
///
/// # Features
///
/// This function is only available with a `whatwg` feature, which is enabled by default.
#[stable]
#[cfg(any(feature="default", feature="whatwg"))]
pub fn encoding_from_whatwg_label(label: &str) -> Option<EncodingRef> {
    use std::ascii::StrAsciiExt;
    match label.trim_chars([' ', '\n', '\r', '\t', '\x0C'][]).to_ascii_lower()[] {
        "unicode-1-1-utf-8" |
        "utf-8" |
        "utf8" =>
            Some(all::UTF_8 as EncodingRef),
        "866" |
        "cp866" |
        "csibm866" |
        "ibm866" =>
            Some(all::IBM866 as EncodingRef),
        "csisolatin2" |
        "iso-8859-2" |
        "iso-ir-101" |
        "iso8859-2" |
        "iso88592" |
        "iso_8859-2" |
        "iso_8859-2:1987" |
        "l2" |
        "latin2" =>
            Some(all::ISO_8859_2 as EncodingRef),
        "csisolatin3" |
        "iso-8859-3" |
        "iso-ir-109" |
        "iso8859-3" |
        "iso88593" |
        "iso_8859-3" |
        "iso_8859-3:1988" |
        "l3" |
        "latin3" =>
            Some(all::ISO_8859_3 as EncodingRef),
        "csisolatin4" |
        "iso-8859-4" |
        "iso-ir-110" |
        "iso8859-4" |
        "iso88594" |
        "iso_8859-4" |
        "iso_8859-4:1988" |
        "l4" |
        "latin4" =>
            Some(all::ISO_8859_4 as EncodingRef),
        "csisolatincyrillic" |
        "cyrillic" |
        "iso-8859-5" |
        "iso-ir-144" |
        "iso8859-5" |
        "iso88595" |
        "iso_8859-5" |
        "iso_8859-5:1988" =>
            Some(all::ISO_8859_5 as EncodingRef),
        "arabic" |
        "asmo-708" |
        "csiso88596e" |
        "csiso88596i" |
        "csisolatinarabic" |
        "ecma-114" |
        "iso-8859-6" |
        "iso-8859-6-e" |
        "iso-8859-6-i" |
        "iso-ir-127" |
        "iso8859-6" |
        "iso88596" |
        "iso_8859-6" |
        "iso_8859-6:1987" =>
            Some(all::ISO_8859_6 as EncodingRef),
        "csisolatingreek" |
        "ecma-118" |
        "elot_928" |
        "greek" |
        "greek8" |
        "iso-8859-7" |
        "iso-ir-126" |
        "iso8859-7" |
        "iso88597" |
        "iso_8859-7" |
        "iso_8859-7:1987" |
        "sun_eu_greek" =>
            Some(all::ISO_8859_7 as EncodingRef),
        "csiso88598e" |
        "csisolatinhebrew" |
        "hebrew" |
        "iso-8859-8" |
        "iso-8859-8-e" |
        "iso-ir-138" |
        "iso8859-8" |
        "iso88598" |
        "iso_8859-8" |
        "iso_8859-8:1988" |
        "visual" =>
            Some(all::ISO_8859_8 as EncodingRef),
        "csiso88598i" |
        "iso-8859-8-i" |
        "logical" =>
            Some(all::whatwg::ISO_8859_8_I as EncodingRef),
        "csisolatin6" |
        "iso-8859-10" |
        "iso-ir-157" |
        "iso8859-10" |
        "iso885910" |
        "l6" |
        "latin6" =>
            Some(all::ISO_8859_10 as EncodingRef),
        "iso-8859-13" |
        "iso8859-13" |
        "iso885913" =>
            Some(all::ISO_8859_13 as EncodingRef),
        "iso-8859-14" |
        "iso8859-14" |
        "iso885914" =>
            Some(all::ISO_8859_14 as EncodingRef),
        "csisolatin9" |
        "iso-8859-15" |
        "iso8859-15" |
        "iso885915" |
        "iso_8859-15" |
        "l9" =>
            Some(all::ISO_8859_15 as EncodingRef),
        "iso-8859-16" =>
            Some(all::ISO_8859_16 as EncodingRef),
        "cskoi8r" |
        "koi" |
        "koi8" |
        "koi8-r" |
        "koi8_r" =>
            Some(all::KOI8_R as EncodingRef),
        "koi8-u" =>
            Some(all::KOI8_U as EncodingRef),
        "csmacintosh" |
        "mac" |
        "macintosh" |
        "x-mac-roman" =>
            Some(all::MAC_ROMAN as EncodingRef),
        "dos-874" |
        "iso-8859-11" |
        "iso8859-11" |
        "iso885911" |
        "tis-620" |
        "windows-874" =>
            Some(all::WINDOWS_874 as EncodingRef),
        "cp1250" |
        "windows-1250" |
        "x-cp1250" =>
            Some(all::WINDOWS_1250 as EncodingRef),
        "cp1251" |
        "windows-1251" |
        "x-cp1251" =>
            Some(all::WINDOWS_1251 as EncodingRef),
        "ansi_x3.4-1968" |
        "ascii" |
        "cp1252" |
        "cp819" |
        "csisolatin1" |
        "ibm819" |
        "iso-8859-1" |
        "iso-ir-100" |
        "iso8859-1" |
        "iso88591" |
        "iso_8859-1" |
        "iso_8859-1:1987" |
        "l1" |
        "latin1" |
        "us-ascii" |
        "windows-1252" |
        "x-cp1252" =>
            Some(all::WINDOWS_1252 as EncodingRef),
        "cp1253" |
        "windows-1253" |
        "x-cp1253" =>
            Some(all::WINDOWS_1253 as EncodingRef),
        "cp1254" |
        "csisolatin5" |
        "iso-8859-9" |
        "iso-ir-148" |
        "iso8859-9" |
        "iso88599" |
        "iso_8859-9" |
        "iso_8859-9:1989" |
        "l5" |
        "latin5" |
        "windows-1254" |
        "x-cp1254" =>
            Some(all::WINDOWS_1254 as EncodingRef),
        "cp1255" |
        "windows-1255" |
        "x-cp1255" =>
            Some(all::WINDOWS_1255 as EncodingRef),
        "cp1256" |
        "windows-1256" |
        "x-cp1256" =>
            Some(all::WINDOWS_1256 as EncodingRef),
        "cp1257" |
        "windows-1257" |
        "x-cp1257" =>
            Some(all::WINDOWS_1257 as EncodingRef),
        "cp1258" |
        "windows-1258" |
        "x-cp1258" =>
            Some(all::WINDOWS_1258 as EncodingRef),
        "x-mac-cyrillic" |
        "x-mac-ukrainian" =>
            Some(all::MAC_CYRILLIC as EncodingRef),
        "chinese" |
        "csgb2312" |
        "csiso58gb231280" |
        "gb18030" |
        "gb2312" |
        "gb_2312" |
        "gb_2312-80" |
        "gbk" |
        "iso-ir-58" |
        "x-gbk" =>
            Some(all::GB18030 as EncodingRef),
        "hz-gb-2312" =>
            Some(all::HZ as EncodingRef),
        "big5" |
        "big5-hkscs" |
        "cn-big5" |
        "csbig5" |
        "x-x-big5" =>
            Some(all::BIG5_2003 as EncodingRef),
        "cseucpkdfmtjapanese" |
        "euc-jp" |
        "x-euc-jp" =>
            Some(all::EUC_JP as EncodingRef),
        "csiso2022jp" |
        "iso-2022-jp" =>
            Some(all::ISO_2022_JP as EncodingRef),
        "csshiftjis" |
        "ms_kanji" |
        "shift-jis" |
        "shift_jis" |
        "sjis" |
        "windows-31j" |
        "x-sjis" =>
            Some(all::WINDOWS_31J as EncodingRef),
        "cseuckr" |
        "csksc56011987" |
        "euc-kr" |
        "iso-ir-149" |
        "korean" |
        "ks_c_5601-1987" |
        "ks_c_5601-1989" |
        "ksc5601" |
        "ksc_5601" |
        "windows-949" =>
            Some(all::WINDOWS_949 as EncodingRef),
        "csiso2022kr" |
        "iso-2022-kr" |
        "iso-2022-cn" |
        "iso-2022-cn-ext" =>
            Some(all::whatwg::REPLACEMENT as EncodingRef),
        "utf-16be" =>
            Some(all::UTF_16BE as EncodingRef),
        "utf-16" |
        "utf-16le" =>
            Some(all::UTF_16LE as EncodingRef),
        "x-user-defined" =>
            Some(all::whatwg::X_USER_DEFINED as EncodingRef),
        _ => None
    }
}

/// Returns an encoding from Windows code page number.
/// http://msdn.microsoft.com/en-us/library/windows/desktop/dd317756%28v=vs.85%29.aspx
/// Sometimes it can return a *superset* of the requested encoding, e.g. for several CJK encodings.
///
/// # Features
///
/// This function is dependent of available encodings from the `encoding::all` module.
#[experimental]
pub fn encoding_from_windows_code_page(cp: uint) -> Option<EncodingRef> {
    // XXX somehow `$feature` gets an error without a helper macro
    macro_rules! match_cp_inner(
        ($e:expr: $($(#[$m:meta])* $(cp$cp:pat)|+ = $enc:path;)+) => (
            match $e {
                $($(#[$m])* $($cp)|+ => Some($enc as EncodingRef),)+
                _ => None
            }
        )
    )
    macro_rules! match_cp(
        ($e:expr: $($({$($feature:tt)+})* $(cp$cp:pat)|+ = $enc:path;)+) => (
            match_cp_inner! { $e:
                $($(#[cfg(any(feature="default" $(, feature=$feature)+))])* $(cp$cp)|+ = $enc;)+
            }
        )
    )

    match_cp! { cp:
        {"whatwg" "unicode"} cp 65001 = all::UTF_8;
        {"whatwg" "cyrl"} cp 866 = all::IBM866;
        cp 28591 = all::ISO_8859_1;
        {"whatwg" "latn"} cp 28592 = all::ISO_8859_2;
        {"whatwg" "latn"} cp 28593 = all::ISO_8859_3;
        {"whatwg" "latn"} cp 28594 = all::ISO_8859_4;
        {"whatwg" "cyrl"} cp 28595 = all::ISO_8859_5;
        {"whatwg" "arab"} cp 28596 = all::ISO_8859_6;
        {"whatwg" "grek"} cp 28597 = all::ISO_8859_7;
        {"whatwg" "hebr"} cp 28598 = all::ISO_8859_8;
        {"whatwg" "hebr"} cp 38598 = all::whatwg::ISO_8859_8_I;
        {"whatwg" "latn"} cp 28603 = all::ISO_8859_13;
        {"whatwg" "latn"} cp 28605 = all::ISO_8859_15;
        {"whatwg" "cyrl"} cp 20866 = all::KOI8_R;
        {"whatwg" "cyrl"} cp 21866 = all::KOI8_U;
        {"whatwg" "latn"} cp 10000 = all::MAC_ROMAN;
        {"whatwg" "thai"} cp 874 = all::WINDOWS_874;
        {"whatwg" "latn"} cp 1250 = all::WINDOWS_1250;
        {"whatwg" "cyrl"} cp 1251 = all::WINDOWS_1251;
        {"whatwg" "latn"} cp 1252 = all::WINDOWS_1252;
        {"whatwg" "grek"} cp 1253 = all::WINDOWS_1253;
        {"whatwg" "latn"} cp 1254 = all::WINDOWS_1254;
        {"whatwg" "hebr"} cp 1255 = all::WINDOWS_1255;
        {"whatwg" "arab"} cp 1256 = all::WINDOWS_1256;
        {"whatwg" "latn"} cp 1257 = all::WINDOWS_1257;
        {"whatwg" "latn"} cp 1258 = all::WINDOWS_1258;
        {"whatwg" "cyrl"} cp 1259 = all::MAC_CYRILLIC;
        {"whatwg" "hans"} cp 936 | cp 54936 = all::GB18030; // XXX technically wrong
        {"whatwg" "hans"} cp 52936 = all::HZ;
        {"whatwg" "hant"} cp 950 = all::BIG5_2003;
        {"whatwg" "jpan"} cp 20932 = all::EUC_JP;
        {"whatwg" "jpan"} cp 50220 = all::ISO_2022_JP;
        {"whatwg" "jpan"} cp 932 = all::WINDOWS_31J;
        {"whatwg" "kore"} cp 949 = all::WINDOWS_949;
        {"whatwg" "unicode"} cp 1201 = all::UTF_16BE;
        {"whatwg" "unicode"} cp 1200 = all::UTF_16LE;
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    #[cfg(any(feature="default", feature="whatwg"))]
    use super::encoding_from_whatwg_label;

    #[test]
    #[cfg(any(feature="default", feature="whatwg"))]
    fn test_encoding_from_whatwg_label() {
        assert!(encoding_from_whatwg_label("utf-8").is_some())
        assert!(encoding_from_whatwg_label("UTF-8").is_some())
        assert!(encoding_from_whatwg_label("\t\n\x0C\r utf-8\t\n\x0C\r ").is_some())
        assert!(encoding_from_whatwg_label("\u00A0utf-8").is_none(),
                "Non-ASCII whitespace should not be trimmed")
        assert!(encoding_from_whatwg_label("greek").is_some())
        assert!(encoding_from_whatwg_label("gree\u212A").is_none(),
                "Case-insensitive matching should be ASCII only. Kelvin sign does not match k.")
    }

    #[bench]
    #[cfg(any(feature="default", feature="whatwg"))]
    fn bench_encoding_from_whatwg_label(bencher: &mut test::Bencher) {
        bencher.iter(|| test::black_box({
            encoding_from_whatwg_label("iso-8859-bazinga")
        }))
    }
}


// This is a part of rust-encoding.
// Copyright (c) 2013, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! An interface for retrieving an encoding (or a set of encodings) from a string/numeric label.

use std::ascii::StrAsciiExt;
use all;
use types::EncodingObj;

/// Returns an encoding from given label, defined in the WHATWG Encoding standard, if any.
/// Implements "get an encoding" algorithm: http://encoding.spec.whatwg.org/#decode
pub fn encoding_from_whatwg_label(label: &str) -> Option<EncodingObj> {
    match label.trim_chars(& &[' ', '\n', '\r', '\t', '\x0C']).to_ascii_lower().as_slice() {
        "unicode-1-1-utf-8" |
        "utf-8" |
        "utf8" =>
            Some(all::UTF_8 as EncodingObj),
        "866" |
        "cp866" |
        "csibm866" |
        "ibm866" =>
            Some(all::IBM866 as EncodingObj),
        "csisolatin2" |
        "iso-8859-2" |
        "iso-ir-101" |
        "iso8859-2" |
        "iso88592" |
        "iso_8859-2" |
        "iso_8859-2:1987" |
        "l2" |
        "latin2" =>
            Some(all::ISO_8859_2 as EncodingObj),
        "csisolatin3" |
        "iso-8859-3" |
        "iso-ir-109" |
        "iso8859-3" |
        "iso88593" |
        "iso_8859-3" |
        "iso_8859-3:1988" |
        "l3" |
        "latin3" =>
            Some(all::ISO_8859_3 as EncodingObj),
        "csisolatin4" |
        "iso-8859-4" |
        "iso-ir-110" |
        "iso8859-4" |
        "iso88594" |
        "iso_8859-4" |
        "iso_8859-4:1988" |
        "l4" |
        "latin4" =>
            Some(all::ISO_8859_4 as EncodingObj),
        "csisolatincyrillic" |
        "cyrillic" |
        "iso-8859-5" |
        "iso-ir-144" |
        "iso8859-5" |
        "iso88595" |
        "iso_8859-5" |
        "iso_8859-5:1988" =>
            Some(all::ISO_8859_5 as EncodingObj),
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
            Some(all::ISO_8859_6 as EncodingObj),
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
            Some(all::ISO_8859_7 as EncodingObj),
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
            Some(all::ISO_8859_8 as EncodingObj),
        "csiso88598i" |
        "iso-8859-8-i" |
        "logical" =>
            Some(all::whatwg::ISO_8859_8_I as EncodingObj),
        "csisolatin6" |
        "iso-8859-10" |
        "iso-ir-157" |
        "iso8859-10" |
        "iso885910" |
        "l6" |
        "latin6" =>
            Some(all::ISO_8859_10 as EncodingObj),
        "iso-8859-13" |
        "iso8859-13" |
        "iso885913" =>
            Some(all::ISO_8859_13 as EncodingObj),
        "iso-8859-14" |
        "iso8859-14" |
        "iso885914" =>
            Some(all::ISO_8859_14 as EncodingObj),
        "csisolatin9" |
        "iso-8859-15" |
        "iso8859-15" |
        "iso885915" |
        "iso_8859-15" |
        "l9" =>
            Some(all::ISO_8859_15 as EncodingObj),
        "iso-8859-16" =>
            Some(all::ISO_8859_16 as EncodingObj),
        "cskoi8r" |
        "koi" |
        "koi8" |
        "koi8-r" |
        "koi8_r" =>
            Some(all::KOI8_R as EncodingObj),
        "koi8-u" =>
            Some(all::KOI8_U as EncodingObj),
        "csmacintosh" |
        "mac" |
        "macintosh" |
        "x-mac-roman" =>
            Some(all::MACINTOSH as EncodingObj),
        "dos-874" |
        "iso-8859-11" |
        "iso8859-11" |
        "iso885911" |
        "tis-620" |
        "windows-874" =>
            Some(all::WINDOWS_874 as EncodingObj),
        "cp1250" |
        "windows-1250" |
        "x-cp1250" =>
            Some(all::WINDOWS_1250 as EncodingObj),
        "cp1251" |
        "windows-1251" |
        "x-cp1251" =>
            Some(all::WINDOWS_1251 as EncodingObj),
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
            Some(all::WINDOWS_1252 as EncodingObj),
        "cp1253" |
        "windows-1253" |
        "x-cp1253" =>
            Some(all::WINDOWS_1253 as EncodingObj),
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
            Some(all::WINDOWS_1254 as EncodingObj),
        "cp1255" |
        "windows-1255" |
        "x-cp1255" =>
            Some(all::WINDOWS_1255 as EncodingObj),
        "cp1256" |
        "windows-1256" |
        "x-cp1256" =>
            Some(all::WINDOWS_1256 as EncodingObj),
        "cp1257" |
        "windows-1257" |
        "x-cp1257" =>
            Some(all::WINDOWS_1257 as EncodingObj),
        "cp1258" |
        "windows-1258" |
        "x-cp1258" =>
            Some(all::WINDOWS_1258 as EncodingObj),
        "x-mac-cyrillic" |
        "x-mac-ukrainian" =>
            Some(all::X_MAC_CYRILLIC as EncodingObj),
        "chinese" |
        "csgb2312" |
        "csiso58gb231280" |
        "gb2312" |
        "gb_2312" |
        "gb_2312-80" |
        "gbk" |
        "iso-ir-58" |
        "x-gbk" =>
            Some(all::GBK18030 as EncodingObj),
        "gb18030" =>
            Some(all::GB18030 as EncodingObj),
        /*
        "hz-gb-2312" =>
            Some(all::HZ_GB_2312 as EncodingObj),
        */
        "big5" |
        "big5-hkscs" |
        "cn-big5" |
        "csbig5" |
        "x-x-big5" =>
            Some(all::BIG5_2003 as EncodingObj),
        "cseucpkdfmtjapanese" |
        "euc-jp" |
        "x-euc-jp" =>
            Some(all::EUC_JP as EncodingObj),
        /*
        "csiso2022jp" |
        "iso-2022-jp" =>
            Some(all::ISO_2022_JP as EncodingObj),
        */
        "csshiftjis" |
        "ms_kanji" |
        "shift-jis" |
        "shift_jis" |
        "sjis" |
        "windows-31j" |
        "x-sjis" =>
            Some(all::WINDOWS_31J as EncodingObj),
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
            Some(all::WINDOWS_949 as EncodingObj),
        "csiso2022kr" |
        "iso-2022-kr" |
        "iso-2022-cn" |
        "iso-2022-cn-ext" =>
            Some(all::whatwg::REPLACEMENT as EncodingObj),
        "utf-16be" =>
            Some(all::UTF_16BE as EncodingObj),
        "utf-16" |
        "utf-16le" =>
            Some(all::UTF_16LE as EncodingObj),
        "x-user-defined" =>
            Some(all::whatwg::X_USER_DEFINED as EncodingObj),
        _ => None
    }
}

/// Returns an encoding from Windows code page number.
/// http://msdn.microsoft.com/en-us/library/windows/desktop/dd317756%28v=vs.85%29.aspx
/// Sometimes it can return a *superset* of the requested encoding, e.g. for several CJK encodings.
pub fn encoding_from_windows_code_page(cp: uint) -> Option<EncodingObj> {
    match cp {
        65001 => Some(all::UTF_8 as EncodingObj),
        866 => Some(all::IBM866 as EncodingObj),
        28591 => Some(all::ISO_8859_1 as EncodingObj),
        28592 => Some(all::ISO_8859_2 as EncodingObj),
        28593 => Some(all::ISO_8859_3 as EncodingObj),
        28594 => Some(all::ISO_8859_4 as EncodingObj),
        28595 => Some(all::ISO_8859_5 as EncodingObj),
        28596 => Some(all::ISO_8859_6 as EncodingObj),
        28597 => Some(all::ISO_8859_7 as EncodingObj),
        28598 => Some(all::ISO_8859_8 as EncodingObj),
        38598 => Some(all::whatwg::ISO_8859_8_I as EncodingObj),
        28603 => Some(all::ISO_8859_13 as EncodingObj),
        28605 => Some(all::ISO_8859_15 as EncodingObj),
        20866 => Some(all::KOI8_R as EncodingObj),
        21866 => Some(all::KOI8_U as EncodingObj),
        10000 => Some(all::MACINTOSH as EncodingObj),
        874 => Some(all::WINDOWS_874 as EncodingObj),
        1250 => Some(all::WINDOWS_1250 as EncodingObj),
        1251 => Some(all::WINDOWS_1251 as EncodingObj),
        1252 => Some(all::WINDOWS_1252 as EncodingObj),
        1253 => Some(all::WINDOWS_1253 as EncodingObj),
        1254 => Some(all::WINDOWS_1254 as EncodingObj),
        1255 => Some(all::WINDOWS_1255 as EncodingObj),
        1256 => Some(all::WINDOWS_1256 as EncodingObj),
        1257 => Some(all::WINDOWS_1257 as EncodingObj),
        1258 => Some(all::WINDOWS_1258 as EncodingObj),
        1259 => Some(all::X_MAC_CYRILLIC as EncodingObj),
        936 => Some(all::GBK18030 as EncodingObj),
        54936 => Some(all::GB18030 as EncodingObj),
        /*
        52936 => Some(all::HZ_GB_2312 as EncodingObj),
        */
        950 => Some(all::BIG5_2003 as EncodingObj),
        20932 => Some(all::EUC_JP as EncodingObj),
        /*
        50220 => Some(all::ISO_2022_JP as EncodingObj),
        */
        932 => Some(all::WINDOWS_31J as EncodingObj),
        949 => Some(all::WINDOWS_949 as EncodingObj),
        1201 => Some(all::UTF_16BE as EncodingObj),
        1200 => Some(all::UTF_16LE as EncodingObj),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    extern mod extra;
    use super::encoding_from_whatwg_label;

    #[test]
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
    fn bench_encoding_from_whatwg_label(harness: &mut extra::test::BenchHarness) {
        do harness.iter() {
            encoding_from_whatwg_label("iso-8859-bazinga");
        }
    }
}


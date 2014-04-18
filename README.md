Rust-encoding
=============

[![Rust-encoding on Travis CI][travis-image]][travis]

[travis-image]: https://travis-ci.org/lifthrasiir/rust-encoding.png
[travis]: https://travis-ci.org/lifthrasiir/rust-encoding

Character encoding support for Rust.
It is based on [WHATWG Encoding Standard](http://encoding.spec.whatwg.org/),
and also provides an advanced interface for error detection and recovery.

Usage
-----

To encode a string:

~~~~ {.rust}
use encoding::*;
all::ISO_8859_1.encode("caf\xe9", EncodeStrict); // => Ok(vec!(99,97,102,233))
~~~~

To encode a string with unrepresentable characters:

~~~~ {.rust}
all::ISO_8859_2.encode("Acme\xa9", EncodeStrict); // => Err(...)
all::ISO_8859_2.encode("Acme\xa9", EncodeReplace); // => Ok(vec!(65,99,109,101,63))
all::ISO_8859_2.encode("Acme\xa9", EncodeIgnore); // => Ok(vec!(65,99,109,101))
all::ISO_8859_2.encode("Acme\xa9", EncodeNcrEscape); // => Ok(vec!(65,99,109,101,38,23,50,51,51,59))
~~~~

To decode a byte sequence:

~~~~ {.rust}
all::ISO_8859_1.decode([99,97,102,233], DecodeStrict); // => Ok(~"caf\xe9")
~~~~

To decode a byte sequence with invalid sequences:

~~~~ {.rust}
all::ISO_8859_6.decode([65,99,109,101,169], DecodeStrict); // => Err(...)
all::ISO_8859_6.decode([65,99,109,101,169], DecodeReplace); // => Ok(StrBuf::from_str("Acme\ufffd"))
all::ISO_8859_6.decode([65,99,109,101,169], DecodeIgnore); // => Ok(StrBuf::from_str("Acme"))
~~~~

A practical example of custom encoder traps:

~~~~ {.rust}
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
all::ASCII.decode(encoded.as_slice(), DecodeStrict); // => Ok(StrBuf::from_str("Hello, &#x4e16;&#x754c;!"))
~~~~

Getting the encoding from the string label,
as specified in the WHATWG Encoding standard:

~~~~ {.rust}
let euckr = label::encoding_from_whatwg_label("euc-kr").unwrap();
euckr.name(); // => "windows-949"
euckr.whatwg_name(); // => Some("euc-kr"), for the sake of compatibility
let broken = &[0xbf, 0xec, 0xbf, 0xcd, 0xff, 0xbe, 0xd3];
euckr.decode(broken, DecodeReplace); // => Ok(Strbuf::from_str("\uc6b0\uc640\ufffd\uc559"))

// corresponding rust-encoding native API:
all::WINDOWS_949.decode(broken, DecodeReplace); // => Ok(StrBuf::from_str("\uc6b0\uc640\ufffd\uc559"))
~~~~

Supported Encodings
-------------------

Rust-encoding is a work in progress and this list will certainly be updated.

* 7-bit strict ASCII (`ascii`)
* UTF-8 (`utf-8`)
* UTF-16 in little endian (`utf-16` or `utf-16le`) and big endian (`utf-16be`)
* All single byte encoding in WHATWG Encoding Standard:
    * IBM code page 866
    * ISO 8859-{2,3,4,5,6,7,8,10,13,14,15,16}
    * KOI8-R, KOI8-U
    * MacRoman (`macintosh`), Macintosh Cyrillic encoding (`x-mac-cyrillic`)
    * Windows code page 874, 1250, 1251, 1252 (instead of ISO-8859-1), 1253,
      1254 (instead of ISO-8859-9), 1255, 1256, 1257, 1258
* Multi byte encodings in WHATWG Encoding Standard:
    * Windows code page 949 (`euc-kr`, since the strict EUC-KR is hardly used)
    * EUC-JP and Windows code page 932 (`shift_jis`,
      since it's the most widespread extension to Shift_JIS)
    * GB 18030 and its GBK subset
    * Big5-2003 with HKSCS-2008 extensions
* ISO 8859-1 (distinct from Windows code page 1252)


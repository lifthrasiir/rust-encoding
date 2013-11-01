Rust-encoding
=============

Character encoding support for Rust.
It is based on [WHATWG Encoding Standard](http://encoding.spec.whatwg.org/),
and also provides an advanced interface for error detection and recovery.

Usage
-----

To encode a string:

~~~~ {.rust}
use encoding::*;
all::ISO_8859_2.encode("caf\xe9", Strict); // => Ok(~[99,97,102,233])
~~~~

To encode a string with unrepresentable characters:

~~~~ {.rust}
all::ISO_8859_2.encode("Acme\xa9", Strict); // => Err(...)
all::ISO_8859_2.encode("Acme\xa9", Replace); // => Ok(~[65,99,109,101,63])
all::ISO_8859_2.encode("Acme\xa9", Ignore); // => Ok(~[65,99,109,101])
all::ISO_8859_2.encode("Acme\xa9", NcrEscape); // => Ok(~[65,99,109,101,38,23,50,51,51,59])
~~~~

To decode a byte sequence:

~~~~ {.rust}
all::ISO_8859_2.decode([99,97,102,233], Strict); // => Ok(~"caf\xe9")
~~~~

To decode a byte sequence with invalid sequences:

~~~~ {.rust}
all::ISO_8859_6.decode([65,99,109,101,169], Strict); // => Err(...)
all::ISO_8859_6.decode([65,99,109,101,169], Replace); // => Ok(~"Acme\ufffd")
all::ISO_8859_6.decode([65,99,109,101,169], Ignore); // => Ok(~"Acme")
~~~~

A practical example of custom encoder traps:

~~~~ {.rust}
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
~~~~

Supported Encodings
-------------------

Rust-encoding is a work in progress and this list will certainly be updated.

* 7-bit strict ASCII (`ascii`)
* UTF-8 (`utf-8`)
* All single byte encoding in WHATWG Encoding Standard:
    * IBM code page 866
    * ISO-8859-{2,3,4,5,6,7,8,10,13,14,15,16}
    * KOI8-R, KOI8-U
    * MacRoman (`macintosh`), Macintosh Cyrillic encoding (`x-mac-cyrillic`)
    * Windows code page 874, 1250, 1251, 1252 (instead of ISO-8859-1), 1253,
      1254 (instead of ISO-8859-9), 1255, 1256, 1257, 1258
* Multi byte encodings in WHATWG Encoding Standard:
    * Windows code page 949 (`euc-kr`, since the strict EUC-KR is hardly used)
    * EUC-JP and Shift_JIS


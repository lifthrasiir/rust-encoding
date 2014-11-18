// This is a part of rust-encoding.
// Copyright (c) 2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

extern crate encoding;
extern crate getopts;

use std::{io, os};
use encoding::{EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use getopts::{optopt, optflag};

fn main() {
    let args = os::args();

    let opts = [
        optopt("f", "from-code", "set input encoding", "NAME"),
        optopt("t", "to-code", "set output encoding", "NAME"),
        optopt("e", "error-policy",
               "set error policy (one of strict, ignore, replace, ncr-escape)", "POLICY"),
        optflag("c", "", "same as `--error-policy=ignore`"),
        optopt("o", "output", "output file", "FILE"),
        optflag("h", "help", "print this help menu"),
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };
    if matches.opt_present("h") {
        println!("{}", getopts::usage("Converts the character encoding using rust-encoding.",
                                      opts));
        return;
    }

    let inencname = matches.opt_str("f");
    let outencname = matches.opt_str("t");
    let inenc = match inencname.as_ref().map(|s| s.as_slice()) {
        Some(name) => match encoding_from_whatwg_label(name) {
            Some(enc) => enc,
            None => panic!("invalid input encoding name {}", name),
        },
        None => encoding::all::UTF_8 as encoding::EncodingRef,
    };
    let outenc = match outencname.as_ref().map(|s| s.as_slice()) {
        Some(name) => match encoding_from_whatwg_label(name) {
            Some(enc) => enc,
            None => panic!("invalid output encoding name {}", name),
        },
        None => encoding::all::UTF_8 as encoding::EncodingRef,
    };

    let mut policy = matches.opt_str("e");
    if matches.opt_present("c") {
        policy = Some("ignore".to_string());
    }
    let (intrap, outtrap) = match policy.as_ref().map(|s| s.as_slice()) {
        Some("strict") | None => (DecoderTrap::Strict, EncoderTrap::Strict),
        Some("ignore") => (DecoderTrap::Ignore, EncoderTrap::Ignore),
        Some("replace") => (DecoderTrap::Replace, EncoderTrap::Replace),
        Some("ncr-escape") => (DecoderTrap::Replace, EncoderTrap::NcrEscape),
        Some(s) => panic!("invalid error policy {}", s),
    };

    let mut input = match matches.free.head().map(|s| s.as_slice()) {
        Some("-") | None => box io::stdin() as Box<Reader>,
        Some(f) => box io::File::open(&Path::new(f)) as Box<Reader>,
    };
    let mut output = match matches.opt_str("o").as_ref().map(|s| s.as_slice()) {
        Some("-") | None => box io::stdout() as Box<Writer>,
        Some(f) => box io::File::create(&Path::new(f)) as Box<Writer>,
    };

    // XXX should really use the incremental interface
    let decoded = match inenc.decode(input.read_to_end().unwrap().as_slice(), intrap) {
        Ok(s) => s,
        Err(e) => panic!("decoder error: {}", e),
    };
    let encoded = match outenc.encode(decoded.as_slice(), outtrap) {
        Ok(s) => s,
        Err(e) => panic!("encoder error: {}", e),
    };
    output.write(encoded.as_slice()).unwrap();
}


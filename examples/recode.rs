// This is a part of rust-encoding.
// Copyright (c) 2014-2015, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

#![feature(core, os, io, path, collections)] // lib stability features as per RFC #507

extern crate core;
extern crate encoding;
extern crate getopts;

use std::old_io as io;
use std::os;
use encoding::{EncoderTrap, DecoderTrap};
use encoding::label::encoding_from_whatwg_label;
use getopts::Options;

fn main() {
    let args = os::args();

    let mut opts = Options::new();
    opts.optopt("f", "from-code", "set input encoding", "NAME");
    opts.optopt("t", "to-code", "set output encoding", "NAME");
    opts.optopt("e", "error-policy",
                "set error policy (one of strict, ignore, replace, ncr-escape)", "POLICY");
    opts.optflag("c", "", "same as `--error-policy=ignore`");
    opts.optopt("o", "output", "output file", "FILE");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(args.tail()) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };
    if matches.opt_present("h") {
        println!("{}", opts.usage("Converts the character encoding using rust-encoding."));
        return;
    }

    let inencname = matches.opt_str("f");
    let outencname = matches.opt_str("t");
    let inenc = match inencname.as_ref().map(|s| &s[]) {
        Some(name) => match encoding_from_whatwg_label(name) {
            Some(enc) => enc,
            None => panic!("invalid input encoding name {}", name),
        },
        None => encoding::all::UTF_8 as encoding::EncodingRef,
    };
    let outenc = match outencname.as_ref().map(|s| &s[]) {
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
    let (intrap, outtrap) = match policy.as_ref().map(|s| &s[]) {
        Some("strict") | None => (DecoderTrap::Strict, EncoderTrap::Strict),
        Some("ignore") => (DecoderTrap::Ignore, EncoderTrap::Ignore),
        Some("replace") => (DecoderTrap::Replace, EncoderTrap::Replace),
        Some("ncr-escape") => (DecoderTrap::Replace, EncoderTrap::NcrEscape),
        Some(s) => panic!("invalid error policy {}", s),
    };

    let mut input = match matches.free.first().map(|s| &s[]) {
        Some("-") | None => Box::new(io::stdin()) as Box<Reader>,
        Some(f) => Box::new(io::File::open(&Path::new(f))) as Box<Reader>,
    };
    let mut output = match matches.opt_str("o").as_ref().map(|s| &s[]) {
        Some("-") | None => Box::new(io::stdout()) as Box<Writer>,
        Some(f) => Box::new(io::File::create(&Path::new(f))) as Box<Writer>,
    };

    // XXX should really use the incremental interface
    let decoded = match inenc.decode(&input.read_to_end().unwrap()[], intrap) {
        Ok(s) => s,
        Err(e) => panic!("decoder error: {}", e),
    };
    let encoded = match outenc.encode(&decoded[], outtrap) {
        Ok(s) => s,
        Err(e) => panic!("encoder error: {}", e),
    };
    output.write_all(&encoded[]).unwrap();
}


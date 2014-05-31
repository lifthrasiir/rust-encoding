// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Internal utilities.

#![macro_escape]

use std::{str, mem};
use std::default::Default;
use types;

/// Unchecked conversion to `char`.
pub fn as_char<T:Int+NumCast>(ch: T) -> char {
    unsafe { mem::transmute(ch.to_u32().unwrap()) }
}

/// External iterator for a string's characters with its corresponding byte offset range.
pub struct StrCharIndexIterator<'r> {
    index: uint,
    string: &'r str,
}

impl<'r> Iterator<((uint,uint), char)> for StrCharIndexIterator<'r> {
    #[inline]
    fn next(&mut self) -> Option<((uint,uint), char)> {
        if self.index < self.string.len() {
            let str::CharRange {ch, next} = self.string.char_range_at(self.index);
            let prev = self.index;
            self.index = next;
            Some(((prev, next), ch))
        } else {
            None
        }
    }
}

/// A trait providing an `index_iter` method.
pub trait StrCharIndex<'r> {
    fn index_iter(&self) -> StrCharIndexIterator<'r>;
}

impl<'r> StrCharIndex<'r> for &'r str {
    /// Iterates over each character with corresponding byte offset range.
    fn index_iter(&self) -> StrCharIndexIterator<'r> {
        StrCharIndexIterator { index: 0, string: *self }
    }
}

/// A helper struct for the stateful decoder DSL.
pub struct StatefulDecoderHelper<'a, St> {
    /// The current buffer.
    pub buf: &'a [u8],
    /// The current index to the buffer.
    pub pos: uint,
    /// The output buffer.
    pub output: &'a mut types::StringWriter,
    /// The last codec error. The caller will later collect this.
    pub err: Option<types::CodecError>,
}

impl<'a, St:Default> StatefulDecoderHelper<'a, St> {
    /// Reads one byte from the buffer if any.
    #[inline(always)]
    pub fn read(&mut self) -> Option<u8> {
        match self.buf.get(self.pos) {
            Some(&c) => { self.pos += 1; Some(c) }
            None => None
        }
    }

    /// Resets back to the initial state.
    /// This should be the last expr in the rules.
    #[inline(always)]
    pub fn reset(&self) -> St {
        Default::default()
    }

    /// Writes one Unicode scalar value to the output.
    /// There is intentionally no check for `c`, so the caller should ensure that it's valid.
    /// If this is the last expr in the rules, also resets back to the initial state.
    #[inline(always)]
    pub fn emit(&mut self, c: u32) -> St {
        self.output.write_char(unsafe {mem::transmute(c)});
        Default::default()
    }

    /// Writes a Unicode string to the output.
    /// If this is the last expr in the rules, also resets back to the initial state.
    #[inline(always)]
    pub fn emit_str(&mut self, s: &str) -> St {
        self.output.write_str(s);
        Default::default()
    }

    /// Issues a codec error with given message at the current position.
    /// If this is the last expr in the rules, also resets back to the initial state.
    #[inline(always)]
    pub fn err(&mut self, msg: &'static str) -> St {
        self.err = Some(types::CodecError { upto: self.pos, cause: msg.into_maybe_owned() });
        Default::default()
    }

    /// Issues a codec error with given message at the current position minus `backup` bytes.
    /// If this is the last expr in the rules, also resets back to the initial state.
    ///
    /// This should be used to implement "prepending byte to the stream" in the Encoding spec,
    /// which corresponds to `ctx.backup_and_err(1, ...)`.
    #[inline(always)]
    pub fn backup_and_err(&mut self, backup: uint, msg: &'static str) -> St {
        // XXX we should eventually handle a negative `upto`
        let upto = if self.pos < backup {0} else {self.pos - backup};
        self.err = Some(types::CodecError { upto: upto, cause: msg.into_maybe_owned() });
        Default::default()
    }
}

/// Defines a stateful decoder from given state machine.
macro_rules! stateful_decoder(
    (
        $(#[$decmeta:meta])*
        struct $dec:ident;
        module $stmod:ident; // should be unique from other existing identifiers
        ascii_compatible $asciicompat:expr;
        $(internal $item:item)* // will only be visible from state functions
        initial state $inist:ident($inictx:ident) {
            $(case $($inilhs:pat)|+ => $($inirhs:expr),+;)+
            final => $($inifin:expr),+;
        }
        $(checkpoint state $ckst:ident($ckctx:ident $(, $ckarg:ident: $ckty:ty)*) {
            $(case $($cklhs:pat)|+ => $($ckrhs:expr),+;)+
            final => $($ckfin:expr),+;
        })*
        $(state $st:ident($ctx:ident $(, $arg:ident: $ty:ty)*) {
            $(case $($lhs:pat)|+ => $($rhs:expr),+;)+
            final => $($fin:expr),+;
        })*
    ) => (
        $(#[$decmeta])*
        pub struct $dec {
            st: $stmod::State
        }

        #[allow(non_snake_case_functions)]
        mod $stmod {
            #[deriving(PartialEq,Clone)]
            pub enum State {
                $inist,
                $(
                    $ckst(() $(, $ckty)*),
                )*
                $(
                    $st(() $(, $ty)*),
                )*
            }

            impl ::std::default::Default for State {
                #[inline(always)] fn default() -> State { $inist }
            }

            pub mod internal {
                pub type Context<'a> = ::util::StatefulDecoderHelper<'a, super::State>;

                $($item)*
            }

            pub mod start {
                use super::internal::*;

                #[inline(always)]
                pub fn $inist($inictx: &mut Context) -> super::State {
                    // prohibits all kind of recursions, including self-recursions
                    #[allow(unused_imports)] use super::transient::*;
                    match $inictx.read() {
                        None => super::$inist,
                        Some(c) => match c { $($($inilhs)|+ => { $($inirhs);+ })+ },
                    }
                }

                $(
                    #[inline(always)]
                    pub fn $ckst($ckctx: &mut Context $(, $ckarg: $ckty)*) -> super::State {
                        // prohibits all kind of recursions, including self-recursions
                        #[allow(unused_imports)] use super::transient::*;
                        match $ckctx.read() {
                            None => super::$ckst(() $(, $ckarg)*),
                            Some(c) => match c { $($($cklhs)|+ => { $($ckrhs);+ })+ },
                        }
                    }
                )*
            }

            pub mod transient {
                use super::internal::*;

                #[inline(always)]
                #[allow(dead_code)]
                pub fn $inist(_: &mut Context) -> super::State {
                    super::$inist // do not recurse further
                }

                $(
                    #[inline(always)]
                    #[allow(dead_code)]
                    pub fn $ckst(_: &mut Context $(, $ckarg: $ckty)*) -> super::State {
                        super::$ckst(() $(, $ckarg)*) // do not recurse further
                    }
                )*

                $(
                    #[inline(always)]
                    pub fn $st($ctx: &mut Context $(, $arg: $ty)*) -> super::State {
                        match $inictx.read() {
                            None => super::$st(() $(, $arg)*),
                            Some(c) => match c { $($($lhs)|+ => { $($rhs);+ })+ },
                        }
                    }
                )*
            }
        }

        impl $dec {
            pub fn new() -> Box<Decoder> { box $dec { st: $stmod::$inist } as Box<Decoder> }
        }

        impl Decoder for $dec {
            fn from_self(&self) -> Box<Decoder> { $dec::new() }
            fn is_ascii_compatible(&self) -> bool { $asciicompat }

            fn raw_feed(&mut self, input: &[u8],
                        output: &mut StringWriter) -> (uint, Option<CodecError>) {
                use self::$stmod::{start, transient};

                output.writer_hint(input.len());

                let mut ctx = ::util::StatefulDecoderHelper {
                    buf: input, pos: 0, output: output, err: None
                };
                let mut processed = 0;
                let mut st = self.st;

                let st_ = match st {
                    $stmod::$inist => $stmod::$inist,
                    $(
                        $stmod::$ckst(() $(, $ckarg)*) => start::$ckst(&mut ctx $(, $ckarg)*),
                    )*
                    $(
                        $stmod::$st(() $(, $arg)*) => transient::$st(&mut ctx $(, $arg)*),
                    )*
                };
                match (ctx.err.take(), st_) {
                    (None, $stmod::$inist) $(| (None, $stmod::$ckst(..)))* =>
                        { st = st_; processed = ctx.pos; }
                    // XXX splitting the match case improves the performance somehow, but why?
                    (None, _) => { self.st = st_; return (processed, None); }
                    (Some(err), _) => { self.st = st_; return (processed, Some(err)); }
                }

                while ctx.pos < ctx.buf.len() {
                    let st_ = match st {
                        $stmod::$inist => start::$inist(&mut ctx),
                        $(
                            $stmod::$ckst(() $(, $ckarg)*) => start::$ckst(&mut ctx $(, $ckarg)*),
                        )*
                        _ => unreachable!(),
                    };
                    match (ctx.err.take(), st_) {
                        (None, $stmod::$inist) $(| (None, $stmod::$ckst(..)))* =>
                            { st = st_; processed = ctx.pos; }
                        // XXX splitting the match case improves the performance somehow, but why?
                        (None, _) => { self.st = st_; return (processed, None); }
                        (Some(err), _) => { self.st = st_; return (processed, Some(err)); }
                    }
                }

                self.st = st;
                (processed, None)
            }

            fn raw_finish(&mut self, output: &mut StringWriter) -> Option<CodecError> {
                #![allow(unused_mut, unused_variable)]
                let mut ctx = ::util::StatefulDecoderHelper {
                    buf: &[], pos: 0, output: output, err: None
                };
                self.st = match ::std::mem::replace(&mut self.st, $stmod::$inist) {
                    $stmod::$inist => { let $inictx = &mut ctx; $($inifin);+ },
                    $(
                        $stmod::$ckst(() $(, $ckarg)*) => { let $ckctx = &mut ctx; $($ckfin);+ },
                    )*
                    $(
                        $stmod::$st(() $(, $arg)*) => { let $ctx = &mut ctx; $($fin);+ },
                    )*
                };
                ctx.err.take()
            }
        }
    )
)

/// Defines an ASCII-compatible stateful decoder from given state machine.
macro_rules! ascii_compatible_stateful_decoder(
    (
        $(#[$decmeta:meta])*
        struct $dec:ident;
        module $stmod:ident; // should be unique from other existing identifiers
        $(internal $item:item)* // will only be visible from state functions
        initial state $inist:ident($inictx:ident) {
            $(case $($inilhs:pat)|+ => $($inirhs:expr),+;)+
        }
        $(state $st:ident($ctx:ident $(, $arg:ident: $ty:ty)*) {
            $(case $($lhs:pat)|+ => $($rhs:expr),+;)+
        })*
    ) => (
        stateful_decoder!(
            $(#[$decmeta])*
            struct $dec;
            module $stmod;
            ascii_compatible true;
            $(internal $item)*
            initial state $inist($inictx) {
                $(case $($inilhs)|+ => $($inirhs),+;)+
                final => $inictx.reset();
            }
            $(state $st($ctx $(, $arg: $ty)*) {
                $(case $($lhs)|+ => $($rhs),+;)+
                final => $ctx.err("incomplete sequence");
            })*
        )
    )
)


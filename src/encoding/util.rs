// This is a part of rust-encoding.
// Copyright (c) 2013-2014, Kang Seonghoon.
// See README.md and LICENSE.txt for details.

//! Internal utilities.

#![macro_escape]

use std::{str, cast};
use std::default::Default;
use types;

/// Unchecked conversion to `char`.
pub fn as_char<T:Int+NumCast>(ch: T) -> char {
    unsafe { cast::transmute(ch.to_u32().unwrap()) }
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
pub struct StatefulDecoderHelper<'a, State> {
    /// The current buffer.
    pub buf: &'a [u8],
    /// The current index to the buffer.
    pub pos: uint,
    /// The output buffer.
    pub output: &'a mut types::StringWriter,
}

impl<'a, State: Default> StatefulDecoderHelper<'a, State> {
    /// Reads one byte from the buffer if any.
    #[inline(always)]
    pub fn read(&mut self) -> Option<u8> {
        match self.buf.get(self.pos) {
            Some(&c) => { self.pos += 1; Some(c) }
            None => None
        }
    }

    /// Writes one Unicode scalar value to the output and resets back to the initial state.
    /// There is intentionally no check for `c`, so the caller should ensure that it's valid.
    #[inline(always)]
    pub fn emit(&mut self, c: u32) -> Result<State,types::CodecError> {
        self.output.write_char(unsafe {cast::transmute(c)});
        Ok(Default::default())
    }

    /// Writes a Unicode string to the output and resets back to the initial state.
    #[inline(always)]
    pub fn emit_str(&mut self, s: &str) -> Result<State,types::CodecError> {
        self.output.write_str(s);
        Ok(Default::default())
    }

    /// Issues a codec error with given message at the current position.
    #[inline(always)]
    pub fn err(&self, msg: &'static str) -> Result<State,types::CodecError> {
        Err(types::CodecError { upto: self.pos, cause: msg.into_maybe_owned() })
    }

    /// Issues a codec error with given message at the current position minus `backup` bytes.
    /// This should be used to implement "prepending byte to the stream" in the Encoding spec,
    /// which corresponds to `ctx.backup_and_err(1, ...)`.
    #[inline(always)]
    pub fn backup_and_err(&self, backup: uint,
                          msg: &'static str) -> Result<State,types::CodecError> {
        // XXX we should eventually handle a negative `upto`
        let upto = if self.pos < backup {0} else {self.pos - backup};
        Err(types::CodecError { upto: upto, cause: msg.into_maybe_owned() })
    }
}

/// Defines an ASCII-compatible stateful decoder from given state machine.
macro_rules! ascii_compatible_stateful_decoder(
    (
        $(#[$decmeta:meta])*
        struct $dec:ident;
        module $stmod:ident; // should be unique from other existing identifiers
        $(internal $item:item)* // will only be visible from state functions
        initial state $inist:ident($inictx:ident) {
            $(case $($inilhs:pat)|+ => $inirhs:expr;)+
        }
        $(state $st:ident($ctx:ident $(, $arg:ident: $ty:ty)*) {
            $(case $($lhs:pat)|+ => $rhs:expr;)+
        })*
    ) => (
        $(#[$decmeta])*
        pub struct $dec {
            st: $stmod::State
        }

        mod $stmod {
            #[deriving(Eq,Clone)]
            pub enum State {
                $inist,
                $(
                    $st(() $(, $ty)*),
                )*
            }

            impl ::std::default::Default for State {
                fn default() -> State { $inist }
            }
        }

        impl $dec {
            pub fn new() -> Box<Decoder> { box $dec { st: $stmod::$inist } as Box<Decoder> }
        }

        impl Decoder for $dec {
            fn from_self(&self) -> Box<Decoder> { $dec::new() }
            fn is_ascii_compatible(&self) -> bool { true }

            fn raw_feed(&mut self, input: &[u8],
                        output: &mut StringWriter) -> (uint, Option<CodecError>) {
                output.writer_hint(input.len());

                type Context<'a> = ::util::StatefulDecoderHelper<'a, $stmod::State>;

                $($item)*

                #[inline(always)]
                fn $inist($inictx: &mut Context) -> Result<$stmod::State,CodecError> {
                    match $inictx.read() {
                        None => Ok($stmod::$inist),
                        Some(c) => match c { $($($inilhs)|+ => $inirhs,)+ },
                    }
                }

                $(
                    #[inline(always)]
                    fn $st($ctx: &mut Context $(, $arg: $ty)*) -> Result<$stmod::State,CodecError> {
                        match $inictx.read() {
                            None => Ok($stmod::$st(() $(, $arg)*)),
                            Some(c) => match c { $($($lhs)|+ => $rhs,)+ },
                        }
                    }
                )*

                let mut ctx = ::util::StatefulDecoderHelper { buf: input, pos: 0, output: output };
                let mut processed = 0;

                let st_or_err = match ::std::mem::replace(&mut self.st, $stmod::$inist) {
                    $stmod::$inist => Ok($stmod::$inist),
                    $(
                        $stmod::$st(() $(, $arg)*) => $st(&mut ctx $(, $arg)*),
                    )*
                };
                match st_or_err {
                    Ok($stmod::$inist) => { processed = ctx.pos; }
                    Ok(st) => { self.st = st; return (processed, None); }
                    Err(err) => { self.st = $stmod::$inist; return (processed, Some(err)); }
                }

                while ctx.pos < ctx.buf.len() {
                    match $inist(&mut ctx) {
                        Ok($stmod::$inist) => { processed = ctx.pos; }
                        Ok(st) => { self.st = st; return (processed, None); }
                        Err(err) => { self.st = $stmod::$inist; return (processed, Some(err)); }
                    }
                }
                (processed, None)
            }

            fn raw_finish(&mut self, _output: &mut StringWriter) -> Option<CodecError> {
                match ::std::mem::replace(&mut self.st, $stmod::$inist) {
                    $stmod::$inist => None,
                    _ => Some(CodecError { upto: 0,
                                           cause: "incomplete sequence".into_maybe_owned() }),
                }
            }
        }
    )
)


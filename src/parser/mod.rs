use std::ops::Range;

#[macro_use]
mod parser;
mod lexer;
mod result;
mod ast;

#[derive(Debug,Clone)]
pub enum ParseError<'a> {
    BadToken(&'a str, Range<usize>),
    FailedOnValidation(&'a str, usize),
    FinishedOnFail,
    ReachedEOF(usize),
    UnreachedEOF(usize),
}

#[macro_export]
macro_rules! token {
  ($obj:expr => $($matcher:pat $(if $pred:expr)* => $result:expr),*) => {
      match $obj {
            Ok((t,p)) => match t {
                $($matcher $(if $pred)* => ParseResult::Success($result, p + 1)),*,
                _ => ParseResult::Fail(p)
            }
            Err(e) => ParseResult::Error(e)
        }

   };
  ($obj:expr => $($matcher:pat $(if $pred:expr)*),*) => {
      match $obj {
            Ok((t,p)) => match t {
                $($matcher $(if $pred)* => ParseResult::Success(EmptyToken{}, p + 1)),*,
                _ => ParseResult::Fail(p)
            }
            Err(e) => ParseResult::Error(e)
        }

   }

}

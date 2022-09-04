use crate::parser::lexer::Token;
use crate::parser::ParseError;
use std::borrow::Borrow;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use ParseError::{FailedOnValidation, ReachedEOF};
use ParseResult::{Error, Fail, Success};

#[derive(Debug, Clone)]
pub enum ParseResult<'a, T> {
    Success(T, usize),
    Fail(usize),
    Error(ParseError<'a>),
}

impl<'a, L, R> ParseResult<'a, (L, R)> {
    pub fn take_left(self) -> ParseResult<'a, L> {
        self.map(|(s, _)| s)
    }
    pub fn take_right(self) -> ParseResult<'a, R> {
        self.map(|(_, s)| s)
    }
}

impl<'a, L> ParseResult<'a, (L, Vec<L>)> {
    pub fn merge(self) -> ParseResult<'a, Vec<L>> {
        self.map(|(h, mut rest)| {
            rest.insert(0, h);
            rest
        })
    }
}
impl<'a, L> ParseResult<'a, (Option<L>, Vec<L>)> {
    pub fn merge(self) -> ParseResult<'a, Vec<L>> {
        self.map(|(h, mut rest)| match h {
            None => rest,
            Some(el) => {
                rest.insert(0, el);
                rest
            }
        })
    }
}

impl<'a, L: Eq + Hash, R> ParseResult<'a, Vec<(L, R)>> {
    pub fn to_map(self) -> ParseResult<'a, HashMap<L, R>> {
        self.map(|r| r.into_iter().collect::<HashMap<_, _>>())
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn then_zip<Res, Then>(self, then: Then) -> ParseResult<'a, (T, Res)>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Res>,
    {
        self.then_combine(then, |a, b| (a, b))
    }
    pub fn then_or_val_zip<Res, Then>(self, then: Then, default: Res) -> ParseResult<'a, (T, Res)>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Res>,
    {
        self.then_or_val_combine(then, default, |a, b| (a, b))
    }

    pub fn then_or_none_zip<Rhs, Then>(self, then: Then) -> ParseResult<'a, (T, Option<Rhs>)>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Option<Rhs>>,
    {
        self.then_or_none_combine(then, |a, b| (a, b))
    }
    pub fn then_or_default_zip<Rhs: Default, Then>(self, then: Then) -> ParseResult<'a, (T, Rhs)>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
    {
        self.then_or_val_zip(then, Rhs::default())
    }

    pub fn then_combine<Rhs, Res, Then, Combine>(
        self,
        then: Then,
        combine: Combine,
    ) -> ParseResult<'a, Res>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
        Combine: FnOnce(T, Rhs) -> Res,
    {
        match self {
            Success(t, pos) => match then(pos) {
                Success(r, pos) => Success(combine(t, r), pos),
                Fail(pos) => Fail(pos),
                Error(e) => Error(e),
            },
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }

    pub fn then_multi_combine<K, R, Then, Combine>(
        self,
        then: Then,
        combine: Combine,
    ) -> ParseResult<'a, R>
    where
        Then: FnOnce(usize) -> ParseResult<'a, K> + Copy,
        Combine: FnOnce(T, Vec<K>) -> R,
    {
        match self {
            Success(t, pos) => {
                let mut vals = vec![];
                let mut pos = pos;
                loop {
                    match then(pos) {
                        Success(r, next_pos) => {
                            vals.push(r);
                            pos = next_pos
                        }
                        _ => break,
                    }
                }
                Success(combine(t, vals), pos)
            }
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }
    pub fn then_multi_zip<R, Then>(self, then: Then) -> ParseResult<'a, (T, Vec<R>)>
    where
        Then: FnOnce(usize) -> ParseResult<'a, R> + Copy,
    {
        self.then_multi_combine(then, |f, v| (f, v))
    }

    pub fn then_or_val_combine<Rhs, Res, Then, Combine>(
        self,
        then: Then,
        default: Rhs,
        combine: Combine,
    ) -> ParseResult<'a, Res>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
        Combine: FnOnce(T, Rhs) -> Res,
    {
        match self {
            Success(t, pos) => match then(pos) {
                Success(r, pos) => Success(combine(t, r), pos),
                Fail(pos) => Success(combine(t, default), pos),
                Error(ReachedEOF(pos)) => Success(combine(t, default), pos),
                Error(e) => Error(e),
            },
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }
    pub fn then_or_none_combine<Rhs, Res, Then, Combine>(
        self,
        then: Then,
        combine: Combine,
    ) -> ParseResult<'a, Res>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Option<Rhs>>,
        Combine: FnOnce(T, Option<Rhs>) -> Res,
    {
        self.then_or_val_combine(then, None, combine)
    }

    pub fn then<Rhs, Then>(self, then: Then) -> ParseResult<'a, Rhs>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
    {
        self.then_combine(then, |_, k| k)
    }

    pub fn then_or_val<Rhs, Then>(self, then: Then, default: Rhs) -> ParseResult<'a, Rhs>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
    {
        match self {
            Success(_, pos) => then(pos).or_val(default),
            other => other.map(|_| default),
        }
    }
    pub fn then_or_default<Rhs: Default, Then>(self, then: Then) -> ParseResult<'a, Rhs>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
    {
        match self {
            Success(_, pos) => then(pos).or_val(Rhs::default()),
            other => other.map(|_| Rhs::default()),
        }
    }
    pub fn then_or_none<Rhs, Then>(self, then: Then) -> ParseResult<'a, Option<Rhs>>
    where
        Then: FnOnce(usize) -> ParseResult<'a, Option<Rhs>>,
    {
        self.then_or_val(then, None)
    }
}
impl<'a, Rhs: Debug, Lhs: Debug> ParseResult<'a, (Lhs, Rhs)> {
    pub fn debug1_show_last(self, prefix: &'a str) -> ParseResult<'a, (Lhs, Rhs)> {
        self.debug1_show(prefix, |(_, x)| x)
    }
    pub fn debug_show_last(self) -> ParseResult<'a, (Lhs, Rhs)> {
        self.debug_show( |(_, x)| x)
    }
}
impl<'a, T: Debug> ParseResult<'a, T> {
    pub fn debug(self) -> ParseResult<'a, T> {
       self.debug1("")
    }
    pub fn debug_show<Show, To>(self, show: Show) -> ParseResult<'a, T>
        where
            Show: FnOnce(&T) -> &To,
            To: Debug,
    {
        self.debug1_show("",show)
    }
    pub fn debug1(self, prefix: &'a str) -> ParseResult<'a, T> {
        match self {
            Success(v, pos) => {
                println!(
                    "debug | {} success, pos: {} , res: {:?}",
                    prefix, pos, v
                );
                Success(v, pos)
            }
            Fail(pos) => {
                println!("debug | {} fail, pos: {}", prefix, pos);
                Fail(pos)
            }
            Error(e) => {
                println!("debug | {} error {:?}", prefix, e);
                Error(e)
            }
        }
    }
    pub fn debug1_show<Show, To>(self, prefix: &'a str, show: Show) -> ParseResult<'a, T>
    where
        Show: FnOnce(&T) -> &To,
        To: Debug,
    {
        match self {
            Success(v, pos) => {
                println!(
                    "debug | {} success, pos: {} , res: {:?}",
                    prefix,
                    pos,
                    show(v.borrow())
                );
                Success(v, pos)
            }
            Fail(pos) => {
                println!("debug | {} fail, pos: {}", prefix, pos);
                Fail(pos)
            }
            Error(e) => {
                println!("debug | {} error {:?}", prefix, e);
                Error(e)
            }
        }
    }


}

impl<'a, T> ParseResult<'a, T> {
    pub fn ok(self) -> ParseResult<'a, Option<T>> {
        self.map(|x| Some(x))
    }
    pub fn map<Rhs, Map>(self, mapper: Map) -> ParseResult<'a, Rhs>
    where
        Map: FnOnce(T) -> Rhs,
    {
        match self {
            Success(t, pos) => Success(mapper(t), pos),
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }
    pub fn combine<Rhs, Res, Combine>(
        self,
        other: ParseResult<'a, Rhs>,
        comb: Combine,
    ) -> ParseResult<'a, Res>
    where
        Combine: FnOnce(T, Rhs) -> Res,
    {
        match (self, other) {
            (Success(t, l), Success(k, r)) => Success(comb(t, k), max(l, r)),
            (Fail(l), Fail(r)) => Fail(max(l, r)),
            (Error(e), _) | (_, Error(e)) => Error(e),
            (Fail(pos), _) | (_, Fail(pos)) => Fail(pos),
        }
    }
    pub fn validate<Validation>(self, validate: Validation) -> ParseResult<'a, T>
    where
        Validation: FnOnce(&T) -> Result<(), &'a str>,
    {
        match self {
            Success(r, pos) => match validate(r.borrow()) {
                Ok(_) => Success(r, pos),
                Err(mes) => Error(FailedOnValidation(mes, pos)),
            },
            other => other,
        }
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn or_val(self, default: T) -> ParseResult<'a, T> {
        match self {
            Fail(pos) => Success(default, pos),
            Error(ReachedEOF(pos)) => Success(default, pos),
            other => other,
        }
    }
    pub fn or_none(self) -> ParseResult<'a, Option<T>> {
        self.map(|x| Some(x)).or_val(None)
    }
    pub fn or<Alt>(self, next: Alt) -> ParseResult<'a, T>
    where
        Alt: FnOnce(usize) -> ParseResult<'a, T>,
    {
        match self {
            Fail(pos) => next(pos),
            Error(ReachedEOF(pos)) => next(pos),
            other => other,
        }
    }

    pub fn or_from(self, pos: usize) -> Alt<'a, T> {
        Alt {
            init_pos: pos,
            current: self,
        }
    }
}

pub struct Alt<'a, T> {
    init_pos: usize,
    current: ParseResult<'a, T>,
}

impl<'a, T> Alt<'a, T> {
    fn next<Next>(self, next: Next) -> Alt<'a, T>
    where
        Next: FnOnce(usize) -> ParseResult<'a, T>,
    {
        Alt {
            init_pos: self.init_pos,
            current: next(self.init_pos),
        }
    }

    pub fn or<Next>(self, next: Next) -> Alt<'a, T>
    where
        Next: FnOnce(usize) -> ParseResult<'a, T>,
    {
        match self.current {
            Fail(_) => self.next(next),
            Error(ReachedEOF(_)) => self.next(next),
            other => Alt {
                init_pos: self.init_pos,
                current: other,
            },
        }
    }
}

impl<'a, T> Into<ParseResult<'a, T>> for Alt<'a, T> {
    fn into(self) -> ParseResult<'a, T> {
        self.current
    }
}

impl<'a, T> Into<Result<T, ParseError<'a>>> for ParseResult<'a, T> {
    fn into(self) -> Result<T, ParseError<'a>> {
        match self {
            Success(t, _) => Ok(t),
            Fail(_) => Err(ParseError::FinishedOnFail),
            Error(e) => Err(e),
        }
    }
}

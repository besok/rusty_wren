use std::borrow::Borrow;
use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;
use ParseError::{FailedOnValidation, ReachedEOF};
use ParseResult::{Error, Fail, Success};
use crate::parser::lexer::Token;
use crate::parser::ParseError;

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

impl<'a, L: Eq + Hash, R> ParseResult<'a, Vec<(L, R)>> {
    pub fn to_map(self) -> ParseResult<'a, HashMap<L, R>> {
        self.map(|r| {
            let m: HashMap<_, _> = r.into_iter().collect();
            m
        })
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn then_zip<Res, Then>(self, then: Then) -> ParseResult<'a, (T, Res)>
        where Then: FnOnce(usize) -> ParseResult<'a, Res> {
        self.then_combine(then, |a, b| (a, b))
    }
    pub fn then_or_default_zip<Res, Then>(self, then: Then, default: Res) -> ParseResult<'a, (T, Res)>
        where Then: FnOnce(usize) -> ParseResult<'a, Res> {
        self.then_or_default_combine(then, default, |a, b| (a, b))
    }

    pub fn then_or_none_zip<Rhs, Then>(self, then: Then) -> ParseResult<'a, (T, Option<Rhs>)>
        where Then: FnOnce(usize) -> ParseResult<'a, Option<Rhs>> {
        self.then_or_none_combine(then, |a, b| (a, b))
    }

    pub fn then_combine<Rhs, Res, Then, Combine>(self, then: Then, combine: Combine) -> ParseResult<'a, Res>
        where
            Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
            Combine: FnOnce(T, Rhs) -> Res {
        match self {
            Success(t, pos) => {
                match then(pos) {
                    Success(r, pos) => Success(combine(t, r), pos),
                    Fail(pos) => Fail(pos),
                    Error(e) => Error(e),
                }
            }
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }

    pub fn then_multi_combine<K, R, Then, Combine>(self, then: Then, combine: Combine) -> ParseResult<'a, R>
        where
            Then: FnOnce(usize) -> ParseResult<'a, K> + Copy,
            Combine: FnOnce(T, Vec<K>) -> R {
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
                        _ => break
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
            Then: FnOnce(usize) -> ParseResult<'a, R> + Copy {
        self.then_multi_combine(then, |f, v| (f, v))
    }

    pub fn then_or_default_combine<Rhs, Res, Then, Combine>(self, then: Then, default: Rhs, combine: Combine) -> ParseResult<'a, Res>
        where
            Then: FnOnce(usize) -> ParseResult<'a, Rhs>,
            Combine: FnOnce(T, Rhs) -> Res {
        match self {
            Success(t, pos) =>
                match then(pos) {
                    Success(r, pos) => Success(combine(t, r), pos),
                    Fail(pos) => Success(combine(t, default), pos),
                    Error(ReachedEOF(pos)) => Success(combine(t, default), pos),
                    Error(e) => Error(e),
                },
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }
    pub fn then_or_none_combine<Rhs, Res, Then, Combine>(self, then: Then, combine: Combine) -> ParseResult<'a, Res>
        where
            Then: FnOnce(usize) -> ParseResult<'a, Option<Rhs>>,
            Combine: FnOnce(T, Option<Rhs>) -> Res {
        self.then_or_default_combine(then, None, combine)
    }

    pub fn then<Rhs, Then>(self, then: Then) -> ParseResult<'a, Rhs>
        where Then: FnOnce(usize) -> ParseResult<'a, Rhs> {
        self.then_combine(then, |_, k| k)
    }

    pub fn then_or_default<Rhs, Then>(self, then: Then, default: Rhs) -> ParseResult<'a, Rhs>
        where Then: FnOnce(usize) -> ParseResult<'a, Rhs> {
        match self {
            Success(_, pos) => then(pos).or_default(default),
            other => other.map(|_| default),
        }
    }
    pub fn then_or_def_val<Rhs:Default, Then>(self, then: Then ) -> ParseResult<'a, Rhs>
        where Then: FnOnce(usize) -> ParseResult<'a, Rhs> {
        match self {
            Success(_, pos) => then(pos).or_default(Rhs::default()),
            other => other.map(|_| Rhs::default()),
        }
    }
    pub fn then_or_none<Rhs, Then>(self, then: Then) -> ParseResult<'a, Option<Rhs>>
        where Then: FnOnce(usize) -> ParseResult<'a, Option<Rhs>> {
        self.then_or_default(then, None)
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn debug(self) -> ParseResult<'a, T> {
        match self {
            Success(v, pos) => {
                println!("success, the pos is {}", pos);
                Success(v, pos)
            }
            Fail(pos) => {
                println!("fail, the pos is {}", pos);
                Fail(pos)
            }
            Error(e) => {
                println!("error {:?}", e);
                Error(e)
            }
        }
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn ok(self) -> ParseResult<'a, Option<T>> {
        self.map(|x| Some(x))
    }
    pub fn map<Rhs, Map>(self, mapper: Map) -> ParseResult<'a, Rhs> where Map: FnOnce(T) -> Rhs {
        match self {
            Success(t, pos) => Success(mapper(t), pos),
            Fail(pos) => Fail(pos),
            Error(e) => Error(e),
        }
    }
    pub fn combine<Rhs, Res, Combine>(self, other: ParseResult<'a, Rhs>, comb: Combine) -> ParseResult<'a, Res>
        where Combine: FnOnce(T, Rhs) -> Res {
        match (self, other) {
            (Success(t, l), Success(k, r)) => Success(comb(t, k), max(l, r)),
            (Fail(l), Fail(r)) => Fail(max(l, r)),
            (Error(e), _) | (_, Error(e)) => Error(e),
            (Fail(pos), _) | (_, Fail(pos)) => Fail(pos),
        }
    }
    pub fn validate<Validation>(self, validate: Validation) -> ParseResult<'a, T>
        where Validation: FnOnce(&T) -> Result<(), &'a str> {
        match self {
            Success(r, pos) => {
                match validate(r.borrow()) {
                    Ok(_) => Success(r, pos),
                    Err(mes) => Error(FailedOnValidation(mes, pos))
                }
            }
            other => other
        }
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn or_default(self, default: T) -> ParseResult<'a, T> {
        match self {
            Fail(pos) => Success(default, pos),
            Error(ReachedEOF(pos)) => Success(default, pos),
            other => other
        }
    }
    pub fn or_none(self) -> ParseResult<'a, Option<T>> {
        self.map(|x| Some(x)).or_default(None)
    }
    pub fn or<Alt>(self, next: Alt) -> ParseResult<'a, T> where Alt: FnOnce(usize) -> ParseResult<'a, T> {
        match self {
            Fail(pos) => next(pos),
            other => other
        }
    }
}

impl<'a, T> Into<Result<T, ParseError<'a>>> for ParseResult<'a, T> {
    fn into(self) -> Result<T, ParseError<'a>> {
        match self {
            Success(t, _) => Ok(t),
            Fail(_) => Err(ParseError::FinishedOnFail),
            Error(e) => Err(e)
        }
    }
}
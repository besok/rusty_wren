use std::collections::HashMap;
use std::iter::Map;
use crate::parser::lexer::{CypherLexer, Token};
use crate::parser::ParseError;
use crate::parser::ParseError::UnreachedEOF;
use crate::parser::result::ParseResult;
use crate::parser::result::ParseResult::Success;
use crate::token;

struct CypherParser<'a> {
    lexer: CypherLexer<'a>,
}

impl<'a> CypherParser<'a> {
    pub fn new(src: &'a str) -> Result<Self, ParseError> {
        Ok(CypherParser { lexer: CypherLexer::new(src)? })
    }
    pub fn token(&self, pos: usize) -> Result<(&Token<'a>, usize), ParseError<'a>> {
        self.lexer.token(pos)
    }
    pub fn one_or_more<T, Then>(&self, pos: usize, then: Then) -> ParseResult<'a, Vec<T>>
        where Then: FnOnce(usize) -> ParseResult<'a, T> + Copy {
        then(pos)
            .then_multi_zip(|p| then(p))
            .merge()
    }

    pub fn validate_eof<T>(&self, res: ParseResult<'a, T>) -> ParseResult<'a, T> {
        match res {
            Success(_, pos) if self.lexer.len() != pos => ParseResult::Error(UnreachedEOF(pos)),
            other => other,
        }
    }
}

impl<'a> CypherParser<'a> {}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use crate::parser::ParseError;
    use crate::parser::parser::CypherParser;
    use crate::parser::result::ParseResult;
    use crate::parser::structure::{Atom, CaseExpression, Expression, FilterExpression, FilterWith, InvocationName, KeyWord, ListComprehension, Literal, Name, Number, Parameter, Properties, RangeLit, RelationDetail, RelationshipPattern, RelationshipTypes, Selector, Symbol};
    use crate::parser::structure::Expression::Null;
    use crate::parser::structure::Number::Int;
    use crate::parser::structure::Symbol::{COUNT, EscLit, Id};


    #[test]
    fn smoke_test() {
        parser("");
    }


    fn parser(src: &str) -> CypherParser {
        match CypherParser::new(src) {
            Ok(p) => p,
            Err(e) => panic!("{:?}", e)
        }
    }

    fn success<T>(res: ParseResult<T>) {
        match res {
            ParseResult::Success(_, _) => {}
            ParseResult::Fail(pos) => panic!("failed on {}", pos),
            ParseResult::Error(e) => panic!("error: {:?}", e)
        }
    }

    fn expect<T>(res: ParseResult<T>, expect: T) where T: PartialEq + Debug {
        match res {
            ParseResult::Success(v, _) => assert_eq!(v, expect),
            ParseResult::Fail(pos) => panic!("failed on {}", pos),
            ParseResult::Error(e) => panic!("error: {:?}", e)
        }
    }

    fn expect_pos<T>(res: ParseResult<T>, expect: usize) where T: PartialEq + Debug {
        match res {
            ParseResult::Success(_, pos) => assert_eq!(pos, expect, "actual:{:?}, expect:{:?}", pos, expect),
            ParseResult::Fail(pos) => panic!("failed on {}", pos),
            ParseResult::Error(e) => panic!("error: {:?}", e)
        }
    }

    fn fail<T: Debug>(res: ParseResult<T>) {
        match res {
            ParseResult::Success(v, pos) => panic!(" expect to get  fail but got {:?} on pos {pos}", v),
            ParseResult::Fail(pos) => {}
            ParseResult::Error(e) => panic!("error: {:?}", e)
        }
    }

    fn fail_on<T: Debug>(res: ParseResult<T>, expect: usize) {
        match res {
            ParseResult::Success(v, pos) => panic!(" expect to get  fail but got {:?} on pos {pos}", v),
            ParseResult::Fail(pos) => assert_eq!(pos, expect, "actual:{:?}, expect:{:?}", pos, expect),
            ParseResult::Error(e) => panic!("error: {:?}", e)
        }
    }
}


use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::Map;
use crate::parser::ast::{AtomExpression, Elvis, Expression, Id, Number, EmptyToken, Enumeration, Params, Statement, Block, Call, BlockOrEnum};
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

impl<'a> CypherParser<'a> {
    fn id(&self, pos: usize) -> ParseResult<'a, Id<'a>> {
        token!(self.token(pos) =>
            Token::Id(value) => Id{value}
        )
    }
    fn number(&self, pos: usize) -> ParseResult<'a, Number> {
        token!(self.token(pos) =>
            Token::Digit(number) => *number
        )
    }
    fn null(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) => Token::Null => AtomExpression::Null)
    }
    fn bool(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) =>
            Token::True => AtomExpression::Bool(true),
            Token::False => AtomExpression::Bool(false)
        )
    }
    fn char(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) =>
            Token::CharLit(v) => AtomExpression::CharLit(v)
        )
    }
    fn string(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) =>
            Token::StringLit(v) => AtomExpression::StringLit(v),
            Token::TextBlock(v) => AtomExpression::StringLit(v)
        )
    }
    fn number_expr(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        self.number(pos).map(AtomExpression::Number)
    }


    fn map_init(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        let one_pair = |p| self.expression(p)
            .then_zip(|p| token!(self.token(p) => Token::Colon))
            .take_left()
            .then_zip(|p| self.expression(p));

        let all_pairs = |p|
            one_pair(p)
                .then_multi_zip(|p| token!(self.token(p) => Token::Comma).then(one_pair))
                .merge()
                .or_default(vec![]);

        token!(self.token(pos) => Token::LBrace)
            .then(all_pairs)
            .then_zip(|p| token!(self.token(p) => Token::RBrace))
            .take_left()
            .map(AtomExpression::MapInit)
    }
    fn list_init(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) => Token::LBrack)
            .then(|p| self.enumeration(p).or_default(Enumeration::default()))
            .then_zip(|p| token!(self.token(p) => Token::RBrack))
            .take_left()
            .map(AtomExpression::ListInit)
    }

    fn elvis(&self, pos: usize) -> ParseResult<'a, Elvis<'a>> {
        token!(self.token(pos) => Token::Question)
            .then(|p| self.expression(p))
            .then_zip(|p| token!(self.token(p) => Token::Colon))
            .take_left()
            .then_zip(|p| self.expression(p))
            .map(|(lhs, rhs)| Elvis { lhs, rhs })
    }

    fn expression(&self, pos: usize) -> ParseResult<'a, Expression<'a>> {
        token!(self.token(pos) =>
            Token::RShift => Expression::E
        )
    }

    fn enumeration(&self, pos: usize) -> ParseResult<'a, Enumeration<'a>> {
        let tail = |p|
            token!(self.token(p) => Token::Comma)
                .then(|p| self.expression(p));

        self.expression(pos)
            .then_multi_zip(tail)
            .merge()
            .map(Enumeration::new)
    }

    fn statement(&self, pos: usize) -> ParseResult<'a, Statement<'a>> {
        self.expression(pos).map(Statement::Expression)
    }
    fn block(&self, pos: usize) -> ParseResult<'a, Block<'a>> {
        let params = |p|
            token!(self.token(p) => Token::BitOr)
                .then(|p| self.params(p))
                .then_zip(|p| token!(self.token(p) => Token::BitOr))
                .take_left();


        token!(self.token(pos) => Token::LBrace)
            .then_or_default(params, Params::default())
            .then_multi_zip(|p| self.statement(p))
            .map(|(params, statements)| Block { params, statements })
            .then_zip(|p| token!(self.token(p) => Token::RBrace))
            .take_left()
    }
    fn params(&self, pos: usize) -> ParseResult<'a, Params<'a>> {
        self.id(pos)
            .then_multi_zip(|p|
                token!(self.token(p) => Token::Comma)
                    .then(|p| self.id(p)))
            .merge()
            .map(|ids| Params { ids })
    }

    fn call(&self, pos: usize) -> ParseResult<'a, Call<'a>> {
        let enumeration = |p|
            token!(self.token(p) => Token::LParen)
                .then_or_def_val(|p| self.enumeration(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .map(BlockOrEnum::Enum);

        let block_or_enum = |p|
            self.block(p).map(BlockOrEnum::Block).or(enumeration);

        let tail = |p|
            token!(self.token(p) => Token::Dot).then(|p| self.call(p)).or_none();

        self.id(pos)
            .then_or_default_zip(block_or_enum, BlockOrEnum::None)
            .then_or_none_zip(tail)
            .map(|((id, middle), tail)|
                Call { id, tail: tail.map(Box::new), middle }
            )
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::Debug;
    use crate::parser::ast::{AtomExpression, Enumeration, Expression};
    use crate::parser::ParseError;
    use crate::parser::parser::CypherParser;
    use crate::parser::result::ParseResult;


    #[test]
    fn null_test() {
        expect(
            parser("null").null(0),
            AtomExpression::Null,
        );
        fail(parser("not_null").null(0));

        expect_pos(parser("? >> : >>").elvis(0), 4);
    }

    #[test]
    fn enum_test() {
        expect_pos(parser(">>").enumeration(0), 1);
        expect_pos(parser(">>, >>").enumeration(0), 3);
    }

    #[test]
    fn call_test() {
        expect_pos(parser("id.id.id").call(0), 5);
        expect_pos(parser("id").call(0), 1);
        expect_pos(parser("id()").call(0), 3);
        expect_pos(parser("id().id").call(0), 5);
        expect_pos(parser("id(>>).id").call(0), 6);
        expect_pos(parser("id(>>,>>).id").call(0), 8);
        expect_pos(parser("id{}.id").call(0), 5);
        expect_pos(parser("id{ >> }.id").call(0), 6);
        expect_pos(parser("id{|a| >> }.id").call(0),9 );
        expect_pos(parser("id{|a,b| >> }.id").call(0),11 );
        expect_pos(parser("id{|a,b| >> }.id().id").call(0),15 );
    }

    #[test]
    fn block_test() {
        expect_pos(parser("{}").block(0), 2);
        expect_pos(parser("{>> >>}").block(0), 4);
        expect_pos(parser("{|a| >> >>}").block(0), 7);
        fail_on(parser("{|| >> >>}").block(0), 1);
    }

    #[test]
    fn map_init_test() {
        expect_pos(parser("{}").map_init(0), 2);
        expect_pos(parser("{>> : >>}").map_init(0), 5);
        expect_pos(parser("{>> : >>, >> : >>}").map_init(0), 9);
    }

    #[test]
    fn list_init_test() {
        expect_pos(parser("[]").list_init(0), 2);
        expect_pos(parser("[>>]").list_init(0), 3);
        expect_pos(parser("[>> , >>]").list_init(0), 5);
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
            ParseResult::Success(v, pos) => {
                println!("{:?}", v);
                assert_eq!(pos, expect, "actual:{:?}, expect:{:?}", pos, expect)
            }
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


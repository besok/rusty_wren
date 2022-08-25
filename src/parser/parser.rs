use crate::parser::ast::{
    AtomExpression, Block, BlockOrEnum, Call, Elvis, EmptyToken, Enumeration,
    Expression, Function, Id, ImportModule, ImportVariable, Logic, LogicOp, Number,
    Params, Range, RangeExpression, Statement,
};
use crate::parser::lexer::{CypherLexer, Token};
use crate::parser::result::ParseResult;
use crate::parser::result::ParseResult::Success;
use crate::parser::ParseError;
use crate::parser::ParseError::UnreachedEOF;
use crate::token;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::Map;

struct CypherParser<'a> {
    lexer: CypherLexer<'a>,
}

impl<'a> CypherParser<'a> {
    pub fn new(src: &'a str) -> Result<Self, ParseError> {
        Ok(CypherParser {
            lexer: CypherLexer::new(src)?,
        })
    }
    pub fn token(&self, pos: usize) -> Result<(&Token<'a>, usize), ParseError<'a>> {
        self.lexer.token(pos)
    }
    pub fn one_or_more<T, Then>(&self, pos: usize, then: Then) -> ParseResult<'a, Vec<T>>
    where
        Then: FnOnce(usize) -> ParseResult<'a, T> + Copy,
    {
        then(pos).then_multi_zip(|p| then(p)).merge()
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
    fn string(&self, pos: usize) -> ParseResult<'a, &'a str> {
        token!(self.token(pos) =>
            Token::StringLit(v) => *v,
            Token::TextBlock(v) => *v
        )
    }
    fn number_expr(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        self.number(pos).map(AtomExpression::Number)
    }

    fn map_init(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        let one_pair = |p| {
            self.expression(p)
                .then_zip(|p| token!(self.token(p) => Token::Colon))
                .take_left()
                .then_zip(|p| self.expression(p))
        };

        let all_pairs = |p| {
            one_pair(p)
                .then_multi_zip(|p| token!(self.token(p) => Token::Comma).then(one_pair))
                .merge()
                .or_val(vec![])
        };

        token!(self.token(pos) => Token::LBrace)
            .then(all_pairs)
            .then_zip(|p| token!(self.token(p) => Token::RBrace))
            .take_left()
            .map(AtomExpression::MapInit)
    }
    fn list_init(&self, pos: usize) -> ParseResult<'a, Enumeration<'a>> {
        token!(self.token(pos) => Token::LBrack)
            .then_or_default(|p| self.enumeration(p))
            .then_zip(|p| token!(self.token(p) => Token::RBrack))
            .take_left()
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
        token!(self.token(pos) =>Token::RShift => Expression::E)
    }

    fn enumeration(&self, pos: usize) -> ParseResult<'a, Enumeration<'a>> {
        let tail = |p| token!(self.token(p) => Token::Comma).then(|p| self.expression(p));

        self.expression(pos)
            .then_multi_zip(tail)
            .merge()
            .map(Enumeration::new)
    }

    fn statement(&self, pos: usize) -> ParseResult<'a, Statement<'a>> {
        self.expression(pos).map(Statement::Expression)
    }
    fn block(&self, pos: usize) -> ParseResult<'a, Block<'a>> {
        let params = |p| {
            token!(self.token(p) => Token::BitOr)
                .then(|p| self.params(p))
                .then_zip(|p| token!(self.token(p) => Token::BitOr))
                .take_left()
        };

        token!(self.token(pos) => Token::LBrace)
            .then_or_default(params)
            .then_multi_zip(|p| self.statement(p))
            .map(|(params, statements)| Block { params, statements })
            .then_zip(|p| token!(self.token(p) => Token::RBrace))
            .take_left()
    }
    fn params(&self, pos: usize) -> ParseResult<'a, Params<'a>> {
        self.id(pos)
            .then_multi_zip(|p| token!(self.token(p) => Token::Comma).then(|p| self.id(p)))
            .merge()
            .map(|ids| Params { ids })
    }

    fn call(&self, pos: usize) -> ParseResult<'a, Call<'a>> {
        let enumeration = |p| {
            token!(self.token(p) => Token::LParen)
                .then_or_default(|p| self.enumeration(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .map(BlockOrEnum::Enum)
        };

        let block_or_enum = |p| self.block(p).map(BlockOrEnum::Block).or(enumeration);

        let tail = |p| {
            token!(self.token(p) => Token::Dot)
                .then(|p| self.call(p))
                .or_none()
        };

        self.id(pos)
            .then_or_val_zip(block_or_enum, BlockOrEnum::None)
            .then_or_none_zip(tail)
            .map(|((id, middle), tail)| Call {
                id,
                tail: tail.map(Box::new),
                middle,
            })
    }

    fn collection_elem(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        self.string(pos)
            .map(Call::just_id)
            .or(|p| self.call(p))
            .then_zip(|p| self.list_init(p))
            .map(|(call, enumeration)| AtomExpression::CollectionElem(call, enumeration))
    }

    fn import_variable(&self, pos: usize) -> ParseResult<'a, ImportVariable<'a>> {
        let alias = |p| token!(self.token(p) => Token::As).then_or_none(|p| self.id(p).or_none());

        self.id(pos)
            .then_or_none_zip(alias)
            .map(|(name, alias)| ImportVariable { name, alias })
    }
    fn import_module(&self, pos: usize) -> ParseResult<'a, ImportModule<'a>> {
        let import_vars = |p| {
            token!(self.token(p) => Token::For)
                .then(|p| self.import_variable(p))
                .then_multi_zip(|p| {
                    token!(self.token(p) => Token::Comma).then(|p| self.import_variable(p))
                })
                .merge()
        };

        token!(self.token(pos) => Token::Import)
            .then(|p| self.string(p))
            .then_or_val_zip(import_vars, vec![])
            .map(|(name, variables)| ImportModule { name, variables })
    }

    fn range(&self, pos: usize) -> ParseResult<'a, Range<'a>> {
        let range_expr = |p| {
            self.call(p)
                .map(RangeExpression::Call)
                .or(|p| self.number(p).map(RangeExpression::Num))
        };
        let ellipsis = |p| {
            token!(self.token(p) =>
                Token::EllipsisIn => false,
                Token::EllipsisOut => true
            )
        };
        let to_range = |((left, is_out), right)| Range {
            left,
            right,
            is_out,
        };

        range_expr(pos)
            .then_zip(ellipsis)
            .then_zip(range_expr)
            .map(to_range)
    }

    fn atom(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        let with_sub = |p| {
            token!(self.token(p) => Token::Sub)
                .then(|p| self.atom(p))
                .map(Box::new)
                .map(AtomExpression::Sub)
        };
        self.bool(pos)
            .or(|p| self.char(p))
            .or(|p| self.string(p).map(AtomExpression::StringLit))
            .or(|p| self.number(p).map(AtomExpression::Number))
            .or(|p| self.null(p))
            .or(|p| self.list_init(p).map(AtomExpression::ListInit))
            .or(|p| self.map_init(p))
            .or(|p| self.call(p).map(AtomExpression::Call))
            .or(|p| self.range(p).map(AtomExpression::Range))
            .or(|p| self.collection_elem(p))
            .or(|p| token!(self.token(p) => Token::Break => AtomExpression::Break))
            .or(|p| token!(self.token(p) => Token::Continue => AtomExpression::Continue))
            .or(|p| self.import_module(p).map(AtomExpression::ImportModule))
            .or(with_sub)
    }

    fn function(&self, pos: usize) -> ParseResult<'a, Function<'a>> {
        let params = |p| {
            token!(self.token(p) => Token::LParen)
                .then_or_default(|p| self.params(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
        };

        let to_fn = |((name, params), block)| Function {
            name,
            params,
            block,
        };
        self.id(pos)
            .then_zip(params)
            .then_or_none_zip(|p| self.block(p).or_none())
            .map(to_fn)
    }

    fn logic_atom(&self, pos: usize) -> ParseResult<'a, Logic<'a>> {
        token!(self.token(pos) =>
            Token::Or => LogicOp::Or,
            Token::Gt => LogicOp::Gt,
            Token::Ge => LogicOp::Ge,
            Token::Equal => LogicOp::Eq,
            Token::NotEqual => LogicOp::NotEq,
            Token::Lt => LogicOp::Lt,
            Token::Le => LogicOp::Le,
            Token::And => LogicOp::And
        )
        .then_zip(|p| self.expression(p))
        .map(|(op, value)| Logic::Atom(op, value))
    }
    fn logic(&self, pos: usize) -> ParseResult<'a, Logic<'a>> {
        let and = |p| {
            self.logic_atom(p)
                .then_multi_zip(|p| {
                    token!(self.token(p) => Token::And)
                        .then(|p| self.expression(p))
                        .then_zip(|p| self.logic_atom(p))
                        .map(|(e, l)| (e, Box::new(l)))
                })
                .map(|(l, tail)| Logic::And(Box::new(l), tail))
        };
        and(pos)
            .then_multi_zip(|p| {
                token!(self.token(p) => Token::Or)
                    .then(|p| self.expression(p))
                    .then_zip(and)
                    .map(|(e, l)| (e, Box::new(l)))
            })
            .map(|(l, tail)| Logic::Or(Box::new(l), tail))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::{AtomExpression, Enumeration, Expression};
    use crate::parser::parser::CypherParser;
    use crate::parser::result::ParseResult;
    use crate::parser::ParseError;
    use std::collections::HashMap;
    use std::fmt::Debug;

    #[test]
    fn null_test() {
        expect(parser("null").null(0), AtomExpression::Null);
        fail(parser("not_null").null(0));

        expect_pos(parser("? >> : >>").elvis(0), 4);
    }
    #[test]
    fn atom_logic_test() {
        expect_pos(parser("|| >>").logic_atom(0), 2);
        expect_pos(parser("&& >>").logic_atom(0), 2);
        expect_pos(parser("< >>").logic_atom(0), 2);
        expect_pos(parser("== >>").logic_atom(0), 2);
        expect_pos(parser("!= >>").logic_atom(0), 2);
    }
    #[test]
    fn logic_test() {
        expect_pos(parser("> >> ").logic(0), 2);
        expect_pos(parser("> >> && >> > >>").logic(0), 6);
        expect_pos(parser("> >> || >> > >> && >> > >>").logic(0), 10);
        expect_pos(parser("> >> || >> && >>").logic(0), 6);
        expect_pos(parser("|| >> && >> && >>").logic(0), 6);
    }

    #[test]
    fn range_test() {
        expect_pos(parser("1..2").range(0), 3);
        expect_pos(parser("1...2").range(0), 3);
        expect_pos(parser("a.b.c...a{}").range(0), 9);
    }
    #[test]
    fn atom_test() {
        expect_pos(parser("a.b.c").atom(0), 5);
        expect_pos(parser("-a.b.c").atom(0), 6);
    }
    #[test]
    fn enum_test() {
        expect_pos(parser(">>").enumeration(0), 1);
        expect_pos(parser(">>, >>").enumeration(0), 3);
    }
    #[test]
    fn import_mod_test() {
        expect_pos(parser("a as b").import_variable(0), 3);
        expect_pos(parser("import \"abc\" ").import_module(0), 2);
        expect_pos(
            parser("import \"abc\" for a as b, b as d").import_module(0),
            10,
        );
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
        expect_pos(parser("id{|a| >> }.id").call(0), 9);
        expect_pos(parser("id{|a,b| >> }.id").call(0), 11);
        expect_pos(parser("id{|a,b| >> }.id().id").call(0), 15);
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
            Err(e) => panic!("{:?}", e),
        }
    }

    fn success<T>(res: ParseResult<T>) {
        match res {
            ParseResult::Success(_, _) => {}
            ParseResult::Fail(pos) => panic!("failed on {}", pos),
            ParseResult::Error(e) => panic!("error: {:?}", e),
        }
    }

    fn expect<T>(res: ParseResult<T>, expect: T)
    where
        T: PartialEq + Debug,
    {
        match res {
            ParseResult::Success(v, _) => assert_eq!(v, expect),
            ParseResult::Fail(pos) => panic!("failed on {}", pos),
            ParseResult::Error(e) => panic!("error: {:?}", e),
        }
    }

    fn expect_pos<T>(res: ParseResult<T>, expect: usize)
    where
        T: PartialEq + Debug,
    {
        match res {
            ParseResult::Success(v, pos) => {
                println!("{:?}", v);
                assert_eq!(pos, expect, "actual:{:?}, expect:{:?}", pos, expect)
            }
            ParseResult::Fail(pos) => panic!("failed on {}", pos),
            ParseResult::Error(e) => panic!("error: {:?}", e),
        }
    }

    fn fail<T: Debug>(res: ParseResult<T>) {
        match res {
            ParseResult::Success(v, pos) => {
                panic!(" expect to get  fail but got {:?} on pos {pos}", v)
            }
            ParseResult::Fail(pos) => {}
            ParseResult::Error(e) => panic!("error: {:?}", e),
        }
    }

    fn fail_on<T: Debug>(res: ParseResult<T>, expect: usize) {
        match res {
            ParseResult::Success(v, pos) => {
                panic!(" expect to get  fail but got {:?} on pos {pos}", v)
            }
            ParseResult::Fail(pos) => {
                assert_eq!(pos, expect, "actual:{:?}, expect:{:?}", pos, expect)
            }
            ParseResult::Error(e) => panic!("error: {:?}", e),
        }
    }
}

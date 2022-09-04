use crate::parser::ast::*;
use crate::parser::lexer::Token::Class;
use crate::parser::lexer::{CypherLexer, Token};
use crate::parser::result::ParseResult;
use crate::parser::result::ParseResult::{Error, Fail, Success};
use crate::parser::ParseError;
use crate::parser::ParseError::{ReachedEOF, UnreachedEOF};
use crate::token;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::iter::Map;

pub struct CypherParser<'a> {
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
        match self.zero_or_more(pos, then) {
            Success(vals, _) if vals.is_empty() => Fail(pos),
            other => other,
        }
    }

    pub fn zero_or_more<T, Then>(&self, pos: usize, then: Then) -> ParseResult<'a, Vec<T>>
    where
        Then: FnOnce(usize) -> ParseResult<'a, T> + Copy,
    {
        match then(pos).then_multi_zip(|p| then(p)).merge() {
            Fail(_) => Success(vec![], pos),
            Error(ReachedEOF(_)) => Success(vec![], pos),
            success => success,
        }
    }

    pub fn validate_eof<T>(&self, res: ParseResult<'a, T>) -> ParseResult<'a, T> {
        match res {
            Success(_, pos) if self.lexer.len() != pos => ParseResult::Error(UnreachedEOF(pos)),
            other => other,
        }
    }
}
impl<'a> CypherParser<'a> {
    pub fn id(&self, pos: usize) -> ParseResult<'a, Id<'a>> {
        token!(self.token(pos) =>
            Token::Id(value) => Id{value}
        )
    }
    pub fn number(&self, pos: usize) -> ParseResult<'a, Number> {
        token!(self.token(pos) =>
            Token::Digit(number) => *number
        )
    }
    pub fn null(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) => Token::Null => AtomExpression::Null)
    }
    pub fn bool(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) =>
            Token::True => AtomExpression::Bool(true),
            Token::False => AtomExpression::Bool(false)
        )
    }
    pub fn char(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        token!(self.token(pos) =>
            Token::CharLit(v) => AtomExpression::CharLit(v)
        )
    }
    pub fn string(&self, pos: usize) -> ParseResult<'a, &'a str> {
        token!(self.token(pos) =>
            Token::StringLit(v) => *v,
            Token::TextBlock(v) => *v
        )
    }
    pub fn number_expr(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        self.number(pos).map(AtomExpression::Number)
    }

    pub fn map_init(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
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
    pub fn list_init(&self, pos: usize) -> ParseResult<'a, Enumeration<'a>> {
        token!(self.token(pos) => Token::LBrack)
            .then_or_default(|p| self.enumeration(p))
            .then_zip(|p| token!(self.token(p) => Token::RBrack))
            .take_left()
    }

    pub fn elvis(&self, pos: usize) -> ParseResult<'a, Elvis<'a>> {
        token!(self.token(pos) => Token::Question)
            .then(|p| self.expression(p))
            .then_zip(|p| token!(self.token(p) => Token::Colon))
            .take_left()
            .then_zip(|p| self.expression(p))
            .map(|(lhs, rhs)| Elvis { lhs, rhs })
    }

    pub fn expression(&self, pos: usize) -> ParseResult<'a, Expression<'a>> {
        let not = |p| {
            token!(self.token(p) => Token::Bang)
                .then(|p| self.expression(p))
                .map(Box::new)
                .map(Expression::Not)
        };
        let wrapped = |p| {
            token!(self.token(p) => Token::LParen)
                .then(|p| self.expression(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
        };

        let atom = |p| self.atom(p).map(Expression::Atom);

        let compound = |p| {
            let atom_or_not: ParseResult<Expression> =
                atom(p).or_from(p).or(not).or(wrapped).into();
            atom_or_not
                .then_zip(|p| self.compound_expr(p))
                .map(|(e, ce)| Expression::Compound(Box::new(e), Box::new(ce)))
        };

        compound(pos)
            .or_from(pos)
            .or(not)
            .or(wrapped)
            .or(atom)
            .into()
    }

    pub fn enumeration(&self, pos: usize) -> ParseResult<'a, Enumeration<'a>> {
        let tail = |p| token!(self.token(p) => Token::Comma).then(|p| self.expression(p));

        self.expression(pos)
            .then_multi_zip(tail)
            .merge()
            .map(Enumeration::new)
    }

    pub fn statement(&self, pos: usize) -> ParseResult<'a, Statement<'a>> {
        let ret = |p| {
            token!(self.token(p) => Token::Return)
                .then(|p| self.expression(p))
                .map(Statement::Return)
        };
        self.assignment(pos)
            .map(Statement::Assignment)
            .or_from(pos)
            .or(|p| self.assignment_null(p).map(Statement::AssignmentNull))
            .or(|p| self.block(p).map(Statement::Block))
            .or(|p| self.expression(p).map(Statement::Expression))
            .or(|p| self.if_statement(p).map(Box::new).map(Statement::If))
            .or(|p| self.while_statement(p).map(Box::new).map(Statement::While))
            .or(|p| self.for_statement(p).map(Box::new).map(Statement::For))
            .or(ret)
            .into()
    }
    pub fn file_unit(&self, pos: usize) -> ParseResult<'a, Unit<'a>> {
        self.class_def(pos)
            .map(Unit::Class)
            .or_from(pos)
            .or(|p| self.function(p).map(Unit::Fn))
            .or(|p| self.import_module(p).map(Unit::Import))
            .or(|p| self.statement(p).map(Unit::Statement))
            .or(|p| self.block(p).map(Unit::Block))
            .into()
    }

    pub fn script(&self, pos: usize) -> ParseResult<'a, Script<'a>> {
        self.one_or_more(pos, |p| self.file_unit(p))
            .map(|units| Script { units })
    }

    pub fn assignment(&self, pos: usize) -> ParseResult<'a, Assignment<'a>> {
        let op = |p| {
            token!(self.token(p) =>
                Token::Assign => AssignOp::Assign,
                Token::MultAssign => AssignOp::Sub,
                Token::AddAssign => AssignOp::Add,
                Token::DivAssign => AssignOp::Div,
                Token::AndAssign => AssignOp::And,
                Token::OrAssign => AssignOp::Or,
                Token::XOrAssign => AssignOp::Xor,
                Token::ModAssign => AssignOp::Mod,
                Token::LShift => AssignOp::LShift,
                Token::RShift => AssignOp::RShift,
                Token::URShiftAssign => AssignOp::URShift,
                Token::SubAssign => AssignOp::Mul
            )
        };

        let tail = |p| {
            self.expression(p)
                .map(Rhs::Expression)
                .or_from(p)
                .or(|p| {
                    self.one_or_more(p, |p| self.assignment(p)).map(|v| {
                        if v.len() == 1 {
                            Rhs::Assignment(v.into_iter().next().take().unwrap())
                        } else {
                            Rhs::Assignments(v)
                        }
                    })
                })
                .into()
        };
        token!(self.token(pos) => Token::Var => true)
            .or_val(false)
            .then_zip(|p| self.expression(p))
            .then_zip(op)
            .then_zip(tail)
            .map(|(((var, e), op), rhs)| Assignment {
                var,
                op,
                lhs: e,
                rhs: Box::new(rhs),
            })
    }
    pub fn assignment_null(&self, pos: usize) -> ParseResult<'a, AssignmentNull<'a>> {
        token!(self.token(pos) => Token::Var)
            .then(|p| self.id(p))
            .map(|id| AssignmentNull { id })
    }

    pub fn if_statement(&self, pos: usize) -> ParseResult<'a, If<'a>> {
        let main = |p| {
            token!(self.token(p) => Token::If)
                .then(|p| token!(self.token(p) => Token::LParen))
                .then(|p| self.expression(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .then_zip(|p| self.statement(p))
                .map(|(cond, action)| IfBranch { cond, action })
        };

        let else_ifs =
            |p| self.zero_or_more(p, |p| token!(self.token(p) => Token::Else).then(main));

        let else_opt = |p| {
            token!(self.token(p) => Token::Else)
                .then(|p| self.statement(p))
                .or_none()
        };

        main(pos)
            .then_zip(else_ifs)
            .then_or_none_zip(else_opt)
            .map(|((main, others), els)| If { main, others, els })
    }

    pub fn block(&self, pos: usize) -> ParseResult<'a, Block<'a>> {
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
    pub fn params(&self, pos: usize) -> ParseResult<'a, Params<'a>> {
        self.id(pos)
            .then_multi_zip(|p| token!(self.token(p) => Token::Comma).then(|p| self.id(p)))
            .merge()
            .map(|ids| Params { ids })
    }

    pub fn call(&self, pos: usize) -> ParseResult<'a, Call<'a>> {
        let enumeration = |p| {
            token!(self.token(p) => Token::LParen)
                .then_or_default(|p| self.enumeration(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .map(BlockOrEnum::Enum)
        };

        let block_or_enum = |p| self.block(p).map(BlockOrEnum::Block).or_last(enumeration);

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

    pub fn collection_elem(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        self.string(pos)
            .map(Call::just_id)
            .or_last(|p| self.call(p))
            .then_zip(|p| self.list_init(p))
            .map(|(call, enumeration)| AtomExpression::CollectionElem(call, enumeration))
    }

    pub fn import_variable(&self, pos: usize) -> ParseResult<'a, ImportVariable<'a>> {
        let alias = |p| token!(self.token(p) => Token::As).then_or_none(|p| self.id(p).or_none());

        self.id(pos)
            .then_or_none_zip(alias)
            .map(|(name, alias)| ImportVariable { name, alias })
    }
    pub fn import_module(&self, pos: usize) -> ParseResult<'a, ImportModule<'a>> {
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

    pub fn range(&self, pos: usize) -> ParseResult<'a, Range<'a>> {
        let range_expr = |p| {
            self.call(p)
                .map(RangeExpression::Call)
                .or_last(|p| self.number(p).map(RangeExpression::Num))
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

    pub fn atom(&self, pos: usize) -> ParseResult<'a, AtomExpression<'a>> {
        let with_sub = |p| {
            token!(self.token(p) => Token::Sub)
                .then(|p| self.atom(p))
                .map(Box::new)
                .map(AtomExpression::Sub)
        };
        self.bool(pos)
            .or_from(pos)
            .or(|p| self.import_module(p).map(AtomExpression::ImportModule))
            .or(|p| self.range(p).map(AtomExpression::Range))
            .or(|p| self.char(p))
            .or(|p| self.string(p).map(AtomExpression::StringLit))
            .or(|p| self.number(p).map(AtomExpression::Number))
            .or(|p| self.null(p))
            .or(|p| self.list_init(p).map(AtomExpression::ListInit))
            .or(|p| self.map_init(p))
            .or(|p| self.collection_elem(p))
            .or(|p| self.call(p).map(AtomExpression::Call))
            .or(|p| token!(self.token(p) => Token::Break => AtomExpression::Break))
            .or(|p| token!(self.token(p) => Token::Continue => AtomExpression::Continue))
            .or(with_sub)
            .into()
    }

    pub fn function(&self, pos: usize) -> ParseResult<'a, Function<'a>> {
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

    pub fn logic_atom(&self, pos: usize) -> ParseResult<'a, Logic<'a>> {
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

    pub fn compound_expr(&self, pos: usize) -> ParseResult<'a, CompoundExpression<'a>> {
        let tail = |p| {
            token!(self.token(p) => Token::Dot)
                .then(|p| self.call(p))
                .map(CompoundExpression::Tail)
        };

        let is = |p| {
            token!(self.token(p) => Token::Is)
                .then(|p| self.expression(p))
                .map(Box::new)
                .map(CompoundExpression::Is)
        };
        let logic = self.logic(pos).map(CompoundExpression::Logic);
        let arithmetic = |p| self.arith(p).map(CompoundExpression::Arith);
        let elvis = |p| self.elvis(p).map(CompoundExpression::Elvis);

        logic
            .or_from(pos)
            .or(arithmetic)
            .or(elvis)
            .or(tail)
            .or(is)
            .into()
    }

    pub fn logic(&self, pos: usize) -> ParseResult<'a, Logic<'a>> {
        let and = |p| {
            self.logic_atom(p)
                .then_multi_zip(|p| {
                    token!(self.token(p) => Token::And)
                        .then(|p| self.expression(p))
                        .then_zip(|p| self.logic_atom(p))
                        .map(|(e, l)| (e, Box::new(l)))
                })
                .map(|(l, tail)| {
                    if tail.is_empty() {
                        l
                    } else {
                        Logic::And(Box::new(l), tail)
                    }
                })
        };
        and(pos)
            .then_multi_zip(|p| {
                token!(self.token(p) => Token::Or)
                    .then(|p| self.expression(p))
                    .then_zip(and)
                    .map(|(e, l)| (e, Box::new(l)))
            })
            .map(|(l, tail)| {
                if tail.is_empty() {
                    l
                } else {
                    Logic::Or(Box::new(l), tail)
                }
            })
    }
    pub fn arith(&self, pos: usize) -> ParseResult<'a, Arithmetic<'a>> {
        let mul = |p| {
            token!(self.token(p) =>
                        Token::Mult => MulSign::Mul,
                        Token::Div => MulSign::Div,
                        Token::Mod => MulSign::Mod
            )
            .then_zip(|p| self.expression(p))
            .map(|(s, e)| Arithmetic::Mul(s, e))
        };
        let add = |p| {
            token!(self.token(p) =>
                        Token::Sub => false,
                        Token::Add => true
            )
            .then_zip(|p| mul(p).or_last(|p| self.expression(p).map(Arithmetic::Expression)))
            .map(|(s, e)| Arithmetic::Add(s, Box::new(e)))
        };
        let range = |p| {
            token!(self.token(p) =>
                        Token::EllipsisIn => false,
                        Token::EllipsisOut => true
            )
            .then_zip(|p| add(p).or_last(|p| self.expression(p).map(Arithmetic::Expression)))
            .map(|(s, e)| Arithmetic::Range(s, Box::new(e)))
        };
        let shift = |p| {
            token!(self.token(p) =>
                        Token::LShift => false,
                        Token::RShift => true
            )
            .then_zip(|p| range(p).or_last(|p| self.expression(p).map(Arithmetic::Expression)))
            .map(|(s, e)| Arithmetic::Shift(s, Box::new(e)))
        };
        let bit = |p| {
            token!(self.token(p) =>
                        Token::BitOr => BitSign::Or,
                        Token::BitAnd => BitSign::And,
                        Token::Caret => BitSign::Xor
            )
            .then_zip(|p| shift(p).or_last(|p| self.expression(p).map(Arithmetic::Expression)))
            .map(|(s, e)| Arithmetic::Bit(s, Box::new(e)))
        };

        mul(pos)
            .or_last(add)
            .or_last(range)
            .or_last(shift)
            .or_last(bit)
    }
    pub fn class_statement(&self, pos: usize) -> ParseResult<'a, ClassStatement<'a>> {
        let op_getter = |p| {
            token!(self.token(p) =>
                Token::Sub => GetterLabel::Sub,
                Token::Tilde => GetterLabel::Tilde,
                Token::Bang => GetterLabel::Bang)
            .or_last(|p| self.id(p).map(GetterLabel::Id))
            .then_or_none_zip(|p| self.block(p).or_none())
            .map(|(g, b)| ClassStatement::OpGetter(g, b))
        };
        let setter = |p| {
            self.id(p)
                .then_zip(|p| {
                    token!(self.token(p) => Token::Assign)
                        .then(|p| self.one_arg(p))
                        .then_zip(|p| self.block(p))
                })
                .map(|(l, (r, b))| ClassStatement::Setter(l, r, b))
        };
        let subscript_get = |p| {
            token!(self.token(p) => Token::LParen)
                .then(|p| self.enumeration(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .then_zip(|p| self.block(p))
                .map(|(e, b)| ClassStatement::SubscriptGet(e, b))
        };
        let subscript_set = |p| {
            token!(self.token(p) => Token::LParen)
                .then(|p| self.enumeration(p))
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .then_zip(|p| token!(self.token(p) => Token::Assign).then(|p| self.one_arg(p)))
                .then_zip(|p| self.block(p))
                .map(|((e, id), b)| ClassStatement::SubscriptSet(e, id, b))
        };
        let op_setter = |p| {
            token!(self.token(p) =>
                    Token::Sub => SetterLabel::Sub,
                    Token::Mult => SetterLabel::Mul,
                    Token::Div => SetterLabel::Div,
                    Token::Mod => SetterLabel::Mod,
                    Token::Add => SetterLabel::Add,
                    Token::EllipsisIn => SetterLabel::EllipsisIn,
                    Token::EllipsisOut => SetterLabel::EllipsisOut,
                    Token::LShift => SetterLabel::LShift,
                    Token::BitAnd => SetterLabel::BitAnd,
                    Token::Caret => SetterLabel::BitXor,
                    Token::BitOr => SetterLabel::BitOr,
                    Token::Gt => SetterLabel::Gt,
                    Token::Lt => SetterLabel::Lt,
                    Token::Equal => SetterLabel::Eq,
                    Token::Le => SetterLabel::Le,
                    Token::Ge => SetterLabel::Ge,
                    Token::NotEqual => SetterLabel::NotEq,
                    Token::Is => SetterLabel::Is)
            .then_zip(|p| self.one_arg(p))
            .then_zip(|p| self.block(p))
            .map(|((l, id), b)| ClassStatement::OpSetter(l, id, b))
        };
        let constructor = |p| {
            token!(self.token(p) => Token::Construct)
                .then(|p| self.id(p))
                .then_zip(|p| self.params(p))
                .then_zip(|p| self.block(p))
                .map(|((id, ps), b)| ClassStatement::Constructor(id, ps, b))
        };

        self.function(pos)
            .map(ClassStatement::Fn)
            .or_from(pos)
            .or(op_getter)
            .or(op_setter)
            .or(setter)
            .or(subscript_get)
            .or(subscript_set)
            .or(constructor)
            .into()
    }
    pub fn class_body(&self, pos: usize) -> ParseResult<'a, ClassUnit<'a>> {
        let foreign = |p| token!(self.token(p) => Token::Foreign => ClassBodyType::Foreign);
        let static_t = |p| token!(self.token(p) => Token::Static => ClassBodyType::Static);

        let tpe = |p| {
            foreign(p)
                .then(static_t)
                .map(|r| ClassBodyType::ForeignStatic)
                .or_from(p)
                .or(|p| {
                    static_t(p)
                        .then(foreign)
                        .map(|r| ClassBodyType::ForeignStatic)
                })
                .or(static_t)
                .or(foreign)
                .into()
        };

        self.zero_or_more(pos, |p| self.attribute(p))
            .then_or_default_zip(tpe)
            .then_zip(|p| self.class_statement(p))
            .map(|((attributes, tpe), statement)| ClassUnit {
                attributes,
                tpe,
                statement,
            })
    }

    pub fn attribute(&self, pos: usize) -> ParseResult<'a, Attribute<'a>> {
        let prefix = |p| {
            token!(self.token(p) => Token::Hash)
                .then_or_val(|p| token!(self.token(p) => Token::Bang => true), false)
        };

        let attr_val = |p| {
            self.id(p)
                .then_or_none_zip(|p| {
                    token!(self.token(p) => Token::Assign)
                        .then(|p| self.atom(p))
                        .or_none()
                })
                .map(|(id, expr)| AttributeValue { id, expr })
        };

        let simple = |p| {
            prefix(p)
                .then_zip(attr_val)
                .map(|(b, v)| Attribute::Simple(b, v))
        };

        let group = |p| {
            prefix(p)
                .then_zip(|p| self.id(p))
                .then_zip(|p| {
                    token!(self.token(p) => Token::LParen)
                        .then(attr_val)
                        .then_multi_zip(|p| token!(self.token(p) => Token::Comma).then(attr_val))
                        .merge()
                })
                .then_zip(|p| token!(self.token(p) => Token::RParen))
                .take_left()
                .map(|((b, id), attrs)| Attribute::Group(b, id, attrs))
        };

        group(pos).or_from(pos).or(simple).into()
    }

    pub fn one_arg(&self, pos: usize) -> ParseResult<'a, Id<'a>> {
        token!(self.token(pos) => Token::LParen)
            .then(|p| self.id(p))
            .then_zip(|p| token!(self.token(p) => Token::RParen))
            .take_left()
    }
    pub fn while_statement(&self, pos: usize) -> ParseResult<'a, While<'a>> {
        let cond = |p| {
            self.expression(p)
                .map(WhileCond::Expression)
                .or_from(p)
                .or(|p| self.assignment(p).map(WhileCond::Assignment))
                .into()
        };

        token!(self.token(pos) => Token::While)
            .then(|p| token!(self.token(p) => Token::LParen))
            .then(cond)
            .then_zip(|p| token!(self.token(p) => Token::RParen))
            .take_left()
            .then_zip(|p| self.statement(p))
            .map(|(cond, body)| While { cond, body })
    }
    pub fn for_statement(&self, pos: usize) -> ParseResult<'a, For<'a>> {
        token!(self.token(pos) => Token::For)
            .then(|p| token!(self.token(p) => Token::LParen))
            .then(|p| self.id(p))
            .then_zip(|p| token!(self.token(p) => Token::In))
            .take_left()
            .then_zip(|p| self.expression(p))
            .then_zip(|p| token!(self.token(p) => Token::RParen))
            .take_left()
            .then_zip(|p| self.statement(p))
            .map(|((elem, collection), body)| For {
                elem,
                collection,
                body,
            })
    }

    pub fn class_def(&self, pos: usize) -> ParseResult<'a, ClassDefinition<'a>> {
        let inherit = |p| token!(self.token(p) => Token::Is).then(|p| self.id(p));

        self.zero_or_more(pos, |p| self.attribute(p))
            .then_zip(|p| token!(self.token(p) => Token::Foreign => true).or_val(false))
            .then_zip(|p| token!(self.token(p) => Token::Class))
            .take_left()
            .then_zip(|p| self.id(p))
            .then_or_none_zip(|p| inherit(p).or_none())
            .then_zip(|p| token!(self.token(p) => Token::LBrace))
            .take_left()
            .then_zip(|p| self.zero_or_more(p, |p| self.class_body(p)))
            .map(|((((attrs, f), name), inherit), elems)| ClassDefinition {
                attributes: attrs,
                foreign: f,
                name,
                inherit,
                elems,
            })
    }
}

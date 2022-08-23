use std::collections::HashMap;
use std::iter::Map;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EmptyToken {}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
    Hex(i64),
    Binary(isize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtomExpression<'a> {
    Null,
    Bool(bool),
    CharLit(&'a str),
    StringLit(&'a str),
    Number(Number),
    MapInit(Vec<(Expression<'a>, Expression<'a>)>),
    ListInit(Enumeration<'a>),
    Break,
    Continue,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Params<'a> {
    pub ids: Vec<Id<'a>>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Id<'a> {
    pub value: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Elvis<'a> {
    pub lhs: Expression<'a>,
    pub rhs: Expression<'a>,

}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Enumeration<'a> {
    pub values: Vec<Expression<'a>>,
}

impl<'a> Enumeration<'a> {
    pub fn new(values: Vec<Expression<'a>>) -> Self {
        Self { values }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    Atom(AtomExpression<'a>),
    E,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block<'a> {
    pub params: Params<'a>,
    pub statements: Vec<Statement<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call<'a> {
    pub id: Id<'a>,
    pub tail: Option<Box<Call<'a>>>,
    pub middle: BlockOrEnum<'a>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum BlockOrEnum<'a> {
    Block(Block<'a>),
    Enum(Enumeration<'a>),
    None,
}

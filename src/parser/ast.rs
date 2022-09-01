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
    Call(Call<'a>),
    Range(Range<'a>),
    Break,
    Continue,
    CollectionElem(Call<'a>, Enumeration<'a>),
    ImportModule(ImportModule<'a>),
    Sub(Box<AtomExpression<'a>>),
}

impl<'a> AtomExpression<'a> {
    pub fn string_or_default(&self, default: &'a str) -> &'a str {
        match self {
            AtomExpression::StringLit(v) | AtomExpression::CharLit(v) => v,
            _ => default,
        }
    }
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
    Compound(Box<Expression<'a>>,Box<CompoundExpression<'a>>),
    Not(Box<Expression<'a>>),
    E
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompoundExpression<'a>{
    Logic(Logic<'a>),
    Arith(Arithmetic<'a>),
    Tail(Call<'a>),
    Is(Box<Expression<'a>>),
    Elvis(Elvis<'a>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
    Assignment(Assignment<'a>),
    AssignmentNull(AssignmentNull<'a>),
    If(Box<If<'a>>),
    While(Box<While<'a>>),
    For(Box<For<'a>>),
    Block(Block<'a>),
    Return(Expression<'a>),
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

impl<'a> Call<'a> {
    pub fn just_id(id: &'a str) -> Call<'a> {
        Call {
            id: Id { value: id },
            tail: None,
            middle: BlockOrEnum::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockOrEnum<'a> {
    Block(Block<'a>),
    Enum(Enumeration<'a>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportVariable<'a> {
    pub name: Id<'a>,
    pub alias: Option<Id<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportModule<'a> {
    pub name: &'a str,
    pub variables: Vec<ImportVariable<'a>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Function<'a> {
    pub name: Id<'a>,
    pub params: Params<'a>,
    pub block: Option<Block<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RangeExpression<'a> {
    Call(Call<'a>),
    Num(Number),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Range<'a> {
    pub left: RangeExpression<'a>,
    pub right: RangeExpression<'a>,
    pub is_out: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicOp {
    Gt,
    Lt,
    Eq,
    Le,
    Ge,
    NotEq,
    Or,
    And,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtomLogic<'a> {
    pub op: LogicOp,
    pub value: Expression<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Logic<'a> {
    Atom(LogicOp, Expression<'a>),
    And(Box<Logic<'a>>, Vec<(Expression<'a>, Box<Logic<'a>>)>),
    Or(Box<Logic<'a>>, Vec<(Expression<'a>, Box<Logic<'a>>)>),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Arithmetic<'a> {
    Expression(Expression<'a>),
    Mul(MulSign, Expression<'a>),
    Add(bool, Box<Arithmetic<'a>>),
    Range(bool, Box<Arithmetic<'a>>),
    Shift(bool, Box<Arithmetic<'a>>),
    Bit(BitSign, Box<Arithmetic<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MulSign {
    Mul,
    Div,
    Mod,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BitSign {
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassStatement<'a> {
    Fn(Function<'a>),
    OpGetter(GetterLabel<'a>, Option<Block<'a>>),
    Setter(Id<'a>, Id<'a>, Block<'a>),
    OpSetter(SetterLabel, Id<'a>, Block<'a>),
    SubscriptGet(Enumeration<'a>, Block<'a>),
    SubscriptSet(Enumeration<'a>, Id<'a>, Block<'a>),
    Constructor(Id<'a>, Params<'a>, Block<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GetterLabel<'a> {
    Id(Id<'a>),
    Sub,
    Tilde,
    Bang,
}
#[derive(Debug, Clone, PartialEq)]
pub enum SetterLabel {
    Sub,
    Mul,
    Div,
    Mod,
    Add,
    EllipsisIn,
    EllipsisOut,
    LShift,
    RShift,
    BitAnd,
    BitOr,
    BitXor,
    Gt,
    Lt,
    Eq,
    Le,
    Ge,
    NotEq,
    Is,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Attribute<'a> {
    Simple(bool, AttributeValue<'a>),
    Group(bool, Id<'a>, Vec<AttributeValue<'a>>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeValue<'a> {
    pub id: Id<'a>,
    pub expr: Option<AtomExpression<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClassBodyType {
    Foreign,
    Static,
    ForeignStatic,
    None,
}
impl Default for ClassBodyType{
    fn default() -> Self {
        ClassBodyType::None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassUnit<'a> {
    pub attributes: Vec<Attribute<'a>>,
    pub tpe: ClassBodyType,
    pub statement: ClassStatement<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDefinition<'a> {
    pub attributes: Vec<Attribute<'a>>,
    pub foreign: bool,
    pub name: Id<'a>,
    pub inherit: Option<Id<'a>>,
    pub elems: Vec<ClassUnit<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Mod,
    LShift,
    RShift,
    URShift,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment<'a> {
    pub var: bool,
    pub op: AssignOp,
    pub lhs: Expression<'a>,
    pub rhs: Box<Rhs<'a>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentNull<'a> {
    pub id: Id<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rhs<'a> {
    Expression(Expression<'a>),
    Assignment(Assignment<'a>),
    Assignments(Vec<Assignment<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfBranch<'a> {
    pub cond: Expression<'a>,
    pub action: Statement<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If<'a> {
    pub main: IfBranch<'a>,
    pub others: Vec<IfBranch<'a>>,
    pub els: Option<Statement<'a>>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum WhileCond<'a> {
    Expression(Expression<'a>),
    Assignment(Assignment<'a>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct While<'a> {
    pub cond: WhileCond<'a>,
    pub body: Statement<'a>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct For<'a> {
    pub elem: Id<'a>,
    pub collection: Expression<'a>,
    pub body: Statement<'a>,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Unit<'a> {
    Class(ClassDefinition<'a>),
    Fn(Function<'a>),
    Import(ImportModule<'a>),
    Statement(Statement<'a>),
    Block(Block<'a>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Script<'a> {
    pub units: Vec<Unit<'a>>,
}

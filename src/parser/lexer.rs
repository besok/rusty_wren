use std::ops::Range;

use crate::parser::ast::Number;
use crate::parser::ParseError;
use logos::Lexer;
use logos::Logos;
use std::error::Error;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct CypherLexer<'a> {
    pub(crate) source: &'a str,
    pub(crate) tokens: Vec<Token<'a>>,
}

impl<'a> CypherLexer<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut delegate = Token::lexer(source);
        let mut tokens = vec![];

        while let Some(t) = delegate.next() {
            match t {
                Token::Error => {
                    return Err(ParseError::BadToken(delegate.slice(), delegate.span()));
                }
                t => tokens.push(t),
            }
        }

        Ok(CypherLexer { source, tokens })
    }
    pub fn token(&self, pos: usize) -> Result<(&Token<'a>, usize), ParseError<'a>> {
        match self.tokens.get(pos) {
            None => Err(ParseError::ReachedEOF(pos)),
            Some(t) => Ok((t, pos)),
        }
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}

#[derive(Logos, Debug, Copy, Clone, PartialEq)]
#[logos(subpattern digit = r"[0-9]([0-9_]*[0-9])?")]
#[logos(subpattern letter = r"[a-zA-Z_]")]
#[logos(subpattern exp = r"[eE][+-]?[0-9]+")]
pub enum Token<'a> {
    #[regex(r"(?i)(?&letter)((?&letter)|(?&digit))*")]
    Id(&'a str),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    StringLit(&'a str),
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\')*'"#)]
    CharLit(&'a str),
    #[regex(r#""""([^"\\]|\\t|\\u|\\n|\\")*""""#)]
    TextBlock(&'a str),

    #[regex(r"-?(?&digit)", number)]
    #[regex(r"-?(?&digit)(?&exp)", number)]
    #[regex(r"-?(?&digit)?\.(?&digit)(?&exp)?[fFdD]?", float)]
    #[regex(r"0[bB][01][01]*", binary)]
    #[regex(r"-?0x[0-9a-f](([0-9a-f]|[_])*[0-9a-f])?", hex)]
    Digit(Number),

    #[token("as")]
    As,
    #[token("break")]
    Break,
    #[token("class")]
    Class,
    #[token("construct")]
    Construct,
    #[token("continue")]
    Continue,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("true")]
    True,
    #[token("for")]
    For,
    #[token("foreign")]
    Foreign,
    #[token("if")]
    If,
    #[token("import")]
    Import,
    #[token("in")]
    In,
    #[token("is")]
    Is,
    #[token("null")]
    Null,
    #[token("return")]
    Return,
    #[token("static")]
    Static,
    #[token("var")]
    Var,
    #[token("while")]
    While,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBrack,
    #[token("]")]
    RBrack,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("++")]
    Inc,
    #[token("--")]
    Dec,
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mult,
    #[token("/")]
    Div,
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("!")]
    Bang,
    #[token("?")]
    Question,
    #[token("#")]
    Hash,
    #[token(">")]
    Gt,
    #[token(">=")]
    Ge,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token("~")]
    Tilde,
    #[token("^")]
    Caret,
    #[token("=")]
    Assign,
    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MultAssign,
    #[token("&=")]
    AndAssign,
    #[token("|=")]
    OrAssign,
    #[token("^=")]
    XOrAssign,
    #[token("%=")]
    ModAssign,
    #[token("/=")]
    DivAssign,
    #[token("%")]
    Mod,
    #[token("..")]
    EllipsisIn,
    #[token("...")]
    EllipsisOut,
    #[token(">>")]
    RShift,
    #[token("<<")]
    LShift,
    #[token(">>=")]
    RShiftAssign,
    #[token("<<=")]
    LShiftAssign,
    #[token(">>>=")]
    URShiftAssign,

    #[regex(r"(?s)/\*.*\*/", logos::skip)]
    #[regex(r"//[^\r\n]*", logos::skip)]
    Comment,

    #[regex(r"[ \t\r\n\u000C\f]+", logos::skip)]
    Whitespace,

    #[error]
    Error,
}

fn number<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    lex.slice()
        .parse::<i64>()
        .map(|r| Number::Int(r))
        .map_err(|s| s.to_string())
}

fn float<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    lex.slice()
        .parse::<f64>()
        .map(|r| Number::Float(r))
        .map_err(|s| s.to_string())
}

fn binary<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    isize::from_str_radix(&lex.slice()[2..], 2)
        .map(Number::Binary)
        .map_err(|s| s.to_string())
}

fn hex<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    i64::from_str_radix(lex.slice().trim_start_matches("0x"), 16)
        .map(|r| Number::Hex(r))
        .map_err(|s| s.to_string())
}


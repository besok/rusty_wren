use std::ops::Range;

use std::error::Error;
use std::num::ParseIntError;
use logos::Lexer;
use logos::Logos;
use crate::parser::ast::Number;
use crate::parser::ParseError;

#[derive(Debug)]
pub struct CypherLexer<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
}

impl<'a> CypherLexer<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut delegate = Token::lexer(source);
        let mut tokens = vec!();

        while let Some(t) = delegate.next() {
            match t {
                Token::Error => {
                    return Err(ParseError::BadToken(delegate.slice(), delegate.span()));
                }
                t => tokens.push(t)
            }
        }

        Ok(CypherLexer { source, tokens })
    }
    pub fn token(&self, pos: usize) -> Result<(&Token<'a>, usize), ParseError<'a>> {
        match self.tokens.get(pos) {
            None => Err(ParseError::ReachedEOF(pos)),
            Some(t) => Ok((t, pos))
        }
    }
    pub fn len(&self) -> usize { self.tokens.len() }
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
    #[regex(r"-?(?&digit)(?&exp)", float)]
    #[regex(r"-?(?&digit)?\.(?&digit)(?&exp)?[fFdD]?", float)]
    #[regex(r"0[bB][01][01]*", binary)]
    #[regex(r"-?0x[0-9a-f](([0-9a-f]|[_])*[0-9a-f])?", hex)]
    Digit(Number),

    #[token("AS")]
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
    lex.slice().parse::<i64>().map(|r| Number::Int(r)).map_err(|s| s.to_string())
}

fn float<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    lex.slice().parse::<f64>().map(|r| Number::Float(r)).map_err(|s| s.to_string())
}

fn binary<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    println!("{}", lex.slice());
    isize::from_str_radix(&lex.slice()[2..], 2)
        .map(Number::Binary)
        .map_err(|s| s.to_string())
}

fn hex<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
    i64::from_str_radix(lex.slice().trim_start_matches("0x"), 16)
        .map(|r| Number::Hex(r))
        .map_err(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use crate::parser::ast::Number::{Binary, Float, Hex, Int};
    use crate::parser::lexer::{CypherLexer, Token};
    use crate::parser::lexer::Token::*;

    fn test_match(src: &str, tokens: Vec<Token>) {
        match CypherLexer::new(src) {
            Ok(lexer) => assert_eq!(lexer.tokens, tokens),
            Err(error) => panic!("{:?}", error)
        }
    }

    fn test_success(src: &str) {
        match CypherLexer::new(src) {
            Ok(lexer) => println!("{:?}", lexer.tokens),
            Err(error) => panic!("{:?}", error)
        }
    }

    fn test_failed(src: &str) {
        match CypherLexer::new(src) {
            Ok(lexer) => panic!("{:?}", lexer.tokens),
            Err(error) => println!("{:?}", error)
        }
    }


    #[test]
    fn number_test() {
        test_match("-1", vec![Digit(Int(-1))]);
        test_match("1.1e1", vec![Digit(Float(11.0))]);
        test_match("1e1", vec![Digit(Float(10.0))]);
        test_match("1 01 01 1e1 1e-1 0x1 1.1 1.0e1", vec![
            Digit(Int(1)),
            Digit(Int(1)),
            Digit(Int(1)),
            Digit(Float(10.0)),
            Digit(Float(0.1)),
            Digit(Hex(1)),
            Digit(Float(1.1)),
            Digit(Float(10.0)),
        ]);
        test_match("0b1101", vec![Digit(Binary(13))]);
    }

    #[test]
    fn words_test() {
        test_match("abc b ~ bca id_9_0", vec![Id("abc"), Id("b"), Tilde, Id("bca"), Id("id_9_0")]);
        test_match("\"text\"", vec![StringLit("\"text\"")]);
        test_match(r#"
            """ some text
            but in as a block
            """
        "#, vec![TextBlock(r#"""" some text
            but in as a block
            """"#)]);
    }

    #[test]
    fn common_test() {
        test_success(r#"
        // Ported from the Python version.

class Tree {
  construct new(item, depth) {
    _item = item
    if (depth > 0) {
      var item2 = item + item
      depth = depth - 1
      _left = Tree.new(item2 - 1, depth)
      _right = Tree.new(item2, depth)
    }
  }

  check {
    if (_left == null) {
      return _item
    }

    return _item + _left.check - _right.check
  }
}

var minDepth = 4
var maxDepth = 12
var stretchDepth = maxDepth + 1

var start = System.clock

System.print("stretch tree of depth %(stretchDepth) check: " +
    "%(Tree.new(0, stretchDepth).check)")
for (i in 1...1000) System.gc()

var longLivedTree = Tree.new(0, maxDepth)

// iterations = 2 ** maxDepth
var iterations = 1
for (d in 0...maxDepth) {
  iterations = iterations * 2
}

var depth = minDepth
while (depth < stretchDepth) {
  var check = 0
  for (i in 1..iterations) {
    check = check + Tree.new(i, depth).check + Tree.new(-i, depth).check
  }

  System.print("%(iterations * 2) trees of depth %(depth) check: %(check)")
  for (i in 1...1000) System.gc()

  iterations = iterations / 4
  depth = depth + 2
}

System.print(
    "long lived tree of depth %(maxDepth) check: %(longLivedTree.check)")
for (i in 1...1000) System.gc()

System.print("elapsed: %(System.clock - start)")

        "#)
    }
}

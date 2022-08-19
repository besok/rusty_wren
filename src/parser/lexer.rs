use std::ops::Range;

use std::error::Error;
use std::num::ParseIntError;
use logos::Lexer;
use logos::Logos;
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

#[derive(Logos, Debug, Clone, Copy, PartialEq)]
#[logos(subpattern digits = r"[1-9]([0-9_]*[0-9])?")]
pub enum Token<'a> {
    #[regex(r"(?i)[a-z_][a-z_0-9]*")]
    Id(&'a str),

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    StringLit(&'a str),
    #[regex(r#"'([^'\\]|\\t|\\u|\\n|\\')*'"#)]
    CharLit(&'a str),
    #[regex(r#"`([^`\\]|\\t|\\u|\\n|\\`)*`"#)]
    EscLit(&'a str),

    // #[regex(r"-?0?(?&digits)", number)]
    // #[regex(r"-?0x[0-9a-f](([0-9a-f]|[_])*[0-9a-f])?", hex)]
    // #[regex(r"-?(?&digits)?\.(?&digits)([e][+-]?(?&digits))?[fd]?", float)]
    // #[regex(r"-?(?&digits)([e][+-]?(?&digits)[fd]?|[fd])", number)]
    // Digit(Number),

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
    LPAREN,
    #[token(")")]
    RPAREN,
    #[token("{")]
    LBRACE,
    #[token("}")]
    RBRACE,
    #[token("[")]
    LBRACK,
    #[token("]")]
    RBRACK,
    #[token(";")]
    SEMI,
    #[token(",")]
    COMMA,
    #[token(".")]
    DOT,

    #[regex(r"(?s)/\*.*\*/", logos::skip)]
    #[regex(r"//[^\r\n]*", logos::skip)]
    Comment,

    #[regex(r"[ \t\r\n\u000C\f]+", logos::skip)]
    Whitespace,

    #[error]
    Error,
}

// fn number<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
//     lex.slice().parse::<i64>().map(|r| Number::Int(r)).map_err(|s| s.to_string())
// }
//
// fn float<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
//     lex.slice().parse::<f64>().map(|r| Number::Float(r)).map_err(|s| s.to_string())
// }
//
// fn hex<'a>(lex: &mut Lexer<'a, Token<'a>>) -> Result<Number, String> {
//     i64::from_str_radix(lex.slice().trim_start_matches("0x"), 16)
//         .map(|r| Number::Hex(r))
//         .map_err(|s| s.to_string())
// }

#[cfg(test)]
mod tests {
    use crate::parser::lexer::Number::{Float, Int};
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
    fn words_test() {
        test_match("filter not any Call", vec![FILTER, NOT, ANY, CALL]);
        test_match("BY by", vec![BY, BY]);
    }
}

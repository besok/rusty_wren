use crate::parser::ast::Number::{Binary, Float, Hex, Int};
use crate::parser::lexer::Token::*;
use crate::parser::lexer::{CypherLexer, Token};

fn expect(src: &str, tokens: Vec<Token>) {
    match CypherLexer::new(src) {
        Ok(lexer) => assert_eq!(lexer.tokens, tokens),
        Err(error) => panic!("{:?}", error),
    }
}

fn expect_succeed(src: &str) {
    match CypherLexer::new(src) {
        Ok(lexer) => println!("{:?}", lexer.tokens),
        Err(error) => panic!("{:?}", error),
    }
}

fn expect_failed(src: &str) {
    match CypherLexer::new(src) {
        Ok(lexer) => panic!("{:?}", lexer.tokens),
        Err(error) => println!("{:?}", error),
    }
}

#[test]
fn number_test() {
    expect("1e1", vec![Digit(Float(10.0))]);
    expect("-1", vec![Digit(Int(-1))]);
    expect("1.1e1", vec![Digit(Float(11.0))]);
    expect(
        "1 01 01 1e1 1e-1 0x1 1.1 1.0e1",
        vec![
            Digit(Int(1)),
            Digit(Int(1)),
            Digit(Int(1)),
            Digit(Float(10.0)),
            Digit(Float(0.1)),
            Digit(Hex(1)),
            Digit(Float(1.1)),
            Digit(Float(10.0)),
        ],
    );
    expect("0b1101", vec![Digit(Binary(13))]);
}

#[test]
fn words_test() {
    expect(
        "abc b ~ bca id_9_0",
        vec![Id("abc"), Id("b"), Tilde, Id("bca"), Id("id_9_0")],
    );
    expect("\"text\"", vec![StringLit("\"text\"")]);
    expect(
        r#"
            """ some text
            but in as a block
            """
        "#,
        vec![TextBlock(
            r#"""" some text
            but in as a block
            """"#,
        )],
    );
}

#[test]
fn common_test() {
    expect_succeed(include_str!("parser/test_scripts/binary_tree.wren"))
}

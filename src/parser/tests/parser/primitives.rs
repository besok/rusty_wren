use crate::parser::ast::AtomExpression;
use crate::parser::tests::parser::{expect, expect_pos, fail, parser};

#[test]
fn enum_test() {
    expect_pos(parser("1").enumeration(0), 1);
    expect_pos(parser("a, b").enumeration(0), 3);
    expect_pos(parser("a, [1,2]").enumeration(0), 7);
}

#[test]
fn null_test() {
    expect(parser("null").null(0), AtomExpression::Null);
    fail(parser("not_null").null(0));
}

#[test]
fn elvis_test(){
    expect_pos(parser("? a : b").elvis(0), 4);
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
fn map_init_test() {
    expect_pos(parser("{}").map_init(0), 2);
    expect_pos(parser("{[1] : [2]}").map_init(0), 9);
    expect_pos(parser("{a : null, b : null}").map_init(0), 9);
}

#[test]
fn list_init_test() {
    expect_pos(parser("[]").list_init(0), 2);
    expect_pos(parser("[1]").list_init(0), 3);
    expect_pos(parser("[1 + 2 , b - a]").list_init(0), 9);
}

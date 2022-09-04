use crate::parser::tests::parser::{expect_pos, parser};


#[test]
fn if_test() {
    expect_pos(parser("if(a > b || b > c && !z) a ").if_statement(0), 14);
    expect_pos(parser("if(a > b || b > c && !z) a else b ").if_statement(0), 16);
    expect_pos(
        parser("if(true) a = b else if(a > b) 1 else if(!c) a").if_statement(0),
        22,
    );
    expect_pos(
        parser("if(1) 0 else if(2) a else if(3) [] else a").if_statement(0),
        20,
    );
    expect_pos(parser("if(a > b || b > c && !z) {a} else { c }").if_statement(0), 20);
}
#[test]
fn assignment_test() {
    expect_pos(parser("a = b").assignment(0), 3);
    expect_pos(parser("var 1 = 2").assignment(0), 4);
    expect_pos(parser("var x = var y = 1").assignment(0), 7);
}
#[test]
fn assignment_null_test() {
    expect_pos(parser("var a").assignment_null(0), 2);
}
#[test]
fn attrs_test() {
    expect_pos(parser("# id").attribute(0), 2);
    expect_pos(parser("# id = 1").attribute(0), 4);
    expect_pos(parser("#!id = 1").attribute(0), 5);
    expect_pos(parser("# !id").attribute(0), 3);
    expect_pos(parser("#id(x = y)").attribute(0), 7);
    expect_pos(parser("#!id(x = y)").attribute(0), 8);
    expect_pos(parser("#!id(x = y, z = f)").attribute(0), 12);
}
#[test]
fn statement_test() {
    expect_pos(parser("while(var x = a) var x = b").statement(0), 11);
    expect_pos(parser("a + 2").statement(0), 3);
    expect_pos(parser("var a = 1").statement(0), 4);
    expect_pos(parser("var a").statement(0), 2);
    expect_pos(parser("if(a > b || b > c && !z) a else c").statement(0), 16);
    expect_pos(parser("if(a > b || b > c && !z) {a} else {c}").statement(0), 20);
    expect_pos(parser("for(x in [1,2,3]) println(a)").statement(0), 16);
    expect_pos(parser("return x").statement(0), 2);
}
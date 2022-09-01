use crate::parser::tests::parser::{expect_pos, parser};

#[test]
fn arith_test() {
    expect_pos(parser("* 1").arith(0), 2);
    expect_pos(parser("/ 2").arith(0), 2);
    expect_pos(parser("+ 3").arith(0), 2);
    expect_pos(parser(".. 4").arith(0), 2);
    expect_pos(parser("1 2").arith(0), 2);
    // expect_pos(simple-parser("| >>").arith(0), 2);
    // expect_pos(simple-parser("- * >>").arith(0), 3);
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
fn atom_logic_test() {
    expect_pos(parser("|| >>").logic_atom(0), 2);
    expect_pos(parser("&& >>").logic_atom(0), 2);
    expect_pos(parser("< >>").logic_atom(0), 2);
    expect_pos(parser("== >>").logic_atom(0), 2);
    expect_pos(parser("!= >>").logic_atom(0), 2);
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
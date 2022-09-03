use crate::parser::tests::parser::{expect_pos, parser};


#[test]
fn expression(){
    expect_pos(parser("!x").expression(0), 2);
    expect_pos(parser("!(1 + 2 * x)").expression(0), 8);
    expect_pos(parser("(1 + 2 * x) + id").expression(0), 9);
    expect_pos(parser("id + 1 || true && false").expression(0), 7);

}

#[test]
fn arith_test() {
    expect_pos(parser("* 1").arith(0), 2);
    expect_pos(parser("/ 2").arith(0), 2);
    expect_pos(parser("+ 3").arith(0), 2);
    expect_pos(parser(".. 4").arith(0), 2);
    expect_pos(parser("* 2").arith(0), 2);
    expect_pos(parser("| 1").arith(0), 2);
    expect_pos(parser("- -id").arith(0), 3);
    expect_pos(parser("- -id").arith(0), 3);
    expect_pos(parser("+ 1 * 2 - (x / 5)").arith(0), 10);
}
#[test]
fn logic_test() {
    expect_pos(parser("> abc ").logic(0), 2);
    expect_pos(parser("> cde && 1 > true").logic(0), 6);
    expect_pos(parser("> [1] || {a:b} > null && id.id.id > -x").logic(0), 21);
    expect_pos(parser("|| true && x && null").logic(0), 6);
    expect_pos(parser("> 1 || [] && id").logic(0), 7);
}


#[test]
fn atom_logic_test() {
    expect_pos(parser("|| true").logic_atom(0), 2);
    expect_pos(parser("&& false").logic_atom(0), 2);
    expect_pos(parser("&& \"abc\"").logic_atom(0), 2);
    expect_pos(parser("< 1..2").logic_atom(0), 4);
    expect_pos(parser("== -id[1]").logic_atom(0), 6);
    expect_pos(parser("!= null").logic_atom(0), 2);
}

#[test]
fn atom_test(){
    expect_pos(parser("true").atom(0), 1);
    expect_pos(parser("false").atom(0), 1);
    expect_pos(parser("'false'").atom(0), 1);
    expect_pos(parser("\"false\"").atom(0), 1);
    expect_pos(parser("1").atom(0), 1);
    expect_pos(parser("null").atom(0), 1);
    expect_pos(parser("[1,2]").atom(0), 5);
    expect_pos(parser("{x:x,y:1}").atom(0), 9);
    expect_pos(parser("id.id.id").atom(0), 5);
    expect_pos(parser("1..2").atom(0), 3);
    expect_pos(parser("id[1]").atom(0), 4);
    expect_pos(parser("break").atom(0), 1);
    expect_pos(parser("continue").atom(0), 1);
    expect_pos(parser("import \"x\" for x as y").atom(0), 6);
    expect_pos(parser("-id[1]").atom(0), 5);
}

#[test]
fn call_test() {
    expect_pos(parser("id.id.id").call(0), 5);
    expect_pos(parser("id").call(0), 1);
    expect_pos(parser("id()").call(0), 3);
    expect_pos(parser("id().id").call(0), 5);
    expect_pos(parser("id(a && b).id").call(0), 8);
    expect_pos(parser("id(1,2).id").call(0), 8);
    expect_pos(parser("id{}.id").call(0), 5);
    expect_pos(parser("id{ !x }.id").call(0), 7);
    expect_pos(parser("id{|a| a + 1 }.id").call(0), 11);
    expect_pos(parser("id{|a,b| [a,b] }.id").call(0), 15);
    expect_pos(parser("id{|a,b| {a:b} }.id().id").call(0), 19);
}
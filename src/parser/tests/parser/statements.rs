use crate::parser::tests::parser::{expect_pos, parser};

#[test]
fn class_unit_test() {
    expect_pos(
        parser("#id #x (y = true) static foreign x()").class_body(0),
        14,
    );
}
#[test]
fn if_test() {
    expect_pos(parser("if(>>) >> ").if_statement(0), 5);
    expect_pos(
        parser("if(>>) >> else if(>>) >> else if(>>) >>").if_statement(0),
        17,
    );
    expect_pos(
        parser("if(>>) >> else if(>>) >> else if(>>) >> else >>").if_statement(0),
        19,
    );
}
#[test]
fn assignment_test() {
    expect_pos(parser(">> = >>").assignment(0), 3);
    expect_pos(parser("var >> = >>").assignment(0), 4);
    expect_pos(parser("var >> = var >> = >>").assignment(0), 7);
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
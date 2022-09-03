use crate::parser::tests::parser::{expect_pos, fail_on, parser};

#[test]
fn import_mod_test() {
    expect_pos(parser("a as b").import_variable(0), 3);
    expect_pos(parser("import \"abc\" ").import_module(0), 2);
    expect_pos(
        parser("import \"abc\" for a as b, b as d").import_module(0),
        10,
    );
}


#[test]
fn block_test() {
    expect_pos(parser("{}").block(0), 2);
    expect_pos(parser("{>> >>}").block(0), 4);
    expect_pos(parser("{|a| >> >>}").block(0), 7);
    fail_on(parser("{|| >> >>}").block(0), 1);
}

#[test]
fn class_unit_test() {
    expect_pos(
        parser("#id #x (y = true) static foreign x()").class_body(0),
        14,
    );
}
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
fn class_statement_test() {

    expect_pos(
        parser(r#"
       construct new(item, depth) {
            _item = item
             if (depth > 0) {
                var item2 = item + item
                depth = depth - 1
                _left = Tree.new(item2 - 1, depth)
                _right = Tree.new(item2, depth)
                 }
             }
        "#).class_statement(0),
        53,
    );
}

#[test]
fn class_body_test() {
    expect_pos(
        parser("#id #x (y = true) static foreign x()").class_body(0),
        14,
    );
    expect_pos(
        parser(r#"
        check {
            if (_left == null) { return _item }
            return _item + _left.check - _right.check
            }
        "#).class_body(0),
        23,
    );
    expect_pos(
        parser(r#"
       construct new(item, depth) {
            _item = item
             if (depth > 0) {
                var item2 = item + item
                depth = depth - 1
                _left = Tree.new(item2 - 1, depth)
                _right = Tree.new(item2, depth)
                 }
             }
        "#).class_body(0),
        81,
    );
}
#[test]
fn class_unit_test() {
    expect_pos(
        parser(r#"
        foreign class Tree {
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
            if (_left == null) { return _item }

            return _item + _left.check - _right.check
            }
        }
        "#).class_def(0),
        81,
    );
}


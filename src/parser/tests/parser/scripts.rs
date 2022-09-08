use crate::parser::tests::parser::{expect_pos, parser};

#[test]
fn script(){
    let script: &str = include_str!("test_scripts/binary_tree.wren");
    expect_pos(parser(script).script(0).debug(),219)
}
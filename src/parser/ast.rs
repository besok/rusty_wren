use std::collections::HashMap;
use std::iter::Map;

#[derive(Debug,Copy,Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
    Hex(i64),
    Binary(isize)
}
mod primitives;
mod expressions;
mod units;
mod statements;
mod scripts;

use crate::parser::parser::CypherParser;
use crate::parser::result::ParseResult;
use std::fmt::Debug;

fn parser(src: &str) -> CypherParser {
    match CypherParser::new(src) {
        Ok(p) => p,
        Err(e) => panic!("{:?}", e),
    }
}

fn expect<T>(res: ParseResult<T>, expect: T)
where
    T: PartialEq + Debug,
{
    match res {
        ParseResult::Success(v, _) =>assert_eq!(v, expect),
        ParseResult::Fail(pos) => panic!("failed on {}", pos),
        ParseResult::Error(e) => panic!("error: {:?}", e),
    }
}

fn expect_pos<T>(res: ParseResult<T>, expect: usize)
where
    T: PartialEq,
{
    match res {
        ParseResult::Success(v, pos) => assert_eq!(pos, expect),
        ParseResult::Fail(pos) => panic!("failed on {}", pos),
        ParseResult::Error(e) => panic!("error: {:?}", e),
    }
}

fn fail<T: Debug>(res: ParseResult<T>) {
    match res {
        ParseResult::Success(v, pos) => {
            panic!(" expect to get a fail but got {:?} on pos {}", v, pos)
        }
        ParseResult::Fail(_) => {}
        ParseResult::Error(e) => panic!("error: {:?}", e),
    }
}

fn fail_on<T: Debug>(res: ParseResult<T>, expect: usize) {
    match res {
        ParseResult::Success(v, pos) => {
            panic!(" expect to get a fail but got {:?} on pos {pos}", v)
        }
        ParseResult::Fail(pos) => {
            assert_eq!(pos, expect, "actual:{:?}, expect:{:?}", pos, expect)
        }
        ParseResult::Error(e) => panic!("error: {:?}", e),
    }
}

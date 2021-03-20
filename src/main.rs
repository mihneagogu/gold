#![feature(int_error_matching)]
mod parsing;
mod tests;

#[allow(unused_imports)]
use parsing::literals::{parse_number, option_parse};
use parsing::Parser;




fn main() {
    let mut p = Parser::from_str("    \nasd");
    p.eat_ws();
    println!("idx: {}, cursor: {} row: {}, col: {}", p.index, p.cursor, p.row, p.col);
    println!("next char: {:?}", p.peek_char());

}

#![feature(int_error_matching)]
mod parsing;
mod tests;

use parsing::literals::{parse_number, option_parse};




fn main() {
    println!("{:?}", parse_number("asd"));
    let bigger = i32::MAX as i64 + 1;
    let smaller = i32::MIN as i64 - 1;
    let res = option_parse(&parse_number, (bigger - 2).to_string().as_str());
    
    println!("{:?} {:?}", parse_number(bigger.to_string().as_str()), parse_number(smaller.to_string().as_str()));
}

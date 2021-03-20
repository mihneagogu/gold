#![feature(int_error_matching)]
#![feature(core_intrinsics)]
use std::num::IntErrorKind;

#[derive(Debug)]
enum NumberParseErr {
    InvalidNumber(ParseNumData),
    PosOverflow(ParseNumData),
    NegOverflow(ParseNumData),
}

#[derive(Debug)]
struct ParseNumData(i32, i32, String);


fn parse_number(inp: &str) -> Result<i32, NumberParseErr> {
    use NumberParseErr::InvalidNumber;

    match inp.parse::<i32>() {
        Ok(n) => Ok(n),
        Err(e) => match e.kind() {
            IntErrorKind::InvalidDigit => Err(InvalidNumber(ParseNumData(0, 0, inp.to_string()))),
            IntErrorKind::PosOverflow => Err(NumberParseErr::PosOverflow(ParseNumData(0, 0, inp.to_string()))),
            IntErrorKind::NegOverflow => Err(NumberParseErr::NegOverflow(ParseNumData(0, 0, inp.to_string()))),
            _ => unreachable!()
        }
    }
}

fn main() {
    println!("i32 max {:?} and min {:?}", parse_number(i32::MAX.to_string().as_str()), parse_number(i32::MIN.to_string().as_str()));
    println!("{:?}", parse_number("asd"));
    let bigger = i32::MAX as i64 + 1;
    let smaller = i32::MIN as i64 - 1;
    println!("{:?} {:?}", parse_number(bigger.to_string().as_str()), parse_number(smaller.to_string().as_str()));
}

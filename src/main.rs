#![feature(assoc_char_funcs)]
#![feature(int_error_matching)]
mod parsing;
mod tests;
mod ast;

#[allow(unused_imports)]
use parsing::combinators::*;
use parsing::{Parser, ParsingBaggage, ParsingContext, combinators::CharParser};
use parsing::literals::{IdentParser, NumberParser};
use parsing::types::{Type, PrimitiveType};
use ast::types;



#[allow(dead_code)]
const NO_ARGS_EXIT: i32 = 2;
#[allow(dead_code)]
const FILE_NOT_FOUND_EXIT: i32 = 3;
#[allow(dead_code)]
const EXECUTABLE_AND_MORE: usize = 2;

const NUMBER_PARSER: NumberParser = NumberParser{};

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < EXECUTABLE_AND_MORE {
        // println!("I cannot dig any gold if you don't give me any arguments!");
        // process::exit(NO_ARGS_EXIT);
    // }
    // let file = &args[1];

    // @MAYBE(mike): Move this in another interface, so that we can deal with this gracefully 
    // when we have more files to compile
    // let file = match fs::read_to_string(file) {
        // Ok(contents) => contents,
        // Err(ioe) => {
          //   println!("ERROR {} found while opening file {}", ioe, file);
            // process::exit(FILE_NOT_FOUND_EXIT);
//         }
    //};
    let t = Type();
    let ms = SepByParser::new(t, CharParser(','));
    
    println!("{:?}", ms.run_parser("&i32, **i32, **&&SomeStruct, Mike  , Mike,     Mike"));

}

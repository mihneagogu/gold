#![feature(int_error_matching)]
mod parsing;
mod tests;

#[allow(unused_imports)]
use parsing::literals::parse_number;
use parsing::combinators::option_parse;
use parsing::ParsingContext;



const NO_ARGS_EXIT: i32 = 2;
const FILE_NOT_FOUND_EXIT: i32 = 3;
const EXECUTABLE_AND_MORE: usize = 2;

use std::{fs, process};

fn main() {
    let args: Vec<String> = std::env::args().collect();
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

    let mut pctx = ParsingContext::new("123");
    parse_number(&mut pctx);
    println!("{:?}", pctx);
}

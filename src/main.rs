use std::io::Error;

use frontend::lexer::Token;
use logos::Logos;

mod backend;
mod frontend;

const FILE_ENDING: &str = "grsp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_filename = std::env::args().nth(1).expect("No input file provided");

    if !input_filename.ends_with(FILE_ENDING) {
        return Err(Box::new(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input file must have the ending '.grsp'",
        )));
    }
    let source_file = std::fs::read_to_string(input_filename)?;
    let tokens = Token::lexer(&source_file).collect::<Vec<_>>();

    println!("{:?}", tokens);
    Ok(())
}

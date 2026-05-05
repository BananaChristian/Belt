mod leather;
mod config;

use leather::lexer::Lexer;
use leather::parser::Parser;
use leather::analyzer::Semantics;

use crate::leather::analyzer;

fn main() {
    let mut lexer = Lexer::new();
    lexer.load(&String::from("belt.lethr"));
    lexer.generate();

    let mut parser=Parser::new();
    parser.load(lexer.get_tokens());
    match parser.parse() {
        Ok(ast) => {
            let analyzer=Semantics::new(ast); 
            match analyzer.analyze(){
                Ok(config)=> println!("{:#?}", config),
                Err(e)=> eprintln!("{}", e),
            }
        },
        Err(e) => eprintln!("{}", e),
    }
}

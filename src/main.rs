mod leather;
use leather::lexer::Lexer;
use leather::parser::Parser;

fn main() {
    let mut lexer = Lexer::new();
    lexer.load(&String::from("belt.lethr"));
    lexer.generate();

    let mut parser=Parser::new();
    parser.load(lexer.get_tokens());
    match parser.parse() {
        Ok(config) => println!("{:#?}", config),
        Err(e) => eprintln!("{}", e),
    }
}

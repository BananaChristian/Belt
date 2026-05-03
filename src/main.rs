mod leather;
use leather::lexer::Lexer;

fn main() {
    let mut lexer = Lexer::new();
    lexer.load(&String::from("belt.lethr"));
    lexer.generate();
    lexer.print_tokens();
}

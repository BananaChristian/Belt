mod config;
mod scaffold;
mod graph;
mod project;
mod leather;

use std::env;

use crate::scaffold::scaffold;
use leather::analyzer::Semantics;
use leather::lexer::Lexer;
use leather::parser::Parser;

use crate::config::Config;

fn load_config() -> Result<Config, String> {
    let mut lexer = Lexer::new();
    lexer.load(&String::from("belt.lethr"));
    lexer.generate();

    let mut parser = Parser::new();
    parser.load(lexer.get_tokens());
    let ast = parser.parse()?;

    let analyzer = Semantics::new(ast);
    analyzer.analyze()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("new") => {
            match args.get(2){
                Some(name) =>{
                    match scaffold(name){
                        Ok(_) => println!("Created project {}",name),
                        Err(_)=>eprintln!("Failed to create project {}",name),
                    }
                }
                None => eprintln!("error: belt new requires a project name")
            }
        }
        Some("build") => match load_config() {
            Ok(config) => {
                println!("{:#?}", config);
            }
            Err(e) => eprintln!("{}", e),
        },
        Some("check") => match load_config() {
            Ok(config) => {
                println!("{:#?}", config);
            }
            Err(e) => eprintln!("{}", e),
        },
        None => eprintln!("error: no command provided"),
        _ => eprintln!("error: unknown command"),
    }
}

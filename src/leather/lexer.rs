use std::{collections::HashMap, fs::read_to_string};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenType {
    Identifier, // a normal ident
    String,     // "string"

    //Section keywords
    Project,
    Layout,
    Dependecies,
    Link,
    Commands,

    //Value keywords
    Executable,
    Static,
    Object,

    //Key keywords
    Version,
    Entry,
    Mode,
    Name,
    Target,
    Script,
    Freestanding,
    True,

    Illegal,
    Assign,  // =
    Comma,   //,
    LSquare, //[
    RSquare, //]
    End,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new() -> Self {
        Token {
            token_type: TokenType::Illegal,
            literal: String::new(),
            line: 0,
            col: 0,
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    tokens: Vec<Token>,
    keywords: HashMap<String, TokenType>,
    pos: usize,
    line: usize,
    col: usize,
}

pub fn get_source(path: &String) -> Result<Vec<char>, std::io::Error> {
    let source = read_to_string(path)?;
    Ok(source.chars().collect())
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            input: Vec::new(),
            tokens: Vec::new(),
            keywords: HashMap::from([
                ("project".to_string(), TokenType::Project),
                ("layout".to_string(), TokenType::Layout),
                ("dependecies".to_string(), TokenType::Dependecies),
                ("link".to_string(), TokenType::Link),
                ("commands".to_string(),TokenType::Commands),
                ("executable".to_string(), TokenType::Executable),
                ("static".to_string(), TokenType::Static),
                ("object".to_string(), TokenType::Object),
                ("version".to_string(), TokenType::Version),
                ("entry".to_string(), TokenType::Entry),
                ("mode".to_string(), TokenType::Mode),
                ("name".to_string(), TokenType::Name),
                ("target".to_string(), TokenType::Target),
                ("script".to_string(), TokenType::Script),
                ("freestanding".to_string(), TokenType::Freestanding),
                ("true".to_string(), TokenType::True),
            ]),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn load(&mut self, filename: &String) {
        match get_source(filename) {
            Ok(chars) => self.input = chars,
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::NotFound => eprintln!("error: {} not found", filename),
                    _ => eprint!("error: {}", e),
                }
                std::process::exit(1);
            }
        }
    }

    pub fn get_tokens(self) -> Vec<Token> {
        self.tokens
    }

    fn advance(&mut self) {
        if self.current_char() == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.pos += 1;
    }

    fn current_char(&self) -> char {
        *self.input.get(self.pos).unwrap_or(&'\0')
    }

    fn next_char(&self) -> char {
        *self.input.get(self.pos + 1).unwrap_or(&'\0')
    }

    fn skip_comments(&mut self) {
        if self.current_char() == '#' {
            self.advance();
            //If it is a multiline comment
            if self.current_char() == '#' {
                self.advance();
                while self.pos < self.input.len() {
                    if self.current_char() == '#' && self.next_char() == '#' {
                        self.advance();
                        self.advance();
                        return;
                    }
                    self.advance();
                }
                eprintln!("error: unterminated multiline comment");
                std::process::exit(1);
            }
            //If it is a single line comment
            while self.pos < self.input.len() && self.current_char() != '\n' {
                self.advance();
            }
        }
    }

    pub fn skip_whitespace(&mut self) {
        loop {
            match self.current_char() {
                ' ' | '\t' | '\n' | '\r' => self.advance(),
                '#' => self.skip_comments(),
                _ => break,
            }
        }
    }

    fn tokenize_string(&mut self) -> Token {
        self.advance();
        let mut value = String::new();
        while self.current_char() != '\0' && self.current_char() != '\n' {
            match self.current_char() {
                '"' => {
                    self.advance(); // consume closing quote
                    return Token {
                        token_type: TokenType::String,
                        literal: value,
                        line: self.line,
                        col: self.col,
                    };
                }
                '\\' => {
                    self.advance(); // skip backslash
                    match self.current_char() {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '"' => value.push('"'),
                        '\\' => value.push('\\'),
                        _ => {} // unknown escape, ignore or error
                    }
                    self.advance(); // skip the escape char
                }
                c => {
                    value.push(c);
                    self.advance();
                }
            }
        }

        // unterminated string, return error token or panic
        Token {
            token_type: TokenType::Illegal,
            literal: String::from("unterminated string"),
            line: self.line,
            col: self.col,
        }
    }

    fn tokenize_identifier(&mut self) -> Token {
        let mut identifier = String::new();
        while self.current_char().is_alphabetic() || self.current_char() == '_' {
            identifier.push(self.current_char());
            self.advance();
        }

        let token_type = match self.keywords.get(identifier.as_str()) {
            Some(tt) => tt.clone(),
            None => TokenType::Identifier,
        };

        Token {
            token_type,
            literal: identifier,
            line: self.line,
            col: self.col,
        }
    }

    fn tokenize_symbols(&mut self) -> Token {
        let token = Token {
            token_type: match self.current_char() {
                '[' => TokenType::LSquare,
                ']' => TokenType::RSquare,
                '=' => TokenType::Assign,
                ',' => TokenType::Comma,
                _ => TokenType::Illegal,
            },
            literal: self.current_char().to_string(),
            line: self.line,
            col: self.col,
        };
        self.advance();
        token
    }

    fn tokenize(&mut self) -> Token {
        if self.current_char().is_alphabetic() || self.current_char() == '_' {
            return self.tokenize_identifier();
        }
        if self.current_char() == '"' {
            return self.tokenize_string();
        }
        return self.tokenize_symbols();
    }

    pub fn generate(&mut self) {
        self.tokens.clear();
        while self.pos < self.input.len() {
            self.skip_whitespace();
            if self.pos >= self.input.len() {
                break;
            }
            let token = self.tokenize();
            self.tokens.push(token);
        }

        //Push the end token
        self.tokens.push(Token {
            token_type: TokenType::End,
            literal: String::from(""),
            line: self.line,
            col: self.col,
        });
    }
}

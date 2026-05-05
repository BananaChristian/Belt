use crate::leather::lexer::{Token, TokenType};
//AST
#[derive(Debug)]
pub struct ArrayLiteral {
    lbracket: Token,
    pub expressions: Vec<Expression>,
    rbracket: Token,
}

//This represents identifiers, "", arrays
#[derive(Debug)]
pub enum Expression {
    Identifier(Token),
    Keyword(Token),
    StringLit(Token),
    Array(ArrayLiteral),
}

//For stuff like [project]
#[derive(Debug)]
pub struct Head {
    lbracket: Token,
    pub head: Token,
    rbracket: Token,
}

//For stuff like version = ""
#[derive(Debug)]
pub struct Variable {
    pub name: Expression,
    pub value: Expression,
}

#[derive(Debug)]
pub struct Section {
    pub head: Head,
    pub contents: Vec<Variable>,
}

#[derive(Debug)]
pub struct Config {
    pub sections: Vec<Section>,
}

//PARSER
pub struct Parser {
    tokens: Vec<Token>,
    sentinel: Token,
    pos: usize,
}

pub type ParseError = String;
pub type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new() -> Self {
        Parser {
            tokens: Vec::new(),
            sentinel: Token {
                literal: String::from("illegal"),
                token_type: TokenType::Illegal,
                line: 0,
                col: 0,
            },
            pos: 0,
        }
    }

    pub fn load(&mut self, tokens: Vec<Token>) {
        self.tokens = tokens;
    }

    fn advance(&mut self) {
        if self.pos >= self.tokens.len() {
            self.pos;
        }
        self.pos += 1;
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.sentinel)
    }

    fn next_token(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&self.sentinel)
    }


    fn expect(&mut self, expected_type: TokenType) -> ParseResult<&Token> {
        if self.current_token().token_type == expected_type {
            Ok(self.current_token())
        } else {
            Err(format!(
                "error: expected {:?} but got {:?} at line {} col {}",
                expected_type,
                self.current_token().token_type,
                self.current_token().line,
                self.current_token().col,
            ))
        }
    }

    fn parse_array_literal(&mut self) -> ParseResult<ArrayLiteral> {
        let lbracket = self.expect(TokenType::LSquare)?.clone();
        self.advance(); //Consume the [ token
        let mut expressions = Vec::new();

        while self.current_token().token_type != TokenType::RSquare
            && self.current_token().token_type != TokenType::End
        {
            self.advance(); //Consume the inner expression token
            if self.current_token().token_type == TokenType::Comma {
                self.advance(); //Consume the comma
            }
            let expression = self.parse_expression()?;
            expressions.push(expression);
        }

        let rbracket = self.expect(TokenType::RSquare)?.clone();
        self.advance();

        Ok(ArrayLiteral {
            lbracket: lbracket,
            expressions: expressions,
            rbracket: rbracket,
        })
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        match self.current_token().token_type {
            TokenType::Identifier => {
                let ident = self.current_token().clone();
                self.advance();
                Ok(Expression::Identifier(ident))
            }
            TokenType::Name
            | TokenType::Version
            | TokenType::Entry
            | TokenType::Mode
            | TokenType::Executable
            | TokenType::Static
            | TokenType::Object
            | TokenType::Project
            | TokenType::Layout
            | TokenType::Dependecies
            | TokenType::Link => {
                let kw = self.current_token().clone();
                self.advance();
                Ok(Expression::Keyword(kw))
            }
            TokenType::String => {
                let string = self.current_token().clone();
                self.advance();
                Ok(Expression::StringLit(string))
            }
            TokenType::LSquare => Ok(Expression::Array(self.parse_array_literal()?)),
            _ => Err("Invalid expression".to_string()),
        }
    }

    fn parse_head(&mut self) -> ParseResult<Head> {
        let lbracket = self.expect(TokenType::LSquare)?.clone();
        self.advance(); //Consume the l-bracket token

        let head: ParseResult<&Token> = match self.current_token().token_type {
            TokenType::Project | TokenType::Layout | TokenType::Dependecies | TokenType::Link => {
                Ok(&self.current_token().clone())
            }
            _ => Err("Invalid section header".to_string()),
        };

        self.advance();
        let rbracket = self.expect(TokenType::RSquare)?.clone();
        self.advance();

        Ok(Head {
            lbracket: lbracket.clone(),
            head: head?.clone(),
            rbracket: rbracket.clone(),
        })
    }

    fn parse_variable(&mut self) -> ParseResult<Variable> {
        let identifier = self.parse_expression()?;

        self.expect(TokenType::Assign)?;
        self.advance();
        let value = self.parse_expression()?;

        Ok(Variable {
            name: identifier,
            value: value,
        })
    }

    fn parse_section(&mut self) -> ParseResult<Section> {
        let head = self.parse_head()?;
        let mut contents = Vec::new();

        while self.current_token().token_type != TokenType::LSquare
            && self.current_token().token_type != TokenType::End
            && self.pos < self.tokens.len()
        {
            let content = self.parse_variable()?;
            contents.push(content);
        }

        Ok(Section { head, contents })
    }

    pub fn parse(&mut self) -> ParseResult<Config> {
        let mut sections = Vec::new();
        while self.current_token().token_type != TokenType::End {
            match self.current_token().token_type {
                TokenType::LSquare => {
                    let section = self.parse_section()?;
                    sections.push(section);
                }
                _ => {
                    return Err(format!(
                        "error: unexpected token {:?} at line {} col {}",
                        self.current_token().token_type,
                        self.current_token().line,
                        self.current_token().col,
                    ));
                }
            }
        }
        Ok(Config { sections })
    }
}

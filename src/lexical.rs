use std::collections::VecDeque;


#[derive(Debug)]
pub enum Token {
    Exit,
    IntLiteral(String),
    SemiColon,
    Let,
    Indent(String),
    Equal,
    OpenBracket,
    CloseBracket,
    Add,
    Multiply,
    Subtract,
    Division,
} 

pub fn tokenize(content: &str) -> Vec<Token>{

    let mut chars : VecDeque<_>= content.chars().collect();
    let mut tokens = vec![];
    let mut buffer = vec![];

    while let Some(char) = chars.pop_front() {
        if char.is_alphabetic() {
            buffer.push(char);
            while let Some(element) = chars.front() {
                if element.is_ascii_alphanumeric() {
                    buffer.push(chars.pop_front().unwrap());    
                } else {
                    break;
                }
            }
            let temp: String = buffer.iter().collect();
            if temp == "let" {
                tokens.push(Token::Let);
                buffer.clear();
            } 
            else if temp == "exit" {
                tokens.push(Token::Exit);
                buffer.clear();
            } else {
                tokens.push(Token::Indent(temp));
                buffer.clear();
            }
        }
        else if char.is_ascii_digit() {
            if char.is_ascii_digit() {
                buffer.push(char);
                while let Some(element) = chars.front() {
                    if element.is_ascii_digit() {
                        buffer.push(chars.pop_front().unwrap());    
                    } else {
                        break;
                    }
                }
                let temp: String = buffer.iter().collect();
                tokens.push(Token::IntLiteral(temp));
                buffer.clear();
            }
        }
        else if char == '=' {
            tokens.push(Token::Equal);
        }
        else if char == ';' {
            tokens.push(Token::SemiColon);
        }
        else if char == '(' {
            tokens.push(Token::OpenBracket);
        }
        else if char == ')' {
            tokens.push(Token::CloseBracket);
        }
        else if char == '+' {
            tokens.push(Token::Add);
        }
        else if char == '-' {
            tokens.push(Token::Subtract);
        }
        else if char == '/' {
            tokens.push(Token::Division);
        }
        else if char == '*' {
            tokens.push(Token::Multiply);
        }
        else if char.is_whitespace() {
        }
        else {
            panic!("Invalid Token");
        }
    }

    tokens
}


pub fn binary_precendence(token: &Token) -> i8 {
    match token {
        Token::Add => 1, 
        Token::Subtract => 1, 
        Token::Multiply => 2, 
        Token::Division => 2, 
        _ => unreachable!()
    }
}

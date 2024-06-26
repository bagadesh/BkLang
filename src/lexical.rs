use std::collections::VecDeque;


#[derive(Debug, PartialEq)]
pub struct TokenData {
    pub token: Token,
    pub line: i32,
}

#[derive(Debug, PartialEq)]
pub enum LitKind {
    Integer,
    Bool,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Exit,
    IntLiteral(String),
    BooleanLiteral(bool),
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
    OpenScope,
    CloseScope,
    If,
    Else,
    ElseIf,
    ReturnSig, // ->
    FuncSig, // fn
    Equality, // ==
    GreaterThan, // >
    GreaterThanEqual, // >=
    LessThan, // <
    LessThanEqual, // <=
    OR, // ||
    AND, // &&
    LitType(LitKind),
} 

pub fn tokenize(content: &str) -> Vec<TokenData>{

    let mut chars : VecDeque<_>= content.chars().collect();
    let mut tokens = vec![];
    let mut buffer = vec![];
    let mut line_count = 1;

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
                tokens.push(TokenData { token: Token::Let, line: line_count });
                buffer.clear();
            } 
            else if temp == "return" {
                tokens.push(TokenData { token: Token::Exit, line: line_count });
                buffer.clear();
            }
            else if temp == "if" {
                if let Some(previous) = tokens.last() {
                    if previous.token == Token::Else {
                        tokens.pop();
                        tokens.push(TokenData { token: Token::ElseIf, line: line_count });
                        buffer.clear();
                        continue;
                    }
                }
                tokens.push(TokenData { token: Token::If, line: line_count });
                buffer.clear();
            }
            else if temp == "else" {
                tokens.push(TokenData { token: Token::Else, line: line_count });
                buffer.clear();
            }
            else if temp == "i32" {
                tokens.push(TokenData { token: Token::LitType(LitKind::Integer), line: line_count });
                buffer.clear();
            }
            else if temp == "bool" {
                tokens.push(TokenData { token: Token::LitType(LitKind::Bool), line: line_count });
                buffer.clear();
            }
            else if temp == "fn" {
                tokens.push(TokenData { token: Token::FuncSig, line: line_count });
                buffer.clear();
            }
            else {
                tokens.push(TokenData { token: Token::Indent(temp),  line: line_count });
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
                if temp == "true" || temp == "false" {
                    let value = if temp == "true" {
                        true
                    } else {
                        false
                    };
                    tokens.push(TokenData { token: Token::BooleanLiteral(value), line: line_count });
                } else {
                    tokens.push(TokenData { token: Token::IntLiteral(temp), line: line_count });
                }
                buffer.clear();
            }
        }
        else if char == '<' && is_next(&chars, '=') {
            chars.pop_front();
            tokens.push(TokenData { token: Token::LessThanEqual, line: line_count });
        }
        else if char == '<' {
            chars.pop_front();
            tokens.push(TokenData { token: Token::LessThan, line: line_count });
        }
        else if char == '>' && is_next(&chars, '=') {
            chars.pop_front();
            tokens.push(TokenData { token: Token::GreaterThanEqual, line: line_count });
        }
        else if char == '>' {
            chars.pop_front();
            tokens.push(TokenData { token: Token::GreaterThan, line: line_count });
        }
        else if char == '-' && is_next(&chars, '>') {
            chars.pop_front();
            tokens.push(TokenData { token: Token::ReturnSig, line: line_count });
        }
        else if char == '/' && is_next(&chars, '/') {
            chars.pop_front();
            while !chars.is_empty() && !is_next(&chars, '\n') {
                chars.pop_front();
            }
        }
        else if char == '=' && is_next(&chars, '=') {
            chars.pop_front();
            tokens.push(TokenData { token: Token::Equality, line: line_count });
        }
        else if char == '|' && is_next(&chars, '|') {
            chars.pop_front();
            tokens.push(TokenData { token: Token::OR, line: line_count });
        }
        else if char == '&' && is_next(&chars, '&') {
            chars.pop_front();
            tokens.push(TokenData { token: Token::AND, line: line_count });
        }
        else if char == '/' && is_next(&chars, '*') {
            chars.pop_front();
            while !chars.is_empty() && !(is_next(&chars, '*') && peek(&chars, '/', 1)) {
                chars.pop_front();
            }
            chars.pop_front();
            chars.pop_front();
        }
        else if char == '=' {
            tokens.push(TokenData { token: Token::Equal, line: line_count });
        }
        else if char == ';' {
            tokens.push(TokenData { token: Token::SemiColon, line: line_count });
        }
        else if char == '(' {
            tokens.push(TokenData { token: Token::OpenBracket, line: line_count });
        }
        else if char == ')' {
            tokens.push(TokenData { token: Token::CloseBracket, line: line_count });
        }
        else if char == '+' {
            tokens.push(TokenData { token: Token::Add, line: line_count });
        }
        else if char == '-' {
            tokens.push(TokenData { token: Token::Subtract, line: line_count });
        }
        else if char == '/' {
            tokens.push(TokenData { token: Token::Division, line: line_count });
        }
        else if char == '*' {
            tokens.push(TokenData { token: Token::Multiply, line: line_count });
        }
        else if char == '{' {
            tokens.push(TokenData { token: Token::OpenScope, line: line_count });
        }
        else if char == '}' {
            tokens.push(TokenData { token: Token::CloseScope, line: line_count });
        }
        else if char == '\n' {
            line_count += 1;
        }
        else if char.is_whitespace() {
        }
        else {
            panic!("Invalid Token {}", char);
        }
    }

    tokens
}

fn peek(chars: &VecDeque<char>, identifier: char, offset: usize) -> bool {
    if let Some(ele) = chars.get(offset)  {
        return *ele == identifier;
    }
    false
}


fn is_next(chars: &VecDeque<char>, identifier: char) -> bool {
    if let Some(ele) = chars.front()  {
        return *ele == identifier
    }
    false
}


pub fn binary_precendence(token: &Token) -> i8 {
    match token {
        Token::Add => 1, 
        Token::Subtract => 1, 
        Token::Multiply => 2, 
        Token::Division => 2, 
        Token::Equality => 2, 
        Token::OR => 1, 
        Token::AND => 1, 
        Token::GreaterThan => 1,
        Token::LessThan => 1,
        Token::GreaterThanEqual => 1,
        Token::LessThanEqual => 1,
        Token::Exit => unreachable!(),
        Token::IntLiteral(_) => unreachable!(),
        Token::BooleanLiteral(_) => unreachable!(),
        Token::SemiColon => unreachable!(),
        Token::Let => unreachable!(),
        Token::Indent(_) => unreachable!(),
        Token::Equal => unreachable!(),
        Token::OpenBracket => unreachable!(),
        Token::CloseBracket => unreachable!(),
        Token::OpenScope => unreachable!(),
        Token::CloseScope => unreachable!(),
        Token::If => unreachable!(),
        Token::Else => unreachable!(),
        Token::ElseIf => unreachable!(),
        Token::ReturnSig => unreachable!(),
        Token::FuncSig => unreachable!(),
        Token::LitType(_) => unreachable!(),
    }
}

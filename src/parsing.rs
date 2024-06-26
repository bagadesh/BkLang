use core::panic;
use std::collections::VecDeque;

use tracing::debug;

use crate::{cast, lexical::{binary_precendence, LitKind, Token, TokenData}};


#[derive(Debug)]
pub struct NodeRoot {
   pub funcs: Vec<NodeFunc>,
}

#[derive(Debug)]
pub struct NodeFunc {
    pub f_name: String,
    pub stmts: Vec<NodeStmt>,
    pub return_type: Option<LitKind>,
}

impl NodeFunc {

    fn new(name: String, stmts: Vec<NodeStmt>, ret_type: Option<LitKind>) -> NodeFunc {
        NodeFunc { f_name: name, stmts, return_type: ret_type }
    }
    
}

#[derive(Debug)]
pub struct NodeScope(pub Vec<NodeStmt>);

#[derive(Debug)]
pub enum NodeStmt {
    Let{  ident: TokenData, expr: NodeExpr },
    Return { expr:NodeExpr },
    Scope { scope: NodeScope },
    If {
        expr: NodeExpr,
        scope: NodeScope,
        chain: Option<NodeElse>
    },
    ReAssign{ expr: NodeExpr, ident: TokenData},
}

#[derive(Debug)]
pub enum NodeElse {
    ElseIf {
        expr: NodeExpr,
        scope: NodeScope,
        chain: Box<Option<NodeElse>>,
    },
    Else(NodeScope),
}


#[derive(Debug)]
pub enum NodeExpr {
   BinaryExpr(Box<NodeBiExpr>),
   Term(NodeTermExpr),
   Call(String),
}

#[derive(Debug)]
pub enum NodeTermExpr {
   IntLiteral(String),
   BooleanLiteral(bool),
   Identifier(String),
   Expression(Box<NodeExpr>),
}

#[derive(Debug)]
pub struct NodeBiExpr {
    pub lhs: NodeExpr,
    pub rhs: NodeExpr,
    pub op: NodeBiOp,
}


#[derive(Debug)]
pub enum NodeBiOp {
    Add, Multiply, Subtract, Division, Equality, OR, AND,
    GreaterThan, LessThan, LessThanEqual, GreaterThanEqual,
}

struct Parser {
    tokens: VecDeque<TokenData>,
} 


impl Parser {

    fn consume_count(&mut self, count : usize) {
        for _ in 0..count  {
            self.tokens.pop_front();
        }
    }

    fn peek(&self, offset: usize) -> Option<&TokenData> {
        self.tokens.get(offset)
    }
 
    fn peek_expect(&self, offset: usize, token: Token) ->bool {
        if let Some(found) =  self.tokens.get(offset) {
            return found.token == token;
        }
        false
    }
    
    fn expect(&mut self ,token: Token) -> Token {
        if let Some(found) = self.tokens.pop_front()  {
            if found.token != token {
                panic!("Missing {:?} at line {} but found {:?}", token, found.line, found.token);
            }
            return found.token;
        }
        panic!("Missing {:?} ", token, );
    }

    fn expect_expr(&mut self) -> NodeExpr {
        if let Some(expr) = self.parse_expr(1) {
            return expr;
        } 
        panic!("Expression Parsing Failed {:?}", self.tokens);
    }

}

// Expression Parser

impl Parser {

    fn parse_func(&mut self) -> Option<NodeFunc> {
        let name_token = self.tokens.pop_front()?.token;
        let fuc_name = cast!(&name_token, Token::Indent);
        self.expect(Token::OpenBracket);
        self.expect(Token::CloseBracket);
        self.expect(Token::ReturnSig);
        let ret_type = self.tokens.pop_front().map(|f| {
            cast!(f.token, Token::LitType)
        });
        self.expect(Token::OpenScope);
        let stmts = self.parse_stmts();
        Some(NodeFunc::new(fuc_name.to_owned(), stmts, ret_type))
    }
    
    fn parse_expr(&mut self, min_prec: i8) -> Option<NodeExpr> {
        if let Some(token) = self.peek(0) {
            if let Token::Indent(ident) = &token.token  {
                if self.peek_expect(1, Token::OpenBracket)
                    && self.peek_expect(2, Token::CloseBracket) {
                        let ident = ident.to_owned();
                        self.consume_count(3);
                        return Some(NodeExpr::Call(ident));
                    }
            }
        }

        let term_option = self.parse_term()?;
        let mut lhs = NodeExpr::Term(term_option);

        while let Some(next) = self.tokens.front()  {
            if !is_binary_operator(&next.token) || binary_precendence(&next.token) < min_prec  {
                break;
            }
            let prec = binary_precendence(&next.token);
            let operator = self.tokens.pop_front().unwrap();
            let rhs = self.parse_expr(prec + 1).expect("Rhs has to be provided");
            let op = operator_to_binary_op(&operator.token);

            lhs = NodeExpr::BinaryExpr(Box::new(NodeBiExpr{ lhs, rhs, op  }));
        }

        Some(lhs)
    }


    fn parse_term(&mut self) -> Option<NodeTermExpr> {
        if let Some(element) = self.tokens.front() {
            if let Token::BooleanLiteral(token) = &element.token {
                let token = token.clone();
                self.tokens.pop_front().unwrap();
                return Some(NodeTermExpr::BooleanLiteral(token));
            }
            if let Token::IntLiteral(token) = &element.token {
                let token = token.to_string();
                self.tokens.pop_front().unwrap();
                return Some(NodeTermExpr::IntLiteral(token));
            }
            if let Token::Indent(token) = &element.token {
                let token = token.to_string();
                self.tokens.pop_front().unwrap();
                return Some(NodeTermExpr::Identifier(token.to_string()));
            }
            if let Token::OpenBracket = element.token {
                self.tokens.pop_front().unwrap();
                let expr = self.parse_expr(1).expect("Unable to parse expression");
                self.tokens.pop_front().expect("')' is missing");
                return Some(NodeTermExpr::Expression(Box::new(expr)));
            }
        }

        None
    }

}

impl Parser {

    fn parse_else(&mut self) -> Option<NodeElse> {
        while let Some(token) = self.tokens.front() {
            match token.token {
                Token::ElseIf => {
                    self.tokens.pop_front();
                    self.expect(Token::OpenBracket);
                    let expr = self.expect_expr();
                    self.expect(Token::CloseBracket);
                    self.expect(Token::OpenScope);
                    let if_statments = self.parse_stmts();
                    let scope = NodeScope(if_statments);
                    let chain = Box::new(self.parse_else());

                    return Some(NodeElse::ElseIf { expr, scope, chain })
                },


                Token::Else => {
                    self.tokens.pop_front();
                    self.expect(Token::OpenScope);
                    let if_statments = self.parse_stmts();
                    let scope = NodeScope(if_statments);
                    return Some(NodeElse::Else(scope))
                },

                _ => {
                    return None;
                },

            }
        }
        None
    }

    fn parse_stmts(&mut self,) -> Vec<NodeStmt> {
        let mut stmts = vec![];
        while let Some(token) = self.tokens.pop_front() {
            match token.token {
                Token::Exit => {
                    let expr = self.expect_expr();
                    stmts.push(NodeStmt::Return { expr });
                    self.expect(Token::SemiColon);
                },
                Token::Let => {
                    let ident = self.tokens.pop_front().expect("identifier missing");
                    self.expect(Token::Equal);
                    let expr = self.expect_expr();
                    self.expect(Token::SemiColon);
                    stmts.push(NodeStmt::Let { expr , ident } );
                },
                Token::OpenScope => {
                    let scoped_stmts = self.parse_stmts();
                    stmts.push(NodeStmt::Scope { scope: NodeScope(scoped_stmts) });
                }

                Token::CloseScope => {
                    return stmts;
                }
                Token::If => {
                    self.expect(Token::OpenBracket);
                    let expr = self.expect_expr();
                    debug!("Parsing If condition {:?}", expr);
                    self.expect(Token::CloseBracket);
                    self.expect(Token::OpenScope);
                    let if_statments = self.parse_stmts();
                    let scope = NodeScope(if_statments);
                    let chain = self.parse_else();
                    stmts.push(NodeStmt::If { expr, scope, chain });
                },

                Token::Indent(_) if self.peek_expect(0, Token::Equal) => {
                    self.expect(Token::Equal);
                    let expr = self.expect_expr();
                    self.expect(Token::SemiColon);
                    stmts.push(NodeStmt::ReAssign { expr , ident: token } );
                },

                Token::FuncSig => {
                    self.parse_func().expect("Function parsing failed");
                },

                _ => { continue; },
            }
        }

        stmts
    }

    fn parse_file(&mut self) -> Vec<NodeFunc> {

        let mut funcs = vec![];

        while let Some(token) = self.tokens.pop_front() {
            match token.token {
                Token::FuncSig => {
                    let func = self.parse_func().expect("Function parsing failed");
                    funcs.push(func);
                },

                _ => {},
            }
        }

        funcs
    }
    
}
pub fn parse(tokens: VecDeque<TokenData>) -> NodeRoot {
    let mut parser = Parser { tokens };
    let stmts = parser.parse_file();
    NodeRoot { funcs: stmts }
}

fn is_binary_operator(token: &Token) -> bool {
    match token {
        Token::Multiply | Token::Add 
        | Token::Subtract | Token::Division |
        Token::AND | Token::OR | Token::Equality => true,
        Token::GreaterThan => true,
        Token::LessThan => true,
        Token::GreaterThanEqual => true,
        Token::LessThanEqual => true,
        Token::Exit => false,
        Token::IntLiteral(_) => false,
        Token::BooleanLiteral(_) => false,
        Token::SemiColon => false,
        Token::Let => false,
        Token::Indent(_) => false,
        Token::Equal => false,
        Token::OpenBracket => false,
        Token::CloseBracket => false,
        Token::OpenScope => false,
        Token::CloseScope => false,
        Token::If => false,
        Token::Else => false,
        Token::ElseIf => false,
        Token::ReturnSig => false,
        Token::FuncSig => false,
        Token::LitType(_) => false,
    }
}

fn operator_to_binary_op(token: &Token) -> NodeBiOp  {
    match token {
        Token::Add => NodeBiOp::Add,
        Token::Multiply => NodeBiOp::Multiply,
        Token::Subtract => NodeBiOp::Subtract,
        Token::Division => NodeBiOp::Division,
        Token::Equality => NodeBiOp::Equality,
        Token::OR => NodeBiOp::OR,
        Token::AND => NodeBiOp::AND,
        Token::GreaterThan => NodeBiOp::GreaterThan,
        Token::LessThan => NodeBiOp::LessThan,
        Token::GreaterThanEqual => NodeBiOp::GreaterThanEqual,
        Token::LessThanEqual => NodeBiOp::LessThanEqual,
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

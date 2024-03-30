use core::panic;
use std::collections::VecDeque;

use crate::lexical::{binary_precendence, Token, TokenData};


#[derive(Debug)]
pub struct NodeRoot {
   pub stmts: Vec<NodeStmt>
}


#[derive(Debug)]
pub struct NodeScope(pub Vec<NodeStmt>);

#[derive(Debug)]
pub enum NodeStmt {
    Let{ expr: NodeExpr, ident: TokenData },
    Exit { expr:NodeExpr },
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
}

#[derive(Debug)]
pub enum NodeTermExpr {
   IntLiteral(String),
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
    Add, Multiply, Subtract, Division,
}

struct Parser {
    tokens: VecDeque<TokenData>,
} 

impl Parser {

    fn peek_expect(&self, offset: usize, token: Token) ->bool {
        if let Some(found) =  self.tokens.get(offset) {
            return found.token == token;
        }
        false
    }
    
    fn expect(&mut self ,token: Token) -> Token {
        if let Some(found) = self.tokens.pop_front()  {
            if found.token != token {
                panic!("Missing {:?} at line {}", token, found.line);
            }
            return found.token;
        }
        panic!("Missing {:?} ", token, );
    }

    fn expect_expr(&mut self) -> NodeExpr {
        if let Some(expr) = parse_expr(&mut self.tokens, 1) {
            return expr;
        } 
         panic!("Expression Parsing Failed {:?}", self.tokens);
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
                    let if_statments = self.parse();
                    let scope = NodeScope(if_statments);
                    let chain = Box::new(self.parse_else());

                    return Some(NodeElse::ElseIf { expr, scope, chain })
                },


                Token::Else => {
                    self.tokens.pop_front();
                    self.expect(Token::OpenScope);
                    let if_statments = self.parse();
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

    fn parse(&mut self,) -> Vec<NodeStmt> {

        let mut stmts = vec![];
        while let Some(token) = self.tokens.pop_front() {
            match token.token {
                Token::Exit => {
                    self.expect(Token::OpenBracket);
                    let expr = self.expect_expr();
                    stmts.push(NodeStmt::Exit { expr });
                    self.expect(Token::CloseBracket);
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
                    let scoped_stmts = self.parse();
                    stmts.push(NodeStmt::Scope { scope: NodeScope(scoped_stmts) });
                }

                Token::CloseScope => {
                    return stmts;
                }
                Token::If => {
                    self.expect(Token::OpenBracket);
                    let expr = self.expect_expr();
                    self.expect(Token::CloseBracket);
                    self.expect(Token::OpenScope);
                    let if_statments = self.parse();
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

                _ => { continue; },
            }
        }

        stmts
    }
    
}
pub fn parse(tokens: VecDeque<TokenData>) -> NodeRoot {
    let mut parser = Parser { tokens };
    let stmts = parser.parse();
    NodeRoot { stmts }
}


fn parse_expr(tokens : &mut VecDeque<TokenData>,  min_prec: i8) -> Option<NodeExpr> {
    let term_option = parse_term(tokens)?;
    let mut lhs = NodeExpr::Term(term_option);

    while let Some(next) = tokens.front()  {
        if !is_binary_operator(&next.token) || binary_precendence(&next.token) < min_prec  {
            break;
        }
        let prec = binary_precendence(&next.token);
        let operator = tokens.pop_front().unwrap();
        let rhs = parse_expr(tokens, prec + 1).expect("Rhs has to be provided");
        let op = operator_to_binary_op(&operator.token);

        lhs = NodeExpr::BinaryExpr(Box::new(NodeBiExpr{ lhs, rhs, op  }));
    }

    Some(lhs)
}


fn parse_term(tokens : &mut VecDeque<TokenData>) -> Option<NodeTermExpr> {

    if let Some(element) = tokens.front() {
        if let Token::IntLiteral(token) = &element.token {
            let token = token.to_string();
            tokens.pop_front().unwrap();
            return Some(NodeTermExpr::IntLiteral(token));
        }
        if let Token::Indent(token) = &element.token {
            let token = token.to_string();
            tokens.pop_front().unwrap();
            return Some(NodeTermExpr::Identifier(token.to_string()));
        }
        if let Token::OpenBracket = element.token {
            tokens.pop_front().unwrap();
            let expr = parse_expr(tokens, 1).expect("Unable to parse expression");
            tokens.pop_front().expect("')' is missing");
            return Some(NodeTermExpr::Expression(Box::new(expr)));
        }
    }

    None
}


fn is_binary_operator(token: &Token) -> bool {
    match token {
        Token::Multiply | Token::Add 
            | Token::Subtract | Token::Division => true,

        _ => false,
    }
}

fn operator_to_binary_op(token: &Token) -> NodeBiOp  {
    match token {
        Token::Add => NodeBiOp::Add,
        Token::Multiply => NodeBiOp::Multiply,
        Token::Subtract => NodeBiOp::Subtract,
        Token::Division => NodeBiOp::Division,
        _ => unreachable!()
    }
}

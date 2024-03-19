use core::panic;
use std::collections::VecDeque;

use crate::lexical::{binary_precendence, Token};


#[derive(Debug)]
pub struct NodeRoot {
   pub stmts: Vec<NodeStmt>
}

#[derive(Debug)]
pub enum NodeStmt {
    Let{ expr: NodeExpr, ident: Token },
    Exit { expr:NodeExpr },
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
    tokens: VecDeque<Token>,
} 

impl Parser {

    fn parse(&mut self,) -> NodeRoot {

        let mut stmts = vec![];
        let mut tokens = &mut self.tokens;
        while let Some(token) = tokens.pop_front() {
            match token {
                Token::Exit => {
                    let open_token = tokens.pop_front().expect("( missing");

                    if !matches!(open_token, Token::OpenBracket) {
                        panic!("( missing but found {:?}", open_token);
                    }

                    if let Some(expr) = parse_expr(&mut tokens, 1) {
                        stmts.push(NodeStmt::Exit { expr });
                    } else {
                        panic!("Expression Parsing Failed {:?}", tokens);
                    }

                    if let Some(element) = tokens.pop_front()  {
                        if !matches!(element, Token::CloseBracket) {
                            panic!(") missing but found {:?}", element);
                        }
                    } else {
                        panic!(") missing ");
                    }
                    if let Some(element) = tokens.pop_front()  {
                        if !matches!(element, Token::SemiColon) {
                            panic!("; missing but found {:?}", element);
                        }
                    } else {
                        panic!("; missing ");
                    }

                },
                Token::Let => {
                    let ident = tokens.pop_front().expect("identifier missing");
                    tokens.pop_front().expect(" Equal Sign missing");

                    if let Some(expr) = parse_expr(&mut tokens, 1) {
                        stmts.push(NodeStmt::Let { expr , ident } );
                    } else {
                        panic!("Expression Parsing Failed");
                    }

                },
                _ => { continue; },
            }
        }

        NodeRoot { stmts }
    }
    
}
pub fn parse(tokens: VecDeque<Token>) -> NodeRoot {
    let mut parser = Parser { tokens };
    parser.parse()
}


fn parse_expr(tokens : &mut VecDeque<Token>,  min_prec: i8) -> Option<NodeExpr> {
    let term_option = parse_term(tokens)?;
    let mut lhs = NodeExpr::Term(term_option);

    while let Some(next) = tokens.front()  {
        if !is_binary_operator(next) || binary_precendence(next) < min_prec  {
            break;
        }
        let prec = binary_precendence(next);
        let operator = tokens.pop_front().unwrap();
        let rhs = parse_expr(tokens, prec + 1).expect("Rhs has to be provided");
        let op = operator_to_binary_op(&operator);

        lhs = NodeExpr::BinaryExpr(Box::new(NodeBiExpr{ lhs, rhs, op  }));
    }

    Some(lhs)
}


fn parse_term(tokens : &mut VecDeque<Token>) -> Option<NodeTermExpr> {

    if let Some(element) = tokens.front() {
        if let Token::IntLiteral(token) = element {
            let token = token.to_string();
            tokens.pop_front().unwrap();
            return Some(NodeTermExpr::IntLiteral(token));
        }
        if let Token::Indent(token) = element{
            let token = token.to_string();
            tokens.pop_front().unwrap();
            return Some(NodeTermExpr::Identifier(token.to_string()));
        }
        if let Token::OpenBracket = element {
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

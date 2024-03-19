use core::panic;
use std::{collections::HashMap, fs::File, io::Write};

use crate::{
    lexical::Token,
    parsing::{NodeBiExpr, NodeExpr, NodeRoot, NodeTermExpr},
};

struct Generator {
    buffer: Vec<String>,
    ident_map: HashMap<String, Var>,
    m_stack_pointer: usize,
    output: File,
}

impl Generator {
    fn push(&mut self, value: &str) {
        self.buffer
            .push(format!("STP {}, X2, [SP, #-16]!\n", value));
        self.m_stack_pointer += 1;
    }

    fn pop(&mut self, register: &str) {
        self.buffer.push(format!("LDP {}, X2, [SP]\n", register));
        self.m_stack_pointer -= 1;
    }

    fn parse_expr(&mut self, expr: &NodeExpr) {
        match expr {
            NodeExpr::BinaryExpr(binary_expr) => {
                self.parse_binary_expr(&binary_expr);
            }
            NodeExpr::Term(rhs_term) => {
                self.parse_term(&rhs_term);
            }
        }
    }

    fn parse_binary_expr(&mut self, binary_expr: &NodeBiExpr) {
        let lhs = &binary_expr.lhs;
        let rhs = &binary_expr.rhs;
        self.parse_expr(&lhs);
        let lhs_pos = self.m_stack_pointer;
        self.parse_expr(&rhs);
        let lhs_pos = self.m_stack_pointer - lhs_pos;
        let lhs_pos = lhs_pos * 16;

        self.buffer.push(format!("LDP X1, X2, [SP, #{}]\n", lhs_pos));
        self.buffer.push(format!("LDP X2, X3, [SP]\n"));

        match binary_expr.op {
            crate::parsing::NodeBiOp::Add => {
                self.buffer.push(format!("ADD X1, X1, X2\n"));
                self.push("X1");
            }
            crate::parsing::NodeBiOp::Multiply => {
                self.buffer.push(format!("MUL X1, X1, X2\n"));
                self.push("X1");
            }
            crate::parsing::NodeBiOp::Subtract => {
                self.buffer.push(format!("SUBS X1, X1, X2\n"));
                self.push("X1");
            }
            crate::parsing::NodeBiOp::Division => {
                self.buffer.push(format!("SDIV X1, X1, X2\n"));
                self.push("X1");
            },
        }
    }

    fn parse_term(&mut self, term: &NodeTermExpr) {
        match term {
            crate::parsing::NodeTermExpr::IntLiteral(value) => {
                self.buffer.push(format!("MOV X1, #{}\n", value));
                self.push("X1");
            }
            crate::parsing::NodeTermExpr::Identifier(value) => {
                let map_value = self
                    .ident_map
                    .get(&value.to_string())
                    .expect(&format!("Undefinied variable {}", value).to_owned());

                let offset = self.m_stack_pointer - map_value.stack_location;
                let offset = offset * 16;

                self.buffer.push(format!("LDP X1, X2, [SP, #{}]\n", offset));
                self.push("X1");
            }
            NodeTermExpr::Expression(expr) => { 
                self.parse_expr(&expr);
            },
        }
    }
}

struct Var {
    stack_location: usize,
}

pub fn generate_code(node_root: NodeRoot) {
    let stmts = node_root.stmts;
    let output = File::create("out.s").expect("Failed to create file");
    let ident_map: HashMap<String, Var> = HashMap::new();
    let m_stack_pointer: usize = 0;
    let mut generator = Generator {
        buffer: vec![],
        ident_map,
        m_stack_pointer,
        output,
    };

    generator.buffer.push(".global _start\n".to_owned());
    generator.buffer.push(".align 2\n".to_owned());
    generator.buffer.push("_start:\n".to_owned());

    for ele in stmts {
        match ele {
            crate::parsing::NodeStmt::Let { expr, ident } => {
                let identifier = cast!(ident, Token::Indent);
                if generator.ident_map.contains_key(&identifier) {
                    panic!("Identifier already defined {}", identifier);
                }

                match expr {
                    NodeExpr::BinaryExpr(binary_expr) => {
                        generator.parse_binary_expr(&binary_expr);
                        generator.ident_map.insert(
                            identifier,
                            Var {
                                stack_location: generator.m_stack_pointer,
                            },
                        );
                    }
                    NodeExpr::Term(term) => {
                        generator.parse_term(&term);
                        generator.ident_map.insert(
                            identifier,
                            Var {
                                stack_location: generator.m_stack_pointer,
                            },
                        );
                    }
                }
            }
            crate::parsing::NodeStmt::Exit { expr } => {
                match expr {
                    NodeExpr::BinaryExpr(binary_expr) => {
                        generator.parse_binary_expr(&binary_expr);
                        generator.pop("X0");
                    }
                    NodeExpr::Term(term) => {
                        generator.parse_term(&term);
                        generator.pop("X0");
                    }
                }

                generator.buffer.push(format!("mov X16, #1\n"));
                generator.buffer.push(format!("svc #0x80\n"));
            }
        }
    }

    let buf: String = generator.buffer.into_iter().collect();
    let buf = buf.as_bytes();

    let _ = generator.output.write_all(buf);
}

#[macro_export]
macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            // #1
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat)); // #2
        }
    }};
}

pub(crate) use cast;

use core::panic;
use std::{collections::HashMap, fs::File, io::Write};

use crate::{
    lexical::Token,
    parsing::{NodeBiExpr, NodeElse, NodeExpr, NodeFunc, NodeRoot, NodeScope, NodeStmt, NodeTermExpr},
};

#[derive(Debug)]
struct Generator {
    buffer: Vec<String>,
    m_stack_pointer: usize,
    output: File,
    scope_map: HashMap<usize, HashMap<String, Var>>,
    scope_ident: usize,
    label_index: usize,
}

impl Generator {

    fn get_variable(&self, identifier: &String) -> Option<&Var> {

        for i in (0..=self.scope_ident).rev() {
            let current_map = self.scope_map.get(&i);
            if let Some(current_map) = current_map {
                if let Some(variable) = current_map.get(identifier) {
                    return Some(variable);
                }
            }
        }

        None
    }

    fn begin_scope(&mut self,) {
        self.scope_ident += 1;
    }

    fn end_scope(&mut self,) {
        let map = self.scope_map.get(&self.scope_ident);
        if let Some(map) = map {
            let ident_scope_count = map.keys().count();
            // Because of 128-bit alignment in Aarch64 we are pushing dummy values
            // all the time when we add data to stack so we have use 16 than 8
            let ident_scope_size = ident_scope_count * 16;
            
            // Remove scope variables from Stack
            self.buffer.push(format!("ADD SP, SP, #{}\n", ident_scope_size));

            // Making our stack pointer track correct head
            self.m_stack_pointer -= ident_scope_count;
        }
        self.scope_ident -= 1;
    }

    fn insert_ident(&mut self, identifier: &String, line_number: i32) {

        let map = self.scope_map.get_mut(&self.scope_ident);
        if let Some(map) = map  {
            map.insert(
                identifier.to_string(),
                Var {
                    stack_location: self.m_stack_pointer,
                    line_number,
                }
            );
        } else {
            let mut value_map = HashMap::new();
            value_map.insert(
                identifier.to_string(),
                Var {
                    stack_location: self.m_stack_pointer,
                    line_number,
                }
            );

            self.scope_map.insert(self.scope_ident, value_map);
        }

    }

    fn push(&mut self, value: &str) {
        self.buffer
            .push(format!("STP {}, X2, [SP, #-16]!\n", value));
        self.m_stack_pointer += 1;
    }

    fn pop(&mut self, register: &str) {
        self.buffer.push(format!("LDP {}, X2, [SP]\n", register));
        self.m_stack_pointer -= 1;
    }

}

impl Generator {

    fn create_label(&mut self) -> String {
        let label = format!("label{}", self.label_index);
        self.label_index += 1;
        label
    }

}

impl Generator {
    
    fn parse_expr(&mut self, expr: &NodeExpr) {
        match expr {
            NodeExpr::BinaryExpr(binary_expr) => {
                self.parse_binary_expr(&binary_expr);
            }
            NodeExpr::Term(rhs_term) => {
                self.parse_term(&rhs_term);
            }
            NodeExpr::Call(f_name) => {
                self.buffer.push(format!("MOV X29, X30\n"));
                self.buffer.push(format!("BL _{}\n", f_name));
                self.buffer.push(format!("MOV X30, X29\n"));
            },
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
                    .get_variable(&value)
                    .expect(&format!("Undefined variable {}", value));

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

// Statement

impl Generator {

    fn gen_func(&mut self, func: &NodeFunc) {
        let f_name = &func.f_name;
        let stmts = &func.stmts;

        if f_name == "main" {
            self.buffer.push("_start:\n".to_owned());
            for stmt in stmts  {
                self.generate_stmt(stmt, true);
            }
            self.buffer.push(format!("mov X16, #1\n"));
            self.buffer.push(format!("svc #0x80\n"));
        } else {
            self.buffer.push(format!("_{}:\n", f_name));
            for stmt in stmts  {
                self.generate_stmt(stmt, false);
            }
        }
    }

    fn generate_stmt(&mut self, ele : &NodeStmt, is_main: bool) {
        match ele {
            crate::parsing::NodeStmt::Let { expr, ident } => {
                let identifier = cast!(&ident.token, Token::Indent);
                let variable = self.get_variable(&identifier);
                if let Some(variable) = variable {
                    panic!("{} already defined at line {}", identifier, variable.line_number);
                }

                self.parse_expr(expr);
                self.insert_ident(&identifier, ident.line);
            }
            crate::parsing::NodeStmt::Return { expr } => {
                self.parse_expr(expr);
                self.pop("X0");
                if !is_main {
                    self.buffer.push("RET\n".to_owned());
                }
            },
            crate::parsing::NodeStmt::Scope { scope } => {
                self.generate_scope(scope);
            },
            NodeStmt::If { expr, scope, chain } => {
                self.parse_expr(expr);
                self.pop("X1");
                let normal_label = self.create_label();
                let next_label = self.create_label();
                self.buffer.push(format!("cmp X1, 0\n"));
                self.buffer.push(format!("b.eq {}\n", next_label));
                self.generate_scope(scope);
                self.buffer.push(format!("MOV X2, 0\n"));
                self.buffer.push(format!("cmp X2, 0\n"));
                self.buffer.push(format!("b.eq {}\n", normal_label));
                self.buffer.push(format!("{}:\n", next_label));
                self.generate_node_else(&chain, &normal_label);
                self.buffer.push(format!("{}:\n", normal_label));
            },
            NodeStmt::ReAssign { expr, ident } => {
                let identifier = cast!(&ident.token, Token::Indent);
                let variable = self.get_variable(&identifier)
                    .expect(&format!("{} not declared but used in line {}", identifier, ident.line));
                // Let's mStackPos=10 and varStackPos=5
                // offset=5*16=80 we need to go upwards to access memory
                // Meaning we have to use -80 rather +80
                let offset = self.m_stack_pointer - variable.stack_location;
                let offset = offset * 16;
                self.parse_expr(expr);
                self.pop("X1");
                self.buffer.push(format!("STP X1, X2, [SP, #{}]\n", offset));
            },
        }

    }

    fn generate_node_else(&mut self, node_else: &Option<NodeElse>, normal_label: &str) {
        if let Some(node_else) = node_else  {
            match node_else {
                NodeElse::ElseIf { expr, scope, chain } => {
                    self.parse_expr(expr);
                    self.pop("X1");
                    let label = self.create_label();
                    self.buffer.push(format!("cmp X1, 0\n"));
                    self.buffer.push(format!("b.eq {}\n", label));
                    self.generate_scope(scope);
                    self.buffer.push(format!("MOV X2, 0\n"));
                    self.buffer.push(format!("cmp X2, 0\n"));
                    self.buffer.push(format!("b.eq {}\n", normal_label));
                    self.buffer.push(format!("{}:\n", label));
                    self.generate_node_else(chain, normal_label);
                },
                NodeElse::Else(scope) => { 
                    self.generate_scope(scope);
                },
            }
        }
    }


    fn generate_scope(&mut self, scope: &NodeScope) {
        let scope_stmts = &scope.0;
        self.begin_scope();
        for scope_stmt in scope_stmts.into_iter() {
            self.generate_stmt(scope_stmt, false);
        }
        self.end_scope();
    }
    
}

#[derive(Debug)]
struct Var {
    stack_location: usize,
    line_number: i32,
}

pub fn generate_code(node_root: NodeRoot) {
    let funcs = node_root.funcs;
    let output = File::create("out.s").expect("Failed to create file");
    let m_stack_pointer: usize = 0;
    let scope_ident : usize = 0;
    let scope_map: HashMap<usize, HashMap<String, Var>> = HashMap::new();
    let mut generator = Generator {
        buffer: vec![],
        scope_map,
        m_stack_pointer,
        output,
        scope_ident,
        label_index: 0,
    };

    generator.buffer.push(".global _start\n".to_owned());
    generator.buffer.push(".align 2\n".to_owned());

    for ele in funcs {
        generator.gen_func(&ele);
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

use std::process::Command;

use crate::{
    lexical::Token,
    parsing::{
        NodeElse, NodeFunc, NodeRoot, NodeScope, NodeStmt,
    },
};

use code_gen_structs::Generator;


impl Generator {
    fn create_label(&mut self) -> String {
        let label = format!("label{}", self.label_index);
        self.label_index += 1;
        label
    }
}


// Statement

impl Generator {
    fn gen_func(&mut self, func: &NodeFunc) {
        let f_name = &func.f_name;
        let stmts = &func.stmts;
        self.begin_func(f_name.to_string());
        tracing::debug!("Generating func {}", f_name);

        if f_name == "main" {
            self.buffer.push("_start:\n".to_owned());
            for stmt in stmts {
                debug!("main => {:?}", stmt);
                self.generate_stmt(stmt);
            }
            self.end_func();
        } else {
            self.buffer_push(&format!("_{}:", f_name));
            for stmt in stmts {
                self.generate_stmt(stmt);
            }
            self.end_func();
            self.buffer.push("RET\n".to_owned());
        }
    }

    fn generate_stmt(&mut self, ele: &NodeStmt) {
        match ele {
            crate::parsing::NodeStmt::Let { expr, ident } => {
                let identifier = cast!(&ident.token, Token::Indent);
                self.comment(&format!("Let stmt {}", identifier));
                let variable = self.get_variable(&identifier);
                if let Some(variable) = variable {
                    debug!("Generator {:#?}", self);
                    panic!(
                        "{} already defined at line {}",
                        identifier, variable.line_number
                    );
                }

                self.parse_expr(expr);
                self.insert_ident(&identifier, ident.line);
            }
            crate::parsing::NodeStmt::Return { expr } => {
                self.comment("Return stmt");
                self.parse_expr(expr);
                self.pop("X0");
                self.end_func();
                self.buffer_push(&format!("mov X16, #1"));
                self.buffer_push(&format!("svc #0x80"));
            }
            crate::parsing::NodeStmt::Scope { scope } => {
                self.generate_scope(scope);
            }
            NodeStmt::If { expr, scope, chain } => {
                self.comment("If condition");

                // Normal is Rest of the code
                // Next Label is Next condition like else if or else
                let normal_label = self.create_label();
                let next_label = self.create_label();

                self.parse_expr(expr);
                self.pop("X1");
                self.buffer_push(&format!("cmp X1, 0"));
                self.buffer_push(&format!("b.eq {}", next_label));
                self.comment("If scope generation start");
                self.generate_scope(scope);
                self.buffer_push(&format!("MOV X2, 0"));
                self.buffer_push(&format!("cmp X2, 0"));
                self.buffer_push(&format!("b.eq {}", normal_label));
                self.buffer_push(&format!("{}:", next_label));
                self.generate_node_else(&chain, &normal_label);
                self.comment("If condition finished");
                self.buffer_push(&format!("{}:", normal_label));
            }
            NodeStmt::ReAssign { expr, ident } => {
                let identifier = cast!(&ident.token, Token::Indent);
                let variable = self.get_variable(&identifier).expect(&format!(
                    "{} not declared but used in line {}",
                    identifier, ident.line
                ));
                // Let's mStackPos=10 and varStackPos=5
                // offset=5*16=80 we need to go upwards to access memory
                // Meaning we have to use -80 rather +80
                let offset = self.current_stack_pointer() - variable.stack_location;
                let offset = offset * 16;
                self.parse_expr(expr);
                self.pop("X1");
                self.buffer.push(format!("STP X1, X2, [SP, #{}]\n", offset));
            }
        }
    }

    fn generate_node_else(&mut self, node_else: &Option<NodeElse>, normal_label: &str) {
        if let Some(node_else) = node_else {
            match node_else {
                NodeElse::ElseIf { expr, scope, chain } => {
                    let pointer = self.current_stack_pointer();
                    self.comment(&format!("Else Compiler mStackPointer {}", pointer));
                    self.parse_expr(expr);
                    self.pop("X1");
                    let label = self.create_label();
                    self.buffer.push(format!("cmp X1, 0\n"));
                    self.buffer.push(format!("b.eq {}\n", label));
                    self.comment("Else If scope generation start");
                    self.generate_scope(scope);
                    self.buffer.push(format!("MOV X2, 0\n"));
                    self.buffer.push(format!("cmp X2, 0\n"));
                    self.buffer.push(format!("b.eq {}\n", normal_label));
                    self.buffer.push(format!("{}:\n", label));
                    self.generate_node_else(chain, normal_label);
                }
                NodeElse::Else(scope) => {
                    self.comment("Else scope generation start");
                    self.generate_scope(scope);
                }
            }
        }
    }

    fn generate_scope(&mut self, scope: &NodeScope) {
        let scope_stmts = &scope.0;
        self.begin_scope();
        let begin_stack_pointer = self.current_stack_pointer();
        for scope_stmt in scope_stmts.into_iter() {
            self.generate_stmt(scope_stmt);
        }
        let current_stack_pointer = self.current_stack_pointer();
        let difference = current_stack_pointer - begin_stack_pointer;
        self.clear_stack(difference);
        self.end_scope();
    }
}

pub fn generate_code(node_root: NodeRoot) {
    let funcs = node_root.funcs;
    let mut generator = Generator::new();

    generator.buffer_push(&format!(".global _start"));
    generator.buffer_push(&format!(".align 2"));

    for ele in funcs {
        generator.gen_func(&ele);
    }

    generator.finalize();
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
use tracing::debug;

use super::code_gen_structs;

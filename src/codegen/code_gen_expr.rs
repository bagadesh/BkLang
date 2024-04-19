use tracing::debug;

use crate::
    parsing::{
        NodeBiExpr, NodeExpr, NodeTermExpr,
    }
;

use super::code_gen_structs::Generator;


impl Generator {
    pub fn parse_expr(&mut self, expr: &NodeExpr) {
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
                self.push("X0");
            }
        }
    }

    fn parse_binary_expr(&mut self, binary_expr: &NodeBiExpr) {
        let lhs = &binary_expr.lhs;
        let rhs = &binary_expr.rhs;
        self.parse_expr(&lhs);
        let lhs_pos = self.current_stack_pointer();
        self.parse_expr(&rhs);
        let lhs_pos = self.current_stack_pointer() - lhs_pos;
        let lhs_pos = lhs_pos * 16;

        self.buffer
            .push(format!("LDP X1, X2, [SP, #{}]\n", lhs_pos));
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
            }
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

                let variable_location =  map_value.stack_location; 


                let m_stack = self.current_stack_pointer();
                let offset = m_stack - variable_location;
                let offset = offset * 16;

                self.buffer.push(format!("LDP X1, X2, [SP, #{}]\n", offset));
                self.push("X1");

                self.comment(&format!("Identifier {}, parse m_stack:{}, variable_location:{}, offset:{}, variable:{}", self.current_stack_pointer(), m_stack, variable_location, offset, value));

            }
            NodeTermExpr::Expression(expr) => {
                self.parse_expr(&expr);
            }
        }
    }
}

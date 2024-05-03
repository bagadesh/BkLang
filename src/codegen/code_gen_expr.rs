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
        self.pop("X2");
        self.pop("X1");

        //let lhs_pos = self.current_stack_pointer() - lhs_pos;

        //self.pop_with_offset("X1", lhs_pos);

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
            crate::parsing::NodeBiOp::Equality => {
                // X1 has LHS value
                // X2 has RHS values
                // X1 will have result
                // CSET will set X1 to 1 if previous operation has set ZERO flag
                // eq will return 1 if previous operation result is 0.
                self.buffer_push(&format!("SUBS X1, X1, X2"));
                self.buffer_push(&format!("CSET X1, eq"));
                self.push("X1");
                self.comment("Equality finsihed");
            }
            crate::parsing::NodeBiOp::OR => {
                // X1 (LHS) has 1 or 0
                // X2 (RHS) has 1 or 0
                //
                // MOV X0, #0  ; W0 = 10 (true)
                self.buffer_push(&format!("MOV X0, #0"));
                self.buffer_push(&format!("CMN X1, X2"));
                self.buffer_push(&format!("CSET X0, NE"));
                self.buffer_push(&format!("ORR X0, X0, X2"));

                self.push("X0");
                self.comment("OR finsihed");
            }
            crate::parsing::NodeBiOp::AND => {

                self.buffer_push(&format!("MOV X3, #0"));
                self.buffer_push(&format!("MOV X4, #0"));

                self.buffer_push(&format!("CMN X1, X2"));
                self.buffer_push(&format!("CSET X3, NE"));
                self.buffer_push(&format!("CMN X1, X1"));
                self.buffer_push(&format!("CSET X4, NE"));
                self.buffer_push(&format!("AND X3, X3, X4"));

                self.push("X3");
                self.comment("AND finsihed");
            }
            crate::parsing::NodeBiOp::GreaterThan => {
                self.buffer_push(&format!("CMP X1, X2"));
                self.buffer_push(&format!("CSET X1, gt"));
                self.push("X1");
                self.comment("Greater than finsihed");
            },
            crate::parsing::NodeBiOp::LessThan => {
                self.buffer_push(&format!("CMP X1, X2"));
                self.buffer_push(&format!("CSET X1, lt"));
                self.push("X1");
                self.comment("Less than finsihed");
            },
            crate::parsing::NodeBiOp::LessThanEqual => {
                self.buffer_push(&format!("CMP X1, X2"));
                self.buffer_push(&format!("CSET X1, le"));
                self.push("X1");
                self.comment("Less than Equal finsihed");
            },
            crate::parsing::NodeBiOp::GreaterThanEqual => {
                self.buffer_push(&format!("CMP X1, X2"));
                self.buffer_push(&format!("CSET X1, ge"));
                self.push("X1");
                self.comment("Greater than Equal finsihed");
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

            NodeTermExpr::BooleanLiteral(value) => {
                let value: i8 = if *value {
                    1
                } else {
                    0
                };
                self.buffer_push(&format!("MOV X1, {}", value));
                self.push("X1");
            }
        }
    }
}

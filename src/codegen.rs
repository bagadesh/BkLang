

mod code_generator;
mod code_gen_structs;
mod code_gen_expr;
mod code_gen_stack;
use crate::parsing::NodeRoot;

pub fn generate(node_root: NodeRoot) {
    code_generator::generate_code(node_root)
}

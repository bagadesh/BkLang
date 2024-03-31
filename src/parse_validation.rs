use core::panic;

use crate::parsing::NodeRoot;

 pub fn parse_validation(root: &NodeRoot) {
     let funcs = &root.funcs;
     let is_main_present = funcs.iter().any(|f| f.f_name == "main");
     if !is_main_present {
         tracing::error!("Main function is missing");
         panic!("Invalid Parsing");
     }
 }

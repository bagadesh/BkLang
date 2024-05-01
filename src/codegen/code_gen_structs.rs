use std::{collections::HashMap, fs::File, io::Write};

use tracing::debug;

#[derive(Debug)]
pub struct Var {
    pub stack_location: usize,
    pub line_number: i32,
}

#[derive(Debug)]
pub struct Generator {
    pub buffer: Vec<String>,
    pub output: File,
    pub label_index: usize,
    pub fn_scope_map: HashMap<String, LocalScopes>,
    pub m_func_name: String,
}

impl Generator {
    pub fn new() -> Generator {
        let output = File::create("out.s").expect("Failed to create file");
        let m_stack_pointer: usize = 0;
        let fn_scopes = HashMap::new();
        Generator {
            buffer: vec![],
            output,
            label_index: 0,
            fn_scope_map: fn_scopes,
            m_func_name: "".to_owned(),
        }
    }

    pub fn comment(&mut self, comment: &str) {
        self.buffer.push(format!(";{}\n", comment));
    }

    pub fn finalize(mut self) {
        let buf: String = self.buffer.into_iter().collect();
        let buf = buf.as_bytes();

        let _ = self.output.write_all(buf);
    }

    pub fn get_variable(&self, identifier: &String) -> Option<&Var> {
        let local_scopes = self.fn_scope_map.get(&self.m_func_name)?;
        local_scopes.get_variable(identifier)
    }

    pub fn begin_func(&mut self, c_func_name: String) {
        self.m_func_name = c_func_name;
    }

    pub fn end_func(&mut self) {
        let local_scope = self.fn_scope_map.get(&self.m_func_name);
        if let Some(local_scope) = local_scope {
            let ident_scope_count = local_scope.m_stack_pointer;
            // Because of 128-bit alignment in Aarch64 we are pushing dummy values
            // all the time when we add data to stack so we have use 16 than 8
            let ident_scope_size = ident_scope_count * 16;

            // Remove scope variables from Stack
            self.buffer_push(&format!("ADD SP, SP, #{}", ident_scope_size));
        }
    }

    pub fn insert_ident(&mut self, identifier: &String, line_number: i32) {
        let e = self.fn_scope_map.entry(self.m_func_name.to_string());
        let local_scopes = e.or_insert_with(|| LocalScopes::new());
        local_scopes.put_identifier(identifier, line_number, local_scopes.m_stack_pointer);
    }

    pub fn begin_scope(&self) {}

    pub fn end_scope(&self) {}

    pub fn buffer_push(&mut self, value: &str) {
        self.buffer.push(format!("{}\n", value));
    }

    pub fn increase_stack_pointer(&mut self) {
        let e = self.fn_scope_map.entry(self.m_func_name.to_string());
        let local_scopes = e.or_insert_with(|| LocalScopes::new());
        local_scopes.m_stack_pointer += 1;

        let pointer = local_scopes.m_stack_pointer;
        self.comment(&format!("Compiler mStackPointer {}", pointer));
    }

    pub fn decrease_stack_pointer(&mut self) {
        let e = self.fn_scope_map.entry(self.m_func_name.to_string());
        let local_scopes = e.or_insert_with(|| LocalScopes::new());
        local_scopes.m_stack_pointer -= 1;

        let pointer = local_scopes.m_stack_pointer;
        self.comment(&format!("Compiler mStackPointer {}", pointer));
    }

    pub fn current_stack_pointer(&self) -> usize {
        let local_scope = self.fn_scope_map.get(&self.m_func_name);
        if let Some(local_scope) = local_scope {
            return local_scope.m_stack_pointer;
        }
        return 0;
    }
}

#[derive(Debug)]
pub struct LocalScopes {
    pub scope: HashMap<usize, HashMap<String, Var>>,
    pub index: usize,
    pub m_stack_pointer: usize,
}

impl LocalScopes {
    fn new() -> LocalScopes {
        let scope_map: HashMap<usize, HashMap<String, Var>> = HashMap::new();
        LocalScopes {
            scope: scope_map,
            index: 0,
            m_stack_pointer: 0,
        }
    }

    fn get_variable(&self, identifier: &String) -> Option<&Var> {
        for i in (0..=self.index).rev() {
            if let Some(map) = self.scope.get(&i) {
                if let Some(var) = map.get(identifier) {
                    return Some(var);
                }
            }
        }
        None
    }

    fn put_identifier(&mut self, identifier: &String, line_number: i32, m_stack_pointer: usize) {
        let map = self
            .scope
            .entry(self.index)
            .or_insert_with(|| HashMap::new());
        map.insert(
            identifier.to_string(),
            Var {
                stack_location: m_stack_pointer,
                line_number,
            },
        );
    }
}


use super::code_gen_structs::Generator;


impl Generator {
    // Does both store and moving SP 
    pub fn push(&mut self, value: &str) {
        self.buffer.push(format!("STP {}, X9, [SP, #-16]!\n", value));
        self.increase_stack_pointer();
    }

    pub fn pop(&mut self, register: &str) {
        self.buffer.push(format!("LDP {}, X9, [SP], #16\n", register));
        self.decrease_stack_pointer();
    }

    pub fn clear_stack(&mut self, offset: usize) {
        let lhs_pos = offset * 16;
        self.buffer_push(&format!("ADD SP, SP, #{}", lhs_pos));
        for _ in 0..offset {
            self.decrease_stack_pointer();
        }
    }

    pub fn pop_with_offset(&mut self, register: &str, offset: usize) {
        let lhs_pos = offset * 16;
        self.buffer_push(&format!("LDP {}, X9, [SP, #{}]", register, lhs_pos));
        self.buffer_push(&format!("ADD SP, SP, #{}", lhs_pos));
        for _ in 0..offset {
            self.decrease_stack_pointer();
        }
    }
}

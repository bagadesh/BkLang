
use super::code_gen_structs::Generator;


impl Generator {
    // Does both store and moving SP 
    pub fn push(&mut self, value: &str) {
        self.buffer.push(format!("STP {}, X2, [SP, #-16]!\n", value));
        self.increase_stack_pointer();
    }

    pub fn pop(&mut self, register: &str) {
        self.buffer.push(format!("LDP {}, X2, [SP], #16\n", register));
        self.decrease_stack_pointer();
    }
}
